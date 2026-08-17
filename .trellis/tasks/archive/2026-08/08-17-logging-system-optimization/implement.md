# Implement: 日志系统安全与可观测优化

父任务不改产品代码，只做集成验收。顺序固定，B 不可再与 A 并行实施 Rust 部分。

## Order

1. **`logging-rust-init`**  
   识别层 + 向量文件、过滤器、`SecureDailyWriter`、`process_id`、`try_enqueue` / receiver。无 Tauri worker。

2. **`logging-frontend-ipc`**  
   依赖 A 已合并的 `redact_log_text` / `redact_log_value`。`logger.ts` 对齐向量。`sanitize_frontend_log` + 硬限额。`just tauri-bindings`。禁止无限回队。

3. **`logging-observability-bridge`**  
   依赖 A 的队列。启动 worker、2s flush、退出 flush、Dashboard、热路径、文档、spec。验证含 `ccr-cli` `ccr-codex` `ccr-checkin` `ccr-config` `ccr-db` `ccr-desktop`。

4. **父任务集成**  
   `just ci`。核对 AC1–AC14。

## Validation

子任务各自命令见子 `implement.md`。父任务交付：

```text
just ci
```

## Risky files

| 路径 | 风险 |
| --- | --- |
| `crates/ccr-core/src/core/logging.rs` | 轮转 chmod / 卸下文件层 |
| `crates/ccr-core/src/log_redact.rs`（新） | 误伤普通句子 |
| `ccr-ui/src-tauri/src/monitoring.rs` | persist 白名单、递归 |
| `ccr-ui/src-tauri/src/main.rs` | 退出未 flush |
| `ccr-ui/src/utils/logger.ts` | 回队打满内存 |
| 热路径平台 apply | 行为不变、只改日志形状 |

## Before start

- 本轮规划已消化 Codex P1/P2。
- 只 `start` 当前子任务。先 A。
- 实施前读 `trellis-before-dev`。
