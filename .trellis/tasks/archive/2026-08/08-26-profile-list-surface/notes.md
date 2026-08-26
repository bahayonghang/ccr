# 08-26-profile-list-surface notes

## 在线旧类名（重写 `profiles-shared.css` 时保留）

`ProfileListRow` / `ProfilesHeader` / `ProfilesSection` / QuickRail / Inspector / Palette / Raw editor 仍在使用：

- 行：`cp-row` `cp-row--active` `cp-row--off` `cp-row__dot` `cp-row__name` `cp-row__label` `cp-row__url` `cp-row__model` `cp-row__meta` `cp-row__tags` `cp-row__actions`
- 页头：`cp-header` `cp-header__icon` `cp-header__back` `cp-btn` `cp-btn--ghost` `cp-btn--primary` `cp-btn--palette-open` `cp-btn__kbd` `cp-menu` `cp-menu__pop` `cp-menu__item`
- 工具条：`cp-toolbar` `cp-search` `cp-search__input` `cp-search__kbd` `cp-pill` `cp-pill--active` `cp-filters` `cp-filters__pop` `cp-seg`
- 轨道：`cp-rail` `cp-chip` `cp-chip--switch` `cp-chip__name` `cp-chip__kbd` `cp-chip__pin`
- Inspector / Diff / Section / Raw：既有 `cp-inspector*` `cp-diff-*` `cp-section*` `profiles-raw-*`

## 共享原子类（供 editor 子任务对接）

- 按钮：`cp-btn` `cp-btn--ghost` `cp-btn--primary` `cp-btn--accent-soft`
- chip / pill：`cp-chip` `cp-pill` `cp-pill--active`
- 表单：`cp-label` `cp-input`
- 搜索：`cp-search__input`

## source mode 映射

| 契约 | 落地 |
| --- | --- |
| 进入前明文警告 | `ProfilesSurface.enterSource` → `surfaceNotify.confirm({ type: 'warning' })`，拒绝则不挂载面板 |
| version token | `ProfilesRawEditorPanel` 从 `getRaw()` 保存，`saveRaw(content, token, force?)` 原样回传 |
| `conflict` | 面板只渲染重载 + 关闭/取消，无覆盖写入 |
| `activation_conflict` | 面板 `requestConfirm({ type: 'danger' })` 后以同一 content/token `force: true` 重试 |
| `invalid` | 透传到 `errorMarker` |
| 保存后顺序 | 面板 `setBaseline` 清 dirty → `onSaved` → Surface `setSourceMode(false)` → `refreshAll()` |

## 搜索热键

仓库绑定是 `/`（`useProfilesHotkeys`），工具栏提示为 `/`，不照抄设计稿 ⌘K。

## 900×800 表格滚动（jsdom 断言）

测试将 `.cp-table-scroll` mock 为 `scrollWidth=1024` / `clientWidth=900`，`document.body` 为 `900/900`。CSS：表格 `min-width: 1024px`，容器 `overflow-x: auto`。真实浏览器走查数字补到 rollout 前的视觉验收；本任务 smoke 断言关系成立。

## AC18 旧页面

`BaseProfiles` 仍渲染 `ProfilesHeader` + `ProfileListRow`，继续消费上表旧类名。本任务未删除这些规则。Claude/Codex 旧页走查在 rollout 接线前保持可用；详细 8 组合视觉记录在父任务发布门补。

QuickRail React 夹具因 JSX 落在 `tests/profiles-quick-rail.smoke.test.tsx`（vitest include 覆盖 `.ts(x)`）。
