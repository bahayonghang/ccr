# typed-ipc 试点推广评估（PRD item 4：决策记录）

评估时点：2026-07-05，usage V2 域（17 命令）试点完成后。

## 结论

**推广，但按域分批、按需立项**：机制已被试点证明可行且成本可控，为其余 domain 立后续任务时优先级按"前端消费强度 × 手写镜像类型规模"排序。首推候选：claude_observer（9 命令，前端富类型已稳定、store/组件活跃）；codex 域体量最大（63 命令 / 76K）应最后做、且先拆子域。**不建议**一次性全仓迁移任务——新旧并存是低成本稳态，registry 30 个分组可独立迁移。

## 成本（实测）

- 一次性脚手架（已付，后续域零成本）：ts-rs 依赖 ×2、ccr-usage feature `ts`/`test-fixtures`、justfile recipes、ci 接线、eslint/gitattributes、spec 文档。
- 每域边际成本（usage 实测）：Rust 侧 DTO derive + 标注 + 命令签名改写 + service 抽取（usage 域约 1.3K 行迁移 + 17 签名）；TS 侧 shim + wrapper 具名化 + 调用方去泛型（4 文件 21 处）+ smoke fixture 补全（9 个测试文件——**这是最大意外成本项**，宽松手写类型时代的部分字段字面量全部要补全）。
- 认知成本：i64→`ts(as = "f64")` 字段级标注（usage 域约 70 处）；输入型/skip 字段的 `ts(optional)` 规则。规则已沉淀 spec，可复制。

## 收益（实测）

- shape 漂移变编译错：type-check 一次暴露 9 个测试文件里的镜像类型漂移 + 2 个组件的不健全收窄（string 当 UsagePlatform 用）+ dashboard `heatmap` 可空性被手写类型隐藏的事实。全部是真实缺陷面。
- 删除手写镜像 ~340 行（types/usage.ts 400 行 → 60 行 shim）；`<T = UnknownRecord>` 在试点域清零（调用方注入镜像类型的通道关闭）。
- 无 Tauri app 单测成为可能：12 个 service 测试 + fixture 库沉淀（ccr_usage::fixtures 可复用于任何读 llmusage 投影的域）。
- usage.rs 从 2779 行减到 ~1447 行，业务逻辑集中 services/（deep module 目标达成）。

## 工程化问题（如实记录）

1. **ts-rs 已发布版 i64/u64 硬编码 bigint**：`TS_RS_LARGE_INT` 是未发布 main 特性。当前用字段级 `as = "f64"`（~70 处标注噪音）；v12 发布后应整体切环境变量并删标注（spec 已写升级路径）。
2. **命令名 ↔ wrapper invoke 串仍是手工映射**：ts-rs 只保 payload shape，命令名拼错仍是运行时错误。后续可从 handler_registry 导出命令名清单给 TS 侧断言（独立小任务，非阻塞）。
3. **export tests 顺带写盘**：src-tauri 任何 `cargo test` 都会重写生成目录（幂等）。可接受，且构成天然守卫；但生成物未提交时本地测试会持续出现未跟踪文件。
4. **serde(alias) 宏警告**：ts-rs 解析不了 alias 属性会打印 stderr 提示（无害，语义正确）。
5. **事件 payload 不在覆盖范围**：`app_handle.emit` 的 payload（job 快照事件等恰好复用了命令 DTO，但 UsageSnapshotUpdatedPayload 等纯事件类型仍手写）。事件面类型化是独立候选。

## 后续任务建议（吸收矩阵见 research/usage-family-overlap.md）

1. `arch-typed-ipc-observer`：claude_observer 域类型化（收益/成本比最高的下一域）。
2. `usage-family-absorb`：stats 10 条命令吸收/下线（9 条零调用 + get_provider_usage 迁移 ConfigsView）→ 顺带清理 CostTracker 链路。
3. `typed-ipc-command-name-guard`：registry → TS 命令名清单守卫（可选）。
4. codex 域：待其自身拆分后分批。
