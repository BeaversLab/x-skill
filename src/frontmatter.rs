use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct SkillFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Extract YAML frontmatter from SKILL.md content.
/// Returns (frontmatter, body) if found, or None if no valid frontmatter.
pub fn extract_frontmatter(content: &str) -> Option<(SkillFrontmatter, String)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let after_start = &trimmed[3..];
    let end_idx = after_start.find("\n---")?;
    let yaml_str = &after_start[..end_idx];
    let body = &after_start[end_idx + 4..];

    let fm: SkillFrontmatter = serde_yml::from_str(yaml_str).ok()?;
    Some((fm, body.trim_start_matches('\n').to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_frontmatter() {
        let input = "---\nname: my-skill\ndescription: A test skill\n---\n# Body";
        let (fm, body) = extract_frontmatter(input).unwrap();
        assert_eq!(fm.name.as_deref(), Some("my-skill"));
        assert_eq!(fm.description.as_deref(), Some("A test skill"));
        assert_eq!(body, "# Body");
    }

    #[test]
    fn test_no_frontmatter() {
        assert!(extract_frontmatter("# Just a heading").is_none());
    }

    #[test]
    fn test_malformed_yaml() {
        let input = "---\n: broken yaml [[\n---\nbody";
        assert!(extract_frontmatter(input).is_none());
    }

    #[test]
    fn test_with_metadata() {
        let input = "---\nname: test\ndescription: d\nmetadata:\n  internal: true\n---\ncontent";
        let (fm, _) = extract_frontmatter(input).unwrap();
        let meta = fm.metadata.unwrap();
        assert_eq!(meta.get("internal").and_then(|v| v.as_bool()), Some(true));
    }
}
