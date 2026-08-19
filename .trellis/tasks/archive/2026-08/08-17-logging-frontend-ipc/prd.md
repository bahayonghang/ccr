# 前端 logger 与 IPC 加固

父任务：`08-17-logging-system-optimization`。

## Goal

`logger.ts` 与 `append_frontend_logs` 按硬限额与同一识别规则处理不可信输入。同一会话 IPC 条目共享 `session_id`。

## Requirements

- R1. 前端识别规则与向量文件对齐。
- R2. 硬限额见父设计 IPC 表。超限只截断，不拒绝整条 warn/error。
- R3. `Error.stack` 丢弃。
- R4. `session_id` 写入 history 与 DTO `correlation_id`。
- R5. 发送队列上限 100，重试最多 3 次后丢弃。
- R6. 服务端调用 A 的 `redact_*` + `sanitize_frontend_log`。直调 sanitize 的 Rust 测试。
- R7. DTO 若增字段：`just tauri-bindings`。

## Acceptance Criteria

- [ ] AC1. 向量中的敏感样例在 history 与待发送载荷中无原文。
- [ ] AC2. `Error` 载荷无 `stack`。
- [ ] AC3. 超大 fields → `{ truncated: true }`；超长 message 截断到 2000。
- [ ] AC4. 33 条一批只处理 32 条，command `Ok`。
- [ ] AC5. 同一会话两条 error 的 `correlation_id` 相同。
- [ ] AC6. 模拟 command 缺失时最多重试 3 次，队列不回涨。
- [ ] AC7. `just frontend-check-quick`；若改 DTO：`just tauri-bindings` 后提交生成文件。

## Out of Scope

文件权限、worker、热路径、文档。

## Ordering

**依赖 `logging-rust-init`。** 服务端不得复制一份识别算法。
