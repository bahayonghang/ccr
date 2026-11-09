# Design: 日志系统安全与可观测优化

## Architecture

```
CLI / TUI / Tauri
        │
        ▼
ccr_core::init_logger | init_file_only_logger
        │
        ├── EnvFilter
        ├── fmt stdout（非 TUI，字段先走识别层）
        ├── SecureDailyWriter → ~/.ccr/logs/ccr.log.YYYY-MM-DD
        ├── PROCESS_ID
        └── try_send(BridgedLogEvent) ──仅 Tauri 启动 worker──►
                bounded 256
                      │
                      ▼
              单一 async worker（TLS 重入保护）
                      │
                      ▼
           MonitoringEntry { channel: runtime }
                      │
           EventLog + emit        SQLite（error + 白名单）
```

前端：

```
logger.ts ──识别/限额──► history + console
    │
    └── warn/error ──append_frontend_logs──► channel=frontend
                         （服务端再识别/限额）
```

## Boundaries

| 模块 | 职责 | 不负责 |
| --- | --- | --- |
| `ccr-core` | 初始化、过滤器、SecureDailyWriter、识别层、process_id、`try_send` 队列 | Tauri emit、SQLite |
| `ccr-types` | DTO 字段稳定 | 识别规则 |
| `ccr-db` | 缓冲、2s 定时 flush、查询、14 天清理 | 在 tracing 失败路径打 `ccr_*` 日志 |
| Tauri worker | 消费队列、映射 Monitoring、退出 flush | 解析 `CCR_LOG_LEVEL` |
| `logger.ts` | 前端识别、session_id、有界发送队列 | 文件路径 |
| Dashboard | 排除诊断频道 | 隐藏事件流 |

## Filter

`resolve_log_filter()`：

1. `CCR_LOG_LEVEL`，空则 `RUST_LOG`。
2. `EnvFilter::try_new` 失败或空 → `info,hyper=warn,reqwest=warn,h2=warn,rustls=warn,tokio=warn`。
3. 单个级别名 `trace|debug|info|warn|error|off` → `{level}` + 同上第三方 `warn`。
4. 含 `=` 或 `,` → 原样 `try_new`；失败回退默认。

## File names and permissions

活动文件：**没有** `ccr.log`。日切文件为 `ccr.log.YYYY-MM-DD`，日期为 UTC，与 `tracing-appender` 0.2.4 `Rotation::DAILY` + prefix `ccr.log` 一致。

`is_managed_log_file`：`name.starts_with("ccr.log.")`。不再把裸 `ccr.log` 当活动文件。

`SecureDailyWriter`（`ccr-core` 自有包装，内部仍用 `RollingFileAppender`）：

1. `create_dir_all`；Unix 目录 `0o700`。失败 → 不安装文件层。
2. 对目录内已有 `ccr.log.*` 补 `0o600`。单个 chmod 失败忽略该文件，不中止。
3. 打开 appender 后立刻对「今天」路径 `chmod 0o600`。失败 → 卸下文件层，stdout 保留。
4. 每次 `write` 后若日期相对上次已变，对新文件 `chmod 0o600`。失败 → 此后文件 `write` 返回 `Ok(len)` 但丢弃字节（停写），并设置 `file_layer_disabled`。禁止在此路径调用 `tracing::*`。

Windows：不 `from_mode`。

测试：在 tempdir 构造 writer，写一条，断言当天文件 `0o600`；把系统日期拨到次日或直接调用轮转钩后写第二条，断言新文件 `0o600`。chmod 失败用只读目录或 mock 在 Unix 测「停写文件、不 panic」。

## Redaction

`mask_sensitive` 只负责「整段值 → 掩码字符串」。识别层是新模块 `ccr_core::log_redact`，不是第二套掩码算法。

### 键名归一化

```
normalize_log_key(key) =
  key 的 ASCII 字母数字转小写，去掉其余字符
```

`api_key` / `apiKey` / `API-KEY` → `apikey`。

敏感键（归一化后精确匹配）：

`token` `apikey` `authorization` `cookie` `cookies` `password` `secret` `bearer` `accesstoken` `refreshtoken` `sessiontoken` `privatekey` `clientsecret` `authjson` `cookiesjson`

不含单独的 `auth`、`key`。

### 值规则 `redact_log_value`

- 深度 > 4 或累计 JSON 字节 > 8192 或数组长度 > 32 → `{ "truncated": true }`。
- 对象：每个键归一化后若敏感，字符串值整段 `mask_sensitive`；非字符串值改成掩码字符串 `"*****"`（避免展开密文 JSON）。不敏感键则递归。
- 数组：父键敏感则每个字符串元素整段 `mask_sensitive`；否则对元素递归。
- 字符串：先按「整段是否 JSON」处理，否则 `redact_log_text`。

### 自由文本 `redact_log_text`

不把整句送进 `mask_sensitive`。只替换命中 span，span 内调用 `mask_sensitive`。

| 规则 | 匹配 | 替换 |
| --- | --- | --- |
| bearer | `(?i)bearer\s+([A-Za-z0-9._\-+/=]{8,})` | 保留 `Bearer `，打码捕获组 |
| sk token | `\bsk-[A-Za-z0-9_-]{8,}` | 整段打码 |
| jwt | `\beyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+` | 整段打码 |
| cookie header | `(?i)(?:cookie|set-cookie):\s*([^\r\n]+)` | 保留头名，打码值 |

不匹配 UUID、URL、普通 hex。

整段 `serde_json::from_str` 成功且为对象或数组 → 走 `redact_log_value`，再序列化（紧凑 JSON）。

### 共用向量

路径：`crates/ccr-core/testdata/log_redaction_vectors.json`

每项：`{ "id", "kind": "text"|"value", "input", "must_not_contain": [], "must_contain": [] }`

Rust 与 `ccr-ui/tests/logger.smoke.test.ts` 读同一文件。

### 应用点

- fmt `FormatFields`：字段名走键规则，值走值规则。
- 桥 worker、`append_frontend_logs`、`logger.ts`：同一规则。TS 复制归一化表与正则，用向量锁对齐。

## Bridge: queue, worker, re-entry

`ccr-core` 只提供同步、无 tracing 的入队：

```text
const QUEUE_CAP: usize = 256

pub fn try_enqueue_bridged_log(event: BridgedLogEvent) -> EnqueueResult
  // Full | Disabled | Accepted | Reentrant

pub fn take_bridged_log_receiver() -> Option<Receiver<BridgedLogEvent>>
  // 只允许取一次
```

Layer `on_event`：

1. 若 TLS `BRIDGE_REENTERED` → 丢弃。
2. target 命中排除前缀则丢弃：`ccr_core::log_redact`、`ccr_core::core::logging`、`ccr_db::services::log_persistence`、`ccr_desktop::monitoring`、`ccr_desktop::bridge`。
3. level 非 WARN/ERROR 或 target 不以 `ccr_` 开头 → 丢弃。
4. `try_send`。满则 `dropped` 原子加一。禁止在此调用 `tracing::*`。

Tauri setup 启动**一个** worker：

1. `take_bridged_log_receiver()`。
2. 循环 `recv`。进入时置 TLS，调用 `record_monitoring_entry`，退出时清 TLS。
3. `record_monitoring_entry` 的 emit/flush 失败不得 `tracing::warn!`/`error!`。改为内部 `bridge_io_failures` 计数。需要可见性时写一条 `target: "ccr_desktop::bridge"` 之外的、已被排除的路径，或只更新计数。
4. 关闭：drop sender → worker 排空；`Exit` 上 `block_on` 等待 worker 结束，超时 500ms。

顺序：队列 FIFO。满载丢失最新（`try_send` 失败）。速率靠队列容量，不再另设 20/s。

测试：在假 sink 里再打一条 `ccr_core` error，断言队列不再增长。满 256 后 `try_send` 为 Full。

## Persistence deadline

`LogStorageConfig`：

- `flush_threshold = 20`
- `flush_interval = 2s`

Tauri setup 启动 interval 任务：每 2s `force_flush`。该任务的错误不走 `ccr_*` tracing。

退出（`RunEvent::ExitRequested` / `Exit`）：

1. 关闭桥 sender。
2. `tauri::async_runtime::block_on(monitoring_logs.force_flush())`，超时 500ms。
3. 再 `ccr_db::database::shutdown()`。

崩溃丢失窗口：最多约 2 秒未刷缓冲。写入 AC13。

## Correlation

| 来源 | `correlation_id` |
| --- | --- |
| 桥接 tracing | `process_id`（init 时 UUID） |
| 前端 logger / IPC | `session_id`（页面加载时 UUID）；DTO 若带合法 `correlation_id` 则覆盖 |
| 签到 job | 仍为 `job_id` |

`current_log_correlation_id()` 返回 `process_id`。热路径字段 `corr` 用该值。

不把前端 `session_id` 注入后端 tracing span。一次点击的前端日志与后端 apply 日志用两个 id 分列。

## IPC hard limits

| 项 | 值 | 超限 |
| --- | --- | --- |
| 单次 command 条目数 | 32 | 只处理前 32，返回 `Ok(())` |
| `message` | 2000 字符 | 截断 |
| `fields` JSON | 8192 字节 | `{ "truncated": true }` |
| `fields` 深度 | 4 | 同上 |
| 数组长度 | 32 | 截断数组 |
| `source` | 64 字符 | 截断；空 → `frontend` |
| `correlation_id` | 64 字符 | 截断；空则用 session/process |
| 前端发送队列 | 100 | 丢弃最旧 |
| 前端重试 | 3 | 丢弃该批，不再 `unshift` |
| 时间戳 | RFC3339 | 缺省/不可解析/偏离现在超过 24h 过去或 1h 未来 → 服务端 `Utc::now()` |

DTO：`FrontendLogInputDto.correlation_id: Option<String>` 可选新增。改完跑 `just tauri-bindings`，禁止只跑 drift check。

纯函数 `sanitize_frontend_log(input) -> FrontendLogInput` 放在可单测的模块，Rust 测试直接调，不启 Tauri。

## Dashboard

```ts
const DIAGNOSTIC_CHANNELS = new Set(['frontend', 'runtime'])
const isCoreSignal = (entry: MonitoringEntry) => !DIAGNOSTIC_CHANNELS.has(entry.channel)
```

更新 `dashboard-presentation-contracts.md`。

## Hot paths

只改：

- `ccr-cli` `platforms/{claude,grok,gemini,droid}.rs` apply/save
- `ccr-codex` `platforms/codex.rs` apply
- `ccr-checkin` `checkin_service.rs` 失败与里程碑
- `ccr-desktop` `main.rs` 启动与 `spawn_supervised`
- `ccr-config` profile/config 保存成功/失败

字段化 + `corr = current_log_correlation_id()`。

## Compatibility

- 环境变量名、日志目录不变。
- init 函数签名不变。
- `FrontendLogInput` 只增可选 `correlation_id`。
- 文件名合同从错误的 `ccr.log` 改为真实的 `ccr.log.YYYY-MM-DD`。

## Rollback

- 不取 receiver → 无桥，仅文件/stdout。
- chmod 失败 → 无文件层。
- Dashboard 排除表可单独回退。
