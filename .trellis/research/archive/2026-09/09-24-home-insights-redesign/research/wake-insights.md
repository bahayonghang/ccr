# Wake Insights 调研笔记

来源：`ref/repo/Wake`（v0.8.1，提交 `269c50b`，只读参考）。Wake 是 Rust + GPUI 桌面应用，自带 20 家 agent 的会话适配器与 SQLite FTS 索引。

## 数据模型

`InsightsData`（`crates/wake-core/src/models.rs:466-537`）一次查询算好，渲染层不再读时钟：

- `as_of`：快照的"今天"，streak 与热力图末列共用。
- 总量：`sessions`、`prompts`、`tokens`、`project_count`、`first_ts`。
- `daily`：全时段活跃日谱 `(日, prompts)` 升序；热力图、streak、最忙日都由它派生。
- `hourly[24]`、`weekday[7]`（周一起始）、`monthly[12]`（跨年叠加）。
- `agents`/`projects`/`models`：`UsageTally { name, sessions, prompts, tokens }`，全量返回，UI 按当前度量排序后取前 N。
- `daily_sessions`：按会话创建日的会话数，用于 Last 7 days 对比。
- `trend_agents`：各 agent 近 53 周每周 prompts，按总量降序。

## 规则

- 周起点为周一；`TREND_WEEKS = 53`（52 整周 + 本周）；`trend_start = week_start(as_of) - 52 周`。
- Last 7 days 窗 = `[as_of - 6, as_of]`；对比窗为其前 7 天。
- current streak：今天无活动时从昨天起算（GitHub 惯例）。
- 未来日期的数据不进任何分桶，但计入总数。
- 项目总数与项目榜使用同一谓词（排除空路径）。
- 模型不出周趋势：会话级 model 是末态，按它切周会把历史归给最后使用的模型。
- tokens 不进按日窗口：会话 token 是终身累计量，没有时间维度。

## 执行方式

`Store::insights`（`crates/wake-core/src/db.rs:2210-2440`）：

- 私有只读连接，不与 UI 读连接抢锁。
- 第一遍按 (日, 时) 分组，weekday/monthly 在 Rust 侧由日期派生。
- 第二遍按 (agent, 项目, 模型, 日) 分组，拆出三张榜单的 prompts 与 agent 周桶。
- 榜单主体只查 sessions 表，prompts 由上一步回填。
- 在后台任务中执行，UI 线程不等待。

## 对 ccr 的差异

- ccr 不做 prompts 口径（父任务 D2）；请求数来自 llmusage `event_count`，天然带 30 分钟时间桶，可以做按日窗口与按小时分布。
- ccr 的 token 来自 llmusage 桶，带时间维度，因此 token 可以按窗口统计（与 Wake 的限制不同）。
- ccr 不做后台快照 job（父任务 D3），改用 30 秒 TTL 缓存与 single-flight。
