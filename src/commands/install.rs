use crate::local_lock;
use crate::output;
use crate::t;
use crate::types::AddOptions;
use console::style;

pub async fn run() -> anyhow::Result<()> {
    output::show_logo();
    println!();

    let cwd = std::env::current_dir()?;
    let lock = local_lock::read_local_lock(&cwd).await;

    if lock.skills.is_empty() {
        cliclack::log::warning(
            t!("no_lock_file", "cmd" => style("x-skill add <source>").bold()),
        )?;
        return Ok(());
    }

    println!(
        "  {} {}\n",
        style("→").dim(),
        t!("installing_from_lock", "count" => lock.skills.len())
    );

    let mut success_count = 0;
    let mut fail_count = 0;

    for (name, entry) in &lock.skills {
        let spinner = cliclack::spinner();
        spinner.start(t!("skill_installing", "name" => style(name).bold()));

        let opts = AddOptions {
            yes: true,
            ..Default::default()
        };

        match crate::commands::add::run(&entry.source, &opts).await {
            Ok(_) => {
                success_count += 1;
                spinner.stop(t!("skill_installed", "name" => name));
            }
            Err(e) => {
                fail_count += 1;
                spinner.error(t!("skill_install_failed", "name" => name, "error" => e));
            }
        }
    }

    println!();
    if fail_count == 0 {
        cliclack::log::success(t!("installed_from_lock", "count" => success_count))?;
    } else {
        cliclack::log::warning(
            t!("install_partial", "success" => success_count, "fail" => fail_count),
        )?;
    }

    Ok(())
}
