# T10：架构回归门禁与规范同步

## 目标与用户价值

优化结果可通过跨入口行为和完整桌面门禁验收，规范与可执行契约一致。

## 已确认事实

当前源码和已执行隔离测试的证据见父任务 research；并发、故障及 OS 后果按报告标明推断或未验证。
- 问题映射：G-01、G-02、G-03、CLI-10。
- 代码证据：`justfile:570-584`；`justfile:1545-1551`；`.github/workflows/tauri-rust-ci.yml:27-49`；`.trellis/spec/ccr/backend/tauri-handler-registry.md:150`；`.trellis/spec/ccr/backend/dependency-governance.md:361`。

## 范围和兼容约束

- 覆盖：justfile 与 ccr-ui/justfile 的门禁组合；scripts/ci/ 与 scripts/quality/；受影响 .trellis/spec/ 与 code_map.md；跨入口 contract fixtures 和实际行为 tests；父任务集成验证记录。
- 保留当前平台支持范围、配置格式、秘密脱敏、备份、原子写、CAS、ACL 和用户确认；新行为须由本文件验收约束。
- 不改写真实用户配置；测试仅使用临时目录、合成账户和受控进程。
- 不重做整个 crate 图、React 架构、视觉系统或全部 legacy IPC；不清理其他任务的文件。

## 前置任务

T01 / 09-28-config-repository-consistency；T02 / 09-28-profile-application-usecases；T03 / 09-28-tauri-config-adapter；T04 / 09-28-cli-diagnostics-contract；T05 / 09-28-safe-persistence-backups；T06 / 09-28-usage-job-lifecycle；T07 / 09-28-command-workbench-lifecycle；T08 / 09-28-settings-lossless-capabilities；T09 / 09-28-frontend-query-error-contracts；T11 / 09-28-desktop-control-oauth-lifecycle

父子关系表示范围归属；本节和 meta.depends_on 才表示顺序。前置任务的相关契约通过验收后再进入实施。

## 需求

- R1：跨模块最终验收覆盖根 workspace 和独立 Tauri workspace 的严格检查。
- R2：每个 P1 发现拥有证明原失败的行为回归，跨入口副作用可比较。
- R3：规范路径、命令计数、包 owner、能力矩阵与当前实现一致。
- R4：验证过程不隐藏工作区基线失败，不改变已有未完成任务的生命周期。

## 验收标准

- [ ] AC1（R1）：文档化 aggregate 包含 desktop clippy 和全量行为测试；注入 desktop 测试失败时 aggregate 非零退出。
- [ ] AC2（R2）：所有 P1 在 evidence ledger 关联旧反例、测试、修复 commit 和结果；不存在仅凭 mock 成功或静态计数验收的条目。
- [ ] AC3（R3）：过期 Vue 当前路径、336/315/323 等冲突描述得到修订；规范链接可解析，ccr-usage 职责在索引可找到。
- [ ] AC4（R4）：正式 lint 与受控诊断分别记录；原有临时脚本和 Insights 任务状态保持独立处理。

## 不在本任务范围

生产部署、发布、提交和归档；未列入范围的新平台；未经测量的性能优化；恢复跨应用重启的后台任务；修改原有 Insights 任务状态。

## 规划状态

2026-09-28 用户已批准父任务及本子任务的最新规划；本任务仍须满足前置契约后按依赖激活。当前状态以 task.json 为准，技术验证在 design.md 和 implement.md 中明确；实施和检查结果不能由文档存在推定通过。
