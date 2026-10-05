---
skill: trellis-plan-review
version: 0.5.0
task_dir: D:/Documents/Code/Github/ccr/.trellis/tasks/09-08-codex-auth-tui-presentation
task_name: 09-08-codex-auth-tui-presentation
task_status: planning
review_scope: single-task
task_count: 1
task_members:
  - 09-08-codex-auth-tui-presentation
task_statuses:
  09-08-codex-auth-tui-presentation: planning
verdict: 可执行
blocking: 0
should_fix: 0
notes: 0
generated_at: 2026-09-08T11:31:25.2498656+08:00
---

# Trellis 规划审阅报告

## 审阅范围

- 根任务：09-08-codex-auth-tui-presentation
- 模式：single-task
- 任务数量：1
- 有序成员（根优先；顺序不代表依赖）：
  - 09-08-codex-auth-tui-presentation — planning

## 结论

可执行 — 阻断 0 / 应修 0 / 提示 0

此结论只表示当前规划产物通过本轮只读审阅；任务仍为 `planning`，不构成开始实现、提交或外部写入的授权。

## 问题清单

无。

## 未能核实

- 80×24、100×22、100×30、120×22、140×40、180×50 及 60×18 的最终 Ratatui buffer 布局、颜色和边框关系 — 本轮按规划边界未实现、未运行产品测试；须由实施阶段的 TestBackend fixture 验证。
- 真实 Windows 终端中的字体、CJK 显示宽度、颜色观感与长错误截断 — 未获桌面操作授权，保持 UNVERIFIED。
- 用户截图所对应二进制与当前 `dev` 源码完全一致，以及截图中私人配额和本地统计的真实性 — 未读取个人认证文件、usage 日志或私有数据库，保持 UNVERIFIED。
- 截图中 5h 重置时间异常是否来自真实服务端窗口映射 — 未调用配额 API；本任务也明确不修改 primary/secondary 映射，保持 UNVERIFIED。

## 可靠部分

- Pass 0 复核为单任务、`planning`、复杂任务产物齐全；13 个 `path:line` 引用全部解析，5 条需求和 5 条 AC 均完成标注，无 `_example`、占位符或缺失上下文文件。报告路径为 ignored。
- 截图布局断言与源码一致：`crates/ccr-tui/src/tui/codex_auth/ui.rs:1329` 的 Wide 分支当前为 60/40，右侧上段固定 13 行；旧用量面板在 `ui.rs:971` 已有配额颜色条，而 Focus 摘要从 `ui.rs:1146` 构建。
- 配额语义与状态边界成立：`crates/ccr-codex/src/models/codex_auth.rs:778` 将 5h/7d 数值定义为剩余百分比，并提供两个 `window_present`；`crates/ccr-tui/src/tui/codex_auth/app.rs:368` 的 `selected_quota()` 已先读 preview cache，当前缺失来自 `ui.rs:911` 先匹配 Idle。D1 为缓存、刷新、错误及窗口存在三态分别给出机制，且不改 service 映射。
- 右下黄色说明的根因成立：`crates/ccr-tui/src/tui/codex_auth/app.rs:624` 已处于 `AccountAttributed`，仅因全局请求数大于归属记录数而生成说明；`crates/ccr-tui/src/tui/codex_auth/ui.rs:1074` 将任何 `fallback_reason` 统一渲染为 warning。D2 使用既有 `attribution_state` 区分中性范围、全局回退与真实错误，不改变 `records_for_account` 或统计口径。
- 两个 CLI 入口的路径已按实码修正：`crates/ccr-tui/src/tui/codex_auth/mod.rs:11` 委托 `crates/ccr-tui/src/tui/mod.rs:163` 的主 App，生产渲染进入 `crates/ccr-tui/src/tui/ui.rs:105` 的 `draw_embedded`。计划不再把未处于真实入口路径的旧 `codex_auth::ui::draw` 当作验收替身。
- 布局算术重新计算一致：80×24 扣除 header 3、compact footer 2、runtime banner 3 后为 16 行；100×30 扣除 3、3、4 后为 20 行；100×22 和 120×22 均只剩 12 行。D3 已针对实际 `content_area.height` 给出 3+9 与 Wide 合并卡降级，并在 AC3 矩阵加入两个高度断点。
- AC1–AC5 的每个可观察子句均能追溯到 R1–R5、D1–D5 和 `implement.md` 的 fixture/命令：窗口缺失不伪装百分比、正常归因说明不冒充警告、全局回退范围随数字出现、低高度不留下无范围数字、数值/CJK 对齐及真实错误优先级都有明确机制。
- 实施范围保持最小：生产代码限定 `codex_auth/{app,ui}.rs`，主 `tui/ui.rs` 只补组合渲染测试，稳定契约回写 ccr-tui spec；不新增依赖、SQL、缓存、持久化字段或账本回填。验证命令已包含 ccr-tui、ccr、ccr-usage、格式与严格 lint，符合当前 TUI spec。

## 盲区

An agent reviewing an agent's plan is not an independent second opinion. The reviewer and the
author share most of the same blind spots. A clean report means "this pass found nothing", not
"the plan is complete". Treat the findings as a triage list, not as an approval.
