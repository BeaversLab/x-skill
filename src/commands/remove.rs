use crate::agents::{build_agent_configs, get_agent_config};
use crate::error::XSkillError;
use crate::installer::sanitize_name;
use crate::output;
use crate::skill_lock;
use crate::t;
use console::style;
use std::fs;
use std::io::IsTerminal;

pub async fn run(
    skill: Option<&str>,
    agent: Option<&str>,
    global: bool,
    yes: bool,
) -> anyhow::Result<()> {
    output::show_logo();
    println!();

    let configs = build_agent_configs();

    // Read current lock
    let mut lock = skill_lock::read_skill_lock().await;

    if lock.skills.is_empty() {
        cliclack::log::warning(t!("no_skills_installed"))?;
        return Ok(());
    }

    // Determine which skills to remove
    let skill_names: Vec<String> = if let Some(name) = skill {
        let sanitized = sanitize_name(name);
        if !lock.skills.contains_key(&sanitized) {
            return Err(XSkillError::SkillNotFound(name.to_string()).into());
        }
        vec![sanitized]
    } else if yes {
        lock.skills.keys().cloned().collect()
    } else {
        // Interactive: list skills for selection
        println!("  {}:", t!("installed_skills"));
        for (i, name) in lock.skills.keys().enumerate() {
            println!("  {} {}", style(format!("{}.", i + 1)).dim(), name);
        }
        println!("\n  {}", t!("remove_specify_name"));
        return Ok(());
    };

    // Determine target agents
    let target_agents: Vec<_> = if let Some(agent_name) = agent {
        match get_agent_config(&configs, agent_name) {
            Some(cfg) => vec![cfg],
            None => return Err(XSkillError::AgentNotFound(agent_name.to_string()).into()),
        }
    } else {
        configs.iter().collect()
    };

    // Confirmation
    if !yes && std::io::stdin().is_terminal() {
        println!("\n  {}", t!("will_remove", "count" => skill_names.len()));
        for name in &skill_names {
            println!("  {} {}", style("•").red(), name);
        }
        println!();
        let should_continue = cliclack::confirm(t!("confirm_continue"))
            .initial_value(false)
            .interact()?;
        if !should_continue {
            cliclack::outro_cancel(t!("cancelled"))?;
            return Ok(());
        }
    }

    // Remove skills
    let mut removed = 0;
    for skill_name in &skill_names {
        for agent in &target_agents {
            let skill_dir = if global {
                agent.global_skills_dir.as_ref().map(|d| d.join(skill_name))
            } else {
                Some(std::path::PathBuf::from(agent.skills_dir).join(skill_name))
            };

            if let Some(dir) = skill_dir {
                if dir.exists() {
                    if let Err(e) = fs::remove_dir_all(&dir) {
                        cliclack::log::error(t!(
                            "remove_failed",
                            "skill" => skill_name,
                            "agent" => agent.display_name,
                            "error" => e
                        ))?;
                    } else {
                        removed += 1;
                    }
                }
            }
        }

        // Also remove from canonical dir
        let canonical = if global {
            dirs::home_dir()
                .unwrap_or_default()
                .join(".agents/skills")
                .join(skill_name)
        } else {
            std::path::PathBuf::from(".agents/skills").join(skill_name)
        };
        let _ = fs::remove_dir_all(&canonical);

        // Remove from lock
        lock.skills.remove(skill_name);
    }

    // Write updated lock
    let _ = skill_lock::write_skill_lock(&lock).await;

    cliclack::log::success(t!(
        "removed_success",
        "skills" => skill_names.len(),
        "locations" => removed
    ))?;

    // Telemetry
    if !crate::telemetry::is_telemetry_disabled() {
        let mut params = std::collections::HashMap::new();
        params.insert("skills".into(), skill_names.join(","));
        if global {
            params.insert("global".into(), "1".into());
        }
        crate::telemetry::track("remove", params);
    }

    Ok(())
}
