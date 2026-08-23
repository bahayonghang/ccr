# 跨平台功能面统一

> 父任务：`08-22-react-migration`

## Goal

将 Settings / Profiles / Auth / MCP / Agents / Plugins / Commands 七个功能面的跨平台重复实现收敛为统一层，使「改一处行为只改一处」成立。

## 已有模式

仓库内已存在可用且已验证的统一模式，本任务扩展该模式，不新造架构。

**参照实现（SlashCommands，已完成统一）**

```
src/components/BaseSlashCommands.vue      507 行   统一实现
src/configs/slashCommands.ts                       每平台一个 config 对象
src/views/SlashCommandsView.vue            18 行   薄壳，传 claudeCodeConfig
src/views/GeminiSlashCommandsView.vue      27 行   薄壳，传 geminiConfig + hide-chrome
src/views/CodexSlashCommandsView.vue      229 行   薄壳
```

三个平台的视图层合计 274 行。平台差异通过 config 对象与少量 props（如 `hide-chrome`）表达。

**部分抽象（已存在，本任务负责收敛剩余重复到其上）**

| 文件 | 行数 |
|---|---|
| `src/views/generic/AgentsView.vue` | 725 |
| `src/views/generic/AgentDetailView.vue` | 481 |
| `src/views/generic/SystemPromptsView.vue` | 655 |
| `src/views/generic/PlatformMcpView.vue` | 407 |
| `src/views/generic/PlatformPluginsView.vue` | 367 |

## Scope

### 需统一的重复实现

| 功能面 | 文件 | 行数 | 收敛目标 |
|---|---|---|---|
| Settings | `ClaudeCodeSettingsView.vue` 1,325<br>`grok/GrokSettingsView.vue` 1,245<br>`CodexSettingsView.vue` 1,023<br>`OpenCodeSettingsView.vue` 330 | 3,923 | 新建 base + 4 个 config + 4 个薄壳 |
| Profiles | `CodexProfilesView.vue` 1,173<br>`grok/GrokProfilesView.vue` 1,078<br>`ClaudeCodeProfilesView.vue` 1,074 | 3,325 | 新建 base + 3 个 config + 3 个薄壳。可复用 `components/profiles/`（10 文件 4,040 行）共享层 |
| Auth | `ClaudeAuthView.vue` 1,179<br>`CodexAuthView.vue` 958<br>`grok/GrokAuthView.vue` 161 | 2,298 | 差异最大的一面，统一范围由差异普查决定 |
| Commands | `CommandsView.vue` 1,744<br>`OpenCodeCommandsView.vue` 346 | 2,090 | 新建 base + config + 薄壳 |
| MCP | `CodexMcpView.vue` 1,301<br>`OpenCodeMcpView.vue` 433 | 1,734 | 收敛到 `generic/PlatformMcpView.vue`（407 行） |
| Agents | `codex/CodexAgentsView.vue` 1,138<br>`OpenCodeAgentsView.vue` 442 | 1,580 | 收敛到 `generic/AgentsView.vue`（725 行） |
| Plugins | `PluginsView.vue` 426<br>`OpenCodePluginsView.vue` 296 | 722 | 收敛到 `generic/PlatformPluginsView.vue`（367 行） |
| 合计 | 20 个文件 | 15,672 | 预估 6,000–7,500 行 |

### 不在统一范围

| 文件 | 行数 | 原因 |
|---|---|---|
| `AppSettingsView.vue` | 1,399 | 应用级设置，非平台功能面 |
| `views/mcp/McpManagerView.vue` | 523 | 跨平台 MCP 统一管理器，本身即统一层 |
| `OpenCodeProvidersView.vue` | 577 | 单一实现，无重复 |
| SlashCommands 三个视图 | 274 | 已完成统一 |

## Requirements

- R1 差异普查先行：七个功能面逐平台枚举差异项，产出差异矩阵。矩阵是统一层设计的输入，也是回归验证的清单。
- R2 统一层采用已有模式：base 组件承载共性，平台 config 对象承载差异，视图层为薄壳。不引入新的抽象机制。
- R2.1 **config 契约为两层**（父任务 `design.md` §8）：
  - descriptor 层（`src/config/platformDescriptors.ts` 现 50 行 → 扩展，`platformCapabilities.ts` 74 行不动）声明「该平台有哪些面」，驱动路由生成与导航。形如 `claude: { rootPath: '/claude', surfaces: ['settings','profiles','auth','mcp','agents','plugins','commands'] }`。
  - per-surface config 层（`src/configs/settings.ts`、`profiles.ts`、`auth.ts`、`commands.ts`；`slashCommands.ts` 192 行不动，作为参照实现）声明「该面在该平台怎么表现」，驱动对应 base 组件。
  - 变更成本：新增平台改 descriptor 一行 + 各 config 模块加一个导出；改某面共性只改该面 base 组件；改某平台某面差异只改该 config 模块的一个导出。
- R3 平台差异通过 config 与 props 表达。禁止在 base 组件内使用平台名称的条件分支（形如 `if (platform === 'codex')`），由 lint 规则或 review 门强制。
- R4 每个平台的现有行为逐项保留。差异矩阵中的每一项在统一后有对应的 config 字段或 props，无静默丢失。
- R5 统一后单个平台的视图文件行数不超过 100 行，超出项需说明原因。
- R6 Auth 面的统一范围由差异普查决定。若 Claude OAuth、Codex OAuth、Grok token 三种流程的统一会引入超过差异矩阵项数的条件复杂度，该面可部分统一或不统一，判定需记录依据。
- R7 收敛到 `views/generic/` 的三个功能面（MCP、Agents、Plugins），其现有 generic 实现需先补齐缺失能力，再接入 Codex 与 OpenCode 的调用点。
- R8 统一层接口写入契约文档。接口变更需通知全部消费方。
- R9 路由路径不变。统一是视图实现层的收敛，不改变 75 条路由与页面划分。
- R10 消费 `08-22-design-system` 的原语与 token，`08-22-arch-quality-perf` 的规模与复杂度约束适用于本任务产出。

## Acceptance Criteria

- [x] AC1 七个功能面的差异矩阵落盘，逐平台逐项列出，无未确认项。
- [x] AC2 20 个重复实现文件全部处理：或收敛为薄壳，或按 R6 判定保留并记录依据。
- [x] AC3 统一后七个功能面的总行数落盘，与 15,672 行基线对比。
- [x] AC4 base 组件内无平台名称条件分支，由检查规则断言。
- [x] AC5 差异矩阵的每一项在统一后可追溯到 config 字段或 props，追溯表落盘。
- [x] AC6 每个平台每个功能面的核心操作路径逐项验证通过。验证矩阵按「平台 × 功能面」组织，无未验证格。
- [x] AC7 统一后各平台视图文件行数不超过 100 行，超出项有说明。
- [x] AC8 75 条路由路径不变，路由清单比对通过。
- [x] AC9 统一层接口契约文档落盘，已登记到 `08-22-test-contract-rebuild` 的范围表。
- [x] AC10 修改一处共性行为后，全部消费平台同时生效，由一个跨平台验证用例证明。
- [x] AC11 `bun run type-check` 与 `bun run lint:ci` 退出码 0。

## 前置与后续

- 前置：`08-22-shell-port`。差异普查（R1）可在 `08-22-arch-quality-perf` 之后即开始，不必等 shell-port 完成。
- 后续：`08-22-views-claude`、`08-22-views-codex`、`08-22-views-secondary-platforms`、`08-22-views-profiles-config`、`08-22-views-sync-tools` 五个子任务的范围因本任务而缩减，需在本任务的差异普查完成后调整其 `prd.md` 范围表。
- `08-22-views-checkin` 与 `08-22-views-usage` 不受影响。

## Out of Scope

- 路由与页面划分调整。
- 新增平台支持。
- `AppSettingsView.vue`、`McpManagerView.vue`、`OpenCodeProvidersView.vue`、SlashCommands 三视图（见「不在统一范围」）。
- `components/profiles/`（10 文件 4,040 行）与 `components/mcp/`（4 文件 2,064 行）等共享组件层的重构。本任务复用它们，不改造它们。
- Rust 侧的平台抽象。`ExecutionEnvironment` trait 与各平台 IPC 命令不变。

## Notes

- 本任务把「照搬 15,672 行」换成「差异普查 + 建 6,000–7,500 行统一层 + 逐平台验证」。净增工程日约 8–14 日，低于最初 25–35 日的估计，原因是 base + config + 薄壳模式已在仓库内验证，不需要设计新架构。
- Auth 是风险最高的一面。三个平台的认证流程存在实质差异（Claude OAuth、Codex OAuth、Grok token），强行统一可能得到比重复更难维护的条件分支。R6 允许该面部分统一或不统一。
- 差异普查的质量决定本任务成败。普查遗漏的差异项在统一后表现为功能静默丢失，且 122 个 smoke 测试无法覆盖全部平台组合。AC6 的「平台 × 功能面」验证矩阵不可省略。
- 统一层的接口是五个视图子任务的共同依赖。接口需在本任务早期定稳，变更成本随并行子任务推进而上升。
