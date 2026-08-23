# nextTick 登记（R8）

| 原调用点 | 原用途 | React 改写 |
| --- | --- | --- |
| `components/grok/GrokProfileCard.vue` `await nextTick()` 后聚焦菜单项 | 菜单打开后聚焦第一个 menuitem | `useEffect` 在 `menuOpen` 为 true 时查询 menuitem 并 `focus()` |
| `components/grok/GrokProfileEditorModal.vue` `void nextTick(() => scrollTo / setupObserver)` | 打开模态后滚到顶部并建立 IntersectionObserver | 段导航改为 tab 切换，不再滚动 spy，去掉 nextTick |
| `views/grok/GrokProfilesView.vue` `void nextTick(() => querySelector locate)` | Inspector locate 滚动到卡片 | 本批次未迁 Inspector locate；卡片带 `data-profile-name` 供后续接线 |

其余本域 Vue 文件无 `nextTick`。
