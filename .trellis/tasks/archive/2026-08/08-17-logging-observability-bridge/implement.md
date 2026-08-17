# Implement: Monitoring 桥接

## Order

1. `ccr-desktop` worker + 排除表 + 失败计数。
2. `should_persist`、去掉每条 `force_flush`、2s interval、Exit flush。
3. `log_persistence` 失败日志改为不含敏感内容且 target 在排除表内，或只计数。
4. Dashboard + spec + smoke。
5. 热路径：`ccr-cli` `ccr-codex` `ccr-checkin` `ccr-config` `ccr-desktop`。
6. `docs/examples/troubleshooting.md`、`ccr-ui/README_CN.md`、`.trellis/spec`。

## Validation

```text
cargo test -p ccr-core -- --test-threads=1
cargo test -p ccr-db -- --test-threads=1
cargo test -p ccr-cli -- --test-threads=1
cargo test -p ccr-codex -- --test-threads=1
cargo test -p ccr-checkin -- --test-threads=1
cargo test -p ccr-config -- --test-threads=1
cargo test -p ccr-desktop -- --test-threads=1
just frontend-check-quick
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/dashboard-presentation.smoke.test.ts
cd docs && npm run build
just lint-strict
```

父任务收口再 `just ci`。

## Risky files

`monitoring.rs`、`main.rs` 退出、`log_persistence.rs`、各平台 apply。
