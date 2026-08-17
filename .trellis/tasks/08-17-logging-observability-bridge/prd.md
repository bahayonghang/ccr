# Monitoring 桥接与热路径结构化

父任务：`08-17-logging-system-optimization`。依赖 `logging-rust-init`。

## Goal

Desktop worker 消费有界队列，把 `ccr_*` warn/error 送进 Monitoring。2 秒内可持久化的 error 落盘。退出先 flush。Dashboard 排除 `runtime`。热路径带 `corr`。文档与 spec 与真实文件名一致。

## Requirements

- R1. setup 取 receiver，启动单一 worker，TLS 重入。
- R2. persist 规则按父设计。flush_threshold=20，interval=2s。
- R3. Exit 先 flush（500ms 超时）再 `database::shutdown`。
- R4. emit/flush 失败不打可入桥的 tracing。
- R5. Dashboard 排除 `frontend`+`runtime`，更新 spec 与 smoke。
- R6. 热路径字段化。
- R7. 文档与 spec。

## Acceptance Criteria

- [ ] AC1. `ccr_core` error 出现在 Monitoring，带 `process_id`。
- [ ] AC2. `runtime.warn` 不进 SQLite；`runtime.error` 与 `frontend.warn` 进 SQLite。
- [ ] AC3. Dashboard 计数忽略诊断频道。
- [ ] AC4. 桥失败路径的测试：再打 tracing 不增加队列。
- [ ] AC5. 2.1s 间隔的 persistable error 出现在 SQLite。
- [ ] AC6. 热路径含 `corr`。
- [ ] AC7. 文档为 `ccr.log.YYYY-MM-DD`。
- [ ] AC8. 受影响 crate 测试 + `just frontend-check-quick` + docs 构建或 `just frontend-check` 中的 docs 部分。

## Out of Scope

全仓插值、request id 传播、官方插件。

## Ordering

A 必须完成。B 建议完成。
