# S4 逐页源码巡检（2026-11-07）

本记录是源码层检查，不是原生像素验收。当前 session 继续既有视觉任务；矩阵与原生截图未运行，不以 headless 测试替代。

## 变更边界

最小差距是共享工具行选中态不可见、非 Claude 工作台用空格串排统计、图表复用动作绿，以及 Claude 工作台面板沿用输入圆角。拥有者分别是 feature 的工具行构造、`WorkbenchView::usage`、Claude trend 的 paint 和 panel 构造。只修改这些呈现入口；业务请求、数据库投影、凭据处理、确认流程、虚拟化与 stable ID 保持现有协议。

## 逐页检查

| 页面 | 当前源码事实与本次处理 | 原生验证 |
| --- | --- | --- |
| Claude Code | `src/features/workbench/claude.rs:17,237` 的三个费用段和本地状态面板改用 12px panel token；`workbench/claude_trend.rs:121,178` 的曲线改用 Claude 身份色、面板 12px。保留刚完成的 30 天费用图、日历空隙、费用口径、独立逐日报告和订阅入口；未按旧 audit 重做页面。 | not_run |
| Codex | `workbench.rs:387` 的 30 天总量替换为空间独立的 label/value 行，值右对齐等宽；`:463` 每日条使用 Codex 身份色和中性轨道。滚动用量与已存在的报告边界保留。 | not_run |
| Grok | 复用同一非 Claude totals/bar 呈现入口，使用 Grok 身份色；Grok 独立读数、部分完成结果与入口未改。 | not_run |
| OpenCode / Antigravity | 复用同一非 Claude totals/bar 呈现入口与对应身份色；只保留真实能力入口，无新增虚假 Agents/Plugins CRUD。 | not_run |
| Insights / Usage | `insights/mod.rs::product_tabs` / `usage_sections`：产品工具行、逐日报告分区行均加入有边界容器，通过共享 `semantics::tab` 呈现选中面/文字/语义。构造单独置于私有 helper，保留 1 MiB UI 栈约束；loading 禁用、字段筛选、数据精度、virtualized rows 不变。 | not_run |
| Settings | 实现在 `shell/settings.rs`；共享 agent 将面板 loop 的 radius 改为 panel token。现有原生导航 control 已有独立 selected/hover/focus 色，未额外重构。 | not_run |
| Commands | `jobs/mod.rs:331,345`：命令目录加入容器与持久 selected/checked；现有 disabled/typed-only admission 保留。 | not_run |
| MCP | `mcp/mod.rs:627,663`：目标来源工具行加入容器并用共享 tab decorator，保留 stable source IDs 与 busy 禁用；列表/批量选项的现有 ghost selected 由共享 `secondary_active = overlay` 修复。未改源文件和 secret patch 行为。 | not_run |
| Check-ins | `checkin/mod.rs:256,364,386`：保留本地读取/无账户原因、选中账户和明细。选择是列表语义，现有 selected 通过共享 ghost active 角色获得不同于纸面的背景；未将列表项改成 segmented 控件。 | not_run |
| Sync | `sync/mod.rs:425,480,514,547`：保留独立状态/配置不存在原因和 Select/Selected 文案，不把未测试的远端显示成已连接；共享按钮/边界修复生效，不改传输与敏感操作。 | not_run |

## 回归检查安排

- Jobs 既有真实控件测试新增 status/list 的 `checked` 切换，并继续断言 typed-only delete 不可切换或执行。
- Insights 既有真实控件测试新增 Daily/Models 的 `checked` 切换，并继续检查 exact-value detail 与数据不变。
- MCP 既有跨来源 preset/copy 控件测试新增 user/codex 的 `checked` 状态，并继续验证凭据保留。
- 本子任务仅运行指定 Rust 文件的 `rustfmt --edition 2024`，未并发运行 Cargo；整合 `desktop-ci` 和窄回归结果由主会话另记。

## 验收边界

四页 × 两主题 × 两语言 × 两尺寸截图矩阵没有采集。DPI/跨显示器、macOS/Linux 均维持 user-deferred / not_run。源码与自动化通过不意味着原生视觉已经通过；S4 的源码巡检可以完成，截图 AC 必须保持开放。
