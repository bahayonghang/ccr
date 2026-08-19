# Rust 日志初始化与文件脱敏

父任务：`08-17-logging-system-optimization`。以父 `design.md` 为准。

## Goal

`init_logger` / `init_file_only_logger` 提供：合法过滤器、UTC 日切文件 `ccr.log.YYYY-MM-DD` 且每次创建 `0o600`、识别层、`process_id`、有界入队 API。本任务不启动 Tauri worker。

## Requirements

- R1. 过滤器按父设计 Filter。
- R2. `SecureDailyWriter`：目录 `0o700`，`ccr.log.YYYY-MM-DD` 创建与日切后 `0o600`。创建目录失败 → 无文件层。当天 chmod 失败 → 停写文件。
- R3. `redact_log_text` / `redact_log_value` + `testdata/log_redaction_vectors.json`。不改 `mask_sensitive`。
- R4. `current_log_correlation_id()` = 初始化 UUID。
- R5. `try_enqueue_bridged_log` / `take_bridged_log_receiver`。Layer 条件与排除表按父设计。满队列不 tracing。
- R6. TUI 仍只写文件。init 签名不变。

## Acceptance Criteria

- [ ] AC1. 非法 directive 回退默认。
- [ ] AC2. Unix：tempdir 中当天文件与轮转后新文件 `0o600`，目录 `0o700`。
- [ ] AC3. 向量文件全部通过；普通句子 `must_contain` 仍在。
- [ ] AC4. `current_log_correlation_id()` init 后稳定非空。
- [ ] AC5. 未 `take` receiver 时入队可 Accepted，无消费者也不 panic。重入与排除 target 不入队。
- [ ] AC6. `cargo test -p ccr-core -- --test-threads=1`；`just fmt-check`。

## Out of Scope

Tauri worker、Dashboard、`logger.ts`、文档、热路径插值。

## Ordering

无前置。
