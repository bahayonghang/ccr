# PRD: Claude Code Profiles 页面落地

父任务：`.trellis/tasks/07-29-profiles-redesign/`。依赖：子任务 `07-29-profiles-shared-layer` 完成。

## 范围

`ccr-ui/src/views/ClaudeCodeProfilesView.vue`（1971 行）接入重构后的共享组件层，抽取内联编辑器为独立模态组件，清理页面级冗余与死代码。

## 需求

1. **接入新骨架**：Header 动作收敛（补回 ⌘K 入口按钮，与 Codex 对齐）、四槽 StatStrip（特色槽 = Auth split）、瘦身 QuickRail（接入 `useProfilesQuickSwitch`，平台键 `claude`）、Filters popover 工具栏（含 provider 筛选入 popover）、ProfilesInspector 右栏（hover/focus 预览 + diff）。
2. **编辑器抽取**：视图内联编辑器（BaseModal + `ClaudeProfileEditorSections` + ~370 行 `--editor-*` 样式）抽取为 `components/claude/ClaudeProfileEditorModal.vue`，props/emit 架构对齐 `CodexProfileEditorModal`；样式迁移到共享编辑器基底，删除 `--editor-*` 体系；新增保存前校验汇总条。
3. **ClaudeProfileRow 视觉统一**：废弃 Tailwind 任意值字号与 per-provider 整卡动态色，迁移 `--cp-*`；provider 以色点/小徽章弱表达；保留搜索匹配高亮；base_url 显示完整 host、model 走统一 fallback；编辑/删除收入角落菜单；保留 apply 主操作。
4. **共享逻辑去重**：model fallback 链 4 份重复收进 `utils/claudeProfiles.ts` 单一函数，row/rail/insights 全部引用；`.cp-list-head` 用共享 `ProfilesSection.vue`；列表行接上 busyAction 反馈。
5. **Apply/Delete 确认升级**：apply 确认框接 `ProfileDiffRows`（当前 → 目标）；delete 确认框加备份 footnote。
6. **死代码清理**：删除 ~20 个未引用 i18n 键（清单见父任务 research）；移除 `ProfilesSection` 内联定义；清理触及组件内的 `translateWithFallback` 硬编码中文回退与 ProviderTemplateSelector 裸英文标签。
7. **视图瘦身**：目标 ≤900 行（模板 + 逻辑），样式块收敛。

## 验收标准

- 父任务跨子任务验收标准 1–11 中适用于本页的项全部满足。
- `cd ccr-ui && bun run test` 通过；`just ui-check` 通过。
- 暗/亮双主题截图走查：首屏信息层级清晰（当前 profile 一眼可见、Apply 目标可验证、无 chip 墙）。
- 行为回归：apply/delete/rename/raw TOML 编辑/导出/搜索高亮/⌘K/数字键（含确认）全部可用。
