# 二次开发指南

## 开发环境

### 前置要求

- **Node.js** >= 18
- **pnpm** (推荐使用 `packageManager` 字段指定的版本: pnpm@10.17.1)

### 初始化

```bash
git clone <repo-url>
cd x-skill
pnpm install
```

### 常用命令

```bash
# 构建（生成 dist/）
pnpm build

# 开发模式运行（直接执行 src/cli.ts）
pnpm dev add vercel-labs/agent-skills --list
pnpm dev check
pnpm dev init my-skill

# 运行所有测试
pnpm test

# 运行指定测试文件
pnpm test tests/sanitize-name.test.ts

# TypeScript 类型检查
pnpm type-check

# 代码格式化
pnpm format

# 检查格式化
pnpm format:check
```

### 构建产物

- 入口：`bin/cli.mjs` → 加载 `dist/cli.mjs`
- 使用 [obuild](https://github.com/nicolo-ribaudo/obuild) 构建，零配置 TypeScript 到 ESM 的编译

---

## 添加新的 Agent 支持

所有代理定义在 `src/agents.ts` 中。

### 步骤

1. 在 `src/types.ts` 的 `AgentType` 联合类型中添加新代理 ID：

```typescript
export type AgentType =
  | 'amp'
  | 'claude-code'
  // ...
  | 'my-new-agent'; // 新增
```

2. 在 `src/agents.ts` 的 `agents` 对象中添加配置：

```typescript
'my-new-agent': {
  name: 'my-new-agent',
  displayName: 'My New Agent',
  skillsDir: '.my-agent/skills',
  globalSkillsDir: join(home, '.my-agent', 'skills'),
  detectInstalled: async () => existsSync(join(home, '.my-agent')),
},
```

### AgentConfig 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `string` | Agent 内部标识符（与 `AgentType` 一致） |
| `displayName` | `string` | 用户界面显示名称 |
| `skillsDir` | `string` | 项目级技能相对路径（如 `.cursor/skills`） |
| `globalSkillsDir` | `string \| undefined` | 全局技能绝对路径；`undefined` 表示不支持全局安装 |
| `detectInstalled` | `() => Promise<boolean>` | 检测该代理是否已安装 |
| `showInUniversalList` | `boolean` | 是否在 Universal Agents 列表中显示（默认 `true`） |

### Universal Agent 机制

共享 `.agents/skills/` 路径的代理被视为 Universal Agent。在 symlink 模式下，文件直接写入规范路径，无需额外链接。判断逻辑在 `agents.ts` 的 `isUniversalAgent()` 中。

3. 运行验证脚本：

```bash
pnpm run -C scripts validate-agents.ts
pnpm run -C scripts sync-agents.ts
```

---

## 添加新的 Provider

Provider 负责从不同来源获取技能。位于 `src/providers/` 目录。

### Provider 接口

```typescript
// src/providers/types.ts
export interface HostProvider {
  id: string;
  match(url: string): ProviderMatch | null;
  fetchAllSkills(url: string): Promise<WellKnownSkill[]>;
  getSourceIdentifier(url: string): string;
}
```

### 添加步骤

1. 在 `src/providers/` 下创建新文件（如 `my-provider.ts`）
2. 实现 `HostProvider` 接口
3. 在 `src/providers/registry.ts` 中注册
4. 在 `src/providers/index.ts` 中导出

---

## 添加新命令

### 步骤

1. 在 `src/` 下创建命令模块（如 `src/my-command.ts`），导出主函数
2. 在 `src/cli.ts` 的 `main()` 函数 `switch` 中添加路由：

```typescript
case 'my-command':
case 'mc': // 别名（可选）
  await runMyCommand(restArgs);
  break;
```

3. 在 `showHelp()` 中添加帮助信息
4. 在 `showBanner()` 中添加（如果该命令需要在启动时展示）
5. 编写测试文件

---

## 技能格式规范

每个技能是一个包含 `SKILL.md` 的目录，SKILL.md 需要 YAML frontmatter：

```markdown
---
name: my-skill
description: 技能功能说明，也是触发条件描述
---

# My Skill

代理在该技能激活时遵循的指令。
```

### 必填字段

- `name`：唯一标识符（小写、连字符分隔）
- `description`：简要功能说明

### 可选字段

- `metadata.internal`：设为 `true` 隐藏于正常发现流程

### 技能发现路径

`skills.ts` 中的 `discoverSkills()` 按以下顺序扫描：

1. 仓库根目录（若含 `SKILL.md`）
2. `skills/`、`skills/.curated/`、`skills/.experimental/`、`skills/.system/`
3. 各代理的标准路径（`.agents/skills/`、`.claude/skills/` 等）
4. 如果以上都未找到，执行递归搜索

---

## Lock 文件机制

### 全局 Lock（`~/.agents/.skill-lock.json`）

版本 3 格式。记录全局安装的技能，用于 `check` 和 `update`。

```json
{
  "version": 3,
  "skills": {
    "skill-name": {
      "source": "owner/repo",
      "sourceType": "github",
      "sourceUrl": "https://github.com/owner/repo.git",
      "skillPath": "skills/skill-name/SKILL.md",
      "skillFolderHash": "<GitHub tree SHA>",
      "installedAt": "2025-01-01T00:00:00.000Z",
      "updatedAt": "2025-01-01T00:00:00.000Z"
    }
  },
  "dismissed": {},
  "lastSelectedAgents": ["claude-code", "cursor"]
}
```

XDG 环境：当 `$XDG_STATE_HOME` 设置时，路径为 `$XDG_STATE_HOME/x-skill/.skill-lock.json`。

### 项目 Lock（`./skills-lock.json`）

版本 1 格式。记录项目级技能，设计为可提交到 Git。

```json
{
  "version": 1,
  "skills": {
    "skill-name": {
      "source": "owner/repo",
      "sourceType": "github",
      "computedHash": "<SHA-256 of file contents>"
    }
  }
}
```

无时间戳字段，减少合并冲突。技能按名称字母排序。

---

## Telemetry 机制

> 完整的接口请求/响应格式和所有事件的数据结构详见 [服务端接口与数据结构](./api-reference.md)。

### 工作方式

- 端点：`https://add-skill.vercel.sh/t`（事件），`https://add-skill.vercel.sh/audit`（安全审计）
- 发送方式：fire-and-forget（`fetch().catch(() => {})`），不阻塞主流程
- 事件类型：`install`、`remove`、`check`、`update`、`find`、`experimental_sync`

### 禁用方式

设置以下任一环境变量：

```bash
export DISABLE_TELEMETRY=1
# 或
export DO_NOT_TRACK=1
```

### 不上报的场景

| 场景 | 原因 |
|------|------|
| 本地路径安装 | `getOwnerRepo()` 返回 `null`，跳过 `track()` |
| 私有 GitHub 仓库 | `isRepoPrivate()` 返回 `true`，跳过 `track()` |
| CI 环境 | 遥测仍发送，但附加 `ci=1` 标记 |
| 环境变量禁用 | `isEnabled()` 返回 `false`，直接返回 |

---

## 测试

使用 [Vitest](https://vitest.dev/) 作为测试框架。

### 测试结构

```
src/
├── cli.test.ts         # CLI 入口测试
├── add.test.ts         # add 命令测试
├── list.test.ts        # list 命令测试
├── init.test.ts        # init 命令测试
├── remove.test.ts      # remove 命令测试
├── add-prompt.test.ts  # add 命令提示测试
└── source-parser.test.ts

tests/
├── sanitize-name.test.ts
├── skill-matching.test.ts
├── source-parser.test.ts
├── installer-symlink.test.ts
├── list-installed.test.ts
├── xdg-config-paths.test.ts
├── sync.test.ts
├── local-lock.test.ts
├── dist.test.ts
└── ...
```

### 测试辅助

`src/test-utils.ts` 提供：
- `runCliOutput(args, cwd?)` — 同步运行 CLI 并捕获输出
- `stripLogo(output)` — 移除 ASCII logo
- `hasLogo(output)` — 检测输出中是否包含 logo

### 运行测试

```bash
pnpm test              # watch 模式
pnpm test --run        # 单次运行
pnpm test src/cli.test.ts  # 指定文件
```

---

## 发布

```bash
# 1. 更新 package.json 中的版本号
# 2. 构建
pnpm build
# 3. 发布到 npm
npm publish
```

快照版本：

```bash
pnpm publish:snapshot
```

---

## 代码风格

- 使用 Prettier 格式化，提交前通过 husky + lint-staged 自动执行
- 手动运行：`pnpm format`
- 检查：`pnpm format:check`
