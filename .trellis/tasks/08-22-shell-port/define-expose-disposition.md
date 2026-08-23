# defineExpose 判定（R9）

全仓 8 处 `defineExpose`。本任务范围内的改写如下；其余归阶段 5 视图子任务。

| 位置 | 暴露 API | 判定 | 落地 |
| --- | --- | --- | --- |
| `components/common/BaseModal.vue` `close()` | 父组件命令式关闭 | 受控属性优先；命令式关闭保留为 ref 句柄 | 已有 `src/ui/base-modal.tsx` `BaseModalHandle.close()`。外壳消费方（ConfirmModal / UpdateModal / Titlebar About）全部走 `modelValue` 受控，不使用 ref。 |
| `components/ui/Input.vue` `focus()` | 父组件聚焦 | 不在本任务移植 Input | 归视图子任务；shadcn Input 用原生 ref / `autoFocus`。 |
| `components/claude/ClaudeProfileEditorModal.vue` `scrollToSection` | 父调用滚动 | 超出本任务 | `08-22-views-claude` |
| `components/codex/CodexProfileEditorModal.vue` `scrollToSection` | 同上 | 超出本任务 | `08-22-views-codex` |
| `components/profiles/ProfilesToolbar.vue` `focusSearch` | 父聚焦搜索框 | 超出本任务 | `08-22-views-profiles-config`；建议受控 `autoFocus` 或把 input ref 留在页面。 |
| `views/checkin/components/AccountActionsMenu.vue` `open/toggle/close` | 命令式菜单 | 超出本任务 | `08-22-views-checkin`；建议受控 `open`。 |
| `views/checkin/components/AccountFormModal.vue` `open` | 命令式打开 | 超出本任务 | `08-22-views-checkin`；受控 `isOpen`。 |
| `views/SshManagementView.vue` `sshListKeys/discoveredKeys` | 暴露状态 | 超出本任务 | `08-22-views-sync-tools`；改为提升状态或 Query。 |

本任务范围内：**没有新增 `useImperativeHandle` 消费点**。弹层全部受控。
