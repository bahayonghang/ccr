# nextTick 登记表（AC8）

| 原调用点 | 意图 | React 改写 |
| --- | --- | --- |
| `CodexProfilesView.vue` `locateProfile` 内 `nextTick` 后 `querySelector` + 滚动高亮 | DOM 更新后再定位卡片 | Vue 已删除。Profiles 面由 `BaseProfiles` 承接，本任务不保留该定位动画。 |
| `ProfileCard.vue` `nextTick` | 卡片展开后量测 | Vue 已删除。共享 Profiles 层已在 `components/profiles`。 |
| `CodexProfileEditorModal.vue` 两处 `nextTick` | 打开后聚焦 / 同步编辑器 | Vue 已删除。编辑器由统一 Profiles 层承接。 |

Sessions / Auth / home / slash-commands 源文件无 `nextTick`。
