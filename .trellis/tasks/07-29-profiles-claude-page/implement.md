# Implement: Claude Code Profiles 页面落地

前置：`07-29-profiles-shared-layer` 已归档或其组件已可用。每步后跑 `cd ccr-ui && bun run test`，收尾跑 `just ui-check`。

## 检查清单

1. [ ] `utils/claudeProfiles.ts`：新增 `resolveClaudePrimaryModel()` 与 descriptor 组装函数；删除 4 份 fallback 重复。
2. [ ] 新建 `components/claude/ClaudeProfileEditorModal.vue`：迁移视图内联编辑器模板/逻辑/段导航 scroll-spy；样式接共享编辑器基底；校验汇总条。
3. [ ] 视图接入编辑器模态（替换内联 BaseModal 块），验证 add/edit/rename 流程。
4. [ ] 重构 `ClaudeProfileRow.vue`：`--cp-*` 迁移、字段策略（完整 host / fallback model）、操作区收敛、保留搜索高亮。
5. [ ] 视图接入新 QuickRail + `useProfilesQuickSwitch('claude')` + 稳定编号 hotkeys。
6. [ ] 视图接入四槽 StatStrip（特色槽 Auth split）。
7. [ ] 视图接入新 Toolbar（provider 筛选入 Filters popover）。
8. [ ] 视图接入 ProfilesInspector：`previewName` ref + 行/卡片 hover/focus 事件 + Health `@locate` 滚动高亮 + tag-select 写筛选。
9. [ ] Apply 确认框接 `ProfileDiffRows`；Delete 确认框加备份 footnote。
10. [ ] Header 收敛（Add / ⌘K / ··· 溢出：Reload/Export/Edit TOML）。
11. [ ] 死代码清理：未引用 i18n 键、内联 ProfilesSection、`.cp-list-head` 重复、硬编码回退、`cp-spin` 重复引用。
12. [ ] i18n 键调整对称入 zh-CN / en-US，更新 `.keys.txt` 检查文件（若存在对应机制）。
13. [ ] 行为回归走查 + 按父 `design.md` §8.1 截图协议产出暗/亮 × 2543px/1280px 四张走查图。

## 验证

- `cd ccr-ui && bun run test`
- `just ui-check`
- 手动走查：apply（含 diff 确认）/ delete（含备份提示）/ Ctrl+数字键稳定编号 / hover 预览 / Filters 徽标 / raw TOML 编辑

## 回滚点

- 步骤 2–3 编辑器抽取独立可回滚；步骤 5–8 若共享组件有缺陷，回滚视图接入到旧组件 props。
