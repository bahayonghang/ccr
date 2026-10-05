# 修复 Overview 与 Agent Sessions 页面报错

## Goal

让 CCR UI 的 Overview 与 Agent Sessions 在真实 Local 桌面数据下不再把可恢复故障渲染成页面级 Error：Overview 不再把运行期窗口异常记成致命 `[startup]` 失败；Agent Sessions 对缺失/失效源给出稳定的「源不可用」状态，并在增量刷新后能打开当前仍存在的会话 transcript。

## Background

用户在 v7.3.0 桌面壳截到两处报错。仓库与本机数据已核对，不需要再向用户确认事实。

### Overview

- Event stream 的 Error 1 来自持久化监控行，不是当前会话崩溃。`C:\Users\lyh\.ccr-ui\ccr-ui.db` 中 `log_entries` 仅有一条 error：`2026-08-26T16:29:15.843+00:00`（本地 12:29 AM），message 为 `[startup] Unhandled window error failed`，`metadata_json.message` 为 `Cannot read properties of undefined (reading 'toString')`。
- 该行仍在 14 天保留期内，因此 2026-09-02 的 Overview 仍显示 Error 1；就绪清单不受影响（frontend 通道按 Dashboard 合同不计入 `signalCounts`）。
- `ccr-ui/src/main.tsx:30` 调用 `installStartupErrorHandlers()` 且丢弃 disposer。`startupRecovery.ts:103-108` 把任意 `window` `error` / `unhandledrejection` 记成 `[startup]` 并调用 `renderFatalStartup`。处理器在 React 挂载成功后仍然驻留。

### Agent Sessions

- 详情面板 `Error` / `agent_session_source_validation_failed` 来自 `ccr-ui/src-tauri/src/services/agent_sessions.rs:661-677`：`restore_source` 把 provider 校验的所有失败都映射成该码。`get_detail` 在打开 transcript 前必须 restore。
- 列表自动选中第一行（`resolveActiveArchiveId`）。本机 `usage.db` 中 Codex 归档最新行是 `rollout-2026-03-06T16-49-57-...`，`file_path` 为 `C:\Users\lyh\.codex\sessions\2026\03\06\...`，`source_state=live`，`message_count=0`。该目录现在不存在。
- Codex 归档只有 2025-10 到 2026-03，共 359 行且全部 0 条消息；`usage_session_source_state` 为 0 行。Provider strip 的 Codex `3077` 来自实时 `discover()`。`sessions/2026/09/02/*.jsonl` 存在且含 `response_item`/`role=user`，但归档没有 `2026\09` 路径。
- 合同已要求：源不在 canonical root 或形状非法时拒绝打开；源在详情读取时消失时返回稳定的 source-unavailable，并保留摘要供刷新对账。当前实现把「文件缺失」和「形状篡改」混成同一个 validation 码，UI 用 `getErrorMessage` 直接展示生码。
- 增量刷新按钮仍是产品入口；本任务不改为进入页面自动刷新。刷新必须能 upsert 现存 jsonl，并把磁盘上已消失的归档标为 missing。

## Requirements

- R1. 启动期窗口处理器只覆盖 React 壳挂载完成之前。挂载成功后必须卸载；之后的 `window` 错误不得再写 `[startup] Unhandled window error`，也不得用 `renderFatalStartup` 替换 `#app`。
- R2. 启动失败仍通过 `logger.error` 进入 Event stream；frontend 通道继续按 Dashboard 合同只出现在 Event stream，不驱动就绪/行动队列。不得删除或手工清空用户已有 `log_entries`。
- R3. `get_detail` 在归档源文件缺失、canonical root 不可用、或源已离开 provider root 时，返回稳定的 `agent_session_source_unavailable`（或合同已有的同等 unavailable 码），不得用 `agent_session_source_validation_failed` 表示缺失。
- R4. `agent_session_source_validation_failed` 仅保留给形状/变体/kind/member 篡改等真正的校验失败。
- R5. Agent Sessions 详情不得把生错误码当作 description。缺失/不可用源使用与 `source missing` 语义一致的 i18n 空态（可 Retry）；其它 `agent_session_*` 码同样映射到中英文文案，日志与 UI 不包含原始路径或 transcript。
- R6. 增量刷新在 Codex（及其余七个 family）discover 成功时必须：upsert 当前仍存在的 source；把本次未见到的旧归档标为 `missing`。刷新后列表按 `updated_at DESC` 能选中现存会话，详情能读到有界 transcript，而不再默认打开已删除的 2026-03 Codex 行并显示 Error。
- R7. Windows 上对真实存在的 `~\.codex\sessions\YYYY\MM\DD\rollout-*.jsonl`，`restore_source` / `validate_stored_source` 必须成功。`canonicalize` + `starts_with` / `strip_prefix` 不得把合法 live 源误判为 escaped/invalid。

## Acceptance Criteria

- [ ] AC1 (R1). 有测试证明：挂载完成后触发 `window` `error` 不再调用 `renderFatalStartup`，日志前缀不是 `[startup] Unhandled window error`。挂载完成前的未处理错误仍走启动失败路径。
- [ ] AC2 (R2). Dashboard 合同保持：该 frontend error 不改变就绪 pill / 行动队列。本任务不修改或删除 `C:\Users\lyh\.ccr-ui\ccr-ui.db` 中已有监控行。
- [ ] AC3 (R3-R4). Tauri `agent_sessions` 测试：归档指向已删除 jsonl 时 `get_detail` 返回 `agent_session_source_unavailable`；形状篡改（错误扩展名/错误 kind/越界路径）仍返回 `agent_session_source_validation_failed`，且不打开用户提供的路径。
- [ ] AC4 (R5). 详情空态对 unavailable 使用 i18n 标题/说明，不出现 `agent_session_source_validation_failed` 原文。en-US 与 zh-CN 均有对应键。Retry 仍触发 refetch。
- [ ] AC5 (R6). 服务测试：刷新一个含「现存 jsonl + 已删除归档行」的 registry 后，现存行可 `get_detail` 成功；已删除行 `source_state=missing`。用户点击 Incremental refresh 后，本机 Codex 当前日期会话可出现在列表并打开 transcript（桌面验证；web preview 无 Tauri 时记 `UNVERIFIED`）。
- [ ] AC6 (R7). `ccr-store` provider 测试覆盖 Windows 风格分隔符与 canonicalize 后的 root 包含关系；现存 Codex live 源 restore 成功。
- [ ] AC7. 窄门禁：`cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml agent_sessions`、`cargo test -p ccr-store sessions::providers`、`cd ccr-ui && bun run test:smoke -- tests/agent-sessions tests/quality/coverage-boost.smoke.test.ts`（若启动 handler 测试独立则一并运行）、相关 i18n 检查，以及 `just frontend-check-quick` 或合同要求的 `just ui-check`。

## Out of Scope

- 删除或迁移用户本机 `log_entries` / `usage.db` 历史行。
- 进入 `/agent-sessions` 时自动开始增量刷新，或引入 SSE/文件 watcher。
- 解析全部 3077 个 Codex 会话的产品化性能调优、FTS、远程/WSL 扫描。
- 把 `developer` role 的 Codex 内部消息扩进默认 transcript（仍只展示 user/assistant/structured tool）。
- Overview 视觉改版、Usage 空态、Event stream 过滤 frontend 通道。
- 修改 `ccr_config::Platform` 或其它 provider 的 canonical root 策略。

## Risks

- 本机第一次增量刷新可能扫描数千个 Codex jsonl；验收只要求功能正确与现有 job 去重，不为 3077 文件新设 SLA。
- 2026-08-26 的 `undefined.toString()` 没有堆栈。若卸载启动处理器后该 TypeError 仍在当前代码路径复现，必须修根因；若无法在当前树复现，记录为历史噪声，不为此猜测大面积改 Dashboard。
- Windows `Path::canonicalize` 产生 `\\?\` 前缀时，`starts_with`/`strip_prefix` 可能误伤合法源；R7 必须用本机或 Windows CI 路径形状覆盖。
