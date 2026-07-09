# Tauri IPC seam 类型化 — 技术设计（usage V2 试点）

## 0. 侦查结论（修正 PRD 预设）

- 试点 domain 定为 handler_registry **"Usage V2" 组全部 17 条命令**（`commands/usage.rs`）。usage-projection 已完成，投影 DTO 归 `ccr-usage`，adapter `queries.rs` 持表现层 DTO——**17 条命令的类型擦除点全部是函数末尾一行 `serde_json::to_value`**，命令体内部早已是具名类型。试点成本远低于 PRD 预估。
- `compute_usage_dashboard_payload` 已是 State-free 函数（入参 `Arc<LlmusageRuntime>` + `DbPool`），服务层抽取模式在仓内已有先例，唯一 `json!()` 宏拼装点也在这里。
- 前端 `types/usage.ts`（~400 行）是手写 Rust DTO 镜像，即 PRD 所述漂移面；4 个调用方（stores/usage.ts、stores/homeUsageOverview.ts、composables/usePlatformUsageInsight.ts、views/MonitoringView.vue）以显式泛型 `getXxxV2<HandWrittenType>()` 传入镜像类型。
- 发现既有重复：`commands/usage.rs` 本地定义 `HomeOverviewPlatformStats/Summary/SeriesItem`（u64）与 `ccr-usage` 同名类型（i64）并存，wire 上走本地 u64 版。本任务不合并（另行标记），但 TS 导出只从 wire 实际使用的本地版出，避免同名导出文件冲突。

## 1. 工具选型：ts-rs（specta 落选）

| 维度 | ts-rs v11 | specta + tauri-specta |
|---|---|---|
| 成熟度 | 稳定版，广泛使用 | tauri-specta 长期 2.0.0-RC，生产锁 RC 风险 |
| 与 handler_registry 契约 | 零接触——只生成类型 | 要求 `collect_commands!` 收集命令、倾向接管 invoke handler，与 `define_command_registry!` 冻结契约（309/317）正面冲突或双注册表并存 |
| 与 api-facade-boundary 契约 | 手写 domains/* wrapper 保留，仅替换返回类型 | 生成 bindings.ts（含调用函数）与现有手写 wrapper 重复，churn 大 |
| serde 兼容 | serde-compat 默认开启（rename_all/default/tag） | 同样支持 |
| 试点可逆性 | 删 derive 即回退 | 引入命令收集宏，回退面大 |

**代价（写入推广评估）**：ts-rs 只保"参数/载荷 shape"编译期一致；"命令名 ↔ wrapper 调用串"仍是手工维护（运行时才暴露）。试点验证 shape 守卫价值后，命令名守卫可作为后续任务（如从 handler_registry 导出命令名清单给 TS 侧断言）。

**关键配置（实现期修正）**：ts-rs 已发布各版（v10/v11）均把 i64/u64 硬编码映射为 `bigint`，`TS_RS_LARGE_INT` 是未发布 main 分支（v12）特性，`.cargo/config.toml [env]` 方案不可用。实际机制：**字段级 `#[ts(as = "f64")]`**（Option 用 `as = "Option<f64>"`，map 用 `as = "HashMap<String, f64>"`），使 64 位整数生成 `number`（wire 是 serde_json number，token 量级 << 2^53 无精度风险）。漏标一个字段 → 生成物出现 `bigint` → 前端消费处 type-check 报错 + 生成 diff 可见。ts-rs 升 v12 后可整体切 `TS_RS_LARGE_INT=number` 并删除字段注解（写入 spec 升级路径）。另实测：`export_to` 相对的是默认 bindings 目录（`<manifest>/bindings/`）而非 manifest 目录，src-tauri 侧为 `../../src/types/generated/usage/`。

## 2. 分层与归属

```
ccr-usage (feature "ts"，默认关闭，CLI 链路零新依赖)
  └─ 4 个上 wire 的投影 DTO 加 cfg_attr derive(TS)：
     DailyTrendDto / ProviderBreakdownDto / SourceBreakdownDto / UsageRecordDto
     export_to = "../../ccr-ui/src/types/generated/usage/"

ccr-ui/src-tauri (直接依赖 ts-rs，无需 feature gate；不在 CLI 依赖图)
  ├─ src/services/mod.rs + src/services/usage.rs   [新增]
  │   State-free 查询编排（同步，spawn_blocking 内运行）：
  │   usage_summary / usage_trends / usage_by_model / usage_by_provider /
  │   usage_by_project / usage_heatmap / usage_logs / dashboard_payload /
  │   home_usage_overview（usage 侧拼装）
  │   入参：&LlmusageRuntime（或 &Dashboard）/ &DbPool + 业务参数
  │   返回：具名 DTO（Result<Dto, String>，错误文案零漂移）
  │   wire DTO 与 UsageLogsQuery 随迁至此；commands/usage.rs `pub use` 保路径
  ├─ commands/usage.rs：17 条命令签名 Result<Value,String> → Result<Dto,String>
  │   命令体 = 计时/metrics + State 提取 + spawn_blocking(service) + 缓存
  │   新增 UsageDashboardResponse 结构体替换 json!() 拼装
  │   dashboard 缓存命中路径：serde_json::from_value（相关 DTO 补 Deserialize）
  ├─ usage_jobs.rs / session_index_jobs.rs / llmusage_adapter/{queries,capabilities}.rs
  │   相关 wire 类型加 derive(TS)
  └─ export_to = "../src/types/generated/usage/"

ccr-ui/src (前端)
  ├─ src/types/generated/usage/   [新增，生成物入库]
  ├─ src/types/usage.ts → 改造为 re-export shim：
  │   镜像接口删除，`export type { UsageSummaryDto as UsageSummary } from ...` 保旧名；
  │   视图专用类型（HomeOverviewViewMode 等）保留手写
  ├─ src/api/domains/stats.ts：17 个 V2 wrapper 去 <T = UnknownRecord>，
  │   返回具名生成类型；UsageLogsQuery 改 re-export 生成版
  └─ 调用方去显式泛型参数（TS2558 保证全部暴露）
```

**17 条命令 → 返回类型对照**：

| 命令 | 返回 DTO | 归属 |
|---|---|---|
| get_usage_summary_v2 | UsageSummaryDto | adapter queries.rs |
| get_usage_capabilities_v2 | CapabilityReport | adapter capabilities.rs |
| get_usage_trends_v2 | Vec\<DailyTrendDto> | ccr-usage |
| get_usage_by_model_v2 | Vec\<ModelStatDto> | adapter queries.rs |
| get_usage_by_provider_v2 | Vec\<ProviderBreakdownDto> | ccr-usage |
| get_usage_by_project_v2 | Vec\<ProjectStatDto> | adapter queries.rs |
| get_usage_heatmap_v2 | HeatmapResponseDto | adapter queries.rs |
| get_usage_logs_v2 | PaginatedLogsDto | adapter queries.rs |
| get_usage_dashboard_v2 | UsageDashboardResponse | **新增**（替换 json!） |
| get_home_usage_overview_v2 | HomeUsageOverviewResponse | usage.rs（随迁 services） |
| ensure_session_index_v2 | StartSessionIndexJobResponse | usage.rs（随迁） |
| get_session_index_job_status_v2 | SessionIndexJobSnapshot | session_index_jobs.rs |
| start_usage_import_job_v2 | StartUsageImportJobResponse | usage.rs（随迁） |
| get_usage_import_job_status_v2 | UsageImportJobSnapshot | usage_jobs.rs |
| cancel_usage_import_job_v2 | 现返回 Value 的取消载荷 → 具名化 | usage.rs |
| import_usage_v2 | UsageImportResultV2 | usage.rs（随迁） |
| import_all_usage_v2 | ImportAllUsageResponse | usage.rs（随迁） |

**wire 不变论断**：Tauri 对返回值做 serde 序列化，与 `serde_json::to_value` 同构 → 前端运行时 shape 零变化，本试点无行为变更（唯一内部差异：dashboard 缓存命中走 from_value，失败显式报错，语义等价）。

## 3. 生成与漂移守卫

- 生成物目录 `ccr-ui/src/types/generated/usage/`，**入库**（reviewer 可见漂移 diff）。`.gitattributes` 标 `linguist-generated`；加入 eslint/prettier ignore；tsconfig 已覆盖 src/** → type-check 自动接入。
- 生成命令（ccr-ui/justfile 新 recipe `bindings`，根 justfile 委托 `tauri-bindings`）：
  1. 删除 `src/types/generated/usage/`（捕获孤儿文件）
  2. `cargo test -p ccr-usage --features ts export_bindings`（在根 workspace 跑）
  3. `cargo test --manifest-path src-tauri/Cargo.toml export_bindings`
- 守卫 recipe `bindings-check`（根委托 `tauri-bindings-check`）：跑生成后 `git diff --exit-code -- ccr-ui/src/types/generated`。
- `just ci` 在 frontend-check 前插入 bindings-check；GitHub workflow 若分步执行（ci.yml/frontend-ci.yml），相应加一步（实现时按现有 workflow 结构接）。
- 该守卫独立于 api-facade-boundary 手工门面 smoke（spec 明文要求分离，天然满足：目录、测试、recipe 全部独立）。

## 4. 测试策略

- **fixture 归属**：llmusage 只读 schema 的建表 DDL 已存在于 ccr-usage db.rs 测试内。提取 `ccr-usage` feature `test-fixtures` 暴露 `fixtures::*`（最小 schema + 种子行 helper），本 crate 测试与 src-tauri dev-dependency 共用，防 DDL 双份漂移。若实现时发现 DDL 高度分散、提取即重写，则降级：只提取"最小可查询 schema"一个函数，其余留原地。
- **src-tauri services 单测**（无 Tauri app）：AppPaths{root_dir,db_path} 指向 TempDir fixture DB → LlmusageRuntime/Dashboard 直连。覆盖 7 个纯查询 service + dashboard 组装 + home overview 的 usage 侧拼装（session 旁路若 ccr-db pool fixture 成本高，拆纯函数注入数据测试）。
- **缓存路径**：UsageDashboardResponse to_value→from_value 往返测试。
- **前端**：type-check 为主门禁 + 既有 smoke 全量 + facade smoke。调用方去泛型后由编译器验证。
- **计数守卫**：命令零增删，handler_registry 312/320 不动（实测基线；设计初稿的 309/317 为过期数字，spec 已随本任务勘误）。

## 5. 兼容性与风险

| 风险 | 处置 |
|---|---|
| ts-rs i64→bigint 默认 | 字段级 `#[ts(as = "f64")]`（§1 实现期修正：`TS_RS_LARGE_INT` 是未发布 v12 特性，v11 环境变量方案不生效） |
| 生成格式跨版本漂移 | 依赖写 `ts-rs = "11"`；生成物入库使任何漂移在 diff 可见 |
| serde(alias) camelCase 入参兼容 | alias 仅影响反序列化，生成 TS 用 canonical snake_case——与现 wrapper 发送的 key 一致 |
| 同名类型导出冲突（HomeOverview* 双份） | 只对 wire 实际类型加 derive；ccr-usage 同名版不导出 |
| tauri-test 顺带跑 export tests 重写生成物 | 幂等重写；源未变则无 diff，反而是天然守卫 |
| 手写镜像与真实 wire 有出入（如 ModelStat 可选字段） | 正是要暴露的漂移：以生成类型为准修调用方，逐个确认非行为回归 |
| 前端 UnknownRecord 强转残留 | 试点 domain 内清零；其他 domain 不动（新旧并存是预期） |

回滚：各阶段独立提交；生成链路整体可回退（删 derive/依赖/目录/recipe 四点）。

## 6. 交付外围（PRD item 4/5）

- **重叠盘点**：stats(统计 3)/统计扩展(7)/claude_observer(9) × usage_v2(17) 吸收矩阵，按前端调用方核对，产出 `research/usage-family-overlap.md`（只标记，不删除）。
- **推广评估**：成本/收益/工程化问题写 task notes；沉淀 spec 新文档 `.trellis/spec/ccr/backend/typed-ipc-bindings.md`（生成契约、TS_RS_LARGE_INT、目录约定、守卫命令、扩展新 domain 的步骤），index 联动更新；推广/放弃结论按 AC 记录在案。
