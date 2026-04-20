# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## Project Overview

x-skill is a CLI tool for the open agent skills ecosystem. It manages skill packages for 42+ AI coding agents (Codex, Cursor, Codex, Gemini CLI, etc.). Skills are defined in `SKILL.md` files with YAML frontmatter and can be installed from GitHub, GitLab, local directories, or well-known URLs.

## Build and Development Commands

```bash
# Build
cargo build                    # Debug build
cargo build --release          # Release build (LTO enabled)

# Run
cargo run -- <command>         # Run with command
cargo run -- add owner/repo --list  # Example: list skills from a repo

# Test
cargo test                     # Run all tests
cargo test test_init_creates   # Run specific test

# Lint and Format
cargo fmt --check              # Check formatting
cargo fmt                      # Format code
cargo clippy -- -D warnings    # Lint (CI enforces zero warnings)
```

## Architecture

### Core Module Structure

| Module | Purpose |
|--------|---------|
| `main.rs` | Entry point, command routing via clap |
| `cli.rs` | CLI definition with clap derive macros |
| `commands/` | Individual command implementations (`add`, `remove`, `list`, `find`, `check`, `update`, `init`, `sync`, `install`) |
| `agents.rs` | Agent configurations for 42+ coding tools |
| `types.rs` | Core types: `AgentType`, `AgentConfig`, `Skill`, `ParsedSource`, lock file structures |
| `source_parser.rs` | Source string parsing (GitHub shorthand, URLs, local paths) |
| `skills.rs` | Skill discovery from directories |
| `installer.rs` | Skill installation (symlink/copy modes) |
| `skill_lock.rs` | Global lock file (`~/.agents/.skill-lock.json`) |
| `local_lock.rs` | Project lock file (`./skills-lock.json`) |
| `git.rs` | Git repository cloning via git2 |
| `providers/` | Remote skill providers (well-known URL RFC 8615) |
| `prompts/` | Interactive terminal prompts |

### Key Concepts

- **Universal Agents**: Agents sharing `.agents/skills/` path (Codex, Cursor, Codex, etc.). Files written once, no symlinks needed.
- **Lock Files**: Global lock tracks installed skills with GitHub tree SHA for update detection; project lock uses content SHA-256 for reproducibility.
- **Source Types**: `github`, `gitlab`, `git`, `local`, `well-known` - parsed from user input strings.

### Data Flow (add command)

1. `source_parser::parse_source()` resolves input to `ParsedSource`
2. Remote sources: `git::clone_repo()` to temp directory
3. `skills::discover_skills()` scans for `SKILL.md` files
4. User selects skills and agents (interactive or via flags)
5. `installer::install_skill()` creates symlinks or copies files
6. Lock files updated with skill metadata

## Adding a New Agent

1. Add variant to `AgentType` enum in `types.rs`
2. Add `AgentConfig` entry in `agents.rs` `build_agent_configs()`:
   - `skills_dir`: Relative project path (e.g., `.cursor/skills`)
   - `global_skills_dir`: Absolute global path using `home()` or `config_home()`
   - `detect`: `DetectStrategy::DirExists(path)` or `AnyDirExists(paths)`
3. If `skills_dir == ".agents/skills"`, agent is automatically a Universal Agent

## Source Parsing Priority

`source_parser::parse_source()` checks in order:
1. Local paths (`./`, `../`, absolute paths)
2. GitHub tree URLs (`github.com/owner/repo/tree/branch/path`)
3. GitLab tree URLs (`gitlab.com/owner/repo/-/tree/branch/path`)
4. GitHub shorthand (`owner/repo`, `owner/repo@skill`)
5. Well-known URLs (HTTP(S) URLs)
6. Generic Git URLs (fallback)

## Skill File Format

```markdown
---
name: skill-name
description: Brief description for triggering
metadata:
  internal: true  # Optional: hide from normal discovery
---

# Skill Name

Instructions for the agent when this skill is active.
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `DISABLE_TELEMETRY` / `DO_NOT_TRACK` | Disable anonymous telemetry |
| `GITHUB_TOKEN` / `GH_TOKEN` | GitHub API authentication (higher rate limit) |
| `CODEX_HOME` | Custom Codex directory |
| `CLAUDE_CONFIG_DIR` | Custom Codex config directory |
| `XDG_STATE_HOME` | Custom global lock file location |

## Release Profile

`Cargo.toml` configures release builds with LTO, symbol stripping, and single codegen unit for smaller binaries. CI builds cross-platform releases on tags.
