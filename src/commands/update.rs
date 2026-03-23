use crate::commands::add;
use crate::http;
use crate::output;
use crate::skill_lock;
use crate::t;
use crate::types::AddOptions;
use console::style;

pub async fn run() -> anyhow::Result<()> {
    output::show_logo();
    println!();

    let lock = skill_lock::read_skill_lock().await;

    if lock.skills.is_empty() {
        cliclack::log::warning(t!("no_skills_installed"))?;
        return Ok(());
    }

    let spinner = cliclack::spinner();
    spinner.start(t!("checking_updates", "count" => lock.skills.len()));

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
        spinner.stop(t!("all_up_to_date"));
        return Ok(());
    }

    spinner.stop(t!("skills_to_update", "count" => to_update.len()));

    let mut success_count = 0;
    let mut fail_count = 0;

    for (name, source_url, _skill_path) in &to_update {
        let update_spinner = cliclack::spinner();
        update_spinner.start(t!("updating", "name" => style(name).bold()));

        let install_url = source_url.clone();

        let opts = AddOptions {
            global: true,
            yes: true,
            ..Default::default()
        };

        match add::run(&install_url, &opts).await {
            Ok(_) => {
                success_count += 1;
                update_spinner.stop(t!("updated", "name" => name));
            }
            Err(e) => {
                fail_count += 1;
                update_spinner.error(t!("update_failed", "name" => name, "error" => e));
            }
        }
    }

    println!();
    if fail_count == 0 {
        cliclack::log::success(
            t!("update_summary", "success" => success_count, "fail" => fail_count),
        )?;
    } else {
        cliclack::log::warning(
            t!("update_summary", "success" => success_count, "fail" => fail_count),
        )?;
    }

    // Telemetry
    let mut params = std::collections::HashMap::new();
    params.insert("skillCount".into(), to_update.len().to_string());
    params.insert("successCount".into(), success_count.to_string());
    params.insert("failCount".into(), fail_count.to_string());
    crate::telemetry::track("update", params);

    Ok(())
}
