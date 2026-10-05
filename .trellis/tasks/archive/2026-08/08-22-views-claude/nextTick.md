# nextTick 登记（08-22-views-claude）

| 原调用点 | 意图 | 改写 |
| --- | --- | --- |
| `src/views/OutputStylesView.vue` 打开编辑弹层后 `await nextTick()` 再 focus 首个 input | 等 Teleport 弹层入树后设焦点 | 删除手搓 Teleport + 焦点陷阱；改 `BaseModal`（Radix 焦点陷阱 / 首元素聚焦）。 |
| `src/views/ClaudeCodeProfilesView.vue` `nextTick` 滚到编辑区 | 打开编辑器后滚到表单 | 本任务删除 Vue Profiles 页，改接 `ClaudeProfilesView` + `BaseProfiles`；不再需要 nextTick。 |
| `src/components/claude/ClaudeProfileEditorModal.vue` `nextTick` 注册 section ref | 弹层打开后测量分区 | Vue 编辑器随 Profiles 薄壳删除；共享层在 `src/components/profiles/`。 |
| `src/components/claude/ClaudeProfileRow.vue` `await nextTick()` | 菜单打开后定位 | 同上，行组件改由共享 `ProfileListRow` 承担。 |

本批次其余 Claude 视图（Home / Auth / Hooks / Statusline / Output Styles / Skills / Observer）无 `nextTick`。
