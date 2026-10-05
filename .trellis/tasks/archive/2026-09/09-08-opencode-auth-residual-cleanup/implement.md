# 清理执行

- [x] 父任务最终获批后启动本子任务，读取配置/Codex/TUI 规范与父研究。
- [x] 检查引用和工作树，删除三处专属遗留，更新混合 fixture。
- [x] 更新对应当前文档，复核没有误删有效功能；共享 TUI 规范修订交由 TUI 子任务集成。
- [x] 静态检查引用与改动边界，并经独立 trellis-check 复核无待修发现；用户要求本轮不测试，以下命令仅作后续参考，全部不执行并记 SKIPPED（用户要求）：

```powershell
rtk cargo test -p ccr-config tui_config -- --test-threads=1
rtk cargo test -p ccr-codex utils -- --test-threads=1
rtk cargo test -p ccr-tui theme -- --test-threads=1
rtk cargo test -p ccr-tui app::tests -- --test-threads=1
rtk proxy just fmt-check
```

本轮父子任务均不运行 just ci、测试或构建。独占 tui_config/utils/theme 及上述文档，其他子任务完成后不回滚其改动。不安装、启动 UI、提交或发布，不操作当前 Grok Build、真实凭据或配置。

源码清理与静态审查已交付；行为验收仍 UNVERIFIED，PRD AC 不勾选。任务保留 in_progress，未提交/归档。
