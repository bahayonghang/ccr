# 技术设计：跨平台功能面统一

> 父任务：`08-22-react-migration`。两层 config 契约见父任务 `design.md` §8。本文件写差异普查方法、统一层形状与 Auth 面的判定标准。

## 1. 差异普查方法（本任务成败的决定项）

普查遗漏的差异项在统一后表现为功能静默丢失，且 122 个 smoke 测试无法覆盖全部平台组合（PRD Notes）。因此普查不能靠通读，需按固定的六个维度逐项抽取。

对每个功能面的每个平台文件，抽取：

| 维度         | 抽取方式                                                                          |
| ------------ | --------------------------------------------------------------------------------- |
| 字段与配置项 | 表单字段名、类型、默认值、可选性。抽取来源为模板中的输入控件与 `v-model` 绑定目标 |
| 操作         | 按钮与菜单项对应的动作清单                                                        |
| 校验规则     | 每个字段的校验条件与错误文案 key                                                  |
| IPC 命令     | `rg` 该文件调用的 `src/api` wrapper 名                                            |
| 文案 key     | `rg -o '\$t\([^)]+\)'` 抽出的 i18n key 集合                                       |
| 分支状态     | 空状态、加载态、错误态、权限不足态的呈现差异                                      |

六个维度对每个「平台 × 功能面」格产出一份清单。差异矩阵的构造方式：同一功能面的各平台清单做集合比较，输出「共有项」与「平台独有项」。

**项的原子化规则见第 5.1 节**，对全部七个功能面适用，不只 Auth 面。规则统一后，矩阵的项数不随执行者的拆分习惯变化。

**差异矩阵格式**：

| 功能面   | 维度 | 项                 | claude | codex | grok | opencode | 归属         |
| -------- | ---- | ------------------ | ------ | ----- | ---- | -------- | ------------ |
| Settings | 字段 | `model`            | ✓      | ✓     | ✓    | ✓        | base         |
| Settings | 字段 | `reasoning_effort` | —      | ✓     | —    | —        | config.codex |

「归属」列的取值：`base`（共有，进 base 组件）、`config.<platform>`（平台独有，进该平台 config）、`props`（调用点决定，如 `hide-chrome`）。

矩阵落盘为 `diff-matrix.md`（AC1）。矩阵同时是 AC5 追溯表与 AC6 验证矩阵的输入——三者共用同一份项清单，不重复枚举。

普查可在 `08-22-arch-quality-perf` 之后即启动，不必等 `08-22-shell-port` 完成（PRD 前置说明）。这是本任务能提前开工的部分。

## 2. 两层 config 的类型形状

### descriptor 层

`src/config/platformDescriptors.ts` 现 50 行，含 `GenericPlatformFeatureRoute`、`GenericPlatformDescriptor`、`genericPlatformDescriptors`、`GenericPlatformId`、`genericPlatformDescriptorList`。

扩展形状（父任务 `design.md` §8）：

```ts
type Surface =
  "settings" | "profiles" | "auth" | "mcp" | "agents" | "plugins" | "commands";

interface PlatformDescriptor {
  rootPath: string;
  surfaces: readonly Surface[];
}
```

`src/config/platformCapabilities.ts`（74 行）不动。

descriptor 驱动路由生成与导航。`08-22-shell-port` 的路由生成逻辑按此形状接收输入（其 `design.md` §2），路由路径不变（R9、AC8）。

### per-surface config 层

`src/configs/` 下每个面一个模块，每个模块每平台一个导出：

```ts
// src/configs/settings.ts
export const claudeSettingsConfig: SettingsConfig = { ... }
export const codexSettingsConfig:  SettingsConfig = { ... }
```

`SettingsConfig` 等接口的字段来自差异矩阵中「归属 = config.*」的项。契约文档落盘为 `platform-surface-contracts.md`，进 `.trellis/spec/ccr-ui/frontend/`，是 `08-22-test-contract-rebuild` 的第 19 份契约（该任务基线 16 + `arch-quality-perf` 2 + 本任务 1）。文件名在此定，不留待实施（AC9）。

字段设计原则：

- 平台独有字段用可选属性，base 组件按 `undefined` 判定不渲染。
- 平台间取值不同的共有字段用必填属性。
- 不用 `platform: 'codex'` 这类标识字段——它会诱导 base 组件写平台分支（R3 禁止）。

`src/configs/slashCommands.ts`（192 行）不动，作为参照实现。

## 3. base 组件的形状

参照 `BaseSlashCommands.vue`（507 行）+ `configs/slashCommands.ts`（192 行）+ 18–27 行薄壳视图，三平台视图层合计 274 行。

统一后每个功能面的结构：

```
features/platform/<surface>/Base<Surface>.tsx      共性实现
src/configs/<surface>.ts                          每平台一个 config 导出
features/<platform>/<Platform><Surface>View.tsx    薄壳，≤100 行（R5）
```

薄壳的内容：`PageShell` + `PageHeader` + `<BaseX config={xConfig} />`，与现有 `SlashCommandsView.vue`（18 行）同形。

**禁止平台名称条件分支**（R3）：base 组件内不得出现 `if (platform === 'codex')` 形态。强制手段为 ESLint `no-restricted-syntax`，匹配 base 组件目录下对平台字面量的比较。规则加入 `08-22-arch-quality-perf` 建立的规则集，由本任务提供匹配模式（AC4）。

## 4. 七个功能面的收敛方案

| 功能面   | 现状            | 方案                                                                                 |
| -------- | --------------- | ------------------------------------------------------------------------------------ |
| Settings | 4 文件 3,923 行 | 新建 base + 4 config + 4 薄壳                                                        |
| Profiles | 3 文件 3,325 行 | 新建 base + 3 config + 3 薄壳。复用 `components/profiles/`（10 文件 4,040 行）共享层 |
| Auth     | 3 文件 2,298 行 | 见第 5 节                                                                            |
| Commands | 2 文件 2,090 行 | 新建 base + config + 薄壳                                                            |
| MCP      | 2 文件 1,734 行 | 收敛到 `generic/PlatformMcpView`（407 行），先补齐能力                               |
| Agents   | 2 文件 1,580 行 | 收敛到 `generic/AgentsView`（725 行），先补齐能力                                    |
| Plugins  | 2 文件 722 行   | 收敛到 `generic/PlatformPluginsView`（367 行），先补齐能力                           |

合计 20 文件 15,672 行 → 预估 6,000–7,500 行。

**收敛到 `generic/` 的三个面（R7）的顺序**：先补齐 generic 实现缺失的能力，再接入 Codex 与 OpenCode 的调用点。缺失能力从差异矩阵中「平台独有项且 generic 未实现」推出。若先接入后补齐，接入期间该平台功能缺失。

### 两个共享层的前置依赖（阶段 4a）

`components/profiles/`（10 文件 4,040 行）与 `components/mcp/`（4 文件 2,064 行）是复用对象，本任务**不改造其接口**（Out of Scope）。但它们迁移前是 `.vue`，React base 组件无法复用未迁移的 Vue 组件，因此其**框架迁移**必须早于本任务的对应批次：

| 共享层                | 迁移与接口公示的所有者                 | 本任务的消费点              | 时序约束                |
| --------------------- | -------------------------------------- | --------------------------- | ----------------------- |
| `components/profiles/` | `08-22-views-profiles-config` 批次 1    | 批次 4 的 `BaseProfiles`    | 必须早于本任务批次 4    |
| `components/mcp/`     | `08-22-views-sync-tools` 批次 3 的前半 | 批次 5 的 `PlatformMcpView` | 必须早于本任务批次 5    |

这两批被提到父任务的阶段 4a（共享层前置门），先于本阶段执行。若不做该拆分，则形成阶段 4 → 5 → 4 的依赖环，统一层门无法闭合。协同点 F 与 F2。

## 5. Auth 面的判定标准

三个平台的认证流程存在实质差异：Claude OAuth、Codex OAuth、Grok token。行数分布也最不均（1,179 / 958 / 161）。

R6 允许该面部分统一或不统一。判定需可操作，因此先固定两个量的定义，否则同一实现会因矩阵拆分粒度不同被判为全统一 / 部分统一 / 不统一三种结果。

### 5.1 差异项的原子化规则

一个差异项 = 六个维度之一 × 一个最小可独立开关的单元。最小单元的定义逐维度固定：

| 维度     | 一项 =                                                      | 不允许的合并                                       |
| -------- | ----------------------------------------------------------- | -------------------------------------------------- |
| 字段     | 一个表单字段名                                              | 不把「三个 OAuth 相关字段」合并计为一项            |
| 操作     | 一个按钮或菜单项对应的一个动作                              | 不把「登录 / 登出」合并为「会话管理」              |
| 校验规则 | 一个字段上的一条校验条件                                    | 不把同字段的多条条件合并                           |
| IPC 命令 | 一个 wrapper 名                                             | 不按域合并                                         |
| 文案 key | 一个 i18n key                                               | 不按前缀合并                                       |
| 分支状态 | 一个状态（空 / 加载 / 错误 / 权限不足）的一处呈现差异        | 不把「全部错误态」合并为一项                       |

拆分粒度由该规则唯一确定，不由执行者选择。

### 5.2 计量单位统一

两个量都按「base 组件内的条件位置数」计量，避免拿「项」比「分支点」：

- **分支点数**：base 组件内每个由 config 字段驱动的条件渲染或条件逻辑算一个。一个 config 字段驱动 3 处分支，计 3。
- **差异项对应的理论分支点数**：按 5.1 拆出的每个差异项，在 base 组件内表达它**至少**需要几个条件位置。多数项为 1（一个可选字段的 `undefined` 判定），个别项为 0（可由 config 的数据驱动而不需条件，如文案 key 直接来自 config）或 >1。逐项标注，求和。

比较对象是「实际分支点数」与「理论分支点数之和」。

### 5.3 该比值是辅助数据，不是判定本身

比值可作参考，但不单独决定结论——单项子集天然满足条件，按比值机械执行会得到「只统一一个字段」这类无意义的部分统一。因此判定由一个评审门做出，输入三项：

1. 5.2 的比值。
2. 保留重复的成本：不统一时需要在 N 个平台各改一次的行为条目数。
3. 统一后的复杂度：base 组件的实际条件密度（分支点数 / base 组件行数）。

判定结果的三种取值与选取依据：

| 取值     | 依据                                                                                             |
| -------- | ------------------------------------------------------------------------------------------------ |
| 全统一   | 实际分支点数 ≤ 理论分支点数之和，且条件密度不高于已验证的 `BaseSlashCommands` 参照实现            |
| 部分统一 | 上述不成立，但存在一个**覆盖 ≥2 个平台且 ≥3 个差异项**的子集满足条件。该子集统一，其余保留        |
| 不统一   | 无满足上述规模下限的子集                                                                          |

「覆盖 ≥2 个平台且 ≥3 个差异项」的下限排除单项子集，使「部分统一」是实质收敛而非形式满足。

判定为「部分统一」或「不统一」时，保留的文件不计入 AC3 的行数对比基线，并在 `diff-matrix.md` 中标注原因（AC2 的「按 R6 判定保留并记录依据」）。判定过程的三项输入与结论一并记录。

## 6. 追溯表与验证矩阵

**追溯表（AC5）**：差异矩阵的每一项 → 统一后的 config 字段名或 props 名。列：功能面、维度、项、归属、统一后位置。无「归属 = config.*」但找不到对应字段的项。

**验证矩阵（AC6）**：按「平台 × 功能面」组织。每格的内容为该平台该功能面的核心操作路径清单与验证结果。格数上限为 4 平台 × 7 面 = 28，实际格数按 descriptor 的 `surfaces` 决定（不是每个平台都有全部面）。无未验证格。

三张表（`diff-matrix.md`、追溯表、验证矩阵）共用第 1 节的项清单，避免三次枚举产生不一致。

## 7. 行数对比

AC3 要求统一后总行数与 15,672 行基线对比。计数范围：

- 计入：新建的 base 组件、新增的 config 导出、薄壳视图、为收敛到 `generic/` 而补齐的能力代码。
- 不计入：`components/profiles/` 与 `components/mcp/` 的既有共享层（复用，未改造）；按 R6 判定保留的 Auth 文件。

对比数据落盘。预估 6,000–7,500 行是目标区间，超出该区间不构成失败，但需说明原因。

## 8. 与五个视图子任务的范围衔接

`08-22-views-claude`、`views-codex`、`views-secondary-platforms`、`views-profiles-config`、`views-sync-tools` 五个子任务的范围因本任务缩减。各自 PRD 的范围表已写「精确切分由本任务的差异普查（R1）确定后回填」。

回填时机：阶段 4 → 5 门的准出项（父任务 `implement.md` §4）。回填内容为各子任务范围表中的行数与文件清单。

回填的方向是本任务接走的文件从对方范围表移出，对方改为提供 config 与薄壳。因此对方的工作量从「迁移 N 行」变为「填 config + 写薄壳」，需在其范围表中明确这一点。

## 9. 未决项

- 七个功能面的差异矩阵内容，按第 1 节的方法（含第 5.1 节的原子化规则）普查后产出。
- Auth 面的判定结果，按第 5 节的三项输入经评审门确定。
- `generic/` 三个视图需补齐的具体能力，从差异矩阵推出。
- 各 `<Surface>Config` 接口的字段集合，从差异矩阵的「归属 = config.*」项推出。
- 统一后的实际行数，与 15,672 行基线的对比在实施后得出。

已定，不再列为未决：统一层契约文件名为 `platform-surface-contracts.md`（第 2 节）。
