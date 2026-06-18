# 实施计划：签到记录空白问题

## 目标

以最小改动修复签到记录高级分页查询的列错位问题，并补足前端错误可见性与回归测试。

## 实施步骤

1. 修复后端高级分页 SQL
   - 文件：
     - [crates/ccr-db/src/database/repositories/checkin_repo.rs](D:/Documents/Code/Github/ccr/crates/ccr-db/src/database/repositories/checkin_repo.rs)
   - 操作：
     - 在 `get_records_paginated_advanced`
     - 和 `get_records_filtered_advanced`
     - 的 `SELECT` 中补回 `r.error_code`
     - 并保证列顺序与 `row_to_record()` 完全一致
   - 验证：
     - 编译通过
     - 相关测试通过

2. 新增仓库级回归测试
   - 优先放在：
     - `crates/ccr-db/src/database/repositories/checkin_repo.rs` 现有测试区
     - 或 `crates/ccr-checkin/src/managers/checkin/record_manager.rs` 邻近测试区
   - 覆盖：
     - advanced paginated query
     - advanced filtered query
     - `error_code` / `reward` / `balance_*` / `checked_in_at` 对位正确
   - 验证：
     - `cargo test -p ccr-db`
     - 如改动穿过 manager 层，再补 `cargo test -p ccr-checkin`

3. 暴露前端 records 单项加载失败
   - 文件：
     - [ccr-ui/src/views/checkin/composables/checkinDataState.ts](D:/Documents/Code/Github/ccr/ccr-ui/src/views/checkin/composables/checkinDataState.ts)
     - [ccr-ui/src/views/checkin/tabs/CheckinRecordsTab.vue](D:/Documents/Code/Github/ccr/ccr-ui/src/views/checkin/tabs/CheckinRecordsTab.vue)
     - 必要时 [ccr-ui/src/views/checkin/composables/useCheckinState.ts](D:/Documents/Code/Github/ccr/ccr-ui/src/views/checkin/composables/useCheckinState.ts)
   - 操作：
     - 为 records 请求引入单独错误状态
     - 记录 tab 在失败时显示明确错误，而不是空态
   - 验证：
     - `cd ccr-ui && bun run type-check`
     - `cd ccr-ui && bun run test:smoke`（若已有相关 smoke）

## 验证命令

- `rtk cargo test -p ccr-db`
- `rtk cargo test -p ccr-checkin`
- `cd ccr-ui && rtk bun run type-check`
- `cd ccr-ui && rtk bun run test:smoke`

## 风险点

- `row_to_record()` 被多个查询复用，修复时只能改高级查询列顺序，避免误伤已正确的基础查询。
- 前端错误状态要局部化到 records，不应把 accounts/providers 成功加载的页面整体打成全局错误。

## 回滚点

- 若前端错误展示引入额外 UI 干扰，可保留后端修复和测试，暂缓前端可见性改动。
- 后端列顺序修复不可省略，它是根因修复。
