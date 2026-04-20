use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct MarketplaceManifest {
    metadata: Option<MarketplaceMetadata>,
    plugins: Option<Vec<PluginManifestEntry>>,
}

#[derive(Debug, Deserialize)]
struct MarketplaceMetadata {
    #[serde(rename = "pluginRoot")]
    plugin_root: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
enum PluginSource {
    Simple(String),
    Complex {
        source: String,
        repo: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
struct PluginManifestEntry {
    source: Option<PluginSource>,
    skills: Option<Vec<String>>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PluginManifest {
    skills: Option<Vec<String>>,
    name: Option<String>,
}

pub fn get_plugin_skill_paths(base_path: &Path) -> Vec<PathBuf> {
    let mut search_dirs = Vec::new();

    // marketplace.json
    let marketplace_path = base_path.join(".claude-plugin/marketplace.json");
    if let Some(manifest) = read_json::<MarketplaceManifest>(&marketplace_path) {
        let plugin_root = manifest
            .metadata
            .as_ref()
            .and_then(|m| m.plugin_root.as_deref())
            .unwrap_or("");

        if !plugin_root.is_empty() && !plugin_root.starts_with("./") {
            // Invalid plugin root — skip
        } else if let Some(plugins) = &manifest.plugins {
            for plugin in plugins {
                let source_str = match &plugin.source {
                    Some(PluginSource::Simple(s)) => Some(s.as_str()),
                    Some(PluginSource::Complex { .. }) => continue, // Skip remote
                    None => None,
                };

                let plugin_base = if let Some(s) = source_str {
                    base_path.join(plugin_root).join(s)
                } else {
                    base_path.join(plugin_root)
                };

                add_plugin_skill_paths(
                    &plugin_base,
                    plugin.skills.as_deref(),
                    &mut search_dirs,
                    base_path,
                );
            }
        }
    }

    // plugin.json
    let plugin_path = base_path.join(".claude-plugin/plugin.json");
    if let Some(manifest) = read_json::<PluginManifest>(&plugin_path) {
        add_plugin_skill_paths(
            base_path,
            manifest.skills.as_deref(),
            &mut search_dirs,
            base_path,
        );
    }

    search_dirs
}

pub fn get_plugin_groupings(base_path: &Path) -> HashMap<PathBuf, String> {
    let mut groupings = HashMap::new();

    let marketplace_path = base_path.join(".claude-plugin/marketplace.json");
    if let Some(manifest) = read_json::<MarketplaceManifest>(&marketplace_path) {
        let plugin_root = manifest
            .metadata
            .as_ref()
            .and_then(|m| m.plugin_root.as_deref())
            .unwrap_or("");

        if let Some(plugins) = &manifest.plugins {
            for plugin in plugins {
                let source_str = match &plugin.source {
                    Some(PluginSource::Simple(s)) => Some(s.as_str()),
                    Some(PluginSource::Complex { .. }) => continue,
                    None => None,
                };

                let plugin_base = if let Some(s) = source_str {
                    base_path.join(plugin_root).join(s)
                } else {
                    base_path.join(plugin_root)
                };

                if let (Some(skills), Some(name)) = (&plugin.skills, &plugin.name) {
                    for skill_path in skills {
                        let skill_dir = plugin_base.join(skill_path);
                        if let Ok(resolved) = fs::canonicalize(&skill_dir) {
                            groupings.insert(resolved, name.clone());
                        }
                    }
                }
            }
        }
    }

    let plugin_path = base_path.join(".claude-plugin/plugin.json");
    if let Some(manifest) = read_json::<PluginManifest>(&plugin_path) {
        if let (Some(skills), Some(name)) = (&manifest.skills, &manifest.name) {
            for skill_path in skills {
                let skill_dir = base_path.join(skill_path);
                if let Ok(resolved) = fs::canonicalize(&skill_dir) {
                    groupings.insert(resolved, name.clone());
                }
            }
        }
    }

    groupings
}

fn add_plugin_skill_paths(
    plugin_base: &Path,
    skills: Option<&[String]>,
    search_dirs: &mut Vec<PathBuf>,
    root_base: &Path,
) {
    if let Some(skill_paths) = skills {
        for sp in skill_paths {
            if !sp.starts_with("./") {
                continue;
            }
            let full = plugin_base.join(sp);
            if is_contained_in(&full, root_base) {
                if let Some(parent) = full.parent() {
                    search_dirs.push(parent.to_path_buf());
                }
            }
        }
    }
    // Always add skills/ subdir
    let skills_dir = plugin_base.join("skills");
    search_dirs.push(skills_dir);
}

fn is_contained_in(target: &Path, base: &Path) -> bool {
    let abs_base = std::path::absolute(base).unwrap_or_else(|_| base.to_path_buf());
    let abs_target = std::path::absolute(target).unwrap_or_else(|_| target.to_path_buf());
    abs_target.starts_with(&abs_base)
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Option<T> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_manifests() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = get_plugin_skill_paths(tmp.path());
        assert!(paths.is_empty());
    }

    #[test]
    fn test_plugin_json() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin_dir = tmp.path().join(".claude-plugin");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(
            plugin_dir.join("plugin.json"),
            r#"{"skills": ["./skills/my-skill/SKILL.md"], "name": "test-plugin"}"#,
        )
        .unwrap();
        let paths = get_plugin_skill_paths(tmp.path());
        assert!(!paths.is_empty());
    }
}
