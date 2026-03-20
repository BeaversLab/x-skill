use crate::types::{ProviderMatch, RemoteSkill};
use regex::Regex;
use serde::Deserialize;
use std::sync::LazyLock;

static NAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9][a-z0-9-]{0,63}$").unwrap());

pub struct WellKnownProvider;

#[derive(Debug, Deserialize)]
struct WellKnownIndex {
    skills: Vec<WellKnownSkillEntry>,
}

#[derive(Debug, Deserialize)]
struct WellKnownSkillEntry {
    name: String,
    description: String,
    files: Vec<String>,
}

impl WellKnownProvider {
    pub fn id(&self) -> &str {
        "wellknown"
    }

    pub fn display_name(&self) -> &str {
        "Well-Known"
    }

    pub fn match_url(&self, url: &str) -> Option<ProviderMatch> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return None;
        }
        let host = extract_host(url)?;
        let excluded = ["github.com", "gitlab.com", "huggingface.co"];
        if excluded.contains(&host.as_str()) {
            return None;
        }
        Some(ProviderMatch {
            matches: true,
            source_identifier: Some(format!("wellknown/{host}")),
        })
    }

    pub fn to_raw_url(&self, url: &str) -> String {
        url.to_string()
    }

    pub fn source_identifier(&self, url: &str) -> String {
        extract_host(url)
            .map(|h| format!("wellknown/{h}"))
            .unwrap_or_else(|| "wellknown/unknown".into())
    }

    pub async fn fetch_skill(&self, url: &str) -> anyhow::Result<Option<RemoteSkill>> {
        let skills = self.fetch_all_skills(url).await?;
        Ok(skills.into_iter().next())
    }

    pub async fn fetch_all_skills(&self, url: &str) -> anyhow::Result<Vec<RemoteSkill>> {
        let index = match self.fetch_index(url).await {
            Some(idx) => idx,
            None => return Ok(Vec::new()),
        };

        let base_url = url.trim_end_matches('/');
        let host = extract_host(url).unwrap_or_default();
        let source_id = format!("wellknown/{host}");

        let mut results = Vec::new();
        for entry in &index.skills {
            if !is_valid_skill_entry(entry) {
                continue;
            }
            if let Some(skill) = self.fetch_skill_by_entry(base_url, entry, &source_id).await {
                results.push(skill);
            }
        }
        Ok(results)
    }

    async fn fetch_index(&self, url: &str) -> Option<WellKnownIndex> {
        let base = url.trim_end_matches('/');

        // Try path-relative first
        let relative_url = format!("{base}/.well-known/skills/index.json");
        if let Some(idx) = try_fetch_index(&relative_url).await {
            return Some(idx);
        }

        // If base has a path, try root
        if let Some(root) = get_root_url(url) {
            let root_url = format!("{root}/.well-known/skills/index.json");
            if let Some(idx) = try_fetch_index(&root_url).await {
                return Some(idx);
            }
        }

        None
    }

    async fn fetch_skill_by_entry(
        &self,
        base_url: &str,
        entry: &WellKnownSkillEntry,
        source_id: &str,
    ) -> Option<RemoteSkill> {
        // Find SKILL.md file
        let skill_md_file = entry
            .files
            .iter()
            .find(|f| f.eq_ignore_ascii_case("SKILL.md"))?;

        let skill_md_url = format!(
            "{base_url}/.well-known/skills/{}/{}",
            entry.name, skill_md_file
        );

        let content = reqwest::get(&skill_md_url)
            .await
            .ok()?
            .text()
            .await
            .ok()?;

        let fm = crate::frontmatter::extract_frontmatter(&content)?;
        let name = fm.0.name.unwrap_or_else(|| entry.name.clone());
        let description = fm.0.description.unwrap_or_else(|| entry.description.clone());

        // Fetch remaining files in parallel
        let other_files: Vec<_> = entry
            .files
            .iter()
            .filter(|f| !f.eq_ignore_ascii_case("SKILL.md"))
            .collect();

        let fetches = other_files.iter().map(|f| {
            let file_url = format!(
                "{base_url}/.well-known/skills/{}/{}",
                entry.name, f
            );
            async move {
                reqwest::get(&file_url)
                    .await
                    .ok()
                    .and_then(|r| futures::executor::block_on(r.text()).ok())
                    .map(|text| (f.to_string(), text))
            }
        });

        let _file_results: Vec<_> = futures::future::join_all(fetches).await;

        Some(RemoteSkill {
            name: name.clone(),
            description,
            content,
            install_name: crate::installer::sanitize_name(&name),
            source_url: base_url.to_string(),
            provider_id: "wellknown".into(),
            source_identifier: source_id.to_string(),
            metadata: fm.0.metadata,
        })
    }
}

fn is_valid_skill_entry(entry: &WellKnownSkillEntry) -> bool {
    if entry.name.is_empty() || entry.description.is_empty() || entry.files.is_empty() {
        return false;
    }
    if !NAME_RE.is_match(&entry.name) {
        return false;
    }
    let has_skill_md = entry
        .files
        .iter()
        .any(|f| f.eq_ignore_ascii_case("SKILL.md"));
    if !has_skill_md {
        return false;
    }
    // Path traversal protection
    for file in &entry.files {
        if file.starts_with('/') || file.starts_with('\\') || file.contains("..") {
            return false;
        }
    }
    true
}

async fn try_fetch_index(url: &str) -> Option<WellKnownIndex> {
    let resp = reqwest::get(url).await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let idx: WellKnownIndex = resp.json().await.ok()?;
    if idx.skills.is_empty() {
        return None;
    }
    Some(idx)
}

fn extract_host(url: &str) -> Option<String> {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host = without_scheme.split('/').next()?;
    let host = host.split(':').next()?;
    Some(host.to_lowercase())
}

fn get_root_url(url: &str) -> Option<String> {
    let scheme_end = url.find("://")? + 3;
    let rest = &url[scheme_end..];
    let host_end = rest.find('/');
    host_end.map(|idx| url[..scheme_end + idx].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_url_accepts_http() {
        let p = WellKnownProvider;
        assert!(p.match_url("https://example.com/docs").is_some());
    }

    #[test]
    fn test_match_url_rejects_github() {
        let p = WellKnownProvider;
        assert!(p.match_url("https://github.com/owner/repo").is_none());
    }

    #[test]
    fn test_match_url_rejects_non_http() {
        let p = WellKnownProvider;
        assert!(p.match_url("git@example.com:owner/repo").is_none());
    }

    #[test]
    fn test_valid_skill_entry() {
        let entry = WellKnownSkillEntry {
            name: "my-skill".into(),
            description: "A skill".into(),
            files: vec!["SKILL.md".into()],
        };
        assert!(is_valid_skill_entry(&entry));
    }

    #[test]
    fn test_invalid_skill_entry_no_skill_md() {
        let entry = WellKnownSkillEntry {
            name: "my-skill".into(),
            description: "A skill".into(),
            files: vec!["README.md".into()],
        };
        assert!(!is_valid_skill_entry(&entry));
    }

    #[test]
    fn test_invalid_skill_entry_path_traversal() {
        let entry = WellKnownSkillEntry {
            name: "evil".into(),
            description: "Bad".into(),
            files: vec!["SKILL.md".into(), "../../../etc/passwd".into()],
        };
        assert!(!is_valid_skill_entry(&entry));
    }

    #[test]
    fn test_source_identifier() {
        let p = WellKnownProvider;
        assert_eq!(
            p.source_identifier("https://mintlify.com/docs"),
            "wellknown/mintlify.com"
        );
    }
}
