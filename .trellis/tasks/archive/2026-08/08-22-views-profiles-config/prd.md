# Profiles 与配置视图迁移

> 父任务：`08-22-react-migration`

## Goal

将 Profiles 共享层、Provider 模板与配置管理视图从 Vue 迁移到 React，约 9,882 行。

## Scope

> **范围回填（`08-22-platform-unify` 普查后，2026-08-24）**：本任务文件仍不移交统一层，行数不变。协同约束：
>
> 1. `src/components/profiles/` 已是 React。统一层经 `features/platform/profiles/shared.ts` 再导出，不改造接口。
> 2. `AppSettingsView.vue`（1,399）仍排除在统一范围外。
> 3. `src/configs/{settings,profiles,commands,mcp,agents,plugins,auth}.ts` 由 `08-22-platform-unify` 所有；本任务 AC10 的 `src/configs` 冻结不含这些新模块。

| 文件 / 目录 | 行数 |
|---|---|
| `src/components/profiles/`（10 文件） | 4,040 |
| `src/views/AppSettingsView.vue` | 1,399 |
| `src/components/provider-templates/ProviderTemplateSelector.vue` | 1,275 |
| `src/views/ConverterView.vue` | 915 |
| `src/components/configs/`（3 文件） | 654 |
| `src/views/ConfigsView.vue` | 520 |
| `src/components/EditConfigModal.vue` | 406 |
| `src/components/AddConfigModal.vue` | 342 |
| `src/components/ConfigCard.vue` | 331 |
| 合计 | 9,882 |

关联的框架无关资产（原样复用，只改调用点）：`src/utils/claudeProfileEditor.ts`、`claudeProfileFields.ts`、`claudeProfiles.ts`、`profileDiff.ts`、`providerTemplates.ts`、`src/config/`（11 文件）、`src/configs/`（5 文件）。

关联的契约：`profiles-page-contracts.md`（19.9 KB，全仓第二大契约）、`provider-template-contracts.md`、`raw-config-editor-contracts.md`。

关联的样式：`src/styles/profiles-page.css`（28 个变量，由 `08-22-design-system` 迁移）。

## Requirements

- R1 上表 20 个文件迁移为 React 组件，对应 `.vue` 文件删除。
- R2 本批次内的 `v-model` 展开为受控属性与回调对，slot 转为 children 或 render props。配置表单是 `v-model` 密度最高的区域，需逐字段核对。
- R3 消费 `08-22-design-system` 产出的原语与 token，本批次不新增硬编码样式值。
- R4 `profiles-page-contracts.md`（19.9 KB）定义的行为在迁移后成立，含其中登记的 `0.75rem` 字号例外（Profiles 共享层的密集元信息）。
- R5 `provider-template-contracts.md` 定义的模板选择与应用行为不变。
- R6 配置写入路径的原子性行为不变：`src-tauri` 侧使用 tempfile + rename，前端不绕过该路径。
- R7 API key 与 auth token 字段的掩码显示行为不变，界面与日志不出现明文。
- R8 配置切换前的备份行为不变。
- R9 `profileDiff.ts` 的差异展示在迁移后视觉与语义一致。
- R10 IPC 调用点沿用 `src/api` 现有 wrapper，不新增或修改 wrapper。

## Acceptance Criteria

- [x] AC1 上表 20 个文件全部迁移，`rg --files -g '*.vue' src/components/profiles src/components/provider-templates src/components/configs src/views/AppSettingsView.vue src/views/ConverterView.vue src/views/ConfigsView.vue src/components/EditConfigModal.vue src/components/AddConfigModal.vue src/components/ConfigCard.vue` 无匹配。
- [x] AC2 3 个视图的路由可达，页面渲染无报错。
- [x] AC3 核心操作路径手动验证通过并记录：配置列表浏览、配置新增、配置编辑、配置切换、配置删除、Provider 模板选择与应用、配置格式转换、应用设置读写、Profile 差异查看。
- [x] AC4 配置表单的每个字段逐个验证读写正确，字段清单落盘。
- [x] AC5 API key 与 auth token 在界面中显示为掩码，在日志中不出现明文，由 smoke 测试断言。
- [x] AC6 配置切换前生成备份，备份文件可用于恢复。
- [x] AC7 配置写入为原子操作，中断写入后原文件完整。
- [x] AC8 `profiles-page-contracts.md`、`provider-template-contracts.md`、`raw-config-editor-contracts.md` 三份契约的验证项全部通过。
- [x] AC9 本批次组件内 px 字面量与 `rgba()` 数量为 0（登记豁免除外，含 `0.75rem` 字号例外的保留）。
- [x] AC10 `src/api`、`src/config`、`src/configs` 的 git diff 为空。
- [x] AC11 `bun run type-check` 与 `bun run lint` 退出码 0。
- [x] AC12 `app-settings` 与 profiles 相关 smoke 测试通过。

## 前置与后续

- 前置：`08-22-shell-port`。
- 可与其余六个视图子任务并行。
- i18n 调用点在本批次内同步转换，运行时切换与收尾校验属 `08-22-i18n-port`。

## Out of Scope

- 新增功能与信息架构调整。
- `src/api`、`src/types`、`src/config`、`src/configs` 的修改。
- `src-tauri/src/commands/config.rs`、`converter.rs` 的改动。
- `crates/ccr-config/` 的改动，含掩码、原子写入、文件锁与备份实现。
- 原始配置编辑器的 CodeMirror 桥接（属 `08-22-views-sync-tools`）。

## Notes

- `profiles-page-contracts.md` 为 19.9 KB，是本批次的主要约束来源。建议先通读该契约再动手，避免迁移后大量返工。
- 凭据掩码、原子写入、备份、文件锁四项行为由 Rust 侧实现，本任务只保证前端不绕过。若发现前端存在绕过路径，登记为独立缺陷。
- `src/components/profiles/` 的 10 个文件是跨平台共享层，被 Claude / Codex / Grok 三个平台的 Profiles 视图消费。接口需在本任务早期定稳并通知并行子任务。
