use crate::local_lock;
use crate::output;
use crate::types::AddOptions;
use colored::Colorize;

pub async fn run() -> anyhow::Result<()> {
    output::show_logo();
    println!();

    let cwd = std::env::current_dir()?;
    let lock = local_lock::read_local_lock(&cwd).await;

    if lock.skills.is_empty() {
        println!(
            "  {} No skills in project lock file.",
            "!".yellow().bold()
        );
        println!(
            "  Run {} first to create a lock file.",
            "x-skill add <source>".bold()
        );
        return Ok(());
    }

    println!(
        "  {} Installing {} skill(s) from lock file...\n",
        "→".dimmed(),
        lock.skills.len()
    );

    let mut success_count = 0;
    let mut fail_count = 0;

    for (name, entry) in &lock.skills {
        println!("  {} Installing {}...", "→".dimmed(), name.bold());

        let opts = AddOptions {
            yes: true,
            ..Default::default()
        };

        match crate::commands::add::run(&entry.source, &opts).await {
            Ok(_) => {
                success_count += 1;
            }
            Err(e) => {
                fail_count += 1;
                eprintln!("  {} {} failed: {}", "✗".red(), name, e);
            }
        }
    }

    println!();
    if fail_count == 0 {
        println!(
            "  {} Installed {} skill(s) from lock file.",
            "✓".green().bold(),
            success_count
        );
    } else {
        println!(
            "  {} {} succeeded, {} failed.",
            "!".yellow().bold(),
            success_count,
            fail_count
        );
    }

    Ok(())
}
