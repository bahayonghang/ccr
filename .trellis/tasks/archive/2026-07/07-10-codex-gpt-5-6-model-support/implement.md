# Codex GPT-5.6 三模型支持实施计划

## Preconditions

- [x] 用户已审阅并批准 `prd.md`、`design.md` 与本计划。
- [x] 运行 `python ./.trellis/scripts/task.py start 07-10-codex-gpt-5-6-model-support`，状态进入 `in_progress`。
- [x] 按 inline workflow 加载 `trellis-before-dev` 与相关 ccr-ui/ccr-codex spec。
- [x] 重新确认 `git status --short`，不覆盖实施期间出现的用户改动。

## Phase 1: Lock Model Catalog Contracts

- [x] 在 Tauri Codex command 模块将内置模型目录更新为：
  - `gpt-5.6-luna`
  - `gpt-5.6-terra`
  - `gpt-5.6-sol`
- [x] 增加目录单测，断言顺序、去重和历史自定义模型合并行为。
- [x] 保持 `codex_list_models` / `CodexModelsResponse` 响应兼容，避免影响 `useCodexAgents`。
- [x] 确认 `custom-models.toml` 只读路径仍保留且不会被迁移或删除。

Validation:

- [x] `cd ccr-ui && just test-backend`

Rollback point:

- 恢复旧内置目录常量；不涉及用户数据回滚。

## Phase 2: Make Profile Catalog Preset-Only

- [x] 从 `CodexProfilesView` 移除 `codexCustomModels` 状态及对 `custom_models` 的合并。
- [x] Profile 预设只消费 `builtin_models`，新建表单默认选中第一个目标预设。
- [x] 保留 `CUSTOM_MODEL_OPTION` 与手动输入校验。
- [x] 保存自定义模型时只构造 Profile request，不调用 `addCodexCustomModel`。
- [x] 增加纯 helper 或等价可测逻辑，构造“三预设 + 可选当前旧值”的目录。
- [x] 确保新建、编辑、关闭、重新打开时当前旧值不会跨表单泄漏。

Validation:

- [x] `cd ccr-ui && bun run test:smoke -- tests/codex-profile-editor.smoke.test.ts tests/codex-profiles-view.smoke.test.ts`
- [x] `cd ccr-ui && bun run type-check`

Rollback point:

- 恢复 Profile 对 `custom_models` 的合并及保存时全局写入调用。

## Phase 3: Preserve Current Non-Preset Models

- [x] 编辑非预设模型 Profile 时追加并选中仅当前表单可见的模型选项。
- [x] 在 modal 中为该选项增加本地化“当前值”标签，value 保持原始模型字符串。
- [x] 允许当前旧值原样保存、迁移到目标预设或切换到新的手动自定义值。
- [x] 新建 Profile 不展示任何旧值选项。
- [x] 测试三个状态：预设 Profile、旧模型 Profile、新手动自定义模型。

Validation:

- [x] `cd ccr-ui && bun run test:smoke -- tests/codex-profile-editor.smoke.test.ts tests/codex-profiles-view.smoke.test.ts`

## Phase 4: Keep Provider Template Behavior Coherent

- [x] 复核 `mapTemplateToCodexProfilePatch` 与 `resolveModelSelection`：预设模型直接选择，非预设模型进入自定义输入。
- [x] 确认模板自定义模型不会写入全局目录。
- [x] 更新 Provider 模板 smoke 测试与 Trellis frontend spec，记录 Profile 预设目录和 per-profile custom contract。

Validation:

- [x] `cd ccr-ui && bun run test:smoke -- tests/provider-templates.smoke.test.ts tests/codex-profile-editor.smoke.test.ts`

## Phase 5: Retire Global Custom Writes From Profile Flow

- [x] 使用 `rg` 重新确认 `addCodexCustomModel` / `codex_add_custom_model` 消费者。
- [x] 若 Profile 是唯一调用方，删除前端 wrapper、兼容导出、请求/响应类型、Tauri command/handler 与只写 helper。
- [x] 保留 `listCodexModels`、只读 custom file helpers 和 Agent 目录行为。
- [x] 清理 `legacy-shells` 中失效的 mock，不改 Codex Agent 模型加载。

Validation:

- [x] `cd ccr-ui && bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts tests/legacy-shells.smoke.test.ts`
- [x] `cd ccr-ui && just test-backend`

Rollback point:

- 恢复 add command/wrapper；历史 `custom-models.toml` 始终保留，因此无需数据修复。

## Phase 6: Prove Runtime Compatibility

- [x] 在 `ccr-codex` 现有 TestCodexEnv 测试中参数化三个目标模型。
- [x] 对每个模型断言 Profile 保存/读取和 apply 成功、根级 `model` 精确一致、第三方 provider 仍使用 `wire_api = responses`。
- [x] 不增加共享 allowlist，不修改 CLI、导入、TUI 或定价行为。

Validation:

- [x] `cargo test -p ccr-codex -- --test-threads=1`
- [x] `just fmt-check`

## Phase 7: Full Review And Gates

- [x] 搜索旧预设与已移除 API，确认只保留合理的历史 fixture/用量测试，不做无关批量替换。
- [x] `git diff --check`
- [x] `cd ccr-ui && bun run type-check`
- [x] `cd ccr-ui && bun run lint:ci`
- [x] `just frontend-check-quick`
- [ ] 使用 ccr-ui web preview 验证新建、旧值编辑和自定义输入，不依赖 Tauri shell 完成视觉检查。
- [x] `just ci`
- [x] 运行 `trellis-check`，核对 PRD、设计、spec、代码和测试一致性。

Visual limitation:

- Web preview returned HTTP 200 at `http://127.0.0.1:5173/`, but the in-app Browser backend was unavailable. Screenshot-based verification remains unchecked; focused DOM smoke tests cover the new, legacy-current, and custom-input states.

## Phase 8: Finish

- [x] 根据实施中学到的稳定契约更新 `.trellis/spec/ccr-ui/frontend/`。
- [x] 更新本任务验收勾选和实施记录。
- [x] 按仓库规范拆分原子中文 Conventional Commits（实现提交 `0e999aaf`）。
- [x] 完成 Trellis archive 与 journal，记录 substantive work commits（`0e999aaf`、`7c535a48`）。
