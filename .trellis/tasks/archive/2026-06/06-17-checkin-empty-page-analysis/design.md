# 技术设计：签到记录空白诊断与修复方案

## 症状拆解

用户看到的是同一页面上的两个区域：

1. 顶部统计卡片：展示当前余额、总额度、已消耗。
2. `签到记录` tab：展示签到历史记录。

这两个区域并不共享同一个后端查询：

- 顶部卡片来自 `list_accounts` 返回的账号聚合余额字段。
- 记录 tab 来自 `get_checkin_records` 返回的 `records`。

因此，记录查询损坏时，顶部卡片仍然可以正常显示，这正好解释了截图里的组合症状。

## 根因

根因是“签到记录高级分页 SQL 的列顺序与 `row_to_record()` 的列读取顺序不一致”。

### 具体链路

1. 前端首屏加载调用：
   - `listCheckinRecords({ page: 1, page_size: 100 })`
2. 由于传入了 `page/page_size`，后端进入：
   - `get_checkin_records(... page, page_size ...)`
   - `RecordManager::get_paginated_advanced(...)`
3. SQL 在 [checkin_repo.rs](D:/Documents/Code/Github/ccr/crates/ccr-db/src/database/repositories/checkin_repo.rs) 里选择了：
   - `id, account_id, status, message, reward, balance_before, balance_after, checked_in_at`
4. 但 `row_to_record()` 期望读取的是：
   - `id, account_id, status, message, error_code, reward, balance_before, balance_after, checked_in_at`

### 直接后果

- 高级分页查询少选了 `error_code`，而 `row_to_record()` 还在按第 4/5/6/7/8 列读取。
- 列位错位后：
  - `error_code` 实际拿到了 `reward`
  - `reward` 实际拿到了 `balance_before`
  - `balance_before` 实际拿到了 `balance_after`
  - `balance_after` 实际拿到了 `checked_in_at`
  - `checked_in_at` 试图读取不存在的第 8 列，最终导致行映射失败
- 该失败沿调用链返回给前端 records 请求。

## 为什么页面表现成“空”而不是“报错”

前端在 [checkinDataState.ts](D:/Documents/Code/Github/ccr/ccr-ui/src/views/checkin/composables/checkinDataState.ts) 使用 `Promise.allSettled()`：

- `providers/accounts/stats/builtin` 成功时照常写入状态
- `records` 单项失败时不会设置页面级错误
- 只有“五个请求全部失败”才会把 `error` 设为“加载签到数据失败”

因此单独的 records 请求失败会被静默吞掉，`records` 保持初始空数组，于是记录页落入：

- `records.length === 0` -> “暂无签到记录”

这就是用户看到“像空数据但其实是查询错误”的根因。

## 运行态证据

- 本机数据库 `C:\Users\lyh\.ccr-ui\ccr-ui.db` 存在。
- 表计数：
  - `checkin_records = 766`
  - `checkin_balances = 165`
  - `checkin_accounts = 5`
  - `checkin_providers = 2`
- 说明真实数据并不为空。
- 直接执行“简单查询”可正常返回记录。
- 直接执行“高级分页 SQL”可查出 8 列，但与 `row_to_record()` 期望的 9 列不匹配。

## 最小修复边界

### 必修 1：修正高级分页 SQL 列顺序

在以下两个查询中补回 `r.error_code`，并保持列顺序与 `row_to_record()` 一致：

- `get_records_paginated_advanced`
- `get_records_filtered_advanced`

目标顺序应为：

1. `r.id`
2. `r.account_id`
3. `r.status`
4. `r.message`
5. `r.error_code`
6. `r.reward`
7. `r.balance_before`
8. `r.balance_after`
9. `r.checked_in_at`

### 必修 2：增加回归测试

需要新增仓库测试覆盖“高级分页查询”路径，而不是只覆盖 `get_all_records/get_records_by_account`。

最小测试集：

- 插入带 `error_code` 的失败记录
- 调用 `get_records_paginated_advanced`
- 断言：
  - 返回条数正确
  - `status/error_code/reward/balance_before/balance_after/checked_in_at` 全部对位正确

并补一个 `get_records_filtered_advanced` 的无分页版本测试，防止同类错位。

### 建议 3：前端暴露 records 单项失败

前端不应把“records 请求失败”伪装成“暂无签到记录”。

最小方案：

- 在 `checkinDataState.ts` 里为 records 请求增加单独错误状态，例如 `recordsLoadError`
- `CheckinRecordsTab.vue` 根据该状态优先显示“记录加载失败，请重试”，而不是空态

这是用户感知层修复，不是根因修复，但应一并纳入方案。

## 不建议的假修复

- 只改空态文案
- 只在前端做重试
- 只把 `Promise.allSettled` 改成 `Promise.all`

这些都不能修复 Rust/SQL 列映射错误本身。

## 验证策略

1. Rust 单测验证高级分页查询不再错位。
2. Tauri/Rust 定向测试确认 `get_checkin_records(page,page_size)` 返回正常。
3. 前端 smoke 或手动验证：
   - 记录页能展示历史记录
   - 如后端查询失败，页面显示明确错误而不是“暂无签到记录”
