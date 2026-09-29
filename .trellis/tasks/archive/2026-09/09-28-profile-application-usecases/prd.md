# T02：跨 CLI TUI Tauri 的 profile 应用用例

## 目标与用户价值

CLI、TUI 和 Tauri 的 profile 切换具有相同校验、副作用和诚实完成结果。

## 已确认事实

当前源码和已执行隔离测试的证据见父任务 research；并发、故障及 OS 后果按报告标明推断或未验证。
- 问题映射：CLI-02、CLI-03、CLI-04、CLI-07、TA-08 profile lifecycle。
- 代码证据：`crates/ccr-tui/src/tui/app.rs:934-993`；`crates/ccr-cli/src/application/profile_switch.rs:60-92`；`crates/ccr-cli/src/application/profile_off.rs:256-261`；`crates/ccr-cli/src/platforms/claude.rs:317-329`。

## 范围和兼容约束

- 覆盖：crates/ccr-cli/src/application/；crates/ccr-cli/src/commands/profile/；crates/ccr-tui/src/tui/app.rs；crates/ccr-cli/src/platforms/；crates/ccr-codex/src/platforms/codex.rs；ccr-ui/src-tauri/src/commands/*profiles.rs。
- 保留当前平台支持范围、配置格式、秘密脱敏、备份、原子写、CAS、ACL 和用户确认；新行为须由本文件验收约束。
- 不改写真实用户配置；测试仅使用临时目录、合成账户和受控进程。
- 不重做整个 crate 图、React 架构、视觉系统或全部 legacy IPC；不清理其他任务的文件。

## 前置任务

T01 / 09-28-config-repository-consistency；T05 / 09-28-safe-persistence-backups

父子关系表示范围归属；本节和 meta.depends_on 才表示顺序。前置任务的相关契约通过验收后再进入实施。

## 需求

- R1：先验证目标和变更计划，再修改现有 runtime；复合 cleanup/apply 的失败不能静默留下旧状态已清除。
- R2：enabled、usage_count、history 与成功定义由共享用例统一决定。
- R3：实际激活后发生历史/审计故障必须可见，调用者不能将已生效误报为完全未生效。
- R4：嵌入式业务调用不得经过终端呈现 handler；保留现有安装 CLI 和公共兼容导出。
- R5：profile rename 的新建、旧名删除与当前项激活由共享生命周期用例定义一致结果。

## 验收标准

- [ ] AC1（R1）：无效、被删除或禁用目标在三个入口均无写入；写入阶段故障产生可验证的回滚或显式可恢复结果。
- [ ] AC2（R2）：相同 fixture 和请求在三个入口产生相同持久状态；实际未生效不得增加成功计数或成功历史。
- [ ] AC3（R3）：历史写失败时结果明确标记 activation 已提交及附属失败，三端均显示相应结果且不自动重做激活。
- [ ] AC4（R4）：profile 的桌面与 TUI 路径均调用同一 application 用例；共享入口没有 stdin、print 或 process::exit。
- [ ] AC5（R5）：rename 在 save-new、delete-old、apply 各阶段失败时结果可解释，current/default 不悬空，CLI/Tauri 同 fixture 后果一致。

## 不在本任务范围

生产部署、发布、提交和归档；未列入范围的新平台；未经测量的性能优化；恢复跨应用重启的后台任务；修改原有 Insights 任务状态。

## 规划状态

2026-09-28 用户已批准父任务及本子任务的最新规划；本任务仍须满足前置契约后按依赖激活。当前状态以 task.json 为准，技术验证在 design.md 和 implement.md 中明确；实施和检查结果不能由文档存在推定通过。
