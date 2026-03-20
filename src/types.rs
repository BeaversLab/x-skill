use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Supported agent types (43+).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentType {
    Amp,
    Antigravity,
    Augment,
    ClaudeCode,
    Openclaw,
    Cline,
    Codebuddy,
    Codex,
    CommandCode,
    Continue,
    Cortex,
    Crush,
    Cursor,
    Droid,
    GeminiCli,
    GithubCopilot,
    Goose,
    IflowCli,
    Junie,
    Kilo,
    KimiCli,
    KiroCli,
    Kode,
    Mcpjam,
    MistralVibe,
    Mux,
    Neovate,
    Opencode,
    Openhands,
    Pi,
    Qoder,
    QwenCode,
    Replit,
    Roo,
    Trae,
    TraeCn,
    Warp,
    Windsurf,
    Zencoder,
    Pochi,
    Adal,
    Universal,
}

/// How an agent's installation is detected on disk.
pub enum DetectStrategy {
    DirExists(PathBuf),
    AnyDirExists(Vec<PathBuf>),
    Never,
}

impl DetectStrategy {
    pub fn is_installed(&self) -> bool {
        match self {
            Self::DirExists(p) => p.exists(),
            Self::AnyDirExists(paths) => paths.iter().any(|p| p.exists()),
            Self::Never => false,
        }
    }
}

/// Static configuration for one agent platform.
#[allow(dead_code)]
pub struct AgentConfig {
    pub agent_type: AgentType,
    pub name: &'static str,
    pub display_name: &'static str,
    /// Relative dir under a project root (e.g. ".cursor/skills").
    pub skills_dir: &'static str,
    /// Absolute global skills directory, resolved at init time.
    pub global_skills_dir: Option<PathBuf>,
    pub detect: DetectStrategy,
    pub show_in_universal_list: bool,
}

/// A discovered skill from disk or remote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// The type of source a skill is being installed from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SourceType {
    Github,
    Gitlab,
    #[default]
    Git,
    Local,
    WellKnown,
}

/// Result of parsing a user-supplied source string.
#[derive(Debug, Clone, Default)]
pub struct ParsedSource {
    pub source_type: SourceType,
    pub url: String,
    pub subpath: Option<String>,
    pub local_path: Option<PathBuf>,
    pub ref_branch: Option<String>,
    pub skill_filter: Option<String>,
}

/// A skill fetched from a remote provider.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RemoteSkill {
    pub name: String,
    pub description: String,
    pub content: String,
    pub install_name: String,
    pub source_url: String,
    pub provider_id: String,
    pub source_identifier: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Install mode selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallMode {
    Symlink,
    Copy,
}

/// Result of installing a single skill for one agent.
#[derive(Debug)]
#[allow(dead_code)]
pub struct InstallResult {
    pub success: bool,
    pub path: PathBuf,
    pub canonical_path: Option<PathBuf>,
    pub mode: InstallMode,
    pub symlink_failed: bool,
    pub error: Option<String>,
}

/// Global lock file entry (version 3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLockEntry {
    pub source: String,
    pub source_type: String,
    pub source_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_path: Option<String>,
    pub skill_folder_hash: String,
    pub installed_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_name: Option<String>,
}

/// Global lock file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLockFile {
    pub version: u32,
    pub skills: std::collections::BTreeMap<String, SkillLockEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dismissed: Option<DismissedState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_selected_agents: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DismissedState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub find_skills_prompt: Option<bool>,
}

/// Project lock file entry (version 1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSkillLockEntry {
    pub source: String,
    pub source_type: String,
    pub computed_hash: String,
}

/// Project lock file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSkillLockFile {
    pub version: u32,
    pub skills: std::collections::BTreeMap<String, LocalSkillLockEntry>,
}

/// Provider match result.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProviderMatch {
    pub matches: bool,
    pub source_identifier: Option<String>,
}

/// Audit API response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResponse {
    #[serde(flatten)]
    pub data: HashMap<String, serde_json::Value>,
}

/// Options for the `add` command.
#[derive(Debug, Clone, Default)]
pub struct AddOptions {
    pub global: bool,
    pub yes: bool,
    pub list_only: bool,
    pub all: bool,
    pub agents: Vec<String>,
    pub skills: Vec<String>,
    pub full_depth: bool,
    pub copy: bool,
}

/// Options for skill discovery.
#[derive(Debug, Clone, Default)]
pub struct DiscoverOptions {
    pub include_internal: bool,
    pub full_depth: bool,
}
