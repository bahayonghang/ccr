# Implement: Insights 投影与 typed 命令

按序执行。每步先跑最窄验证。Rust 测试直接运行时加 `-- --test-threads=1`（仓库规则）。

## Checklist

### 1. ccr-usage：时区与能力门控

- [x] `timezone.rs`：新增 `ResolvedZone::local_at(DateTime<Utc>) -> NaiveDateTime`，实现方式与 `date_at` 相同。
- [x] `capabilities.rs`：新增 `FeatureKey::Insights`（`as_str` = `"insights"`），`required_columns` 声明 `usage_bucket_30m` 的 `source`、`model`、`hour_start`、`project_hash`、`project_label`、`event_count`、`total_tokens`。
- [x] 验证：`cargo test -p ccr-usage -- --test-threads=1`

### 2. ccr-usage：纯聚合函数

- [x] 新建 `src/insights.rs`：`TimeRow`、`InsightsTimeAggregate`、`aggregate_time_rows`、`week_start`、周下标、streak、7 天窗函数（design §2-§3）。
- [x] `lib.rs` 注册模块，公开 `InsightsPayload` 及其子结构（`Serialize`，带 `ts` feature 的 `cfg_attr` 与现有 DTO 一致）。
- [x] 单元测试（design §3 与 prd AC1 的每一条各一个测试）：空输入全零；UTC 与 +05:30 固定偏移下的本地日/小时；周一起点与 53 周下标边界（第 0 周、第 52 周、窗外）；`as_of` 无请求时 streak 从前一天起算；最长 streak；busiest_day 并列取较早日；未来日期只计总量。
- [x] 验证：`cargo test -p ccr-usage insights -- --test-threads=1`

### 3. ccr-usage：`Dashboard::insights`

- [x] `db.rs`：`pub fn insights(&self, filter: &QueryFilter, as_of: NaiveDate) -> Result<InsightsPayload, UsageError>`；门控 `FeatureKey::Insights`；执行 Q1-Q3（design §2）；`gemini` 并入 `antigravity`。
- [x] 测试（`test-fixtures` feature，`create_projection_db` + `seed_bucket`）：9 个来源各一行时趋势含 9 条序列；`gemini` 行计入 `antigravity`；项目总数与项目榜行数一致；空 `model` 不进模型榜。
- [x] 验证：`cargo test -p ccr-usage --features test-fixtures -- --test-threads=1`

### 4. 来源别名

- [x] `rg -n "parse_id|parse_source_filter" crates ccr-ui/src-tauri` 列出调用方，确认补 `kimi` 别名无冲突后在 `source.rs::parse_id` 增加 `"kimi"`；有冲突则改为 Insights 服务内局部映射（design §4），并在任务 notes 记录原因。
- [x] 补 `source.rs` 现有别名测试。

### 5. ccr-db：会话归档计数

- [x] `repositories/usage_repo.rs` 新增两个只读函数：全历史按平台计数；按 UTC 半开区间 `[start, end)` 按平台计数。不修改现有 `get_session_archive_*` 函数。
- [x] 测试：结束日当天创建的会话计入窗口；区间外不计入。
- [x] 验证：`cargo test -p ccr-db -- --test-threads=1`

### 6. src-tauri：服务与 DTO

- [x] 新建 `services/home_insights.rs`（或放入 `services/usage.rs`，按文件长度决定；`services/usage.rs` 已超过 1,200 行，优先新文件）：DTO（design §5）、`compute_home_insights(llmusage, pool, as_of)`、平台映射。
- [x] 服务层测试（prd AC2）：`omp`→`pi`、`kimi`→`kimi_code`、未知平台单独成行且计入总数；最近 7 天会话含 `as_of` 当天。复用 `ccr_usage::fixtures`。
- [x] 验证：`cd ccr-ui && bun run tauri:test`

### 7. src-tauri：命令、注册、缓存

- [x] `commands/usage.rs`：`get_home_insights`（design §6），single-flight 缓存与 `record_command_duration`/`record_db_duration`。
- [x] `commands/handler_registry.rs`：按 `.trellis/spec/ccr/backend/tauri-handler-registry.md` 登记；更新注册表计数测试（+1）。
- [x] 验证：`cd ccr-ui && bun run tauri:check && bun run tauri:clippy`

### 8. 生成物

- [x] `just tauri-bindings`，确认 `src/types/generated/usage/HomeInsights*.ts` 与 `src/api/generated/usageV2.ts` 新增 `getHomeInsights`。
- [x] `just tauri-command-inventory`，提交中英文命令清单文档。
- [x] 验证：`just tauri-bindings-check`、`just ci-governance-check`

### 9. 收尾

- [x] `just fmt-check`、`just lint-strict`、`just test`（`just test` 红：3 个既有失败与本任务无关，见 task.json notes）
- [x] 在任务 notes 记录一次真实库上的 `db_ms`（design §7）。

## Review Gates

- 步骤 3 完成后：对照 design §2 检查 SQL 是否全部位于 `crates/ccr-usage`。
- 步骤 7 完成后：dispatch `tauri-ipc-reviewer` 审命令与注册；步骤 5 完成后 dispatch `sqlite-migration-reviewer` 确认无 schema 变更、只读查询走索引 `idx_usage_session_archive_platform_created_at`。

## Rollback Points

- 步骤 4 的别名改动独立提交，可单独撤回。
- 步骤 8 生成物与步骤 7 同一提交，撤回时一起撤回。
