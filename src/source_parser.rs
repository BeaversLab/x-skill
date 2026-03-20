use crate::error::XSkillError;
use crate::types::{ParsedSource, SourceType};
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;

static SOURCE_ALIASES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([("coinbase/agentWallet", "coinbase/agentic-wallet-skills")])
});

static RE_GITHUB_PREFIX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^github:(.+)$").unwrap());
static RE_GITLAB_PREFIX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^gitlab:(.+)$").unwrap());
static RE_GITHUB_TREE_PATH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"github\.com/([^/]+)/([^/]+)/tree/([^/]+)/(.+)").unwrap());
static RE_GITHUB_TREE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"github\.com/([^/]+)/([^/]+)/tree/([^/]+)$").unwrap());
static RE_GITHUB_REPO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"github\.com/([^/]+)/([^/]+)").unwrap());
static RE_GITLAB_TREE_PATH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(https?):\/\/([^/]+)\/(.+?)\/-\/tree\/([^/]+)\/(.+)").unwrap()
});
static RE_GITLAB_TREE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(https?):\/\/([^/]+)\/(.+?)\/-\/tree\/([^/]+)$").unwrap()
});
static RE_GITLAB_REPO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"gitlab\.com/(.+?)(?:\.git)?/?$").unwrap());
static RE_AT_SKILL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([^/]+)/([^/@]+)@(.+)$").unwrap());
static RE_SHORTHAND: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([^/]+)/([^/]+)(?:/(.+))?$").unwrap());
static RE_WINDOWS_ABS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z]:[/\\]").unwrap());

pub fn parse_source(input: &str) -> ParsedSource {
    // Step 0: Alias resolution
    let input = if let Some(&alias) = SOURCE_ALIASES.get(input) {
        alias
    } else {
        input
    };

    // Step 1: github: prefix → recursive parse
    if let Some(caps) = RE_GITHUB_PREFIX.captures(input) {
        return parse_source(caps.get(1).unwrap().as_str());
    }

    // Step 2: gitlab: prefix → recursive parse
    if let Some(caps) = RE_GITLAB_PREFIX.captures(input) {
        let rest = caps.get(1).unwrap().as_str();
        return parse_source(&format!("https://gitlab.com/{rest}"));
    }

    // Step 3: Local path detection
    if is_local_path(input) {
        let resolved = std::path::absolute(input).unwrap_or_else(|_| PathBuf::from(input));
        return ParsedSource {
            source_type: SourceType::Local,
            url: resolved.to_string_lossy().into_owned(),
            local_path: Some(resolved),
            ..Default::default()
        };
    }

    // Step 4: GitHub tree URL with path
    if let Some(caps) = RE_GITHUB_TREE_PATH.captures(input) {
        let owner = caps.get(1).unwrap().as_str();
        let repo = caps.get(2).unwrap().as_str();
        let ref_branch = caps.get(3).unwrap().as_str();
        let subpath = caps.get(4).unwrap().as_str();
        return ParsedSource {
            source_type: SourceType::Github,
            url: format!("https://github.com/{owner}/{repo}.git"),
            ref_branch: Some(ref_branch.to_string()),
            subpath: sanitize_subpath(subpath).ok().map(|s| s.to_string()),
            ..Default::default()
        };
    }

    // Step 5: GitHub tree URL (branch only)
    if let Some(caps) = RE_GITHUB_TREE.captures(input) {
        let owner = caps.get(1).unwrap().as_str();
        let repo = caps.get(2).unwrap().as_str();
        let ref_branch = caps.get(3).unwrap().as_str();
        return ParsedSource {
            source_type: SourceType::Github,
            url: format!("https://github.com/{owner}/{repo}.git"),
            ref_branch: Some(ref_branch.to_string()),
            ..Default::default()
        };
    }

    // Step 6: GitHub repo URL
    if let Some(caps) = RE_GITHUB_REPO.captures(input) {
        let owner = caps.get(1).unwrap().as_str();
        let repo = caps.get(2).unwrap().as_str().trim_end_matches(".git");
        return ParsedSource {
            source_type: SourceType::Github,
            url: format!("https://github.com/{owner}/{repo}.git"),
            ..Default::default()
        };
    }

    // Step 7: GitLab tree URL with path (any host, identified by /-/tree/)
    if let Some(caps) = RE_GITLAB_TREE_PATH.captures(input) {
        let protocol = caps.get(1).unwrap().as_str();
        let hostname = caps.get(2).unwrap().as_str();
        let repo_path = caps.get(3).unwrap().as_str();
        let ref_branch = caps.get(4).unwrap().as_str();
        let subpath = caps.get(5).unwrap().as_str();
        if hostname != "github.com" {
            let clean = repo_path.trim_end_matches(".git");
            return ParsedSource {
                source_type: SourceType::Gitlab,
                url: format!("{protocol}://{hostname}/{clean}.git"),
                ref_branch: Some(ref_branch.to_string()),
                subpath: sanitize_subpath(subpath).ok().map(|s| s.to_string()),
                ..Default::default()
            };
        }
    }

    // Step 8: GitLab tree URL (branch only)
    if let Some(caps) = RE_GITLAB_TREE.captures(input) {
        let protocol = caps.get(1).unwrap().as_str();
        let hostname = caps.get(2).unwrap().as_str();
        let repo_path = caps.get(3).unwrap().as_str();
        let ref_branch = caps.get(4).unwrap().as_str();
        if hostname != "github.com" {
            let clean = repo_path.trim_end_matches(".git");
            return ParsedSource {
                source_type: SourceType::Gitlab,
                url: format!("{protocol}://{hostname}/{clean}.git"),
                ref_branch: Some(ref_branch.to_string()),
                ..Default::default()
            };
        }
    }

    // Step 9: GitLab.com repo URL
    if let Some(caps) = RE_GITLAB_REPO.captures(input) {
        let repo_path = caps.get(1).unwrap().as_str();
        if repo_path.contains('/') {
            return ParsedSource {
                source_type: SourceType::Gitlab,
                url: format!("https://gitlab.com/{repo_path}.git"),
                ..Default::default()
            };
        }
    }

    // Step 10: GitHub shorthand with @skill filter
    if let Some(caps) = RE_AT_SKILL.captures(input) {
        if !input.contains(':') && !input.starts_with('.') && !input.starts_with('/') {
            let owner = caps.get(1).unwrap().as_str();
            let repo = caps.get(2).unwrap().as_str();
            let skill_filter = caps.get(3).unwrap().as_str();
            return ParsedSource {
                source_type: SourceType::Github,
                url: format!("https://github.com/{owner}/{repo}.git"),
                skill_filter: Some(skill_filter.to_string()),
                ..Default::default()
            };
        }
    }

    // Step 11: GitHub shorthand (owner/repo or owner/repo/subpath)
    if let Some(caps) = RE_SHORTHAND.captures(input) {
        if !input.contains(':') && !input.starts_with('.') && !input.starts_with('/') {
            let owner = caps.get(1).unwrap().as_str();
            let repo = caps.get(2).unwrap().as_str();
            let subpath = caps.get(3).map(|m| m.as_str());
            return ParsedSource {
                source_type: SourceType::Github,
                url: format!("https://github.com/{owner}/{repo}.git"),
                subpath: subpath
                    .and_then(|s| sanitize_subpath(s).ok())
                    .map(|s| s.to_string()),
                ..Default::default()
            };
        }
    }

    // Step 12: Well-known URL
    if is_well_known_url(input) {
        return ParsedSource {
            source_type: SourceType::WellKnown,
            url: input.to_string(),
            ..Default::default()
        };
    }

    // Step 13: Fallback to generic Git URL
    ParsedSource {
        source_type: SourceType::Git,
        url: input.to_string(),
        ..Default::default()
    }
}

fn is_local_path(input: &str) -> bool {
    std::path::Path::new(input).is_absolute()
        || input.starts_with("./")
        || input.starts_with("../")
        || input == "."
        || input == ".."
        || RE_WINDOWS_ABS.is_match(input)
}

fn is_well_known_url(input: &str) -> bool {
    if !input.starts_with("http://") && !input.starts_with("https://") {
        return false;
    }
    if input.ends_with(".git") {
        return false;
    }
    let excluded = ["github.com", "gitlab.com", "raw.githubusercontent.com"];
    if let Some(host) = extract_hostname(input) {
        if excluded.contains(&host.as_str()) {
            return false;
        }
        return true;
    }
    false
}

fn extract_hostname(url: &str) -> Option<String> {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host = without_scheme.split('/').next()?;
    let host = host.split(':').next()?;
    Some(host.to_lowercase())
}

pub fn sanitize_subpath(subpath: &str) -> Result<String, XSkillError> {
    let normalized = subpath.replace('\\', "/");
    for segment in normalized.split('/') {
        if segment == ".." {
            return Err(XSkillError::PathTraversal(subpath.to_string()));
        }
    }
    Ok(subpath.to_string())
}

pub fn get_owner_repo(parsed: &ParsedSource) -> Option<String> {
    if parsed.source_type == SourceType::Local {
        return None;
    }

    // SSH URLs
    let ssh_re = Regex::new(r"^git@[^:]+:(.+)$").unwrap();
    if let Some(caps) = ssh_re.captures(&parsed.url) {
        let path = caps.get(1).unwrap().as_str().trim_end_matches(".git");
        if path.contains('/') {
            return Some(path.to_string());
        }
        return None;
    }

    // HTTP(S) URLs
    if !parsed.url.starts_with("http://") && !parsed.url.starts_with("https://") {
        return None;
    }
    let without_scheme = parsed
        .url
        .strip_prefix("https://")
        .or_else(|| parsed.url.strip_prefix("http://"))?;
    let path = without_scheme
        .split_once('/')?
        .1
        .trim_end_matches(".git");
    if path.contains('/') {
        Some(path.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_resolution() {
        let p = parse_source("coinbase/agentWallet");
        assert_eq!(p.source_type, SourceType::Github);
        assert!(p.url.contains("agentic-wallet-skills"));
    }

    #[test]
    fn test_github_prefix() {
        let p = parse_source("github:owner/repo");
        assert_eq!(p.source_type, SourceType::Github);
        assert_eq!(p.url, "https://github.com/owner/repo.git");
    }

    #[test]
    fn test_gitlab_prefix() {
        let p = parse_source("gitlab:owner/repo");
        assert_eq!(p.source_type, SourceType::Gitlab);
        assert_eq!(p.url, "https://gitlab.com/owner/repo.git");
    }

    #[test]
    fn test_local_absolute_path() {
        let p = parse_source("/tmp/my-skill");
        assert_eq!(p.source_type, SourceType::Local);
        assert!(p.local_path.is_some());
    }

    #[test]
    fn test_local_relative_path() {
        let p = parse_source("./my-skill");
        assert_eq!(p.source_type, SourceType::Local);
        assert!(p.local_path.is_some());
    }

    #[test]
    fn test_local_dot() {
        let p = parse_source(".");
        assert_eq!(p.source_type, SourceType::Local);
    }

    #[test]
    fn test_github_tree_with_path() {
        let p = parse_source("https://github.com/owner/repo/tree/main/skills/my-skill");
        assert_eq!(p.source_type, SourceType::Github);
        assert_eq!(p.url, "https://github.com/owner/repo.git");
        assert_eq!(p.ref_branch.as_deref(), Some("main"));
        assert_eq!(p.subpath.as_deref(), Some("skills/my-skill"));
    }

    #[test]
    fn test_github_tree_branch_only() {
        let p = parse_source("https://github.com/owner/repo/tree/develop");
        assert_eq!(p.source_type, SourceType::Github);
        assert_eq!(p.ref_branch.as_deref(), Some("develop"));
        assert!(p.subpath.is_none());
    }

    #[test]
    fn test_github_repo_url() {
        let p = parse_source("https://github.com/owner/repo");
        assert_eq!(p.source_type, SourceType::Github);
        assert_eq!(p.url, "https://github.com/owner/repo.git");
    }

    #[test]
    fn test_github_repo_url_with_git_suffix() {
        let p = parse_source("https://github.com/owner/repo.git");
        assert_eq!(p.source_type, SourceType::Github);
        assert_eq!(p.url, "https://github.com/owner/repo.git");
    }

    #[test]
    fn test_gitlab_tree_with_path() {
        let p = parse_source("https://gitlab.com/group/repo/-/tree/main/skills/s");
        assert_eq!(p.source_type, SourceType::Gitlab);
        assert_eq!(p.ref_branch.as_deref(), Some("main"));
        assert_eq!(p.subpath.as_deref(), Some("skills/s"));
    }

    #[test]
    fn test_gitlab_tree_branch_only() {
        let p = parse_source("https://gitlab.com/group/repo/-/tree/main");
        assert_eq!(p.source_type, SourceType::Gitlab);
        assert_eq!(p.ref_branch.as_deref(), Some("main"));
    }

    #[test]
    fn test_gitlab_repo_url() {
        let p = parse_source("https://gitlab.com/group/subgroup/repo");
        assert_eq!(p.source_type, SourceType::Gitlab);
        assert_eq!(p.url, "https://gitlab.com/group/subgroup/repo.git");
    }

    #[test]
    fn test_github_shorthand_at_skill() {
        let p = parse_source("owner/repo@my-skill");
        assert_eq!(p.source_type, SourceType::Github);
        assert_eq!(p.skill_filter.as_deref(), Some("my-skill"));
    }

    #[test]
    fn test_github_shorthand() {
        let p = parse_source("owner/repo");
        assert_eq!(p.source_type, SourceType::Github);
        assert_eq!(p.url, "https://github.com/owner/repo.git");
    }

    #[test]
    fn test_github_shorthand_with_subpath() {
        let p = parse_source("owner/repo/skills/my-skill");
        assert_eq!(p.source_type, SourceType::Github);
        assert_eq!(p.subpath.as_deref(), Some("skills/my-skill"));
    }

    #[test]
    fn test_well_known_url() {
        let p = parse_source("https://mintlify.com/docs");
        assert_eq!(p.source_type, SourceType::WellKnown);
    }

    #[test]
    fn test_git_url_fallback() {
        let p = parse_source("git@bitbucket.org:owner/repo.git");
        assert_eq!(p.source_type, SourceType::Git);
    }

    #[test]
    fn test_sanitize_subpath_valid() {
        assert!(sanitize_subpath("skills/my-skill").is_ok());
    }

    #[test]
    fn test_sanitize_subpath_traversal() {
        assert!(sanitize_subpath("../etc/passwd").is_err());
        assert!(sanitize_subpath("skills/../../etc").is_err());
    }

    #[test]
    fn test_get_owner_repo_github() {
        let p = parse_source("owner/repo");
        assert_eq!(get_owner_repo(&p).as_deref(), Some("owner/repo"));
    }

    #[test]
    fn test_get_owner_repo_local() {
        let p = parse_source("./local");
        assert!(get_owner_repo(&p).is_none());
    }

    #[test]
    fn test_get_owner_repo_ssh() {
        let p = ParsedSource {
            source_type: SourceType::Git,
            url: "git@github.com:owner/repo.git".to_string(),
            ..Default::default()
        };
        assert_eq!(get_owner_repo(&p).as_deref(), Some("owner/repo"));
    }
}
