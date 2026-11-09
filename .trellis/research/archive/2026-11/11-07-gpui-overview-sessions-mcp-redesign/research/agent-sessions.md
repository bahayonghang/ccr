# Research: Agent sessions 页面（Tauri ccr-ui vs GPUI ccr-desktop）

- **Query**: 对比 Tauri 与 GPUI 的 Agent sessions 页面（布局、徽章、样式、行结构、搜索/筛选、废纸篓、状态），盘点 GPUI 可复用原语与测试契约，给出基于现有数据的 GPUI 重设计建议
- **Scope**: internal（附 Kit 依赖源码核对）
- **Date**: 2026-11-07

---

## 0. 结论速览

| 维度 | Tauri (`ccr-ui`) | GPUI (`ccr-desktop`) 侧栏 Agent sessions |
|---|---|---|
| 页头 | 图标方块 + 标题 + 描述 + 刷新状态徽章 + 次按钮“用量”+ 主按钮“刷新” | 标题 + 描述 + “刷新索引”按钮；无图标、无状态徽章 |
| 提供方条 | 独立卡片内 8 格网格，可点按钮（多选筛选），图标 + 名称 + 可用性 + 源数量 + fidelity | 一行 `secondary` 底的 28px pill 文本，只有“readable”用 primary 色，其余全部 muted；不可点；无图标、无计数 |
| 筛选 | 独立卡片：搜索 / cwd / 起止日期 / 状态 / fidelity + 清除 | 仅 1 个输入（标签写“Title”，实际匹配 session_id/title/cwd）+ ghost “Search”按钮 |
| 列表/详情 | ≥900px 时左右分栏（列表 20–24rem + 详情自适应），首个可读会话自动选中 | 无分栏：点行后整页替换为转录（drill-in），带“返回列表” |
| 行 | 图标 + 单行截断标题 + mono session_id + 右侧时间；meta：agent·variant / 消息数 / 工具数；cwd 截断 + fidelity | 84px 定高；标题无截断（长中文顶满）；meta 一行纯文本 “claude · 272 messages · complete”；路径一行；时间右上 |
| 废纸篓 | Agent sessions 页没有（只在 Codex sessions 详情里） | 列表上方常驻 ghost 按钮“Open / refresh CCR trash”（Codex 专用），视觉上像居中孤立链接 |
| 分页 | 虚拟列表 + “加载更多”（追加） | 列表无翻页按钮（状态行写“Older ones follow”但 Sessions 产品面没有 `insights-next`）；转录有“更早的消息” |

---

## 1. Tauri 页面（`ccr-ui/src/features/agent-sessions/`）

### 1.1 文件

| 文件 | 说明 |
|---|---|
| `ccr-ui/src/features/agent-sessions/AgentSessionsView.tsx` | 页面容器：查询、页头、提供方条、筛选、分栏 |
| `ccr-ui/src/features/agent-sessions/AgentSessionProviderStrip.tsx` | 8 个提供方切换按钮 |
| `ccr-ui/src/features/agent-sessions/AgentSessionList.tsx` | 虚拟化会话列表与行 |
| `ccr-ui/src/features/agent-sessions/AgentSessionTranscript.tsx` | 详情/转录、局部查找、加载更早 |
| `ccr-ui/src/features/agent-sessions/AgentSessionPageStates.tsx` | 刷新徽章、错误横幅、自动选中逻辑 |
| `ccr-ui/src/features/agent-sessions/model.ts` | agent 顺序、图标映射、可用性/时间格式化、错误码 |
| `ccr-ui/src/types/generated/agent_sessions/AgentSessionListItemDto.ts` | 行 DTO：archive_id, session_id, agent, variant, title?, cwd, message_count, user/assistant/tool 计数, created_at, updated_at, source_state, fidelity |
| `ccr-ui/src/types/generated/agent_sessions/AgentSessionProviderStatusDto.ts` | 提供方 DTO：agent, availability, variants, source_count, fidelity?, error_category? |

### 1.2 页面布局（AgentSessionsView.tsx）

- 常量：`LIST_LIMIT = 80`、`DETAIL_LIMIT = 100`（`AgentSessionsView.tsx:30-31`）。
- 进入页面自动启动一次增量刷新（`AgentSessionsView.tsx:219-223`），刷新结束后失效全部查询（`:225-231`）。
- 页头 `AgentSessionsHeader`（`:108-139`）：
  - `leading`：`h-10 w-10 rounded-xl border border-accent-primary/25 bg-accent-primary/10` 的图标方块，内为 `MessagesSquare` 图标 `text-accent-primary`（`:119-123`）。
  - `status`：`AgentRefreshBadge`（`:124`）。
  - `actions`：次按钮 `Link to="/usage"`（Activity 图标，`:127-130`）+ 主按钮刷新（RefreshCw，刷新中 `animate-spin`，`:131-134`）。
- 环境门 `AgentSessionEnvironmentGate`（`:64-98`）：加载中 Spinner；错误 `EmptyState icon="AlertTriangle"`；非本地环境 `EmptyState icon="MonitorOff"`。
- 内容自上而下（`:283-385`，`space-y-4`）：
  1. 提供方卡片：`rounded-2xl border border-border-default/20 bg-bg-surface p-4`（`:291-300`）。
  2. 筛选卡片（同样卡片类，`:302-346`）：标题“筛选”+ quiet 小按钮“清除筛选”（RotateCcw，`:303-309`）；栅格 `grid gap-3 md:grid-cols-2 xl:grid-cols-[minmax(16rem,2fr)_minmax(12rem,1.5fr)_repeat(4,minmax(8rem,1fr))]`（`:310`）；字段：搜索 `type=search`、cwd、开始日期、结束日期、状态（all/live/missing/deleted_by_user）、fidelity（all/full/partial/locked）（`:311-344`）。输入类统一 `rounded-xl border border-border-interactive bg-bg-elevated px-3 py-2 text-sm`。标签 `text-xs font-medium text-text-muted`。
  3. 错误横幅 `AgentSessionErrorBanner`（`:348`）。
  4. 分栏：`grid gap-4 min-[56.25rem]:grid-cols-[minmax(20rem,24rem)_minmax(0,1fr)]`（`:350`）。左：列表卡片（h2“会话”，`:351-366`）；右：详情卡片（sr-only h2，`:367-381`）。
- 自动选中：首个 `source_state` 不是 missing/deleted_by_user 且未被跳过的会话（`AgentSessionPageStates.tsx:8-20`）；详情读不到（unreadable 错误码）时自动跳过该会话（`AgentSessionsView.tsx:233-239`）。
- 筛选使用 `useDeferredValue`，输入即查询，无“搜索”按钮（`:146`、`:164-167`）。提供方多选通过 `selectedAgents` 进入请求 `agents`（`:50-62`、`:241-245`）。

### 1.3 提供方条（AgentSessionProviderStrip.tsx）

- 小节标题“提供方”+ 右侧提示 `providerFilterHint`（`:84-90`）。
- 栅格 `grid grid-cols-2 gap-2 sm:grid-cols-4 xl:grid-cols-8`，始终渲染全部 8 个 agent（`:91-104`；顺序 `model.ts:6-15`：grok, claude, codex, opencode, pi, omp, antigravity, kimi）。
- 每格是 `<button aria-pressed>`（`:42-51`）：
  - 基础类 `min-w-0 rounded-xl border bg-bg-elevated px-3 py-3 text-left`；选中 `border-accent-primary/50 bg-accent-primary/10`；未选 `border-border-default/20 hover:border-border-default/50`（`:46-49`）。
  - 第一行：agent 图标（`text-text-secondary`，映射 `model.ts:17-26`：grok Zap / claude Code2 / codex Settings / opencode TerminalSquare / pi Bot / omp Workflow / antigravity Sparkles / kimi Moon）+ 名称 `text-xs font-semibold`（`:52-55`）。
  - 第二行：可用性文本（truncate）+ 右侧 `tabular-nums` 的 `source_count`（`:56-59`）。可用性文案 `model.ts:49-60`（not_installed / no_data / available / error）。
  - 第三行（可选）：fidelity `text-text-ghost`（`:60`）。
- **注意**：Tauri 不对可用性使用语义颜色，也不使用平台身份色，全部是中性墨色；状态区分靠文字。

### 1.4 列表行（AgentSessionList.tsx）

- 行是 `<button>`（`:22-51`）：`w-full rounded-xl border bg-bg-elevated p-3 text-left`；激活 `border-accent-primary/50 bg-accent-primary/10`；否则 `border-border-default/20 hover:border-border-default/50`（`:25-26`）。
- 结构：
  - 顶行左：agent 图标 + `strong.truncate text-sm` 标题（缺省用 session_id）（`:32-35`）；其下 `font-mono text-xs text-text-ghost` 的 session_id（`:36`）。
  - 顶行右：`shrink-0 text-xs tabular-nums text-text-muted` 的时间（`:38-40`），格式 `Intl.DateTimeFormat(month short, day, hour, minute)`（`model.ts:67-76`），非相对时间。
  - meta 行：`flex flex-wrap gap-x-3 text-xs text-text-muted`：`agent · variant`、`N 条消息`、`N 次工具`（`:42-46`）。
  - 底行：cwd `truncate text-text-ghost`（空显示 `—`）+ 右侧 fidelity 文本（`:47-50`）。
- 没有 status chip / 彩色徽章；source_state 不显示在行上。
- 虚拟化：`useVirtualizer` estimate 132px、overscan 8、key=archive_id（`:83-89`）；滚动容器 `h-[34rem] overflow-y-auto`，`data-testid="agent-session-virtual-list"`（`:110`）；行间 `pb-2`（`:120`）。
- “加载更多”：quiet 全宽按钮，追加下一页（`:135-140`）。
- 状态：加载中 Spinner（`:91-98`）；错误 EmptyState + 重试（`:99-103`）；空 `EmptyState icon="Inbox"`（`:104-106`）。

### 1.5 详情 / 转录（AgentSessionTranscript.tsx）

- 未选中：`EmptyState icon="MessagesSquare"`（`:95-97`）；加载/错误态（`:98-109`，错误码映射 `model.ts:100-123`）。
- 头部（`:114-137`）：`truncate text-lg font-semibold` 标题 + mono session_id；右侧 `rounded-full border border-border-default/20 bg-bg-elevated px-3 py-1 text-xs` 的 “agent · variant” pill（`:120-122`）；说明 `detailHint`；partial / locked 时 `rounded-lg bg-accent-warning/10 text-accent-warning` 提示条（AlertTriangle / Lock 图标，`:125-136`）。
- 已加载消息内查找：带 Search 图标前缀的输入（`:139-150`），仅前端过滤（`:81-85`）。
- “加载更早”（ChevronUp，`:152-157`）；虚拟化消息列表 `h-[38rem]`（`:162`）。
- 消息卡（`:17-43`）：`rounded-xl border bg-bg-elevated p-4`，user 加 `border-accent-primary/25`；头部角色图标（User/Wrench/Bot）+ 角色名 + mono tool_name + 右侧时间或 `#ordinal`；正文 `whitespace-pre-wrap text-sm leading-6`；clipped 时 warning 色 Scissors 提示。

### 1.6 其他状态组件（AgentSessionPageStates.tsx）

- `AgentRefreshBadge`（`:31-46`）：`inline-flex rounded-full border border-border-default/20 bg-bg-elevated px-3 py-1.5 text-xs text-text-secondary`，Spinner 或 CheckCircle2/AlertTriangle + “刷新完成 · parsed/discovered”。
- `AgentSessionErrorBanner`（`:54-67`）：`rounded-xl border border-accent-danger/20 bg-accent-danger/10 text-accent-danger`，role=alert。

### 1.7 废纸篓

- `ccr-ui/src/features/agent-sessions/` 中没有任何 trash 相关代码（rg 0 命中）。Tauri 的 Codex 废纸篓在 `ccr-ui/src/features/codex/sessions/SessionDetailPanel.tsx`（独立 Codex 会话页），不在 Agent sessions。

### 1.8 Tauri 设计 token / 规则

- `ccr-ui/src/styles/tokens.css`（light）：`--color-bg-elevated #f2f7f5`（:13）、`--color-bg-surface #fff`（:15）、`--color-border-default #b3c3bc`（:27）、`--color-text-muted #5c6b76`（:40）、`--color-text-ghost #74828b`（:42）、`--color-accent-primary #156f4e`（:50）。
- 平台身份 token（每个平台都有 base / surface / border / text 四档）：claude `:108-112`（text #a15840，surface #f7ece9）、codex `:113-117`、grok `:118-122`、gemini `:123-127`、opencode `:128-132`、antigravity `:133-137`、kimi-code `:142-146`（#b65790 / text #a3467f）、pi `:147-151`（#9773d0 / text #7b56b0）、zcode `:152-`。暗色同名 token 在 `:293` 起。omp 无 token。
- `ccr-ui/DESIGN.md:146-150`：绿色仅用于主操作/链接/选中/焦点；未知、pending、失败不能转为绿或紫；success/warning/danger/info 各自独立且须配文字/图标。
- `ccr-ui/DESIGN.md:163-165`：平台身份色是分类色，只用于 2px tick、圆点、图例色块、图表段，不给面板整体染色。
- `ccr-ui/DESIGN.md:211`：pill 适合独立操作或标签，不适合表格或整页。

---

## 2. GPUI 页面（`ccr-desktop`）

### 2.1 文件与挂载

| 文件 | 说明 |
|---|---|
| `ccr-desktop/src/features/insights/product.rs` | **侧栏 Agent sessions 实际渲染器**（header / status_line / providers / search / session_list / transcript） |
| `ccr-desktop/src/features/insights/mod.rs` | `InsightsView` 状态、查询、分页、旧 Workbench 渲染器（测试仍用） |
| `ccr-desktop/src/features/insights/rows.rs` | Workbench 渲染器的行数据 / 详情行（`:29-30`、`:140-152`） |
| `ccr-desktop/src/features/insights/session_operations.rs` | Codex 会话操作（导出/克隆/移入废纸篓）与确认框 |
| `ccr-desktop/src/features/insights/session_operations/delivery.rs` | 废纸篓面板、分页、恢复确认 |
| `ccr-desktop/src/runtime/insights.rs` | `InsightsSnapshot`、`RefreshStatus`、`read_sessions` |
| `crates/ccr-app/src/sessions/mod.rs` | `SessionListRequest` / `SessionPage` / `SessionDetail` |
| `crates/ccr-db/src/database/repositories/usage_repo.rs` | `AgentSessionArchiveRow`、`UsageSourceState`、搜索 SQL |
| `crates/ccr-store/src/sessions/providers.rs` | `ProviderAvailability`、`AgentSessionProviderStatus`、`AgentSessionMessage(Page)` |

- 外壳：`Destination::AgentSessions` → `page-agent-sessions`（`ccr-desktop/src/shell/mod.rs:54`）；进入时 `open_global(InsightsPage::Sessions)`（`shell/mod.rs:473-474`）；该目的地没有 lens 胶囊（`shell/mod.rs:879-882` 返回 None）。平台工具栏的 `Lens::Sessions` 走 `open_platform(InsightsPage::Sessions, platform)`（`shell/mod.rs:420-430`）。
- `open_product` 把 `surface` 设为 `Surface::Sessions`（`mod.rs:178-190`），`Render` 在该 surface 下直接交给 `product::sessions`（`mod.rs:684-686`）。`Surface::Workbench` 的旧渲染器（`mod.rs:687-1089`）只在 `InsightsView::new` 默认状态下出现，主要服务测试。
- `open_platform` 只为 Claude/Codex 预填 agent family（`fields[7]`，`mod.rs:208-215`）；`product.rs` 不渲染 `fields[7]`，所以平台作用域在 UI 上不可见。

### 2.2 当前渲染结构（product.rs）

- 根：`session-page`，`size_full flex_col overflow_y_scroll p(CONTENT_PADDING=24) gap(SECTION_GAP=16)`（`product.rs:27-39`）。子项：header → status_line → providers。
- 分支（`product.rs:40-48`）：
  - 有待确认操作 → 只追加 `render_session_operations`（确认框）。
  - 快照是 `Detail` → 追加 `transcript`（**整页替换列表**，无分栏）。
  - 否则 → `render_session_operations`（含废纸篓面板）+ `search` + `session_list`。
- **header**（`product.rs:51-94`）：左列 `session-title`（TITLE_SIZE 20 / SEMIBOLD）+ META_SIZE muted 描述；右侧 `insights-index-refresh` 主按钮（“刷新索引 / 正在索引”，running 时 disabled+busy）。无图标、无状态徽章、无次要操作。
- **status_line**（`product.rs:96-176`）：`session-status`，role=Status，META_SIZE muted 单行文字。来源：loading / error / `RefreshStatus`（Running / Failed / Interrupted / Finished→“发现 X，写入 Y”）/ Idle 时按快照给出“本页 N 条已索引会话，还有更早的。”等（`:150-163`）。
- **providers**（`product.rs:178-232`）：遍历 `statuses`，每项 `px(8) h(28) rounded(PILL_RADIUS) bg(theme.secondary) text META_SIZE`；文案 `"{agent} {readable|no files|not installed|unreadable}"`；颜色只有两档：Available → `theme.primary`，其余（NoData / NotInstalled / Error）全部 `muted_foreground`（`:184-228`）。无图标、无计数、无点击、无 id/aria。浅色主题下 `secondary` 底与纸面接近，因此用户看到的是“绿色/灰色纯文字”。
- **search**（`product.rs:234-266`）：280px 宽列：META_SIZE 标签“Title / 标题”+ `Input::new(&fields[6]).id("insights-filter-6")`；旁边 ghost 按钮 `session-search`（“Search / 搜索”）触发 `refresh`。无前缀图标、无 placeholder、没有 Enter 提交或 Change 即查。
- **session_list**（`product.rs:268-386`）：
  - 未索引 / 空页：带边框的 `INPUT_RADIUS` 提示块（`:272-300`）。
  - `uniform_list("session-cards")`，`flex_1 min_h(240)`（`:301-385`）。
  - 行 `insights-row:{archive_id}`（`:321-339`）：`h(84) w_full px(16) flex_col justify_center gap(4) bg(background) border_1 border_color(border) rounded(INPUT_RADIUS=8) hover bg(secondary)`，点击 `select(index)`。
  - 行内容：
    - 第一行 `justify_between`：标题 div 仅 `font_weight(SEMIBOLD)`，**无 `min_w_0` / `text_ellipsis` / `flex_1`**（`:340-348`），长标题撑满整行；右侧时间 META_SIZE muted，格式 `updated_at` 本地 `%m-%d %H:%M`（`:316-320`、`:349-354`）。
    - 第二行：`"{platform} · {message_count} 条消息 · {fidelity}"` 纯文本（`:356-367`），fidelity 映射 full→complete/完整、partial、locked、unknown（`:463-470`）。
    - 第三行：cwd 原样或“没有项目路径”（`:368-377`），非 mono、无截断。
  - 行没有 role / aria / 键盘选择；行之间没有间距（uniform_list 紧贴，边框相邻）。
  - **列表没有翻页按钮**：`insights-next` 只出现在 transcript（`:450-458`）与 Workbench 渲染器（`mod.rs:1078-1087`）。`next()` 支持 Sessions 快照（`mod.rs:457-462`），但产品面未暴露入口。
- **transcript**（`product.rs:388-461`）：顶部 `render_session_operations`（Codex 导出/克隆/废纸篓按钮），“返回列表”`insights-back` + “更早的消息”`insights-next`（ghost），然后 `session-transcript` 消息卡：`p(16) border rounded(8)`，角色名 META_SIZE（assistant 用 `palette.output` 紫，其余 primary 绿，`:415-425`）+ 正文。不显示 timestamp / tool_name / clipped / fidelity 提示，也没有会话标题头部。

### 2.3 数据来源与可用字段

- 快照：`InsightsSnapshot::Sessions { page: SessionPage, statuses: Vec<AgentSessionProviderStatus>, indexed: bool }` 与 `Detail(SessionDetail)`（`ccr-desktop/src/runtime/insights.rs:303-314`）。**同一时刻只有一个快照**（`mod.rs:87`，`submit` 先清空 `mod.rs:383`），所以进入详情会丢掉列表页。
- `read_sessions`（`runtime/insights.rs:428-465`）：每次列表查询都调用 `registry.provider_statuses()`（`:432-434`）；归档 DB 不存在或无数据 → `indexed:false`。
- 查询（`mod.rs:352-375`）：`agents` 来自 `fields[7]`，`query` 来自 `fields[6]`，`limit: Some(100)`；`cwd_prefix / started_at / ended_at / source_state / fidelity` 后端支持（`crates/ccr-app/src/sessions/mod.rs:52-62`）但 GPUI 未使用。
- 搜索语义：`(session_id LIKE ? OR title LIKE ? OR cwd LIKE ?)`（`crates/ccr-db/src/database/repositories/usage_repo.rs:991-1000`），GPUI 标签写“Title”与实际不符。
- **行字段** `AgentSessionArchiveRow`（`usage_repo.rs:176-191`）：`archive_id, session_id, platform, source_variant, title: Option, cwd, message_count, user_message_count, assistant_message_count, tool_use_count, created_at, updated_at, source_state (Live/Missing/DeletedByUser, :38-43), source_fidelity: String`。注释：不含物理源路径（`ccr-app/src/sessions/mod.rs:65`）。
- **提供方字段** `AgentSessionProviderStatus`（`crates/ccr-store/src/sessions/providers.rs:252-259`）：`agent, availability, variants, source_count, fidelity: Option, error_category: Option`。`provider_statuses()` 对全部 8 个定义各产出一项（`providers.rs:337-382`）：根目录都不存在 → NotInstalled；发现 0 个源 → NoData；否则 Available；discover 失败 → Error + `error_category = "discovery_failed"`。`source_count` = 磁盘上发现的源数量（不是已索引会话数）；`fidelity` 取所有源中最差一档。
- `ProviderAvailability`：NotInstalled / NoData / Available / Error（`providers.rs:114-119`）。Agent 顺序与 id（`providers.rs:38-60`）：grok, claude, codex, opencode, pi, omp, antigravity, kimi。
- **转录字段** `AgentSessionMessage`（`providers.rs:157-165`）：`key, ordinal, role, timestamp: Option, content, tool_name: Option, clipped`；页 `AgentSessionMessagePage { messages, next_before, has_older, fidelity }`（`:168-173`）；`SessionDetail { archive_id, agent, variant, page }`（`ccr-app/src/sessions/mod.rs:70-75`）。
- **刷新报告**：`RefreshStatus::{Idle, Running(id), Finished(id, report), Failed, Interrupted}`（`runtime/insights.rs:317-324`），`report.total` 含 discovered / unchanged / parsed / upserted / partial / locked / errors（Workbench 渲染器用到，`mod.rs:765-789`）。
- 不存在的数据：跨页总会话数（只有本页 `items.len()` + `next_cursor`）；按提供方统计的已索引会话数；会话“进行中/完成”状态（GPUI 的 “complete” 其实是 fidelity=full）。

### 2.4 废纸篓与会话操作

- `render_session_operations`（`session_operations.rs:184-206`）在列表视图下始终渲染：actions（仅 Detail 且 `agent==Codex && variant=="codex-live"` 时出现导出/克隆/移入废纸篓，`:250-296`）+ feedback（`:297-349`）+ export 预览 + 废纸篓面板。
- 废纸篓面板 `session-trash-panel`（`delivery.rs:156-206`）：`flex_col gap(8)`，首个子元素 ghost 按钮 `session-open-trash`“Open / refresh CCR trash / 打开 / 刷新 CCR 废纸篓”（`:164-176`）。父容器是 `flex_col` 且按钮未设 `self_start`/宽度，按 flex 默认 stretch 推断会拉满整行、ghost 无底色、文字居中——即截图中“居中孤立链接”（推断，未做像素验证）。
- 打开后：说明文字 + 空提示 + 分页（`session-trash-prev/next`、`session-trash-page`，`delivery.rs:208-250`，每页 `TRASH_PAGE_SIZE = 6`，`delivery.rs:5`）+ 行（`session-trash-row:…`，标题/元数据 `text_ellipsis` + Tooltip，`session-restore:{index}`“确认恢复…”，`delivery.rs:251-310`）。
- 废纸篓服务是 Codex 专用（`ccr_codex::CodexSessionTrashService`），但入口在所有提供方的列表上方都显示。
- 确认框：AlertDialog + focus trap + Esc 取消（`session_operations.rs:207-249`）；恢复确认在 `delivery.rs:311-`；确认中 `product.rs:40-42` 会隐藏搜索与列表。

---

## 3. GPUI 可复用原语清单

### 3.1 `ccr-desktop/src/theme.rs`

| 名称 | 位置 | 说明 |
|---|---|---|
| 尺寸常量 `CONTENT_PADDING 24 / SECTION_GAP 16 / CONTROL_GAP 8 / ROW_HEIGHT 36 / META_SIZE 12 / BODY_SIZE 14 / TITLE_SIZE 20 / SECTION_TITLE_SIZE 16` | `theme.rs:12-19,37` | 排版与间距 |
| 圆角 `SELECTION_RADIUS 4 / INPUT_RADIUS 8 / PANEL_RADIUS 12 / PILL_RADIUS 9999 / CAPSULE_RADIUS 20` | `theme.rs:20-25` | |
| 断点 `NARROW_WIDTH 1040 / SPLIT_WIDTH 1200` | `theme.rs:29-30` | 分栏断点已有常量 |
| `Palette`（surface / neutral / elevated / overlay / text / secondary_text / muted / subtle / control / primary / output / success / warning / danger / info） | `theme.rs:105-126`，值 `:128-192` | 语义色：light success #166534、warning #92400e、danger #b91c1c、info #245cb3（`:186-189`）；dark `:155-158` |
| `hairline(mode)` | `theme.rs:204-206` | 装饰分隔线 |
| `tab_colors` / `tab_selected` | `theme.rs:208-239` | 分段胶囊配色 |
| `nav_colors` | `theme.rs:250-266` | 侧栏 reading pill |
| `platform_color(id, mode)` | `theme.rs:273-284` | 平台身份墨色，**仅** claude / codex / grok / antigravity(gemini) / opencode；kimi / pi / omp 回落为 `muted` |
| `chart_series(id, mode)` | `theme.rs:287-294` | 降饱和身份色 |
| `tabular_figures()` | `theme.rs:297-` | 数字 tnum |
| 对比度测试 | `theme.rs:600-660`（`composited_material_surfaces_preserve_every_ink_contrast`、`solid_surface_pairs_preserve_..._status_contrast`、`chart_series_keep_identity_hue...` 只覆盖 4 个平台） | 新增色调需扩展 |

- **没有** badge / chip / tag / status-dot 的通用 helper，也没有平台图标映射到 agent id 的函数。

### 3.2 其他位置

| 名称 | 位置 | 说明 |
|---|---|---|
| `page_header::capsule(cx)` | `ccr-desktop/src/shell/page_header.rs:21-36` | 内容宽度分段胶囊容器（tab-well + 1px 亮边） |
| `semantics::button(button, disabled, busy, cx)` | `ccr-desktop/src/shell/semantics.rs:16-39` | pill 圆角 + disabled/busy 无障碍 |
| `semantics::tab(button, selected, disabled, cx)` | `ccr-desktop/src/shell/semantics.rs:45-82` | 分段选项（selected/toggled、浮起 pill） |
| `status_dot(color)`（私有） | `ccr-desktop/src/features/overview.rs:967-974` | 7px 圆点 |
| `platform_cell` 2px 身份色顶线 | `ccr-desktop/src/features/overview.rs:1094-1114` | 身份色用法先例 |
| 侧栏平台图标 | `ccr-desktop/src/shell/platform_navigation.rs:5-22`（Claude `Bot` / Codex `Cpu` / Grok `Asterisk` / Antigravity `Globe` / OpenCode `SquareTerminal` / AgentSessions `BookOpen`）；身份色只在 hover/选中显示（`shell/mod.rs:645-658`） | |
| `text_ellipsis()` + Tooltip 先例 | `delivery.rs:278-295`、`shell/mod.rs:666`、`features/mcp/batch.rs:490`、`observer/presentation.rs:108` | 单行省略 + 完整提示 |
| Kit `Tag`（primary/secondary/danger/success/warning/info/custom/outline/rounded_full） | `~/.cargo/registry/.../gpui-component-0.7.3/src/tag.rs:10-27,125-210` | 仓库内未使用；`Tag::custom(color, fg, border)` 可接 Palette |
| Kit `Badge`（dot / count / icon / color） | `gpui-component-0.7.3/src/badge.rs:31-85` | 仓库内未使用；是角标型而非文字 chip |
| Kit `Input::prefix/suffix/cleanable` | `gpui-component-0.7.3/src/input/input.rs:276-320` | 可做带 Search 图标的搜索框；`InputState::placeholder` 用法见 `ccr-desktop/src/probe.rs:22`；`InputEvent::Change` 订阅先例 `features/mcp/mod.rs:55` |

### 3.3 图标资源限制（重要）

- 应用注册的是 `gpui_kit::assets::Assets`（`ccr-desktop/src/lib.rs:87`），即 Kit **默认图标包**（`gpui-kit-assets-0.7.3/default-icons.txt`，106 项），不是 `AllAssets`（1830 项）。
- 默认包中已核对存在：search, book-open, folder, folder-open, circle-alert, triangle-alert, circle-check, circle-x, inbox, info, bot, cpu, globe, square-terminal, asterisk, user, arrow-left, chevron-right, refresh-cw。
- **不在默认包**：trash, messages-square, clock, lock, funnel（以及 Tauri 用的 zap / sparkles / moon / workflow 需另查）。使用它们需要切换 `AllAssets` 或自定义资产源（二进制体积影响未评估）。

### 3.4 设计合同（`ccr-desktop/DESIGN.md`）

- 平台身份为“小图标与文字，不给整个页面染色”；success/warning/danger 只表示真实状态且配文字或图标（`DESIGN.md:103`）。
- 主列表与详情默认 3:2，列表 ≥360px、详情 ≥320px；空间不足改为内容区内 drill-in 并带返回；列表行 36px、表头 32px，独立滚动；分页/虚拟化保留键盘选择和 stable ID（`DESIGN.md:137`）。
- 形状：按钮 pill、输入 8px、列表可选背景 4px、面板 12px；“数据列表不用 pill 包每一项”（`DESIGN.md:159-161`）。
- 表格/列表：行 hover/selection 用中性色阶；过长名称可 tooltip 补全（`DESIGN.md` Components“表格/列表”条）。
- 状态：empty 说明原因并给下一步；loading 保留布局；unsupported 与未接入区分（`DESIGN.md` Components“状态”条）。
- 阅读面（列表/输入）实色，不叠玻璃（`DESIGN.md` Elevation 第 1-3 条）。
- Windows UI 线程 1 MiB 栈：可选 builder 要拆到独立函数（`mod.rs:572-574`、`delivery.rs:207`、`page_header.rs:152` 注释）。

### 3.5 原生能力（`.trellis/research/archive/2026-11/11-07-gpui-airy-glass-polish/research/native-capabilities.md`）

- `border_dashed()` 原生可用（§3），适合空态虚线面板。
- `overflow_x_scroll()` 需在有 id 的 InteractiveElement 上（§4）。
- pill 统一走 `semantics::button` 或 `.rounded(px(PILL_RADIUS))`（§5）。
- `Icon::data` / `svg().data` 可载入字节 SVG（单色 mask），可作为补充图标的途径（§1/§6 + 实施结论）。
- `uniform_list` 要求定高行：标题必须单行省略（`text_ellipsis` + `min_w_0` 父级），否则溢出。

---

## 4. 测试 / 契约（须保留或同步更新）

### 4.1 直接作用于 Sessions 产品面（`Surface::Sessions`）

`ccr-desktop/src/features/insights/session_operations.rs` 测试模块（`:444-731`）：
- `session_trash_pagination_bounds_large_collections_and_keeps_controls_reachable`（`:456-`）：用 `open_product(InsightsPage::Sessions)`；点击 `session-open-trash`，断言 6 个“Review restore…”标签、`session-restore:5` 存在 / `:6` 不存在，`session-trash-prev/next` 翻页边界。
- `selected_codex_controls_export_cancel_and_clear_plaintext`（`:536-`）：`session-export`、`session-save-markdown`、`session-copy-markdown`、`session-trash`、`session-action-cancel`、`session-open-trash`、`session-restore:0`、`session-restore-prepare-cancel`、`session-restore-cancel`、`session-restore-confirm`；**关键焦点契约**：先点击 `insights-filter-6` 再点恢复，Esc 后焦点必须回到 `insights-filter-6`（`:670-698`）。因此搜索输入 id `insights-filter-6` 必须保留，并且废纸篓入口和搜索框需在同一列表视图中都可点击。
- `session-trash-row:{session_id}:{deleted_at}:{relative_path}`、`session-trash-page` 为稳定 id。

### 4.2 作用于 Workbench 渲染器（`mod.rs` 旧渲染，非侧栏页）

`ccr-desktop/src/features/insights/tests.rs`：
- `archive_candidate_rows_are_virtualized_and_keyboard_selection_uses_opaque_ids`（`:314-376`）：10 000 行、`rendered_rows` 虚拟化、上下键选中 `as-fixture-00001`、`row(1).label` 以 “omp” 开头、Esc 清除。
- `rendered_archive_drill_in_pages_real_provider_messages_and_clears_plaintext`（`:451-583`）：点击 `insights-sessions` tab、`insights-row:{archive_id}` drill-in、100 条消息、`insights-exact-detail` 可见、`insights-next` 翻页、离页清除明文。
- 这两项依赖 `rows.rs` 与 `mod.rs:921-1024` 的通用行 id `insights-row:{key}`；产品面行也复用 `insights-row:{archive_id}`（`product.rs:322-325`）。

### 4.3 外壳

`ccr-desktop/tests/native_shell.rs`：`nav-agent-sessions` 标签“Agent sessions”、`page-agent-sessions` 可见、checked/focus 键盘导航（`:253-261`、`:576-580`、`:631-639`）；平台作用域 `tool-platform-sessions` 存在、`workbench-sessions` 不存在（`:900-912`）。

### 4.4 未被测试引用的产品面 id

`session-page`、`session-title`、`session-status`、`session-search`、`session-cards`、`session-transcript`、`session-trash-panel`、`session-operations` 只在源码中出现，无测试断言（rg 核对），可调整但建议保留 id 以便新增测试。

---

## 5. GPUI 重设计建议（仅基于现有数据；“需新数据/新状态”单独标注）

> 原则：沿用 `ccr-desktop/DESIGN.md` 的中性阅读面 + 语义色仅表状态 + 身份色只做小图标/圆点/细 tick；参考 Tauri 的信息分区，但不照搬它的“每项一张卡”。

### 5.1 页头（一行）

| 元素 | 内容 | 数据 |
|---|---|---|
| 左：图标方块 | 32–40px `PANEL_RADIUS` 方块，primary 10% 底 + `BookOpen`（与侧栏一致，默认包内） | 可用 |
| 标题 + 描述 | 保留 `session-title`、描述缩为一句 | 可用 |
| 状态徽章 | 替代当前 status_line：pill（`PILL_RADIUS`，elevated 底 + hairline 边）+ 图标（running=refresh-cw / finished=circle-check / failed=triangle-alert）+ “已索引 · 发现 X / 写入 Y”或“本页 N 条”；保持 `session-status` role=Status | 可用（`RefreshStatus` + `page.items.len()` + `next_cursor`） |
| 右：次操作 | ghost/secondary pill “CCR 废纸篓”（保留 id `session-open-trash`，点击后在页内展开面板或右侧详情区显示），仅在 Codex 相关时强调；可放到页头右侧，与主按钮同行 | 可用（入口与服务已有）；“只在 Codex 存在时显示”需要以 `statuses` 中 codex 的 availability 判定，可用 |
| 右：主操作 | `insights-index-refresh` 主按钮（running 时 busy 文案） | 可用 |

### 5.2 提供方状态条

- 形态：一行可换行的 chip 组（不是 8 张卡）；每个 chip：平台身份圆点或 16px 图标 + agent 显示名 + 状态文字 + 右侧 tabular `source_count`。
- 语义样式（底色用中性，状态色只在圆点/图标/文字上）：

| availability | 文案 | 样式 |
|---|---|---|
| Available | 可读 | success 墨色圆点 + 文字 secondary；计数显示 `source_count` |
| NoData | 无文件 | muted 空心/灰点，文字 muted，计数 `0` 可显示（这是实测 0 个源） |
| NotInstalled | 未安装 | muted 文字 + 虚线边（`border_dashed`）或更低强调；不显示计数 |
| Error | 不可读 | danger 墨色 triangle-alert/circle-alert 图标 + 文字；tooltip 显示 `error_category`（目前仅 `discovery_failed`） |

- fidelity（partial/locked）可作次级小字或 tooltip。数据可用（`AgentSessionProviderStatus` 全字段）。
- 可点击筛选（Tauri 同款 aria-pressed 多选）：后端 `SessionListRequest.agents: Option<Vec<_>>` 支持多选，GPUI 目前只用 `fields[7]` 单值——**需新状态**（selected agents 集合），无需新数据。平台作用域（`platform_scope`）下应锁定为对应 agent。
- 身份色：`platform_color` 缺 kimi / pi / omp——**需新 token**（可参考 Tauri `--color-platform-kimi-code-text #a3467f`、`--color-platform-pi-text #7b56b0`；omp 两端都无），并扩展 `theme.rs` 对比度测试。
- 图标：agent → 图标映射需新增（默认包可用：claude Bot / codex Cpu / grok Asterisk / antigravity Globe / opencode SquareTerminal；kimi / pi / omp 若用 Tauri 的 Moon / Bot / Workflow 需确认资源或改用圆点）。

### 5.3 工具栏

- 搜索框：`Input` 加 `prefix(Search 图标)` + placeholder“搜索标题、会话 ID 或路径”（与 SQL 语义一致，`usage_repo.rs:991-1000`）+ `cleanable`；保留 id `insights-filter-6`（焦点契约）。可去掉“Title”外置标签改用 aria_label；Enter 提交或订阅 `InputEvent::Change` 防抖即查（需新交互状态，无需新数据）。
- 可选筛选（后端已支持，`ccr-app/src/sessions/mod.rs:52-62`）：source_state（live/missing/deleted_by_user）、fidelity（full/partial/locked）用 `page_header::capsule` + `semantics::tab` 的分段胶囊；cwd 前缀、日期范围可后置。数据可用，需新 UI 状态。
- 结果计数放工具栏右侧：“本页 N 条 · 还有更早”（可用）。

### 5.4 行结构（uniform_list 定高）

建议 2 行、约 56–64px（仍定高），列表置于 12px 圆角实色面板内，行之间用 hairline 分隔而不是每行独立边框（DESIGN “数据列表不用 pill 包每一项”）：

| 位置 | 内容 | 数据 |
|---|---|---|
| 左侧 | 16px agent 图标或 8px 身份色圆点（或 2px 左 tick） | 可用（`row.platform`），颜色需补 kimi/pi/omp |
| 第 1 行 | 标题单行省略：父级 `flex_1 min_w_0`，标题 `text_ellipsis` + Tooltip（先例 `delivery.rs:278-284`）；缺标题回落 `session_id` | 可用 |
| 第 1 行右 | 时间 META_SIZE tabular、右对齐；绝对时间 `%m-%d %H:%M` 可用；相对时间（“3 分钟前”）只需当前时钟——可用，但需注意 `services.benchmark_clock()`（`mod.rs:112`）下测试确定性 | 可用 |
| 第 2 行 meta | agent 显示名 · `source_variant` · `N 条消息` · `N 次工具`（tool_use_count）用 muted 小字 + 中点 | 可用 |
| 第 2 行 路径 | cwd 用 mono META_SIZE、单行省略 + Tooltip；空显示“无项目路径” | 可用（不含物理源路径） |
| 状态 chip | 仅在非常态时显示：fidelity=partial（warning）、locked（warning + lock 文字）、source_state=Missing（muted/“来源缺失”）、DeletedByUser（muted/“已删除”）。full + Live 不显示 chip，避免满屏绿色 | 可用（`source_fidelity`、`source_state`） |
| 交互 | hover neutral、选中 `SELECTION_RADIUS 4` 中性实色；补 role=ListBoxOption + aria_selected + 键盘上下/Enter（Workbench 渲染器已有实现可参照 `mod.rs:936-960,995-1023`） | 可用 |

- “complete”文案应改为“完整记录/完整”之类的 fidelity 语义，不暗示会话已结束（GPUI 无会话进行中/结束数据）。
- 列表翻页：补“更早一页”按钮（`next()` 已支持 Sessions，`mod.rs:457-462`）——可用；若要 Tauri 式追加加载，**需新状态**（累积多页 items）。

### 5.5 列表 / 详情分栏（可选）

- Tauri 有分栏（`AgentSessionsView.tsx:350`），GPUI DESIGN 也规定列表/详情 3:2、窄宽 drill-in（`DESIGN.md:137`），`theme::SPLIT_WIDTH = 1200` 已有。
- 数据：转录数据可用（`SessionDetail` + messages 含 timestamp / tool_name / clipped / fidelity）。
- 阻碍：`InsightsView` 只有一个 `snapshot` 槽，Detail 会替换 Sessions（`mod.rs:87,383,473-479`）——**需新状态**（列表快照与详情快照并存、独立请求 gate），属于结构性改动；窄于断点时保持现有 drill-in 与 `insights-back`。
- 若暂不分栏，至少给转录页加头部：标题 + session_id（mono）+ “agent · variant” pill + partial/locked 警示条 + 消息卡显示 timestamp / tool_name / clipped 提示——全部可用。

### 5.6 状态面

- 未索引 / 空页 / 错误：改为虚线面板（`border_dashed`）+ 原因 + 下一步按钮（“刷新索引”）；可用。
- 加载：保留布局的骨架或行内 busy 文案；可用。

### 5.7 废纸篓

- 入口移到页头右侧次操作（见 5.1），面板在列表区上方以 12px 实色面板展开，含标题“CCR 废纸篓（Codex）”与关闭；保留 `session-open-trash`、`session-trash-prev/next`、`session-trash-page`、`session-restore:{i}`、`session-trash-row:*`、`session-restore-prepare-cancel` 等 id 与 `insights-filter-6` 焦点回归路径。数据可用。

---

## Caveats / Not Found

- “居中孤立链接”成因基于 flex 默认 stretch + ghost 按钮居中标签的推断，未做像素或布局树验证。
- Kit 默认图标包与全量包的差异通过 `default-icons.txt` 核对；Tauri 使用的 zap / sparkles / moon / workflow 是否在默认包内未逐一确认（仅确认 trash / messages-square / clock / lock / funnel 不在）。
- `Tag` / `Badge` 是否满足 GPUI DESIGN 对比度与焦点要求未验证；它们读取 Kit theme 的 success/danger 等色，需确认已映射到 `theme::palette`。
- 用户截图提到的提供方只有 4 个，但 `provider_statuses()` 对 8 个 agent 都会返回条目；截图可能只截了部分或换行。
- 相对时间在 benchmark/测试时钟下的表现未调查。
