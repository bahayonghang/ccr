---
name: CCR UI
description: Market-terminal operating surface for advanced AI CLI configuration, runtime monitoring, sync, and usage insight workflows.
colors:
  terminal-base: "#100f0c"
  terminal-sidebar: "#171410"
  terminal-panel: "#1f1b14"
  terminal-elevated: "#2a251b"
  paper-base: "#e9e4d8"
  paper-elevated: "#f2eee3"
  paper-surface: "#faf7ec"
  paper-overlay: "#ddd5c2"
  amber-command: "#f0a32b"
  amber-command-hover: "#f5b14a"
  amber-command-light: "#8f650e"
  amber-command-light-hover: "#a2740f"
  amber-contrast: "#1d1204"
  sand-secondary: "#c9a35f"
  status-ready: "#5fa05a"
  status-warn: "#d9c05a"
  status-attention: "#e06852"
  status-attention-hover: "#ec7a63"
  status-attention-light: "#9f513f"
  status-info: "#7d94b0"
  ink-primary: "#e9e1d1"
  ink-secondary: "#c9bda8"
  ink-muted: "#a1937c"
  ink-light-primary: "#211c12"
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
  chart-heat-0: "#332f27"
  chart-heat-1: "#5b5344"
  chart-heat-2: "#7a7263"
  chart-heat-3: "#9b9383"
  chart-heat-4: "#beb5a4"
  chart-heat-0-light: "#ebe5db"
  chart-heat-1-light: "#b4ab9b"
  chart-heat-2-light: "#958d7d"
  chart-heat-3-light: "#776f60"
  chart-heat-4-light: "#5b5344"
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
    backgroundColor: "{colors.amber-command}"
    textColor: "{colors.amber-contrast}"
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
    backgroundColor: "rgb(240 163 43 / 10%)"
    textColor: "{colors.ink-primary}"
    rounded: "{rounded.lg}"
    padding: "0.625rem 0.75rem"
  status-bar:
    backgroundColor: "{colors.terminal-base}"
    textColor: "{colors.ink-muted}"
    typography: "{typography.data}"
    height: "1.75rem"
    padding: "0.25rem 0.75rem"
---

# Design System: CCR UI

## Overview

**Creative North Star: "The Market Terminal" (行情终端)**

CCR UI is a never-sleeps operator terminal for AI CLI toolchains: every managed platform is a quote line, and state, data, and commands share one character grid. The world was user-picked in the 09-03 direction round (concept seed `19fe1fa0`, kind=pick, mode=operate), overriding the dice-assigned train-diagram world, and it deliberately rejects the category-default dashboard of a metric-card row plus an unbounded hero chart.

The surface is a warm-black phosphor terminal, dark-first because operators watch it for hours in low light. Amber is the command color — it marks the next action, the active route, and the focus ring, nothing else. Green and red are status semantics only. Monospace type is earned by data: numbers, versions, timestamps, and the command status bar. The light theme is a warm paper reading surface built from the same geometry, not a separate design.

Three disciplines survived from declined challengers and bind the whole world: color lives only on status hairlines; disabled and blocked states must read at a glance; chart dimensions stay honest (length is quantity).

**Key Characteristics:**

- Warm dark surface ladder with hairline rules instead of decorative depth.
- Terminal amber reserved for command, active, and focus; status colors reserved for state.
- True monospace stack with tabular figures for every numeric readout.
- Bounded charts, fixed row heights, and honest empty states (dashed baselines, `—` placeholders).
- zh-CN-first copy, fully globalized through the app i18n contract.

## Colors

The palette is a terminal phosphor system: a neutral warm-black ladder carries the surface, amber carries command, and a small status set carries operational truth.

### Primary

- **Terminal Amber** (#f0a32b dark / #8f650e light): the command and focus color — primary buttons, active navigation, focus outlines, the dock's active tick, route-hint hover. It is synchronized at four definition points (`:root`, `[data-theme='dark']`, `[data-accent='clay']`, `[data-theme='dark'][data-accent='clay']`); change all four or change none.

### Secondary

- **Warm Sand** (#c9a35f dark / #a0854f light): supporting emphasis for quiet charts and paired accents. Never a competing brand axis.

### Tertiary

- **Ready Green** (#5fa05a dark / #5b8a62 light): healthy, connected, done.
- **Attention Red** (#e06852 dark, hover #ec7a63 / #9f513f light, hover #a55644): errors, destructive actions, attention-required states. Holds ≥4.5:1 contrast on its status tints (4.65–4.79 measured).
- **Signal Warn** (#d9c05a dark / #a07c1e light): pending, scanning, degraded tracking.
- **Info Steel** (#7d94b0 dark / #7d97b6 light): neutral events, loading, web-preview notices.
- Status tints (`--color-*-tint`) carry status backgrounds; pair them with the matching contrast ink, never with raw white.

### Neutral

- **Terminal Ladder** (dark): content base (#100f0c), sidebar chrome (#171410), panel (#1f1b14), elevated overlay (#2a251b) — brightness rises with elevation.
- **Paper Ladder** (light): base (#e9e4d8), elevated (#f2eee3), surface (#faf7ec), overlay (#ddd5c2) — the desktop is darkened, cards are brightest.
- **Ink**: dark theme #e9e1d1 / #c9bda8 / #a1937c (primary / secondary / muted); light theme #211c12 / #4a4232 / #6b6150. All text tokens are solid and hold AA on their surfaces.
- **Hairlines**: border-subtle #332c21, default #453b2a, strong #5d5138 on dark; solid, not alpha.
- **Clay Flavor**: a warmer neutral variant (`data-flavor='clay'`) — dark family #17120f / #221b18 / #2a221e / #342b26, light family #ebe1d0 / #f5eee1 / #fefaf2 / #e2d6c3. Same geometry, warmer ink.

### Platform Identity

Claude clay (#d97757), Codex sage (#7cab82), Grok lavender (#a79bc4), Gemini steel (#7d97b6), OpenCode umber (#735f52), Antigravity sky (#98afc9). Each platform color has dot / surface / border / text roles per theme.

Four usage sources that appear only in Insights have the same five roles (`--color-platform-{key}`, `-rgb`, `-surface`, `-border`, `-text`): Kimi Code rose (#b65790, key `kimi-code`), Pi violet (#9773d0, key `pi`), ZCode teal (#1d83a4, key `zcode`), DeepSeek Harness azure (#1e8dfe, key `deepseek-harness`). The dot value is the same in both themes and both flavors. The dot holds ≥3:1 against the card face in all four theme × flavor combinations (#faf7ec, #fefaf2, #1f1b14, #2a221e). The surface and border roles mix the dot into the neutral card face (light #faf7ec at 12% / 32%, dark #1f1b14 at 22% / 40%). The text role holds ≥4.5:1 against its surface role and against the card face of both flavors. Source keys `kimi_code` and `deepseek_harness` become kebab case in token names. These four colors are flat identity swatches under the Platform Tick Rule; they never form gradients and never repaint panels.

The nine usage-source colors cannot all be told apart pair by pair (validator record: the `home-insights-design` task's `research/palette-validation.md`). Any chart that shows more than one source at a time must carry a secondary encoding: segment gaps, a legend, direct labels when the chart shows 4 or fewer series, and a per-segment tooltip.

### Named Rules

**The Status-Only Color Rule.** Green, red, warn, and info live only on status hairlines, dots, and state values. Panels, charts, and chrome stay neutral; a colored panel is a bug.

**The Amber Scarcity Rule.** Amber marks command, active, and focus — under 10% of any screen. If everything glows, nothing is next.

**The Platform Tick Rule.** Platform colors identify; they never repaint. Allowed surfaces: 2px ticks, legend swatches, chart segments, identity dots.

## Typography

**Sans Font:** MapleBright with SF Pro Text and system CJK fallbacks (PingFang SC, Microsoft YaHei UI).
**Brand/Display Font:** SF Pro Display with Segoe UI Variable Display and the same CJK fallbacks.
**Mono Font:** Cascadia Code, Cascadia Mono, SFMono-Regular, ui-monospace, Consolas — a true monospace stack, earned by data contexts (metrics, versions, timestamps, axes, the command status bar).

**Character:** A quiet sans for prose and labels, a proportional display face for page headers, and a real mono for the terminal's numeric voice. Hierarchy comes from weight, size, and spacing — not decorative faces.

### Hierarchy

- **Display** (600, clamp(1.625rem, 1.4rem + 0.7vw, 2rem), 1.12, tracking -0.022em): page-level headers on workbench surfaces.
- **Section** (600, 1.0625rem, 1.24): panel and card titles; one title per panel, no duplicate eyebrows.
- **Body** (400, 1rem, 1.56; 0.875rem on dense home surfaces): descriptions, hints, messages. Cap long prose at 65–75ch.
- **Label/Meta** (500–600, 0.8125rem / 0.6875rem): form labels, metadata, eyebrows. Wide tracking (0.14em) only for short Latin mono labels.
- **Data** (500–600 mono, 0.8125–1.125rem, tabular-nums): every numeric readout, version string, clock, and route index.

### Named Rules

**The Mono Is Earned Rule.** Mono appears where the terminal speaks: data, code, time, commands. Prose stays sans; never set paragraphs in mono for atmosphere.

**The Tabular Number Rule.** Any number a user compares or watches uses tabular figures (`font-variant-numeric: tabular-nums`). Columns of digits must not jitter.

**The Product Type Rule.** Fixed rem scales inside app surfaces; no viewport-fluid hero type beyond the one display clamp.

## Layout

Desktop-first workbench: a persistent nav rail, scrollable content column, and surfaces composed on a 4px spacing base (`--space-*`). Section gap 1.25rem, panel padding 1rem.

The overview first viewport is the world's signature composition: a full-width platform quote band on top (horizontal scroll under narrow widths — it never wraps), a main usage panel whose chart height is bounded, an event stream rail, and a bottom command status bar pinned to the scroll container. Data surfaces use fixed row heights and aligned character grids; resizing must not reflow a table into a card pile.

Reference breakpoints: 640 / 768 / 1024 / 1280 / 1536px. Below the `lg` breakpoint the sidebar becomes a modal with an explicit backdrop and close control.

## Elevation & Depth

Depth is tonal first: the surface ladder plus 1px hairlines separates shell, panel, and overlay at rest. Shadows are reserved for overlays that must detach from the workspace — sticky bars, dropdowns, modals — never for resting cards.

### Shadow Vocabulary

- **Subtle Rest** (`--shadow-sm`: 0 2px 6px rgb(25 27 32 / 9%); dark rgb(0 0 0 / 26%)): optional field and hover separation only.
- **Structural Overlay** (`--shadow-md` / `--elevation-2`): sticky bars and interactive overlays.
- **Modal Depth** (`--shadow-lg`+ / `--elevation-3`–`4`): dialogs and floating layers.
- **Focus Glow** (`--glow-*`: 2px ring at 40% of the accent or status color): focus rings only. Never decorative.

The scrim behind modals and drawers (32% black light / 56% dark) is the only permitted persistent translucency. The material contract keeps chrome and inline tiers fully opaque; only the floating tier (modals, command palette) may blur — 92% opaque, blur ≤ 12px, at most one on screen.

### Named Rules

**The Hairline-First Rule.** Separate with a hairline or a tonal step before reaching for shadow. A resting card casts none.

**The Glass Budget Rule.** At most one backdrop-filter surface per screen, floating tier only; no nested glass, no glass inside scrolling content.

## Shapes

Squared terminal geometry with a tight radius ladder: 0, 6px, 6px, 8px, 12px, and pill. Controls and inputs take 8px; panels and cards top out at 12px; pill is reserved for badges and status dots.

Signature geometry is the 2px tick: platform identity rides a 2px top tick on quote cells, and the active settings dock carries a 2px amber left tick. A dashed 1px hairline means "no data" — an honest zero baseline, not a missing chart.

### Named Rules

**The 12px Ceiling Rule.** No product surface exceeds a 12px corner. Large radii read as consumer SaaS, not terminal.

**The Two-Pixel Tick Rule.** Identity and active state arrive as 2px ticks, never as stripes, glows, or filled rails.

## Components

### Buttons

- **Shape:** 8px corners, padding-defined height (0.625rem 1rem), 0.8125rem medium-weight label.
- **Primary:** solid Terminal Amber fill with deep contrast ink (#1d1204); hover brightens (#f5b14a). No gradient, no shadow stack.
- **Secondary / Ghost / Danger:** secondary is panel fill with a default hairline border; ghost is quiet text that gains an overlay wash on hover; danger is solid Attention Red with its contrast ink.
- **State:** active presses with scale(0.98); disabled drops to 50% opacity and must still be legible at a glance. Focus uses the amber focus ring.

### Inputs / Fields

- **Style:** 8px corners, panel background, 1px default hairline, 0.625rem 1rem padding.
- **Focus:** amber border shift plus the focus ring; never decorative glow.
- **Error / Disabled:** error uses Attention Red plus `aria-invalid`; disabled reduces opacity while labels stay readable.

### Cards / Panels

- **Corner Style:** 12px on workbench panels.
- **Background:** opaque panel token with a 1px subtle hairline border; internal padding 1rem on dense dashboards, up to 2rem on large configuration panels.
- **Shadow Strategy:** none at rest; hover may shift background one ladder step.

### Navigation

Persistent sidebar items use 8px corners and quiet secondary text. The active route reads as a 10% amber tint with a 24% amber border and primary text — state conveyed by tint, not by a filled bar. Mobile collapses to a modal sidebar with backdrop.

### Platform Quote Band (signature)

A single hairline-ruled strip holding every platform cell: 1px separators between cells, a 2px platform-color top tick per cell, mono tabular metrics with 0.14em-tracked micro labels. Under narrow widths it scrolls horizontally rather than wrapping. Sparkline bars render neutral with the peak bar in the platform color; an empty series shows a dashed hairline baseline; unindexed sessions render a muted `—`, never a misleading 0.

### Bounded Usage Chart (signature)

Stacked platform segments on a hairline baseline with gridlines at 25/50/75%. Height is hard-bounded (`clamp(10rem, 26vh, 16rem)`) — the chart informs the band, it never becomes a hero. Length is quantity; skeletons pulse in place; the hero metric reads in mono.

### Event Stream (signature)

One section title, no duplicate eyebrow. Rows align a mono tabular timestamp, a severity dot, a level tag, a channel, and a single-ellipsis message. Severity color lives on the dot and level only (errors additionally lift the message to primary ink). The list caps at 16rem and scrolls; the empty state is a dashed hairline panel with a concrete next action.

### Bottom Command Status Bar (signature)

Sticky to the scroll container's bottom edge: a 1px hairline top rule over the base background, mono meta-size text, items separated by hairlines. It carries backend status (ok in amber, pending in warn, error in attention red), the last event's severity dot, 01–04 numbered route hints whose indices brighten amber on hover, and a tabular clock.

### Settings Dock (signature)

The sidebar's settings anchor: a card with two rows — a title row and a mono meta row ending in the app version — both single-line ellipsis with a native tooltip for the full string. Active state is the overlay background plus a 2px amber left tick.

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
- Colors: `--color-chart-heat-0` … `--color-chart-heat-4`, a warm neutral single-hue ramp. Heat-0 is the no-activity cell fill only; it stays visibly apart from the card face in all four theme × flavor combinations (≥1.15:1). One value per theme serves both flavors. Heat-1 is below 3:1 against the surface. The table view is the relief channel for colors below 3:1 (dataviz rule: a tooltip alone is not enough). Each cell's `title` / tooltip also gives the date and the value.
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

- Empty store (no requests and no sessions): the header row, the totals readout with every value `—`, and one dashed panel (`1px dashed --color-border-default`) with "还没有可分析的记录", a short description, and the primary "导入用量" button. The button is amber because it is the next command.
- Sessions only (no requests, `sessions_indexed = true`, sessions present): request values show `—`. The heatmap follows its empty rule. The 7-day comparison has values in the sessions row only. The leaderboard defaults to the session metric. The weekly chart and the distribution are not rendered.
- Usage only (`sessions_indexed = false`): every session count shows `—` with `title="会话尚未索引"`. One info dot plus an explanation line sits below the totals readout. The session metric in the leaderboard is disabled. All other blocks render as usual.
- Web preview (not the Tauri runtime): the header row plus one solid panel with an info dot, "Web 预览不读取本机数据", and a short description. The block makes no IPC call.

### Boot Loader

The index.html boot screen is synced to the world: light background #e9e4d8, dark background #100f0c, dark spinner arc in amber #f0a32b. Known open item: the light spinner arc still uses #0071e3 pending an accent review — flag it before shipping a light-first surface.

### Copy & Localization

zh-CN is fully globalized and is the primary copy voice. Components subscribe to translation via `useAppT()` — never a bare captured `t`. The locale leaf count is governed at 4501 (`scripts/check-i18n.mjs`, enforced by `tests/i18n.test.cjs`); add keys through the contract, not ad hoc. Single-line strings truncate with ellipsis and carry a tooltip.

## Do's and Don'ts

### Do:

- **Do** reserve amber for command, active route, and focus — and keep it under 10% of the screen.
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
- **Don't** add purple or purple-blue gradients, guofeng, neko, anime, catgirl, or mascot variants — legacy debt, not parallel themes.
- **Don't** widen 2px ticks into side stripes, or use gradient text and 12px-plus radii on product surfaces.
- **Don't** use fluid hero typography or decorative fonts inside app surfaces.
- **Don't** capture a bare `t` outside `useAppT()`, and don't leave zh-CN copy unglobalized.
- **Don't** hide disabled, blocked, or destructive states — they must read at a glance.
- **Don't** color a 7-day change green or red; the change is not a status.
- **Don't** use amber or status colors in the heatmap ramp or the distribution bars; they use `--color-chart-heat-*` only.
- **Don't** add a chart library for Insights; charts are plain DOM and CSS.
- **Don't** build the Insights totals as metric cards or as a copy of the platform quote band; it is one hairline-ruled reading line.
- **Don't** give each of the nine sources its own weekly-chart color; merge everything past the top 4 into "其他".
- **Don't** use platform colors on project or model rows in the leaderboard; only Agent rows carry the platform tick and bar.
