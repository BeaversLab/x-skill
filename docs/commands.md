# 命令参考

## 概览

| 命令 | 别名 | 说明 |
|------|------|------|
| `x-skill add <source>` | `a`, `install`, `i` | 从仓库或本地路径安装技能 |
| `x-skill remove [skills]` | `rm`, `r` | 移除已安装的技能 |
| `x-skill list` | `ls` | 列出已安装的技能 |
| `x-skill find [query]` | `search`, `f`, `s` | 搜索技能 |
| `x-skill check` | | 检查技能更新 |
| `x-skill update` | `upgrade` | 更新所有技能至最新版本 |
| `x-skill init [name]` | | 创建新的 SKILL.md 模板 |
| `x-skill experimental_install` | | 从 skills-lock.json 还原技能 |
| `x-skill experimental_sync` | | 从 node_modules 同步技能 |

不带参数执行 `npx x-skill` 将显示 banner 和命令摘要。

---

## x-skill add

安装技能到编码代理目录。

```bash
npx x-skill add <source> [options]
```

### 来源格式

```bash
# GitHub shorthand
npx x-skill add owner/repo

# GitHub shorthand + 指定技能
npx x-skill add owner/repo@skill-name

# 完整 GitHub URL
npx x-skill add https://github.com/owner/repo

# GitHub tree URL（指定子目录）
npx x-skill add https://github.com/owner/repo/tree/main/skills/my-skill

# GitLab URL
npx x-skill add https://gitlab.com/org/repo

# Git SSH URL
npx x-skill add git@github.com:owner/repo.git

# 本地路径
npx x-skill add ./my-local-skills
npx x-skill add /absolute/path/to/skills

# Well-known URL（RFC 8615）
npx x-skill add https://example.com
```

### 选项

| 选项 | 说明 |
|------|------|
| `-g, --global` | 安装到全局目录（~/ 下）而非项目目录 |
| `-a, --agent <agents...>` | 指定目标代理，如 `claude-code cursor`；`'*'` 表示所有 |
| `-s, --skill <skills...>` | 按名称安装特定技能；`'*'` 表示所有 |
| `-l, --list` | 仅列出仓库中的可用技能，不安装 |
| `--copy` | 复制文件而非创建符号链接 |
| `-y, --yes` | 跳过所有交互确认 |
| `--all` | 等同于 `--skill '*' --agent '*' -y` |
| `--full-depth` | 即使根目录有 SKILL.md，也搜索所有子目录 |

### 示例

```bash
# 列出仓库中的技能
npx x-skill add owner/repo --list

# 安装特定技能到特定代理
npx x-skill add owner/repo --skill my-skill -a claude-code -g -y

# 安装所有技能到所有代理
npx x-skill add owner/repo --all

# 从本地目录安装（不会触发遥测上报）
npx x-skill add ./my-skills -y
```

### 安装范围

| 范围 | 标志 | 位置 | 适用场景 |
|------|------|------|---------|
| 项目 | 默认 | `./<agent>/skills/` | 随项目提交，团队共享 |
| 全局 | `-g` | `~/<agent>/skills/` | 跨项目可用 |

---

## x-skill remove

移除已安装的技能。不带参数时进入交互选择模式。

```bash
npx x-skill remove [skills...] [options]
```

### 选项

| 选项 | 说明 |
|------|------|
| `-g, --global` | 从全局范围移除 |
| `-a, --agent <agents...>` | 从指定代理移除；`'*'` 表示所有 |
| `-s, --skill <skills...>` | 指定要移除的技能；`'*'` 表示所有 |
| `-y, --yes` | 跳过确认 |
| `--all` | 等同于 `--skill '*' --agent '*' -y` |

### 示例

```bash
# 交互式选择移除
npx x-skill remove

# 按名称移除
npx x-skill remove my-skill

# 从全局范围移除所有
npx x-skill remove --all -g
```

---

## x-skill list

列出已安装的技能。

```bash
npx x-skill list [options]
```

### 选项

| 选项 | 说明 |
|------|------|
| `-g, --global` | 列出全局技能（默认列出项目技能） |
| `-a, --agent <agents...>` | 按代理过滤 |
| `--json` | 以 JSON 格式输出，无 ANSI 色码 |

### 示例

```bash
npx x-skill list          # 项目技能
npx x-skill ls -g         # 全局技能
npx x-skill ls --json     # JSON 输出
npx x-skill ls -a cursor  # 仅 Cursor 代理的技能
```

---

## x-skill find

通过 skills.sh API 搜索技能。

```bash
npx x-skill find [query]
```

- 不带 query：进入交互式 fzf 风格搜索
- 带 query：直接搜索并输出结果

### 示例

```bash
npx x-skill find                  # 交互搜索
npx x-skill find react testing    # 关键词搜索
```

---

## x-skill check / update

检查和更新已安装的技能。

```bash
# 检查是否有可用更新
npx x-skill check

# 更新所有技能到最新版本
npx x-skill update
```

更新机制：读取全局 lock 文件中的 `skillFolderHash`，通过 GitHub Trees API 获取最新 hash 进行对比。仅支持从 GitHub 安装的技能。

---

## x-skill init

创建新的 SKILL.md 模板。

```bash
# 在当前目录创建 SKILL.md
npx x-skill init

# 创建 my-skill/SKILL.md 子目录
npx x-skill init my-skill
```

---

## x-skill experimental_install

从项目的 `skills-lock.json` 文件还原技能。仅安装到 `.agents/skills/`（universal agents）。

```bash
npx x-skill experimental_install
```

---

## x-skill experimental_sync

从 `node_modules` 中发现并同步技能包。

```bash
npx x-skill experimental_sync [options]
```

### 选项

| 选项 | 说明 |
|------|------|
| `-a, --agent <agents...>` | 指定目标代理 |
| `-y, --yes` | 跳过确认 |
| `--force` | 强制重新安装（即使未变化） |

---

## 环境变量

| 变量 | 说明 |
|------|------|
| `DISABLE_TELEMETRY` | 设置后禁用匿名遥测 |
| `DO_NOT_TRACK` | 同上，DNT 标准 |
| `INSTALL_INTERNAL_SKILLS` | 设为 `1` 或 `true` 显示并安装标记为 `internal: true` 的技能 |
| `GITHUB_TOKEN` / `GH_TOKEN` | GitHub API 认证，提高 rate limit |
| `XDG_STATE_HOME` | 自定义全局 lock 文件存储路径 |
| `CODEX_HOME` | 自定义 Codex 技能目录 |
| `CLAUDE_CONFIG_DIR` | 自定义 Claude Code 配置目录 |
