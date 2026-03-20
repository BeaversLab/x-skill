/// Shared test utilities for integration and unit tests.
use std::path::PathBuf;

/// Build a CLI invocation for testing.
pub fn cargo_bin() -> assert_cmd::Command {
    assert_cmd::Command::cargo_bin("x-skill").expect("binary not found")
}

/// Strip ANSI escape codes from output.
pub fn strip_ansi(input: &str) -> String {
    let re = regex::Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]").unwrap();
    re.replace_all(input, "").to_string()
}

/// Create a temporary directory with a SKILL.md file.
pub fn create_skill_dir(name: &str, description: &str) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let skill_dir = tmp.path().join(name);
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        format!("---\nname: {name}\ndescription: {description}\n---\n# {name}\n"),
    )
    .unwrap();
    (tmp, skill_dir)
}
