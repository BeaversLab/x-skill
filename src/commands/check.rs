use crate::http;
use crate::output;
use crate::skill_lock;
use crate::t;
use console::style;

pub async fn run() -> anyhow::Result<()> {
    output::show_logo();
    println!();

    let lock = skill_lock::read_skill_lock().await;

    if lock.skills.is_empty() {
        cliclack::log::warning(t!("no_skills_installed"))?;
        return Ok(());
    }

    println!(
        "  {}\n",
        t!("checking_updates", "count" => lock.skills.len())
    );

    let mut updates_available = 0;
    let mut errors = 0;

    for (name, entry) in &lock.skills {
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
                    cliclack::log::info(
                        t!("has_updates", "name" => style(name).bold()),
                    )?;
                } else {
                    cliclack::log::success(
                        t!("up_to_date", "name" => style(name).dim()),
                    )?;
                }
            }
            Ok(None) => {
                cliclack::log::warning(
                    t!("cannot_determine_version", "name" => style(name).dim()),
                )?;
            }
            Err(e) => {
                errors += 1;
                cliclack::log::error(t!("check_error", "name" => name, "error" => e))?;
            }
        }
    }

    println!();
    if updates_available > 0 {
        cliclack::log::info(t!(
            "updates_available",
            "count" => updates_available,
            "cmd" => style("x-skill update").bold()
        ))?;
    } else if errors == 0 {
        cliclack::log::success(t!("all_up_to_date"))?;
    }

    // Telemetry
    let mut params = std::collections::HashMap::new();
    params.insert("skillCount".into(), lock.skills.len().to_string());
    params.insert("updatesAvailable".into(), updates_available.to_string());
    crate::telemetry::track("check", params);

    Ok(())
}
