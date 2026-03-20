use crate::constants::{GLOBAL_LOCK_VERSION, SKILL_LOCK_FILENAME};
use crate::types::SkillLockFile;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub fn get_skill_lock_path() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_STATE_HOME") {
        PathBuf::from(xdg).join("x-skill").join(SKILL_LOCK_FILENAME)
    } else {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".agents")
            .join(SKILL_LOCK_FILENAME)
    }
}

pub async fn read_skill_lock() -> SkillLockFile {
    let path = get_skill_lock_path();
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => match serde_json::from_str::<SkillLockFile>(&content) {
            Ok(lock) if lock.version >= GLOBAL_LOCK_VERSION => lock,
            _ => create_empty_lock(),
        },
        Err(_) => create_empty_lock(),
    }
}

pub async fn write_skill_lock(lock: &SkillLockFile) -> anyhow::Result<()> {
    let path = get_skill_lock_path();
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let content = serde_json::to_string_pretty(lock)?;
    let tmp = path.with_extension("tmp");
    tokio::fs::write(&tmp, &content).await?;
    tokio::fs::rename(&tmp, &path).await?;
    Ok(())
}

fn create_empty_lock() -> SkillLockFile {
    SkillLockFile {
        version: GLOBAL_LOCK_VERSION,
        skills: BTreeMap::new(),
        dismissed: None,
        last_selected_agents: None,
    }
}
