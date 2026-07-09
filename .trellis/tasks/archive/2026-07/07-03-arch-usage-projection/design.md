# Design: 统一 usage 投影

## 侦查结论（与 PRD 差异）

1. adapter 的 `provider_breakdown` **已经**委托 ccr-usage（07-01 任务遗产），本任务收敛的是其余 9 个查询 + capabilities + paths + source。
2. adapter `AppPaths` 的 6 个扩展字段（bin_dir/backups_dir/exports_dir/hook_cmd_path/hook_sh_path/lock_path）与 `with_cli_home` **全仓零使用**，是死代码——不迁移，直接消失。cli.rs 只用 `root_dir`。
3. `LlmusageAdapterError` 在 commands 层只有一处显式 match（usage.rs:1918-1925 的 provider_stats 降级），错误类型保住则 commands/usage.rs（2778 行）近乎零改动。
4. CLAUDE.md "git dependency pinned rev" 表述与事实不符：src-tauri/Cargo.toml 只有 `ccr-usage = { path = ... }`，`llmusage_no_crate_guard` 明文禁止 llmusage crate 依赖。mod.rs 的不变量（CLI + 只读 SQLite）才是对的。

## 边界划分（目标态）

### crates/ccr-usage —— 唯一只读投影 owner

| 模块 | 内容 |
|---|---|
| `paths.rs` | `AppPaths { root_dir, db_path }` 不变（契约 Signatures 原样）；补 `default_discovery_uses_llmusage_root_not_legacy_ccr_root` 测试 |
| `source.rs` | 不变；补 gemini-cli / Open Code 别名断言（被丢测试回归） |
| `capabilities.rs` | 吸收 adapter 超集：`FeatureKey` 11 键全集、`UnsupportedReason`、`FeatureCapability`、`required_columns`、`min_schema_version`、`ensure_feature`、`read_schema_version`/`table_exists`/`column_exists`/`open_readonly_for_capabilities`；新增 `DbCapabilities { db_exists, db_readable, schema_version, features }::detect(paths)`（含 DB 缺失/不可读时对 9 个 DB-backed 特征的统一降级填充，即原 `populate_db_features` + detect 的 DB 半边） |
| `db.rs` | `QueryFilter` 增 `model`/`project_hash` 字段（契约细化）；`Dashboard` 吸收全部查询：overview / trends_daily / model_breakdown / provider_breakdown / project_breakdown / source_breakdown / heatmap / logs / diagnostics / home_overview；`ensure_feature_for_filter`（任何查询带 provider filter → ProviderBreakdown 门控）；`LogsQuery`/`LogsPage`/`SourceDiagnostics`/`DiagnosticsPayload`/`build_filter`/cursor 编解码；新增 `provider_breakdown_by_source(&[SourceKind], &QueryFilter) -> Vec<TaggedProviderBreakdown>`（逐 source 查询，保契约 "request separately" 语义） |
| `queries.rs` | 投影 DTO 全集（serde 形状逐字节不变）：TokenSummary、OverviewPayload、DailyTrendDto、ModelBreakdown、SourceBreakdownDto、ProviderBreakdownDto（+ 新方法 `display_provider()`/`output_tokens_total()`/`cache_tokens_total()`，不动字段）、ProjectBreakdown、HeatmapPoint、UsageRecordDto、HomeOverview* 4 型、`generated_at()`；新增 `TaggedProviderBreakdown { source: SourceKind, breakdown: ProviderBreakdownDto }`（组合，无字段拷贝） |
| `error.rs` | `UsageError` 不变 |

新依赖：`base64`（cursor，workspace 已有 0.22.1）。

### llmusage_adapter —— CLI sync / 事件 / Tauri DTO·错误映射

| 模块 | 处置 |
|---|---|
| `source.rs`、`paths.rs` | **删除文件**；mod.rs `pub use ccr_usage::{AppPaths, discover_llmusage_paths, SourceKind, canonical_source_id, parse_source_filter, platform_scope_label}` |
| `capabilities.rs` | 保留 `CapabilityReport`（Tauri payload：+ cli_available/cli_version/root_dir/db_path 展示字段）与 `detect_cli_version`（std_command Windows 隐窗，Tauri 特有）；`detect()` = `ccr_usage::DbCapabilities::detect` + SyncJsonEvents/Cancel 两个 CLI 域键拼装；re-export `FeatureKey` 等 |
| `db.rs` | `Dashboard` 保留为**同名薄 wrapper**（内含 `ccr_usage::Dashboard`，10 方法逐个委托 + `shared_usage_error` 映射）→ commands 层错误匹配、`LlmusageRuntime::dashboard()` 均零改动；`QueryFilter`/`ReportTimezone`/`LogsQuery`/`LogsPage`/`build_filter` 改为 re-export；删除全部 SQL 与 `SqlFilter` |
| `error.rs` | `LlmusageAdapterError` 不变（CliMissing/Cli 变体超出投影域）；`shared_usage_error` 保留 |
| `queries.rs` | 保留表现层转换 DTO（UsageSummaryDto/ModelStatDto/ProjectStatDto/HeatmapResponseDto/PaginatedLogsDto）+ `to_*` + `max_rfc3339`（通用时间工具，与投影无关）；投影 DTO 改为 re-export（`queries::OverviewPayload` 等路径对 commands 层不变） |
| `cli.rs`、`events.rs`、`LlmusageRuntime` | 不变（AppPaths import 经 re-export 自动解析） |

DTO 归属细化（契约更新点）：投影 DTO 定义归 ccr-usage、adapter 以 re-export 暴露并保留分叉权；**错误类型**与表现层转换 DTO 仍归 adapter 本地。理由：provider_breakdown 时代的"每查询双 DTO + 手工映射"扩展到 10 个查询会制造 ~10 套逐字段拷贝——TUI 的 `UsageProviderRow` 正是该模式的既成事故（PRD 点名删除）。

### ccr-tui —— 删影子结构 + 注入 seam

```rust
// app.rs 目标形状
pub type UsageLoader =
    std::sync::Arc<dyn Fn() -> Result<UsageDataset, UsageError> + Send + Sync>;

pub struct UsageDataset { pub rows: Vec<ccr_usage::TaggedProviderBreakdown> }

impl UsageApp {
    pub fn with_task_executor(exec) -> Self      // 默认真实 loader（现 load_usage_dataset）
    pub fn with_loader(exec, loader) -> Self     // 注入点
}

fn state_from_load_result(result) -> UsageLoadState  // 纯函数：错误分类唯一出口
```

- 删除 `UsageProviderRow` 与 `provider_row_from_shared`；ui.rs 改 `row.source` / `row.breakdown.*`，显示辅助改用 ProviderBreakdownDto 新方法。
- `load_usage_dataset` 改为一行 `dashboard.provider_breakdown_by_source(&[Claude, Codex], ...)`。
- 测试两层：`state_from_load_result` 纯函数 6 分类（Ok非空/Ok空/SchemaUnsupported/FeatureUnavailable/DbMissing/Query）；`with_loader` 注入 + `AsyncTaskExecutor::Handle`（tokio multi_thread 测试）驱动 start_fetch → drain 的状态机转移，无需真实 ~/.llmusage。

## 行为保持清单（迁移必须逐字节保住）

1. get_usage_dashboard_v2 降级：无 provider filter 时 SchemaUnsupported/FeatureUnavailable → `provider_stats: []`；显式 filter → 错误上抛（commands 层 match 不动）。
2. provider filter 附着在任意查询 → ProviderBreakdown 能力门控（`ensure_feature_for_filter` 整体迁移）。
3. logs：`usage_event_raw` 表探测、cursor 编解码、page_size+1 探测 has_next、include_total 单独 COUNT。
4. project_breakdown：`project_path` 列存在性探测选表达式。
5. overview：run_log 两查询失败降级 None + `tracing::warn`。
6. heatmap：滚动窗口覆盖 caller 的 since/until（防双 date 约束）。
7. source_breakdown：忽略 source filter、canonical 四源白名单、share 归一化。
8. TUI：Claude/Codex 分开请求；`provider = null` 显示 "unattributed"。
9. 所有 Tauri payload serde 形状零变化（DTO 逐字节搬家）；前端零改动。
10. schema 门控数值（MIN=10 / PROVIDER=14）与错误文案不变。

## 测试迁移矩阵

- adapter db.rs 14 测试：8 个 filter SQL 生成 + 6 个 fixture 查询 → 全迁 ccr-usage（fixture 用 adapter 超集版含 pricing 列；与 ccr-usage 现有 4 测试合并去重）。
- adapter capabilities.rs 5 测试 → 迁 ccr-usage（与现有 2 测试合并）。
- adapter paths.rs 2 测试 → `default_discovery...` 补进 ccr-usage；HOME override 去重留一份。
- adapter 侧新留：shared_usage_error 5 变体映射测试、CapabilityReport 拼装测试（CLI 域两键 + DB 键并入）。
- queries.rs 表现层 3 测试不动。

## 文档与契约修正

- 根 CLAUDE.md llmusage 段落改写：上游集成 = 已安装 llmusage CLI（sync）+ 只读 SQLite（schema-gated），**无 crate 依赖**（`llmusage_no_crate_guard` 锁定）；共享投影 = `crates/ccr-usage`（path dep）；`llmusage_adapter` 仅 CLI sync/事件/DTO·错误映射；升级流程 = 升级已安装 CLI + 核对 schema 版本与 `SourceSyncStats` NDJSON 字段兼容。
- `llmusage-provider-adapter.md` 契约细化（trellis-update-spec）：QueryFilter 扩展字段、Dashboard 全查询归属、DTO re-export 规则、TaggedProviderBreakdown、`rg 'usage_bucket_30m'` SQL 只命中 crates/ccr-usage 的审查清单语义。

## 提交切分

1. `refactor(usage)`: ccr-usage 扩容（capabilities/db/DTO/tagged API + 测试迁入）——纯新增，不动 adapter。
2. `refactor(tauri)`: adapter 收敛为薄委托（删 source/paths/SQL，wrapper + re-export）。
3. `refactor(tui)`: 删 UsageProviderRow、接 TaggedProviderBreakdown、loader seam + 状态机测试。
4. `docs`: CLAUDE.md 修正 + mod.rs 措辞 + 契约 spec 更新。

回滚点：每步独立可编译可测；B2 若发现 payload 形状回归，回滚仅涉及 adapter 一层。
