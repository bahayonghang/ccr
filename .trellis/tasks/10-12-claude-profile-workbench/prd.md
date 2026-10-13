# Claude Profile 标题与配置活动视图优化

## Goal

统一 Claude Profile 页面身份与共享工作台，使模型映射、认证模式、活动和 provider 用量易于读取。父任务：[TUI Profile workbench](../10-12-tui-profile-workbench/prd.md)。

## Background

改动前 `PlatformTab::display_label` 英文返回 `Claude Code`；已有 ClaudeRuntimeSummary 和 ProfileConfig 提供配置/认证字段，用量由共享引擎缓存。

## Requirements

- L1：英文页面标题、完整 tab 标签及相关反馈身份统一 `Claude Profile`；中文为 `Claude 配置`，Claude Auth 不改名。
- L2：复用共享表格及末次成功启用时间、表头和辅助线，状态/current/selection 一致。
- L3：顶部/详情使用已有 runtime summary 展示真实 current、模式和认证来源，区分 official subscription 与 API-key profile。
- L4：详情突出默认模型、已配置的各模型映射/子代理模型/effort/timeout 等，以紧凑分组显示；缺省不假定为实际运行值。
- L5：展示真实活动快照和 source=Claude/provider 聚合，包括 token 拆分及估算口径。品牌位置去除机器人 emoji。
- L6：继承父任务异步加载/切换与事务安全，保留凭据优先级和脱敏。

## Acceptance Criteria

- [x] A-L1：EN 页面身份为 `Claude Profile`，compact label 与 ZH 可读；tab-order 配置 ID 和 Claude Auth 标签保持兼容。
- [x] A-L2：表格日期与历史平台键正确，分页/鼠标/选中记忆验证通过。
- [x] A-L3：API key、official、待认证、冲突/未知/错误均有诚实状态；不把选中 Profile 当生效配置。
- [x] A-L4：有值的模型映射可读，未设项压缩；不输出 env 值、密钥、credentials 或任意 platform_data。
- [x] A-L5：provider 相同/缺失、无库、旧 schema、错误和刷新状态覆盖；估算范围在窄屏仍可读。
- [x] A-L6：默认并行的安全契约与异步状态测试通过，两语言/主题和父任务尺寸矩阵有渲染证据。

## Out of scope

不改 Claude Auth 功能、官方 CLI 产品名称、认证优先级和持久化格式；不以全局字符串替换把所有 Claude Code 文档/消息改名。

## Dependencies and status

用户已批准三平台采用 A 并开始实施。Claude 标题和平台字段已由共享视觉负责人在 Codex 后整合，继承父任务布局、活动快照和异步适配器；最终校验由父任务统一记录。

验收依据：父任务 [整合验证](../10-12-tui-profile-workbench/research/integration-validation.md) 与 [独立复核](../10-12-tui-profile-workbench/research/check-final.md)。专项回归和工作区测试已通过；全仓 CI 分项已通过（完整运行加修复后续跑，详见父任务记录）。图像来自实际 TestBackend 单元格，原生终端字体效果尚未验证。
