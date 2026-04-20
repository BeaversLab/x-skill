use crate::constants::{MAX_DISCOVERY_DEPTH, SKILL_MD, SKIP_DIRS};
use crate::frontmatter::extract_frontmatter;
use crate::plugin_manifest;
use crate::types::{DiscoverOptions, Skill};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Priority sub-directories to scan for skills (in order).
const PRIORITY_DIRS: &[&str] = &[
    "",
    "skills",
    "skills/.curated",
    "skills/.experimental",
    "skills/.system",
    ".agent/skills",
    ".agents/skills",
    ".claude/skills",
    ".cline/skills",
    ".codebuddy/skills",
    ".codex/skills",
    ".commandcode/skills",
    ".continue/skills",
    ".github/skills",
    ".goose/skills",
    ".iflow/skills",
    ".junie/skills",
    ".kilocode/skills",
    ".kiro/skills",
    ".mux/skills",
    ".neovate/skills",
    ".opencode/skills",
    ".openhands/skills",
    ".pi/skills",
    ".qoder/skills",
    ".roo/skills",
    ".trae/skills",
    ".windsurf/skills",
    ".zencoder/skills",
];

pub fn discover_skills(
    base_path: &Path,
    subpath: Option<&str>,
    options: &DiscoverOptions,
) -> anyhow::Result<Vec<Skill>> {
    let search_path = if let Some(sp) = subpath {
        if !is_subpath_safe(base_path, sp) {
            anyhow::bail!("unsafe subpath: {sp}");
        }
        base_path.join(sp)
    } else {
        base_path.to_path_buf()
    };

    if !search_path.exists() {
        return Ok(Vec::new());
    }

    let should_install_internal = options.include_internal || is_internal_env_set();

    // 1. Direct SKILL.md at search_path
    let direct = search_path.join(SKILL_MD);
    if direct.exists() && !options.full_depth {
        if let Some(skill) = parse_skill_file(&direct, should_install_internal) {
            return Ok(vec![skill]);
        }
    }

    // 2. Scan priority directories
    let mut skills = Vec::new();
    let mut seen_paths = HashSet::new();

    for subdir in PRIORITY_DIRS {
        let dir = if subdir.is_empty() {
            search_path.clone()
        } else {
            search_path.join(subdir)
        };
        if dir.exists() && dir.is_dir() {
            scan_dir_for_skills(&dir, &mut skills, &mut seen_paths, should_install_internal);
        }
    }

    // 3. Plugin skill paths
    let plugin_paths = plugin_manifest::get_plugin_skill_paths(&search_path);
    for dir in &plugin_paths {
        if dir.exists() && dir.is_dir() {
            scan_dir_for_skills(dir, &mut skills, &mut seen_paths, should_install_internal);
        }
    }

    // 4. Apply plugin groupings
    let groupings = plugin_manifest::get_plugin_groupings(&search_path);
    for skill in &mut skills {
        let resolved = fs::canonicalize(&skill.path).unwrap_or_else(|_| skill.path.clone());
        if let Some(plugin_name) = groupings.get(&resolved) {
            skill.plugin_name = Some(plugin_name.clone());
        }
    }

    // 5. Recursive search if no skills found or full_depth requested
    if skills.is_empty() || options.full_depth {
        find_skill_dirs(
            &search_path,
            0,
            &mut skills,
            &mut seen_paths,
            should_install_internal,
        );
    }

    Ok(skills)
}

fn scan_dir_for_skills(
    dir: &Path,
    skills: &mut Vec<Skill>,
    seen: &mut HashSet<PathBuf>,
    include_internal: bool,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let skill_file = path.join(SKILL_MD);
            if skill_file.exists() {
                let canonical = fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
                if seen.insert(canonical) {
                    if let Some(skill) = parse_skill_file(&skill_file, include_internal) {
                        skills.push(skill);
                    }
                }
            }
        }
    }
}

fn find_skill_dirs(
    dir: &Path,
    depth: usize,
    skills: &mut Vec<Skill>,
    seen: &mut HashSet<PathBuf>,
    include_internal: bool,
) {
    if depth > MAX_DISCOVERY_DEPTH {
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if SKIP_DIRS.contains(&name_str.as_ref()) {
            continue;
        }

        let skill_file = path.join(SKILL_MD);
        if skill_file.exists() {
            let canonical = fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            if seen.insert(canonical) {
                if let Some(skill) = parse_skill_file(&skill_file, include_internal) {
                    skills.push(skill);
                }
            }
        }

        find_skill_dirs(&path, depth + 1, skills, seen, include_internal);
    }
}

fn parse_skill_file(skill_md_path: &Path, include_internal: bool) -> Option<Skill> {
    let content = fs::read_to_string(skill_md_path).ok()?;
    let (fm, _body) = extract_frontmatter(&content)?;

    let name = fm.name?;
    let description = fm.description?;

    // Filter internal skills
    if let Some(ref meta) = fm.metadata {
        if let Some(internal) = meta.get("internal") {
            if internal.as_bool() == Some(true) && !include_internal {
                return None;
            }
        }
    }

    let skill_dir = skill_md_path.parent()?.to_path_buf();

    Some(Skill {
        name,
        description,
        path: skill_dir,
        raw_content: Some(content),
        plugin_name: None,
        metadata: fm.metadata,
    })
}

pub fn is_subpath_safe(base_path: &Path, subpath: &str) -> bool {
    let base =
        normalize_path(&std::path::absolute(base_path).unwrap_or_else(|_| base_path.to_path_buf()));
    let target_raw = base_path.join(subpath);
    let target = normalize_path(&std::path::absolute(&target_raw).unwrap_or(target_raw));
    target.starts_with(&base)
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for comp in path.components() {
        match comp {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            other => components.push(other),
        }
    }
    components.iter().collect()
}

fn is_internal_env_set() -> bool {
    std::env::var("INSTALL_INTERNAL_SKILLS")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Filter skills by name or skill_filter pattern.
pub fn filter_skills(skills: &[Skill], names: &[String]) -> Vec<Skill> {
    if names.len() == 1 && names[0] == "*" {
        return skills.to_vec();
    }
    skills
        .iter()
        .filter(|s| names.iter().any(|n| s.name.eq_ignore_ascii_case(n)))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subpath_safe() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(is_subpath_safe(tmp.path(), "skills/my-skill"));
        assert!(!is_subpath_safe(tmp.path(), "../../etc/passwd"));
    }

    #[test]
    fn test_filter_skills_wildcard() {
        let skills = vec![
            Skill {
                name: "a".into(),
                description: "".into(),
                path: PathBuf::new(),
                raw_content: None,
                plugin_name: None,
                metadata: None,
            },
            Skill {
                name: "b".into(),
                description: "".into(),
                path: PathBuf::new(),
                raw_content: None,
                plugin_name: None,
                metadata: None,
            },
        ];
        assert_eq!(filter_skills(&skills, &["*".into()]).len(), 2);
    }

    #[test]
    fn test_filter_skills_by_name() {
        let skills = vec![
            Skill {
                name: "alpha".into(),
                description: "".into(),
                path: PathBuf::new(),
                raw_content: None,
                plugin_name: None,
                metadata: None,
            },
            Skill {
                name: "beta".into(),
                description: "".into(),
                path: PathBuf::new(),
                raw_content: None,
                plugin_name: None,
                metadata: None,
            },
        ];
        let filtered = filter_skills(&skills, &["beta".into()]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "beta");
    }

    #[test]
    fn test_discover_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let opts = DiscoverOptions::default();
        let result = discover_skills(tmp.path(), None, &opts).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_discover_direct_skill() {
        let tmp = tempfile::tempdir().unwrap();
        let skill_md = tmp.path().join("SKILL.md");
        fs::write(
            &skill_md,
            "---\nname: test-skill\ndescription: A test\n---\n# Test",
        )
        .unwrap();
        let opts = DiscoverOptions::default();
        let result = discover_skills(tmp.path(), None, &opts).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "test-skill");
    }

    #[test]
    fn test_discover_skills_subdir() {
        let tmp = tempfile::tempdir().unwrap();
        let skill_dir = tmp.path().join("skills/my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: my-skill\ndescription: desc\n---\n# My Skill",
        )
        .unwrap();
        let opts = DiscoverOptions::default();
        let result = discover_skills(tmp.path(), None, &opts).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "my-skill");
    }
}
