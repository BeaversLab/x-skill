use crate::agents::build_agent_configs;
use crate::constants::SKILL_MD;
use crate::output;
use crate::skills;
use crate::types::DiscoverOptions;
use colored::Colorize;
use std::fs;
use std::path::Path;

pub async fn run() -> anyhow::Result<()> {
    output::show_logo();
    println!();
    println!("  {} Scanning node_modules for skills...", "→".dimmed());

    let cwd = std::env::current_dir()?;
    let node_modules = cwd.join("node_modules");

    if !node_modules.exists() {
        println!(
            "  {} No node_modules directory found.",
            "!".yellow().bold()
        );
        return Ok(());
    }

    let mut skill_sources = Vec::new();
    scan_for_skills(&node_modules, &mut skill_sources, 0)?;

    if skill_sources.is_empty() {
        println!(
            "  {} No skills found in node_modules.",
            "!".yellow().bold()
        );
        return Ok(());
    }

    println!(
        "  {} Found {} package(s) with skills",
        "✓".green(),
        skill_sources.len()
    );

    let configs = build_agent_configs();
    let mut total_installed = 0;

    for source_path in &skill_sources {
        let opts = DiscoverOptions::default();
        let discovered = skills::discover_skills(source_path, None, &opts)?;

        for skill in &discovered {
            for config in &configs {
                let result = crate::installer::install_skill_for_agent(
                    &skill.path,
                    &skill.name,
                    config,
                    false,
                    crate::types::InstallMode::Copy,
                )
                .await;
                if result.success {
                    total_installed += 1;
                }
            }
        }
    }

    println!(
        "\n  {} Synced {} skill(s) from node_modules.",
        "✓".green().bold(),
        total_installed
    );

    Ok(())
}

fn scan_for_skills(dir: &Path, results: &mut Vec<std::path::PathBuf>, depth: usize) -> anyhow::Result<()> {
    if depth > 2 {
        return Ok(());
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Check for scoped packages (@scope/package)
        if name_str.starts_with('@') {
            scan_for_skills(&path, results, depth + 1)?;
            continue;
        }

        // Check if this package has skills
        if path.join(SKILL_MD).exists() || path.join("skills").exists() {
            results.push(path);
        }
    }
    Ok(())
}
