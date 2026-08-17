# 日志系统现状

调研日期：2026-08-17  
范围：`crates/` 与 `ccr-ui/`（含 `src-tauri`）  
方法：源码与 spec 对照，辅以调用计数。

## 1. 两条管道并存

CCR 没有单一日志系统。诊断日志和产品监控是两套独立管道。

| 管道 | 所有者 | 入口 | 落盘 | 消费方 |
| --- | --- | --- | --- | --- |
| 诊断日志 | `ccr-core` | `init_logger()` / `init_file_only_logger()` | `~/.ccr/logs/ccr.log`（按天轮转） | CLI stdout、排障、文件 tail |
| 产品监控 | `ccr-types` + `ccr-db` + Tauri | `record_monitoring_entry` / `append_frontend_logs` | SQLite `log_entries` | `/monitoring`、Dashboard 信号流 |

后端 `tracing::*` 默认不进入 Monitoring。Monitoring 只收录显式构造的 `MonitoringEntry`，以及前端 `logger.warn` / `logger.error`。

## 2. 诊断日志（Rust / tracing）

### 2.1 初始化

- 实现：`crates/ccr-core/src/core/logging.rs`
- CLI：`crates/ccr/src/main.rs` — TUI 走 `init_file_only_logger()`，其余走 `init_logger()`
- Desktop：`ccr-ui/src-tauri/src/main.rs:113` 在 Tauri builder 之前调用 `ccr_core::init_logger()`
- 过滤器：`CCR_LOG_LEVEL` 优先于 `RUST_LOG`，缺省 `info`
- 构造：`EnvFilter::new(log_level)`，非法指令被丢弃
- 兼容：`tracing_log::LogTracer::init()`，仓库内未发现 `log::` 宏调用
- 工作线程：`tracing-appender` `NonBlocking` + `WorkerGuard` 存在 `OnceLock<Mutex<Vec<WorkerGuard>>>`

### 2.2 文件行为

- 目录：`$HOME/.ccr/logs`
- 文件名：活动文件 `ccr.log`，轮转 `ccr.log.YYYY-MM-DD`
- 清理：仅删除修改时间超过 14 天、且文件名匹配上述规则的文件
- 权限：创建后不设 owner-only（Unix 上通常为 umask 决定的 `0644`）
- 格式：`fmt` 文本；stdout 开 ANSI，文件关 ANSI
- 未启用：`json` feature、`with_file` / `with_line_number`、span events、size-based rotation

文档与实现不一致：

- `docs/examples/troubleshooting.md:356` 写的是 `~/.ccr/logs/ccr.$(date +%Y-%m-%d).log`
- `ccr-ui/README_CN.md:449-451` 仍写 `./ccr-ui/logs/`、`ccr-ui/logs/backend-console.log`，并描述已退役的 Axum REST

### 2.3 调用量（2026-08-17 扫描）

`tracing::{info,warn,error,debug,trace}!`：71 个文件，374 次。

| 级别 | 次数 |
| --- | --- |
| debug | 124 |
| info | 122 |
| warn | 94 |
| error | 25 |
| trace | 9 |

按 crate：`ccr-ui/src-tauri` 127，`ccr-cli` 56，`ccr-config` 52，`ccr-sync` 41，`ccr-checkin` 38，`ccr-core` 24，其余合计 36。

结构化启发式：约 99 次字段写法，约 197 次字符串插值。仓库内 **0** 处 `#[instrument]`。大量 `info!` 带 emoji（`✅` / `💾`），更像操作成功提示，而不是可检索事件。

`println!` / `eprintln!`：CLI 帮助与表格输出是规格允许的人机输出。生产路径里残留的 `eprintln!` 主要在测试 / benchmark（`codex_usage_service.rs`、`sessions/parser.rs`）。TUI 启动失败仍用 `eprintln!` 写到 stderr，符合“避免污染终端绘制”的边界。

## 3. 产品监控（Desktop）

### 3.1 契约

- 类型：`crates/ccr-types/src/monitoring.rs`
- 字段：`id`、`timestamp`、`level`、`channel`、`event_type`、`source`、`message`、`correlation_id?`、`fields?`
- `correlation_id` 目前几乎只用于签到 job id
- 前端日志映射：`frontend_log_entry()` 把 level 映射为 `frontend.warn` / `frontend.error` / `frontend.debug` / `frontend.info`，`channel` 固定为 `frontend`，`correlation_id` 为 `None`

### 3.2 持久化

- `crates/ccr-db/src/services/log_persistence.rs`
- 默认保留 14 天，缓冲 100 条后刷盘
- `record_monitoring_entry(..., persist=true)` 每次都会 `force_flush()`
- 落盘条件：`Warn` / `Error`，或白名单事件（`environment.changed`、`usage.import.completed`、`checkin.job.finished` / `timeout`、`frontend.warn` / `frontend.error`）
- 前端 `info` / `debug` 进入内存事件环，默认不进 SQLite

### 3.3 前端 logger

- 唯一实现：`ccr-ui/src/utils/logger.ts`
- ESLint `no-console: error`，源码收口到该文件
- 调用：60 个文件，约 170 次；`error` 130，`warn` 27，`info` 11，`debug` 2
- 内存历史：最多 100 条
- 原生桥：仅 `warn` / `error`，250ms 批处理，`append_frontend_logs`
- `normalizeFields` 会序列化任意 `data`，`Error` 会带上 `stack`
- **无脱敏**，**无字段大小限制**，**无 source 细分**（恒为 `frontend`）
- 无 `logger.ts` 单测；其它测试只 mock logger

### 3.4 Monitoring UI

- `useMonitoringFeed.ts` 合并：SQLite/事件 feed + 前端 logger 历史 + `app:monitoring`
- Dashboard 已有契约：`channel === 'frontend'` 不得驱动就绪度 / 信号计数 / “打开监控”动作（`dashboard-presentation-contracts.md`）
- 原因：一次前端重试 `logger.error` 曾同时把三处健康指示打红

## 4. 已有安全约定

规格已要求：

- 凭据字段用 `Secret`，`expose()` 结果不得进入 `format!` / `tracing` / 错误字符串
- 唯一脱敏算法：`ccr_core::utils::mask::mask_sensitive`
- 配置 / auth / 密钥文件写盘走 atomic + 可选 `0o600`
- Tauri audit 只记 descriptor，不记 invoke payload
- Check-in / Codex / Config 明确禁止把 cookie、token、auth JSON 打进日志

缺口：这些约定作用在业务写盘和 DTO，**没有作用在日志写出边界**。`~/.ccr/logs/ccr.log` 与 `append_frontend_logs` 的 `fields` 都没有二次脱敏。

## 5. 已确认的问题

1. 诊断日志与 Monitoring 不互通：Desktop 排障要同时看文件和 UI。
2. 日志文件权限未收紧，桌面共享目录上的明文日志可被同机其它用户读取。
3. 前端任意对象（含 stack）可经 IPC 进入 SQLite 与 Monitoring 事件。
4. `append_frontend_logs` 是不可信输入面：存在日志注入与密钥泄漏路径。
5. 文档文件名与实现不一致，README 仍描述旧 Axum / `ccr-ui/logs`。
6. 结构化字段覆盖率低，缺少 span / correlation，难以按一次操作串联 CLI 与 Desktop。
7. `info` 噪声偏高：大量“已保存 / 已应用”成功日志。
8. `ColorOutput` 与 logger 初始化挤在同一模块，职责混杂。
9. 无 `logger.ts` 行为测试，也无日志脱敏回归。
10. `force_flush` 在每条需持久化的监控条目上执行，签到 / 前端错误突发时放大 SQLite 写次数。

## 6. 明确不应做的事

- 用 `tauri-plugin-log` 替换现有管道：会与 `~/.ccr/logs` + SQLite Monitoring 重叠，且官方插件声明不内置脱敏，`Webview` target 会把 Rust 日志送进前端。
- 把 OpenTelemetry 作为本期目标：CCR 是本机 CLI/桌面工具，没有集中采集后端。
- 一次性改写全部 374 处 `tracing!` 插值。
- 把前端 `debug` / `info` 默认持久化进 SQLite。
