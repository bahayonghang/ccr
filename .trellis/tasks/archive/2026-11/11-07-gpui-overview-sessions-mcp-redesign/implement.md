# 执行计划

分四个阶段，每个阶段单独提交、可单独回滚。阶段 2/3/4 依赖阶段 1 的共享层；2、3、4 之间互不依赖，但都要改 `ccr-desktop/DESIGN.md`，所以按顺序执行，不并行。

所有命令都在仓库根目录执行（`just` 会进入 `ccr-desktop/justfile`）。每个阶段结束的门禁：

```
just desktop-fmt-check && just desktop-lint && just desktop-test && just desktop-test-ui
```

阶段 4 结束后，再跑一次完整的 `just desktop-ci`，它包含 release 构建（上一轮出现过 test 全绿但 release 构建失败）。

## 0. 准备

- [x] `trellis-before-dev`：读 `implement.jsonl` 列出的 spec 和 research，以及 `ccr-desktop/DESIGN.md`、`theme.rs`、`shell/semantics.rs`、`shell/page_header.rs`。
- [x] 原生基线：release 构建后，截三页 light/en 在 2000×1080 和 1280×800 下的 before 图，存到 `research/native/before-*.png`（实际采用用户提供的三张截图作为 before）。如果截图自动化不稳定，只尝试一次，然后请用户提供截图（/ui 原生截图规则）。

## 1. 共享层（提交 1：`feat(GPUI): ✨ 新增状态标签与筛选 chip 共享组件`）

- [x] DESIGN.md：按 design §1.1 新增组件条目、修改行高规则、补充身份色范围；frontmatter 同步新增的 token 角色。
- [x] `theme.rs`：新增常量、`Tone`/`ToneColors`/`tone_colors`、`heat_level`；`platform_color` 补 kimi 和 pi；补对比度单测（Tone × light/dark、kimi/pi、heat 档位）。
- [x] 新增 `shell/badge.rs`（`tag`、`dot`、`hollow_dot`、`identity`、`filter_chip`、`search_field`、`empty_panel`、`section_heading`），并在 `shell/mod.rs` 中导出。
- [x] `overview.rs` 私有的 `status_dot` 改为调用 `badge::dot`，行为不变。
- [x] 单测：`filter_chip` 的 toggled/disabled 语义；`empty_panel` 的 action 可聚焦。
- [x] 门禁全绿后提交。

## 2. Overview（提交 2a：move-only 拆分；提交 2b：`feat(GPUI): ✨ 补全概览活动洞察并消除空白`）

- [x] 2a：把 `features/overview.rs` 拆到 `features/overview/` 下（design §2.2 列出的文件），只移动代码、不改行为。门禁全绿后提交 `refactor(GPUI): ♻️ 拆分概览模块`。
- [x] runtime：在 `WorkbenchReport` 中加入 `last_sync_at` 和 `activity`；`session_window` 改为 `activity_window`，归档不可用时用 `merge_home_insights(..., InsightsSessionCounts::default(), ..)` 降级；更新 `runtime/insights/tests.rs` 的两个用例，再新增一个“归档不可用仍有 activity”的用例。
- [x] 视图：
  - [x] 状态行显示新鲜度（`overview-last-sync`）。
  - [x] 计算 `rail_w`/`main_w`。
  - [x] 新建 `overview-main` 列，把 `overview-usage` 放进去，再加 activity。
  - [x] 右栏依次为 actions、week、data-source（`overview-insights` 改名为 `overview-data-source`）。
  - [x] 首屏之下加 distribution 和 leaders 两个模块。
  - [x] 逐日图标题补充 stale 提示。
  - [x] 窄宽时单列，padding 为 16。
- [x] 测试：按 design §2.4 更新、新增断言（等高、死区 ≤24、热力图格子尺寸、stale、空 daily、窄宽）；`tests/native_shell.rs` 的 overview 断言保持通过。
- [x] DESIGN.md 运行概览章节按 design §2.2/§2.3 重写（与代码同一个提交）。
- [x] 门禁全绿后提交 2b。
- 回滚点：revert 2b 回到拆分后的旧版式；2a 是纯移动，可以保留。

## 3. MCP Manager（提交 3：`feat(GPUI): ✨ 重设计 MCP 管理页布局与来源状态`）

- [x] 先写特征测试：锁住现有 pane 的优先级（batch > editor > source picker > presets > detail），以及全部掩码断言。
- [x] 外壳：`embeds_lens` 加入 McpManager；为 `McpView` 和 `Converter` 加 `embed_lens` 槽；Converter 标题加 heading 语义。
- [x] 页头与工具栏：标题加 LensBar；Reload、Add server 按钮；`mcp-filter` 改为 `badge::search_field` 并加 placeholder；平台筛选胶囊（带计数）；Presets、Batch & sync 按钮。删除全宽的 `mcp-batch-open` 行（id 移到工具栏按钮上）。
- [x] 来源健康条（design §3.3）：chip 的 aria label 用原句。
- [x] 引入 `Pane` 枚举，替换 render 中的 early return；实现宽窗分栏和窄窗 drill-in（`mcp-pane-back`）。
- [x] 列表行（design §3.4）与详情（design §3.5）；`visible()` 改为同时匹配名称和命令，并与平台筛选取交集。
- [x] 预设改为紧凑行；批量模式切换改用 `semantics::tab`；来源选择的项目根输入框补 placeholder 和可见标签。
- [x] 测试：更新 `mcp/tests.rs`、`batch.rs` 内的测试、`native_shell.rs`。新增：标题与 lens 同行；`mcp-filter` 有 placeholder；按命令筛选；平台计数；宽窗下 `mcp-list` 与 `mcp-pane` 并排；窄窗 drill-in 后能返回；`mcp-copy` 不被裁切。
- [x] DESIGN.md 新增 MCP 页面规则。
- [x] 门禁全绿后提交。
- 回滚点：revert 提交 3；共享层与 Overview 不受影响。

## 4. Agent sessions（提交 4：`feat(GPUI): ✨ 重设计 Agent 会话页筛选与分栏`）

- [x] 核对 `usage_repo` 接受的 fidelity 和 source_state 取值，写进 design §4.1 的注释，并据此确定常量（实际校验在 `ccr-app` 的 `sessions::list_sessions`，见 design §4.1）。
- [x] 状态：加入 `SessionDetailSlot`，以及 agents、fidelity、source_state 三个筛选状态；拆分 `select`/`submit`/`submit_detail`/`next` 的语义（design §4.1）；搜索 Enter 立即查询，输入后 300ms 防抖查询。
- [x] 把 `product.rs` 拆到 `features/insights/sessions/{mod.rs, header.rs, providers.rs, list.rs, pane.rs}`。和提交 2a 一样，先做一个 move-only 提交再改代码。
- [x] 页头、提供方 chip、工具栏、列表行（带 listbox 键盘操作）、翻页条、详情 pane、废纸篓位置、空态（design §4.2）。
- [x] 文案：fidelity 改为 “Full record”；fidelity 和 source_state 筛选项的中英文文案。
- [x] 测试：
  - [x] 保留：`insights-filter-6` 的 Esc 焦点回归；废纸篓分页；Workbench 渲染器的两个测试。
  - [x] 新增：
    - chip 切换后请求中的 agents 正确，且可多选；
    - NotInstalled 与 Error 状态的 chip 不可选；
    - 平台作用域下锁定；
    - 宽窗选中会话后列表仍在；
    - 快速连续选择两行时，只显示后一个的详情；
    - 列表刷新会丢弃在途的详情请求；
    - 窄窗进入 drill-in 后可以返回；
    - 长标题单行（标题元素宽度不超过行宽）；
    - 有/无 `next_cursor` 时翻页条分别显示和隐藏。
- [x] DESIGN.md 新增 Agent sessions 页面规则。
- [x] 门禁全绿后提交。
- 回滚点：revert 提交 4。如果双快照出问题，可以只回退 `SessionDetailSlot`，保留视觉改动，详情退回 drill-in。

## 5. 质量门与原生复核

- [x] `trellis-check` 全范围检查，范围是 diff 加上 `implement.jsonl` 中的合同。（已完成，修复提交 911e9d7b；遗留建议见 `research/visual-review.md` 与根目录 handoff）
- [x] `just desktop-ci` 全绿（包含 release 构建），收据写入 `research/integration.md`。
- [ ] 原生 after 截图，与 before 并排比较：
  - 三页 × light/dark × 2000×1080 和 1280×800；
  - 加上 960×640 最小窗口和中文各一张。
  - 测量并记录：Overview 图表面板内的空白（≤24px）、首屏底部的空白（≤24px）。
  - 结论写入 `research/native-visual.md`，未运行的项目逐条列出，包括 DPI、macOS/Linux、读屏和 IME。
- [ ] 按 design-atlas `anti-slop.md` §6 的人工判定清单逐条走查，结论写入 `research/native-visual.md`（`slop_check.mjs` 不适用于 Rust GPUI，写明原因）。
- [x] 让一个全新上下文的子代理做视觉复核，输入只有截图和 DESIGN.md，不给出本文的结论（`review-prompt.md`）。（已完成，分诊见 `research/visual-review.md`；修复轮未完成）
- [x] Spec 更新：`theme-token-contracts.md` 的 GPUI 小节加入 Tone 与 heat 的对比度规则；`dashboard-presentation-contracts.md` 加入 GPUI 新鲜度和口径标注规则；`agent-session-observability-contracts.md` 加入 GPUI 提供方 chip 的状态映射。

## 续作结果

- [x] handoff §5.1 A–F：全部实现，设计/规格同步，几何与失败读取回归通过；提交 `a145abba`。
- [x] 第二轮源码复核及全新截图复核已记录；后者只覆盖 light/en 非受控宽窗。
- [x] `research/integration.md`、`research/native-visual.md` 已补充真实门禁、已运行/未运行项目和剩余缺口。
- [ ] 完成原生尺寸/主题矩阵、校准 Overview 空白测量及中文/最小窗复核后再归档。窗口输入/前台变化使本轮自动化不稳定，未用旧截图代替新验收。

状态仍为 `in_progress`。这是实现检查点，不是完整原生验收完成。

## User-requested archive — 2026-11-09

The user explicitly requested committing all changes and archiving this task. The earlier in-progress/do-not-archive notes are historical and superseded by that instruction. Implementation and the recorded desktop gates passed. The unchecked native theme/size/Chinese matrix, calibrated gap measurements and other native acceptance remain **UNVERIFIED**; archival does not turn them into passes. Preserve the unchecked acceptance items and remaining gaps for future work. No push.
