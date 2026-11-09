# Design: Claude Code Profiles 页面落地（差异点）

总体契约见父任务 `../07-29-profiles-redesign/design.md`；共享组件形态见 `../07-29-profiles-shared-layer/design.md`。

## 平台槽位

- StatStrip 特色槽 = Auth split（subscription · api_key 计数）。
- Toolbar Filters popover 内含 provider 下拉（`allProviders`，唯一有 provider 筛选的平台）。
- Inspector 预览字段：base_url / model（fallback 后）/ opus / sonnet / haiku / subagent / effort / auth_mode / provider / account / tags。
- diff 行：base_url / model（fallback 后）/ auth_mode。

## 编辑器模态抽取架构

```
ClaudeCodeProfilesView.vue
└─ ClaudeProfileEditorModal.vue（新，props: form/updateField/modelCatalog?/templateDraft/...，emit: save/cancel）
   ├─ 共享编辑器外壳（profile-editor-shell）
   ├─ 段导航（basic/connection/auth/status + scroll-spy，逻辑随组件迁移）
   ├─ ProviderTemplateSelector（保持现有契约，见 provider-template-contracts.md）
   └─ ClaudeProfileEditorSections.vue（保留，仅样式迁移 token）
```

- 视图保留：数据加载、apply/delete/rename 流程、表单 state 组装与 `buildRequest`、rename 碰撞检测。
- 模态收 ~20 个 props 的现状可以接受（与 Codex 对齐），但保存按钮 disabled/校验汇总由模态内部派生。
- 模板填充自动翻 `auth_mode = api_key` 的既有行为保留，但在 UI 上给用户可见提示（父 design §3.6 的校验/提示面）。

## 视图瘦身切分

- `rowDescriptor` / `railDescriptor` 等 descriptor 组装移入 `utils/claudeProfiles.ts` 或独立 `composables/useClaudeProfileDescriptors.ts`。
- `ProfilesSection` 内联定义删除，引用共享文件。
- 重复 model fallback 删除，统一 `resolveClaudePrimaryModel(profile)`。

## 不做

- 不改 `normalizeClaudeProfilesState` 修复逻辑。
- 不动 raw TOML 编辑流程（仅入口进 Header 溢出菜单）。
