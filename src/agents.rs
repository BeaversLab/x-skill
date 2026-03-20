use crate::constants::UNIVERSAL_SKILLS_DIR;
use crate::types::{AgentConfig, AgentType, DetectStrategy};
use std::path::PathBuf;

fn home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

fn config_home() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| home().join(".config"))
}

fn codex_home() -> PathBuf {
    std::env::var("CODEX_HOME")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".codex"))
}

fn claude_home() -> PathBuf {
    std::env::var("CLAUDE_CONFIG_DIR")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".claude"))
}

pub fn get_openclaw_global_skills_dir() -> PathBuf {
    let h = home();
    for name in &[".openclaw", ".clawdbot", ".moltbot"] {
        let dir = h.join(name);
        if dir.exists() {
            return dir.join("skills");
        }
    }
    h.join(".openclaw/skills")
}

pub fn build_agent_configs() -> Vec<AgentConfig> {
    let h = home();
    let cfg = config_home();
    let codex = codex_home();
    let claude = claude_home();
    let cwd = std::env::current_dir().unwrap_or_default();

    vec![
        AgentConfig {
            agent_type: AgentType::Amp,
            name: "amp",
            display_name: "Amp",
            skills_dir: ".agents/skills",
            global_skills_dir: Some(cfg.join("agents/skills")),
            detect: DetectStrategy::DirExists(cfg.join("amp")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Antigravity,
            name: "antigravity",
            display_name: "Antigravity",
            skills_dir: ".agent/skills",
            global_skills_dir: Some(h.join(".gemini/antigravity/skills")),
            detect: DetectStrategy::DirExists(h.join(".gemini/antigravity")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Augment,
            name: "augment",
            display_name: "Augment",
            skills_dir: ".augment/skills",
            global_skills_dir: Some(h.join(".augment/skills")),
            detect: DetectStrategy::DirExists(h.join(".augment")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::ClaudeCode,
            name: "claude-code",
            display_name: "Claude Code",
            skills_dir: ".claude/skills",
            global_skills_dir: Some(claude.join("skills")),
            detect: DetectStrategy::DirExists(claude.clone()),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Openclaw,
            name: "openclaw",
            display_name: "OpenClaw",
            skills_dir: "skills",
            global_skills_dir: Some(get_openclaw_global_skills_dir()),
            detect: DetectStrategy::AnyDirExists(vec![
                h.join(".openclaw"),
                h.join(".clawdbot"),
                h.join(".moltbot"),
            ]),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Cline,
            name: "cline",
            display_name: "Cline",
            skills_dir: ".agents/skills",
            global_skills_dir: Some(h.join(".agents/skills")),
            detect: DetectStrategy::DirExists(h.join(".cline")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Codebuddy,
            name: "codebuddy",
            display_name: "CodeBuddy",
            skills_dir: ".codebuddy/skills",
            global_skills_dir: Some(h.join(".codebuddy/skills")),
            detect: DetectStrategy::AnyDirExists(vec![
                cwd.join(".codebuddy"),
                h.join(".codebuddy"),
            ]),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Codex,
            name: "codex",
            display_name: "Codex",
            skills_dir: ".agents/skills",
            global_skills_dir: Some(codex.join("skills")),
            detect: DetectStrategy::AnyDirExists(vec![
                codex.clone(),
                PathBuf::from("/etc/codex"),
            ]),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::CommandCode,
            name: "command-code",
            display_name: "Command Code",
            skills_dir: ".commandcode/skills",
            global_skills_dir: Some(h.join(".commandcode/skills")),
            detect: DetectStrategy::DirExists(h.join(".commandcode")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Continue,
            name: "continue",
            display_name: "Continue",
            skills_dir: ".continue/skills",
            global_skills_dir: Some(h.join(".continue/skills")),
            detect: DetectStrategy::AnyDirExists(vec![
                cwd.join(".continue"),
                h.join(".continue"),
            ]),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Cortex,
            name: "cortex",
            display_name: "Cortex Code",
            skills_dir: ".cortex/skills",
            global_skills_dir: Some(h.join(".snowflake/cortex/skills")),
            detect: DetectStrategy::DirExists(h.join(".snowflake/cortex")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Crush,
            name: "crush",
            display_name: "Crush",
            skills_dir: ".crush/skills",
            global_skills_dir: Some(h.join(".config/crush/skills")),
            detect: DetectStrategy::DirExists(h.join(".config/crush")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Cursor,
            name: "cursor",
            display_name: "Cursor",
            skills_dir: ".agents/skills",
            global_skills_dir: Some(h.join(".cursor/skills")),
            detect: DetectStrategy::DirExists(h.join(".cursor")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Droid,
            name: "droid",
            display_name: "Droid",
            skills_dir: ".factory/skills",
            global_skills_dir: Some(h.join(".factory/skills")),
            detect: DetectStrategy::DirExists(h.join(".factory")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::GeminiCli,
            name: "gemini-cli",
            display_name: "Gemini CLI",
            skills_dir: ".agents/skills",
            global_skills_dir: Some(h.join(".gemini/skills")),
            detect: DetectStrategy::DirExists(h.join(".gemini")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::GithubCopilot,
            name: "github-copilot",
            display_name: "GitHub Copilot",
            skills_dir: ".agents/skills",
            global_skills_dir: Some(h.join(".copilot/skills")),
            detect: DetectStrategy::DirExists(h.join(".copilot")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Goose,
            name: "goose",
            display_name: "Goose",
            skills_dir: ".goose/skills",
            global_skills_dir: Some(cfg.join("goose/skills")),
            detect: DetectStrategy::DirExists(cfg.join("goose")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::IflowCli,
            name: "iflow-cli",
            display_name: "iFlow CLI",
            skills_dir: ".iflow/skills",
            global_skills_dir: Some(h.join(".iflow/skills")),
            detect: DetectStrategy::DirExists(h.join(".iflow")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Junie,
            name: "junie",
            display_name: "Junie",
            skills_dir: ".junie/skills",
            global_skills_dir: Some(h.join(".junie/skills")),
            detect: DetectStrategy::DirExists(h.join(".junie")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Kilo,
            name: "kilo",
            display_name: "Kilo Code",
            skills_dir: ".kilocode/skills",
            global_skills_dir: Some(h.join(".kilocode/skills")),
            detect: DetectStrategy::DirExists(h.join(".kilocode")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::KimiCli,
            name: "kimi-cli",
            display_name: "Kimi Code CLI",
            skills_dir: ".agents/skills",
            global_skills_dir: Some(h.join(".config/agents/skills")),
            detect: DetectStrategy::DirExists(h.join(".kimi")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::KiroCli,
            name: "kiro-cli",
            display_name: "Kiro CLI",
            skills_dir: ".kiro/skills",
            global_skills_dir: Some(h.join(".kiro/skills")),
            detect: DetectStrategy::DirExists(h.join(".kiro")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Kode,
            name: "kode",
            display_name: "Kode",
            skills_dir: ".kode/skills",
            global_skills_dir: Some(h.join(".kode/skills")),
            detect: DetectStrategy::DirExists(h.join(".kode")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Mcpjam,
            name: "mcpjam",
            display_name: "MCPJam",
            skills_dir: ".mcpjam/skills",
            global_skills_dir: Some(h.join(".mcpjam/skills")),
            detect: DetectStrategy::DirExists(h.join(".mcpjam")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::MistralVibe,
            name: "mistral-vibe",
            display_name: "Mistral Vibe",
            skills_dir: ".vibe/skills",
            global_skills_dir: Some(h.join(".vibe/skills")),
            detect: DetectStrategy::DirExists(h.join(".vibe")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Mux,
            name: "mux",
            display_name: "Mux",
            skills_dir: ".mux/skills",
            global_skills_dir: Some(h.join(".mux/skills")),
            detect: DetectStrategy::DirExists(h.join(".mux")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Neovate,
            name: "neovate",
            display_name: "Neovate",
            skills_dir: ".neovate/skills",
            global_skills_dir: Some(h.join(".neovate/skills")),
            detect: DetectStrategy::DirExists(h.join(".neovate")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Opencode,
            name: "opencode",
            display_name: "OpenCode",
            skills_dir: ".agents/skills",
            global_skills_dir: Some(cfg.join("opencode/skills")),
            detect: DetectStrategy::DirExists(cfg.join("opencode")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Openhands,
            name: "openhands",
            display_name: "OpenHands",
            skills_dir: ".openhands/skills",
            global_skills_dir: Some(h.join(".openhands/skills")),
            detect: DetectStrategy::DirExists(h.join(".openhands")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Pi,
            name: "pi",
            display_name: "Pi",
            skills_dir: ".pi/skills",
            global_skills_dir: Some(h.join(".pi/agent/skills")),
            detect: DetectStrategy::DirExists(h.join(".pi/agent")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Qoder,
            name: "qoder",
            display_name: "Qoder",
            skills_dir: ".qoder/skills",
            global_skills_dir: Some(h.join(".qoder/skills")),
            detect: DetectStrategy::DirExists(h.join(".qoder")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::QwenCode,
            name: "qwen-code",
            display_name: "Qwen Code",
            skills_dir: ".qwen/skills",
            global_skills_dir: Some(h.join(".qwen/skills")),
            detect: DetectStrategy::DirExists(h.join(".qwen")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Replit,
            name: "replit",
            display_name: "Replit",
            skills_dir: ".agents/skills",
            global_skills_dir: Some(cfg.join("agents/skills")),
            detect: DetectStrategy::DirExists(cwd.join(".replit")),
            show_in_universal_list: false,
        },
        AgentConfig {
            agent_type: AgentType::Roo,
            name: "roo",
            display_name: "Roo Code",
            skills_dir: ".roo/skills",
            global_skills_dir: Some(h.join(".roo/skills")),
            detect: DetectStrategy::DirExists(h.join(".roo")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Trae,
            name: "trae",
            display_name: "Trae",
            skills_dir: ".trae/skills",
            global_skills_dir: Some(h.join(".trae/skills")),
            detect: DetectStrategy::DirExists(h.join(".trae")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::TraeCn,
            name: "trae-cn",
            display_name: "Trae CN",
            skills_dir: ".trae/skills",
            global_skills_dir: Some(h.join(".trae-cn/skills")),
            detect: DetectStrategy::DirExists(h.join(".trae-cn")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Warp,
            name: "warp",
            display_name: "Warp",
            skills_dir: ".agents/skills",
            global_skills_dir: Some(h.join(".agents/skills")),
            detect: DetectStrategy::DirExists(h.join(".warp")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Windsurf,
            name: "windsurf",
            display_name: "Windsurf",
            skills_dir: ".windsurf/skills",
            global_skills_dir: Some(h.join(".codeium/windsurf/skills")),
            detect: DetectStrategy::DirExists(h.join(".codeium/windsurf")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Zencoder,
            name: "zencoder",
            display_name: "Zencoder",
            skills_dir: ".zencoder/skills",
            global_skills_dir: Some(h.join(".zencoder/skills")),
            detect: DetectStrategy::DirExists(h.join(".zencoder")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Pochi,
            name: "pochi",
            display_name: "Pochi",
            skills_dir: ".pochi/skills",
            global_skills_dir: Some(h.join(".pochi/skills")),
            detect: DetectStrategy::DirExists(h.join(".pochi")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Adal,
            name: "adal",
            display_name: "AdaL",
            skills_dir: ".adal/skills",
            global_skills_dir: Some(h.join(".adal/skills")),
            detect: DetectStrategy::DirExists(h.join(".adal")),
            show_in_universal_list: true,
        },
        AgentConfig {
            agent_type: AgentType::Universal,
            name: "universal",
            display_name: "Universal",
            skills_dir: ".agents/skills",
            global_skills_dir: Some(cfg.join("agents/skills")),
            detect: DetectStrategy::Never,
            show_in_universal_list: false,
        },
    ]
}

pub fn detect_installed_agents(configs: &[AgentConfig]) -> Vec<&AgentConfig> {
    configs.iter().filter(|c| c.detect.is_installed()).collect()
}

pub fn get_universal_agents(configs: &[AgentConfig]) -> Vec<&AgentConfig> {
    configs
        .iter()
        .filter(|c| is_universal_agent(c) && c.show_in_universal_list)
        .collect()
}

pub fn get_non_universal_agents(configs: &[AgentConfig]) -> Vec<&AgentConfig> {
    configs.iter().filter(|c| !is_universal_agent(c)).collect()
}

pub fn is_universal_agent(config: &AgentConfig) -> bool {
    config.skills_dir == UNIVERSAL_SKILLS_DIR
}

pub fn get_agent_config<'a>(configs: &'a [AgentConfig], name: &str) -> Option<&'a AgentConfig> {
    configs.iter().find(|c| c.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builds_all_agents() {
        let configs = build_agent_configs();
        assert_eq!(configs.len(), 42);
    }

    #[test]
    fn test_universal_agents_filtered() {
        let configs = build_agent_configs();
        let universal = get_universal_agents(&configs);
        for c in &universal {
            assert_eq!(c.skills_dir, UNIVERSAL_SKILLS_DIR);
            assert!(c.show_in_universal_list);
        }
    }

    #[test]
    fn test_non_universal_agents() {
        let configs = build_agent_configs();
        let non_universal = get_non_universal_agents(&configs);
        for c in &non_universal {
            assert_ne!(c.skills_dir, UNIVERSAL_SKILLS_DIR);
        }
    }

    #[test]
    fn test_get_agent_config_by_name() {
        let configs = build_agent_configs();
        let cursor = get_agent_config(&configs, "cursor").unwrap();
        assert_eq!(cursor.display_name, "Cursor");
        assert_eq!(cursor.skills_dir, ".agents/skills");
    }

    #[test]
    fn test_universal_agent_is_never_detected() {
        let configs = build_agent_configs();
        let universal = get_agent_config(&configs, "universal").unwrap();
        assert!(!universal.detect.is_installed());
        assert!(!universal.show_in_universal_list);
    }
}
