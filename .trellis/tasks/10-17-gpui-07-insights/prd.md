# GPUI 07：用量与会话观测

## Goal

迁移用量、Insights、会话、监控、预算和定价并实现大数据虚拟化。

## Parent and Dependencies

父任务：[gpui-desktop](../10-17-gpui-desktop/prd.md)。需求映射：R4、R6、R7、R9。

前置：02、03 完成。 父子关系不自动保证依赖完成。

## Requirements

- 迁移首页、Usage/Insights、sessions、Claude observer、monitoring、budget/pricing。
- ccr-usage 读取 llmusage SQLite，已安装 llmusage CLI 执行 sync，维护 NDJSON 与 job 契约。
- 大列表分页/虚拟化、后台查询、有界日志、图表与表格双通道，真实 empty/unindexed/stale/error 状态。

## Acceptance Criteria

- [ ] 7.1：相同 fixture 与 filter 下 Tauri/GPUI 聚合一致，时区、价格、零值与未索引语义一致。
- [ ] 7.2：GPUI/ccr-app 无新增 usage SQL、无 upstream llmusage Rust 依赖；本地 DB 只读 projection owner 不变。
- [ ] 7.3：10,000 行只构建可见区域，S2 日志背压保持交互；图表有确切数值表格，颜色不是唯一编码。
- [ ] 7.4：import admission/cancel_requested/immutable terminal 与五秒清理、一小时 deadline 回归通过。
- [ ] 7.5：session provider fidelity、分页、归档/刷新与环境能力边界准确，台账全部覆盖。

## Out of Scope

不扩大到本任务未列的业务域；不删除旧表面、不改真实用户数据、不降低已有门禁。父任务声明的完整迁移范围由后续任务继续承接。

## Status

planning。技术待验证项由本任务检查闭环，不视作已完成。开始前读父任务 design.md、implement.md、当前子任务全部规划和 curated context。
