use crate::constants::GIT_CLONE_TIMEOUT_SECS;
use crate::error::XSkillError;
use std::path::PathBuf;
use std::time::Duration;

pub async fn clone_repo(url: &str, ref_branch: Option<&str>) -> anyhow::Result<PathBuf> {
    let timeout = Duration::from_secs(GIT_CLONE_TIMEOUT_SECS);

    let result = tokio::time::timeout(timeout, clone_with_cli(url, ref_branch)).await;

    match result {
        Ok(Ok(path)) => Ok(path),
        Ok(Err(e)) => Err(e),
        Err(_) => anyhow::bail!(
            "git clone timed out after {}s. Check your network connection.",
            GIT_CLONE_TIMEOUT_SECS
        ),
    }
}

/// Clone using the system `git` CLI, falling back to libgit2 if `git` is not found.
async fn clone_with_cli(url: &str, ref_branch: Option<&str>) -> anyhow::Result<PathBuf> {
    let temp_dir = tempfile::Builder::new().prefix("x-skill-").tempdir()?;
    let dest = temp_dir.path().to_path_buf();

    let mut cmd = tokio::process::Command::new("git");
    cmd.args(["clone", "--depth", "1"]);
    if let Some(branch) = ref_branch {
        cmd.args(["--branch", branch]);
    }

    let auth_url = inject_token_if_available(url);
    cmd.arg(&auth_url);
    cmd.arg(&dest);

    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::piped());

    match cmd.output().await {
        Ok(output) if output.status.success() => {
            let _ = temp_dir.keep();
            Ok(dest)
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git clone failed: {}", stderr.trim());
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            drop(temp_dir);
            clone_with_libgit2(url, ref_branch).await
        }
        Err(e) => anyhow::bail!("failed to run git: {e}"),
    }
}

/// Inject a GitHub token into HTTPS GitHub/GitLab URLs for private repo access.
fn inject_token_if_available(url: &str) -> String {
    if let Some(rest) = url.strip_prefix("https://github.com/") {
        if let Some(token) = crate::http::get_github_token() {
            return format!("https://x-access-token:{token}@github.com/{rest}");
        }
    }
    url.to_string()
}

/// Fallback: clone using libgit2 (only reached when `git` CLI is not installed).
async fn clone_with_libgit2(url: &str, ref_branch: Option<&str>) -> anyhow::Result<PathBuf> {
    let url = url.to_string();
    let ref_branch = ref_branch.map(|s| s.to_string());

    let result =
        tokio::task::spawn_blocking(move || clone_repo_libgit2(&url, ref_branch.as_deref())).await;

    match result {
        Ok(Ok(path)) => Ok(path),
        Ok(Err(e)) => Err(e),
        Err(e) => anyhow::bail!("git clone task panicked: {e}"),
    }
}

fn clone_repo_libgit2(url: &str, ref_branch: Option<&str>) -> anyhow::Result<PathBuf> {
    let temp_dir = tempfile::Builder::new().prefix("x-skill-").tempdir()?;
    let dest = temp_dir.path().to_path_buf();

    let mut fo = git2::FetchOptions::new();
    fo.depth(1);

    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, allowed_types| {
        if allowed_types.contains(git2::CredentialType::SSH_KEY) {
            let user = username_from_url.unwrap_or("git");
            return git2::Cred::ssh_key_from_agent(user);
        }
        if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
            if let Some(token) = crate::http::get_github_token() {
                return git2::Cred::userpass_plaintext("x-access-token", &token);
            }
        }
        if allowed_types.contains(git2::CredentialType::DEFAULT) {
            return git2::Cred::default();
        }
        Err(git2::Error::from_str("no authentication method available"))
    });
    fo.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);
    if let Some(branch) = ref_branch {
        builder.branch(branch);
    }

    match builder.clone(url, &dest) {
        Ok(_) => {
            let _ = temp_dir.keep();
            Ok(dest)
        }
        Err(e) => {
            let msg = e.message().to_lowercase();
            let is_auth = msg.contains("authentication failed")
                || msg.contains("could not read username")
                || msg.contains("permission denied")
                || msg.contains("repository not found");
            let is_timeout = msg.contains("timed out");

            if is_auth {
                Err(XSkillError::GitClone {
                    url: url.to_string(),
                    source: e,
                    is_timeout: false,
                    is_auth: true,
                }
                .into())
            } else if is_timeout {
                Err(XSkillError::GitClone {
                    url: url.to_string(),
                    source: e,
                    is_timeout: true,
                    is_auth: false,
                }
                .into())
            } else {
                Err(XSkillError::GitClone {
                    url: url.to_string(),
                    source: e,
                    is_timeout: false,
                    is_auth: false,
                }
                .into())
            }
        }
    }
}

pub fn cleanup_temp_dir(dir: &std::path::Path) -> anyhow::Result<()> {
    let temp_root = std::env::temp_dir();
    let resolved = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    let resolved_root = std::fs::canonicalize(&temp_root).unwrap_or(temp_root);
    if !resolved.starts_with(&resolved_root) {
        anyhow::bail!(
            "refusing to delete directory outside temp: {}",
            dir.display()
        );
    }
    std::fs::remove_dir_all(dir)?;
    Ok(())
}
