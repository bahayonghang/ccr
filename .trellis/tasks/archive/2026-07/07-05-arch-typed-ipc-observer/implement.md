# claude_observer 域类型化 — 执行计划

前置：design.md 已定型（服务层映射 ccr-db 类型 / 生成目录 claude_observer/ / tauri.ts 区块整体迁出）。
每阶段独立可提交、独立可回滚；与设计冲突的事实先回改 design.md 再继续。

## 阶段 1：服务抽取 + DTO 随迁 + TS derive（Rust 侧一体）

- [x] 新建 `src/services/claude_observer.rs`（services/mod.rs 注册）：
      5 个 wire DTO 随迁 + 新增 HeatmapCell/TopToolRow 映射 DTO，全部 derive(TS) + `#[ts(as = "f64")]` 按 design §3 清单
- [x] 7 个 State-free 服务函数（默认值/clamp/日期窗口 helper 随迁）；仓储类型以 `claude_tool_calls_repo::` 限定引用
- [x] `subscription.rs` SubscriptionDto 加 derive(TS) + export_to
- [x] `commands/claude_observer.rs` 9 条命令改薄壳；`pub use crate::services::claude_observer::{...}` 保 DTO 路径
- 验证：`cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml export_bindings` 生成 8 文件、无 bigint
- 验证：`cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml -- --test-threads=1` 全绿
- 验证：registry 计数测试 312/320 不变；`rg 'serde_json::to_value|Result<Value' ccr-ui/src-tauri/src/commands/claude_observer.rs` 零命中（AC1）

## 阶段 2：服务单元测试

- [x] `service_tests` 模块复用 usage 模式：fixture 投影库（seed_bucket）+ temp ccr-db pool（INSERT claude_tool_calls 种子）
- [x] 覆盖 design §4 清单：3 个 llmusage 侧、3 个 ccr-db 侧、insight 双源（roi None/Some 两分支）
- 验证：`cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml services -- --test-threads=1` 全绿（AC3）

## 阶段 3：前端迁移 + 生成物入库

- [x] `just tauri-bindings` 全量生成，`src/types/generated/claude_observer/` 8 文件入库
- [x] 新建 `src/api/domains/claudeObserver.ts`（9 方法 + 生成类型返回）；index.ts 显式 re-export `claudeObserver`
- [x] tauri.ts 删 Claude Observer 区块；smoke allowlist 删 9 条
- [x] `types/claudeObserver.ts` → re-export shim（8 旧名）
- 验证：`cd ccr-ui && bun run type-check`；`bun run test:smoke -- tests/api-facade-boundary.smoke.test.ts`（AC4/AC5）
- 验证：`just tauri-bindings-check` 绿（AC2）；`just frontend-check-quick`

## 阶段 4：收口

- [x] 最后一轮全域检查：`just version-check && just fmt-check && just lint-strict && just test`
- [x] spec 回写 `typed-ipc-bindings.md`（observer 域 + 仓储类型映射决策）（AC6）
- [x] 按 concern 拆分提交（Rust 服务化+derive / 测试 / 前端迁移+生成物 / spec）
- [x] task.py archive + journal

## 回滚点

- 阶段 1 整体可弃（删 services/claude_observer.rs + 还原命令文件 + 删 subscription derive）
- 阶段 3 前端与 Rust 独立；shim 保旧名，单文件粒度可回退
- 生成目录随 derive 回退删除，bindings-check 自动回到只含 usage 的状态
