use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum XSkillError {
    #[error("git clone failed for {url}")]
    GitClone {
        url: String,
        #[source]
        source: git2::Error,
        is_timeout: bool,
        is_auth: bool,
    },

    #[error("path traversal detected in: {0}")]
    PathTraversal(String),

    #[error("invalid source: {0}")]
    InvalidSource(String),

    #[error("lock file corrupted: {}", path.display())]
    LockFileCorrupted { path: PathBuf },

    #[error("well-known index validation failed: {reason}")]
    WellKnownValidation { reason: String },

    #[error("skill not found: {0}")]
    SkillNotFound(String),

    #[error("agent not found: {0}")]
    AgentNotFound(String),

    #[error("installation failed for {skill} -> {agent}: {reason}")]
    InstallFailed {
        skill: String,
        agent: String,
        reason: String,
    },
}
