use crate::t;
use std::fs;

pub fn run(name: Option<&str>) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let skill_name = name.unwrap_or_else(|| {
        cwd.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-skill")
    });

    let skill_dir = if name.is_some() {
        cwd.join(skill_name)
    } else {
        cwd.clone()
    };

    let skill_file = skill_dir.join(crate::constants::SKILL_MD);
    if skill_file.exists() {
        anyhow::bail!("{}", t!("already_exists", "path" => skill_file.display()));
    }

    if name.is_some() {
        fs::create_dir_all(&skill_dir)?;
    }

    let content = format!(
        r#"---
name: {skill_name}
description: A brief description of what this skill does
---

# {skill_name}

Instructions for the agent to follow when this skill is activated.

## When to use

Describe when this skill should be used.

## Instructions

1. First step
2. Second step
3. Additional steps as needed
"#
    );

    fs::write(&skill_file, content)?;
    println!("{}", t!("created", "path" => skill_file.display()));
    Ok(())
}
