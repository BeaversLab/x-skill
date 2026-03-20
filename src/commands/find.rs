use crate::output;
use colored::Colorize;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct SearchResult {
    name: String,
    description: String,
    source: String,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
}

pub async fn run(query: Option<&str>) -> anyhow::Result<()> {
    output::show_logo();
    println!();

    let api_base = std::env::var("SKILLS_API_URL")
        .unwrap_or_else(|_| crate::constants::SKILLS_API_URL.to_string());

    let url = if let Some(q) = query {
        format!("{api_base}/api/search?q={}", urlencoded(q))
    } else {
        format!("{api_base}/api/search")
    };

    println!("  {} skills.sh...", "Searching".dimmed());

    let resp = crate::http::client().get(&url).send().await;

    let results = match resp {
        Ok(r) if r.status().is_success() => {
            r.json::<SearchResponse>()
                .await
                .map(|s| s.results)
                .unwrap_or_default()
        }
        _ => {
            println!("  {} Could not reach skills.sh", "!".yellow().bold());
            return Ok(());
        }
    };

    if results.is_empty() {
        println!("  {} No skills found.", "!".yellow().bold());
        return Ok(());
    }

    println!("  {} {} skill(s) found:\n", "✓".green(), results.len());

    for r in &results {
        println!("  {} {}", "•".green(), r.name.bold());
        println!("    {}", r.description.dimmed());
        println!("    {}", format!("x-skill add {}", r.source).cyan());
        println!();
    }

    // Telemetry
    let mut params = std::collections::HashMap::new();
    params.insert("query".into(), query.unwrap_or("").to_string());
    params.insert("resultCount".into(), results.len().to_string());
    crate::telemetry::track("find", params);

    Ok(())
}

fn urlencoded(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}
