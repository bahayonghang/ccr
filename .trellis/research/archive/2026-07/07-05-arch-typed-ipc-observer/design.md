# claude_observer 域类型化 — 技术设计

## 0. 侦查结论（约束设计的事实）

- 9 条命令已全部是具名 DTO 签名，无 `Result<Value>`/`to_value` 擦除点——工作面是生成链路 + 服务抽取 + 前端迁移，不是去擦除。
- wire 类型三处分布：命令文件 5 个（InsightDto/CacheStatsDto/DailyPoint/BreakdownRow/SessionRow）、subscription.rs 1 个（SubscriptionDto，已有 Deserialize+PartialEq+单测）、ccr-db 仓储 2 个（HeatmapCell/TopToolRow 直接上 wire）。
- 前端 wrapper 在冻结门面 tauri.ts 的 allowlist 豁免区（9 条）；`ClaudeObserver*` 别名类型在 tauri.ts 之外零消费（rg 验证），可随迁移删除。
- store 导入 `import { claudeObserver } from '@/api'`；组件/store 的类型导入全部走 `@/types/claudeObserver`。
- 无缓存层、无事件发射复用这些 DTO（`claude_observer:updated` 事件 P4 未发）——无 from_value 回读路径，DTO 不需要补 Deserialize（SubscriptionDto 已有，保留）。

## 1. 分层与归属

```
ccr-ui/src-tauri
  ├─ src/services/claude_observer.rs   [新增]
  │   wire DTO 随迁：InsightDto / CacheStatsDto / DailyPoint / BreakdownRow / SessionRow
  │   新增 wire DTO：HeatmapCell / TopToolRow（服务层映射 ccr-db 仓储同名类型，
  │     字段一一对应，wire shape 与今日相同；ccr-db 不引入 ts-rs concern）
  │   全部加 derive(TS) + export_to = "../../src/types/generated/claude_observer/"
  │   State-free 同步服务函数（spawn_blocking 内运行）：
  │     insight(&LlmusageRuntime, &DbPool) -> Result<InsightDto, String>
  │     daily_trend(&LlmusageRuntime, days: Option<i64>) -> Result<Vec<DailyPoint>, String>
  │     cost_breakdown(&LlmusageRuntime, dim: &str, days/limit: Option<i64>) -> Result<Vec<BreakdownRow>, String>
  │     cache_stats(&LlmusageRuntime) -> Result<CacheStatsDto, String>
  │     top_sessions(&DbPool, limit: Option<i64>, by: Option<&str>) -> Result<Vec<SessionRow>, String>
  │     tool_heatmap(&DbPool, days: Option<i64>) -> Result<Vec<HeatmapCell>, String>
  │     top_tools(&DbPool, days/limit: Option<i64>) -> Result<Vec<TopToolRow>, String>
  │   默认值/clamp 随闭包体一并迁入服务函数（可测）；日期窗口 helper（today/month_window）随迁
  ├─ src/claude_observer/subscription.rs：SubscriptionDto 加 derive(TS)+export_to（同目录）
  │   订阅 get/set 本就是 State-free 函数，不再包一层服务——命令直接调用（现状保留）
  └─ src/commands/claude_observer.rs：9 条命令改薄壳
      （debug 日志 + State 提取 + spawn_blocking(service) + join 错误映射）
      DTO 路径以 `pub use crate::services::claude_observer::{...}` 保同一性

ccr-ui/src (前端)
  ├─ src/types/generated/claude_observer/   [新增，8 个生成文件入库]
  ├─ src/types/claudeObserver.ts → re-export shim（8 个旧名全保，注释指向生成目录）
  ├─ src/api/domains/claudeObserver.ts      [新增]
  │   `export const claudeObserver = {...}` 9 个方法原样迁入，返回类型改生成绑定
  ├─ src/api/index.ts：`export { claudeObserver } from './domains/claudeObserver'`
  ├─ src/api/tauri.ts：删除 Claude Observer 区块（import/9 wrapper/ClaudeObserver* 别名导出）
  └─ tests/api-facade-boundary.smoke.test.ts：allowlist 删 9 条 claude_observer_*
```

## 2. 关键决策

| 决策 | 理由 |
|---|---|
| HeatmapCell/TopToolRow 在服务层做 wire DTO 映射，而非给 ccr-db 加 feature `ts` | ccr-db 是 DB 仓储层，不应携带前端绑定 concern；映射只有 2 个小结构体（3+3 字段），成本远低于给 ccr-db 加 feature/export 测试/bindings recipe 第三段。生成物名与前端旧名一致（HeatmapCell/TopToolRow），shim 零别名 |
| 服务函数入参 `&DbPool` 而非 `&Connection` | 与 services/usage.rs 既有签名约定一致；pool.get() 错误文案（"DB pool error: {e}"）留在服务内，保证错误方向零漂移 |
| subscription get/set 不再包服务层 | 已是 State-free + 有单测，再包一层是纯转发（07-03 刚删过 pool.rs 转发层，不重蹈） |
| 生成目录 `claude_observer/`（snake_case） | 与命令前缀一致；usage 域先例是按 registry 组名小写 |
| justfile bindings recipe 不改 | observer 全部 8 个导出类型都在 src-tauri crate，既有 `cargo test --manifest-path src-tauri/Cargo.toml export_bindings` 已覆盖；recipe 删除的是整个 generated/ 目录，天然含新子目录 |
| tauri.ts 观察者区块整体删除（而非保留转发） | facade 契约默认修复方向就是迁 domains/；`claudeObserver` 具名导出经 index.ts 显式 re-export 保住 store 导入路径；`ClaudeObserver*` 别名零消费直接删 |

## 3. ts-rs 标注清单（契约 §3）

64 位整数字段全部 `#[ts(as = "f64")]`（Option 无、map 无）：

- InsightDto: today_tokens / month_tokens / total_sessions / total_projects（roi 是 Option<f64> 无需标注）
- CacheStatsDto: 4 个 total_*_tokens
- DailyPoint: input/output/cache_read/cache_write_tokens
- BreakdownRow: tokens / count
- SessionRow: tokens / tool_call_count
- HeatmapCell: dow / hour / count
- TopToolRow: call_count
- SubscriptionDto: 无（String×2 + f64）

输入侧参数全是标量（days/limit/dim/by/mode/plan/monthly_usd），不构成输入 DTO，无 `ts(optional)` 面。

## 4. 测试策略

复用 services/usage.rs `service_tests` 模式（fixture 投影库 + temp ccr-db pool + local_noon_utc 防时区漂移）：

- llmusage 侧（seed_bucket 当日/窗口内数据）：daily_trend（默认 30 天 + clamp 边界）、cost_breakdown（project/model 两维 + 未知 dim 报错 + limit 截断）、cache_stats（hit_rate 与 4 token 汇总）。
- ccr-db 侧（直接 INSERT claude_tool_calls 种子行）：tool_heatmap、top_tools、top_sessions（by=cost 与 by=calls 两种排序）。
- 双源：insight（订阅默认值 → roi=None；subscription::set 后 mode=subscription → roi=Some(month/monthly)；total_sessions 来自 distinct session_id）。
- 前端：type-check + facade smoke（allowlist 缩减后）+ 既有 smoke 全量。

## 5. 兼容性与风险

| 风险 | 处置 |
|---|---|
| store `import { claudeObserver } from '@/api'` 断裂 | index.ts 显式 `export { claudeObserver } from './domains/claudeObserver'`；type-check 兜底 |
| 生成物与手写镜像有隐性出入（如 roi 可空性） | 以生成类型为准修消费方；roi `number | null` 与手写一致，SessionRow 三个 Option 字段手写已是 `| null`，预期零修 |
| services 模块 DTO 随迁导致 commands 内部引用断裂 | `pub use` 保路径；`cargo check` 全量验证 |
| 仓储层与服务层同名类型（HeatmapCell/TopToolRow）混淆 | services 内以 `claude_tool_calls_repo::` 限定引用仓储版，不 use 进作用域 |
| registry 计数漂移 | 命令零增删，312/320 冻结测试守卫 |

回滚：Rust 侧（服务抽取+derive）与前端侧（迁移+shim+allowlist）各自独立提交，单侧可回退；生成目录随 derive 回退删除即可。

## 6. 交付外围

- spec `typed-ipc-bindings.md`：Scope 补 claude_observer 域（services/claude_observer.rs、generated/claude_observer/）、"仓储类型不直接上 wire，服务层映射"决策入 Contracts。
- 顺带勘误面（若触及）：无——facade spec 的 allowlist 缩减属常规测试更新，不需要 spec 改动。
