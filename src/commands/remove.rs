use crate::agents::{build_agent_configs, get_agent_config};
use crate::installer::sanitize_name;
use crate::output;
use crate::skill_lock;
use colored::Colorize;
use std::fs;

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
        println!("  {} No skills installed.", "!".yellow().bold());
        return Ok(());
    }

    // Determine which skills to remove
    let skill_names: Vec<String> = if let Some(name) = skill {
        let sanitized = sanitize_name(name);
        if !lock.skills.contains_key(&sanitized) {
            anyhow::bail!("skill '{}' not found in lock file", name);
        }
        vec![sanitized]
    } else if yes {
        lock.skills.keys().cloned().collect()
    } else {
        // Interactive: list skills for selection
        println!("  Installed skills:");
        for (i, name) in lock.skills.keys().enumerate() {
            println!("  {} {}", format!("{}.", i + 1).dimmed(), name);
        }
        println!("\n  Specify skill name with: x-skill remove <name>");
        return Ok(());
    };

    // Determine target agents
    let target_agents: Vec<_> = if let Some(agent_name) = agent {
        match get_agent_config(&configs, agent_name) {
            Some(cfg) => vec![cfg],
            None => anyhow::bail!("unknown agent: {}", agent_name),
        }
    } else {
        configs.iter().collect()
    };

    // Confirmation
    if !yes && atty::is(atty::Stream::Stdin) {
        println!("\n  Will remove {} skill(s):", skill_names.len());
        for name in &skill_names {
            println!("  {} {}", "•".red(), name);
        }
        print!("\n  Continue? [y/N] ");
        std::io::Write::flush(&mut std::io::stdout()).ok();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("  Cancelled.");
            return Ok(());
        }
    }

    // Remove skills
    let mut _removed = 0;
    for skill_name in &skill_names {
        for agent in &target_agents {
            let skill_dir = if global {
                agent
                    .global_skills_dir
                    .as_ref()
                    .map(|d| d.join(skill_name))
            } else {
                Some(std::path::PathBuf::from(agent.skills_dir).join(skill_name))
            };

            if let Some(dir) = skill_dir {
                if dir.exists() {
                    if let Err(e) = fs::remove_dir_all(&dir) {
                        eprintln!(
                            "  {} Failed to remove {} for {}: {}",
                            "✗".red(),
                            skill_name,
                            agent.display_name,
                            e
                        );
                    } else {
                        _removed += 1;
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

    println!(
        "\n  {} Removed {} skill(s).",
        "✓".green().bold(),
        skill_names.len()
    );

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
