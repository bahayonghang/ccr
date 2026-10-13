# Grok Profile 认证路由与详情优化

## Goal

复用 Profile 工作台，紧凑展示 Grok backend、认证路由和活动，并接入底层已支持的 Grok provider 用量。父任务：[TUI Profile workbench](../10-12-tui-profile-workbench/prd.md)。

## Background

改动前 Grok 详情无 Usage 分组；SourceKind::Grok 和 provider 查询在 ccr-usage 已存在，TUI loader 仅加载 Claude/Codex。能力支持不等于用户数据库已有 Grok 记录。证据见父任务数据研究。

## Requirements

- G1：复用共享表头/辅助线/状态/上次使用列，英文身份为 `Grok Profile`；品牌位置去除火箭等通用 emoji。
- G2：详情展示 model、reasoning、api_backend、context window、backend-search、provider、已脱敏 endpoint，以及 inline/env/session 认证来源。
- G3：运行态只使用已有域层 current/auth evidence，session 模式不是已登录的充分证据；读操作后台执行。
- G4：将 SourceKind::Grok 加入现有 usage 引擎和平台映射，显示 source+provider 精确聚合，明确费用为估算且是 provider 级全时段。
- G5：区分无 provider、无记录、无库/旧 schema/查询失败，不用虚假零值填充；复用只读活动快照。
- G6：保持 Grok 脱敏、CAS、backup/entry/session 安全契约和父任务异步切换语义。

## Acceptance Criteria

- [x] A-G1：共享表格日期/状态准确，分页、点击、中英文列宽和两主题通过。
- [x] A-G2：inline/env/session 三模式和无效值覆盖；URL userinfo/query/fragment 不泄露，inline token 连掩码预览也不新增。
- [x] A-G3：current != selected、runtime drift、session 未登录/未知/错误按已有语义呈现。
- [x] A-G4：Grok fixture 可显示有记录的 provider 用量，跨平台同名 provider 不串数据；无数据不被称为平台不支持。
- [x] A-G5：载入/刷新失败保留正确状态和范围说明，切换选中行不新增 SQL/文件读取。
- [x] A-G6：Grok lifecycle 成功/回放/附属告警/每阶段失败/外部冲突测试保持通过，交互不被延迟 worker 阻塞。

## Out of scope

不实现新 Grok 用量采集器，不触发登录/认证探测来填满页面，不新增密钥预览，不重做 Grok Auth 主体。

## Dependencies and status

用户已批准三平台采用 A 并开始实施。Grok 运行态、路由字段和既有用量投影已整合，复用父任务布局、活动及后台读写；最终校验由父任务统一记录。

验收依据：父任务 [整合验证](../10-12-tui-profile-workbench/research/integration-validation.md) 与 [独立复核](../10-12-tui-profile-workbench/research/check-final.md)。专项回归和工作区测试已通过；全仓 CI 分项已通过（完整运行加修复后续跑，详见父任务记录）。图像来自实际 TestBackend 单元格，原生终端字体效果尚未验证。
