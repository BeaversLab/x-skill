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
use crate::t;
use crate::telemetry;
use crate::types::{
    AddOptions, AgentConfig, DiscoverOptions, InstallMode, Skill, SkillLockEntry, SourceType,
};
use console::style;
use std::collections::{HashMap, HashSet};
use std::io::IsTerminal;

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
            let spinner = cliclack::spinner();
            spinner.start(t!("clone_start", "url" => &parsed.url));
            let temp = git::clone_repo(&parsed.url, parsed.ref_branch.as_deref()).await?;
            spinner.stop(t!("clone_complete"));
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
        cliclack::log::warning(t!("no_skills_found"))?;
        return Ok(());
    }

    // Apply skill_filter from @skill syntax
    if let Some(ref filter) = parsed.skill_filter {
        discovered.retain(|s| s.name.eq_ignore_ascii_case(filter));
        if discovered.is_empty() {
            return Err(XSkillError::SkillNotFound(filter.clone()).into());
        }
    }

    // List-only mode
    if opts.list_only {
        println!(
            "  {}\n",
            t!("skills_found_count", "count" => discovered.len())
        );
        for s in &discovered {
            println!("  {} {}", style("•").dim(), style(&s.name).bold());
            if !s.description.is_empty() {
                println!("    {}", style(&s.description).dim());
            }
        }
        return Ok(());
    }

    // Skill selection
    let selected_skills = select_skills(&discovered, opts)?;
    if selected_skills.is_empty() {
        cliclack::log::info(t!("no_skills_selected"))?;
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
        cliclack::log::info(t!("no_agents_selected"))?;
        return Ok(());
    }

    // Scope selection: Project vs Global
    let is_interactive = !opts.yes && std::io::stdin().is_terminal();
    let global = if opts.global {
        true
    } else if is_interactive
        && selected_agents
            .iter()
            .any(|a| a.global_skills_dir.is_some())
    {
        let scope: bool = cliclack::select(t!("scope_prompt"))
            .item(false, t!("scope_project"), t!("scope_project_hint"))
            .item(true, t!("scope_global"), t!("scope_global_hint"))
            .interact()?;
        scope
    } else {
        false
    };

    // Method selection: Symlink vs Copy
    let unique_dirs: HashSet<_> = selected_agents.iter().map(|a| a.skills_dir).collect();
    let mode = if opts.copy {
        InstallMode::Copy
    } else if is_interactive && unique_dirs.len() > 1 {
        let method: &str = cliclack::select(t!("method_prompt"))
            .item("symlink", t!("method_symlink"), t!("method_symlink_hint"))
            .item("copy", t!("method_copy"), t!("method_copy_hint"))
            .interact()?;
        if method == "symlink" {
            InstallMode::Symlink
        } else {
            InstallMode::Copy
        }
    } else {
        InstallMode::Copy
    };

    // Await audit result
    if let Some(handle) = audit_handle {
        let _audit_result = handle.await.ok().flatten();
    }

    // Summary
    println!(
        "\n  {}\n",
        t!("installing_summary",
            "skills" => style(selected_skills.len()).bold(),
            "agents" => style(selected_agents.len()).bold()
        )
    );
    for s in &selected_skills {
        println!("  {} {}", style("•").green(), s.name);
    }
    println!();
    for a in &selected_agents {
        println!("  {} {}", style("→").dim(), a.display_name);
    }
    println!();

    // Confirmation
    if !opts.yes && std::io::stdin().is_terminal() {
        let should_continue = cliclack::confirm(t!("confirm_continue"))
            .initial_value(true)
            .interact()?;
        if !should_continue {
            cliclack::outro_cancel(t!("cancelled"))?;
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
                global,
                mode,
            )
            .await;

            if result.success {
                success_count += 1;
                let mode_label = match result.mode {
                    InstallMode::Symlink => t!("mode_symlink"),
                    InstallMode::Copy => String::new(),
                };
                cliclack::log::success(format!(
                    "{} → {} [{}{}]",
                    skill.name,
                    style(agent.display_name).dim(),
                    result.path.display(),
                    mode_label,
                ))?;
            } else {
                fail_count += 1;
                let reason = result.error.unwrap_or_default();
                let install_err = XSkillError::InstallFailed {
                    skill: skill.name.clone(),
                    agent: agent.display_name.to_string(),
                    reason,
                };
                cliclack::log::error(format!("{install_err}"))?;
            }

            if result.symlink_failed {
                cliclack::log::warning(
                    t!("symlink_fallback", "agent" => agent.display_name),
                )?;
            }
        }
    }

    println!();
    if fail_count == 0 {
        cliclack::log::success(t!("install_success", "count" => success_count))?;
    } else {
        cliclack::log::warning(
            t!("install_partial", "success" => success_count, "fail" => fail_count),
        )?;
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
    if !global {
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
        params.insert(
            "sourceType".into(),
            parsed.source_type.telemetry_source_type().to_string(),
        );
        params.insert("skills".into(), skill_slugs.join(","));
        params.insert(
            "agents".into(),
            selected_agents
                .iter()
                .map(|a| a.name.to_string())
                .collect::<Vec<_>>()
                .join(","),
        );
        if global {
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
    let spinner = cliclack::spinner();
    spinner.start(t!("provider_fetching", "url" => url, "provider" => provider.id()));

    let remote_skills = provider.fetch_all_skills(url).await?;

    if remote_skills.is_empty() {
        spinner.stop(t!("done"));
        cliclack::log::warning(
            t!("provider_no_skills", "name" => provider.display_name()),
        )?;
        return Ok(());
    }

    let source_id = provider.source_identifier(url);

    spinner.stop(t!("provider_skills_found", "count" => remote_skills.len()));

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
                cliclack::log::success(format!(
                    "{} → {} [{}]",
                    rs.name,
                    style(agent.display_name).dim(),
                    result.path.display(),
                ))?;
            }
        }
    }

    cliclack::log::success(t!(
        "provider_install_success",
        "count" => success_count,
        "name" => provider.display_name(),
        "id" => source_id
    ))?;

    // Telemetry
    let mut params = HashMap::new();
    params.insert("source".into(), source.to_string());
    params.insert(
        "sourceType".into(),
        SourceType::WellKnown.telemetry_source_type().to_string(),
    );
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
        prompt: t!("select_skills"),
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
        prompt: t!("select_agents"),
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
