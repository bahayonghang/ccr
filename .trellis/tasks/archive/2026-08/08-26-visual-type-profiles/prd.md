# Profile 列表采用视觉类型

父任务：`08-26-profile-visual-types`
依赖：`08-26-visual-type-primitives`
规格：`../08-26-profile-visual-types/research/visual-language.md`
标注：`../08-26-profile-visual-types/research/claude-profiles-annotation.png`

## Goal

把共享 Profile 呈现层接到全局原语，使页头、Off 横幅、卡片字段和表格 URL/枚举的层级与标注截图一致。Claude / Codex / Grok 一起变。

## Requirements

- R1：`ProfilesPageHeader` 新建为 `Button variant="primary" size="md"`，Reload / Export / Edit source 为 `ghost`。
- R2：`ProfilesOffBanner` 动作为 `warning`；容器背景 `var(--color-warning-tint)`、边框 `var(--color-warning)`。确认仍走 `surfaceNotify.confirm({ type: 'warning' })`。按钮 class 不得当作容器已改的证据。
- R3：`ProfileFieldSlot.chip?: boolean` 改为 `kind?: 'text' | 'url' | 'chip'`。Claude 第四槽 `provider` 为 `chip`；三平台 Base URL 为 `url`；auth 为 `chip`；model 为 `text`。
- R4：卡片字段：`dt` 用 `FieldLabel`；`url` 用 `UrlText`；`chip` 用 `Badge mode="static"`。`dd` 只做布局容器。
- R5：卡片 Edit 为 `quiet` + `sm`；Apply 非运行中 `ghost sm`，运行中 `accent-soft sm`。Overflow 触发器保持 icon 按钮，可用 `quiet sm`。
- R6：表格 `slots[0]` 用 `UrlText`；`slots[2]` 在 `kind==='chip'` 时用 static Badge。不渲染 `slots[3]`。
- R7：空态、编辑器脚（`ProfileEditorModal` / `ProfilesEmptyState`）与 `ProfilesHeader.tsx` 改用 `Button`。删除对本页 `.cp-btn` / `.pe-btn` 的依赖，禁止 alias。`ProfilesHeader` 即使无生产消费方也必须迁移。
- R8：卡片 tags 用 static Badge；QuickRail / Toolbar pill **不改**。
- R9：删除 `profiles-shared.css` / `profile-editor-shell.css` 里的 `.cp-btn` / `.pe-btn` 规则（不是改成 `.ui-btn` 别名）。禁止再定义一套按钮色板。
- R10：测试在 `tests/profiles/`：kind 映射、URL 展示、Off 按钮与容器、页头变体、状态徽章与 `record.badges`。走查 8 组合，**每行必查项 PASS** 才算过；格式见 design.md。
- R11：无平台名分支；无硬编码 hex。
- R12：行状态徽章（现 `.cp-card__badge`）改为 `Badge mode="static"`；`state.badge.tone === 'accent'` 时 `tone="accent"`，否则 `neutral`。
- R13：`record.badges` 改为 `Badge mode="static"`，`tone` 取 `badge.tone`（`neutral` | `accent` | `warning`）。不得再使用带 pointer 的 `.cp-chip`。

## Acceptance Criteria

- [ ] AC1（R1、R2）：测试能查到页头 primary + 三个 ghost，Off 横幅 button 带 `.ui-btn--warning`。Off 横幅容器的计算背景使用 `--color-warning-tint`（或 stylesheet 声明 `background`/`border-color` 为这两个 token），与按钮 class 分开断言。
- [ ] AC2（R3、R4）：Claude 夹具卡片上 Base URL 节点有 `title` 为完整 URL；AUTH 与 PROVIDER 为 static badge；MODEL `-` 不是 badge。
- [ ] AC3（R5）：运行中卡 Apply 为 accent-soft，非运行中为 ghost。
- [ ] AC4（R6）：表格第一数据列为 UrlText，第三数据列对 auth 为 badge；DOM 中无第四字段列。
- [ ] AC5（R7）：编辑器保存主按钮为 primary；取消为 ghost。`ProfilesHeader` 渲染出的动作按钮为 `.ui-btn`，文件内无 `cp-btn`。
- [ ] AC6（R8、R12、R13）：`.cp-chip--switch` 仍在 QuickRail。卡片 tags、行状态徽章、`record.badges` 均为 static Badge，computed `cursor` 不是 `pointer`。运行中状态徽章 `tone=accent`。
- [ ] AC7（R9）：`rg "cp-btn|pe-btn" ccr-ui/src/components/profiles` 无 className 或 CSS 选择器命中（注释中的「已迁移」除外）。不存在 `.cp-btn { }` 指向 `.ui-btn` 的 alias 规则。
- [ ] AC8（R10）：`notes.md` 含下方 8 行走查表，每行 `result=PASS`；任一 FAIL 则本条失败。`just frontend-check-quick` 通过。
- [ ] AC9（R11）：`tests/quality` 或 platform-surface-unify 无平台名分支仍通过。

## Out of scope

- 其它 features 的按钮
- 表格加列
- URL 点击行为
- QuickRail / Filters pill 重写

## Notes

- `ProfilesHeader.tsx` 必须迁移，即使当前只有 barrel / 测试消费。
- 走查失败不得只写「已记录」；AC8 要求全 PASS。
