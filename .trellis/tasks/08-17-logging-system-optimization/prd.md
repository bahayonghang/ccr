# 日志系统安全与可观测优化

## Goal

排障时能在 `~/.ccr/logs` 与 Desktop Monitoring 里读到同一套安全、可检索的事件语义：级别、来源、结构化字段、以及进程级或会话级关联 id。

密钥、token、cookie、可解析的 auth JSON 不得以原文出现在文件日志、SQLite `log_entries`、前端 history、console 或 Monitoring DTO。Desktop 上 `ccr_*` 的 `tracing` `warn`/`error` 进入 Monitoring 实时流。同一 Desktop 进程的后端 runtime 事件共享 `process_id`；同一前端会话的 logger 事件共享 `session_id`；经 `append_frontend_logs` 上报的条目在 Monitoring 中使用该前端 id。

## User Value

本机排障不必同时猜两套目录和两种文件名。共享电脑上的日志文件以 owner-only 权限落盘。前端 `logger.error` 不会把任意对象或 stack 送进持久化存储。后端运行时告警能在 `/monitoring` 按进程分组查看，且不会把 Dashboard 就绪度打红。

## Decisions

| ID | 决定 |
| --- | --- |
| D1 | MVP = 安全加固 + 契约统一 + 可观测打通。不迁官方插件，不做全量插值改写 |
| D2 | 保留双管道。不引入 `tauri-plugin-log` / `tauri-plugin-tracing` / OpenTelemetry |
| D3 | `mask_sensitive` 的可见前后缀规则不变。写出边界新增识别层 `redact_log_text` / `redact_log_value`：只对敏感键的整个值、以及自由文本中命中的片段调用 `mask_sensitive` |
| D4 | 新代码用结构化字段。旧调用只改热路径（profile apply、签到里程碑、Tauri 启动/监督任务、config 保存） |
| D5 | 父任务持有契约与集成验收。实施顺序：A `logging-rust-init` → B `logging-frontend-ipc` → C `logging-observability-bridge`。B 的 Rust 服务端脱敏依赖 A |
| D6 | 桥接事件使用 `channel=runtime`。Dashboard `isCoreSignal` 排除 `frontend` 与 `runtime`。`/monitoring` 展示全部频道 |
| D7 | 桥接范围：target 以 `ccr_` 开头的 `warn`/`error`，扣除内部排除表。实时流全收（有界队列）；SQLite 只持久化 `runtime.error` 以及白名单 / 前端 warn-error |
| D8 | 关联语义为本期进程/会话分组，不实现经全部 Tauri command 传播的 operation/request id |
| D9 | 活动日志文件名为 `ccr.log.YYYY-MM-DD`（UTC 日）。每次创建与轮转后 Unix 权限为 `0o600`。目录 `0o700` |
| D10 | 桥为有界 `try_send` + 单一消费 worker + TLS 重入保护。sink 的 `emit` 禁止再打 `ccr_*` 的 tracing |
| D11 | persist 最大延迟 2 秒（定时 flush）。正常退出先 `force_flush` 再 `database::shutdown`。崩溃丢失窗口 ≤ 2 秒缓冲 |
| D12 | IPC 限额为硬数字，超限行为唯一：截断字段/消息，不拒绝整条 `error`/`warn`；批次超出丢弃多余条目 |

## Confirmed Facts

- 诊断日志由 `ccr-core` 初始化。CLI TUI 仅写文件，其它 CLI 与 Tauri 写 stdout + 文件。`crates/ccr-core/src/core/logging.rs`。入口：`crates/ccr/src/main.rs:22-29`、`ccr-ui/src-tauri/src/main.rs:113`。
- `RollingFileAppender::new(Rotation::DAILY, dir, "ccr.log")` 生成 `~/.ccr/logs/ccr.log.YYYY-MM-DD`（UTC），没有稳定的 `ccr.log` 活动文件。`tracing-appender` 0.2.4 `create_writer` 每次 `OpenOptions::create`。
- `mask_sensitive` 对传入整段无条件打码（≤10 全 `*`，>10 留前后 4 字符）。`mask_if_sensitive` 只看变量名是否含 `TOKEN`/`KEY`/`SECRET`。
- Desktop Monitoring：`MonitoringEntry` → 内存 `EventLog` + 可选 SQLite。后端 `tracing` 不自动进入。
- 前端唯一出口：`ccr-ui/src/utils/logger.ts`。`warn`/`error` 经 `append_frontend_logs`；失败会把整批 `unshift` 回队列。
- Tauri 包名 `ccr-desktop`，target `ccr_desktop`。`record_monitoring_entry` 为 async；emit/flush 失败会 `tracing::warn`/`error`。
- 退出：`RunEvent::Exit` 同步 `ccr_db::database::shutdown()`，该函数只打日志，不 flush Monitoring。
- Dashboard 当前只排除 `channel=frontend`。

研究见 `research/current-state.md`、`research/best-practices.md`、`research/codex-review-verification.md`。

## Requirements

- R1. `init_logger` / `init_file_only_logger` 独占初始化。TUI 只写文件。目录 `~/.ccr/logs`。
- R2. Unix：目录 `0o700`；每个 `ccr.log.YYYY-MM-DD` 在首次创建与日切后为 `0o600`。Windows 不扩大其它用户读权限。
- R3. 过滤器：`CCR_LOG_LEVEL` > `RUST_LOG`；解析失败回退 `info` + 第三方 `warn`。单级别名自动附带第三方 `warn`。
- R4. 写出前走识别层。敏感键的值整段 `mask_sensitive`。自由文本只打码命中片段。可解析为 JSON 对象的整段字符串按值规则递归。`Error.stack` 不进 IPC。
- R5. `logger.ts` 在 history / console / 原生桥之前应用同一识别规则与限额。
- R6. `append_frontend_logs` 视为不可信输入，服务端再次识别与限额。直调 command 的 Rust 测试覆盖绕过前端的路径。
- R7. 前端 `warn`/`error` 进入 Monitoring，`channel=frontend`。
- R8. Dashboard 计数忽略 `frontend` 与 `runtime`；事件流仍展示。
- R9. Tauri 注册桥消费者。`ccr_*` 的 `warn`/`error`（排除内部 target）进入 `channel=runtime`。CLI/TUI 不注册。
- R10. 后端桥接事件 `correlation_id=process_id`。前端 logger 与 IPC 条目 `correlation_id=session_id`（DTO 可选字段可覆盖，需过长度限额）。
- R11. persist：阈值 20 或每 2 秒定时 flush，先到先刷。退出路径 `block_on(force_flush)` 成功或超时 500ms 后再 `shutdown`。
- R12. 文档指向 `ccr.log.YYYY-MM-DD`。删除 `ccr-ui/logs` 与 Axum 过期描述。
- R13. spec 覆盖初始化、轮转权限、识别层、桥队列、flush 时限、前端 IPC、Dashboard 排除。
- R14. 共享测试向量：`crates/ccr-core/testdata/log_redaction_vectors.json`，Rust 与 TS 共用。

## Acceptance Criteria

- [x] AC1. `CCR_LOG_LEVEL=debug` 时 CLI 非 TUI 在 stdout 与当天 `~/.ccr/logs/ccr.log.YYYY-MM-DD` 同时出现诊断输出；TUI 终端不被日志打乱。
- [x] AC2. Unix：新日志目录 `0o700`；当天文件与模拟日切后的新文件均为 `0o600`。目录创建失败则省略文件层且 stdout 仍可用。当天文件 chmod 失败则停止后续文件写入，stdout 仍可用。（Windows 本机跳过 `cfg(unix)` 权限用例）
- [x] AC3. 共享向量中的 API key / Bearer / cookie 键值 / 可解析 auth JSON，在 history、IPC 载荷、SQLite、Monitoring DTO 中均无原文。自由文本里未命中的普通句子保持可读。`Error.stack` 不出现在 IPC。
- [x] AC4. 单条 `message` 超过 2000 字符则截断；`fields` 超过 8 KiB 或深度 4 则改写为 `{ "truncated": true }`；单次 command 超过 32 条只处理前 32 条并成功返回。不 panic。
- [x] AC5. Dashboard `signalCounts` / 就绪度 / `open-monitoring` 忽略 `frontend` 与 `runtime`。事件流仍渲染。
- [x] AC6. Tauri 中 `tracing::error!(target: "ccr_core", ...)` 出现在 Monitoring，`channel=runtime`，`correlation_id` 等于该进程 `process_id`。同等 `warn` 在实时流、默认不进 SQLite。（映射由单测覆盖；未起完整桌面进程）
- [x] AC7. CLI/TUI 无桥接。文件层按 AC1/AC2 工作。
- [x] AC8. 热路径 apply / 签到失败 / 监督任务 panic 的日志含结构化字段与 `corr=process_id`，字段值经识别层。
- [x] AC9. `docs/examples/troubleshooting.md` 指向 `~/.ccr/logs/ccr.log.YYYY-MM-DD`；`ccr-ui/README_CN.md` 不再写 `ccr-ui/logs` 或 Axum `:38081`。
- [x] AC10. spec 覆盖 R1–R14 的可执行合同。
- [x] AC11. 父任务集成门禁：`just ci`。
- [x] AC12. emit 失败或 SQLite flush 失败不会再产生可被桥接收的 `ccr_*` tracing 事件。有界队列满时丢弃并在内部计数，不递归。
- [x] AC13. 进程存活期间，间隔 2.1 秒写入一条可持久化 `runtime.error`，第二条出现在 SQLite 中。正常退出后缓冲区空。（实现为 2s ticker + 退出 flush；未跑实机 2.1s 计时）
- [x] AC14. 前端同一会话的两条 `logger.error` 经 IPC 后 `correlation_id` 相同且等于 `session_id`。

## Out of Scope

- 官方日志插件、OpenTelemetry、JSON 文件格式。
- 全仓插值改写、全仓 `#[instrument]`。
- 默认持久化前端 `debug`/`info` 或 `runtime.warn`。
- 把 `runtime` 算进 Dashboard 就绪度。
- 经全部 Tauri command 传播 operation/request id。
- VS Code 日志、迁移到 `app_log_dir`、拆分 `ColorOutput`。
- 修改 `mask_sensitive` 的前后缀规则。

## Technical Notes

设计见 `design.md`。顺序见 `implement.md`。

| 子任务 | 交付 | 依赖 |
| --- | --- | --- |
| `logging-rust-init` | 过滤器、轮转 chmod、识别层、process_id、有界队列 API | 无 |
| `logging-frontend-ipc` | logger 识别/限额、IPC 硬限额、服务端再识别、向量测试 | A 的 `redact_*` |
| `logging-observability-bridge` | worker、flush 时限、Dashboard、热路径、文档 spec | A，建议 B 已完成 |
