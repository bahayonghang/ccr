# 实施计划：备份

前置：P1 独立验收通过。用户已确认本轮保留全部旧备份；不启动数量或期限清理。

白名单：`crates/ccr-codex/src/services/{mod.rs,codex_auth_backup.rs,codex_registry_store.rs,codex_auth_service.rs}`；必要固定文本与断言限 Codex Auth TUI。现有 runtime backup 池不改。

1. 实现单池锁、最新内容去重、同秒序号和私有原子写；注册表/账号备份复用同一窄帮助函数。
2. 保留全部旧备份，测试注册表/各账号/相似名字池隔离和不清理。
3. delete 和 force rename 收集所有必需备份后才删除或移动；传播备份失败。
   沿用 P1 稳定路径/完整身份锁并在锁后重读；补配额刷新中rename等待且新凭据保留回归。只读拒绝在操作锁副作用前。
4. 测试同内容、同秒、并发不覆盖、恢复字节、三个备份阶段故障，保留既有命令语义。
5. 独立 Trellis check，写回 spec 和证据。运行 `cargo test -p ccr-codex --all-features -- --skip export_bindings`、相关 crate strict clippy、`just fmt-check`。最终 workspace 门由父任务统一运行。

仅使用合成临时数据；不清理真实用户目录，不提交或归档。
