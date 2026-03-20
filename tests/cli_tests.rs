use assert_cmd::Command;
use predicates::prelude::*;

fn x_skill() -> Command {
    Command::cargo_bin("x-skill").unwrap()
}

#[test]
fn test_no_args_shows_help() {
    // clap with arg_required_else_help exits with code 2 and prints help to stderr
    x_skill()
        .assert()
        .code(2)
        .stderr(predicate::str::contains("x-skill"));
}

#[test]
fn test_version_flag() {
    x_skill()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("x-skill"));
}

#[test]
fn test_help_flag() {
    x_skill()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("agent skills ecosystem"));
}

#[test]
fn test_init_creates_skill_md() {
    let tmp = tempfile::tempdir().unwrap();
    x_skill()
        .arg("init")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));

    assert!(tmp.path().join("SKILL.md").exists());

    let content = std::fs::read_to_string(tmp.path().join("SKILL.md")).unwrap();
    assert!(content.contains("---"));
    assert!(content.contains("name:"));
    assert!(content.contains("description:"));
}

#[test]
fn test_init_with_name() {
    let tmp = tempfile::tempdir().unwrap();
    x_skill()
        .args(["init", "my-test-skill"])
        .current_dir(tmp.path())
        .assert()
        .success();

    assert!(tmp.path().join("my-test-skill/SKILL.md").exists());
}

#[test]
fn test_init_fails_if_exists() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("SKILL.md"), "existing").unwrap();

    x_skill()
        .arg("init")
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_list_no_skills() {
    let tmp = tempfile::tempdir().unwrap();
    x_skill()
        .arg("list")
        .current_dir(tmp.path())
        .env("HOME", tmp.path())
        .assert()
        .success();
}

#[test]
fn test_add_local_nonexistent() {
    let tmp = tempfile::tempdir().unwrap();
    x_skill()
        .args(["add", "/nonexistent/path", "-y"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_add_local_skill() {
    let tmp = tempfile::tempdir().unwrap();
    let skill_dir = tmp.path().join("my-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: test-skill\ndescription: A test skill\n---\n# Test",
    )
    .unwrap();

    // List-only mode to verify discovery works
    x_skill()
        .args(["add", &skill_dir.to_string_lossy(), "-l"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("test-skill"));
}
