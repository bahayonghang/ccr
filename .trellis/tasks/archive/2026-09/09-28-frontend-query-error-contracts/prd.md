# T09：前端查询、编辑会话与错误状态契约

## 目标与用户价值

Grok Auth 查询失败可恢复，Settings 草稿不被刷新覆盖，配置页语言切换实时生效。

## 已确认事实

当前源码和已执行隔离测试的证据见父任务 research；并发、故障及 OS 后果按报告标明推断或未验证。
- 问题映射：F06、F07、F08。
- 代码证据：`ccr-ui/src/features/platform/auth/BaseAuth.tsx:15-67`；`ccr-ui/src/features/platform/settings/BaseSettings.tsx:26-41`；`ccr-ui/src/features/configs/ConfigsView.tsx:32-38`。

## 范围和兼容约束

- 覆盖：ccr-ui/src/features/platform/auth/BaseAuth.tsx；ccr-ui/src/configs/auth.ts 与 Grok Auth adapter；ccr-ui/src/features/platform/settings/BaseSettings.tsx；Settings snapshot/draft 与环境 query key；ccr-ui/src/features/configs/ 的 locale 订阅。
- 保留当前平台支持范围、配置格式、秘密脱敏、备份、原子写、CAS、ACL 和用户确认；新行为须由本文件验收约束。
- 不改写真实用户配置；测试仅使用临时目录、合成账户和受控进程。
- 不重做整个 crate 图、React 架构、视觉系统或全部 legacy IPC；不清理其他任务的文件。

## 前置任务

T03 / 09-28-tauri-config-adapter；T08 / 09-28-settings-lossless-capabilities

父子关系表示范围归属；本节和 meta.depends_on 才表示顺序。前置任务的相关契约通过验收后再进入实施。

## 需求

- R1：Grok Auth 的 probe/load/refresh/off 必须分别呈现 pending、失败、未登录、已登录和不支持状态。
- R2：Settings 服务端快照与编辑草稿独立，refetch 和环境切换不得无提示丢弃输入。
- R3：触及的 Configs 表面随语言变化订阅更新，不依赖重挂载或查询数据改变。

## 验收标准

- [ ] AC1（R1）：probe 或 load 拒绝后出现错误及正确重试入口，不显示确定的 signedOut 或无限 loading；退出失败保留旧状态且不重复提交。
- [ ] AC2（R2）：dirty 表单收到后台 refetch 时保留已编辑字段；旧环境响应不覆盖新会话；仅保存成功或用户明确放弃后重置。
- [ ] AC3（R3）：保持配置数据引用不变，从 zh-CN 切换 en-US 后 tab、summary 和已挂载卡片文案全部更新。

## 不在本任务范围

生产部署、发布、提交和归档；未列入范围的新平台；未经测量的性能优化；恢复跨应用重启的后台任务；修改原有 Insights 任务状态。

## 规划状态

2026-09-28 用户已批准父任务及本子任务的最新规划；本任务仍须满足前置契约后按依赖激活。当前状态以 task.json 为准，技术验证在 design.md 和 implement.md 中明确；实施和检查结果不能由文档存在推定通过。

## R2 scope clarification — 2026-09-28

The affected Claude settings command boundary and single-environment read/update helper are included to enforce the approved editor-session contract. Codex/OpenCode retain their existing local-only backend capability; the frontend must report that capability accurately. Acceptance covers old-session rejection, no redirection during asynchronous writes, and draft retention. No new remote platform support is authorized.
