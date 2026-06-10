# 签到报错链路修复（P0）

> 父任务: [06-10-checkin-optimize-templates](../06-10-checkin-optimize-templates/prd.md) · 工作包 1

## Goal

让签到/余额/记录操作的失败信息端到端不丢失：后端构造的结构化错误（message + error_code）原样到达 UI，「未知错误」只在信息真正缺失时出现；修复记录页假过滤。纯 bugfix，不改架构。

## Requirements

1. **前端错误提取修复（「未知错误」主犯）**：Tauri v2 `invoke()` 对 `Result<_, String>` 以**纯字符串** reject。新建统一工具函数（如 `utils/errorMessage.ts` 的 `toErrorMessage(error, fallback)`）：string → 原样返回；`Error` → `.message`；其他 → fallback。替换 4 处重复实现：
   - `views/checkin/composables/useCheckinState.ts:44-45`
   - `views/checkin/tabs/CheckinAccountsTab.vue:732-733`
   - `views/checkin/tabs/CheckinRecordsTab.vue:325-326`
   - `views/checkin/tabs/CheckinProvidersTab.vue:420-421`
   - 实现前先用一个故意失败的命令实测确认 rejection 形态（research 标注为推断）。
2. **Job 路径 error_code 保留**：`build_failed_checkin_result`（`ccr-ui/src-tauri/src/commands/checkin.rs:86-91`）不再硬编码 `task_error`；`execute_checkin_job_accounts` 的 `Ok(Err(error))` 分支（:231-239）透传 `CheckinServiceError::error_code()`。`task_error` 仅保留给 spawn/JoinSet 基础设施失败。
3. **error_code 分类补测试**：`crates/ccr-checkin/src/core/error.rs:34-65` 的消息关键词分类（waf_blocked/cf_blocked/cookie_expired/api_error/...）是隐式契约且零测试 —— 为每个分类路径补单测，锁住关键词矩阵。
4. **记录筛选/分页接通**（修复假过滤；**已被用户截图实锤**：2026-06-10 截图中「失败历史记录 (5)」面板列表混入两条「成功」记录，且面板计数 (5) 与列表行数不一致 —— 计数来源与列表数据源不同步）：
   - api 层 `listCheckinRecords`（`ccr-ui/src/api/domains/checkin.ts:156-169`）透传 `status / provider_id / keyword / page / page_size`；
   - Tauri `get_checkin_records`（`commands/checkin.rs:695-716`）扩参并改调 `RecordManager::get_paginated_advanced`（`record_manager.rs:66-107`，现存零调用方）；
   - `CheckinRecordsTab.loadFailedHistory`（:405-427）的失败历史面板真实只显示 failed 记录、翻页生效，面板计数与列表同源。
5. **批量刷余额失败可见**：`refreshAllBalances`（`useCheckinState.ts:214-239`）`Promise.allSettled` 后处理 rejected：失败账号计数 + 名单进 UI（toast 汇总或行内状态），不再静默丢弃。

## Acceptance Criteria

- [ ] 后端错误信息（如 `检测到 WAF 挑战页面（响应为 HTML）`）原样出现在签到失败详情/toast 中；前端单测覆盖「字符串 rejection → 原样透传」「Error → message」「其他 → fallback」三分支。
- [ ] Job 内单账号解密失败的 error_code 为 `crypto_error`（而非 `task_error`）；WAF 失败为 `waf_blocked`；Rust 测试断言。
- [ ] `cargo test -p ccr-checkin -- --test-threads=1` 含 error_code 分类矩阵新测试，全绿。
- [ ] 记录页「失败历史」仅显示 failed 记录；按 provider/关键词过滤与翻页生效；有前端回归测试（api 层参数透传断言）。
- [ ] 批量刷余额时断网/无效账号的失败在 UI 可见（数量 + 账号名）。
- [ ] 不引入行为外的重构；`just frontend-check-quick` + `just lint-strict` 绿。

## Out of Scope

- 4 态结果契约、宽容判定归一（子任务 checkin-engine-hardening）。
- toast/alert 通道统一（子任务 checkin-ux-concurrency）；本任务新增提示可直接用 uiStore toast，但不动存量 alert。

## Technical Notes

- **截图佐证的影响面修正**（2026-06-10）：Job 日志路径的错误展示（`getFailedDetail` + error hint）截图显示工作正常 —— stumail 账号正确显示 `Cookie 过期` 标签 + `HTTP 401: 签到失败（建议：请更新 Cookie）`。因此 Requirement 1 的「未知错误」修复主要影响**直连命令路径**（账号/提供商 CRUD、刷余额、记录加载等 catch 分支），不是 Job 结果面板；Requirement 2 的 error_code 覆盖问题影响的是 Job 内基础设施失败的子集。实施时按此校准预期。
- 错误五层转换路径详见 [`../06-10-checkin-optimize-templates/research/internal-checkin-architecture.md`](../06-10-checkin-optimize-templates/research/internal-checkin-architecture.md)（§错误产生与展示路径）。
- 遵守 `.trellis/spec/ccr-checkin/backend/backend-guidelines.md`：保留 WAF/CF/cookie-expired 显式分类；日志不得输出 cookie/token。
- `get_paginated_advanced` 返回结构若与前端期望不一致，以最小适配为准（不顺手重构 RecordManager）。
