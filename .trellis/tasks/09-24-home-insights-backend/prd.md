# Insights 投影与 typed 命令（后端）

父任务：`09-24-home-insights-redesign`（决策 D1-D3 与现状事实见父任务 `prd.md`）。

## Goal

为首页 Insights 区块提供一次调用即可取得的统计快照：

- `crates/ccr-usage` 新增 `Dashboard::insights`，对 `usage_bucket_30m` 做有界扫描，在 Rust 侧派生日、小时、星期、月、周、来源、模型、项目维度。
- `ccr-ui/src-tauri` 新增 typed 命令 `get_home_insights`，合并会话归档计数，复用 `usage:snapshot:` 缓存与 single-flight 填充。
- 生成 ts-rs 绑定、generated client、命令清单。

## Requirements

- R1 快照字段（全部为请求数/会话数/token 三种口径，不含 prompts）：
  - 总量：`first_date`（最早有用量的本地日）、`requests`、`tokens`、`sessions`、`agents`（有用量的来源数）、`projects`（非空 `project_hash` 去重数）、`active_days`（请求数 > 0 的本地日数）。
  - 最近 7 天与其前 7 天：各自的 `requests`、`sessions`、`active_days`。7 天窗 = 以 `as_of` 收尾的 7 个本地日闭区间。
  - 活动日谱：53 周窗（52 整周 + `as_of` 所在周，周一为周起点）内每个有请求的本地日 `(date, requests)`，升序。
  - 连续活跃天数：`current_streak`（`as_of` 当天无请求时从前一天起算）、`longest_streak`（全历史）、`busiest_day`（全历史请求数最高的日及其请求数）。
  - 分布：`hourly[24]`、`weekday[7]`（周一起始）、`monthly[12]`，按请求数，全历史。
  - 周趋势：每个来源在 53 周窗内的每周请求数，数组长度 53，末项为 `as_of` 所在周；按总量降序。
  - 榜单：`agents`（来源：requests、tokens、sessions）、`projects`（`project_label` 展示、`project_hash` 分组：requests、tokens）、`models`（requests、tokens）。全量返回，不截断，前端按当前度量排序取前 N。
- R2 来源键：用 llmusage canonical 键（`SourceKind::as_str`），`gemini` 旧键并入 `antigravity`。9 个来源全部参与，不硬编码子集。
- R3 会话计数来自 `usage_session_archive`；平台名经 `SourceKind::parse_id` 映射到来源键。映射失败的平台计入 `sessions` 总数，并在 `agents` 榜单中以原平台名单独成行（`unmapped = true`）。`omp` 与 `kimi` 两个归档平台名必须映射到 `pi` 与 `kimi_code`。
- R3a 会话诚实状态：DTO 携带 `sessions_indexed: bool`（`has_any_session_archive` 的结果）。该值为 `false` 时，前端按 `.trellis/spec/ccr-ui/frontend/dashboard-presentation-contracts.md` 的 Sessions honesty 规则显示 `–`，不显示 0。本命令不触发会话索引 job。
- R4 时区：沿用 `Dashboard` 的报告时区（`QueryFilter.timezone`，默认值与现有首页一致）。本地日与小时在 Rust 侧由 `hour_start`（UTC RFC3339）换算。`as_of` 由命令层传入，服务层与测试不读系统时钟。
- R5 晚于 `as_of` 的桶（时钟漂移的脏数据）不进入任何按日/小时/周分桶，但计入 `requests`/`tokens` 总数。
- R6 能力门控：新增 `FeatureKey::Insights`，声明所需列；schema 不满足时返回现有 `UsageError` 的不支持错误，命令层映射为可显示的错误字符串。
- R7 命令 `get_home_insights()` 无参数，返回 `HomeInsightsResponse`；缓存键 `usage:snapshot:home_insights:<as_of>`，TTL 30 秒；导入完成后随前缀整体失效（现有机制）。
- R8 性能：扫描限定在 `usage_bucket_30m` 的 53 周窗（周趋势与日谱）加一次全历史聚合（总量、分布、榜单）。命令整体在 `spawn_blocking` 中执行。

## Acceptance Criteria

- AC1 `crates/ccr-usage` 单元测试覆盖：空库全零；固定偏移时区（含 +05:30）的本地日/小时换算；周一周起点与 53 周下标；`as_of` 当天无请求时 streak 从前一天起算；未来日期不进分桶但计入总数；`gemini` 并入 `antigravity`；9 个来源都能出现在周趋势中。
- AC2 `ccr-ui/src-tauri` 服务层测试覆盖：会话平台映射（`omp`→`pi`、`kimi`→`kimi_code`、未知平台单独成行且计入总数）；最近 7 天会话数包含 `as_of` 当天创建的会话。
- AC3 所有 `i64`/`u64` 字段带 `#[ts(as = "f64")]`；`just tauri-bindings-check` 与 `just tauri-command-inventory` 校验通过，生成文件已提交。
- AC4 `commands/handler_registry.rs` 登记新命令，注册表计数测试只因新增 1 个命令而变化。
- AC5 `just lint-strict`、`just test`、`bun run tauri:test`、`bun run tauri:clippy` 通过。

## Out of Scope

- 前端组件与查询 hook（frontend 子任务）。
- 修改 `get_home_usage_overview_v2` 的 4 平台 series 或首页"会话 0"问题。
- 数据库迁移、prompts 口径、后台刷新 job。
