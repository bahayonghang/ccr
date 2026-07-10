# Codex GPT-5.6 三模型支持技术设计

## 1. Scope

本任务修改 ccr-ui 的 Codex Profile 模型选择体验，并用 Rust 回归测试确认共享 Profile/runtime 链路能够无损处理三个目标模型。限制只属于 UI 预设目录，不成为 CLI、导入、TUI 或 `ccr-codex` 的运行时 allowlist。

目标预设按固定顺序为：

1. `gpt-5.6-luna`
2. `gpt-5.6-terra`
3. `gpt-5.6-sol`

## 2. Existing Data Flow

```text
codex_list_models
  -> CodexProfilesView modelCatalog
  -> CodexProfileEditorModal select/custom input
  -> buildCodexProfileRequest
  -> ProfileConfig.model
  -> CodexPlatform::build_switch_spec
  -> ~/.codex/config.toml root model
```

当前 `codex_list_models` 同时服务 Codex Profile 和 Codex Agent 编辑器，因此不能删除。它返回：

- `builtin_models`: Tauri 内置目录；
- `custom_models`: `custom-models.toml` 中的历史全局自定义值；
- `models`: 两者合并结果，供 Agent 编辑器使用。

Profile 编辑器目前消费前两者并再次合并，保存自由输入时调用 `codex_add_custom_model`，导致任意模型永久污染所有后续 Profile 的下拉列表。

## 3. Ownership And Contracts

### 3.1 Tauri model catalog

`ccr-ui/src-tauri/src/commands/codex.rs` 继续拥有内置模型目录的单一来源。将 `CODEX_BUILTIN_MODELS` 更新为三个目标模型，`codex_list_models` 的响应结构保持兼容。

`custom-models.toml` 继续只读加载，以免破坏 Codex Agent 编辑器及历史数据；本任务不删除或重写该文件。

`codex_add_custom_model` 不再由 Profile 编辑器调用。若确认没有其他调用方，可删除对应前端 wrapper、Tauri handler 和只用于写入的函数；保留只读目录 API。

### 3.2 Profile editor catalog

Profile 编辑器只把 `builtin_models` 作为固定预设，不再合并 `custom_models`。目录构造规则为：

```text
new profile     = three presets
edit preset     = three presets
edit non-preset = three presets + current profile model
```

“自定义模型...”是一个操作入口，不计入固定预设。自定义输入只进入当前 `CodexProfileRequest.model`，不更新任何全局目录。

### 3.3 Current/legacy model option

编辑 Profile 时，如果现有模型不属于三个预设：

- 将原模型追加为仅当前表单可见的选项；
- 用本地化后缀标明“当前值”；
- 默认选中该值，允许只修改其他字段后原样保存；
- 切换到预设后完成显式迁移；
- 切换到“自定义模型...”后可输入另一个任意非空模型。

关闭或重置表单时必须清空当前值状态，防止旧模型泄漏到下一次新建操作。

### 3.4 Provider templates

保留现有 Provider 模板契约：

- 模板模型属于三个预设时，直接选中预设；
- 模板模型不属于预设时，选择“自定义模型...”并预填原值；
- 模板不写入 token、env key 或其他秘密字段；
- 模板模型不会进入全局模型目录。

### 3.5 Runtime compatibility

`ProfileConfig.model` 继续为 `Option<String>`，`ccr-codex` 不新增型号枚举或 allowlist。CLI、导入、TUI 和旧 Profile apply 的行为保持不变。

在 `crates/ccr-codex/src/platforms/codex.rs` 增加参数化回归，确认三个目标字符串都能写入根级 `model`，并保持 `wire_api = responses` 的既有第三方路由契约。

## 4. API And Type Changes

保留：

- `listCodexModels`
- `CodexModelsResponse`
- `codex_list_models`
- 模型目录只读 helpers

候选删除：

- `addCodexCustomModel` 前端 wrapper 和兼容导出
- `CodexAddCustomModelRequest`
- `CodexAddCustomModelResponse`
- `codex_add_custom_model` Tauri command/handler
- `write_codex_custom_models`

删除前必须用 `rg` 复核无其他消费者。API facade 变更后运行 `api-facade-boundary.smoke.test.ts`。

## 5. Persistence And Migration

无需数据库或 Profile schema migration。

- 已有 `profiles.toml`: 原样兼容。
- 已有 `custom-models.toml`: 原样保留，不再影响 Profile 新建目录；Codex Agent 仍可通过合并目录读取历史值。
- 新的手动自定义值: 只存入对应 Profile 的 `model` 字段。
- 回滚: 恢复 Profile 对 `custom_models` 的合并和 add command 调用即可；任务不删除历史文件，因此数据可恢复。

## 6. Pricing Boundary

三个名称暂按 Provider 模型别名处理。本任务不修改 `ccr-types::ModelRateCatalog`，也不伪造价格、上下文窗口或推理能力。

- 当前 ccr-ui 用量继续透传 `llmusage.db` 的定价状态与价格来源。
- 旧定价路径遇到三个名称时保持 `unpriced`。
- 获得权威价格后应另开定价任务，同步上游 llmusage 与 CCR legacy catalog。

## 7. UI Copy And Accessibility

- 保留已有“自定义模型...”与自定义输入提示。
- 新增中英文“当前值”选项后缀或等价可访问标签。
- 原生 `<select>` 的 value 仍为真实模型名，显示后缀不能进入保存 payload。
- 新建与编辑状态切换后选项数量稳定，不因异步目录加载产生错误默认值。

## 8. Verification Strategy

### Focused Rust

- Tauri model catalog unit tests：内置预设精确顺序、历史自定义值仍只出现在合并目录。
- `ccr-codex` runtime test：三个模型逐一 apply 并断言根级 `model`。

### Focused Frontend

- `codex-profile-editor.smoke.test.ts`: 三个预设、自定义入口、当前旧值标签。
- `codex-profiles-view.smoke.test.ts` 或纯 helper 测试：新建目录不含历史自定义值，自定义保存不写全局目录。
- `provider-templates.smoke.test.ts`: 非预设模板模型进入自定义路径。
- `legacy-shells.smoke.test.ts`: 移除已废弃 API mock 后页面仍可加载。
- `api-facade-boundary.smoke.test.ts`: API wrapper/compat facade 未漂移。

### Gates

- `just fmt-check`
- `cargo test -p ccr-codex -- --test-threads=1`
- `cd ccr-ui && just test-backend`
- `cd ccr-ui && bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts tests/provider-templates.smoke.test.ts tests/codex-profile-editor.smoke.test.ts tests/codex-profiles-view.smoke.test.ts tests/legacy-shells.smoke.test.ts`
- `cd ccr-ui && bun run type-check`
- `cd ccr-ui && bun run lint:ci`
- `just frontend-check-quick`
- `just ci`

## 9. Task Shape

保持单一 Trellis 任务，不拆父子任务。模型目录、表单状态、API 清理和兼容测试共同形成一个原子用户行为；任一部分单独交付都会产生目录污染、旧值丢失或消费者不一致。
