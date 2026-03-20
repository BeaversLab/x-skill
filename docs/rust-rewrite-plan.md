# x-skill Rust 重写实施计划

> 基于 x-skill-js (TypeScript/Node.js) 的 Rust 重写计划
> 目标目录：`/Users/marco/Documents/git/github.com/BeaversLab/x-skill/`

---

## 需求重述

将 **x-skill-js**（TypeScript/Node.js CLI 工具）重写为 Rust。该工具管理 AI 编码代理技能，支持 43+ 平台，功能包括：

- 多来源技能安装（GitHub, GitLab, Git URL, 本地路径, Well-known endpoints）
- Symlink 和 Copy 两种安装模式
- Lock 文件管理（全局 + 项目双系统）
- 更新检查和批量更新
- 交互式技能发现和搜索
- 跨平台支持（macOS, Linux, Windows）

---

## 文档对照检查

根据 `x-skill-js/docs/` 目录中的 5 份文档，确保完整覆盖：

### 来自 `architecture.md`
- ✅ 核心模块划分、安装流程
- ✅ **Providers 层架构**（`providers/` 目录的扩展机制）

### 来自 `commands.md`
- ✅ 所有命令及其选项
- ✅ **环境变量完整列表**

### 来自 `source-types.md`
- ✅ **别名系统**（Aliases）
- ✅ **前缀解析**（github:/gitlab:）
- ✅ **解析优先级**

### 来自 `api-reference.md`
- ✅ **安全审计 API**（并行请求，3s 超时）
- ✅ **Well-known 协议完整实现**（RFC 8615）
- ✅ **GitHub Token 获取优先级**

### 来自 `development.md`
- ✅ **测试辅助工具**
- ✅ **Universal Agent 机制**

---

## 项目结构

```
x-skill/
├── Cargo.toml
├── README.md
├── .gitignore
├── src/
│   ├── main.rs              # 入口，命令路由
│   ├── cli.rs               # Clap CLI 定义
│   ├── types.rs             # 核心类型
│   ├── constants.rs         # 常量（AGENTS_DIR, URLs）
│   ├── agents.rs            # 43+ Agent 配置
│   ├── source_parser.rs     # 来源解析（含别名、前缀、优先级）
│   ├── git.rs               # Git 克隆操作
│   ├── skills.rs            # 技能发现（按文档规定的扫描顺序）
│   ├── installer.rs         # 安装逻辑
│   ├── skill_lock.rs        # 全局 Lock（~/.agents/.skill-lock.json）
│   ├── local_lock.rs        # 项目 Lock（./skills-lock.json）
│   ├── telemetry.rs         # 遥测（fire-and-forget）
│   ├── http.rs              # HTTP 客户端工具
│   ├── output.rs            # 终端输出格式化
│   ├── test_utils.rs        # 测试辅助工具
│   ├── plugin_manifest.rs   # Claude Code 插件清单发现
│   ├── prompts/
│   │   └── search_multiselect.rs  # 交互式多选
│   ├── providers/           # Provider 层
│   │   ├── mod.rs
│   │   ├── types.rs         # HostProvider trait
│   │   ├── registry.rs      # Provider 注册表
│   │   └── wellknown.rs     # RFC 8615 实现
│   └── commands/
│       ├── mod.rs
│       ├── add.rs           # 含安全审计 API 并行请求
│       ├── remove.rs
│       ├── list.rs
│       ├── find.rs
│       ├── check.rs
│       ├── update.rs
│       ├── init.rs
│       ├── sync.rs          # experimental_sync
│       └── install.rs       # experimental_install
├── tests/
│   ├── test_source_parser.rs
│   ├── test_skills.rs
│   ├── test_installer.rs
│   └── ...
└── .github/
    └── workflows/
        ├── ci.yml
        └── release.yml
```

---

## 依赖配置（Cargo.toml）

```toml
[package]
name = "x-skill"
version = "0.1.0"
edition = "2021"
description = "The open agent skills ecosystem"
license = "MIT"

[dependencies]
# CLI
clap = { version = "4", features = ["derive", "color"] }

# Async runtime
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# HTTP client
reqwest = { version = "0.12", features = ["json"] }

# Git operations
git2 = "0.19"

# Terminal output
colored = "2"

# Interactive prompts
dialoguer = "0.11"

# Error handling
thiserror = "1"
anyhow = "1"

# Cross-platform directories
dirs = "5"

# Hashing
sha2 = "0.10"

# File system
tempfile = "3"
walkdir = "2"

# Regex
regex = "1"

# Lazy static
once_cell = "1"

[dev-dependencies]
# Testing
assert_cmd = "2"
predicates = "3"
```

---

## 核心类型定义（src/types.rs）

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 支持的 Agent 类型（43+）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    Amp,
    Antigravity,
    Augment,
    ClaudeCode,
    Openclaw,
    Cline,
    Codebuddy,
    Codex,
    CommandCode,
    Continue,
    Cortex,
    Crush,
    Cursor,
    Droid,
    GeminiCli,
    GithubCopilot,
    Goose,
    IflowCli,
    Junie,
    Kilo,
    KimiCli,
    KiroCli,
    Kode,
    Mcpjam,
    MistralVibe,
    Mux,
    Neovate,
    Opencode,
    Openhands,
    Pi,
    Qoder,
    QwenCode,
    Replit,
    Roo,
    Trae,
    TraeCn,
    Warp,
    Windsurf,
    Zencoder,
    Pochi,
    Adal,
    Universal,
}

/// 技能定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Agent 配置
pub struct AgentConfig {
    pub name: String,
    pub display_name: String,
    pub skills_dir: String,
    pub global_skills_dir: Option<String>,
    pub detect_installed: fn() -> bool,
    pub show_in_universal_list: bool,
}

/// 解析后的来源
#[derive(Debug, Clone)]
pub struct ParsedSource {
    pub source_type: SourceType,
    pub url: String,
    pub subpath: Option<String>,
    pub local_path: Option<String>,
    pub ref_branch: Option<String>,
    pub skill_filter: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    Github,
    Gitlab,
    Git,
    Local,
    WellKnown,
}

/// 远程技能（来自 Provider）
#[derive(Debug, Clone)]
pub struct RemoteSkill {
    pub name: String,
    pub description: String,
    pub content: String,
    pub install_name: String,
    pub source_url: String,
    pub provider_id: String,
    pub source_identifier: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// 全局 Lock 条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLockEntry {
    pub source: String,
    pub source_type: String,
    pub source_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_path: Option<String>,
    pub skill_folder_hash: String,
    pub installed_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_name: Option<String>,
}

/// 全局 Lock 文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLockFile {
    pub version: u32,
    pub skills: HashMap<String, SkillLockEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dismissed: Option<DismissedState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_selected_agents: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DismissedState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub find_skills_prompt: Option<bool>,
}

/// 项目 Lock 条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSkillLockEntry {
    pub source: String,
    pub source_type: String,
    pub computed_hash: String,
}

/// 项目 Lock 文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSkillLockFile {
    pub version: u32,
    pub skills: HashMap<String, LocalSkillLockEntry>,
}
```

---

## Provider 接口（src/providers/types.rs）

```rust
use async_trait::async_trait;
use crate::types::RemoteSkill;

/// Provider 匹配结果
pub struct ProviderMatch {
    pub matches: bool,
    pub source_identifier: Option<String>,
}

/// Host Provider trait（可扩展）
#[async_trait]
pub trait HostProvider: Send + Sync {
    /// Provider 唯一标识
    fn id(&self) -> &str;

    /// 显示名称
    fn display_name(&self) -> &str;

    /// 检查 URL 是否匹配此 Provider
    fn match_url(&self, url: &str) -> Option<ProviderMatch>;

    /// 获取技能
    async fn fetch_skill(&self, url: &str) -> anyhow::Result<Option<RemoteSkill>>;

    /// 获取所有技能
    async fn fetch_all_skills(&self, url: &str) -> anyhow::Result<Vec<RemoteSkill>>;

    /// 转换为原始 URL
    fn to_raw_url(&self, url: &str) -> String;

    /// 获取来源标识符
    fn source_identifier(&self, url: &str) -> String;
}
```

---

## 来源解析优先级（src/source_parser.rs）

```rust
/// 解析优先级（按文档 flowchart）
///
/// 1. 别名匹配 → GitHub Shorthand
/// 2. 前缀匹配（github:/gitlab:）→ GitHub/GitLab 处理
/// 3. 本地路径检测 → type: 'local'
/// 4. GitHub Tree URL → type: 'github'
/// 5. GitLab Tree URL → type: 'gitlab'
/// 6. GitLab 仓库 URL → type: 'gitlab'
/// 7. GitHub Shorthand（含 @skill）→ type: 'github'
/// 8. Well-known URL → type: 'well-known'
/// 9. 通用 Git URL → type: 'git'

pub fn parse_source(input: &str) -> ParsedSource {
    // 1. 别名替换
    let input = apply_aliases(input);

    // 2. 前缀处理
    if let Some(parsed) = parse_prefix(input) {
        return parsed;
    }

    // 3. 本地路径
    if is_local_path(input) {
        return ParsedSource {
            source_type: SourceType::Local,
            url: canonicalize_path(input),
            local_path: Some(canonicalize_path(input)),
            ..Default::default()
        };
    }

    // 4-9. 其他解析逻辑...
}
```

---

## 实施阶段

### Phase 1: 项目脚手架 & 核心类型
**预估工时：2-3 小时**

- [ ] 初始化 Cargo 项目
- [ ] 配置 `Cargo.toml` 依赖
- [ ] 定义核心类型（`types.rs`）
- [ ] 定义常量（`constants.rs`）

### Phase 2: Agent 配置系统
**预估工时：2-3 小时**

- [ ] 实现 43+ agent 配置（与 `agents.ts` 完全对应）
- [ ] 实现 `detect_installed_agents()`
- [ ] 实现 `get_universal_agents()`, `get_non_universal_agents()`
- [ ] 处理环境变量：`CODEX_HOME`, `CLAUDE_CONFIG_DIR`
- [ ] OpenClaw 多路径检测逻辑

### Phase 3: 来源解析（完整实现）
**预估工时：4-5 小时**

- [ ] 别名系统
- [ ] 前缀解析（`github:`, `gitlab:`）
- [ ] 本地路径检测（绝对路径、`./`, `../`, Windows 盘符）
- [ ] GitHub Tree URL 解析
- [ ] GitLab Tree URL 解析（含子组）
- [ ] GitHub Shorthand（`owner/repo`, `owner/repo/path`, `owner/repo@skill`）
- [ ] Well-known URL 检测
- [ ] 通用 Git URL fallback
- [ ] 安全性：`sanitize_name()`, `is_path_safe()`

### Phase 4: Git 操作
**预估工时：4-5 小时**

- [ ] 使用 `git2` 克隆仓库
- [ ] 临时目录管理
- [ ] 认证处理（SSH, HTTPS with token）
- [ ] 超时处理
- [ ] 错误类型定义（`GitCloneError`）
- [ ] 清理临时目录

### Phase 5: Providers 层
**预估工时：3-4 小时**

- [ ] `HostProvider` trait 定义
- [ ] `ProviderRegistry` 实现
- [ ] **WellKnownProvider**：完整 RFC 8615 实现
  - URL 尝试顺序（相对路径 → 根路径回退）
  - 索引文件校验（`skills` 数组、`name` 格式、`files` 包含 SKILL.md）
  - 文件并行获取
  - 路径穿越防护
- [ ] Provider 注册机制

### Phase 6: 技能发现
**预估工时：2-3 小时**

- [ ] 扫描顺序（按文档）：
  1. 仓库根目录（若含 SKILL.md）
  2. `skills/`, `skills/.curated/`, `skills/.experimental/`, `skills/.system/`
  3. 各代理标准路径
  4. 递归搜索（`--full-depth` 时）
- [ ] YAML frontmatter 解析（使用 `serde_yaml`）
- [ ] `metadata.internal` 过滤（`INSTALL_INTERNAL_SKILLS` 环境变量）
- [ ] 技能名称清洗

### Phase 7: 安装系统
**预估工时：4-5 小时**

- [ ] **Symlink 模式**（优先）：
  - `.agents/skills/` 存一份
  - 其他代理目录创建符号链接
- [ ] **Copy 模式**（fallback）：Windows 或不支持 symlink
- [ ] **Universal Agent** 优化：直接写入规范路径
- [ ] 路径安全校验
- [ ] 已存在技能检测

### Phase 8: Lock 文件管理（双系统）
**预估工时：3-4 小时**

**全局 Lock**（`~/.agents/.skill-lock.json` 或 `$XDG_STATE_HOME/x-skill/`）：
- [ ] 版本 3 格式
- [ ] `skill_folder_hash`（GitHub tree SHA）
- [ ] 支持 `check`/`update` 命令
- [ ] `dismissed` 状态
- [ ] `last_selected_agents`

**项目 Lock**（`./skills-lock.json`）：
- [ ] 版本 1 格式
- [ ] `computed_hash`（本地文件 SHA-256）
- [ ] 用于 `experimental_install`
- [ ] 技能按名称字母排序

### Phase 9: CLI 命令实现
**预估工时：10-14 小时**

| 命令 | 功能 | 关键点 |
|------|------|--------|
| `add` | 安装技能 | 并行安全审计 API（3s 超时） |
| `remove` | 移除技能 | 交互式或命令行指定 |
| `list` | 列出技能 | 支持 `--json` 输出 |
| `find` | 搜索技能 | skills.sh API 集成 |
| `check` | 检查更新 | GitHub Trees API 对比 |
| `update` | 批量更新 | 逐个重新安装 |
| `init` | 创建模板 | SKILL.md 生成 |
| `experimental_sync` | 同步 node_modules | 发现并安装 |
| `experimental_install` | 从 lock 还原 | 项目 lock 恢复 |

### Phase 10: 遥测系统
**预估工时：2-3 小时**

- [ ] **Fire-and-forget**：不阻塞主流程（`tokio::spawn`）
- [ ] **禁用条件**：`DISABLE_TELEMETRY` 或 `DO_NOT_TRACK`
- [ ] **跳过场景**：
  - 本地路径安装
  - 私有 GitHub 仓库
  - 无法判断是否私有
- [ ] **事件类型**：install, remove, check, update, find, experimental_sync
- [ ] CI 环境标记（`ci=1`）

### Phase 11: HTTP 服务
**预估工时：2-3 小时**

**GitHub API**：
- [ ] Token 获取优先级：`GITHUB_TOKEN` → `GH_TOKEN` → `gh auth token`
- [ ] Trees API（更新检测）
- [ ] Repos API（私有检测）
- [ ] Rate limit 处理

**安全审计 API**：
- [ ] 3s 超时（`tokio::time::timeout`）
- [ ] 并行请求（与用户交互同时进行）
- [ ] 静默失败

**技能搜索 API**：
- [ ] skills.sh 集成
- [ ] `SKILLS_API_URL` 环境变量覆盖

### Phase 12: 终端 UI
**预估工时：3-4 小时**

- [ ] ASCII Logo 显示（256 色灰度，兼容明暗背景）
- [ ] 彩色输出（`colored` crate）
- [ ] 进度动画（spinner）
- [ ] 交互式多选（fzf 风格，`dialoguer`）
- [ ] JSON 输出模式（`--json`）

### Phase 13: 测试 & CI/CD
**预估工时：4-5 小时**

**测试辅助工具**（`test_utils.rs`）：
- [ ] `run_cli_output(args, cwd)` — 运行 CLI 并捕获输出
- [ ] `strip_logo(output)` — 移除 ASCII logo
- [ ] `has_logo(output)` — 检测 logo 存在

**测试覆盖**：
- [ ] `test_source_parser.rs` — 来源解析
- [ ] `test_skills.rs` — 技能发现
- [ ] `test_installer.rs` — 安装逻辑
- [ ] `test_lock.rs` — Lock 文件
- [ ] 集成测试（CLI 命令）

**CI/CD**：
- [ ] GitHub Actions：多 OS（Ubuntu, macOS, Windows）
- [ ] Release 构建
- [ ] crates.io 发布

---

## 环境变量完整支持

| 变量 | 用途 | 默认值 |
|------|------|--------|
| `DISABLE_TELEMETRY` | 禁用遥测 | - |
| `DO_NOT_TRACK` | 同上（DNT 标准） | - |
| `INSTALL_INTERNAL_SKILLS` | 显示 internal 技能（设为 `1` 或 `true`） | - |
| `GITHUB_TOKEN` | GitHub API 认证（优先级 1） | - |
| `GH_TOKEN` | GitHub API 认证（优先级 2） | - |
| `XDG_STATE_HOME` | 全局 lock 路径 | `~/.local/state` |
| `CODEX_HOME` | Codex 技能目录 | `~/.codex` |
| `CLAUDE_CONFIG_DIR` | Claude 配置目录 | `~/.claude` |
| `SKILLS_API_URL` | 技能搜索 API 基地址 | `https://skills.sh` |

---

## 风险评估

| 风险 | 级别 | 缓解措施 |
|------|------|----------|
| Git 操作复杂性 | 高 | 使用 `git2` crate，优雅处理认证错误 |
| Windows symlink 支持 | 中 | 检测能力，fallback 到 copy 模式 |
| GitHub API rate limit | 中 | 支持 token 认证，实现缓存 |
| RFC 8615 协议细节 | 中 | 严格按文档实现 URL 尝试顺序和校验规则 |
| 43+ agent 配置迁移 | 低 | 直接移植，逐一验证 |
| Lock 文件版本兼容 | 低 | 实现版本迁移逻辑（v2 → v3） |
| 跨平台路径处理 | 中 | 使用 `dirs` crate，充分测试 |

---

## 预估总工时

| 阶段 | 工时 |
|------|------|
| Phase 1-2: 脚手架 & Agent | 4-6 小时 |
| Phase 3: 来源解析 | 4-5 小时 |
| Phase 4: Git 操作 | 4-5 小时 |
| Phase 5: Providers 层 | 3-4 小时 |
| Phase 6: 技能发现 | 2-3 小时 |
| Phase 7: 安装系统 | 4-5 小时 |
| Phase 8: Lock 双系统 | 3-4 小时 |
| Phase 9: CLI 命令 | 10-14 小时 |
| Phase 10-11: 遥测 & HTTP | 4-6 小时 |
| Phase 12: 终端 UI | 3-4 小时 |
| Phase 13: 测试 & CI/CD | 4-5 小时 |
| **总计** | **45-61 小时** |

---

## 关键设计决策

1. **本地安装不上报遥测**：当 `parse_source()` 返回 `type: Local` 时，跳过所有 `track()` 调用
2. **符号链接优先**：多目标代理安装时默认使用 symlink，失败自动 fallback 到 copy
3. **路径安全**：通过 `sanitize_name()` 和 `is_path_safe()` 防止路径穿越攻击
4. **遥测可选**：设置 `DISABLE_TELEMETRY` 或 `DO_NOT_TRACK` 环境变量即可关闭
5. **Fire-and-forget 遥测**：`track()` 不阻塞主流程
6. **安全审计并行**：与用户交互同时进行，3s 超时
7. **GitHub Token 优先级**：`GITHUB_TOKEN` → `GH_TOKEN` → `gh auth token`

---

## 参考文档

- [architecture.md](../x-skill-js/docs/architecture.md) - 系统架构总览
- [commands.md](../x-skill-js/docs/commands.md) - 命令参考
- [source-types.md](../x-skill-js/docs/source-types.md) - 来源类型详解
- [api-reference.md](../x-skill-js/docs/api-reference.md) - 服务端接口与数据结构
- [development.md](../x-skill-js/docs/development.md) - 二次开发指南
