# Tauri IPC seam 类型化 — 执行计划

前置：design.md 已定型（ts-rs v11 / usage V2 组 17 命令 / 生成物入库 + git diff 守卫）。
每阶段独立可提交、独立可回滚；出现与设计冲突的事实先回改 design.md 再继续。

## 阶段 1：依赖与生成链路脚手架

- [x] ~~`.cargo/config.toml` 增 `[env] TS_RS_LARGE_INT = "number"`~~ 实现期修正：v11 无该特性，改用字段级 `#[ts(as = "f64")]`（design §1）；config.toml 未引入 env 项
- [x] `crates/ccr-usage/Cargo.toml`：`ts-rs = { version = "11", optional = true }` + feature `ts`
- [x] `ccr-ui/src-tauri/Cargo.toml`：`ts-rs = "11"`
- [x] 选 1 个类型（UsageSummaryDto）先行打通 derive → export → 落盘路径正确
- 验证：`cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml export_bindings`
  生成 `ccr-ui/src/types/generated/usage/UsageSummaryDto.ts`，字段全为 `number`（无 bigint）
- 验证：`cargo check --workspace`（feature 默认关，CLI 依赖图零变化；llmusage_no_crate_guard 不受影响）

## 阶段 2：ccr-usage DTO 导出（4 类型）

- [x] DailyTrendDto / ProviderBreakdownDto / SourceBreakdownDto / UsageRecordDto
      加 `cfg_attr(feature = "ts", derive(ts_rs::TS))` + export_to；同名 HomeOverview* 不导出
- [x] 补 Deserialize（dashboard 缓存 from_value 路径需要，见阶段 3）
- 验证：`cargo test -p ccr-usage --features ts export_bindings -- --test-threads=1` 生成 4 文件
- 验证：`cargo test -p ccr-usage -- --test-threads=1` 全绿（无 feature 时零影响）

## 阶段 3：src-tauri 服务抽取 + 17 命令类型化

- [x] 新建 `src/services/mod.rs` + `src/services/usage.rs`；wire DTO 与 UsageLogsQuery 随迁，
      `commands/usage.rs` 保 `pub use` 兼容（先 grep 引用面：events.rs / stores 消费点）
- [x] 纯查询 7 条的 spawn_blocking 闭包体平移为 State-free service 函数
- [x] `UsageDashboardResponse` 结构体替换 `json!()`；缓存命中 `serde_json::from_value`
- [x] home overview 拼装逻辑迁 service（session 旁路依赖 &DbPool 传参）
- [x] 17 条命令签名 → `Result<具名DTO, String>`，删除末尾 to_value；job/import 命令原地具名化
- [x] 相关类型补 derive(TS)+export_to（queries.rs 5 个、capabilities.rs、usage_jobs / session_index_jobs 快照与枚举、usage.rs 随迁 wire 类型、UsageLogsQuery/UsageLogsMode）
- [x] 评估 `run_usage_query` 计时样板收敛 helper：仅当 6+ 命令形状真正一致才做，否则放弃（记录进推广评估）
- 验证：`cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml -- --test-threads=1` 全绿
- 验证：`cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::handler_registry -- --nocapture` 计数 312/320 不变（实测基线，设计初稿 309/317 为过期数字）
- 验证：`rg 'Result<Value, String>' ccr-ui/src-tauri/src/commands/usage.rs` 零命中（AC1）

## 阶段 4：service 单元测试（无 Tauri app）

- [x] ccr-usage 提取 feature `test-fixtures`（最小 schema + 种子 helper）；DDL 分散则降级为单函数
- [x] src-tauri dev-dependencies 启用之；TempDir + AppPaths 构造 fixture DB
- [x] 覆盖：7 纯查询 service、dashboard 组装（含 provider 缺列降级分支）、home overview usage 侧、
      UsageDashboardResponse to_value/from_value 往返
- 验证：`cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml services -- --test-threads=1` 全绿（AC3）

## 阶段 5：前端接生成类型

- [x] 全量生成；`.gitattributes` linguist-generated、eslint/prettier ignore 就位
- [x] `types/usage.ts` → re-export shim（旧名 alias 生成类型；视图专用类型保留）
- [x] `domains/stats.ts` 17 个 V2 wrapper 去泛型、具名返回；UsageLogsQuery 改 re-export
- [x] 4 个调用方去显式泛型参数；shape 出入以生成类型为准修正并逐个确认非行为回归
- 验证：`cd ccr-ui && bun run type-check`；`rg 'UnknownRecord' ccr-ui/src/api/domains/stats.ts` 的 V2 区零命中（AC2）
- 验证：`bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts` + `just frontend-check-quick`（AC5）

## 阶段 6：漂移守卫接线

- [x] ccr-ui/justfile：`bindings`（删目录→两段 export 测试）与 `bindings-check`（生成→git diff --exit-code）
- [x] 根 justfile 委托 `tauri-bindings` / `tauri-bindings-check`；`just ci` 于 frontend-check 前插入
- [x] ~~按 ci.yml / frontend-ci.yml 现状接 workflow 步骤~~ 决策：GitHub 质量门现状完全不含 src-tauri 构建面（tauri 仅在 release.yml），接线需先引入 src-tauri CI 覆盖，属结构性变更超出本任务；守卫落在 `just ci`（仓库验收重门），workflow 接线随未来 src-tauri CI 覆盖任务一并处理
- 验证：~~手改一个生成文件字段 → bindings-check 红；恢复 → 绿（AC4）~~ 已证：绿（入库态 exit 0）→ 改 UsageSummaryDto.ts 字段并 stage → 红（exit 1，列出 `MM .../UsageSummaryDto.ts`）→ 恢复复绿。注：未入库的手改会被重生成静默修复（canonical 胜出），红条件是"入库绑定 ≠ canonical"，与守卫真实语义一致

## 阶段 7：盘点与推广评估

- [x] `research/usage-family-overlap.md`：stats/统计扩展/claude_observer × usage_v2 吸收矩阵（只标记）
- [x] 推广评估写 task notes：成本（依赖/配置/迁移工时）、收益（漂移变编译错、去手写镜像行数）、
      工程化问题（命令名守卫缺口、export test 顺带写盘）、是否为其余 29 domain 立后续任务（AC6）
- [x] trellis-update-spec：新增 `.trellis/spec/ccr/backend/typed-ipc-bindings.md` + index 更新；
      勘误 ccr-ui CLAUDE.md 中 tauri.ts "141+ invoke 包装" 等过时表述若与本次触面相关

## 阶段 8：收口

- [x] 最后一轮全域检查：`just version-check && just fmt-check && just lint-strict && just test`
- [x] `just frontend-check-quick` + src-tauri 全量测试（235 通过 + registry 3/3 + trellis-check 独立复核 6/6 AC PASS）
- [x] 按 concern 拆分提交（脚手架 / Rust 类型化 / 测试 / 前端 / 守卫 / spec）：
      4451e219 / f68d6026 / f16c3d48 / 1a131cea / cc571935 / 658265aa
- [x] task.py archive + journal

## 回滚点

- 阶段 1-2 独立可弃（删依赖与 derive）
- 阶段 3 以 `pub use` 保路径，任何外部引用断裂即刻可见于编译
- 阶段 5 前端 shim 保旧名，单文件粒度可回退
- 守卫接线独立于功能，可单独摘除
