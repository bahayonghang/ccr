# Design: Codex Profiles 页面落地（差异点）

总体契约见父任务 `../07-29-profiles-redesign/design.md`；共享组件形态见 `../07-29-profiles-shared-layer/design.md`。

## 平台槽位

- StatStrip 特色槽 = Config mode（official vs custom relay 计数）。
- Toolbar Filters popover 内 = 标签 + 排序（无 provider 筛选）。
- Inspector 预览字段：base_url（official fallback 解析后）/ model / reasoning_effort / wire_api / auth_mode / auth_source / env_key（有值时）/ provider / tags。
- diff 行：base_url（解析后）/ model / auth_mode。

## Codex 特有清理

- **派生字段单源化**：`syncDerivedAuthFields()` 从表单流程删除；`buildCodexProfileRequest` 内部由 `auth_mode` 计算 `requires_openai_auth` / `openai_login_method`。废弃模式（`openai_chatgpt` / `provider_env_key`）仅作为编辑遗留 profile 时的 select 追加项保留。
- **`env_key` 序列化契约（易漏点）**：`syncDerivedAuthFields` 还承担「退出 `provider_env_key` 模式时清空 `env_key`」的隐藏职责（`CodexProfilesView.vue:790`），而当前 builder 无条件发送 `env_key`（`codexProfileEditor.ts:150`）。新契约：`env_key` 仅在 `auth_mode === 'provider_env_key'` 时序列化，其余模式一律置空；补模式切换回归测试（父 design §8.2）。
- **fallback/标签单源化**：`resolveCodexBaseUrl(profile)` 与 `codexAuthModeLabel(mode)` 收进 `utils/codexProfileEditor.ts`；ProfileCard / descriptors / rail 全部引用。
- **编辑器样式迁移**：模板结构不动，只换样式基底；`!important` 与硬编码 RGBA 随 `--editor-*` 一并删除。

## env-export 的归宿

Codex 卡片独有的 `shell_export_script` 复制按钮保留为卡片操作区第三图标按钮（复制图标 + aria-label + toast 反馈），位置与编辑/删除同组，不破坏卡片统一规范。

## 不做

- 不改 `onActivated` TTL 刷新策略。
- 不改 `listCodexModels` 模型目录加载与 `resolveModelSelection` 交互。
