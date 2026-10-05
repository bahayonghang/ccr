# 技术设计：Profiles 与配置视图迁移

> 父任务：`08-22-react-migration`。本域 9,882 行，不移交统一层，但持有 `08-22-platform-unify` 的 Profiles 统一层依赖。

## 1. 本批次共性转换

与 `08-22-views-claude` 的 `design.md` §1 同表，不重复。前置阅读同，另加 `profiles-page-contracts.md`（19.9 KB，全仓第二大契约）。

PRD Notes：建议先通读该契约再动手，避免迁移后大量返工。

## 2. 范围与两点协同约束

20 个文件，9,882 行，行数不变（PRD Scope）。两点协同约束：

1. `src/components/profiles/`（10 文件 4,040 行）是 `08-22-platform-unify` 的 Profiles 统一层的依赖。其接口需与统一层协同定稳，接口变更会同时波及 Claude / Codex / Grok 三个平台（协同点 F）。
2. `AppSettingsView.vue`(1,399) 为应用级设置，不属跨平台功能面，明确排除在统一范围外。

## 3. `components/profiles/` 接口定稳

本域最关键的交付项。该共享层被三个平台的 Profiles 视图消费，而那三个视图已移交统一层——因此消费方从「三个平台视图」变为「一个 base 组件 + 三个 config」。

定稳的公示内容（与 `08-22-shell-port` 的 `shared-interfaces.md` 同格式）：

- 10 个文件各自的 props 完整列表与类型。
- slot → children / render props 映射。
- 状态责任划分：哪些状态由共享层持有，哪些由消费方传入。

**与统一层的协同顺序**：`08-22-platform-unify` 的 `BaseProfiles` 在其批次 4 建立。本任务的接口需在其批次 4 之前公示，否则 base 组件按猜测的接口写。因此本任务的批次 1 是接口定稳，优先级高于其余批次。

Out of Scope 明确：本任务复用 `components/profiles/`，不改造它。因此「定稳」多数情况下是把现有 Vue 接口如实映射到 React，不重新设计。改造需求登记为独立缺陷。

## 4. 配置表单（`v-model` 密度最高的区域）

R2：配置表单是 `v-model` 密度最高的区域，需逐字段核对。

处理：

- react-hook-form 非受控注册。`AppSettingsView`(1,399) 是 `08-22-arch-quality-perf` 场景 1 的三个测量页之一，输入延迟直接受此影响。
- zod schema 承载字段校验，经 `@hookform/resolvers` 接入。
- **字段清单落盘**（AC4）：每个字段的名称、类型、默认值、校验规则、读写验证结果。这是逐字段验证的唯一依据。字段散落在多个表单页，清单需覆盖全部。

## 5. 凭据与写入安全（R6–R8）

四项行为由 Rust 侧实现，本任务只保证前端不绕过：

| 行为                                | 实现位置                          | 前端责任                         |
| ----------------------------------- | --------------------------------- | -------------------------------- |
| 配置写入原子性（tempfile + rename） | `src-tauri` / `crates/ccr-config` | 只调现有 wrapper，不新增直写路径 |
| API key 与 auth token 掩码          | 界面渲染 + `logRedact.ts`         | 掩码显示，日志无明文（AC5）      |
| 配置切换前备份                      | `crates/ccr-config`               | 不跳过备份步骤（AC6）            |
| 文件锁                              | `crates/ccr-config`               | 不并发发起冲突写入               |

若发现前端存在绕过路径，登记为独立缺陷（PRD Notes），不在本任务修复。

AC7（配置写入为原子操作，中断写入后原文件完整）的验证方式：该行为在 Rust 侧，前端验证只能确认调用路径正确。中断测试需在 Rust 侧或手工制造中断，验证方法在实施时确定。

## 6. `profileDiff.ts` 的差异展示（R9）

迁移后视觉与语义一致。`profileDiff.ts` 为框架无关资产（原样复用），本任务只改渲染。

差异展示的视觉一致性依赖 token——diff 的增删标记颜色来自 `profiles-page.css`（28 变量，由 `08-22-design-system` 迁移）。迁移后变量名不变。

## 7. `0.75rem` 字号例外

`profiles-page-contracts.md` 与 `theme-token-contracts.md` 均登记了该例外：Profiles 共享层的密集元信息可用 `0.75rem`（低于 Label 下限 `0.8125rem` 一档）。

该例外在新体系中保留（R4、AC9）。硬编码收口时不把它当违规项——它需出现在 `hardcode-exemptions.md` 的登记中，且 `profiles-page-contracts.md` 重写时保留说明。

## 8. 原始配置编辑器的边界

`raw-config-editor-contracts.md` 在本任务与 `08-22-views-sync-tools` 之间分责：

- CodeMirror 桥接的实现属 `08-22-views-sync-tools`（其 R2，Out of Scope 明确排除在本任务外）。
- 本任务的 `ConfigsView` / `EditConfigModal` 等若嵌入编辑器，消费对方产出的桥接组件。
- 契约验证（AC8 含该契约）在本任务与对方各验一次，各验自己的调用路径。

因此本任务需等 `08-22-views-sync-tools` 交付桥接组件后才能完成嵌入编辑器的部分。该依赖是两个并行子任务间的唯一顺序约束，需在实施时协调。

## 9. `ConverterView`(915)

配置格式转换。IPC 走 `src-tauri/src/commands/converter.rs`（不改）。

转换结果的展示与 `profileDiff` 的差异展示可能共用渲染组件，迁移时核对。

## 10. 框架无关资产

`src/utils/claudeProfileEditor.ts`、`claudeProfileFields.ts`、`claudeProfiles.ts`、`profileDiff.ts`、`providerTemplates.ts`，以及 `src/config/`（11 文件）、`src/configs/`（5 文件）原样复用，只改调用点。

`src/config` 与 `src/configs` 的 git diff 须为空（AC10）——但 `08-22-platform-unify` 会改这两个目录（其批次 3 扩展 descriptor、新建 per-surface config）。因此 AC10 的检查范围需排除统一层的改动，只针对本任务的提交。检查方式：对本任务的分支范围做 diff，不对工作区做 diff。

## 11. 不变量

- IPC 调用点沿用现有 wrapper（R10）。
- `src/api`、`src/types` 不改。
- `src-tauri/src/commands/config.rs`、`converter.rs` 不改。
- `crates/ccr-config/` 不改，含掩码、原子写入、文件锁与备份实现。

## 12. 未决项

- AC7 的中断写入验证方法（第 5 节末段）。
- 嵌入编辑器部分与 `08-22-views-sync-tools` 的交付时序（第 8 节末段）。
- `ConverterView` 与 `profileDiff` 是否共用渲染组件（第 9 节）。
- AC10 的 diff 检查范围界定（第 10 节末段）。
