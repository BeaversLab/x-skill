pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const UNIVERSAL_SKILLS_DIR: &str = ".agents/skills";

pub const SKILL_LOCK_FILENAME: &str = ".skill-lock.json";
pub const LOCAL_LOCK_FILENAME: &str = "skills-lock.json";
pub const SKILL_MD: &str = "SKILL.md";

pub const GLOBAL_LOCK_VERSION: u32 = 3;
pub const LOCAL_LOCK_VERSION: u32 = 1;

pub const SKILLS_API_URL: &str = "https://skills.sh";
pub const TELEMETRY_URL: &str = "https://skills.sh/api/telemetry";
pub const AUDIT_URL: &str = "https://skills.sh/api/audit";

pub const GITHUB_API_BASE: &str = "https://api.github.com";

pub const GIT_CLONE_TIMEOUT_SECS: u64 = 60;
pub const AUDIT_TIMEOUT_MS: u64 = 3000;

/// Directories to skip during recursive skill search.
pub const SKIP_DIRS: &[&str] = &["node_modules", ".git", "dist", "build", "__pycache__"];

/// Max recursion depth for full-depth skill discovery.
pub const MAX_DISCOVERY_DEPTH: usize = 5;
