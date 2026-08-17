# Implement: 前端 logger 与 IPC

## Order

1. 确认 A 已提供 `redact_log_text` / `redact_log_value`。
2. `sanitize_frontend_log`（Tauri 可测纯函数）。
3. `append_frontend_logs` 使用 sanitize；批次 32。
4. `logger.ts`：识别、限额、session_id、队列 100、重试 3。
5. `tests/logger.smoke.test.ts` 读 `crates/ccr-core/testdata/log_redaction_vectors.json`。
6. 需要时 `just tauri-bindings`。

## Validation

```text
cargo test -p ccr-core -- --test-threads=1
cargo test -p ccr-desktop sanitize_frontend -- --test-threads=1
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/logger.smoke.test.ts
just frontend-check-quick
just tauri-bindings-check
```

改了 DTO 则先 `just tauri-bindings`。

## Risky files

`logger.ts`、`commands/system.rs`。
