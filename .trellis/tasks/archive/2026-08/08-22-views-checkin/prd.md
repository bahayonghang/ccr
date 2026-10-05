# CheckIn 视图迁移

> 父任务：`08-22-react-migration`

## Goal

将 CheckIn 签到的全部视图与组件从 Vue 迁移到 React，约 8,607 行，并保证 OAuth 向导与 WAF WebView bypass 流程在迁移后仍可用。

## Scope

| 文件 / 目录 | 行数 |
|---|---|
| `src/views/checkin/`（11 文件，含 `AccountFormModal.vue` 1,054、`OAuthWizardModal.vue` 1,050、`CheckinAccountDashboardView.vue` 1,048、`CheckinProvidersTab.vue` 1,007） | 6,861 |
| `src/views/CheckinView.vue` | 1,452 |
| `src/components/CheckinProgressModal.vue` | 294 |
| 合计 | 8,607 |

关联但不在本任务范围的资产：`src/styles/checkin-shared.css`（1,136 字节，由 `08-22-design-system` 落位）、`src-tauri/src/commands/checkin.rs` 与 `waf.rs`（不改动）。

## Requirements

- R1 上表 13 个文件迁移为 React 组件，对应 `.vue` 文件删除。
- R2 本批次内的 `v-model` 展开为受控属性与回调对，slot 转为 children 或 render props。
- R3 消费 `08-22-design-system` 产出的原语与 token，本批次不新增硬编码样式值。
- R4 IPC 调用点沿用 `src/api` 现有 wrapper，不新增或修改 wrapper。
- R5 `checkin-ux-contracts.md` 定义的交互行为在迁移后成立。
- R6 WAF WebView bypass 流程可用。前端侧的事件等待逻辑（对应 `checkin-waf-event-wait.smoke.test.ts`）语义不变，`src-tauri/src/commands/waf.rs` 不改动。
- R7 OAuth 向导（`OAuthWizardModal.vue` 1,050 行）的多步流程、状态机与错误分支行为不变。
- R8 凭据与 Cookie 的脱敏行为不变（`logRedact.ts`），日志与界面不显示明文凭据。
- R9 签到进度推送沿用现有 Tauri Event，事件名不变，页面卸载后订阅正确解绑。
- R10 余额查询队列行为不变（对应 `checkin-balance-queue.smoke.test.ts`）。

## Acceptance Criteria

- [x] AC1 上表 13 个文件全部迁移，`rg --files -g '*.vue' src/views/checkin src/views/CheckinView.vue src/components/CheckinProgressModal.vue` 无匹配。
- [x] AC2 CheckIn 相关路由全部可达，页面渲染无报错。
- [x] AC3 核心操作路径手动验证通过并记录：账号添加、账号编辑、OAuth 向导完整走通、手动签到、批量签到、余额查询、签到记录浏览、Provider 配置。
- [ ] AC4 WAF WebView bypass 在真实签到流程中验证通过。政策边界：凭据未提供，不伪造；WAF wait / cookie smoke 通过。
- [x] AC5 OAuth 向导的每一步与每个错误分支逐个验证，记录落盘。
- [x] AC6 日志与界面中无明文凭据与 Cookie，由 smoke 测试断言。
- [x] AC7 签到进度事件订阅在页面卸载后正确解绑，挂载卸载 100 次后监听器数量回到基线。
- [x] AC8 本批次组件内 px 字面量与 `rgba()` 数量为 0（登记豁免除外）。
- [x] AC9 `src/api` 的 git diff 为空，`src-tauri` 的 git diff 为空。
- [x] AC10 `bun run type-check` 与 `bun run lint` 退出码 0。
- [x] AC11 CheckIn 相关 smoke 测试通过：`checkin-accounts-tab`、`checkin-balance-queue`、`checkin-cookie-fix`、`checkin-progress-modal`、`checkin-records-api`、`checkin-runtime-coverage`、`checkin-state`、`checkin-waf-event-wait`。

## 前置与后续

- 前置：`08-22-shell-port`。
- 可与其余六个视图子任务并行。
- i18n 调用点在本批次内同步转换，运行时切换与收尾校验属 `08-22-i18n-port`。

## Out of Scope

- 新增功能与信息架构调整。
- `src/api` 与 `src/types` 的修改。
- `src-tauri/src/commands/checkin.rs`、`waf.rs` 的改动。
- `crates/ccr-checkin/` 的改动。
- 加密与凭据存储实现（属 `crates/ccr-db`，不改动）。

## Notes

- 本批次含 8 个 CheckIn 相关 smoke 测试，是全仓测试密度最高的域。测试重写与实现迁移需同步推进，不宜先迁实现后补测试。
- WAF WebView bypass 依赖 WebView 的实际行为，自动化测试无法完全覆盖，需在 `08-22-regression-release` 中再次人工验证。
- `CheckinProgressModal.vue` 原位于 `src/components/` 根目录，本任务将其归入 CheckIn 域，迁移后建议移到 `src/views/checkin/components/` 或对应 React 目录。
