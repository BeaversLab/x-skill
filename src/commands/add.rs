use crate::agents::{self, build_agent_configs, detect_installed_agents, get_universal_agents};
use crate::error::XSkillError;
use crate::git;
use crate::installer::{self, sanitize_name};
use crate::local_lock;
use crate::output;
use crate::prompts::search_multiselect::{MultiSelectOptions, SearchItem};
use crate::providers::registry;
use crate::skill_lock;
use crate::skills::{self, filter_skills};
use crate::source_parser::{self, get_owner_repo};
use crate::telemetry;
use crate::types::{
    AddOptions, AgentConfig, DiscoverOptions, InstallMode, Skill, SkillLockEntry, SourceType,
};
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::io::Write;

pub async fn run(source: &str, opts: &AddOptions) -> anyhow::Result<()> {
    output::show_logo();
    println!();

    let parsed = source_parser::parse_source(source);

    // Handle well-known skills via provider registry
    if parsed.source_type == SourceType::WellKnown {
        if let Some(provider) = registry::find_provider(&parsed.url) {
            return handle_provider_skills(provider, &parsed.url, source, opts).await;
        }
        anyhow::bail!("no provider matched URL: {}", parsed.url);
    }

    // Determine skills directory
    let (skills_dir, _temp_dir) = match parsed.source_type {
        SourceType::Local => {
            let local_path = parsed
                .local_path
                .as_ref()
                .ok_or_else(|| XSkillError::InvalidSource(source.to_string()))?;
            if !local_path.exists() {
                anyhow::bail!("local path does not exist: {}", local_path.display());
            }
            (local_path.clone(), None)
        }
        _ => {
            println!("  {} {}", "Cloning".dimmed(), parsed.url.dimmed());
            let temp = git::clone_repo(&parsed.url, parsed.ref_branch.as_deref()).await?;
            let dir = temp.clone();
            (dir, Some(temp))
        }
    };

    // Discover skills
    let discover_opts = DiscoverOptions {
        include_internal: false,
        full_depth: opts.full_depth,
    };
    let mut discovered =
        skills::discover_skills(&skills_dir, parsed.subpath.as_deref(), &discover_opts)?;

    if discovered.is_empty() {
        println!("  {} No skills found in source.", "!".yellow().bold());
        return Ok(());
    }

    // Apply skill_filter from @skill syntax
    if let Some(ref filter) = parsed.skill_filter {
        discovered = discovered
            .into_iter()
            .filter(|s| s.name.eq_ignore_ascii_case(filter))
            .collect();
        if discovered.is_empty() {
            return Err(XSkillError::SkillNotFound(filter.clone()).into());
        }
    }

    // List-only mode
    if opts.list_only {
        println!("  {} skills found:\n", discovered.len());
        for s in &discovered {
            println!("  {} {}", "•".dimmed(), s.name.bold());
            if !s.description.is_empty() {
                println!("    {}", s.description.dimmed());
            }
        }
        return Ok(());
    }

    // Skill selection
    let selected_skills = select_skills(&discovered, opts)?;
    if selected_skills.is_empty() {
        println!("  No skills selected.");
        return Ok(());
    }

    // Start parallel audit
    let skill_slugs: Vec<String> = selected_skills.iter().map(|s| s.name.clone()).collect();
    let owner_repo = get_owner_repo(&parsed);
    let audit_handle = if owner_repo.is_some() && parsed.source_type != SourceType::Local {
        let or = owner_repo.clone().unwrap();
        let slugs = skill_slugs.clone();
        Some(tokio::spawn(
            async move { telemetry::fetch_audit_data(&or, &slugs).await },
        ))
    } else {
        None
    };

    // Agent selection
    let all_configs = build_agent_configs();
    let selected_agents = select_agents(&all_configs, opts)?;
    if selected_agents.is_empty() {
        println!("  No agents selected.");
        return Ok(());
    }

    // Determine install mode
    let unique_dirs: HashSet<_> = selected_agents.iter().map(|a| a.skills_dir).collect();
    let mode = if opts.copy {
        InstallMode::Copy
    } else if unique_dirs.len() > 1 && !opts.yes {
        InstallMode::Symlink
    } else {
        InstallMode::Copy
    };

    // Await audit result
    if let Some(handle) = audit_handle {
        let _audit_result = handle.await.ok().flatten();
    }

    // Summary
    println!(
        "\n  Installing {} skill(s) for {} agent(s):\n",
        selected_skills.len().to_string().bold(),
        selected_agents.len().to_string().bold(),
    );
    for s in &selected_skills {
        println!("  {} {}", "•".green(), s.name);
    }
    println!();
    for a in &selected_agents {
        println!("  {} {}", "→".dimmed(), a.display_name);
    }
    println!();

    // Confirmation
    if !opts.yes && atty::is(atty::Stream::Stdin) {
        print!("  Continue? [Y/n] ");
        std::io::stdout().flush().ok();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().eq_ignore_ascii_case("n") {
            println!("  Cancelled.");
            return Ok(());
        }
    }

    // Install loop
    let mut success_count = 0;
    let mut fail_count = 0;

    for skill in &selected_skills {
        for agent in &selected_agents {
            let result = installer::install_skill_for_agent(
                &skill.path,
                &skill.name,
                agent,
                opts.global,
                mode,
            )
            .await;

            if result.success {
                success_count += 1;
                let mode_label = match result.mode {
                    InstallMode::Symlink => " (symlink)",
                    InstallMode::Copy => "",
                };
                println!(
                    "  {} {} → {} [{}{}]",
                    "✓".green(),
                    skill.name,
                    agent.display_name.dimmed(),
                    result.path.display(),
                    mode_label,
                );
            } else {
                fail_count += 1;
                let err = result.error.unwrap_or_default();
                eprintln!(
                    "  {} {} → {}: {}",
                    "✗".red(),
                    skill.name,
                    agent.display_name,
                    err.red()
                );
            }

            if result.symlink_failed {
                eprintln!(
                    "  {} symlink failed, fell back to copy for {}",
                    "⚠".yellow(),
                    agent.display_name
                );
            }
        }
    }

    println!();
    if fail_count == 0 {
        println!(
            "  {} Installed {} skill(s) successfully.",
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

    // Update global lock
    if parsed.source_type != SourceType::Local {
        let mut lock = skill_lock::read_skill_lock().await;
        let or = owner_repo.unwrap_or_else(|| source.to_string());
        let now = chrono_like_now();
        for skill in &selected_skills {
            lock.skills.insert(
                sanitize_name(&skill.name),
                SkillLockEntry {
                    source: or.clone(),
                    source_type: format!("{:?}", parsed.source_type).to_lowercase(),
                    source_url: parsed.url.clone(),
                    skill_path: parsed.subpath.clone(),
                    skill_folder_hash: String::new(),
                    installed_at: now.clone(),
                    updated_at: now.clone(),
                    plugin_name: skill.plugin_name.clone(),
                },
            );
        }
        let _ = skill_lock::write_skill_lock(&lock).await;
    }

    // Update project lock
    if !opts.global {
        let cwd = std::env::current_dir().unwrap_or_default();
        let mut local_lock = local_lock::read_local_lock(&cwd).await;
        let source_type_str = format!("{:?}", parsed.source_type).to_lowercase();
        for skill in &selected_skills {
            let hash = local_lock::compute_skill_folder_hash(&skill.path).unwrap_or_default();
            local_lock::add_skill_to_local_lock(
                &mut local_lock,
                &sanitize_name(&skill.name),
                source,
                &source_type_str,
                &hash,
            );
        }
        let _ = local_lock::write_local_lock(&local_lock, &cwd).await;
    }

    // Telemetry
    if parsed.source_type != SourceType::Local {
        let mut params = HashMap::new();
        params.insert("source".into(), source.to_string());
        params.insert("skills".into(), skill_slugs.join(","));
        params.insert(
            "agents".into(),
            selected_agents
                .iter()
                .map(|a| a.name.to_string())
                .collect::<Vec<_>>()
                .join(","),
        );
        if opts.global {
            params.insert("global".into(), "1".into());
        }
        telemetry::track("install", params);
    }

    // Cleanup temp dir
    if let Some(temp) = _temp_dir {
        let _ = git::cleanup_temp_dir(&temp);
    }

    Ok(())
}

async fn handle_provider_skills(
    provider: &crate::providers::types::Provider,
    url: &str,
    source: &str,
    opts: &AddOptions,
) -> anyhow::Result<()> {
    println!(
        "  {} {} (provider: {})",
        "Fetching skills from".dimmed(),
        url.dimmed(),
        provider.id().cyan()
    );

    let remote_skills = provider.fetch_all_skills(url).await?;

    if remote_skills.is_empty() {
        println!(
            "  {} No skills found at {} endpoint.",
            "!".yellow().bold(),
            provider.display_name()
        );
        return Ok(());
    }

    let source_id = provider.source_identifier(url);

    println!(
        "  {} {} skill(s) found",
        "✓".green(),
        remote_skills.len()
    );

    let all_configs = build_agent_configs();
    let selected_agents = select_agents(&all_configs, opts)?;

    let mut success_count = 0;
    for rs in &remote_skills {
        let tmp = tempfile::tempdir()?;
        let skill_dir = tmp.path().join(&rs.install_name);
        std::fs::create_dir_all(&skill_dir)?;
        std::fs::write(skill_dir.join("SKILL.md"), &rs.content)?;

        for agent in &selected_agents {
            let result = installer::install_skill_for_agent(
                &skill_dir,
                &rs.install_name,
                agent,
                opts.global,
                InstallMode::Copy,
            )
            .await;
            if result.success {
                success_count += 1;
                println!(
                    "  {} {} → {} [{}]",
                    "✓".green(),
                    rs.name,
                    agent.display_name.dimmed(),
                    result.path.display(),
                );
            }
        }
    }

    println!(
        "\n  {} Installed {} item(s) from {} ({})",
        "✓".green().bold(),
        success_count,
        provider.display_name(),
        source_id
    );

    // Telemetry
    let mut params = HashMap::new();
    params.insert("source".into(), source.to_string());
    params.insert("provider".into(), provider.id().to_string());
    params.insert(
        "skills".into(),
        remote_skills
            .iter()
            .map(|s| s.name.clone())
            .collect::<Vec<_>>()
            .join(","),
    );
    telemetry::track("install", params);

    Ok(())
}

fn select_skills(discovered: &[Skill], opts: &AddOptions) -> anyhow::Result<Vec<Skill>> {
    if opts.all || opts.yes || (opts.skills.len() == 1 && opts.skills[0] == "*") {
        return Ok(discovered.to_vec());
    }

    if !opts.skills.is_empty() {
        return Ok(filter_skills(discovered, &opts.skills));
    }

    if discovered.len() == 1 {
        return Ok(discovered.to_vec());
    }

    let items: Vec<SearchItem<String>> = discovered
        .iter()
        .map(|s| SearchItem {
            label: format!("{} - {}", s.name, s.description),
            value: s.name.clone(),
        })
        .collect();

    let multi_opts = MultiSelectOptions {
        prompt: "Select skills to install".into(),
        items,
        locked_values: Vec::new(),
        locked_labels: Vec::new(),
        max_visible: 15,
    };

    let selected_names = crate::prompts::search_multiselect::search_multiselect(multi_opts)?;

    Ok(discovered
        .iter()
        .filter(|s| selected_names.contains(&s.name))
        .cloned()
        .collect())
}

fn select_agents<'a>(
    configs: &'a [AgentConfig],
    opts: &AddOptions,
) -> anyhow::Result<Vec<&'a AgentConfig>> {
    if opts.all || (opts.agents.len() == 1 && opts.agents[0] == "*") {
        return Ok(configs.iter().filter(|c| c.show_in_universal_list).collect());
    }

    if !opts.agents.is_empty() {
        let mut result = Vec::new();
        for name in &opts.agents {
            match agents::get_agent_config(configs, name) {
                Some(cfg) => result.push(cfg),
                None => return Err(XSkillError::AgentNotFound(name.clone()).into()),
            }
        }
        let universal = get_universal_agents(configs);
        for u in universal {
            if !result.iter().any(|r| r.name == u.name) {
                result.push(u);
            }
        }
        return Ok(result);
    }

    let installed = detect_installed_agents(configs);

    if installed.is_empty() || opts.yes {
        let universal = get_universal_agents(configs);
        let mut result: Vec<&AgentConfig> = universal;
        for i in &installed {
            if !result.iter().any(|r| r.name == i.name) {
                result.push(i);
            }
        }
        return Ok(result);
    }

    // Interactive agent selection
    let universal = get_universal_agents(configs);
    let non_universal_installed: Vec<_> = installed
        .iter()
        .filter(|c| !agents::is_universal_agent(c))
        .collect();

    let items: Vec<SearchItem<String>> = non_universal_installed
        .iter()
        .map(|c| SearchItem {
            label: c.display_name.to_string(),
            value: c.name.to_string(),
        })
        .collect();

    let locked_values: Vec<String> = universal.iter().map(|c| c.name.to_string()).collect();
    let locked_labels: Vec<String> =
        universal.iter().map(|c| c.display_name.to_string()).collect();

    let multi_opts = MultiSelectOptions {
        prompt: "Select agents to install for".into(),
        items,
        locked_values: locked_values.clone(),
        locked_labels,
        max_visible: 15,
    };

    let selected_names = crate::prompts::search_multiselect::search_multiselect(multi_opts)?;

    Ok(configs
        .iter()
        .filter(|c| selected_names.contains(&c.name.to_string()))
        .collect())
}

fn chrono_like_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{secs}")
}
