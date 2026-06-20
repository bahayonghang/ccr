# ccr Claude profile fable 层与显示名端到端支持

## Background

承接父任务 `06-20-claude-fable-and-glm-profile`。ccr 的 Claude profile 缺少 fable 模型层与各层 `*_MODEL_NAME` 显示名（全仓库 `FABLE` 零命中）。本任务沿用前置任务 `06-19-claude-third-party-model-authmode` 将 `custom_model_option` 提升为一等字段的同一模式，把 fable 层与显示名做成一等 typed 字段并纳入托管。

## Requirements

### R1 — fable 模型层（核心）
- R1.1 `ConfigSection`（`crates/ccr-config/src/managers/config/types.rs`）与 `ProfileConfig`（`crates/ccr-config/src/models/platform.rs`）新增 `default_fable_model: Option<String>`，serde 语义与现有 `default_opus_model` 完全同构（`skip_serializing_if = "Option::is_none"`）。
- R1.2 `settings.rs` 新增常量 `ANTHROPIC_DEFAULT_FABLE_MODEL = "ANTHROPIC_DEFAULT_FABLE_MODEL"`，在 `update_from_config` 中映射写出。
- R1.3 该常量纳入 `clear_anthropic_vars`/`clear_managed_vars` 的清理集合（与 OPUS/SONNET/HAIKU 同列），保证切换 profile 时被清掉，**消除跨 profile 串档**。
- R1.4 `ClaudePlatform::get_env_var_names`（`crates/ccr-cli/src/platforms/claude.rs:335`）登记该 env 名。

### R2 — 各层显示名 `*_MODEL_NAME`
- R2.1 新增 typed 字段：`default_opus_model_name`、`default_sonnet_model_name`、`default_haiku_model_name`、`default_fable_model_name`（均 `Option<String>`），映射到对应 `ANTHROPIC_DEFAULT_*_MODEL_NAME`。
- R2.2 同 R1.2~R1.4：常量、`update_from_config`、清理集合、`get_env_var_names` 四处同步。
- R2.3 显示名字段与模型字段解耦：仅当对应 `_NAME` 非空时写出，不强制与模型字段共存。

### R3 — 加载兼容/迁移
- R3.1 旧 profile（platform_data 里残留 `default_fable_model` / `*_model_name` 字符串键）在加载/保存时抬升到新 typed 字段，并从 platform_data 移除残留键（参考前置任务对 `custom_model_option` 的迁移做法）。
- R3.2 缺省（字段为 None）时行为与今日一致，不写出对应 env。

### R4 — ccr-ui 表单录入
- R4.1 在 Claude profile 编辑器「高级模型映射」区暴露 fable 模型与四个显示名输入（`ccr-ui/src/components/claude/ClaudeProfileEditorSections.vue` + `ccr-ui/src/types/claudeProfileEditor.ts` + `ccr-ui/src/types/claude.ts`）。
- R4.2 i18n（zh-CN / en-US）补对应 label 与 helper 文案。
- R4.3 与后端 `profile_to_json` / 保存回填字段名保持一致（snake_case ↔ 前端命名约定遵循既有字段）。

## Assumptions（需确认/已确认）

- **A1（已确认）**：TUI（`crates/ccr-tui`）经核对**没有**「按模型层编辑 profile」入口（仅 profile 切换与各平台 auth），故本任务不在 TUI 内新增模型字段编辑器；父任务「UI/TUI 录入」实际落在 ccr-ui。若后续需要 TUI 编辑器，另开任务。
- **A2（待研究，列入 implement 验证项）**：`ANTHROPIC_DEFAULT_*_MODEL_NAME` 与 `ANTHROPIC_DEFAULT_FABLE_MODEL` 来源于用户截图；需在 Claude Code 官方文档侧确认字段名拼写与语义。截图为真实可用配置，作为权威来源；若官方文档与之冲突以实际生效为准并记录。

## Constraints

- 遵守根 CLAUDE.md 安全约束：masking、改动前备份、文件锁、原子写不回退；日志不得泄露 token。
- 内部注释中文，公共 API doc 英文。
- 不破坏既有 `custom_model_option` / auth_mode 行为与测试。
- 改动需 `just lint-strict` / `just test`（含 `-- --test-threads=1`）/ `just frontend-check-quick` 验证。

## Acceptance Criteria

- [ ] AC1：构造含 `default_fable_model` 的 profile，`apply_profile` 后 `settings.json` 写出 `ANTHROPIC_DEFAULT_FABLE_MODEL`（新增单测）。
- [ ] AC2：先 apply 含 fable 的 profile，再 apply 不含 fable 的 profile，`settings.json` 中 `ANTHROPIC_DEFAULT_FABLE_MODEL` 被清除（防串档单测）。
- [ ] AC3：四个 `*_MODEL_NAME` 字段非空时各自写出对应 env；为空时不写出（新增单测）。
- [ ] AC4：platform_data 内残留 `default_fable_model` / `*_model_name` 键经加载/保存自动迁移到 typed 字段且不残留（迁移单测）。
- [ ] AC5：ccr-ui 可录入 fable 模型与四显示名，保存后回填正确；type-check / lint / smoke 通过。
- [ ] AC6：`just lint-strict`、`just test`、`just frontend-check-quick` 全绿。

## Out of Scope

- TUI 模型字段编辑器（见 A1）。
- 非模型 env（ENABLE_TOOL_SEARCH 等）。
- Codex/Gemini/Droid 平台。

## References

- 前置：`.trellis/tasks/archive/2026-06/06-19-claude-third-party-model-authmode`（custom_model_option 一等字段化 + 迁移 + 前端暴露，可直接对照实现）
- 父任务：`06-20-claude-fable-and-glm-profile`
