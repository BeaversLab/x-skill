# 来源类型详解

`source-parser.ts` 负责将用户输入的来源字符串解析为结构化的 `ParsedSource` 对象。理解不同来源类型对二次开发至关重要。

## ParsedSource 类型

```typescript
interface ParsedSource {
  type: 'github' | 'gitlab' | 'git' | 'local' | 'well-known';
  url: string;           // 规范化后的 URL 或解析后的绝对路径
  subpath?: string;      // 仓库内的子路径
  localPath?: string;    // 本地路径（仅 type === 'local'）
  ref?: string;          // Git 分支/标签
  skillFilter?: string;  // @skill 语法提取的技能名
}
```

## 解析优先级

`parseSource()` 按以下顺序检查输入：

```mermaid
flowchart TD
    Input["用户输入"]
    Alias["1. 别名匹配"]
    Prefix["2. 前缀匹配<br/>github: / gitlab:"]
    Local["3. 本地路径"]
    GHTree["4. GitHub tree URL"]
    GLTree["5. GitLab tree URL"]
    GLRepo["6. GitLab 仓库 URL"]
    GHShort["7. GitHub shorthand"]
    WellKnown["8. Well-known URL"]
    Fallback["9. 通用 Git URL"]

    Input --> Alias
    Alias -->|匹配| GHShort
    Alias -->|不匹配| Prefix
    Prefix -->|匹配| GHShort
    Prefix -->|不匹配| Local
    Local -->|是本地路径| LocalResult["type: 'local'"]
    Local -->|不是| GHTree
    GHTree -->|匹配| GHResult["type: 'github'"]
    GHTree -->|不匹配| GLTree
    GLTree -->|匹配| GLResult["type: 'gitlab'"]
    GLTree -->|不匹配| GLRepo
    GLRepo -->|匹配| GLResult
    GLRepo -->|不匹配| GHShort
    GHShort -->|匹配| GHResult
    GHShort -->|不匹配| WellKnown
    WellKnown -->|HTTP(S)| WKResult["type: 'well-known'"]
    WellKnown -->|不匹配| Fallback
    Fallback --> GitResult["type: 'git'"]
```

---

## 各类来源详解

### 1. 别名（Aliases）

预定义的缩写映射，在解析前先进行替换。

```typescript
// 示例别名
'coinbase/agentWallet' → 'coinbase/agentic-wallet-skills'
```

用法：

```bash
npx x-skill add coinbase/agentWallet
# 实际解析为 coinbase/agentic-wallet-skills
```

### 2. 前缀（Prefixes）

| 前缀 | 转换 |
|------|------|
| `github:owner/repo` | `https://github.com/owner/repo.git` |
| `gitlab:owner/repo` | `https://gitlab.com/owner/repo.git` |

```bash
npx x-skill add github:owner/repo
npx x-skill add gitlab:org/project
```

### 3. 本地路径

**判断条件**（满足任一即为本地路径）：

- 绝对路径（`isAbsolute(input)`）
- 以 `./` 开头
- 以 `../` 开头
- 就是 `.` 或 `..`
- Windows 盘符（如 `C:\`）

**返回结果**：

```typescript
{
  type: 'local',
  url: '/resolved/absolute/path',
  localPath: '/resolved/absolute/path'
}
```

**特点**：
- 路径通过 `resolve()` 转换为绝对路径
- 不触发 Git 克隆
- 不上报遥测数据
- `getOwnerRepo()` 返回 `null`

```bash
npx x-skill add ./my-skills
npx x-skill add ../shared-skills
npx x-skill add /absolute/path/to/skills
npx x-skill add .
```

### 4. GitHub Tree URL

匹配模式：`github.com/<owner>/<repo>/tree/<branch>/<path>`

```bash
npx x-skill add https://github.com/owner/repo/tree/main/skills/my-skill
```

解析结果：

```typescript
{
  type: 'github',
  url: 'https://github.com/owner/repo.git',
  ref: 'main',
  subpath: 'skills/my-skill'
}
```

### 5. GitLab Tree URL

匹配模式：`gitlab.com/<owner>/<repo>/-/tree/<branch>/<path>`

```bash
npx x-skill add https://gitlab.com/org/repo/-/tree/main/skills/my-skill
```

也支持 GitLab 子组（subgroups）。

### 6. GitLab 仓库 URL

```bash
npx x-skill add https://gitlab.com/org/project
```

### 7. GitHub Shorthand

最常用的格式，支持多种变体：

| 格式 | 说明 |
|------|------|
| `owner/repo` | 基本格式 |
| `owner/repo/path/to/skill` | 带子路径 |
| `owner/repo@skill-name` | `@` 指定技能名作为 filter |

```bash
npx x-skill add vercel-labs/agent-skills
npx x-skill add vercel-labs/agent-skills/skills/web-design
npx x-skill add vercel-labs/agent-skills@react-best-practices
```

对于 `owner/repo@skill` 格式，解析结果中 `skillFilter` 字段会被设置，在后续流程中合并到 `options.skill` 中。

### 8. Well-known URL（RFC 8615）

当输入是 HTTP(S) URL 且不匹配 GitHub/GitLab 模式时，尝试作为 well-known 来源处理。

```bash
npx x-skill add https://example.com
```

CLI 会请求 `https://example.com/.well-known/skills/index.json` 获取技能列表。

### 9. 通用 Git URL（Fallback）

不匹配以上任何模式时，作为通用 Git URL 处理。

```bash
npx x-skill add git@github.com:owner/repo.git
npx x-skill add https://custom-git.example.com/repo.git
```

---

## 安全性

### 路径安全

`installer.ts` 中的 `sanitizeName()` 函数对技能名进行清洗：

- 替换非法字符为连字符
- 移除路径穿越片段（`..`、`.`）
- 转换为 kebab-case

`isPathSafe()` 函数验证安装路径不会逃逸到目标目录之外。

### 私有仓库

- SSH URL（`git@...`）在 lock 文件中保留原始格式，不规范化为 HTTPS
- 私有仓库自动跳过遥测上报
- 支持 `GITHUB_TOKEN` / `GH_TOKEN` / `gh auth token` 进行认证

---

## 扩展新来源类型

如需支持新的来源格式：

1. 在 `src/source-parser.ts` 的 `parseSource()` 函数中添加新的匹配逻辑
2. 如果需要，在 `src/types.ts` 的 `ParsedSource.type` 中添加新类型
3. 在 `src/add.ts` 中处理新类型的安装逻辑
4. 在 `tests/source-parser.test.ts` 中添加测试用例
