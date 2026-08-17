# Implement: Rust 日志初始化

## Order

1. `crates/ccr-core/testdata/log_redaction_vectors.json` + `src/log_redact.rs`（或 `core/log_redact.rs`）+ 导出。
2. 改 `resolve_log_filter` 为 `try_new`。
3. `SecureDailyWriter` 替换直接 `RollingFileAppender`。
4. fmt 字段走识别层。
5. `PROCESS_ID`、队列、Layer。
6. 测试：过滤器、权限/轮转、向量、入队/重入。

## Validation

```text
cargo test -p ccr-core -- --test-threads=1
just fmt-check
just lint-strict
```

## Risky files

`logging.rs`：文件层失败不得影响 stdout / TUI 文件-only 选择。
