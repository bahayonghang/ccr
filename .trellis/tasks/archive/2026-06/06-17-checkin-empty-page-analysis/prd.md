# 分析签到页面内容为空

## Goal

深入分析 `ccr-ui` 签到页面中“顶部余额/额度卡片有数据，但签到记录区域显示为空”的真实原因，并产出最小、可验证的修复方案。

## Confirmed Facts

- 截图显示签到页顶部三张统计卡片已有数值，但 `签到记录` 区域显示“暂无签到记录”。
- 前端签到页由 [ccr-ui/src/views/CheckinView.vue](D:/Documents/Code/Github/ccr/ccr-ui/src/views/CheckinView.vue) 驱动，共享状态来自 [ccr-ui/src/views/checkin/composables/useCheckinState.ts](D:/Documents/Code/Github/ccr/ccr-ui/src/views/checkin/composables/useCheckinState.ts)。
- 页面初始加载会并行请求 providers、accounts、records、today stats、builtin providers，逻辑在 [ccr-ui/src/views/checkin/composables/checkinDataState.ts](D:/Documents/Code/Github/ccr/ccr-ui/src/views/checkin/composables/checkinDataState.ts)。
- 顶部统计卡片的金额来自 `accounts[].latest_balance / total_quota / total_consumed` 聚合，而不是来自 `records`。
- 记录页空态只取决于 `records.length === 0`，逻辑在 [ccr-ui/src/views/checkin/tabs/CheckinRecordsTab.vue](D:/Documents/Code/Github/ccr/ccr-ui/src/views/checkin/tabs/CheckinRecordsTab.vue)。
- `records` 的前端调用固定走 `listCheckinRecords({ page: 1, page_size: 100 })`，这会触发后端 `get_checkin_records` 的“高级分页/过滤”分支。
- 该后端命令位于 [ccr-ui/src-tauri/src/commands/checkin.rs](D:/Documents/Code/Github/ccr/ccr-ui/src-tauri/src/commands/checkin.rs)，进一步调用 [crates/ccr-checkin/src/managers/checkin/record_manager.rs](D:/Documents/Code/Github/ccr/crates/ccr-checkin/src/managers/checkin/record_manager.rs) 的 `get_paginated_advanced`。
- 高级分页 SQL 位于 [crates/ccr-db/src/database/repositories/checkin_repo.rs](D:/Documents/Code/Github/ccr/crates/ccr-db/src/database/repositories/checkin_repo.rs)，其 `SELECT` 列顺序与 `row_to_record()` 读取顺序不一致。
- 本机运行态证据：`C:\Users\lyh\.ccr-ui\ccr-ui.db` 中 `checkin_records = 766`、`checkin_balances = 165`、`checkin_accounts = 5`、`checkin_providers = 2`，说明“数据库无记录”不是原因。

## Requirements

- 明确区分“确实无签到记录”与“记录查询路径损坏/被吞掉”。
- 解决方案必须优先修复真实根因，而不是只在前端改空态文案。
- 解决方案必须覆盖前端错误可见性，避免单个请求失败时被误呈现为“空数据”。
- 解决方案必须包含针对高级分页记录查询的回归保护。

## Acceptance Criteria

- [ ] 能用一句话解释为什么“顶部卡片有数据”与“记录区为空”会同时出现。
- [ ] 能指出导致问题的具体文件、函数、条件和字段/列错位位置。
- [ ] 方案中包含最小修复范围、验证方法和必要测试。
- [ ] 方案能防止后续再次把“记录查询失败”误显示为“暂无签到记录”。

## Out Of Scope

- 本次不处理签到页面整体视觉改版。
- 本次不重构签到模块所有数据流。
- 本次不讨论与签到记录无关的余额计算逻辑。
