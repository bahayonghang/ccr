# 前端事件 inventory

> `08-22-test-contract-rebuild` 批次 1。合并 `event-adjudication.md` 全局部分、`TAURI_GLOBAL_EVENTS`、高频事件，以及 CheckIn 局部事件（协同点 M）。
>
> **新增局部事件须同时登记到本文件**，否则下一次新增会再次绕过 `tests/tauri-event-inventory.smoke.test.ts` 的断言。

断言口径：

- 全局集合：本表 `eventBridge` + 生命周期 `常驻` 的事件名 = `TAURI_GLOBAL_EVENTS`（相等）。
- 全表：前端 inventory 事件名 ⊆ Rust emit 名（`events.rs` `channels` 常量 + `.emit` / `.emit_to` 字面量 + `EVENT_*` 常量）。

CheckIn 局部事件由 `08-22-views-checkin` 登记。WAF 等待复用既有签到任务事件，**不新造 emit 名**。`src-tauri/src/commands/waf.rs` 无 `emit`。WAF WebView bypass 走既有 `openWafLogin` / `validateWafCookieForAccount` wrapper。`checkin:job-finished` / `checkin:job-timeout` 由 `waitForCheckinJob.ts` 组件级 `listen()` 一次性等待，取消协议为 `disposed` + 迟到 unlisten 立即调用。

| 事件名 | 所有者 | 生命周期 | Rust emit 位置 |
| --- | --- | --- | --- |
| `usage:snapshot-updated` | `eventBridge` | 常驻 | `commands/usage.rs` |
| `usage:job-progress` | `eventBridge` | 常驻 | `commands/usage.rs` |
| `usage:job-finished` | `eventBridge` | 常驻 | `commands/usage.rs` |
| `usage:job-failed` | `eventBridge` | 常驻 | `commands/usage.rs` |
| `usage:job-recent-ready` | `eventBridge` | 常驻 | `commands/usage.rs` |
| `usage:session-index-progress` | `eventBridge` | 常驻 | `commands/usage.rs` |
| `usage:session-index-finished` | `eventBridge` | 常驻 | `commands/usage.rs` |
| `usage:session-index-failed` | `eventBridge` | 常驻 | `commands/usage.rs` |
| `usage:import-completed` | `eventBridge` | 常驻 | `events.rs USAGE_IMPORT / commands/usage.rs` |
| `claude_observer:updated` | `eventBridge` | 常驻 | `commands/usage.rs` |
| `env:refresh-requested` | `eventBridge` | 常驻 | `events.rs / commands/ssh.rs / main.rs` |
| `env:changed` | `eventBridge` | 常驻 | `events.rs ENVIRONMENT_CHANGED / commands/environment.rs` |
| `commands:job-progress` | `eventBridge` | 常驻 | `commands/command_exec.rs` |
| `commands:job-finished` | `eventBridge` | 常驻 | `commands/command_exec.rs` |
| `commands:job-cancelled` | `eventBridge` | 常驻 | `commands/command_exec.rs` |
| `app-log` | `useMonitoringFeed` | 常驻（批量） | `events.rs APP_LOG` |
| `token-stats` | `useMonitoringFeed` | 常驻（批量） | `events.rs TOKEN_STATS` |
| `app:monitoring` | `useMonitoringFeed` | 常驻（批量） | `events.rs MONITORING_ENTRY / monitoring.rs` |
| `checkin:completed` | `src/features/checkin/lib/checkinJob.ts` | 组件级 | `events.rs CHECKIN_COMPLETED` |
| `checkin:failed` | `src/features/checkin/lib/checkinJob.ts` | 组件级 | `events.rs CHECKIN_FAILED` |
| `checkin:job-delta` | `src/features/checkin/lib/checkinJob.ts` | 组件级（任务进度） | `events.rs / commands/checkin.rs` |
| `checkin:job-progress` | `src/features/checkin/lib/checkinJob.ts` | 组件级 | `events.rs CHECKIN_JOB_PROGRESS` |
| `checkin:job-finished` | `src/features/checkin/lib/waitForCheckinJob.ts` | 一次性（WAF 等待复用） | `events.rs / commands/checkin.rs` |
| `checkin:job-timeout` | `src/features/checkin/lib/waitForCheckinJob.ts` | 一次性（WAF 等待复用） | `events.rs / commands/checkin.rs` |
| `sync:status` | `views-sync-tools` | 组件级（待并入） | `events.rs SYNC_STATUS` |
| `task:progress` | `views-sync-tools` | 组件级（待并入） | `events.rs TASK_PROGRESS` |
| `app:notification` | `views-sync-tools` | 组件级（待并入） | `events.rs NOTIFICATION` |
| `app:task-panicked` | `views-sync-tools` | 组件级（待并入） | `events.rs / main.rs` |
| `codex-oauth-login-completed` | `views-codex` | 一次性 | `events.rs / commands/codex_auth.rs` |
| `codex-oauth-login-timeout` | `views-codex` | 一次性 | `events.rs / commands/codex_auth.rs` |
| `codex-tray:refresh` | `views-codex` | 窗口级 | `events.rs / desktop_shell.rs` |
| `shell:navigate` | `shell-port` | 窗口级 | `desktop_shell.rs` |
