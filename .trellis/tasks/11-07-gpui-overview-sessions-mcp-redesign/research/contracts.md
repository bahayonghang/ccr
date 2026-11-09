# 保留合同摘要（原文摘自三份调研，供子代理注入；完整上下文见原文件）

## Overview（摘自 research/overview.md §4、§6）


来源：`ccr-desktop/DESIGN.md`、`.trellis/spec/ccr-ui/frontend/dashboard-presentation-contracts.md`、`.trellis/tasks/archive/2026-11/11-07-gpui-airy-glass-material/{prd.md,design.md}`。

1. **不铺零 / 不编数**：无记录给原因、库不存在或查询失败不铺零；会话未索引写“未索引”（DESIGN.md:124、:130、:136、:217、:245；归档 prd R4 平台带）。spec：sessions honesty 场景、平台 sparkline “all-zero 不等于 untracked”，`overview == null` 时不造零数组。
2. **派生值必须标注**：日均 = 总量 ÷ 窗口天数，必须标“日均 / avg”（DESIGN.md:135；`overview.rs:424-431`）。
3. **KPI 带**：单面板 4 栏 hairline，Cost 32/600 为每屏唯一主读数，其余 24/500；<1040 改 2×2（DESIGN.md:120、:135）。
4. **平台带**：只有有逐日记录的平台单独成格，其余合并“其他来源”，窄宽横向滚动不换行（DESIGN.md:136；`overview.rs:575-607`）。
5. **图表**：高度 160–240px（Overview 逐日图 216px）；自动收紧到首个有数据日前一天并标 “since MM-DD”；全窗口无记录保留完整虚线基线；最上段 2px 圆角、段间 1px；系列色 `theme::chart_series`；数据更新无入场动画（DESIGN.md:243）。**若要让图表随面板拉伸超过 240px，需要先改 DESIGN.md。**
6. **精确值表焦点**：默认收起；最新在上；连续全零日折叠为一行且可聚焦；第 i 个显示行对应表格第 i+1 个子项；Tab/Shift+Tab 同时 reveal 表格与外层页面（DESIGN.md:244；`overview.rs:273-297`、:1309-1337；单测 :1588-1607、:1760-1800 区间）。
7. **右栏**：下一步 + “数据源”小节，数据来自视图已有状态，不新增查询；两列等高，不留死区；窄宽右栏堆到主列下方，不横向滚动整页（DESIGN.md:137；归档 design.md §4 “右栏…不新增查询”；归档 prd R4 “不能留死区，也不能编造数字”）。
8. **页头**：唯一 h1 + 内嵌 lens 胶囊同一行；范围胶囊作用整页（DESIGN.md:134；`tests/native_shell.rs:190-205`）。
9. **材质**：阅读面（表格、列表）一律实色，不叠玻璃（DESIGN.md Elevation 1-3 条）。
10. **数据层边界**：归档任务约束“Overview 只能使用已有 report / services 数据，不新增伪造统计”；用量 SQL 只在 `crates/ccr-usage`（AGENTS.md）。
11. **演示数据标识**：fixture 带“Isolated demo / 隔离演示”（`overview.rs:377-384`、:911-917）。
12. **先改 DESIGN.md，再改 theme.rs，最后改视图**（归档 prd 约束节）。

---


### 6.1 `ccr-desktop/src/features/overview.rs` 内单测

- `exact_rows_are_newest_first_and_fold_every_quiet_run`（:1588-1607），`chart_trims_to_one_day_before_the_first_record`（:1609-1619）：纯函数测试，不受版式影响。
- `overview_ranges_hide_stale_totals_and_preserve_exact_metrics`（:1621 起，1280×800 窗口，:1643-1646）：手工构造 `WorkbenchReport`（:1659-1680 等），**给 `WorkbenchReport` 加字段时每个字面量都要补**。依赖的 id / 断言：
  - `overview-total-tokens / -requests / -sessions / -cost`、`overview-codex-requests`、`overview-grok-requests`（不存在）、`overview-other-grok`、`overview-other-pi`
  - `overview-exact-toggle`、`overview-exact-table`、`overview-exact-{MM-DD}`、`overview-exact-quiet-{MM-DD}`（焦点顺序 :1760-1800、:1888-1940 区间）
  - `overview-root` bounds（:1787，精确值行 reveal 验证）
  - `overview-heading`、`overview-window-7/30/90`、`overview-refresh`（同行断言 :1849-1858）
  - **`overview-rail.bottom == overview-usage.bottom`（:1862-1865）** —— 主列改为多面板容器时，此断言的目标 id 需要随之调整
  - `overview-insights` 可见（:1866）
  - `overview-chart-since`（:1883）、`overview-day-10-31` 不存在 / `overview-day-11-01` 存在（:1886-1887）
  - `overview-open-usage / -sessions / -commands / -install` → `take_pending`（:1974-1981 区间）
- `ccr-desktop/tests/native_shell.rs`：
  - `f6_tab_navigation_does_not_intercept_search_editing`（:157）内 `overview-heading` 与 `tool-overview` 同行（:190-205）
  - `overview_states_missing_usage_without_sample_numbers`（:442-501）读 `overview-status` 文案（:469-475），断言 “No local usage source”
- `ccr-desktop/src/runtime/insights/tests.rs`：`workbench_reads_seeded_usage_and_does_not_invent_sessions`（:293）、`workbench_folds_stored_source_aliases_without_inventing_daily_bars`（:352）—— `WorkbenchReport` 增字段时需更新
- `ccr-desktop/src/runtime/benchmark/tests.rs:843`：使用 `InsightsQuery::Home`，改动 Home 读取路径时相关
- `ccr-desktop/tests/test_commands.py`：未引用 overview id

### 6.2 验证命令

- `just desktop-test`（`ccr-desktop/justfile:49-53`）、`just desktop-test-ui`（:56-58，`--test native_ui --test native_shell`）、`just desktop-lint`、`just desktop-fmt-check`。

---

## MCP Manager（摘自 research/mcp-manager.md §4）


### 4.1 `ccr-desktop/src/features/mcp/tests.rs`

| 测试 | 断言要点（行号） |
|---|---|
| `mcp_page_lists_real_entries_and_hides_secret_fields` | 等待 `mcp-entry-4`（:124-127）；`page-mcp-manager`、`tool-converter` 可见，`nav-converter` 不存在（:130-132）；label 中含 5 个名称、`disabled in the file`（:134-139）；**来源整句** `Claude state file: 2 entries.`、`Claude settings.json: 1 entry.`、`Codex config.toml: 1 entry.`、`Antigravity mcp_config.json: 1 entry.`、`Claude project .mcp.json is not read`、`OpenCode MCP is not read`、`Grok MCP credentials are not listed as servers`（:140-146）；不含 `mcp-secret-sentinel`、`Home insights`、`Apply / refresh`、`no MCP manager connection`（:147-150）；点 `mcp-entry-1` 详情含 `Command: fixture-mcp-command`、`Arguments are set in this entry.`，不含 URL（:151-157）；`mcp-entry-0` 含 `fixture-project`、`A URL is set in this entry.`，不含 `https://`/`example.test`（:158-165）；`mcp-entry-2` 含 `disabled is true in the file.`，**不含 `TOKEN`、`apiKey`**（:166-172）；`mcp-entry-3` 含 `enabled is false in the file.`，不含 `bearer`（:173-179）；`mcp-entry-4` 含 `Command: agy`，不含 `Authorization`（:180-185）；`mcp-filter` 输入后 `mcp-entry-0` label 含 `codex-server`、`mcp-entry-1` 与 `mcp-detail` 消失（:186-202） |
| `mcp_page_with_no_files_stays_empty` | `mcp-empty` 文案 `No MCP entries were found in the files this page reads.`；`Claude state file: not found.` 等（:227-230）；无 `mcp-entry-0`；不创建 fixture 目录（:234） |
| `preset_and_cross_source_copy_use_real_controls_and_keep_stored_credentials_hidden` | 点击序列 `mcp-presets` → `mcp-preset-fetch` → `mcp-source-codex` → `mcp-open-editor`（:262-268）；`mcp-source-codex` checked（:266-267）；`mcp-copy` 按钮不得被 `mcp-manager` 视口裁切（:290-295）；复制后不显示 `copy-secret-sentinel`、不存在 “MCP JSON patch” 节点（:303-313） |
| `mcp_native_actions_write_confirm_and_keep_conflicting_drafts` | `mcp-add` → `mcp-open-editor`（:359-360）；编辑时补丁初值必须为 `{}`（:408-412，“existing secrets must not populate patch controls”）；冲突提示 “The source changed.”（:421）；`mcp-edit`、`mcp-delete`、`mcp-editor-close` 等 id |
| `batch.rs` 内 `rendered_batch_review_focus_escape_confirm_and_navigation_keep_safe_receipt` | 直接用 `McpView::new` 窗口（`batch.rs:594-600`），点 `mcp-batch-open` → `mcp-batch-mode-sync` → `mcp-batch-review`；焦点在 `mcp-batch-back`，Tab/Shift-Tab 循环、Esc 返回 `mcp-batch-review` 聚焦（:617-636）；`suspend` 后 `batch` 为 None、`last_batch` 保留（:658-660）；快照不含凭据（:691） |
| `editor/form/tests.rs`（13.1K，未逐行展开） | 表单字段 id/掩码行为 |

**条目索引依赖排序**：测试中 `mcp-entry-0` = project-server（ClaudeState Project）、`-1` state-server、`-2` settings-server、`-3` codex-server、`-4` agy-server（由 `runtime/mcp.rs:436-441` 排序决定）。若改为分组/分区渲染或改变 id 分配，需同步改测试。

### 4.2 `ccr-desktop/tests/native_shell.rs`

- `nav-mcp-manager` → `tool-converter` → `converter-acknowledge`，离页后输入/输出清空（:300-310）。
- 导航表 `("nav-mcp-manager", "page-mcp-manager", "MCP Manager")`（:582）；焦点/可见性（:638-641）。
- 平台工作台 `tool-platform-mcp`、`workbench-mcp` 存在于 Claude、不存在于 Codex（:893、:906、:956-957）。

### 4.3 运行时测试

- `runtime/mcp.rs:585-760` 系列：`format!("{catalog:?}")` 不含 `mcp-secret-sentinel`（:614、:684、:729）；非法结构时连命令都不采用（:730）。

### 4.4 掩码/安全契约清单（重设计不得破坏）

1. 读取器只复制 name / scope / command / has_url / has_args / disabled / enabled；env、headers、args、URL、token 不出读取器（`runtime/mcp.rs:1-3`、:53-62）。
2. 列表、详情、a11y label 中不得出现 URL 值、args 值、env/header **键名与值**（测试断言不含 `TOKEN`、`Authorization`、`bearer`、`apiKey`：tests.rs:170-171、:178、:184）。⇒ 显示 env 键名会直接违反现有断言；env 数量目前也没有数据。
3. 编辑器不预填已有密钥：补丁初值 `{}`（tests.rs:408-412；`editor.rs:106-109`）；URL、env/header 值输入框 `masked(true)`（`form.rs:36`、:64、:227）；批量凭据输入 masked 且保存后清空（`batch.rs:130`、:267-268）。
4. 跨源复制在后端不透明会话中保留明文，UI 草稿与操作快照不含凭据（`.trellis/spec/ccr-app/backend/shared-desktop-services.md:51-53`；tests.rs:303-313；batch.rs:691）。
5. 写入前确认 + 原始版本冲突检测，草稿保留（spec :79；tests.rs:421-428）。
6. 本页不启动服务器、不推断进程状态（`mod.rs:1`、:535）——**不得新增“Running/Connected”类徽章**。
7. Converter：需确认后才打开、离页清空（`converter.rs:53-62`、:106-108；native_shell.rs:300-310）。

---

## Agent sessions（摘自 research/agent-sessions.md §4）


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
