# T08：设置页无损补丁与能力恢复

## 目标与用户价值

设置页仅保存用户修改的字段，保留现有值并呈现平台规定的编辑能力和限制。

## 已确认事实

当前源码和已执行隔离测试的证据见父任务 research；并发、故障及 OS 后果按报告标明推断或未验证。
- 问题映射：F04、F05。
- 代码证据：`ccr-ui/src/configs/settings-codex-map.ts:39`；`ccr-ui/src/configs/settings-codex-map.ts:94`；`ccr-ui/src/configs/settings-types.ts:1`；`ccr-ui/src/features/platform/settings/BaseSettings.tsx:1`。

## 范围和兼容约束

- 覆盖：ccr-ui/src/configs/settings*.ts；ccr-ui/src/features/platform/settings/；各平台 SettingsView 薄壳；现有 ConfigSourcePanel 和 raw-source domain wrapper；相关设置 smoke fixtures。
- 保留当前平台支持范围、配置格式、秘密脱敏、备份、原子写、CAS、ACL 和用户确认；新行为须由本文件验收约束。
- 不改写真实用户配置；测试仅使用临时目录、合成账户和受控进程。
- 不重做整个 crate 图、React 架构、视觉系统或全部 legacy IPC；不清理其他任务的文件。

## 前置任务

无子任务前置；仍须用户批准规划后才可实施。

父子关系表示范围归属；本节和 meta.depends_on 才表示顺序。前置任务的相关契约通过验收后再进入实施。

## 需求

- R1：未编辑字段的值、联合类型和未知扩展不得被表单保存改写。
- R2：托管锁和未知枚举在表单可见，后端依然是权限和校验权威。
- R3：既有 raw-source、策略层说明和 Local-only 限制在实际设置路由中可达。

## 验收标准

- [ ] AC1（R1）：已有 Codex tui.notifications 数组时只修改 model，保存后数组逐项不变；OpenCode 原值 notify 时只修改其他字段，保存请求不携带 autoupdate。
- [ ] AC2（R2）：Grok managed_keys_locked 对应输入禁用并说明原因；未知合法枚举显示当前值且保存其他字段不覆盖。
- [ ] AC3（R3）：支持 raw source 的平台可以从 Settings 打开现有编辑器并处理 CAS 冲突；非 Local 环境不调用 Local-only 设置接口。 Grok typed/raw 保存及 invalid/stale 等失败路径均不产生备份，页面保留现有无备份提示。

## 不在本任务范围

生产部署、发布、提交和归档；未列入范围的新平台；未经测量的性能优化；恢复跨应用重启的后台任务；修改原有 Insights 任务状态。

## 规划状态

2026-09-28 用户已批准父任务及本子任务的最新规划；本任务仍须满足前置契约后按依赖激活。当前状态以 task.json 为准，技术验证在 design.md 和 implement.md 中明确；实施和检查结果不能由文档存在推定通过。
