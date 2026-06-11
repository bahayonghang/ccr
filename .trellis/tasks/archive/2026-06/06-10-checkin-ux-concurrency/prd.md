# 签到前端体验与并发治理（节流 / 事件化 / 4 态展示 / toast 统一）

> 父任务: [06-10-checkin-optimize-templates](../06-10-checkin-optimize-templates/prd.md) · 工作包 4 + 工作包 5 前端侧 · 依赖: checkin-engine-hardening（4 态契约后端先行）

## Goal

批量操作对站点友好（并发上限 + 节流 + 同站串行）、进度反馈全部事件化、签到结果按 4 态语义准确展示、错误提示通道统一为 toast。

## Requirements

1. **余额刷新并发治理**（`useCheckinState.ts:214-239` 现为无界 `Promise.allSettled`）：
   - 并发上限（建议 5，与后端签到 Semaphore 对齐）；**同 origin 串行、异 origin 并行**（per-key 队列，参考 all-api-hub `runPerKeySequential` ~30 行实现）。
   - minInterval 节流：距 `last_sync_time` < 30s 的账号跳过（手动单账号刷新 force 绕过）。
   - 实现位置取舍：前端队列 或 后端新增批量刷新命令（带 Semaphore + 事件推送），实现时按改动面最小选择，PRD 不锁定。
2. **WAF 补救重试事件化**：`checkinWafRecovery.ts:210-223` 的 500ms × 240 轮询改为复用 `startAndTrackCheckinJob` 的事件监听（`checkin:job-delta/finished/timeout` + 一次对账），删除轮询。
3. **4 态结果展示**：结果面板与进度弹窗按 `success / already_checked / failed / skipped` 分组；`skip_reason` 映射 i18n（zh-CN + en-US 同步补 key）；summary 文案区分「已签到」与「成功」与「跳过」。
4. **toast 统一**：替换 `alert()`（`checkinJobRuntime.ts:194`、`useCheckinState.ts:252/265`）为 `uiStore` toast；错误提示样式与现有 CheckinAccountsTab 的 showError 模式一致。
5. **`AccountManager::list` 去无谓解密**（`account_manager.rs:52-66, 90-114`）：列表路径不再为生成掩码逐账号解密 cookies（掩码改为存储时生成或惰性查询）。后端小改，归入本任务避免与子任务 3 冲突。
6. **Cookie 过期快捷修复入口**（用户截图驱动，2026-06-10：`anyrouter_stumail` 失败卡片显示 `Cookie 过期 / 建议：请更新 Cookie`，但用户需自行找到账号管理 Tab → 找账号 → 编辑）：失败结果卡片与失败记录行的 `cookie_expired` 类错误提供「更新 Cookie」操作，点击直接打开对应账号的编辑弹窗（聚焦 cookies 字段）。低成本高频价值。

## Acceptance Criteria

- [ ] 20 账号批量刷余额时并发不超上限、同站请求串行（单测 per-key 队列；可用计数 mock 断言）。
- [ ] 30s 内重复「全部刷新」时已刷新账号被跳过且 UI 提示跳过数量；单账号手动刷新不受限。
- [ ] WAF 补救重试期间无轮询定时器（代码删除 + smoke 测试改用事件路径，现有 checkin-state.smoke 用例更新仍绿）。
- [ ] 结果面板四组渲染 + skip 原因文案（zh/en）；进度弹窗 recovering 阶段行为不回退（checkin-progress-modal.smoke 仍绿）。
- [ ] 仓库内签到相关 `alert(` 出现次数为 0。
- [ ] `cookie_expired` 失败卡片/记录行可一键打开对应账号编辑弹窗（smoke 测试覆盖入口渲染与点击事件）。
- [ ] `list_accounts` 不再触发逐账号解密（Rust 测试或 trace 断言）；`bun run test` + `just frontend-check-quick` + `cargo test -p ccr-checkin -- --test-threads=1` 绿。

## Out of Scope

- 组件文件拆分与 CSS 治理（子任务 checkin-component-split）。
- 页面级数据缓存策略（保持每次进入全量拉取，必要时另行任务）。
- 定时自动签到调度。

## Technical Notes

- 热点定位：[`../06-10-checkin-optimize-templates/research/internal-checkin-architecture.md`](../06-10-checkin-optimize-templates/research/internal-checkin-architecture.md)（§性能与体验热点）。
- 并发/节流模式：[`research/all-api-hub.md`](../06-10-checkin-optimize-templates/research/all-api-hub.md)（minInterval + force 两档、per-origin 串行队列、签到后小批量补刷余额）。
- 事件系统现状：`checkin:job-delta` 增量推送已存在且设计良好（checkinJobRuntime.ts:132-197），本任务只是让 WAF 重试路径复用它。
- i18n 文件：`ccr-ui/src/i18n/locales/{zh-CN,en-US}.ts`（checkin.\* 命名空间）。
