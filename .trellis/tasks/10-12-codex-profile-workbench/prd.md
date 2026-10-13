# Codex Profile 运行态与信息工作台优化

## Goal

让 Codex Profile 的当前运行态、配置比较和 provider 用量在一屏内清晰可读，切换期间保持响应。父任务：[TUI Profile workbench](../10-12-tui-profile-workbench/prd.md)。

## Background

截图直接来自 Codex Profile。改动前 runtime banner 为两行文字；Profile 列表无表头；详情单列。代码证据及数据语义见父任务 research，不重复建立用量/历史数据源。

## Requirements

- C1：重新设计 Current runtime 的 Mode、Profile/Model、Auth 三段层级，覆盖 unbound、profile-only、profile-with-auth、pending-auth、unresolved。
- C2：复用共享对齐表格，增加上次使用列；宽度足够时显示模型/描述；当前配置与选中行不同状态。
- C3：详情采用父任务第二版 A：身份与连接、provider 用量、启用记录；model 与 reasoning 相邻，活动时间使用成功启用记录口径。
- C4：用量显示 source=Codex 且 provider 精确匹配的累计请求、token 拆分和费用估算；显示 provider/all-time/approx 限定，不冒充当前 Profile 的账单。
- C5：Codex 品牌位置去除通用电脑 emoji；复用父任务文本/颜色及官方图形兼容策略。
- C6：保持原有认证独立性、脱敏、runtime drift 判断、切换 outcome 和后台性能合同。

## Acceptance Criteria

- [x] A-C1：所有 runtime 状态在 EN/ZH 和两主题有可读语义；选择其他行不改变当前运行态。
- [x] A-C2：表头/末次时间对齐、分页和首末行鼠标命中准确；当前与选中可区分。
- [x] A-C3：reasoning 的已知/未知/缺失/无效值仍按类型表达；无密钥或 URL 凭据泄露。
- [x] A-C4：两 Profile 同 provider 时相同聚合且范围说明可见；空 provider 不落到 unattributed 桶；刷新失败可见。
- [x] A-C5：成功/附属告警/恢复要求都在实际适配器路径验证；切换中不冻结输入，且不重复执行。
- [x] A-C6：80×20、100×30、140×30、200×50 及父任务边界矩阵通过实际渲染检查。

## Out of scope

不重做 Codex Auth 主体，不修改认证写入/配额策略，不向 provider 用量中混入 auth-account 或 global 的数值。

## Dependencies and status

用户已批准三平台采用 A 并开始实施。共享 table/runtime/detail、只读活动快照和异步 Profile 操作已落地；平台呈现由同一视觉负责人依次整合，最终校验由父任务统一记录。

验收依据：父任务 [整合验证](../10-12-tui-profile-workbench/research/integration-validation.md) 与 [独立复核](../10-12-tui-profile-workbench/research/check-final.md)。专项回归和工作区测试已通过；全仓 CI 分项已通过（完整运行加修复后续跑，详见父任务记录）。图像来自实际 TestBackend 单元格，原生终端字体效果尚未验证。
