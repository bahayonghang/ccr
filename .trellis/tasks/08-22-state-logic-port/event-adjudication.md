# 事件桥接判定表 + 前端事件 inventory（全局部分，协同点 M）

> 08-22-state-logic-port 批次 3 交付物。`design.md` §3：事件 payload 含完整新数据用
> `setQueryData`；payload 只是变更通知用 `invalidateQueries`。Rust 侧数据源：
> `ccr-ui/src-tauri/src/events.rs`（channels 常量表）+ `commands/usage.rs` 的 emit 调用点。
> 本文件的清单是 `08-22-test-contract-rebuild` AC6「全部 Tauri Event 名」断言的数据源之一；
> 局部事件（组件级 `listen()`）由各视图子任务登记后并入（见 §4）。

## 1. 桥接层管理的全局事件与逐事件判定

| 事件 | payload 形态 | 判定 | 动作 | Rust emit 位置 |
| --- | --- | --- | --- | --- |
| `usage:snapshot-updated` | 变更通知 | `invalidateQueries` | 失效 `usageKeys.all` + `homeUsageKeys.all` | usage.rs（snapshot 管线） |
| `usage:job-progress` | 进度快照（部分） | `invalidateQueries` | 失效 `usageKeys.all`（快照随之刷新） | commands/usage.rs:614 |
| `usage:job-finished` | 完成通知 | `invalidateQueries` | 同上 | commands/usage.rs:686 |
| `usage:job-failed` | 失败通知 | `invalidateQueries` | 同上 | commands/usage.rs:597、459、1457 |
| `usage:job-recent-ready` | 就绪通知 | `invalidateQueries` | 同上 | commands/usage.rs:539 |
| `usage:import-completed` | 完成通知 | `invalidateQueries` | 同上（events.rs `USAGE_IMPORT`） | events.rs channels |
| `usage:session-index-progress` | 进度通知 | `invalidateQueries` | 失效 `homeUsageKeys.all` | commands/usage.rs:774–843 |
| `usage:session-index-finished` | 完成通知 | `invalidateQueries` | 同上 | commands/usage.rs:862 |
| `usage:session-index-failed` | 失败通知 | `invalidateQueries` | 同上 | commands/usage.rs:887 |
| `claude_observer:updated` | 变更通知 | `invalidateQueries` | 失效 `claudeObserverKeys.all`（原 store 单事件驱动全切片 refetch 的等价语义） | claude observer 管线 |
| `env:refresh-requested` | 通知 | `invalidateQueries` | 全量失效（`[]` 根前缀） | main.rs |
| `env:changed` | 通知 | `invalidateQueries` | 同上 | desktop_shell.rs / events.rs |

判定说明：上表事件的 payload 均不含「可直接写入 Query 缓存的完整切片」（progress 类为
任务级快照而非用量切片），故全部走 `invalidateQueries`，由既有 queryFn 重新拉取——与
原 store 的「事件 → refetch」语义一致。`setQueryData` 路径保留给高频事件（§3）。

## 2. 高频事件（不走逐条失效）

| 事件 | 消费方 | 处理 | 状态 |
| --- | --- | --- | --- |
| `app-log` | logger → MonitoringView | `createEventBatcher` ref 累积 + 250ms 定时批量提交 | **待接线**：Monitor feed 的 Query 缓存随批次 5 `useMonitoringFeed` hook 建立，接线动作与拼接语义在该批次落地 |
| `token-stats` | MonitoringView / Dashboard | 同上 | 同上 |
| `app:monitoring` | 监控条目流 | 同上 | 同上 |

间隔取值 250ms 为保守值，**待复核**：arch-quality-perf 场景 3（日志流）的 React 侧
基线数据由 `08-22-regression-release` 步骤 7 补测，届时按「日志流无掉帧且批量提交
不超预算」复核并调整（父任务 L 协同点的 React 侧路径）。

## 3. 取消协议与泄漏断言

`listen()` 返回 `Promise<UnlistenFn>`，cleanup 可能先于 resolve 执行。桥接层的
`disposed + track()` 写法保证迟到 unlisten 立即调用（`shell/eventBridge.ts`）。
泄漏断言三用例（立即 resolve / 卸载后 resolve / StrictMode 双挂载 + 延迟 resolve）
见批次 6 `tests/event-bridge-leak.smoke.test.tsx`。

## 4. 全部 Tauri Event 的完整 inventory（供 test-contract-rebuild 合并）

Rust 侧 emit 全集（`rg` src-tauri 实测）分三类：

1. **桥接层常驻**（§1 的 12 个事件名，`TAURI_GLOBAL_EVENTS` 常量）。
2. **高频批量**（§3 的 3 个：`app-log`、`token-stats`、`app:monitoring`）。
3. **组件级 / 窗口级（不在桥接层，登记待并入）**：
   - `checkin:completed` / `checkin:failed` / `checkin:job-delta` / `checkin:job-progress` / `checkin:job-finished` / `checkin:job-timeout`（events.rs；消费方 CheckIn 流程，`08-22-views-checkin` 登记）
   - `sync:status`、`task:progress`、`app:notification`、`app:task-panicked`、`usage:import-completed`（events.rs；sync/tray 视图消费，`08-22-views-sync-tools` 登记）
   - `env:changed` / `env:refresh-requested`（已入桥接层；tray 消费侧由 shell-port 核对）
   - `codex-oauth-login-completed` / `codex-oauth-login-timeout`（codex_auth.rs；`08-22-views-codex` 的 OAuth 弹窗一次性等待）
   - `codex-tray:refresh`（codex tray；`08-22-views-codex` 的托盘窗口快照刷新）
   - `shell:navigate`（shell 导航命令；`08-22-shell-port` 登记）
   - CheckIn WAF 等待的组件级事件（如存在）由 `08-22-views-checkin` 按其 design.md §4 登记

以上 3 类合并后即为「全部 Tauri Event 名」断言的数据源；本文件交付第 1、2 类与第 3 类的归属登记。
