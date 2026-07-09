# Tauri IPC seam 类型化 — claude_observer 域（第二个 typed-ipc 域）

## 背景

usage V2 试点（07-03-arch-typed-ipc）已建立完整机制：ts-rs 生成绑定入库 + `just tauri-bindings-check` 漂移守卫 + 服务层抽取 + 前端去手写镜像。推广评估（research/promotion-evaluation.md）判定按域分批推广，首推 claude_observer 域（9 命令，前端富类型稳定、store/组件活跃消费）。契约见 `.trellis/spec/ccr/backend/typed-ipc-bindings.md` "Extending to a new domain" 清单。

## 现状（侦查结论）

- 后端 `commands/claude_observer.rs`（567 行）9 条命令**已全部是具名 DTO 签名**，无 `Result<Value>` 擦除点——本域无"去擦除"工作，核心是生成链路接入 + 服务抽取。
- wire 类型分布三处：命令文件本地 5 个（InsightDto/CacheStatsDto/DailyPoint/BreakdownRow/SessionRow）、`claude_observer/subscription.rs` 1 个（SubscriptionDto）、`ccr_db::claude_tool_calls_repo` 2 个（HeatmapCell/TopToolRow，DB 仓储层类型直接上 wire）。
- 前端 `types/claudeObserver.ts`（87 行）是手写镜像；9 个 wrapper 住在**冻结门面 tauri.ts**（allowlist 豁免中），不在 domains/——与 api-facade-boundary 契约的目标态不符。
- 业务逻辑在命令的 spawn_blocking 闭包内（insight 是多步组装），无法脱离 Tauri app 单测。

## 需求

1. observer 域 8 个 wire DTO 全部接入 ts-rs 生成链路，生成物落 `ccr-ui/src/types/generated/claude_observer/` 并入库，进入既有 `just tauri-bindings` / `tauri-bindings-check` 守卫。
2. 业务逻辑抽取为 State-free 服务函数（`services/claude_observer.rs`），命令层只留计时/State 提取/spawn_blocking 薄壳；服务函数可用 fixture DB 单测（llmusage 侧复用 `ccr_usage::fixtures`，ccr-db 侧用内存池 + 迁移）。
3. ccr-db 仓储层类型不得携带前端绑定 concern：wire DTO 归服务层所有（对 HeatmapCell/TopToolRow 做服务层 DTO 映射），ccr-db 不引入 ts-rs 依赖。
4. 前端 wrapper 从 tauri.ts 迁出到 `src/api/domains/claudeObserver.ts`，tauri.ts allowlist 缩减 9 条；`claudeObserver` 具名导出保持（store 导入路径 `@/api` 不变）。
5. `types/claudeObserver.ts` 改造为 re-export shim（旧名保留），手写镜像删除。
6. wire shape 零变化：本任务是纯内部重构 + 类型化，前端运行时行为不变。

## 非目标

- 不新增/删除命令（registry 计数 312/320 不变）。
- 不动 `claude_observer:updated` 事件面（事件 payload 类型化是独立候选，spec 已记录）。
- 不做 scanner/jsonl/pricing 摄取链路重构。
- 不处理 usage-family-absorb（stats 命令下线）——独立候选任务。

## 验收标准（AC）

- AC1: `rg 'serde_json::to_value|Result<Value' ccr-ui/src-tauri/src/commands/claude_observer.rs` 零命中；9 条命令体均为薄壳（State 提取 + spawn_blocking(service) + join 错误映射）。
- AC2: `ccr-ui/src/types/generated/claude_observer/` 含 8 个生成类型文件且全部入库，无 `bigint`；`just tauri-bindings-check` 绿。
- AC3: `services/claude_observer.rs` 单测覆盖 7 个查询服务函数（订阅 get/set 已有 subscription.rs 测试），`cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml services -- --test-threads=1` 全绿。
- AC4: `types/claudeObserver.ts` 无手写 interface 镜像（只余 re-export）；`domains/claudeObserver.ts` 返回类型全部来自生成绑定。
- AC5: facade smoke allowlist 中 `claude_observer_*` 9 条移除且测试绿；`bun run type-check`、`just frontend-check-quick` 全绿。
- AC6: registry 计数 312/320 不变；`cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml -- --test-threads=1` 全绿；spec `typed-ipc-bindings.md` 回写 observer 域扩展记录。
