# x-skill

[![CI](https://github.com/BeaversLab/x-skill/actions/workflows/ci.yml/badge.svg)](https://github.com/BeaversLab/x-skill/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

开放的 Agent Skills 生态系统 CLI 工具。为 42+ 种 AI 编码代理安装、管理和发现技能包。

## 支持的代理

支持以下编码代理（持续增加中）：

| 代理 | 项目路径 | 全局路径 |
|------|----------|----------|
| Claude Code | `.claude/skills` | `~/.claude/skills` |
| Cursor | `.agents/skills` | `~/.cursor/skills` |
| Codex | `.agents/skills` | `~/.codex/skills` |
| Gemini CLI | `.agents/skills` | `~/.gemini/skills` |
| Cline | `.agents/skills` | `~/.agents/skills` |
| Windsurf | `.windsurf/skills` | `~/.codeium/windsurf/skills` |
| Continue | `.continue/skills` | `~/.continue/skills` |
| Goose | `.goose/skills` | `~/.config/goose/skills` |
| OpenHands | `.openhands/skills` | `~/.openhands/skills` |
| ... | ... | ... |

完整列表请参阅 [src/agents.rs](./src/agents.rs)。

## 安装

### Homebrew Tap

```bash
brew tap beaverslab/tap
brew install x-skill
```

当前 Homebrew formula 会从 `BeaversLab/x-skill` 仓库源码构建，并自动安装所需的 Rust 构建依赖。

### 从源码构建

```bash
git clone https://github.com/BeaversLab/x-skill.git
cd x-skill
cargo build --release
```

编译后的二进制文件位于 `target/release/x-skill`。

### 预编译二进制

从 [Releases](https://github.com/BeaversLab/x-skill/releases) 页面下载对应平台的版本。

## 快速开始

```bash
# 从 GitHub 仓库安装技能
x-skill add owner/repo

# 列出仓库中的可用技能
x-skill add owner/repo --list

# 安装特定技能到特定代理
x-skill add owner/repo --skill my-skill --agent claude-code

# 全局安装（所有项目可用）
x-skill add owner/repo -g

# 列出已安装的技能
x-skill list

# 搜索技能
x-skill find react

# 检查更新
x-skill check

# 更新所有技能
x-skill update

# 创建新的技能模板
x-skill init my-skill
```

## 命令参考

### `x-skill add <source>`

从指定来源安装技能。

**来源格式：**

```bash
# GitHub shorthand
x-skill add owner/repo

# GitHub shorthand + 指定技能
x-skill add owner/repo@skill-name

# GitHub tree URL（指定子目录）
x-skill add https://github.com/owner/repo/tree/main/skills/my-skill

# GitLab URL
x-skill add https://gitlab.com/org/repo

# 本地路径
x-skill add ./my-skills

# Well-known URL（RFC 8615）
x-skill add https://example.com
```

**选项：**

| 选项 | 说明 |
|------|------|
| `-g, --global` | 安装到全局目录 |
| `-a, --agent <agents...>` | 指定目标代理 |
| `-s, --skill <skills...>` | 指定要安装的技能 |
| `-l, --list` | 仅列出可用技能，不安装 |
| `-y, --yes` | 跳过所有确认提示 |
| `--all` | 安装所有技能到所有代理 |
| `--copy` | 复制文件而非创建符号链接 |
| `--full-depth` | 递归搜索所有子目录 |

### `x-skill remove [skill]`

移除已安装的技能。

```bash
# 交互式选择
x-skill remove

# 按名称移除
x-skill remove my-skill

# 从全局范围移除
x-skill remove my-skill -g
```

### `x-skill list`

列出已安装的技能。

```bash
x-skill list           # 项目技能
x-skill list -g        # 全局技能
x-skill list --json    # JSON 格式输出
```

### `x-skill find [query]`

搜索技能。

```bash
x-skill find                # 交互式搜索
x-skill find react testing  # 关键词搜索
```

### `x-skill check` / `x-skill update`

检查和更新已安装的技能。

```bash
x-skill check   # 检查更新
x-skill update  # 更新所有技能
```

### `x-skill init [name]`

创建新的 SKILL.md 模板。

```bash
x-skill init           # 当前目录
x-skill init my-skill  # 创建 my-skill/SKILL.md
```

## 技能格式

技能是一个包含 `SKILL.md` 文件的目录：

```markdown
---
name: my-skill
description: 技能功能说明，也是触发条件描述
metadata:
  internal: false  # 可选：设为 true 隐藏于正常发现流程
---

# My Skill

代理在该技能激活时遵循的指令。
```

### 必填字段

- `name`：唯一标识符（小写、连字符分隔）
- `description`：简要功能说明

### 可选字段

- `metadata.internal`：设为 `true` 隐藏于正常发现流程

## 安装模式

| 模式 | 说明 | 适用场景 |
|------|------|---------|
| **Symlink** | 在 `.agents/skills/` 下存一份，其他代理目录创建符号链接 | 推荐，单一数据源 |
| **Copy** | 直接复制到每个代理目录 | Windows 或不支持符号链接时 |

对于 **Universal Agent**（共享 `.agents/skills/` 路径的代理），文件直接写入规范路径，无需符号链接。

## Lock 文件

### 全局 Lock (`~/.agents/.skill-lock.json`)

记录全局安装的技能，用于 `check` 和 `update` 命令。使用 GitHub Tree SHA 检测更新。

### 项目 Lock (`./skills-lock.json`)

记录项目级技能，可提交到 Git。使用内容 SHA-256 hash。

## 环境变量

| 变量 | 说明 |
|------|------|
| `DISABLE_TELEMETRY` | 禁用匿名遥测 |
| `DO_NOT_TRACK` | 同上，DNT 标准 |
| `GITHUB_TOKEN` / `GH_TOKEN` | GitHub API 认证，提高 rate limit |
| `CODEX_HOME` | 自定义 Codex 技能目录 |
| `CLAUDE_CONFIG_DIR` | 自定义 Claude Code 配置目录 |
| `XDG_STATE_HOME` | 自定义全局 lock 文件存储路径 |

## 开发

```bash
# 构建
cargo build

# 运行测试
cargo test

# 代码格式化
cargo fmt

# Lint 检查
cargo clippy -- -D warnings

# 运行
cargo run -- add owner/repo --list
```

## 许可证

[MIT](./LICENSE)
