use crate::constants::GITHUB_API_BASE;
use serde::Deserialize;
use std::sync::LazyLock;

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(format!("x-skill/{}", crate::constants::VERSION))
        .build()
        .expect("failed to build HTTP client")
});

pub fn client() -> &'static reqwest::Client {
    &HTTP_CLIENT
}

pub fn get_github_token() -> Option<String> {
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            return Some(token);
        }
    }
    if let Ok(token) = std::env::var("GH_TOKEN") {
        if !token.is_empty() {
            return Some(token);
        }
    }
    std::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .filter(|s| !s.is_empty())
}

#[derive(Debug, Deserialize)]
struct TreeEntry {
    path: String,
    #[serde(rename = "type")]
    entry_type: String,
    sha: String,
}

#[derive(Debug, Deserialize)]
struct TreeResponse {
    sha: String,
    tree: Vec<TreeEntry>,
}

/// Fetch the SHA of a folder in a GitHub repo using the Trees API.
/// Tries `main` then `master`.
pub async fn fetch_skill_folder_hash(
    owner_repo: &str,
    skill_path: Option<&str>,
) -> anyhow::Result<Option<String>> {
    let token = get_github_token();

    for branch in &["main", "master"] {
        let url = format!(
            "{}/repos/{}/git/trees/{}?recursive=1",
            GITHUB_API_BASE, owner_repo, branch
        );

        let mut req = client().get(&url);
        if let Some(ref t) = token {
            req = req.header("Authorization", format!("Bearer {t}"));
        }

        let resp = match req.send().await {
            Ok(r) => r,
            Err(_) => continue,
        };

        if !resp.status().is_success() {
            continue;
        }

        let tree_resp: TreeResponse = match resp.json().await {
            Ok(t) => t,
            Err(_) => continue,
        };

        let folder_path = skill_path
            .map(|p| {
                p.replace('\\', "/")
                    .trim_end_matches("/SKILL.md")
                    .trim_end_matches('/')
                    .to_string()
            })
            .unwrap_or_default();

        if folder_path.is_empty() {
            return Ok(Some(tree_resp.sha));
        }

        for entry in &tree_resp.tree {
            if entry.entry_type == "tree" && entry.path == folder_path {
                return Ok(Some(entry.sha.clone()));
            }
        }
    }

    Ok(None)
}

/// Check if a GitHub repo is private.
#[allow(dead_code)]
pub async fn is_repo_private(owner: &str, repo: &str) -> Option<bool> {
    #[derive(Deserialize)]
    struct RepoInfo {
        private: Option<bool>,
    }

    let url = format!("{}/repos/{}/{}", GITHUB_API_BASE, owner, repo);
    let mut req = client().get(&url);
    if let Some(t) = get_github_token() {
        req = req.header("Authorization", format!("Bearer {t}"));
    }

    let resp = req.send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let info: RepoInfo = resp.json().await.ok()?;
    info.private
}
