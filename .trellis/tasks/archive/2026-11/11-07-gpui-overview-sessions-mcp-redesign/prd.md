# GPUI Overview 填充与会话/MCP 配置页重设计

## 背景与诊断

用户 2026-11-07 提供 3 张 GPUI 截图（浅色、英文、约 2000×1080 窗口），要求参考 Tauri 版：

1. Overview 有大片空白，设计需要优化。
2. Agent sessions 与 MCP Manager 两个配置页的徽章、样式和排版要重新设计，符合现代审美。

上一轮任务（`archive/2026-11/11-07-gpui-airy-glass-material`）已把底部空白记作“内容高度自然结束”。这次用户明确拒绝这一结论：大窗口的首屏不能留空。

调研结论（详见 `research/overview.md`、`research/agent-sessions.md`、`research/mcp-manager.md`）：

### Overview

- **图表面板内约 130px 死区**：柱区固定 216px（`theme.rs:45`），面板内没有可伸展的子项；下排 `items_stretch`（`overview.rs:266`）让 240px 右栏决定行高。右栏是 4 个动作行加一段折成约 4 行的长句，约 400px，于是图表面板外框被拉高，图表本身没变。等高单测（`overview.rs:1862`）通过，但视觉上就是死区。
- **页面底部约 200px 空白**：`overview-root` 的子项全是 `flex_shrink_0`，没有一项会吸收剩余高度（`overview.rs:245-269`）。Tauri 并不靠拉伸，而是在主行之后放了一整段 Insights 内容（`DashboardInsights.tsx:24-149`）。
- **11-06/11-07 无柱**：真实数据缺口。本机 llmusage 最后一次同步（11-05）失败，之后没有新数据。GPUI 丢弃了已查询到的 `last_sync_at`（`runtime/insights.rs:577-584`），用户看不出数据有多旧。
- **数据现成**：`session_window`（`runtime/insights.rs:606-645`）每次刷新都会算出完整的 `HomeInsightsSnapshot`，包括 53 周逐日、streak、近 7 日与前 7 日、小时/星期分布、模型/项目/Agent 排行，但只取了会话数就丢弃了。

### Agent sessions（实际渲染器是 `features/insights/product.rs`）

- 提供方状态是 `secondary` 底的纯文字，只有两种颜色（readable 用 primary，其余 muted）；没有图标和计数，也不能点击（`product.rs:178-232`）。
- 搜索框标签写 “Title”，但查询实际匹配 session_id/title/cwd（`usage_repo.rs:991-1000`）；旁边是 ghost 的 “Search” 文字按钮。
- 行高固定 84px，标题没有截断，长中文会顶满整行（`product.rs:340-348`）；meta 是纯文本，其中 “complete” 实际是 fidelity，不代表会话已结束；路径不是 mono；行没有 list 语义，也不支持键盘选择。
- 列表没有翻页入口，状态行却写着 “Older ones follow”。
- 只对 Codex 有效的废纸篓入口被 stretch 成全宽的居中文字。
- 打开会话会整页替换为转录，因为 `InsightsView` 只有一个 `snapshot` 槽（`mod.rs:87,383`）。

### MCP Manager（`features/mcp/mod.rs`）

- lens 胶囊位于标题上方：只有 Dashboard 在 `embeds_lens()` 里（`shell/mod.rs:788-790`）。
- 筛选框没有 placeholder、标签和图标，且撑满整行（`mod.rs:53,544`）；三个操作都是无图标的 ghost 按钮，没有主按钮。
- “Batch presets and synchronization” 挂在 `flex_col` 根上，被 stretch 后文字居中（`mod.rs:571-583`）。
- 来源状态逐条输出整句文字（`mod.rs:350-406,700-720`）。
- 条目是根容器的独立子节点：36px 行加上根 gap 16，行距 52px，外面没有列表容器（`mod.rs:522,756-775`）。行标签只有 `平台 · 名称`，同名不同作用域的条目（如两个项目下的 drawio）看起来像重复。
- 详情只在选中后追加到页面底部；没有列表/详情分栏。

## 目标

三页都回到 `ccr-desktop/DESIGN.md` 的工作台语言：静态光场上的半透明纸面、实色阅读面、中性墨色加语义状态色、平台身份色只用于小圆点或细线。各页的具体目标：

- **Overview**：在 2000×1080 和 1280×800 下首屏都被真实内容填满，面板内没有超过 24px 的死区；所有新模块都来自已有数据，并标明口径。
- **Agent sessions**：提供方状态一眼可读、可筛选；行紧凑可扫读；宽窗下列表与转录并排。
- **MCP Manager**：标题和 lens 同一行；工具栏有明确的主操作；来源健康状态是一行 chip；服务器列表是紧凑的两行行，宽窗下列表与详情并排。
- **徽章 / chip / 圆点 / 搜索框 / 空态面板**：抽成一套共享组件，三页共用，两个主题下都满足对比度合同。

## 需求

### R0 共享组件与设计合同

- R0.1 先改 `ccr-desktop/DESIGN.md`，再改 `theme.rs` 和视图。新增 Components 条目：状态 chip、筛选 chip、身份圆点、搜索框、工具栏、两行列表行、来源健康条、虚线空态面板；更新 Overview 版式章节和列表行高规则（单行 36px，两行 56–60px）。
- R0.2 新增共享模块（`shell/badge.rs`）。它提供语调枚举（neutral / success / warning / danger / info）、状态 chip、可切换筛选 chip（带 toggled 语义）、身份圆点、搜索输入框和虚线空态面板。不在页面里各自手写一套。
- R0.3 `theme::platform_color` 补 kimi 与 pi 两个身份色（取 Tauri token：light `#a3467f`/`#7b56b0`，dark `#d775ae`/`#ac88e7`）；omp 继续回落 muted，并在 DESIGN.md 中写明。对比度单测覆盖新增身份色和 chip 的全部语调。
- R0.4 只使用 Kit 默认图标集（106 项），不切换到全量 `AllAssets`。平台标识统一用身份圆点加名称，不给每个 agent 配图标。

### R1 Overview

- R1.1 `WorkbenchReport` 透传 `last_sync_at` 和一个活动快照（来自已经计算出的 `HomeInsightsSnapshot`）。不新增 SQL，所有用量 SQL 仍在 `crates/ccr-usage`。会话归档不可用时，纯用量模块（热力图、分布、模型/项目排行）仍要显示；只依赖会话的数字显示“未索引 / 不可用”。
- R1.2 版式（宽窗 ≥1040）：
  - 页头、KPI 带、平台带保持不变，状态行追加“最后同步 MM-DD HH:mm”。
  - 下排是主列加右栏。主列：逐日请求图（216px，不变）和 53 周活动热力图。右栏宽度随窗口在 280–360px 之间变化，依次为下一步、近 7 日对比前 7 日、数据源。
  - 两列等高，各自最后一个面板吸收剩余高度，且内容顶部对齐、脚注贴底。
  - 首屏之下：小时/星期分布与模型/项目/Agent 排行左右 1:1 并排。
- R1.3 每个全历史或 53 周模块都要在标题旁写明口径（如 “Last 53 weeks · since MM-DD”、“All history”），不能让人误以为它受 7D/30D/90D 控制。
- R1.4 热力图：53 列 × 7 行，5 档强度用主色色阶，0 档用中性 hairline 格。格子尺寸按主列宽度在 8–14px 之间计算，不够时在面板内横向滚动，整页不横滚。必须有文字通道：活跃天数、当前 streak、最长 streak、最忙日。有记录的格子带 tooltip（日期 · 请求数）。
- R1.5 周对比：请求、会话、活跃天各一行，显示本期值、前期值和变化。变化方向只用箭头加中性墨色，不用红绿。会话未索引时写“未索引”。
- R1.6 数据源小节：配置状态、报告截止、最后同步时间。仅显示最后一次成功同步时间。已知成功时间早于报告截止日时，给出 warning 圆点、tooltip 和“同步成功前，较新的用量可能缺失”的说明；无成功记录时只显示圆点与“未记录成功同步”，不附缺失推断。失败同步仍可能导入数据，不能断言之后未导入。逐日图可标最新非零记录日期，不新增导入动作。
- R1.7 分布：用分段胶囊在小时（24 柱）与星期（7 柱）之间切换，并标出峰值。排行：用分段胶囊在模型、项目、Agent 之间切换，取前 8 行，tnum 右对齐，带比例条。
- R1.8 窄宽（<1040）：全部模块单列堆叠，KPI 改为 2×2，不追求填满视口，整页不横滚。

### R2 MCP Manager

- R2.1 lens 改为嵌入式：`McpView` 与 `Converter` 各加 `embed_lens` 槽，`Destination::McpManager` 加入 `Shell::embeds_lens()`；Converter 标题补 heading 语义。每页仍只有一个 h1。
- R2.2 页头：左侧是标题加 lens 胶囊；右侧是 Reload（次按钮，`refresh-cw` 图标）和 Add server（主按钮，`plus` 图标）。描述压成一行 muted 文字。
- R2.3 工具栏：固定宽度的搜索框（`search` 前缀图标、placeholder，按名称和命令筛选）、平台筛选胶囊（全部 / Claude Code / Codex / Antigravity，带计数）、Presets 和 Batch & sync 两个次按钮。不再有任何全宽居中的按钮。
- R2.4 来源健康条：每个来源一个 chip，内容是状态圆点、来源短名和读数。NotRead 用虚线边框；Invalid/Unreadable 用 warning/danger 语调；完整原句作为 aria label 和 tooltip。窄宽时在条内横向滚动。
- R2.5 列表/详情：宽度 ≥1200 时左右 3:2 分栏，列表 ≥360px、详情 ≥320px；窄于此时详情改为页内 drill-in，带返回按钮。列表放在实色阅读面板里，行间用 hairline 分隔，不再受根 gap 影响。列表及未选中空态按内容收紧，条目过多时限制在列内滚动；只有真实详情内容填满 pane。
- R2.6 行结构（两行定高）：
  - 第一行：传输图标（stdio 用 `square-terminal`，URL 用 `globe`）、名称，右侧是身份圆点和平台名。
  - 第二行：mono 命令，超长省略并带 tooltip；只有 URL 时显示 “URL set (hidden)”。接着是作用域文字（User / Project · 末段目录名，完整键放 tooltip）。
  - 只在异常时显示 chip：Disabled、Read-only。
- R2.7 详情：头部是传输图标块、名称和眉题（平台 · 来源文件 · 作用域），右侧是 Copy / Edit / Delete（Delete 用 danger 文字），只读时显示说明。下面是键值网格：传输、命令、URL 和参数（只写是否设置）、作用域、来源、启用状态。`mcp-detail` 的 aria label 保持原有 7 句。
- R2.8 编辑器、预设、来源选择、批量与同步都渲染在详情区（窄宽时 drill-in），不再替换整个列表区。批量模式切换改用 `semantics::tab` 胶囊。预设改成紧凑行，包含名称、描述、标签和“需 API key”（只显示变量名）。
- R2.9 空态与筛选无结果：虚线面板加图标、原因和 Add server 动作。

### R3 Agent sessions

- R3.1 状态拆分：`InsightsView` 在 Sessions 产品面同时持有列表快照和详情快照，二者用独立的请求 gate。选中会话时只加载详情，不清空列表；列表查询条件变化时清除详情。Workbench 渲染器（测试用）的行为不变。
- R3.2 页头：左侧是与 Overview/MCP 左缘一致的标题和一行描述（视觉复核后去除图标块）；右侧是刷新状态 chip（running / finished / failed）、CCR trash 次按钮（只在 Codex 状态不是 NotInstalled 时显示）和 Refresh index 主按钮。
- R3.3 提供方筛选条：8 个 agent 各一个可切换 chip，内容是身份圆点、名称、状态文字和 `source_count`（tnum）。
  - 四种状态：Available 常规；NoData muted；NotInstalled 虚线边框且不可选；Error 用 danger 图标，tooltip 显示 `error_category`。
  - 多选后写入 `SessionListRequest.agents`；列表读取失败时保留上次成功读取的提供方状态，让仍然生效的筛选可见。
  - 窗口宽度低于 `SPLIT_WIDTH` 时，chip 单行横向滚动，不换行。
  - 平台作用域下锁定为对应 agent，并显示锁定说明。
- R3.4 工具栏：搜索框（保留 id `insights-filter-6`，`search` 图标，placeholder “Search title, session ID or path”，Enter 提交，输入停顿约 300ms 后自动查询）、fidelity 分段胶囊（All records / Full / Partial / Locked，aria 组名 Record completeness）、来源状态分段胶囊（All sources / Live / Missing / Deleted，aria 组名 Source state），本页计数与这两个胶囊共行，窄窗下也不独占一行。
- R3.5 列表行（两行定高约 60px，放在实色面板内，行间 hairline）：
  - 第一行：身份圆点、标题（单行省略加 tooltip，缺标题时用 session_id）、右侧 tnum 时间。
  - 第二行：agent · variant · N messages · N tools，以及 mono 路径（省略加 tooltip）。
  - 只在异常时显示 chip：Partial / Locked（warning）、Source missing / Deleted（neutral）。
  - 行具备 listbox/option 语义，支持上下键和 Enter，选中态用中性色阶。
- R3.6 列表与空详情/空废纸篓按内容收紧，长列表在列内滚动，保留 3:2 分栏。列表底部提供带右箭头图标的 ghost “Older sessions” 翻页（`insights-next`，复用 `next()`），以及回到第一页的入口。
- R3.7 详情：
  - 宽度 ≥1200 时 3:2 分栏，窄于此时 drill-in，带 `insights-back`。
  - 转录头部：标题、mono session_id、agent · variant chip；fidelity 为 partial/locked 时显示 warning 提示条；Codex 操作（导出/克隆/移入废纸篓）放在头部。
  - 消息用 hairline 分隔的内容块呈现，不嵌套描边卡片。角色名用 muted 文字并本地化（用户 / 助手 / 工具 / 系统），未知角色保留原文；保留角色图标、时间、mono tool_name、clipped 提示与更早消息翻页。已显示消息数、总消息数和工具次数合并为一行。
- R3.8 废纸篓：入口移到页头。宽窗下面板占据详情区，窄窗下显示在列表上方，带标题 “CCR trash (Codex)” 和关闭按钮。
- R3.9 fidelity 文案从 “complete” 改为 “full record / 完整记录”，不暗示会话已结束。
- R3.10 未索引、空页、错误三种状态：虚线面板，写明原因，并给出 Refresh index 动作。

## 约束（必须保留）

- **不铺零、不编数**：无记录要给原因；派生值要标注；会话未索引时显示“未索引”；不新增 Running/Connected 这类进程状态徽章（`DESIGN.md` 状态条款、`dashboard-presentation-contracts.md`）。
- **MCP 掩码**：读取器只复制 name/scope/command/has_url/has_args/disabled/enabled。URL、args、env/header 的键和值都不进入 UI、aria 或快照；编辑器补丁初值为 `{}`；凭据输入框保持 masked（`research/mcp-manager.md` §4.4、`shared-desktop-services.md:51-81`）。
- **Overview 精确值表焦点合同、图表 216px 与 “since” 收紧、KPI 带与平台带合同**保持不变（`DESIGN.md:243-244`）。
- **材质**：阅读面（列表、表格、输入、详情）为实色，不叠玻璃；平台身份色不用来给面板染色。
- **稳定 id 与焦点契约**：`insights-filter-6` 的 Esc 焦点回归、`session-*` 废纸篓与恢复 id、`insights-row:{archive_id}`、`mcp-entry-{index}` 的排序、`mcp-source-{index}`、`mcp-copy/edit/delete` 不被裁切、批量面板焦点循环、`tool-overview`/`tool-monitoring`/`tool-converter`。测试只按设计变化同步更新，不删断言。
- **工程**：render 中不读文件或 SQLite，不启动进程；Windows UI 线程栈为 1 MiB，可选 builder 要拆成独立函数；生产路径不用 `unwrap`/`expect`；内部注释用中文。
- **门禁**：`just desktop-ci`（含 release 构建）。上一轮出现过“fmt/lint/test 全绿但 release 构建失败”。

## 不做

- 不修复 llmusage CLI 的同步失败（`no such table: message`）；本任务只如实展示新鲜度。
- 不新增 MCP 的 env 数量、args 数量、http/sse 细分、源文件绝对路径（这些需要改读取器并重新评估掩码合同，另开任务）。
- 不做 Tauri 那样的 Agent sessions 6 字段筛选卡（cwd 前缀、日期范围），也不做“追加加载”式分页。
- 不做 Overview 的信号流和 CLI 版本矩阵（需要新接线）。
- 不改侧栏、材质和其他页面。

## 验收标准

- [x] `ccr-desktop/DESIGN.md` 已先于代码更新：新增组件条目、Overview 版式、行高规则、身份色范围；`theme.rs` frontmatter 与 token 同步。
- [x] 共享组件只在 `shell/badge.rs`（或同级共享模块）中定义；三页的 chip、圆点、搜索框、空态面板全部复用它，不存在页面私有的同类实现（`overview.rs` 原有的 `status_dot` 并入共享模块）。
- [x] 对比度单测覆盖：kimi/pi 身份色；chip 五种语调在 light/dark 两个主题下与 reading 面的文字对比 ≥4.5:1；描边为装饰（标签不是控件），只要求比填充更清楚地勾出形状（实施中修订，见 DESIGN.md 与 theme-token-contracts.md）。
- [ ] Overview 2000×1080（逻辑尺寸以实际 DPI 换算）和 1280×800：首屏内容到达状态栏上方 ≤24px；逐日图面板 x 轴标签下方空白 ≤24px（headless bounds 断言，加原生截图）。
- [x] Overview 新模块都标明口径；会话归档不可用的 fixture 下，热力图和排行仍显示，会话数字显示“未索引 / 不可用”；最后同步早于报告截止日时，显示 warning 说明（单测）。
- [x] MCP：`page-mcp-manager` 中标题与 `tool-mcp`/`tool-converter` 同行；`mcp-filter` 有 placeholder；来源 chip 的 aria label 等于原整句；宽窗下 `mcp-detail` 与列表并排、窄窗下出现 drill-in 返回；掩码相关断言全部保留且通过。
- [x] Agent sessions：提供方 chip 可切换且有 toggled 语义，筛选结果正确（多选、平台作用域锁定）；宽窗下选中会话后列表仍在、详情在右侧；`insights-filter-6` 的 Esc 焦点回归与废纸篓分页测试通过；长标题单行省略。
- [x] `just desktop-fmt-check`、`just desktop-lint`、`just desktop-test`、`just desktop-test-ui`、`just desktop-ci` 全绿，收据写入 `research/integration.md`。
- [ ] 原生截图：三页 × light/dark × 2000×1080 和 1280×800，外加 960×640 最小窗口和中文各一张，存到 `research/native/`。未运行的项目逐条列出，不记为通过。
- [ ] 截图对照 `ccr-desktop/DESIGN.md` 和 design-atlas 反 AI 味判定清单逐条走查，结论写入 `research/native-visual.md`。

## 决策记录（2026-11-07 用户确认）

- Overview：补真实洞察模块（热力图、周对比、数据源新鲜度在首屏；分布、排行在首屏之下）。
- Agent sessions：做列表/详情分栏。
- Agent sessions：提供方 chip 做成可点击的多选筛选。

## 本轮续作验收补充（2026-11-08）

来源：`HANDOFF-gpui-redesign.md` §5.1 与 `research/visual-review.md`。只修复已分诊项目 A–F，保留表中暂不修复项目。

- [x] 少量 MCP/会话条目的面板明显短于列；长列表不越过列底；空详情和空废纸篓按内容收紧。
- [x] 960×640 的 `session-cards` 顶部小于窗口高度的 45%。
- [x] disabled ghost 保持透明与 muted；Overview 非峰值 sparkline 对 reading 在双主题下 ≥3:1。
- [x] 搜索完整文案、Codex 废纸篓可见条件、失败读取后的 provider 筛选状态都有回归覆盖。

## 续作检查点

§5.1 实现与规格已提交 `a145abba`，完整 desktop-ci 及最终错误状态增量回归通过，详见 `research/integration.md`。Overview 原生 ≤24px 测量、完整主题/尺寸/中文截图矩阵仍未完成，故对应验收不勾选且任务不归档。新截图独立复核发现本地报告没有显式时区后缀；源码审计确认是既有展示缺口，时间换算正确，另记在 `research/native-visual.md`，不扩展本轮 A–F 修复。

## User-requested archive — 2026-11-09

The user explicitly requested committing all changes and archiving this task. The earlier in-progress/do-not-archive notes are historical and superseded by that instruction. Implementation and the recorded desktop gates passed. The unchecked native theme/size/Chinese matrix, calibrated gap measurements and other native acceptance remain **UNVERIFIED**; archival does not turn them into passes. Preserve the unchecked acceptance items and remaining gaps for future work. No push.
