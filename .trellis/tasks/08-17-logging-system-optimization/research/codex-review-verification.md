# Codex 规划审阅核对

日期：2026-08-17  
对照：仓库源码 + `tracing-appender` 0.2.4。

| 编号 | 结论 | 证据 |
| --- | --- | --- |
| 1 脱敏 vs `mask_sensitive` | 成立 | `mask.rs:20-30` 对整段无条件打码，不扫描子串。`design.md:74` 把整段委托给它会毁掉普通日志。 |
| 2 轮转文件名与 chmod | 成立 | `Rotation::DAILY` + prefix `ccr.log` 生成 `ccr.log.YYYY-MM-DD`（UTC）。`create_writer`（`rolling.rs:780-795`）每次 `OpenOptions::create`，不继承 chmod。活动文件不是 `ccr.log`。 |
| 3 同步 sink / 递归 | 成立 | `record_monitoring_entry` 为 async。emit 失败 `tracing::warn`（`monitoring.rs:38`），flush 失败 `tracing::error`（`log_persistence.rs:71`）。Tauri crate 名 `ccr-desktop`，target `ccr_desktop`，与 `ccr_db` 都以 `ccr_` 开头。 |
| 4 去掉 force_flush | 成立 | 默认阈值 100。`RunEvent::Exit` 同步调用 `ccr_db::database::shutdown()`（`main.rs:394-406`），当前无 Monitoring flush。`shutdown` 只打日志。 |
| 5 关联 id 目标矛盾 | 成立 | PRD 写「一次操作前后端对上」，设计写「不要求同一条链」。 |
| 6 IPC 限额是建议 | 成立 | 深度/大小/速率均为建议；AC4 允许截断或拒绝；前端失败无限回队。 |
| 7 子任务依赖与门禁 | 成立 | B 的服务端脱敏依赖 A 的函数，却写成可并行。C 的 jsonl 只有 Dashboard spec。 |

规划按下列冻结项回写：识别层 + `mask_sensitive` 只打码命中片段；文件名改为 `ccr.log.YYYY-MM-DD` 且每次创建 chmod；有界队列 + 重入保护；2 秒定时 flush + 退出先 flush；Goal 改为进程/会话分组；IPC 数字写死；B 依赖 A；父任务交付跑 `just ci`。
