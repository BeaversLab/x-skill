use crate::agents;
use crate::types::{AgentConfig, InstallMode, InstallResult};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn sanitize_name(name: &str) -> String {
    let sanitized: String = name
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = sanitized
        .trim_matches(|c: char| c == '.' || c == '-')
        .to_string();
    let truncated = if trimmed.len() > 255 {
        &trimmed[..255]
    } else {
        &trimmed
    };
    if truncated.is_empty() {
        "unnamed-skill".to_string()
    } else {
        truncated.to_string()
    }
}

pub fn is_path_safe(base: &Path, target: &Path) -> bool {
    let norm_base = normalize(base);
    let norm_target = normalize(target);
    norm_target.starts_with(&norm_base)
}

fn normalize(p: &Path) -> PathBuf {
    std::fs::canonicalize(p)
        .unwrap_or_else(|_| std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf()))
}

pub async fn install_skill_for_agent(
    skill_path: &Path,
    skill_name: &str,
    agent_config: &AgentConfig,
    global: bool,
    mode: InstallMode,
) -> InstallResult {
    let sanitized = sanitize_name(skill_name);

    let agent_dir = if global {
        agent_config
            .global_skills_dir
            .as_ref()
            .map(|d| d.join(&sanitized))
    } else {
        Some(PathBuf::from(agent_config.skills_dir).join(&sanitized))
    };

    let agent_dir = match agent_dir {
        Some(d) => d,
        None => {
            return InstallResult {
                success: false,
                path: PathBuf::new(),
                canonical_path: None,
                mode,
                symlink_failed: false,
                error: Some("no global skills dir for this agent".into()),
            }
        }
    };

    let is_universal = agents::is_universal_agent(agent_config);

    // Canonical dir is where the single authoritative copy lives
    let canonical_dir = if global {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".agents/skills")
            .join(&sanitized)
    } else {
        PathBuf::from(".agents/skills").join(&sanitized)
    };

    // For universal agents with global install, canonical IS the agent dir
    if is_universal && global {
        match copy_skill(skill_path, &canonical_dir) {
            Ok(_) => {
                return InstallResult {
                    success: true,
                    path: canonical_dir.clone(),
                    canonical_path: Some(canonical_dir),
                    mode: InstallMode::Copy,
                    symlink_failed: false,
                    error: None,
                }
            }
            Err(e) => {
                return InstallResult {
                    success: false,
                    path: canonical_dir,
                    canonical_path: None,
                    mode: InstallMode::Copy,
                    symlink_failed: false,
                    error: Some(e.to_string()),
                }
            }
        }
    }

    match mode {
        InstallMode::Copy => match copy_skill(skill_path, &agent_dir) {
            Ok(_) => InstallResult {
                success: true,
                path: agent_dir,
                canonical_path: None,
                mode: InstallMode::Copy,
                symlink_failed: false,
                error: None,
            },
            Err(e) => InstallResult {
                success: false,
                path: agent_dir,
                canonical_path: None,
                mode: InstallMode::Copy,
                symlink_failed: false,
                error: Some(e.to_string()),
            },
        },
        InstallMode::Symlink => {
            // First copy to canonical dir
            if let Err(e) = copy_skill(skill_path, &canonical_dir) {
                return InstallResult {
                    success: false,
                    path: agent_dir,
                    canonical_path: None,
                    mode: InstallMode::Symlink,
                    symlink_failed: false,
                    error: Some(e.to_string()),
                };
            }

            // Then create symlink from agent dir to canonical
            let link_target = resolve_parent_symlinks(&agent_dir);
            match create_symlink(&canonical_dir, &link_target) {
                Ok(_) => InstallResult {
                    success: true,
                    path: agent_dir,
                    canonical_path: Some(canonical_dir),
                    mode: InstallMode::Symlink,
                    symlink_failed: false,
                    error: None,
                },
                Err(_) => {
                    // Fallback to copy
                    match copy_skill(skill_path, &agent_dir) {
                        Ok(_) => InstallResult {
                            success: true,
                            path: agent_dir,
                            canonical_path: Some(canonical_dir),
                            mode: InstallMode::Symlink,
                            symlink_failed: true,
                            error: None,
                        },
                        Err(e) => InstallResult {
                            success: false,
                            path: agent_dir,
                            canonical_path: Some(canonical_dir),
                            mode: InstallMode::Symlink,
                            symlink_failed: true,
                            error: Some(e.to_string()),
                        },
                    }
                }
            }
        }
    }
}

fn copy_skill(src: &Path, dest: &Path) -> io::Result<()> {
    clean_and_create_dir(dest)?;
    copy_directory(src, dest)
}

fn clean_and_create_dir(dir: &Path) -> io::Result<()> {
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    fs::create_dir_all(dir)
}

fn copy_directory(src: &Path, dest: &Path) -> io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_directory(&src_path, &dest_path)?;
        } else if src_path.is_symlink() {
            // Handle potentially broken symlinks gracefully
            match fs::read_to_string(&src_path) {
                Ok(content) => {
                    fs::write(&dest_path, content)?;
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    eprintln!(
                        "Skipping broken symlink: {}",
                        src_path.display()
                    );
                }
                Err(e) => return Err(e),
            }
        } else {
            fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}

fn resolve_parent_symlinks(path: &Path) -> PathBuf {
    let abs = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let Some(parent) = abs.parent() else {
        return abs;
    };
    let base = abs.file_name().unwrap_or_default();
    match fs::canonicalize(parent) {
        Ok(real_parent) => real_parent.join(base),
        Err(_) => abs,
    }
}

fn create_symlink(target: &Path, link: &Path) -> io::Result<()> {
    if let Some(parent) = link.parent() {
        fs::create_dir_all(parent)?;
    }

    // Remove existing link/dir if present
    if link.exists() || link.is_symlink() {
        // Try to remove, handle ELOOP (circular symlink)
        if let Err(_) = fs::remove_dir_all(link) {
            let _ = fs::remove_file(link);
        }
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link)?;
    }

    #[cfg(windows)]
    {
        // On Windows, use junction for directory symlinks (matches TS behavior)
        std::os::windows::fs::symlink_dir(target, link)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name_basic() {
        assert_eq!(sanitize_name("My Skill"), "my-skill");
    }

    #[test]
    fn test_sanitize_name_special_chars() {
        assert_eq!(sanitize_name("skill@v2!"), "skill-v2");
        assert_eq!(sanitize_name("--hello--"), "hello");
    }

    #[test]
    fn test_sanitize_name_empty() {
        assert_eq!(sanitize_name("---"), "unnamed-skill");
    }

    #[test]
    fn test_sanitize_name_preserves_dots_underscores() {
        assert_eq!(sanitize_name("skill_v2.0"), "skill_v2.0");
    }

    #[test]
    fn test_is_path_safe() {
        let tmp = tempfile::tempdir().unwrap();
        // Create a real child dir so canonicalize works
        let child = tmp.path().join("child");
        fs::create_dir(&child).unwrap();
        let base = fs::canonicalize(tmp.path()).unwrap();
        assert!(is_path_safe(&base, &child));
    }

    #[test]
    fn test_copy_directory() {
        let src = tempfile::tempdir().unwrap();
        let dest = tempfile::tempdir().unwrap();
        let dest_target = dest.path().join("target");

        fs::write(src.path().join("file.txt"), "hello").unwrap();
        fs::create_dir(src.path().join("sub")).unwrap();
        fs::write(src.path().join("sub/nested.txt"), "world").unwrap();

        copy_directory(src.path(), &dest_target).unwrap();

        assert_eq!(
            fs::read_to_string(dest_target.join("file.txt")).unwrap(),
            "hello"
        );
        assert_eq!(
            fs::read_to_string(dest_target.join("sub/nested.txt")).unwrap(),
            "world"
        );
    }

    #[test]
    fn test_resolve_parent_symlinks() {
        let tmp = tempfile::tempdir().unwrap();
        let real = tmp.path().join("real");
        fs::create_dir(&real).unwrap();

        let result = resolve_parent_symlinks(&real.join("child"));
        // Parent is resolved, child is appended
        assert!(result.to_string_lossy().contains("child"));
    }
}
