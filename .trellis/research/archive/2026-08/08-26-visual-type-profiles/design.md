# Profile 采用 — 设计

## 改动面

- `ccr-ui/src/configs/profilePresentation.ts`：`kind` 取代 `chip`
- `ProfileCardGrid.tsx` / `ProfileTable.tsx`：按 kind 渲染；状态徽章与 `record.badges` 改 static Badge
- `ProfilesPageHeader.tsx` / `ProfilesOffBanner.tsx` / `ProfilesEmptyState.tsx` / `ProfileEditorModal.tsx` / `ProfileOverflowMenu.tsx` / `ProfilesHeader.tsx`
- `profiles-shared.css` / `profile-editor-shell.css`：删除 `.cp-btn` / `.pe-btn`；Off 横幅改 warning 表面
- 测试：现有 `tests/profiles/*surface*` 更新选择器；新增 kind / URL / Off 容器 / 徽章断言

## 字段渲染

抽一个 `ProfileFieldValue`（放 `components/profiles/`，不是 `src/ui`）按 kind 分支。平台名不得出现在该组件。

行状态徽章不走 `ProfileFieldValue`：在 `cp-card__top` 把 `.cp-card__badge` 换成 `<Badge mode="static" tone={state.badge.tone === 'accent' ? 'accent' : 'neutral'}>`。`record.badges` 换成 `<Badge mode="static" tone={badge.tone}>`。

## 排版

- `.cp-card__field dt` 不再自设 0.625rem；改 FieldLabel
- `.cp-off-banner` 的 `background` 为 `var(--color-warning-tint)`，`border-color` 为 `var(--color-warning)`。验收读容器样式，不读按钮 class。
- 页头 `.cp-page-header__actions` 保持 flex + gap；主按钮已经在最后，不要改顺序

## 走查

夹具：`tests/fixtures/profiles.ts`。组合 8 个：viewport `1440x900` | `900x800` × theme `light` | `dark` × flavor `neutral` | `clay`。

`notes.md` 必须是下表，缺行或 `result` 非 PASS 则 AC 失败。截图放到 `ccr-ui/tests/__screenshots__/visual-types-{theme}-{flavor}-{w}x{h}.png`（已 gitignore，路径仍要写进表）。

必查项（每格 PASS/FAIL）：

1. `header`：主按钮 visually 重于 ghost（实底 vs 透明弱边）
2. `off-surface`：容器 warning-tint + warning 边，且按钮为 warning
3. `fields`：Claude 四字段为 URL / 纯文本 / chip / chip
4. `url-overflow`：长 URL 不撑破卡片网格列
5. `status-badge`：Running 徽章为 static（非 pointer）

| id | viewport | theme | flavor | header | off-surface | fields | url-overflow | status-badge | result | screenshot |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| W1–W8 | … | … | … | PASS/FAIL | … | … | … | … | 五格皆 PASS 才为 PASS | 路径 |

`result` 列：五格皆 PASS 则为 PASS，否则 FAIL。AC 要求 8 行 `result=PASS`。Web 预览：`cd ccr-ui && bun run dev:web -- --host 127.0.0.1 --strictPort`，`/claude-code/profiles`。
