use crate::commands::add;
use crate::http;
use crate::output;
use crate::skill_lock;
use crate::types::AddOptions;
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

    let mut to_update = Vec::new();

    for (name, entry) in &lock.skills {
        if entry.skill_folder_hash.is_empty()
            || entry.source_type == "local"
            || entry.source_type == "git"
        {
            continue;
        }

        let source = &entry.source;
        match http::fetch_skill_folder_hash(source, entry.skill_path.as_deref()).await {
            Ok(Some(latest_hash)) if latest_hash != entry.skill_folder_hash => {
                to_update.push((name.clone(), entry.source_url.clone(), entry.skill_path.clone()));
            }
            _ => {}
        }
    }

    if to_update.is_empty() {
        println!("  {} All skills are up to date.", "✓".green().bold());
        return Ok(());
    }

    println!(
        "  {} {} skill(s) to update:\n",
        "↑".cyan().bold(),
        to_update.len()
    );

    let mut success_count = 0;
    let mut fail_count = 0;

    for (name, source_url, _skill_path) in &to_update {
        println!("  {} Updating {}...", "→".dimmed(), name.bold());

        // Build the install URL from source_url
        let install_url = source_url.clone();

        let opts = AddOptions {
            global: true,
            yes: true,
            ..Default::default()
        };

        match add::run(&install_url, &opts).await {
            Ok(_) => {
                success_count += 1;
                println!("  {} {} updated", "✓".green(), name);
            }
            Err(e) => {
                fail_count += 1;
                eprintln!("  {} {} failed: {}", "✗".red(), name, e);
            }
        }
    }

    println!();
    println!(
        "  {} {} updated, {} failed.",
        if fail_count == 0 {
            "✓".green().bold()
        } else {
            "!".yellow().bold()
        },
        success_count,
        fail_count,
    );

    // Telemetry
    let mut params = std::collections::HashMap::new();
    params.insert("skillCount".into(), to_update.len().to_string());
    params.insert("successCount".into(), success_count.to_string());
    params.insert("failCount".into(), fail_count.to_string());
    crate::telemetry::track("update", params);

    Ok(())
}
