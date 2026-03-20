use crate::agents::build_agent_configs;
use crate::constants::SKILL_MD;
use crate::output;
use colored::Colorize;
use std::fs;

pub async fn run(json: bool, global: bool) -> anyhow::Result<()> {
    if !json {
        output::show_logo();
        println!();
    }

    let configs = build_agent_configs();
    let mut all_skills: Vec<SkillInfo> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for config in &configs {
        let skills_dir = if global {
            config.global_skills_dir.as_ref().cloned()
        } else {
            Some(std::path::PathBuf::from(config.skills_dir))
        };

        let Some(dir) = skills_dir else { continue };
        if !dir.exists() {
            continue;
        }

        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let skill_md = path.join(SKILL_MD);
            if !skill_md.exists() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            if seen.insert(name.clone()) {
                let description = crate::frontmatter::extract_frontmatter(
                    &fs::read_to_string(&skill_md).unwrap_or_default(),
                )
                .and_then(|(fm, _)| fm.description)
                .unwrap_or_default();

                all_skills.push(SkillInfo {
                    name,
                    description,
                    path: path.to_string_lossy().to_string(),
                    agent: config.display_name.to_string(),
                });
            }
        }
    }

    if json {
        let json_output = serde_json::to_string_pretty(&all_skills)?;
        println!("{json_output}");
        return Ok(());
    }

    if all_skills.is_empty() {
        println!(
            "  {} No skills installed{}.",
            "!".yellow().bold(),
            if global { " globally" } else { "" }
        );
        println!(
            "  Run {} to install skills.",
            "x-skill add <source>".bold()
        );
        return Ok(());
    }

    println!(
        "  {} skill(s) installed{}:\n",
        all_skills.len().to_string().bold(),
        if global { " globally" } else { "" }
    );

    for skill in &all_skills {
        println!("  {} {}", "•".green(), skill.name.bold());
        if !skill.description.is_empty() {
            println!("    {}", skill.description.dimmed());
        }
    }

    Ok(())
}

#[derive(serde::Serialize)]
struct SkillInfo {
    name: String,
    description: String,
    path: String,
    agent: String,
}
