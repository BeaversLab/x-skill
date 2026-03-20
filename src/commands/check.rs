use crate::http;
use crate::output;
use crate::skill_lock;
use colored::Colorize;

pub async fn run() -> anyhow::Result<()> {
    output::show_logo();
    println!();

    let lock = skill_lock::read_skill_lock().await;

    if lock.skills.is_empty() {
        println!("  {} No skills installed.", "!".yellow().bold());
        return Ok(());
    }

    println!(
        "  Checking {} skill(s) for updates...\n",
        lock.skills.len()
    );

    let mut updates_available = 0;
    let mut errors = 0;

    for (name, entry) in &lock.skills {
        // Skip entries without a hash or that are local/git
        if entry.skill_folder_hash.is_empty()
            || entry.source_type == "local"
            || entry.source_type == "git"
        {
            continue;
        }

        let source = &entry.source;
        match http::fetch_skill_folder_hash(source, entry.skill_path.as_deref()).await {
            Ok(Some(latest_hash)) => {
                if latest_hash != entry.skill_folder_hash {
                    updates_available += 1;
                    println!(
                        "  {} {} has updates available",
                        "↑".cyan().bold(),
                        name.bold()
                    );
                } else {
                    println!(
                        "  {} {} is up to date",
                        "✓".green(),
                        name.dimmed()
                    );
                }
            }
            Ok(None) => {
                println!(
                    "  {} {} could not determine version",
                    "?".yellow(),
                    name.dimmed()
                );
            }
            Err(e) => {
                errors += 1;
                eprintln!(
                    "  {} {} error: {}",
                    "✗".red(),
                    name,
                    e.to_string().red()
                );
            }
        }
    }

    println!();
    if updates_available > 0 {
        println!(
            "  {} {} update(s) available. Run {} to update.",
            "↑".cyan().bold(),
            updates_available,
            "x-skill update".bold()
        );
    } else if errors == 0 {
        println!("  {} All skills are up to date.", "✓".green().bold());
    }

    // Telemetry
    let mut params = std::collections::HashMap::new();
    params.insert("skillCount".into(), lock.skills.len().to_string());
    params.insert("updatesAvailable".into(), updates_available.to_string());
    crate::telemetry::track("check", params);

    Ok(())
}
