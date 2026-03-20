use crate::constants::{LOCAL_LOCK_FILENAME, LOCAL_LOCK_VERSION};
use crate::types::{LocalSkillLockEntry, LocalSkillLockFile};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub async fn read_local_lock(cwd: &Path) -> LocalSkillLockFile {
    let path = cwd.join(LOCAL_LOCK_FILENAME);
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => match serde_json::from_str::<LocalSkillLockFile>(&content) {
            Ok(lock) if lock.version >= 1 => lock,
            _ => create_empty_local_lock(),
        },
        Err(_) => create_empty_local_lock(),
    }
}

pub async fn write_local_lock(lock: &LocalSkillLockFile, cwd: &Path) -> anyhow::Result<()> {
    let path = cwd.join(LOCAL_LOCK_FILENAME);
    let content = serde_json::to_string_pretty(lock)?;
    let tmp = path.with_extension("tmp");
    tokio::fs::write(&tmp, &content).await?;
    tokio::fs::rename(&tmp, &path).await?;
    Ok(())
}

pub fn compute_skill_folder_hash(skill_dir: &Path) -> anyhow::Result<String> {
    let mut files = Vec::new();
    collect_files(skill_dir, skill_dir, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hasher = Sha256::new();
    for (rel_path, content) in &files {
        hasher.update(rel_path.as_bytes());
        hasher.update(content);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn collect_files(
    base: &Path,
    dir: &Path,
    files: &mut Vec<(String, Vec<u8>)>,
) -> anyhow::Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == ".git" || name_str == "node_modules" {
            continue;
        }
        if path.is_dir() {
            collect_files(base, &path, files)?;
        } else if path.is_file() {
            let rel = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let content = fs::read(&path)?;
            files.push((rel, content));
        }
    }
    Ok(())
}

pub fn add_skill_to_local_lock(
    lock: &mut LocalSkillLockFile,
    skill_name: &str,
    source: &str,
    source_type: &str,
    hash: &str,
) {
    lock.skills.insert(
        skill_name.to_string(),
        LocalSkillLockEntry {
            source: source.to_string(),
            source_type: source_type.to_string(),
            computed_hash: hash.to_string(),
        },
    );
}

fn create_empty_local_lock() -> LocalSkillLockFile {
    LocalSkillLockFile {
        version: LOCAL_LOCK_VERSION,
        skills: BTreeMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_compute_hash_deterministic() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("SKILL.md"), "---\nname: x\n---\n# X").unwrap();
        fs::write(tmp.path().join("extra.txt"), "data").unwrap();

        let h1 = compute_skill_folder_hash(tmp.path()).unwrap();
        let h2 = compute_skill_folder_hash(tmp.path()).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_compute_hash_changes_with_content() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("SKILL.md"), "version1").unwrap();
        let h1 = compute_skill_folder_hash(tmp.path()).unwrap();

        fs::write(tmp.path().join("SKILL.md"), "version2").unwrap();
        let h2 = compute_skill_folder_hash(tmp.path()).unwrap();
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_add_skill_to_local_lock() {
        let mut lock = create_empty_local_lock();
        add_skill_to_local_lock(&mut lock, "my-skill", "owner/repo", "github", "abc123");
        assert!(lock.skills.contains_key("my-skill"));
        assert_eq!(lock.skills["my-skill"].computed_hash, "abc123");
    }
}
