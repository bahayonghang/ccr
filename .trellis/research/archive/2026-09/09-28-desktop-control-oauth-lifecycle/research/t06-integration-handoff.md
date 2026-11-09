# T06 → T11 接口协调

2026-09-28，实施中的协调记录；最终签名以 T06 implementation-report/check-report 为准。

- Usage snapshot、取消 token、active admission 由同一 registry record 持有。
- start admission 在发布 job 前取得，持到 executor 退出和有界 reader cleanup 完成。
- cancel_requested 不是终态；timed_out 和 cleanup_failed 必须由后端和 React 全面消费。
- T06 仅定向生成 UsageImportJobStatus.ts 与 UsageImportJobStage.ts，不修改 handler_registry/manifest。
- T11 的 usage cancel/get 应直接到 owner，不能等待 Usage foreground module 的长 permit。
- T11 接入前检查 T06 独立审查结果，不能根据此记录预先假设接口已通过。
