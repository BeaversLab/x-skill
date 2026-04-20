mod agents;
mod cli;
mod commands;
mod config;
mod constants;
mod error;
mod frontmatter;
mod git;
mod http;
mod i18n;
mod installer;
mod local_lock;
mod output;
mod plugin_manifest;
mod prompts;
mod providers;
mod skill_lock;
mod skills;
mod source_parser;
mod telemetry;
mod types;

use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    config::load_dotenv();
    config::ensure_language();
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add {
            source,
            global,
            yes,
            list,
            all,
            agent,
            skill,
            full_depth,
            copy,
        }) => {
            let opts = types::AddOptions {
                global,
                yes,
                list_only: list,
                all,
                agents: agent,
                skills: skill,
                full_depth,
                copy,
            };
            commands::add::run(&source, &opts).await?;
        }
        Some(Commands::Remove {
            skill,
            agent,
            global,
            yes,
        }) => {
            commands::remove::run(skill.as_deref(), agent.as_deref(), global, yes).await?;
        }
        Some(Commands::List { json, global }) => {
            commands::list::run(json, global).await?;
        }
        Some(Commands::Find { query }) => {
            commands::find::run(query.as_deref()).await?;
        }
        Some(Commands::Check) => {
            commands::check::run().await?;
        }
        Some(Commands::Update) => {
            commands::update::run().await?;
        }
        Some(Commands::Config) => {
            commands::config::run()?;
        }
        Some(Commands::Init { name }) => {
            commands::init::run(name.as_deref())?;
        }
        Some(Commands::ExperimentalSync) => {
            commands::sync::run().await?;
        }
        Some(Commands::ExperimentalInstall) => {
            commands::install::run().await?;
        }
        None => {
            output::show_banner();
        }
    }

    Ok(())
}
