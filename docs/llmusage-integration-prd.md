# llmusage × ccr-ui 集成 PRD

> 目标读者：llmusage 维护者（含本仓 owner）
> 立场：以 ccr-ui 为下游消费者，盘点 llmusage 当前缺口，约定补齐顺序。

## 0. TL;DR

ccr-ui 计划**完全弃用**自家 `ccr-db::usage_repo` + `ccr-store::cost_tracker` 这条用量统计管线，改为：

1. 直接读 llmusage 维护的本地 SQLite（`~/.llmusage/llmusage.db` 或可配置路径）做仪表盘查询。
2. 让用户在 ccr-ui 里点"导入"等价于触发 `llmusage sync`（带进度回调）。
3. 前端 Vue 契约（`ccr-ui/src/types/usage.ts`）保持不变，差异由 llmusage 一侧补全或 Tauri 适配层兜底。

要让这条路跑通，llmusage 需要补的东西分四个层次：**数据模型**、**Rust 库 API**、**同步任务化**、**查询能力**。本 PRD 逐项列出。

---

## 1. 目标 / 非目标

### 1.1 目标
- llmusage 成为 ccr-ui 用量统计的**唯一数据源**，包括 token / cache / cost / 趋势 / 模型分布 / 项目分布 / 日志列表 / 热力图。
- ccr-ui 通过**库依赖**（首选）或**子进程**（备选）触发 `llmusage sync` 与读 dashboard，不再自行解析 JSONL。
- ccr-ui 现有 UI 不变，所有差异在 llmusage / Tauri 适配层闭环。

### 1.2 非目标
- 不要求 llmusage 完全复刻 ccr-db 的 schema；ccr-ui 接受字段重命名，但语义必须可对齐。
- 不要求 llmusage 处理 ccr 项目侧的 Profile / API Key 配置等非用量话题。
- 不要求 llmusage 替代 ccr-ui 的 session 索引（`SessionIndexer`）功能。

---

## 2. 目标架构

```
┌────────────┐  read   ┌──────────────────────┐
│  ccr-ui    │ ──────► │ llmusage SQLite DB   │ ◄── hooks (Codex notify / Claude Stop / OpenCode plugin / Gemini ?)
│ (Vue/Tauri)│         │  ~/.llmusage/        │
│            │         └──────────────────────┘
│            │  trigger sync
│            │ ──────► llmusage::sync::run_once  (lib call) / subprocess llmusage sync
│            │
│            │  subscribe progress
│            │ ◄────── tokio::mpsc<SyncEvent>   (lib)  / NDJSON stdout (subprocess)
└────────────┘
```

ccr-ui 侧只剩两件事：**调 query 层组装响应** + **包装 sync 任务并广播 Tauri 事件**。

---

## 3. 字段级 GAP 总览

下表把 `ccr-ui/src/types/usage.ts` 里每个字段映射到 llmusage 当前能力，标出补缺方向。

| ccr-ui 契约 | llmusage 现状 | 缺口 | 类别 |
|---|---|---|---|
| `UsageSummary.total_requests` | 无；只有 `bucket_count`（30 分钟桶数） | 加 `event_count` 到 `OverviewPayload`，按过滤条件计 `COUNT(*)` from `usage_event` | F1 |
| `UsageSummary.total_input_tokens` | `total.input_tokens` | 直接映射 | ✅ |
| `UsageSummary.total_output_tokens` | `total.output_tokens` (+`reasoning_output_tokens`?) | 决定口径：是否合并 reasoning；建议 ccr-ui 显示时合并 | ✅ |
| `UsageSummary.total_cache_read_tokens` | `total.cached_input_tokens` | 名称不同，且 llmusage 没有 cache_creation 维度，需要拆 | F1 |
| **`cache_creation_tokens`**（per-model & per-record） | 不存在 | Anthropic JSONL 同时给 `cache_creation_input_tokens` 和 `cache_read_input_tokens`，llmusage 当前并入 `cached_input_tokens`，必须拆开 | F1 |
| `UsageSummary.total_cost_usd` | 由 `pricing::estimate_cost_usd` 现算，简化静态档位 | 需要更细的 catalog（含 Anthropic / Codex / Gemini 各档），并落库为 `cost_with_cache_usd` / `cost_without_cache_usd` | F1 |
| `UsageSummary.cache_efficiency` | 不存在 | 由 `cache_read_tokens / (input_tokens + cache_read_tokens)` 派生即可，由 llmusage 直接返回，避免前端口径漂移 | F1 |
| `DailyTrend.input_tokens / output_tokens / cache_read_tokens / cost_usd / request_count` | `/api/trends` 仅返回 `total_tokens` | trends 必须返回完整分项；每日一行而不是 30 分钟桶 | F4 + F7 |
| `ModelStat.cost_with_cache / cost_without_cache / cache_savings / pricing_status / pricing_source / pricing_rate` | 仅 `estimated_cost_usd` | 需要双价口径 + 定价档位元数据 | F1 |
| `UsageRecordV2.record_json` | 不存储 | 决策点：是否在 `usage_event` 落盘原始 JSON。建议**新增 opt-in `usage_event_raw` 表**，避免污染主路径 | F5 |
| `UsageRecordV2.platform / project_path / source_id / recorded_at` | `usage_event` 已有 `source / project_label / event_at`；缺 `source_id / project_path` 原文 | 加 `source_file_id` 关联 + 可选 `project_path`（默认隐藏，ccr-ui 不要求 path 隐私） | F1 |
| `PaginatedLogs`（cursor + offset 双模式） | 无 logs API | 加 `Dashboard::logs(filter, page, cursor)` + `/api/logs` | F4 + F5 |
| `HeatmapResponse`（365 天 daily count） | 无；底层 buckets 够用 | 加 `Dashboard::heatmap(days, source)` + `/api/heatmap` | F4 |
| `UsageArchiveDiagnostics.live_sources / missing_sources / deleted_sources / archived_sessions / recent_completed_at / history_completed_at` | `source_cursor` + `run_log` 部分对应 | 引入 source 文件状态机（live / missing / deleted_by_user）和"recent vs history"两阶段游标 | F6 |
| Codex 重导（reset + recent N 天） | `sync --rebuild` 全量重导，无 per-source；recent_days 不存在 | 加 `--rebuild --source codex` + `--recent-days N` | F3 |
| 实时导入进度（job-progress / recent-ready / finished / failed） | 无；CLI 一次性 | 任务化 + 进度通道 | F3 |
| Gemini 平台 | 不支持 | 新增 `SourceKind::Gemini` + parser + integration | F1 |

---

## 4. 功能需求

### F1 数据模型扩展

#### F1.1 新增 Gemini 源

**主数据源：session transcript JSON**
- 路径：`~/.gemini/tmp/<projectHash>/chats/session-*.json`（与 ccr-db `usage_import_service.rs` 现行实现对齐，已经在生产环境跑过）。
- 字段抽取：每条 assistant 消息读 `tokens.{input,output,cached}` 或 `usageMetadata` / `usage_metadata`（兼容老版本）。`output_tokens` 含 thought tokens；如需拆 `reasoning_output_tokens`，按 Gemini API 的 `thoughtsTokenCount` 单独抽取。
- 项目识别：用 `~/.gemini/projects.json` 把 `projectHash` 反查回原始 cwd，再交给 `project::resolve_project_info` 走和 Claude/Codex 同样的归一化流水线。
- 文件状态：用与 Claude 一致的 `FileCursor`（fingerprint + size + mtime + tail signature + offset）。

**可选 hook（推荐安装但不强依赖）**
- Gemini CLI v1 hooks 已支持 `SessionStart / SessionEnd / Notification / PreCompress`，配置写在 `~/.gemini/settings.json::hooks.SessionEnd`，约定 stdin/stdout JSON、exit 0/2/其他三档。
- `integrations::gemini::install` 行为（与 Claude integration 同结构）：
  1. 备份 `~/.gemini/settings.json` 到 `~/.llmusage/backups/`。
  2. 合并写入：
     ```json
     {
       "hooks": {
         "SessionEnd": [
           { "command": "<HookTarget::shell_command(gemini, SessionEnd)>" }
         ]
       }
     }
     ```
  3. probe 通过校验该数组里是否还存在自家命令字符串。
- 没有 hook 时：依赖 ccr-ui 主动调 `sync` 或定时器，不影响数据正确性，只影响实时性。

**不依赖 OpenTelemetry**
- Gemini CLI 还能开 `telemetry.target=local` 写 `.gemini/telemetry.log`（含 `gemini_cli.token.usage` counter），但这条路径需要用户主动 opt-in 且 schema 与 OTLP 强绑定，留作未来增强。v1 不引入。

**其他配置**
- `models.rs::SourceKind::Gemini`（`as_str() == "gemini"`）。
- `sources::registered_parsers / registered_integrations` 注册新源。
- `pricing.rs` 加 Gemini 档位（`gemini-2.5-pro / gemini-2.5-flash / gemini-2.0-flash` 等）。
- `usage_bucket_30m` schema 无需改动，靠 `source` 列区分。

#### F1.2 拆 cached_input_tokens → cache_creation + cache_read
- `UsageTokens` 拆字段：
  ```rust
  pub struct UsageTokens {
      pub input_tokens: i64,
      pub cache_read_tokens: i64,        // 原 cached_input_tokens 的"读"语义
      pub cache_creation_tokens: i64,    // 新增
      pub output_tokens: i64,
      pub reasoning_output_tokens: i64,
      pub total_tokens: i64,
  }
  ```
- `usage_event` / `usage_bucket_30m` 加列 `cache_creation_tokens INTEGER NOT NULL DEFAULT 0`。
- `parsers/claude.rs` 同时读 `cache_creation_input_tokens` 和 `cache_read_input_tokens`。
- 其他源（Codex / OpenCode / Gemini）若不区分，`cache_creation_tokens = 0`。

#### F1.3 双价 + 定价元数据
- `pricing::estimate_cost_usd` 改为返回结构：
  ```rust
  pub struct CostBreakdown {
      pub cost_with_cache_usd: f64,    // 真实账单口径（cache_read 走折扣价）
      pub cost_without_cache_usd: f64, // 全量不打折，用于估"省下了多少"
      pub pricing_status: PricingStatus, // priced | unpriced | partial
      pub pricing_source: Option<String>, // "static-v1" | "litellm-snapshot-2026-04" | None
      pub pricing_rate: Option<String>,   // 简短描述，例 "input=$3/MTok, cache=$0.3/MTok, output=$15/MTok"
  }
  ```
- `usage_event` 加列：`cost_with_cache_usd / cost_without_cache_usd / pricing_status / pricing_source`。
- bucket 聚合时同步 sum；`Dashboard` 查询返回 `cache_savings = cost_without_cache - cost_with_cache`。

**价目表生命周期（v1 决策）**
- 内置静态档位 `pricing/static-v1.json`（编译期 embed），覆盖 codex / claude / gemini / opencode 主流模型。`pricing_source = "static-v1"`。
- 用户可执行 `llmusage doctor --refresh-pricing <path>` 把外部 JSON 快照（建议来自 LiteLLM `model_prices_and_context_window.json` 这类社区维护源）落到 `~/.llmusage/pricing/litellm-snapshot-YYYY-MM.json`。命中时 `pricing_source = "litellm-snapshot-YYYY-MM"`。
- llmusage 本身**不直连远端**，避免破坏"本地优先"立场；快照刷新由用户/CI 手动触发。
- 命中不到任何档位 → `pricing_status = unpriced`，`pricing_source = None`，cost 字段 0。

#### F1.4 总请求数（event count）
- `OverviewPayload` 加 `total_events: i64`、`last_24h_events: i64`。
- `model_breakdown / source_breakdown / project_breakdown / cost_breakdown` 全部加 `event_count`。

#### F1.5 raw record（opt-in，默认关）
- 新表 `usage_event_raw(event_key TEXT PRIMARY KEY, raw_json TEXT NOT NULL, created_at TEXT NOT NULL)`，与 `usage_event` 1:1。
- 入口：
  ```rust
  pub struct BootstrapOptions { pub enable_raw_archive: bool /* 默认 false */ }
  pub fn Store::bootstrap_with(opts: BootstrapOptions) -> Result<()>;
  ```
  原 `Store::bootstrap()` 等价于 `bootstrap_with(Default::default())`。
- 状态读取：`Store::raw_archive_enabled() -> bool`，写在 `meta` kv 表里，避免靠环境变量飘移。
- 解析 driver 在 `UsageEvent` 落库的同一事务里写 raw 行，保证两表一致。开关关闭时 driver 跳过 raw 写入。
- ccr-ui 在 init 时调 `bootstrap_with(BootstrapOptions { enable_raw_archive: true })`，从此 sync 自动写 raw；想关掉时调 `Store::set_raw_archive(false)` + `sync --rebuild` 清旧数据。

### F2 Rust 库 API（ccr-ui crate 直连）

#### F2.1 Cargo 元信息
- `Cargo.toml` 显式声明：
  ```toml
  [lib]
  name = "llmusage"
  path = "src/lib.rs"

  [[bin]]
  name = "llmusage"
  path = "src/main.rs"
  ```
- 公开版本号语义：`0.5.x` 起 lib API 进入 SemVer 守门。

#### F2.2 公开表面
明确文档承诺 stable 的 mod 路径与类型：
- `llmusage::paths::AppPaths`：`AppPaths::discover()` 与 `AppPaths::with_root(PathBuf)`（**新增**，让 ccr-ui 指定自定义 DB 根路径）。
- `llmusage::store::Store`：`Store::new(&AppPaths)`、`Store::bootstrap()` / `Store::bootstrap_with(BootstrapOptions)`、`Store::open_connection()`、`Store::raw_archive_enabled()`、`Store::reset_for_source(SourceKind)`。要求 `Store: Clone + Send + Sync`（已基本满足，Cargo 测试覆盖）。
- `llmusage::store::WorkerLock`：携带 `holder_pid + holder_kind (cli|library|hook) + acquired_at` 元信息；`Store::acquire_worker_lock_with(timeout: Duration, holder_kind: HolderKind)` 等待版入口（默认超时 30s）。
- `llmusage::query::{Dashboard, QueryFilter, ReportTimezone}`：方法签名见 §6。
- `llmusage::sync::{run_with_progress, SyncOptions, SyncEvent, SyncSummary}`：见 F3。
- `llmusage::integrations::{probe_all, install_all, uninstall_all, IntegrationProbe, IntegrationAction}`。

非公开（保留内部）：`parsers::*`、`store::sync_writer::*`、`web::*`、`tui::*`、`commands::*`。

#### F2.3 错误类型
- 当前用 `anyhow::Result`。ccr-ui 需要更稳定的错误，建议：
  - 引入 `thiserror::Error` 派生的 `pub enum LlmusageError { Io, Db, Parse, LockBusy, ConfigInvalid, NotInitialized, ... }`。
  - 公开 API 改为 `Result<T, LlmusageError>`；CLI 内部继续可以用 `anyhow` 包装。

### F3 同步任务化（progress + cancel + recent_ready）

#### F3.1 选项与事件
```rust
pub struct SyncOptions {
    pub source_filter: Option<SourceKind>,   // None = 全部
    pub recent_days: Option<u32>,            // 仅扫描最近 N 天文件，None = 全量
    pub rebuild: bool,                       // 等价 sync --rebuild
    pub parallelism: Option<usize>,
}

pub enum SyncEvent {
    Started { job_id: String, files_total: u64 },
    SourceStarted { source: SourceKind, files_total: u64 },
    Progress { source: SourceKind, files_scanned: u64, records_imported: u64 },
    RecentReady { source: SourceKind },         // 最近窗口扫完，可以让 UI 提前刷新
    SourceFinished { source: SourceKind, stats: SourceSyncStats },
    Finished { summary: SyncSummary },
    Failed { error: String },
}
```

#### F3.2 入口
```rust
pub async fn run_with_progress(
    store: &Store,
    options: SyncOptions,
    cancel: tokio_util::sync::CancellationToken,
    sender: tokio::sync::mpsc::Sender<SyncEvent>,
) -> Result<SyncSummary, LlmusageError>;
```
- 取消语义：`CancellationToken::cancel()` 后 driver 在下一个 file/event 边界中断，已写入数据保留，cursor 不回滚。
- 进度节流：`Progress` 事件至少 200ms 一次，避免淹没消费者。

#### F3.3 子进程兼容
- `llmusage sync --json-events` 把 `SyncEvent` 以 NDJSON 形式写到 stdout，方便不能直接链接 lib 的下游（如未来分发独立 CLI 时）。

#### F3.4 子命令补全
- `llmusage sync --source codex` / `--source claude` / `--source gemini` / `--source opencode`。
- `llmusage sync --recent-days 30`。
- `llmusage sync --rebuild --source codex`（用于 ccr-ui 的"修复 Codex"按钮）。
- `--rebuild --source <X>` 仅 reset 该源的 cursor / event / bucket / source_file，其他源不动。需要把 `Store::reset_usage_data()` 拆出 `Store::reset_for_source(SourceKind)`，原方法保留为 `for_source(All)` 的语义糖。

### F4 查询过滤与扩展端点

#### F4.1 Dashboard 方法接受 `QueryFilter`（含 timezone）
```rust
pub struct QueryFilter {
    pub source: Option<SourceKind>,
    pub model: Option<String>,
    pub since: Option<chrono::NaiveDate>,    // 按 timezone 解释
    pub until: Option<chrono::NaiveDate>,    // 按 timezone 解释
    pub project_hash: Option<String>,
    pub timezone: ReportTimezone,            // 默认 Local；DB 仍按 UTC 存
}
```
所有 `Dashboard::overview/trends/model_breakdown/source_breakdown/project_breakdown/cost_breakdown/heatmap/logs/health` 都接 `&QueryFilter`。

**timezone 处理约定**
- 存储层始终是 UTC（`event_at` / `hour_start` 都是 RFC3339 Z 字符串）。
- 查询层在 SQL 里把 `event_at` 偏移到 `timezone`，再 `substr(..., 1, 10)` 取本地日期；`since/until` 反向偏移成 UTC 边界后比较。
- 已有的 `query::reports::ReportTimezone` 直接搬过来作为 stable type。

#### F4.2 趋势按日聚合 + 分项
```rust
pub struct DailyTrendPoint {
    pub date: String,              // YYYY-MM-DD（按时区）
    pub event_count: i64,
    pub input_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub cost_with_cache_usd: f64,
}
pub fn trends_daily(&self, filter: &QueryFilter, timezone: ReportTimezone) -> Result<Vec<DailyTrendPoint>>;
```
原 `Dashboard::trends("day"|"week"|"month"|"all")` 保留兼容，标 `#[deprecated]`。

#### F4.3 Heatmap
```rust
pub struct HeatmapPoint { pub date: String, pub event_count: i64, pub total_tokens: i64 }
pub fn heatmap(&self, filter: &QueryFilter, days: u32) -> Result<Vec<HeatmapPoint>>;
```
HTTP 镜像：`GET /api/heatmap?source=&days=365`。

#### F4.4 Logs 分页
```rust
pub struct LogsQuery {
    pub filter: QueryFilter,
    pub page_size: u32,            // 默认 50
    pub cursor: Option<String>,    // base64(event_at + event_key)
    pub include_total: bool,
    pub include_raw_json: bool,    // true 时 join usage_event_raw
}
pub struct LogsPage {
    pub records: Vec<LogRecord>,
    pub next_cursor: Option<String>,
    pub total: Option<i64>,
}
pub fn logs(&self, query: &LogsQuery) -> Result<LogsPage>;
```
HTTP 镜像：`GET /api/logs?source=&model=&since=&until=&page_size=&cursor=&include_total=&include_raw=`。

### F5 健康/诊断 + 文件状态机

#### F5.1 source 文件状态
- 现 `source_cursor` 仅记录"哪个文件扫到哪了"，不记"文件还在不在"。
- 引入 `source_file` 表（或扩展 `source_cursor`）：
  ```sql
  CREATE TABLE source_file (
    source TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER,
    file_state TEXT NOT NULL,    -- live / missing / deleted_by_user
    last_seen_at TEXT NOT NULL,
    PRIMARY KEY (source, file_path)
  );
  ```
- 每次 sync 扫描后比对："本次没看到 + 上次状态是 live" → 标 `missing`；用户主动剔除 → `deleted_by_user`。

#### F5.2 Recent vs History 完成时间
- `source_sync_status` 加 `recent_completed_at TEXT` / `history_completed_at TEXT`，覆盖 ccr-ui 的 `UsageArchiveDiagnostics.recent_completed_at / history_completed_at`。
- "recent" 定义：最近 `recent_days` 窗口内的文件全部扫完。
- "history" 定义：cursor 完成至最早文件。

#### F5.3 Diagnostics 端点
```rust
pub struct DiagnosticsPayload {
    pub archive_root: String,
    pub by_source: Vec<SourceDiagnostics>,
    pub recent_failures: Vec<RunRecord>,
}
pub struct SourceDiagnostics {
    pub source: SourceKind,
    pub live_files: u64,
    pub missing_files: u64,
    pub deleted_files: u64,
    pub recent_completed_at: Option<String>,
    pub history_completed_at: Option<String>,
}
pub fn diagnostics(&self) -> Result<DiagnosticsPayload>;
```
HTTP 镜像：`GET /api/diagnostics`。

### F6 兼容已有 archive 概念
- ccr-ui 当前的 archive root 来自 `ccr_db::database::get_usage_archive_db_path()`。改造后 `archive_root` 含义改为 llmusage 的 DB 路径（`AppPaths.db_path`）。前端字段不变，语义升级。

### F7 路径与多实例

- `AppPaths::with_root(PathBuf)` 让 ccr-ui 指定 `~/.ccr/llmusage/`（或共享标准 `~/.llmusage/`）。
- 环境变量 `LLMUSAGE_HOME` 优先级最高，其次 `--home <PATH>` CLI flag，最后默认 `~/.llmusage`。
- DB 默认走 SQLite WAL，确保读端和 sync 写端可并发；ccr-ui 一侧只读连接打 `PRAGMA query_only=ON`。

### F8 Tauri 友好的序列化

- 全部公开 struct 加 `#[derive(Serialize, Deserialize)]`，casing **全部 snake_case**（与 ccr-ui 现有 TypeScript 契约一致）。
- 当前 `query::reports::*` 是 camelCase（CLI JSON 兼容 ccusage）。建议拆两套：CLI JSON 保留 camelCase，library / `Dashboard` 输出 snake_case。

---

## 5. 非功能需求

| 维度 | 要求 |
|---|---|
| 性能 | 单次 `Dashboard::overview` < 50ms（10 万 event）；`logs` 分页 < 30ms/页 |
| 内存 | sync 单轮峰值 < 200MB |
| 并发 | 支持多个只读 reader + 单 writer（WAL） |
| 取消 | sync 任务可在 < 500ms 内中断 |
| 兼容 | DB schema 升级走 `Store::bootstrap` 内的 migration 序号；旧版本 db 自动迁移 |
| 可测 | 公共 API 100% 有集成测试覆盖；提供 `llmusage::testing::Fixture` 给下游做 e2e |
| 隐私 | raw_json 默认关；hook payload 不出本机 |

---

## 6. 关键接口契约（示例）

```rust
// ──── library entry ────
let paths = llmusage::paths::AppPaths::discover()?;
let store = llmusage::store::Store::new(&paths);
store.bootstrap()?;

// dashboard
let dash = llmusage::query::Dashboard::open(&store)?;
let filter = llmusage::query::QueryFilter {
    source: Some(SourceKind::Claude),
    since: Some(NaiveDate::from_ymd_opt(2026, 4, 1).unwrap()),
    ..Default::default()
};
let overview = dash.overview(&filter)?;
let trends = dash.trends_daily(&filter, ReportTimezone::Local)?;
let heat = dash.heatmap(&filter, 365)?;

// sync with progress
let (tx, mut rx) = tokio::sync::mpsc::channel(64);
let cancel = CancellationToken::new();
tokio::spawn(llmusage::sync::run_with_progress(
    &store,
    SyncOptions { source_filter: None, recent_days: Some(30), rebuild: false, parallelism: None },
    cancel.clone(),
    tx,
));
while let Some(event) = rx.recv().await {
    match event {
        SyncEvent::RecentReady { source } => app.emit("usage:job-recent-ready", source)?,
        SyncEvent::Finished { summary } => app.emit("usage:job-finished", summary)?,
        ev => app.emit("usage:job-progress", ev)?,
    }
}
```

```ts
// ccr-ui 一侧契约不变（保留 src/types/usage.ts 全部字段），
// Tauri 命令层做最薄的字段重命名 + 默认值填充。
```

---

## 7. 阶段划分

| 阶段 | 范围 | 验收 |
|---|---|---|
| **M0**：可调通（不改字段） | F2 公开 API 整理 + Cargo lib 声明；F4.1 `QueryFilter` 接入；F3 `run_with_progress` 雏形（仅 Started/Finished/Failed 事件） | ccr-ui 在一个隐藏开关后能成功调 llmusage 库渲染 summary / model / project，gemini 走老路径 |
| **M1**：数据模型对齐 | F1.2 cache 拆分；F1.3 双价 + 定价元数据；F1.4 event_count；F4.2 daily trends 分项；F4.3 heatmap | 关闭 ccr-db 的 claude/codex/opencode importer，前端字段 100% 由 llmusage 提供 |
| **M2**：完整 UX | F1.5 raw archive；F4.4 logs 分页；F5 source state + diagnostics；F3.2 RecentReady / 进度节流 / 取消；F3.4 子命令补全 | ccr-ui archive 视图、Codex 修复、日志分页全部走 llmusage |
| **M3**：Gemini + 收尾 | F1.1 Gemini 源 + 价目；F8 序列化口径统一；error 类型 thiserror 化；版本号升 0.5.0 | ccr-db 用量管线代码可以删除；ccr-ui 仅保留 llmusage 适配层 |

---

## 8. 验收标准（M3 完成时）

1. ccr 仓中 `crates/ccr-db/src/database/repositories/usage_repo.rs` + `crates/ccr-store/src/cost_tracker.rs` 标 `#[deprecated]` 并删除业务调用。
2. ccr-ui `useUsageStore` 内所有 `getUsageXxxV2` 命令的 Tauri 实现改为薄包装：调用 `llmusage::query::Dashboard` + `llmusage::sync::run_with_progress`。
3. `ccr-ui/src/types/usage.ts` 字段不变；Vue 组件零改动通过冒烟。
4. `llmusage --json-events sync --recent-days 30` 子进程模式作为 fallback 路径也可用。
5. llmusage 单测/集测覆盖：四源解析、双价计算、cache 拆分、heatmap、logs 分页、source 状态机、cancel 路径。

---

## 9. 已决议事项（原开放问题）

以下六个问题在 v1 阶段已落定方案，对应实现细节已经回写到 §F1–F5。

### 9.1 Gemini hook + 数据源
- **决议**：与 Claude 同款双层方案。主路径解析 `~/.gemini/tmp/<projectHash>/chats/session-*.json`（ccr-db 已验证的字段：`tokens.{input,output,cached}` / `usageMetadata` / `usage_metadata`），可选安装 `SessionEnd` hook 触发实时 sync。OTel telemetry 不进 v1。
- **依据**：Gemini CLI v1 hooks 系统已 GA（`SessionStart / SessionEnd / Notification / PreCompress`，stdin/stdout JSON 协议与 Claude 完全同构）；`~/.gemini/tmp` JSON 是当前 ccr-db 实测可用的取数路径。
- **影响**：F1.1 已展开完整 spec。

### 9.2 raw_json 归属
- **决议**：放 llmusage，opt-in，默认关。`Store::bootstrap_with(BootstrapOptions { enable_raw_archive })` 入口；状态写 `meta` 表，不靠环境变量。ccr-ui 在 init 时打开。
- **理由**：避免 ccr-ui 自建 raw 表造成双写；llmusage 默认关满足"本地优先 + 隐私优先"立场，下游一个开关即可启用。
- **影响**：F1.5 已展开。

### 9.3 timezone 一等公民
- **决议**：`QueryFilter.timezone` 成为 stable 字段，默认 `Local`；存储仍 UTC，查询层在 SQL 里按 timezone 折算本地日期与 since/until 边界。
- **理由**：ccr-ui 用户跨时区，硬塞 UTC 必然导致"今天"对不上。`ReportTimezone` 类型已存在，只需提级公开。
- **影响**：F4.1 已更新。

### 9.4 多 ccr 实例共享 `~/.llmusage`
- **决议**：沿用 `Store::acquire_worker_lock`，扩展 `WorkerLock` 元信息 `holder_pid + holder_kind (cli|library|hook) + acquired_at`；新增 `acquire_worker_lock_with(timeout, holder_kind)` 等待版入口（默认超时 30s）；读端走 SQLite WAL 不持锁，永不阻塞。
- **理由**：sync 不能多写并发，但读必须永远畅通。元信息便于 `llmusage status` 排查"谁正在写"。
- **影响**：F2.2 加 `WorkerLock` 公开签名；非功能需求章节确认 WAL。

### 9.5 Codex repair 精确语义
- **决议**：`sync --rebuild --source codex` 仅清 codex 一源的 cursor / event / bucket / source_file，其他源不动。`Store::reset_usage_data()` 拆出 `reset_for_source(SourceKind)`，原方法保留为 `for_source(All)` 等价语义糖。
- **理由**：符合 ccr-ui 现有按钮的用户预期"修 Codex 不影响 Claude"。
- **影响**：F3.4 已更新。

### 9.6 价目表来源
- **决议**：本地静态档位（`pricing/static-v1.json` 编译期 embed）+ 用户手动刷新的离线快照（`~/.llmusage/pricing/litellm-snapshot-YYYY-MM.json`）。**不直连远端**。
- **`pricing_source` 取值集合**：`"static-v1"` / `"litellm-snapshot-YYYY-MM"` / `None`（未命中时配合 `pricing_status = "unpriced"`）。
- **理由**：保持本地优先；ccr-ui 拿到的字段值域稳定，便于在 UI 上着色提示"这条数据用的是哪份价目"。
- **影响**：F1.3 已更新生命周期描述。

---

## 9b. 仍待确认（不影响 v1 开工）

1. **OpenTelemetry 增强**：v2 是否在 Gemini 之外把 telemetry 引入到 Codex / Claude 也作为补充数据源？需评估 OTel parser 维护成本。
2. **pricing 快照来源**：用 LiteLLM `model_prices_and_context_window.json` 还是社区另起一份？需要选一个 stable 上游。
3. **migration 框架选型**：当前 `Store::bootstrap` 内是顺序 SQL；schema 频繁演进时是否引入 `refinery` / `barrel` / 自家 versioned migration？M1 之前要拍。

---

## 10. 不在范围内

- 不替代 ccr-ui 的 `SessionIndexer` / 会话归档功能。
- 不改 ccr-ui Profile / Auth / VS Code 扩展。
- 不引入云端上传 / 远程聚合（坚持 llmusage 本地优先立场）。
- 不要求 llmusage 提供 Web 仪表盘的 i18n 与 ccr-ui 对齐（两套 UI 各自演进）。
- v1 不引入 OpenTelemetry 作为 Gemini / Codex / Claude 的数据源（见 §9b）。
- v1 不要求 llmusage 自动联网刷新价目（见 §9.6）。

---

## 附录 A：ccr-ui 适配层最小落地草图

M0 阶段，ccr-ui 在 `ccr-ui/src-tauri` 一侧大致是这样接入的：

```rust
// crates/ccr-ui/src-tauri/src/llmusage_adapter.rs（新增）
use llmusage::{
    paths::AppPaths,
    store::{Store, BootstrapOptions},
    query::{Dashboard, QueryFilter, ReportTimezone},
    sync::{self, SyncOptions, SyncEvent},
};

pub struct LlmusageHandle {
    pub store: Store,
}

impl LlmusageHandle {
    pub fn init() -> anyhow::Result<Self> {
        let paths = match std::env::var("LLMUSAGE_HOME").ok() {
            Some(home) => AppPaths::with_root(home.into())?,
            None => AppPaths::discover()?,
        };
        let store = Store::new(&paths);
        store.bootstrap_with(BootstrapOptions { enable_raw_archive: true })?;
        Ok(Self { store })
    }

    pub fn dashboard(&self, filter: QueryFilter) -> anyhow::Result<serde_json::Value> {
        let dash = Dashboard::open(&self.store)?;
        Ok(serde_json::json!({
            "summary": dash.overview(&filter)?,
            "trends": dash.trends_daily(&filter)?,
            "model_stats": dash.model_breakdown(&filter)?,
            "project_stats": dash.project_breakdown(&filter)?,
            "heatmap": dash.heatmap(&filter, 365)?,
            "archive": dash.diagnostics()?,
            "generated_at": chrono::Utc::now().to_rfc3339(),
        }))
    }
}
```

```rust
// 现有 Tauri 命令 get_usage_dashboard_v2 退化为：
#[tauri::command]
pub async fn get_usage_dashboard_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Value, String> {
    let filter = QueryFilter {
        source: platform.as_deref().map(parse_source).transpose()?,
        since: start_date.as_deref().map(parse_naive_date).transpose()?,
        until: end_date.as_deref().map(parse_naive_date).transpose()?,
        timezone: ReportTimezone::Local,
        ..Default::default()
    };
    state.llmusage.dashboard(filter).map_err(|e| e.to_string())
}
```

核心观察：**ccr-ui Tauri 层从"自行查 SQLite + 拼 JSON"退化为"参数翻译 + 透传"**，前端 Vue 侧零改动。
