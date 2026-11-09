# 技术设计：GPUI Overview 填充与会话/MCP 配置页重设计

关联：`prd.md`；调研 `research/overview.md`、`research/agent-sessions.md`、`research/mcp-manager.md`；设计合同 `ccr-desktop/DESIGN.md`（design-atlas：项目已有 DESIGN.md，直接按它实现、在其上增量修改，不重新 apply 目录条目）。

## 0. 视觉方向（/ui 方向锁）

- **视觉主张**：沿用 airy-glass 工作台：薄荷到淡紫的静态光场上漂着半透明纸面，纸面里放实色阅读面板。墨色保持中性，读数用 tnum。颜色只用于两件事：真实状态（语调 chip、圆点）和平台身份（8px 圆点、2px 细线）。
- **内容顺序**（工具页，不做 hero）：先定位（页头、状态与新鲜度）→ 看状态（KPI、平台、来源健康、提供方 chip）→ 执行操作（主按钮，列表到详情）。
- **交互主张**：不新增动效。依据是 DESIGN.md 的高频规则：行和筛选都是高频操作，不加动画；按钮继续沿用现有的指针按压反馈。筛选 chip 和分段胶囊点击后立即切换状态。
- **设计签名**：用两种形状区分可点与不可点。非交互的状态标签是 4px 直角矩形，交互的筛选 chip 和按钮是 pill。数字统一 tnum 右对齐。
- **反 AI 味底线**：不做同形卡片墙（KPI 仍是一张面板分 4 栏）；不出现全宽居中的孤立按钮；不给每个列表项都套 pill，状态标签只在异常时出现；不用渐变文字；玻璃不盖在文字上；变化量不用红绿。`slop_check.mjs` 只扫 HTML/CSS/JSX，GPUI 是 Rust，所以不适用，这里改为逐条走查 `anti-slop.md` §6 的人工判定清单。

## 1. 共享层

### 1.1 DESIGN.md 增量（先于代码）

- Components 新增以下条目：
  - **状态标签**：Tone 五种，22px 高，4px 角，12/500。
  - **筛选 chip**：28px 高，pill，带 toggled 语义；未选时透明底加控件边界 border，选中时是 tab-selected 浮起面加主色文字。
  - **身份圆点**：8px。
  - **搜索框**：前缀 search 图标、placeholder、固定宽度 280–320px。
  - **工具栏**：一行，左侧筛选，右侧操作，可换行，但不出现全宽按钮。
  - **两行列表行**：56–60px，实色面板内用 hairline 分隔，选中用中性色阶加 4px 角。
  - **来源健康条**。
  - **虚线空态面板**：1px dashed，8px 角，图标 + 原因 + 下一步。
- Layout 中的“列表行 36px”改为“单行 36px / 两行 56–60px”。
- 平台身份色范围写明：claude / codex / grok / antigravity(gemini) / opencode / kimi / pi；omp 用 muted。
- 运行概览章节按 §2.2 重写；MCP 与 Agent sessions 各新增一段页面规则（§3、§4）。

### 1.2 `theme.rs`（唯一 token 层）

```rust
pub const TAG_HEIGHT: f32 = 22.;
pub const FILTER_CHIP_HEIGHT: f32 = 28.;
pub const DOT_SIZE: f32 = 8.;
pub const SEARCH_WIDTH: f32 = 300.;
pub const TWO_LINE_ROW_HEIGHT: f32 = 60.;   // Sessions；MCP 用 56
pub const MCP_ROW_HEIGHT: f32 = 56.;
pub const OVERVIEW_RAIL_MIN: f32 = 280.;     // 取代 OVERVIEW_RAIL_WIDTH
pub const OVERVIEW_RAIL_MAX: f32 = 360.;
pub const HEATMAP_CELL_MIN: f32 = 8.;
pub const HEATMAP_CELL_MAX: f32 = 14.;
pub const HEATMAP_GAP: f32 = 2.;
pub const DISTRIBUTION_HEIGHT: f32 = 160.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone { Neutral, Success, Warning, Danger, Info }
pub struct ToneColors { pub ink: Hsla, pub fill: Hsla, pub edge: Hsla }
/// fill = ink 在 reading 面上的 light 10% / dark 18% 叠加；edge = light 24% / dark 32%。
pub fn tone_colors(tone: Tone, mode: ThemeMode) -> ToneColors;
/// 热力图 5 档：0 档为中性 hairline 格，1–4 档为 primary 的 25/45/70/100% 叠加于 reading。
pub fn heat_level(level: u8, mode: ThemeMode) -> Hsla;
```

- `platform_color` 增加 `"kimi" => (0xa3467f, 0xd775ae)`、`"pi" => (0x7b56b0, 0xac88e7)`。
- 单测（与已有的合成对比度测试同一组）：
  - 5 种 Tone 在 light/dark 下 ink 对 (fill ∘ reading) ≥4.5:1；edge 是装饰描边，对 reading 的对比只需高于 fill（实施中修订：3:1 的描边会让直角标签读作描边按钮）。
  - kimi/pi 身份色对 reading 面 ≥3:1（图形对比）。
  - heat 1–4 档两两可区分，第 4 档对 reading ≥3:1。

### 1.3 `shell/badge.rs`（新增，所有页面共用）

```rust
pub fn tag(id: impl Into<ElementId>, label: impl Into<SharedString>, tone: Tone, cx: &App) -> Stateful<Div>;
pub fn dot(color: Hsla) -> Div;                                   // 并入 overview.rs 原私有 status_dot
pub fn hollow_dot(color: Hsla) -> Div;                            // Missing / NotRead
pub fn identity(platform_id: &str, label: impl Into<SharedString>, cx: &App) -> Div; // 圆点 + 名称
pub fn filter_chip(button: Button, pressed: bool, disabled: bool, cx: &App) -> Button; // 复用 semantics 的 toggled/disabled 语义
pub fn search_field(input: Input, cx: &App) -> Input;             // prefix Icon(Search) + 宽度 + 8px 角；id/aria 由调用方给
pub fn empty_panel(id: impl Into<ElementId>, icon: IconName, title: SharedString,
                   body: SharedString, action: Option<AnyElement>, cx: &App) -> Stateful<Div>;
pub fn section_heading(title: impl Into<SharedString>, scope: Option<SharedString>, cx: &App) -> Div; // 16/600 + 12 muted 口径
```

- 不使用 Kit 的 `Tag`/`Badge`。原因是它们读 Kit 主题色，圆角和字号也不在本项目的 token 体系里，而本合同要求颜色统一出自 `theme::tone_colors`。
- 图标只用默认图标集：search、plus、refresh-cw、globe、square-terminal、book-open、circle-alert、triangle-alert、circle-check、ban、eye、user、bot、arrow-up、arrow-down、arrow-left、star、inbox。trash、lock、clock、messages-square 不在默认集里：对应位置改用文字，或换成上面的替代图标。
- 遵守 1 MiB 栈约束：可选子树（如 `action`）由调用方先构造好再传入，builder 拆成独立函数。

## 2. Overview

### 2.1 数据契约（`runtime/insights.rs`）

```rust
pub struct WorkbenchReport {
    // ……现有字段不变
    pub last_sync_at: Option<String>,                 // OverviewPayload.last_sync_at，RFC3339 UTC
    pub activity: Option<HomeInsightsSnapshot>,       // 复用 ccr-app 类型，不建镜像类型
}
```

- `session_window` 改名为 `activity_window`，返回 `(Option<HomeInsightsSnapshot>, bool, Option<i64>, SessionScope)`：
  - 用量投影 `dashboard.insights` 失败：返回 `(None, false, None, Unavailable)`。
  - 归档 pool 打不开，或 `assemble_home_insights` 失败：调用 `merge_home_insights(projected, InsightsSessionCounts::default(), clock.as_of)`，得到纯用量快照，`SessionScope::Unavailable`。
  - 其余情况与现在一致（Unindexed / Window / AllHistory）。
- 会话诚实规则：渲染层同时看 `session_scope` 和 `snapshot.sessions_indexed`。Unavailable 时显示“不可用”，Unindexed 时显示“未索引”，两种情况都不能显示 0。
- `last_sync_at` 按报告时区换算成本地 `MM-DD HH:mm`。`stale = last_sync_local_date < as_of`。值为 None 时写“未记录成功同步 / No successful sync recorded”，并按 stale 处理，但不附“较新用量可能缺失”说明。已知时间过期时才显示该说明及 warning tooltip。失败同步仍可能导入数据，不能由最后成功时间推断记录边界。
- 不新增 SQL；`crates/ccr-usage` 不改。

### 2.2 版式与几何

先只做一次 move-only 拆分（单独提交，不改行为）：`features/overview.rs` 拆为 `features/overview/{mod.rs, header.rs, kpis.rs, platforms.rs, chart.rs, activity.rs, insights.rs, rail.rs, tests.rs}`。

```
overview-root (flex_col, overflow_y_scroll, p 24/16 窄, gap 16)
├ header（不变；状态行追加 “· Last sync MM-DD HH:mm”，stale 时加带 tooltip 的 warning 圆点）
├ overview-kpis（不变）
├ overview-platform-band（不变）
├ overview-lower（宽：flex_row items_stretch gap 16；窄：flex_col）
│  ├ overview-main（flex_1 min_w_0, flex_col gap 16）
│  │  ├ overview-usage（逐日图 216px + 精确值；不伸展）
│  │  └ overview-activity（flex_1：内容顶对齐，图例/摘要贴底）
│  └ overview-rail（w = rail_w, flex_col gap 16）
│     ├ overview-actions（4 个 OverviewLink，原 id 不变）
│     ├ overview-week（近 7 日 vs 前 7 日）
│     └ overview-data-source（flex_1；原 id overview-insights 更名）
└ overview-insights-row（宽：1:1 items_stretch；窄：flex_col）
   ├ overview-distribution（Hourly | Weekday 胶囊）
   └ overview-leaders（Models | Projects | Agents 胶囊，前 8）
```

- `content_w = viewport_w − sidebar_w − 2·scene_inset − 2·padding`。三个量都复用 shell 和 theme 里现有的函数，不在 overview 里另写一份常量。
- `rail_w = clamp(round_to_8(content_w × 0.24), 280, 360)`；`main_w = content_w − rail_w − 16`。
- 热力图：`cell = clamp(floor((main_w − 32 − 24 − 52·2) / 53), 8, 14)`，其中 24 是星期标签列。`cell < 8` 时取 8，并让 `overview-heatmap` 设 id，在面板内 `overflow_x_scroll`。热力图宽度 + 200 ≤ 面板内宽时，摘要列放在热力图右侧，否则放到下方。
- 首屏估算：
  - 2000×1080：页头 56 + KPI 110 + 平台带 140 + 下排约 552 + 3 个间距 ≈ 906，可用高度约 964，洞察行的上沿会露出在首屏。
  - 1280×800：下排已经超出一屏，页面自然滚动。
  - 两种尺寸下，图表面板内都不再被拉伸。
- 等高规则：两列都是 stretch，多余高度只交给每列最后一个面板（activity / data-source）。这两个面板内容顶对齐、脚注贴底，所以吸收的高度只表现为面板内部的正常间距，不会形成死区。

### 2.3 模块规格

| 模块 | 内容 | 数据 | 空 / 降级 |
| --- | --- | --- | --- |
| 状态行新鲜度 | `Last sync 11-05 14:34`；stale 时加带 tooltip 的 warning 圆点 | `last_sync_at` | None：只写“No successful sync recorded”并保留圆点 |
| 逐日图标题 | 现有 `since MM-DD`；stale 且窗口覆盖 last_sync 之后的日期时，不推断导入缺口；仅在尾部为空时追加 `· latest record MM-DD` | 同上 | 无 |
| activity | `section_heading("Activity", "Last 53 weeks · since MM-DD")`；53×7 格（周一在上，标签只标 Mon/Wed/Fri）；摘要：活跃天、当前 streak、最长 streak、最忙日（日期 · 请求数）；图例 Less ▢▢▢▢▢ More；有记录的格子带 tooltip `YYYY-MM-DD · N requests` | `daily` / `current_streak` / `longest_streak` / `busiest_day` / `totals.active_days` / `trend_start` | `activity == None`：用虚线空态面板说明原因；`daily` 为空时写“No requests in the last 53 weeks”，网格不铺 0 |
| week | 3 行：Requests / Sessions / Active days；列为 This 7 days、Previous 7、变化（↑ / ↓ / = 加百分比，中性墨色；前期为 0 时写 “new”，不出现除零） | `last_7_days` / `previous_7_days` | 会话按 §2.1 诚实规则显示 |
| data-source | 配置状态、Report as of、Last sync、stale 说明、图表来源活跃日覆盖（现有句子缩短） | 现有 + `last_sync_at` | 无 |
| distribution | 胶囊 Hourly(24) / Weekday(7)；中性柱 + 峰值柱用 primary；caption “Peak 14:00 · N requests / All history” | `hourly` / `weekday` | 全为 0：虚线基线 + 文字 |
| leaders | 胶囊 Models / Projects / Agents；每行：名称（单行省略 + tooltip；Projects 显示末段目录名）+ 比例条（中性底、primary 填充）+ tnum 请求数；口径 “All history” | `models` / `projects` / `agents` 取前 8 | 无行：写“No records” |

- 可访问性：heatmap 容器的 aria label 就是摘要句；分布和排行用胶囊 `semantics::tab` 切换，每个柱和每行都有 aria label（值和标签）。
- 颜色：热力图用 `theme::heat_level`；柱色用中性加 primary 峰值，不用平台身份色，因为这些是全平台聚合数据。

### 2.4 测试 id 与断言

- 新增 id：`overview-main`、`overview-activity`、`overview-heatmap`、`overview-week`、`overview-data-source`、`overview-insights-row`、`overview-distribution`、`overview-dist-hourly/weekday`、`overview-leaders`、`overview-leaders-models/projects/agents`、`overview-last-sync`。
- 改动的断言：
  - 等高断言改为 `overview-main.bottom == overview-rail.bottom`。
  - 新增死区断言：`overview-usage.bottom − overview-chart-axis.bottom ≤ 24`，需要新增轴标签行 id `overview-chart-axis`。
  - `overview-insights` 断言改为 `overview-data-source`。
- 新增单测：
  - 归档不可用时 activity 仍显示，会话写“Unavailable”。
  - stale 新鲜度提示。
  - `daily` 为空时不出现 0 格。
  - 热力图格子尺寸在 1280 与 2000 宽下分别为 9 和 14。
  - 窄宽（<1040）时单列，且没有整页横滚。
  - `WorkbenchReport` 的所有字面量补上 `last_sync_at` 与 `activity`。

## 3. MCP Manager

### 3.1 外壳

- `Shell::embeds_lens()` 改为 `matches!(self.destination, Destination::Dashboard | Destination::McpManager)`。
- `McpView::embed_lens(Option<AnyView>)` 与 `Converter::embed_lens(...)` 在各自标题行渲染 LensBar；Converter 标题补 `Role::Heading` 和 aria level 1。
- Claude 工作台里复用的 `McpView`（`tool-platform-mcp`）不拿到嵌入槽，标题行不带胶囊，现有平台工具栏不变。

### 3.2 版式

```
page-mcp-manager (flex_col size_full p 24 gap 16)
├ mcp-header: [h1 MCP Manager][LensBar] ……………… [Reload ↻ ghost][+ Add server primary]
│  └ 一行 muted 描述（不写长句；“不启动服务器 / 密钥不显示”放进 tooltip 或详情空态）
├ mcp-toolbar: [🔍 mcp-filter 300px] [All n | Claude Code n | Codex n | Antigravity n] …… [★ Presets][Batch & sync]
├ mcp-sources: 来源健康条（id + overflow_x_scroll）
└ mcp-body（flex_1 min_h_0）
   宽（≥1200）：flex_row gap 16 → mcp-list（3, min 360, 实色面板, overflow_y_scroll）| mcp-pane（2, min 320, 实色面板, overflow_y_scroll）
   窄：pane 有内容时只渲染 pane（顶部 mcp-pane-back），否则只渲染 list
```

- 详情区的内容由一个枚举决定，取代 render 里的多处 early return：

```rust
enum Pane { Empty, Detail(usize), Editor, Presets, SourcePicker, Batch }
// 优先级：batch > editor > source picker > presets > selected detail > Empty
```

- `Empty`（宽窗、未选中）：虚线空态面板，提示“Select a server to see its source, scope and actions”，并汇总当前读到的条目总数，数字来自 catalog。不自动选中第一项，这样现有测试里“筛选后 detail 消失”的语义不变。

### 3.3 来源健康条

| McpSourceState | 呈现 | 语调 |
| --- | --- | --- |
| Entries(n>0) | 实心圆点 + `Claude state · 5` | Success 圆点，文字 secondary |
| Empty / Entries(0) | 实心圆点 + `Claude settings · none` | Neutral |
| Missing | 空心圆点 + `Antigravity · not found` | Neutral |
| Invalid | triangle-alert + `… · invalid` | Warning |
| Unreadable | circle-alert + `… · unreadable` | Danger |
| NotRead | 虚线边 + `Project · not read` / `OpenCode · not read` / `Grok · not listed` | Neutral |

- 每个 chip 保持 `id = mcp-source-{index}`，`aria_label = note_line()` 的原句（测试断言依赖它），tooltip 显示同一句。
- 短名表 `source_short(source)`，中英两套。

### 3.4 列表行（56px，两行）

```
[▣ stdio/globe]  name (14/500, ellipsis)                         ● Claude Code
                 npx -y @x/server (mono 12, ellipsis+tooltip) · User      [Disabled][Read-only]
```

- 作用域文字：`User`；`Project · <末段目录名>`，完整项目键放 tooltip（同名不同作用域的条目由此区分）。
- 只在异常时显示标签：
  - `Disabled`：`disabled == Some(true)` 或 `enabled == Some(false)`，Warning，前置 ban 图标。
  - `Read-only`：`McpTarget::for_entry` 返回 Err，Neutral，前置 eye 图标。
- 只有 URL 时第二行写 `URL set (hidden)`，永不显示 URL 值。
- 行的 aria label 保持现有 `平台 · 名称 [· disabled in the file]` 规则，再追加作用域，测试按包含关系断言。
- 筛选：`visible()` 改为按名称或命令匹配（子串，不区分大小写），平台胶囊与搜索取交集。平台计数在搜索之前计算，只出现有条目的平台。

### 3.5 详情与编辑区

- 头部：40px 方块（primary 10% 底）放传输图标；眉题 `平台 · 来源文件 · 作用域`（12 muted）；名称 20/600；右侧 `mcp-copy` / `mcp-edit` / `mcp-delete`（Delete 用 danger 文字）。只读来源显示 eye 图标和一句说明。
- 键值网格（标签 12 muted 在上，值 14 在下，两列，窄时单列）：Transport（stdio / URL）、Command（mono）、URL（Set (hidden) / Not set）、Arguments（Set / Not set）、Scope（mono，长路径换行）、Source（source_title）、State（enabled/disabled 原文语义）。
- `mcp-detail` 的 aria label 保持 7 句拼接不变。
- 编辑器、预设（行：名称 + 描述 + 标签 + requires-key 时显示 `KEY_ENV`）、来源选择（现有 5 个 tab 加项目根输入框，补 placeholder 与可见标签）、批量（模式切换改为 `semantics::tab` 胶囊）全部放进 pane。
- 批量的焦点契约不变：首焦点 `mcp-batch-back`，Tab 循环，Esc 回到 `mcp-batch-review`。`suspend` 后 batch 为 None。

## 4. Agent sessions

### 4.1 状态机（`features/insights/mod.rs`）

```rust
// 仅 Surface::Sessions 使用；Workbench/Usage 维持单 snapshot 路径
struct SessionDetailSlot {
    archive_id: Option<String>, before: Option<u32>,
    detail: Option<Arc<SessionDetail>>, loading: bool, error: Option<&'static str>,
    gate: ResponseGate, generation: u64, task: Option<Task<()>>,
}
agents: BTreeSet<AgentSessionAgentId>,   // 提供方多选；平台作用域时锁定为该 agent
fidelity: Option<&'static str>,          // "full" | "partial" | "locked"；None = 全部
source_state: Option<&'static str>,      // "live" | "missing" | "deleted_by_user"；None = 全部
// 已核对（2026-11-08）：取值校验在 crates/ccr-app/src/sessions/mod.rs 的 list_sessions → validate_choice，
// 不在 usage_repo。允许值即上面三项再加 "all"（trim + 小写）；越界返回 SessionError::InvalidFilter
// （code agent_session_invalid_filter）。UnknownFidelity 只用于库里存了未知 fidelity 的行。
// agents 为空 Vec 时视为全部提供方。
```

- `select(index)`（Sessions surface）：设置 `slot.archive_id` 并让 `before = None`，再调用 `submit_detail`。列表快照保留。
- `submit()`（列表）：刷新索引或筛选条件变化时清空 slot；列表翻页（`next` 作用于 Sessions）保留 slot。
- 详情翻页：“Earlier messages” 设置 `slot.before` 后调用 `submit_detail`。
- 两条请求用不同的 `RequestKey.environment`（`insights-explicit` / `insights-detail`），generation 各自独立；gate 分开，晚到的旧响应会被丢弃。
- 搜索：Enter 立即 submit。`InputEvent::Change` 时启动 300ms 定时任务，带一个 generation token，旧 token 到期后不执行。
- 平台作用域（`open_platform`）：`agents = {platform}` 并锁定，这时不渲染筛选 chip，改为一行“Showing Claude sessions”。原来的 `fields[7]` 只给 Workbench 渲染器用。

### 4.2 版式

```
session-page (flex_col size_full p 24 gap 16)
├ session-header: [h1 Agent sessions / 一行描述] …… [session-status tag][CCR trash][Refresh index primary]
├ session-providers: 8 个 filter_chip（● name · state · n），可换行
├ session-toolbar: [🔍 insights-filter-6] [All|Full|Partial|Locked] [All|Live|Missing|Deleted] …… “100 on this page · older available”
└ session-body (flex_1 min_h_0)
   宽（≥1200）：session-list（3, min 360）| session-pane（2, min 320）
   窄：pane 有内容（详情或废纸篓）时只显示 pane（insights-back / 关闭），否则只显示 list
```

- 提供方 chip 的状态映射：

| availability | chip | 可选 |
| --- | --- | --- |
| Available | 身份圆点 + 名称 + `n sources`（tnum） | 是 |
| NoData | muted 圆点 + 名称 + `no files` | 是 |
| NotInstalled | 虚线边 + muted 名称 + `not installed` | 否（disabled） |
| Error | circle-alert（danger）+ 名称 + `unreadable`；tooltip 显示 `error_category` | 否 |

- `session-status` 标签：Running 用 Info，文字 “Indexing…”；Finished 用 Success，“Indexed · parsed X / upserted Y”；Failed 和 Interrupted 用 Danger 并给出原因；Idle 不显示标签。`role=Status` 保留。
- 列表（`uniform_list("session-cards")`，行高 60，放在 12px 实色面板内，行底 hairline）：

```
● 标题单行省略（14/500, tooltip 全文）                           11-05 03:09 (tnum)
  claude · claude-code · 272 messages · 18 tools · D:\Documents\…\thesis (mono, ellipsis)   [Partial]
```

- 行的语义：`role=ListBoxOption`、`aria_selected`；列表获得焦点后上下键移动，Enter 打开详情，点击直接打开。实现参照 Workbench 渲染器（`mod.rs:936-960,995-1023`）。行 id 仍为 `insights-row:{archive_id}`。
- 底部翻页条：`insights-next`（Older sessions，没有 `next_cursor` 时不渲染）和 `session-first-page`（不在第一页时显示）。
- 详情 pane 头部：标题（16/600，省略）、mono session_id、`agent · variant` 标签（Neutral）。fidelity 为 partial/locked 时显示 Warning 提示条，图标为 triangle-alert。Codex 操作放在头部右侧。消息卡头部依次是角色图标（user/bot，tool 角色用 square-terminal）、角色名、mono tool_name、右侧时间（没有时间时显示 `#ordinal`）；clipped 时显示 Warning 文本 “Clipped in archive”。
- 废纸篓：由页头的 `session-open-trash` 打开，宽窗下占据 pane，窄窗下显示在列表上方；面板标题为 “CCR trash (Codex)”，带 `session-trash-close`。分页、恢复和确认的 id 都不变。焦点契约要求废纸篓打开时 `insights-filter-6` 仍在同一视图内可点击，宽窗与窄窗的布局都满足这一点。
- 文案：fidelity 显示为 full → “Full record / 完整记录”，partial → “Partial / 部分”，locked → “Locked / 锁定”。

## 5. 响应式矩阵

| 窗口 | Overview | MCP | Sessions |
| --- | --- | --- | --- |
| 2000×1080（用户） | rail 360；热力图 14px；洞察行在首屏底部露出 | 3:2 分栏 | 3:2 分栏 |
| 1280×800（默认） | rail 280；热力图 9px；下排超出一屏、整页滚动 | 3:2 分栏（内容宽约 976 ≥ 360+320+16） | 同左 |
| 1040–1199 | rail 280；热力图按公式计算，可能面板内横滚 | drill-in | drill-in |
| 960×640（最小） | 单列；KPI 2×2；平台带横滚 | drill-in；健康条横滚 | drill-in；chip 单行横滚，计数与筛选共行 |

注意：分栏断点用的是 `SPLIT_WIDTH`（1200）对窗口宽度的判定，沿用现有约定。

## 6. 兼容、回滚与风险

- 每个阶段单独提交，可分别回滚（见 `implement.md`）。共享层只新增 API，不改现有调用方的语义；overview 的 move-only 拆分单独成一个提交。
- 风险与应对：
  1. 热力图 371 个元素外加 tooltip 的渲染开销 → 只给有记录的格子加 tooltip 和 id；实施后用现有 benchmark 测试（`runtime/benchmark`）对比渲染时间。
  2. Sessions 双快照的竞态 → gate 和 generation 分开，加单测“快速连续选择两行只显示后者”“列表刷新会丢弃在途的详情”。
  3. MCP 的 pane 枚举改写了 early return 的顺序 → 先用测试锁住现有优先级（batch > editor > …），再重构。
  4. fidelity 和 source_state 的取值字符串 → 已核对，取值见 §4.1 注释。取值无效会返回 `InvalidFilter`（不是 `UnknownFidelity`）。常量只从这一组值里取，并加测试覆盖每个筛选项发出的请求值。
  5. release 构建差异（上一轮踩过的坑）→ 每个阶段结束都跑 `just desktop-ci`。

## 7. 视觉复核后的局部修订（2026-11-08）

- MCP/会话外层分栏只负责 3:2 尺寸，无 padding/border。列表阅读面板不再无条件 flex_1：少量内容收紧，长列表 min_h_0 并在内部滚动。未选中详情和空废纸篓同样收紧；真实详情/编辑器/转录/非空废纸篓继续填满与滚动。
- ghost 次操作统一使用 `semantics::ghost_button`，禁用时仍透明且文字 muted，不改变全局 `button`。
- Overview Refresh 为 RefreshCw ghost；安装入口写安装或更新；平台 sparkline 中性柱由 theme token 提供并满足双主题 ≥3:1。
- 会话 header 去掉图标块；两个筛选胶囊分别有组名及 All records/All sources 文案。低于 1200 时 provider 单行横滚；计数与筛选共行，960×640 第一行顶部 < 窗高 45%。
- 会话消息为 hairline 分隔的内容块；角色 muted 且本地化；显示/总消息及工具次数合一行。列表身份点对齐标题；Older sessions 为右箭头 ghost。
- Sessions 列表读失败保留上次成功 provider 可用性，同时显示错误。Codex 为 NotInstalled 时不显示废纸篓。搜索 placeholder 与 aria 明确匹配标题、会话 ID、路径。
- MCP 批量增加 Preset/Options 组名；详情操作图标保持一致。

以上修订覆盖前文早期版式描述；实际颜色、几何与文案以同轮更新的 `ccr-desktop/DESIGN.md` 为准。
