# Codex GPT-5.6 三模型支持

## Goal

为 CCR 的 Codex Profile 配置链路加入以下三个受控模型名称，并让所有 Codex Profile 模型选择器的预设目录全局只保留这三个值：

- `gpt-5.6-luna`
- `gpt-5.6-terra`
- `gpt-5.6-sol`

模型应能从 ccr-ui 编辑器保存到统一 Profile，随后由 `ccr-codex` 原样写入 Codex `config.toml`，同时避免旧 Profile、Provider 模板和用量定价发生静默破坏。

## User Value

用户可以直接在 CCR 中选择新的三个 GPT 模型别名，无需反复手工输入模型名称；预设列表保持明确、受控，同时仍可通过“自定义模型...”输入其他 Provider 暴露的模型名称。

## Confirmed Facts

- `ProfileConfig.model` 是 `Option<String>`，没有模型枚举或 GPT 型号白名单。
- `ccr-codex` 在构建切换配置时只 trim `profile.model`，随后将其原样写入 Codex 根级 `model` 字段。
- Codex Profile 的运行时协议仍要求 `wire_api = "responses"`；推理强度合法值为 `minimal/low/medium/high/xhigh`。
- ccr-ui Tauri 后端当前通过 `CODEX_BUILTIN_MODELS` 提供 `gpt-5.3-codex`、`gpt-5.4`，并与 `~/.ccr/platforms/codex/custom-models.toml` 合并。
- 前端 Profile 编辑器会合并内置模型与自定义模型，并额外提供“自定义模型...”自由输入入口；自由输入在保存时调用 `codex_add_custom_model` 持久化。
- 截图中的 `gpt-5.5` 不在当前 Tauri 内置常量中，可能来自已有 `custom-models.toml`。
- Provider 模板目前可以携带 OpenRouter、DeepSeek、Kimi 等不同模型；现行规范要求模板模型不在目录时自动转入自由输入路径。
- 当前 ccr-ui 用量界面调用已安装的 `llmusage` CLI 并只读 `llmusage.db`，不会在 UI 适配层重新计算模型价格。
- `ccr-types::ModelRateCatalog` 仍服务于旧 `ccr-db` 定价路径；未知模型会标记为 `unpriced`，价格为零。
- 目前没有可核实的公开官方资料证明三个目标名称的能力、价格或上下文窗口，因此本任务先将其视为目标 Provider 暴露的模型别名，不推断官方定价或能力参数。

## Requirements

- R1. 所有 Codex Profile 模型选择器的预设目录全局只显示三个指定模型，顺序固定为 luna、terra、sol。
- R2. 所有 Codex Profile 编辑器保留“自定义模型...”自由输入入口，允许保存任意非空、trim 后的模型名称。
- R3. 三个模型都必须经过 UI 请求、Profile TOML 序列化、Profile 读取和 `ccr-codex` apply 路径无损 round-trip。
- R4. 不为模型名新增共享 Rust 枚举，不在 `ccr-config` 或数据库中引入不必要的 schema migration。
- R5. 旧 Profile 中的其他模型名称不得在读取、列出、导入或直接应用时被静默改写；ccr-ui 编辑旧 Profile 时将该模型显示为仅属于当前 Profile 的“旧值/当前值”选项，CLI 和 `ccr-codex` apply 保持接受任意非空模型字符串。
- R6. 不自动删除用户现有 `custom-models.toml` 或旧 Profile 数据；如果停止使用该文件，应保留可回滚性并记录迁移行为。
- R7. Provider 模板填入的模型不属于三个预设时，不得静默替换成另一个模型；应复用“自定义模型...”路径并预填模板模型。
- R8. 不根据名称猜测 `model_reasoning_effort`、上下文窗口、价格或其他运行时能力；仅在拿到权威契约后增加对应配置。
- R9. ccr-ui 的当前用量链路继续透传 `llmusage.db` 中的 `pricing_status`、`pricing_source` 和 `pricing_rate`。
- R10. 如果提供了三个模型的权威价格，再以独立、可测试的定价改动同步上游 llmusage 与 CCR 旧定价目录；没有价格时保持 `unpriced`。

## Acceptance Criteria

- [x] AC1. 预设模型目录严格等于 `gpt-5.6-luna`、`gpt-5.6-terra`、`gpt-5.6-sol`，顺序稳定且无重复。
- [x] AC2. 新建任意 Codex Profile 时，下拉框提供三个目标预设和“自定义模型...”入口，不混入旧模型或此前手动输入的模型。
- [x] AC3. 分别保存三个模型后，`profiles.toml` 和应用后的 Codex `config.toml` 中 `model` 值与选择完全一致。
- [x] AC4. CLI 创建/更新、Profile 导入和旧 Profile apply 对非目标模型保持兼容，且不会被 UI 模型目录改动影响。
- [x] AC5. Provider 模板携带非预设模型时进入自定义输入路径并预填原始模型，不会静默选择错误模型。
- [x] AC6. 现有 `custom-models.toml` 不被无提示删除，旧 Profile 仍可通过 CLI/TUI/runtime 读取和应用。
- [x] AC7. 没有权威价格时，三个模型在旧定价路径中保持 `unpriced`；当前 ccr-ui 用量展示继续使用 llmusage 的定价字段。
- [x] AC8. Rust 定向测试覆盖三个模型的 Profile round-trip/apply；前端 smoke 测试覆盖精确预设、自定义输入、当前旧值和模板非预设模型处理。
- [x] AC9. 通过 `just fmt-check`、`cargo test -p ccr-codex -- --test-threads=1`、`cd ccr-ui && just test-backend`、相关前端 smoke、`bun run type-check`、`bun run lint:ci` 和 `just frontend-check-quick`。
- [x] AC10. 跨层最终验收运行 `just ci`，或在无法运行时记录明确阻塞与已通过的最小门禁。

## Verification Evidence

- `cargo test -p ccr-codex -- --test-threads=1`: 206 passed, 3 ignored; the GPT-5.6 regression saves, reloads, and applies all three models.
- `cd ccr-ui && just test-backend`: 247 command/backend tests plus 2 no-crate guard tests passed.
- Focused frontend smoke: 5 files, 35 tests passed; full `just frontend-check-quick`: 82 files, 376 tests passed.
- `just ci`: all steps passed, including strict Clippy, workspace tests, release build, RustSec audit, frontend/docs build, and VS Code CI.
- Web preview at `http://127.0.0.1:5173/` returned HTTP 200. In-app Browser was unavailable, so no screenshot-based visual evidence was collected; DOM states are covered by smoke tests.

## Out of Scope

- 猜测或伪造三个模型的官方价格、上下文窗口、发布日期、推理能力或 API 参数。
- 自动修改用户当前启用的 Codex Profile。
- 自动删除旧 Profile、用户自定义模型文件或其他 Provider 模板。
- 重构 Codex Profile 页面整体布局、认证方式或用量仪表盘。
- 将所有模型名称改造成 Rust enum。

## Decisions

- D1. 三模型限制指所有 Codex Profile 共用的“预设目录”全局严格只有三项，不按 Provider、模板或 `owlc-sub` 单独分流；它不是运行时 allowlist。
- D2. Provider 模板携带其他模型时，继续通过自定义模型路径填入该模型；模板仍只能填充非秘密 Provider 元数据。
- D3. 严格三选一只约束 ccr-ui 的 Codex Profile 新建/编辑入口；CLI、导入、TUI/runtime 读取与旧 Profile apply 保持兼容，不在共享 Rust Profile 验证边界增加 allowlist。
- D4. ccr-ui 编辑使用非预设模型的旧 Profile 时，额外显示一个仅属于当前 Profile 的旧值/当前值选项；用户可以只修改其他字段并继续保留该模型，也可以主动迁移到三个预设或新的手动自定义值。
- D5. 新建 Profile 的固定预设仍严格只有三个，但保留“自定义模型...”入口允许手动输入其他模型。
- D6. 手动输入的自定义模型只保存到当前 Profile，不调用全局自定义模型写入命令，也不加入后续新建 Profile 的下拉目录；再次编辑时按当前值选项展示。

## Open Questions

- None.
