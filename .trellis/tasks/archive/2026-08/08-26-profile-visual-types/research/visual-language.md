# 操作面视觉类型（权威规格）

Operate 模式。服务对象是 AI CLI 操作员：密排扫描、快速动作、可信层级。不引入新视觉世界，不复活 Vue 时代的 glass / elevation / density / motion 修饰轴。

标注证据：`claude-profiles-annotation.png`（Claude Profiles，dark × clay）。

## 阅读顺序（页头）

1. 身份（glyph + 标题）
2. 主动作 `新建 Profile`
3. 次动作 Reload / Export / Edit source
4. 风险动作 Turn profile off（单独横幅，不进页头）

## Button

组件：`src/ui/button.tsx`。样式写在 `src/ui/primitives.css` 的 `.ui-btn` 族。另导出 `buttonClass()`，供 `<Link>` 与暂未改成组件的调用点共用。

| variant | 语义 | 外观 | 代表用法 |
| --- | --- | --- | --- |
| `primary` | 本页唯一主动作 | accent 实底，字色 inverted，字重 600 | 新建 Profile、保存、添加 |
| `secondary` | 中性实底次动作 | surface 底 + default 边 | Codex/Grok 现 `secondaryBtnClass` |
| `ghost` | 工具动作 | 透明底，subtle 边，muted 字；hover 升到 primary 字色 + elevated 底 | Reload / Export / Edit source |
| `quiet` | 行内弱动作 | 无边框、无底，muted 字；hover 才显底 | 卡片 Edit |
| `warning` | 可逆风险 | warning-tint 底 + warning 边 | Turn profile off；ConfirmModal `type=warning` 的确认按钮 |
| `danger` | 破坏性 | danger 实底 | ConfirmModal `type=danger` 的确认按钮、Grok danger CTA |
| `accent-soft` | 运行中态动作 | accent 软底 + 软边 | 运行中卡的 Apply / 停用 |

尺寸：`sm`（卡片/表格/chip 行）、`md`（页头与表单脚，默认）。同排按钮同高。

状态：default / hover / focus-visible / active / disabled。`active` 用 `scale(0.96)`，`prefers-reduced-motion: reduce` 时取消 transform。焦点环用 accent glow，不用高对比描边冒充主按钮。

禁止：`glass` variant、hover 放大超过 1、`transition: all`、硬编码 hex/px。

ConfirmModal 确认按钮按 `type` 映射，取消一律 `ghost`。禁止把 danger / warning / info 都映射成 `primary`。

| ConfirmModal `type` | 确认 `Button` variant | 取消 |
| --- | --- | --- |
| `danger` | `danger` | `ghost` |
| `warning` | `warning` | `ghost` |
| `info` | `primary` | `ghost` |

## Badge

组件：`src/ui/badge.tsx`。`.ui-badge` 族。

| 模式 | 交互 | 用法 |
| --- | --- | --- |
| `static` | 无 pointer、无 hover 抬升 | 卡片字段 AUTH / PROVIDER；卡片 tags；行状态徽章（`.cp-card__badge` / Running）；`record.badges`（如 Grok `profile_kind`） |
| `interactive` | pointer + hover + 可选 selected | 仅当该 chip 本身就是控件；**不含** QuickRail / Toolbar 筛选 pill（那些是专用控件） |

tone：`neutral` / `accent` / `warning` / `success`。默认 `neutral`。

静态 chip 的 padding 紧于 interactive，避免和字段 label 抢层级。

## FieldLabel

组件：`src/ui/field-label.tsx`。`.ui-field-label`。

- 字号 `0.75rem`（theme-token-contracts 登记的 Profiles 密排下限；废弃卡片里的 `0.625rem`）
- 字重 600，uppercase tracking `0.08em`（与 `page-header__eyebrow` 对齐）
- 颜色 `--color-text-muted`，不用 ghost

## UrlText

组件：`src/ui/url-text.tsx`。`.ui-url-text`。

- 展示用 `formatBaseUrlDisplay`（`src/utils/text.ts`）：保留 host，长路径截断
- `title` 为完整原始字符串
- mono `0.75rem`，颜色 `--color-text-secondary`；hover 才到 accent（仍不是链接）
- **不是** `<a>`，不复制、不外开。空值走调用方占位符，不渲染 UrlText

## 排版

- 字段 `dd` 是块级容器；chip / URL 是 `dd` 的子节点，不把 `ui-badge` class 直接打在 `dd` 上
- 2×2 字段网格里，URL 与 chip 第一行基线对齐
- 页头动作：次按钮一组，主按钮在最右；主按钮视觉重量必须明显高于 ghost

## 明确不抽

- 窗口控件、托盘、分页、`PillToggleGroup`、Configs `FilterChip`、Codex 账号卡 icon-only `ActionButton`
- 表格不增加第四数据列（Claude PROVIDER 仍只在卡片）
- 新 token 名（用已有 accent / warning-tint / text / border）
