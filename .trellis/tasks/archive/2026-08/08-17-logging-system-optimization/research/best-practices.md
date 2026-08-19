# 日志最佳实践对照

调研日期：2026-08-17  
来源：`tracing-subscriber` 文档、Tauri v2 plugin-log、OWASP Logging / Desktop Top 10、Rust structured logging 指南。

## 1. Rust tracing

官方与生产指南的共同要求：

- 用 `EnvFilter` 做运行时级别控制，支持 `info,ccr_checkin=debug` 这类分 crate 指令。
- 优先结构化字段（`user_id = %id`），避免 `format!` 插值。
- 生产文件日志用 JSON 或至少可解析的稳定字段；开发期可用人类可读 `fmt`。
- 禁用级别时开销应接近零；热路径不要做昂贵计算。
- `#[instrument(skip(...))]` 用于跨异步边界的操作，不要给每个小函数加 span。
- 不要在循环里建 span。
- 从 `log` 迁到 `tracing` 可用 `tracing-log`；CCR 已接但业务侧已全部用 `tracing`。

`EnvFilter` 细节：

- `EnvFilter::new` / `from_env` 会忽略非法指令。
- 不可信输入应关闭 regex 匹配。
- 可用 per-layer filter：stdout 与 file 使用不同详细度。
- 默认应压低第三方 crate（`hyper`、`reqwest`、`tokio`）的噪声。

CCR 现状：有 `CCR_LOG_LEVEL`，无默认第三方过滤，无 JSON feature，无 span，插值仍占多数。

## 2. Tauri 桌面日志

`tauri-plugin-log`（v2，文档更新于 2026-05-26）：

- Target：`Stdout` / `Stderr` / `LogDir` / `Folder` / `Webview`
- `LogDir` 走各平台推荐目录（macOS `~/Library/Logs/<bundle>` 等），不是 `~/.ccr/logs`
- 默认可按大小丢弃旧文件；可改 rotation
- `Webview` + `attachConsole` 把 Rust 日志送到前端控制台
- SECURITY.md 明确：插件不脱敏；前端被 XSS 后，Webview target 或前端日志可泄漏密钥

`tauri-plugin-tracing`：

- 不强制设置全局 subscriber
- 可用 `WebviewLayer` 或自建 `tracing-appender`
- 推荐在 Tauri 启动前初始化 tracing，以便启动失败也有日志

Tauri 维护者讨论（plugins-workspace #2516）把桌面日志的难点归纳为：多进程、多语言、本机部署、隐私、统一语义。结论倾向：应用自己定义管道，插件只做适配，而不是换一套全局系统。

对 CCR 的含义：现有自定义管道（`ccr-core` 初始化 + Monitoring IPC）比迁到官方插件更贴合本仓库。应保留 `~/.ccr/logs`，补齐权限、脱敏和契约，而不是引入第二套文件目录。

## 3. OWASP

适用条目：

- Logging Cheat Sheet：文件权限收紧；不要记录 token、密码、连接串、密钥；记录失败（磁盘满、权限不足）要可测；集中写出例程。
- Desktop Top 10 DA3：日志中的敏感信息算敏感数据暴露。
- Desktop Top 10 DA5：日志目录/文件对低权限用户可读属于授权失败。
- Desktop Top 10 DA10：缺少安全日志与监控；用户可控字段写入审计日志可被注入。
- Proactive Control C9：统一格式；编码不可信输入；保护完整性；不要记过多或过少。

对 CCR 的直接映射：

| OWASP 要求 | CCR 现状 |
| --- | --- |
| 日志文件 owner-only | `ccr.log` 未设 `0o600` |
| 不记录密钥 | 业务侧有 `Secret`，日志边界无二次脱敏 |
| 集中写出 | Rust 初始化集中；前端有 `logger.ts`；两条管道未汇合 |
| 编码不可信输入 | `append_frontend_logs` 原样接受 `message` / `fields` |
| 安全相关操作可审计 | Monitoring 覆盖环境切换、导入、签到；配置切换多在文件日志 |
| 前端日志噪声不等于健康信号 | Dashboard 已排除 `channel=frontend` |

## 4. 推荐采用

1. 保持两层语义，但写清契约：诊断日志（文件/stdout）与产品事件（Monitoring）职责不同。
2. 在写出边界做一次脱敏：Rust fmt layer 或显式 helper；前端 `logger.ts` 在 history / console / IPC 之前脱敏。
3. Unix 日志文件与目录设为 owner-only；Windows 保持当前用户 ACL，不扩大继承。
4. `EnvFilter` 默认 `info`，并对 `ccr_*=info`、第三方 `warn` 给出稳定默认；`CCR_LOG_LEVEL` 仍可覆盖。
5. 新代码强制结构化字段；旧插值按热路径分批改，不一次清库。
6. 前端 `warn` / `error` 继续进 Monitoring；`debug` / `info` 默认只留内存与开发控制台。
7. IPC 入口限制条数、单条大小，并对 `fields` 做键名黑名单 / 脱敏。
8. 文档与实现使用同一文件名：`~/.ccr/logs/ccr.log` 与 `ccr.log.YYYY-MM-DD`。

## 5. 推荐拒绝

1. 用 `tauri-plugin-log` 替换 `init_logger`。
2. 打开 `Webview` target 把全部 Rust 日志灌进前端。
3. 本期接入 OpenTelemetry / 远程采集。
4. 把 Monitoring SQLite 当成完整诊断日志仓库（会膨胀，且与 14 天文件日志重复）。
5. 为每个函数加 `#[instrument]`。
6. 把前端 `debug` 默认刷进 SQLite。
