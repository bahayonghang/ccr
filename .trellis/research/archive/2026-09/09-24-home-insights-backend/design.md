# Design: Insights 投影与 typed 命令

## 1. 分层与所有权

| 层                   | 文件                                                                                                                                                                                                                                            | 职责                                                                                    |
| -------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| 投影（SQL + 纯聚合） | `crates/ccr-usage/src/insights.rs`（新）、`crates/ccr-usage/src/db.rs`（加 `Dashboard::insights` 入口）、`crates/ccr-usage/src/capabilities.rs`（加 `FeatureKey::Insights`）、`crates/ccr-usage/src/timezone.rs`（加 `ResolvedZone::local_at`） | 全部 usage SQL 留在 `ccr-usage`（仓库规则：`crates/ccr-usage` 是 usage SQL 唯一所有者） |
| 服务（State-free）   | `ccr-ui/src-tauri/src/services/usage.rs` 或同目录新文件 `services/home_insights.rs`                                                                                                                                                             | 合并会话归档计数、平台键映射、组装 wire DTO                                             |
| 命令                 | `ccr-ui/src-tauri/src/commands/usage.rs`、`commands/handler_registry.rs`                                                                                                                                                                        | State 提取、`spawn_blocking`、缓存、计时                                                |
| 绑定                 | `ccr-ui/src/types/generated/usage/*`、`ccr-ui/src/api/generated/usageV2.ts`、命令清单文档                                                                                                                                                       | 生成物，不手改                                                                          |

`llmusage_adapter/` 不新增 SQL（仓库规则）。

## 2. 投影：一次取行，Rust 侧派生

参考 Wake `Store::insights`（`ref/repo/Wake/crates/wake-core/src/db.rs:2210-2440`）：SQL 只做粗分组，所有时间维度在 Rust 侧派生，避免每个维度各扫一遍，也避免在 SQL 中做 IANA 小时换算。

查询 Q1（时间维度，全历史）：

```sql
SELECT hour_start,
       CASE WHEN source = 'gemini' THEN 'antigravity' ELSE source END AS src,
       SUM(event_count), SUM(total_tokens)
FROM usage_bucket_30m
GROUP BY hour_start, src
```

行数上界 = 30 分钟桶数 × 来源数。一年约 17,520 个桶；实际只有有用量的桶才有行。结果在 Rust 中逐行处理：

- 用 `ResolvedZone::local_at(DateTime<Utc>) -> NaiveDateTime`（新增，与 `date_at` 同实现方式）换算本地日与小时。
- 累加总量、`first_date`、日谱（`BTreeMap<NaiveDate, i64>`）、`hourly`/`weekday`/`monthly`、来源周桶（53 周窗内）、来源总量。
- 日期晚于 `as_of` 的行只计入总量（R5）。

查询 Q2（模型榜）与 Q3（项目榜）：

```sql
SELECT model, SUM(event_count), SUM(total_tokens) FROM usage_bucket_30m
WHERE model != '' GROUP BY model
SELECT project_hash, MAX(project_label), SUM(event_count), SUM(total_tokens) FROM usage_bucket_30m
WHERE project_hash != '' GROUP BY project_hash
```

`projects` 总数 = Q3 行数，与项目榜同一谓词（Wake 2026-08-27 review 的教训：总数与榜单谓词必须一致）。

聚合核心是纯函数，便于用任意时区测试：

```rust
pub(crate) fn aggregate_time_rows(
    rows: impl IntoIterator<Item = TimeRow>,
    zone: &ResolvedZone,
    as_of: NaiveDate,
) -> InsightsTimeAggregate
```

`Dashboard::insights(&self, filter: &QueryFilter, as_of: NaiveDate) -> Result<InsightsPayload, UsageError>` 负责门控、执行 Q1-Q3、调用纯函数。

## 3. 周与 streak 规则（移植 Wake，保持一处定义）

- `INSIGHTS_WEEKS: usize = 53`。
- `week_start(day)` = 该日所在周的周一。
- 周下标：`weeks_back = (week_start(as_of) - week_start(day)).num_days() / 7`；`0..53` 内映射为 `52 - weeks_back`，窗外返回 `None`。
- `current_streak`：若 `as_of` 有请求，从 `as_of` 向前数；否则从 `as_of - 1` 向前数；遇到无请求日停止。
- `longest_streak`：日谱升序扫描的最长连续段。
- `busiest_day`：请求数最大的日；并列取较早的日（确定性）。
- 最近 7 天窗：`[as_of - 6, as_of]`；前 7 天窗：`[as_of - 13, as_of - 7]`。

## 4. 服务层：会话合并

读取 `usage_session_archive`（经 `ccr_db` repository 新函数，不在服务层写 SQL）：

- 全历史按 `platform` 分组计数 → agents 榜的 `sessions`。
- 最近 7 天与前 7 天的会话数：按 `created_at` 本地日归属。时间范围比较用半开区间 `created_at >= start_utc AND created_at < end_exclusive_utc`（由本地日边界换算为 UTC RFC3339）。不能沿用 `get_session_archive_daily_trends` 的 `created_at <= 'YYYY-MM-DD'` 写法，该写法把结束日当天 `T..` 之后的记录全部排除（`crates/ccr-db/src/database/repositories/usage_repo.rs:1305-1330`）。
- 平台键映射：`SourceKind::parse_id(platform)`；在 `parse_id` 中补 `kimi` → `KimiCode`（当前只认 `kimi_code`/`kimi-code`/`kimi code`，`crates/ccr-usage/src/source.rs:71-88`），`omp` 已映射到 `Pi`。映射失败的平台保留原名、`unmapped = true`。
  - 补 `kimi` 别名会影响 `parse_source_filter` 的其他调用方。实施前用 `rg "parse_id|parse_source_filter"` 列出调用方并确认无冲突；有冲突则改为只在 Insights 服务内做局部映射。

## 5. Wire DTO（`ccr-ui/src-tauri`，ts-rs 导出到 `src/types/generated/usage/`）

```rust
pub struct HomeInsightsResponse {
    pub as_of: String,                 // YYYY-MM-DD
    pub first_date: Option<String>,
    pub totals: InsightsTotals,        // requests, tokens, sessions, agents, projects, active_days
    pub sessions_indexed: bool,        // has_any_session_archive；false 时前端显示 `–`
    pub last_7_days: InsightsWindow,   // requests, sessions, active_days
    pub previous_7_days: InsightsWindow,
    pub daily: Vec<InsightsDay>,       // 53 周窗内有请求的日，升序
    pub current_streak: u32,
    pub longest_streak: u32,
    pub busiest_day: Option<InsightsDay>,
    pub hourly: Vec<f64>,              // 长度 24（ts 端 number[]）
    pub weekday: Vec<f64>,             // 长度 7，周一起始
    pub monthly: Vec<f64>,             // 长度 12
    pub trend_start: String,           // 53 周窗首个周一
    pub trend: Vec<InsightsTrendSeries>, // source, weekly: Vec<f64>(53)
    pub agents: Vec<InsightsAgentTally>, // source, requests, tokens, sessions, unmapped
    pub projects: Vec<InsightsTally>,    // key(project_hash), label, requests, tokens
    pub models: Vec<InsightsTally>,
    pub generated_at: String,
}
```

- 全部 `i64`/`u64` 字段带 `#[ts(as = "f64")]`（`.trellis/spec/ccr/backend/typed-ipc-bindings.md` §3）。数组字段在 Rust 侧直接用 `Vec<f64>` 或带 `#[ts(as = "Vec<f64>")]`。
- 不向前端暴露 `project_ref`、原始路径或 `record_json`。`project_label` 已是 llmusage 的展示标签。

## 6. 命令与缓存

```rust
#[ccr_tauri_command_macros::command]
pub async fn get_home_insights(state: State<'_, AppState>) -> Result<HomeInsightsResponse, String>
```

- `as_of = Local::now().date_naive()`，在命令层取得后传入服务与投影。
- 缓存键 `format!("{USAGE_SNAPSHOT_CACHE_PREFIX}home_insights:{as_of}")`，TTL `USAGE_SNAPSHOT_CACHE_TTL_SECS`；沿用 `get_usage_dashboard_v2` 的 single-flight 写法（`commands/usage.rs:1180-1215`），序列化失败也必须 `finish_cache_fill`。
- 导入完成后 `invalidate_usage_snapshot_cache` 按前缀失效（已有，`commands/usage.rs:128-133`），无需新增失效点。
- llmusage 未安装或 schema 不支持：返回现有错误字符串，前端显示空态；不 panic，不 `unwrap`。

## 7. 取舍

- 不做后台快照 job（D3）。代价：冷缓存首次调用同步执行 Q1-Q3。可接受依据：Q1 行数上界约 17.5k × 活跃来源数；Wake 同量级扫描为几十毫秒（其代码注释）。实施后在 `tracing` 中记录 `db_ms`，超过 500ms 时在任务 notes 记录实测值，另开后台 job 任务。
- 时间派生放 Rust 侧而非 SQL：IANA 小时换算在 SQL 中需要新增标量函数；Rust 侧换算与 `date_at` 同源，测试可直接喂固定偏移时区。
- 30 分钟分桶对 +05:45 等 45 分钟偏移时区会产生跨小时误差（桶边界不对齐本地整点）。小时分布只用于展示，接受该误差，在代码注释中说明。

## 8. 回滚

所有改动为新增：新文件、新 `FeatureKey` 变体、新命令、新 DTO。回滚 = 撤销本子任务提交并重新生成绑定与清单。`parse_id` 的 `kimi` 别名若引起回归，单独撤回该行。
