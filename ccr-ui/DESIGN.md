---
name: CCR UI
description: Airy-glass workbench for advanced AI CLI configuration, runtime monitoring, sync, and usage insight workflows.
colors:
  workbench-base: "#10191b"
  terminal-sidebar: "#172225"
  terminal-panel: "#1d292c"
  terminal-elevated: "#233033"
  paper-base: "#e7f1ee"
  paper-elevated: "#f2f7f5"
  paper-surface: "#ffffff"
  paper-overlay: "#e3ece8"
  green-command: "#85d5b1"
  green-command-hover: "#9ce0c3"
  green-command-light: "#156f4e"
  green-command-light-hover: "#146a4a"
  green-contrast: "#102b20"
  output-secondary: "#b9a4f4"
  status-ready: "#73ba6e"
  status-ready-light: "#356e45"
  status-warn: "#d9c05a"
  status-warn-light: "#7b5e16"
  status-attention: "#ec8d7a"
  status-attention-hover: "#f29f8e"
  status-attention-light: "#9f513f"
  status-info: "#91a9c7"
  status-info-light: "#42648a"
  ink-primary: "#f0f8f5"
  ink-secondary: "#c5d5d0"
  ink-muted: "#9fafa9"
  ink-light-primary: "#1e2932"
  platform-claude: "#d97757"
  platform-codex: "#7cab82"
  platform-grok: "#a79bc4"
  platform-gemini: "#7d97b6"
  platform-opencode: "#735f52"
  platform-antigravity: "#98afc9"
  platform-kimi-code: "#b65790"
  platform-pi: "#9773d0"
  platform-zcode: "#1d83a4"
  platform-deepseek-harness: "#1e8dfe"
  chart-other: "#a1937c"
  chart-other-light: "#968b76"
  chart-heat-0: "#394349"
  chart-heat-1: "#596971"
  chart-heat-2: "#7a8b91"
  chart-heat-3: "#9badb2"
  chart-heat-4: "#c0d0d3"
  chart-heat-0-light: "#e4e9e7"
  chart-heat-1-light: "#b0bdb7"
  chart-heat-2-light: "#8a9f95"
  chart-heat-3-light: "#647f72"
  chart-heat-4-light: "#405f50"
typography:
  headline:
    fontFamily: "SF Pro Display, Segoe UI Variable Display, PingFang SC, Microsoft YaHei UI, Microsoft YaHei, sans-serif"
    fontSize: "clamp(1.625rem, 1.4rem + 0.7vw, 2rem)"
    fontWeight: 600
    lineHeight: 1.12
    letterSpacing: "-0.022em"
  title:
    fontFamily: "MapleBright, SF Pro Text, PingFang SC, Microsoft YaHei UI, Microsoft YaHei, sans-serif"
    fontSize: "1.0625rem"
    fontWeight: 600
    lineHeight: 1.24
    letterSpacing: "0"
  body:
    fontFamily: "MapleBright, SF Pro Text, PingFang SC, Microsoft YaHei UI, Microsoft YaHei, sans-serif"
    fontSize: "1rem"
    fontWeight: 400
    lineHeight: 1.56
    letterSpacing: "0"
  label:
    fontFamily: "MapleBright, SF Pro Text, PingFang SC, Microsoft YaHei UI, Microsoft YaHei, sans-serif"
    fontSize: "0.8125rem"
    fontWeight: 500
    lineHeight: 1.24
    letterSpacing: "0"
  data:
    fontFamily: "Cascadia Code, Cascadia Mono, SFMono-Regular, ui-monospace, Consolas, MapleBright, monospace"
    fontSize: "0.8125rem"
    fontWeight: 500
    lineHeight: 1.3
    letterSpacing: "0"
    fontFeature: "'tnum' 1, 'cv11' 1, 'ss01' 1"
rounded:
  none: "0"
  sm: "6px"
  md: "6px"
  lg: "8px"
  xl: "12px"
  pill: "9999px"
spacing:
  xs: "0.25rem"
  sm: "0.5rem"
  md: "1rem"
  lg: "1.5rem"
  xl: "2rem"
  xxl: "4rem"
components:
  button-primary:
    backgroundColor: "{colors.green-command}"
    textColor: "{colors.green-contrast}"
    rounded: "{rounded.lg}"
    padding: "0.625rem 1rem"
  button-secondary:
    backgroundColor: "{colors.terminal-panel}"
    textColor: "{colors.ink-primary}"
    rounded: "{rounded.lg}"
    padding: "0.625rem 1rem"
  input-default:
    backgroundColor: "{colors.terminal-panel}"
    textColor: "{colors.ink-primary}"
    rounded: "{rounded.lg}"
    padding: "0.625rem 1rem"
  card-panel:
    backgroundColor: "{colors.terminal-panel}"
    textColor: "{colors.ink-primary}"
    rounded: "{rounded.xl}"
    padding: "1rem"
  nav-item-active:
    backgroundColor: "rgb(133 213 177 / 10%)"
    textColor: "{colors.ink-primary}"
    rounded: "{rounded.lg}"
    padding: "0.625rem 0.75rem"
  status-bar:
    backgroundColor: "{colors.workbench-base}"
    textColor: "{colors.ink-muted}"
    typography: "{typography.data}"
    height: "1.75rem"
    padding: "0.25rem 0.75rem"
---

# Design System: CCR UI

## Overview

**Creative North Star: "Airy Glass Workbench" (轻盈玻璃工作台)**

The approved 2026-10-26 `style:airy-glass` adaptation replaces the market-terminal appearance. Its source snapshot and CCR-specific decisions live in `.trellis/tasks/archive/2026-10/10-26-ccr-ui-airy-glass/research/catalog/DESIGN.md` and the parent `design.md`. The adaptation takes precedence where the catalog lacks dark mode, data visualization, or accessibility rules.

A static 160deg mint-to-lavender field surrounds one paper-like application sheet. Light mode uses white at 78%; dark mode uses a low-saturation deep field and a 90% sheet. The sheet alone may use `blur(20px) saturate(1.15)`. Cards, data wells, inputs, charts, sticky regions and all dialogs/popovers remain solid. The result must preserve the existing dense workflow, bounded charts, platform strip, Insights, and command status bar.

## Colors

### Primary and output

Green marks primary action, link, active route and focus: light #156f4e (hover #146a4a), dark #85d5b1 (hover #9ce0c3). The persisted `clay` accent key remains unchanged, including its legacy migration map; it now displays green. Purple (#6d4bd6 light / #b9a4f4 dark) is reserved for explicitly identified output. Do not convert unknown, pending or failed state to green or purple.

Success, warning, danger and info retain separate semantic roles with readable contrast ink. Red destructive actions and yellow warnings must remain distinguishable by text/icon, not just color. User custom accents retain the existing runtime extension behavior; heuristic contrast ink does not guarantee compliance for every arbitrary user color.

### Neutral reading surfaces

| Combination | base | elevated | surface | overlay | primary | secondary | muted |
|---|---|---|---|---|---|---|---|
| neutral light | #e7f1ee | #f2f7f5 | #ffffff | #e3ece8 | #1e2932 | #404e59 | #5c6b76 |
| clay light | #eae6f2 | #f5f2fa | #fefcff | #e9e4f1 | #222630 | #464758 | #686477 |
| neutral dark | #10191b | #172225 | #1d292c | #233033 | #f0f8f5 | #c5d5d0 | #9fafa9 |
| clay dark | #171722 | #20212e | #282a38 | #2c2e3d | #f5f4fc | #d4d2e3 | #b0adbf |

These are solid shared roles. Only dedicated `--surface-app-sheet-*` tokens may introduce translucency. Measure primary ≥12:1, secondary ≥7:1, muted ≥4.5:1 against their actual rendered backgrounds, including the sheet composited over gradient endpoints, midpoint and worst position. Normal text requires ≥4.5:1 and meaningful control boundaries/state indicators ≥3:1. A pale sheet outline cannot substitute for an input boundary.

### Platform and chart identity

Existing `--color-platform-*` identities remain categorical: 2px ticks, dots, legend swatches and chart segments. Do not repaint panels in platform colors or map all sources to green and purple. Keep source text and dot contrast checks against all four card faces. Multi-source charts require segment gaps, labels/legend, per-segment tooltip and the exact table view, because the source palette is not pairwise distinct.

Heatmaps and distribution use a monotonic neutral `--color-chart-heat-*` ramp with heat-0 ≥1.15:1 against all card faces. Low-contrast cells retain table access to exact values. A seven-day change remains neutral secondary ink; it is not a success/failure signal.

## Typography

**Sans Font:** MapleBright with SF Pro Text and system CJK fallbacks (PingFang SC, Microsoft YaHei UI).
**Brand/Display Font:** SF Pro Display with Segoe UI Variable Display and the same CJK fallbacks.
**Mono Font:** Cascadia Code, Cascadia Mono, SFMono-Regular, ui-monospace, Consolas — a true monospace stack, earned by data contexts (metrics, versions, timestamps, axes, the command status bar).

**Character:** A quiet sans for prose and labels, a proportional display face for page headers, and a real mono for the workbench's numeric voice. Hierarchy comes from weight, size, and spacing — not decorative faces.

### Hierarchy

- **Display** (600, clamp(1.625rem, 1.4rem + 0.7vw, 2rem), 1.12, tracking -0.022em): page-level headers on workbench surfaces.
- **Section** (600, 1.0625rem, 1.24): panel and card titles; one title per panel, no duplicate eyebrows.
- **Body** (400, 1rem, 1.56; 0.875rem on dense home surfaces): descriptions, hints, messages. Cap long prose at 65–75ch.
- **Label/Meta** (500–600, 0.8125rem / 0.6875rem): form labels, metadata, eyebrows. Wide tracking (0.14em) only for short Latin mono labels.
- **Data** (500–600 mono, 0.8125–1.125rem, tabular-nums): every numeric readout, version string, clock, and route index.

### Named Rules

**The Mono Is Earned Rule.** Mono appears where the workbench shows data: data, code, time, commands. Prose stays sans; never set paragraphs in mono for atmosphere.

**The Tabular Number Rule.** Any number a user compares or watches uses tabular figures (`font-variant-numeric: tabular-nums`). Columns of digits must not jitter.

**The Product Type Rule.** Fixed rem scales inside app surfaces; no viewport-fluid hero type beyond the one display clamp.

## Layout

Desktop-first workbench: a persistent nav rail, scrollable content column, and surfaces composed on a 4px spacing base (`--space-*`). Section gap 1.25rem, panel padding 1rem.

The overview first viewport is the world's signature composition: a full-width platform quote band on top (horizontal scroll under narrow widths — it never wraps), a main usage panel whose chart height is bounded, an event stream rail, and a bottom command status bar pinned to the scroll container. Data surfaces use fixed row heights and aligned character grids; resizing must not reflow a table into a card pile.

Reference breakpoints: 640 / 768 / 1024 / 1280 / 1536px. Below the `lg` breakpoint the sidebar becomes a modal with an explicit backdrop and close control.

## Elevation & Depth

The sole blurred surface is the non-scrolling application sheet. It has a 20px radius, one shadow and a subtle outline; its static field remains visible around the paper. Do not scale the whole sheet, animate blur, add drifting blobs or nest glass. Shared `--surface-shell-*`, `--surface-workspace-*`, `--surface-card-*`, `--surface-status-*` and `--surface-modal-*` remain solid/no-blur. Modal scrims may remain translucent without backdrop blur.

**The Glass Budget Rule.** Count at most one visible computed backdrop-filter, on the app sheet only. Below 960 CSS px, under `prefers-reduced-transparency: reduce`, or without backdrop-filter support, the sheet uses alpha=1 and blur=none. The 960px material breakpoint is independent of the existing 1024px navigation breakpoint. Test both sides of both boundaries.

Keep the existing scroll owner, cache/Outlet lifetime, resize hit areas, portal positioning, focus trapping, Esc, focus restoration and mobile navigation behavior. The separate `/tray/codex` window uses a compact solid surface with its original dimensions and responsibilities.

## Shapes

The dedicated app sheet uses 20px. Static reading wells may use 16px where appropriate; dense controls keep their existing heights and 8px corners, and existing panels may remain 12px. The shared radius scale and virtual row estimates are unchanged. Pill shapes are suitable for independent actions or labels, not tables or whole pages. Preserve the 2px identity/active tick, tabular figures and dashed empty baselines.

## Components

### Buttons

- **Shape:** 8px corners, padding-defined height (0.625rem 1rem), 0.8125rem medium-weight label.
- **Primary:** solid green fill with theme-specific contrast ink; hover follows the accent token. No gradient, no shadow stack.
- **Secondary / Ghost / Danger:** secondary is panel fill with a default hairline border; ghost is quiet text that gains an overlay wash on hover; danger is solid Attention Red with its contrast ink.
- **State:** active presses with scale(0.98); disabled drops to 50% opacity and must still be legible at a glance. Focus uses the green focus ring.

### Inputs / Fields

- **Style:** 8px corners, panel background, 1px default hairline, 0.625rem 1rem padding.
- **Focus:** green border shift plus the focus ring; never decorative glow.
- **Error / Disabled:** error uses Attention Red plus `aria-invalid`; disabled reduces opacity while labels stay readable.

### Cards / Panels

- **Corner Style:** 12px on workbench panels.
- **Background:** opaque panel token with a 1px subtle hairline border; internal padding 1rem on dense dashboards, up to 2rem on large configuration panels.
- **Shadow Strategy:** none at rest; hover may shift background one ladder step.

### Navigation

Persistent sidebar items use 8px corners and quiet secondary text. The active route reads as a 10% green tint with a 24% green border and primary text — state conveyed by tint, not by a filled bar. Mobile collapses to a modal sidebar with backdrop.

### Platform Quote Band (signature)

A single hairline-ruled strip holding every platform cell: 1px separators between cells, a 2px platform-color top tick per cell, mono tabular metrics with 0.14em-tracked micro labels. Under narrow widths it scrolls horizontally rather than wrapping. Sparkline bars render neutral with the peak bar in the platform color; an empty series shows a dashed hairline baseline; unindexed sessions render a muted `—`, never a misleading 0.

### Bounded Usage Chart (signature)

Stacked platform segments on a hairline baseline with gridlines at 25/50/75%. Height is hard-bounded (`clamp(10rem, 26vh, 16rem)`) — the chart informs the band, it never becomes a hero. Length is quantity; skeletons pulse in place; the hero metric reads in mono.

### Event Stream (signature)

One section title, no duplicate eyebrow. Rows align a mono tabular timestamp, a severity dot, a level tag, a channel, and a single-ellipsis message. Severity color lives on the dot and level only (errors additionally lift the message to primary ink). The list caps at 16rem and scrolls; the empty state is a dashed hairline panel with a concrete next action.

### Bottom Command Status Bar (signature)

Sticky to the scroll container's bottom edge: a 1px hairline top rule over the base background, mono meta-size text, items separated by hairlines. It carries backend status (ok in green, pending in warn, error in attention red), the last event's severity dot, 01–04 numbered route hints whose indices brighten green on hover, and a tabular clock.

### Settings Dock (signature)

The sidebar's settings anchor: a card with two rows — a title row and a mono meta row ending in the app version — both single-line ellipsis with a native tooltip for the full string. Active state is the overlay background plus a 2px green left tick.

### Insights Block (signature)

The home Insights block sits below the first viewport. It shares the first viewport's scroll container and the bottom command status bar. The first-viewport composition does not change. Source of the decisions: the `home-insights-design` task's `design.md` §6.

**Shared rules.**

- Order, top to bottom: section header row, totals readout, activity heatmap beside the 7-day comparison, weekly requests by source, time distribution beside the leaderboard.
- Panels reuse the first-viewport usage panel: surface background, 1px subtle hairline border, `--home-card-radius`, `--home-card-pad`. The gap between blocks is `--home-section-gap`.
- The heatmap and the 7-day comparison share one row at the `.dashboard-lower` column ratio `1.85fr : 1fr` (heatmap : 7-day comparison). The distribution and the leaderboard share one row in two equal columns. Below 1024px every paired row becomes a single column.
- Chart gridlines sit at 25/50/75%. Bars have no corner radius. Every bar and cell carries a `title` and a hover tooltip. A chart that shows 4 or fewer series adds direct labels in addition to its legend.
- Charts are plain DOM and CSS. Do not add a chart dependency.
- Large numbers use the same compact formatter as the first viewport (`compactLabel`). It formats with the runtime default locale (the WebView system language), not with the app UI language: a Chinese system shows 万 / 亿, an English system shows K / M / B. The full value goes in the `title`.
- View switches are segmented controls (`role="radiogroup"`). Arrow keys move the selection. The focus ring is visible.

**Section header row.**

- Form: title "使用洞察", a one-line description, and the date range in mono on the right.
- Height: natural height.
- Colors: text tokens only.
- Empty: always rendered, in every empty state.

**Totals readout.**

- Form: one full-width reading line, not cards. A 1px `--color-border-subtle` hairline runs above and below it. Six groups of label + value share one baseline: sessions, tokens, requests, agents, projects, active days. A 1px vertical hairline separates the groups. The label is sans 0.8125rem muted ink. The value is mono 1.125rem semibold primary ink with tabular figures.
- Height: one line.
- Colors: text tokens and `--color-border-subtle`. No panel fill, no cells, no 2px tick, no mini bars. The readout must not look like the platform quote band.
- Empty: a value with no data shows `—`. An unindexed session count shows `—` with `title="会话尚未索引"`.

**7-day comparison.**

- Form: a table inside a panel. Columns: metric, last 7 days, previous 7 days, change. Rows: sessions, requests, active days.
- Height: header row 1.5rem, then 3 rows at a fixed 2.25rem each.
- Change format: sign plus text — `+21.7%`, `−17.6%`, `+2 天`, `持平`. When the previous 7 days are 0, write `新增`.
- Colors: the change column stays secondary ink. Change is not a status, so it gets no status color.
- Empty: with no requests, only the sessions row has values. With sessions not indexed, the sessions row shows `—`.

**Activity heatmap.**

- Form: 53 week columns × 7 day rows of daily request counts, in 5 steps. Step 0 means no activity. A month label row sits above the grid. A caption line below gives the current streak, the longest streak, and the busiest day. A "少 → 多" legend shows the steps. A segmented control in the panel header switches between the chart view and the table view (图表 / 表格); the chart view is the default.
- Size: cell width scales with the container between 0.5rem and 0.875rem. The cell gap is `--space-0-5`. When the cell width would go below 0.5rem, the container scrolls horizontally and opens scrolled to the latest week. Height is the 7 cell rows plus the month row plus the caption line.
- Colors: `--color-chart-heat-0` … `--color-chart-heat-4`, a neutral sequential ramp. Heat-0 is the no-activity cell fill only; it stays visibly apart from the card face in all four theme × flavor combinations (≥1.15:1). One value per theme serves both flavors. Heat-1 is below 3:1 against the surface. The table view is the relief channel for colors below 3:1 (dataviz rule: a tooltip alone is not enough). Each cell's `title` / tooltip also gives the date and the value.
- Table view: one row per week, newest week first (53 rows). Columns: week start date (mono), Monday … Sunday, week total. Cells show full request counts with grouping separators in mono tabular figures. A day after `as_of` is an empty cell. The table height is `clamp(10rem, 26vh, 16rem)`, with a sticky header row, fixed 2rem rows, and vertical scroll. The caption line stays below the table.
- Empty: when there are no requests, every cell is heat-0. A caption "导入用量后显示" and an "导入用量" button follow.

**Weekly requests by source.**

- Form: a full-width panel with one stacked bar per week (53 bars) of request counts by source. A segmented control in the panel header switches between the chart view and the table view (图表 / 表格); the chart view is the default.
- Merge rule: the top 4 sources by total over the 53-week window get their own color. Ties follow `SourceKind::ALL` order. All other sources merge into one "其他" segment. The top 4 are fixed for the whole chart; color follows the source, not the rank.
- Stack order, bottom to top: claude, zcode, grok, deepseek_harness, antigravity, kimi_code, opencode, codex, pi; "其他" is always on top. The legend uses the same order.
- Height: the chart is `clamp(10rem, 26vh, 16rem)`, the same bound as the first-viewport chart.
- Colors: `--color-platform-*` per source. The "其他" segment is `--color-chart-other` with a 45° stripe texture: `repeating-linear-gradient(45deg, var(--color-chart-other) 0 0.1875rem, var(--color-bg-surface) 0.1875rem 0.3125rem)`.
- Secondary encoding (required, because the nine source colors are not all pairwise distinct): a `--space-0-5` surface-color gap between segments, the legend, direct labels when the chart shows 4 or fewer series, a tooltip per segment, and the table view. The table view is the relief channel for colors below 3:1 (dataviz rule: a tooltip alone is not enough).
- Table view: one row per week, newest week first (53 rows). Columns: week start date (mono), one column per shown series in stack order (the top 4 sources, then 其他), and the week total. Each source column header carries the 2px platform-color tick and the source name. Cells show full request counts with grouping separators in mono tabular figures. The table height is the chart height `clamp(10rem, 26vh, 16rem)`, with a sticky header row, fixed 2rem rows, and vertical scroll.
- Empty: not rendered when there are no requests.

**Time distribution.**

- Form: one panel with three views — hour, weekday, month — selected by a segmented control.
- Height: the chart is a fixed 10rem.
- Colors: bars `--color-chart-heat-2`; the peak bar `--color-chart-heat-4`.
- Empty: not rendered when there are no requests.

**Leaderboard.**

- Form: one panel. A segmented control selects the dimension (Agent / 项目 / 模型). A second segmented control selects the metric (请求 / Token / 会话). The session metric is available for Agent only. When sessions are not indexed, the session option is disabled and has a `title` that gives the reason.
- Height: fixed 2rem rows, 8 rows at most.
- Colors: an Agent row has a 2px platform-color tick and a 0.375rem-high platform-color bar on a `--color-bg-overlay` track. Project and model rows use a neutral `--color-chart-heat-2` bar.
- Empty: with no requests, the default metric is sessions.

**Empty states.**

- Empty store (no requests and no sessions): the header row, the totals readout with every value `—`, and one dashed panel (`1px dashed --color-border-default`) with "还没有可分析的记录", a short description, and the primary "导入用量" button. The button is green because it is the next command.
- Sessions only (no requests, `sessions_indexed = true`, sessions present): request values show `—`. The heatmap follows its empty rule. The 7-day comparison has values in the sessions row only. The leaderboard defaults to the session metric. The weekly chart and the distribution are not rendered.
- Usage only (`sessions_indexed = false`): every session count shows `—` with `title="会话尚未索引"`. One info dot plus an explanation line sits below the totals readout. The session metric in the leaderboard is disabled. All other blocks render as usual.
- Web preview (not the Tauri runtime): the header row plus one solid panel with an info dot, "Web 预览不读取本机数据", and a short description. The block makes no IPC call.

### Boot Loader

The `index.html` boot screen, shell critical CSS, flavor preview swatches and macOS window background mirror the four built-in palette combinations. Retain the bootstrap migration maps, system-theme resolution, font overrides, native macOS-only lazy import and `setTheme` behavior. No transparent native window or new permission is introduced. Mock native tests are not macOS device evidence.

### Copy & Localization

zh-CN is fully globalized and is the primary copy voice. Components subscribe to translation via `useAppT()` — never a bare captured `t`. The locale leaf count is governed at 4501 (`scripts/check-i18n.mjs`, enforced by `tests/i18n.test.cjs`); add keys through the contract, not ad hoc. Single-line strings truncate with ellipsis and carry a tooltip.

## Do's and Don'ts

### Do:

- **Do** reserve green for command, active route, and focus — and keep it under 10% of the screen.
- **Do** keep green and red on status dots, values, and tints only, paired with their contrast ink.
- **Do** set every numeric readout, timestamp, version, and route index in mono with tabular figures.
- **Do** separate surfaces with hairlines and tonal steps before shadows.
- **Do** render honest empties: dashed baselines, `—` placeholders, muted unindexed metrics — never a fabricated zero.
- **Do** keep charts bounded (`clamp(10rem, 26vh, 16rem)`) and data rows at fixed heights.
- **Do** subscribe copy through `useAppT()` and keep the locale contract green.
- **Do** honor `data-reduced-motion`: micro-interactions collapse to 0ms and lifts flatten.
- **Do** give the Insights weekly chart a secondary encoding: segment gaps, a legend, direct labels at 4 or fewer series, a tooltip per segment, the stripe texture on "其他", and a table view.
- **Do** give the Insights weekly chart and the activity heatmap a chart / table switch; the table view gives exact values for colors below 3:1.
- **Do** keep the top 4 sources and the stack order fixed across the whole weekly chart; color follows the source, not the rank.
- **Do** give every Insights chart a fixed or clamped height, and every Insights table row a fixed height.
- **Do** write a 7-day change as sign plus text (`+21.7%`, `−17.6%`, `+2 天`, `持平`, `新增`) in secondary ink.

### Don't:

- **Don't** rebuild the metric-card row plus unbounded hero chart — that dashboard cliché is the rejected rut.
- **Don't** repaint panels or chrome in platform colors; identity stays on 2px ticks and swatches.
- **Don't** add decorative glass, nested glass, or glass inside scrolling content.
- **Don't** add gradients inside reading surfaces, extra blurred layers, guofeng, neko, anime or mascot variants; the single mint-to-lavender field is the approved scene.
- **Don't** widen identity ticks, add gradient text or spread the sheet's 20px radius onto dense controls.
- **Don't** use fluid hero typography or decorative fonts inside app surfaces.
- **Don't** capture a bare `t` outside `useAppT()`, and don't leave zh-CN copy unglobalized.
- **Don't** hide disabled, blocked, or destructive states — they must read at a glance.
- **Don't** color a 7-day change green or red; the change is not a status.
- **Don't** use green or status colors in the heatmap ramp or the distribution bars; they use `--color-chart-heat-*` only.
- **Don't** add a chart library for Insights; charts are plain DOM and CSS.
- **Don't** build the Insights totals as metric cards or as a copy of the platform quote band; it is one hairline-ruled reading line.
- **Don't** give each of the nine sources its own weekly-chart color; merge everything past the top 4 into "其他".
- **Don't** use platform colors on project or model rows in the leaderboard; only Agent rows carry the platform tick and bar.
