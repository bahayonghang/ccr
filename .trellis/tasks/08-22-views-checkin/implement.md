# 执行计划：CheckIn 视图迁移

> 父任务：`08-22-react-migration`（阶段 5，七个视图子任务并行）。
> 分支：`feature/react-migration/views-checkin`，PR 目标 `feature/react-migration`。
>
> 本域测试密度全仓最高。测试重写与实现迁移同批次推进，不先迁实现后补测试。

## 前置确认

- [ ] 父任务统一层门已通过（本域范围不受影响，无需回填）。
- [ ] 前置阅读完成（`08-22-views-claude/design.md` §1 末段的五份文档）。
- [ ] 前置阅读：`.trellis/spec/ccr-ui/frontend/react-rerender-discipline.md`（R8，动手前必读）。
- [ ] `08-22-test-contract-rebuild` 已提供 `checkin-ux-contracts.md` 重写稿与 8 个 smoke 测试的重写稿。
- [ ] `08-22-design-system` 已落位 `checkin-shared.css`（1,136 字节）。
- [ ] `git checkout -b feature/react-migration/views-checkin feature/react-migration`

## 批次 0：OAuth 向导错误分支抽取

在迁移 `OAuthWizardModal` 之前完成。不先抽清单则迁移后无法逐条验证（AC5）。

- [ ] 从现有 `OAuthWizardModal.vue`(1,050) 抽出全部步骤与错误分支。
- [ ] `oauth-wizard-branches.md` 落盘：步骤清单 + 每步的错误分支 + 触发条件 + 期望呈现。
- [ ] 抽出 WAF 等待与 OAuth 回调两处外部事件等待的超时与取消路径。

## 批次 1：账号与 Provider

- [ ] `AccountFormModal`(1,054)：表单用 react-hook-form，凭据字段掩码显示逐个核对。
- [ ] `CheckinProvidersTab`(1,007)。
- [ ] 同批次重写 `checkin-accounts-tab`、`checkin-cookie-fix` 两个 smoke 测试。

验证：账号添加与编辑走通；Provider 配置走通；两个测试通过。

## 批次 2：OAuth 向导

- [ ] `OAuthWizardModal`(1,050) 迁移，状态机用 `useReducer`（`design.md` §3）。
- [ ] 每个错误分支对应一个 reducer action，与批次 0 的清单逐条对应。
- [ ] 步骤间表单数据用单个 react-hook-form 实例共享。
- [ ] 超时与取消由 reducer 的超时 action 处理，不用裸 `setTimeout`。
- [ ] WAF 等待用组件级 `listen()` + `useEffect` cleanup，不进全局桥接层（`design.md` §4）。
- [ ] 按 `08-22-state-logic-port` §3 的取消协议实现：`disposed` 标记 + 迟到 resolve 的 unlisten 立即调用。
- [ ] 该事件登记到前端事件 inventory 的**局部部分**（事件名、所有者组件路径、一次性生命周期、Rust `emit` 位置 `src-tauri/src/commands/waf.rs`），交 `08-22-test-contract-rebuild` 合并（协同点 M）。
- [ ] StrictMode 下等待幂等性验证：同一事件重复到达只推进一次状态机。
- [ ] 同批次重写 `checkin-waf-event-wait` smoke 测试。

验证：向导每一步与每个错误分支逐个验证并记录（AC5）；WAF bypass 在真实签到流程中验证通过（AC4）。

### AC4 的验证方式

WAF bypass 依赖 WebView 实际行为，需真实账号与真实签到请求。`bun run tauri dev` 下执行一次完整签到，记录 bypass 是否完成。该验证在打包产物上由 `08-22-regression-release` AC6 再做一次。

## 批次 3：看板与主视图

- [ ] `CheckinAccountDashboardView`(1,048)。
- [ ] `CheckinView`(1,452)。
- [ ] 余额查询队列按 `design.md` §7 判定归属并迁移，并发上限与重试策略不变。
- [ ] 同批次重写 `checkin-balance-queue`、`checkin-state`、`checkin-records-api` 三个 smoke 测试。

验证：手动签到、批量签到、余额查询、签到记录浏览走通。

## 批次 4：进度弹层与剩余文件

- [ ] `CheckinProgressModal`(294) 迁移，位置按 `path-mapping.md` 归入 CheckIn 域。
- [ ] 进度事件走全局事件桥接层 + Query 的 `setQueryData`（`design.md` §6）。事件名不变。
- [ ] `views/checkin/` 剩余文件。
- [ ] 同批次重写 `checkin-progress-modal`、`checkin-runtime-coverage` 两个 smoke 测试。
- [ ] 订阅解绑验证：挂载卸载 100 次后监听器数量回到基线（AC7）。

## 批次 5：脱敏与收口

- [ ] 凭据与 Cookie 脱敏断言：注入含凭据字段的对象，检查 `logger` 输出与渲染结果无明文（AC6）。
- [ ] 本批次组件内 px 与 `rgba()` 归零，豁免逐条登记（AC8）。
- [ ] `checkin-ux-contracts.md` 的断言逐条验证（R5）。
- [ ] `rg --files -g '*.vue' src/views/checkin src/views/CheckinView.vue src/components/CheckinProgressModal.vue` 无匹配（AC1）。
- [ ] `git diff --stat src/api src/types src-tauri`（应全为空，AC9）。

## 验证命令

| 时机      | 命令                                                                                                        |
| --------- | ----------------------------------------------------------------------------------------------------------- |
| 每批次后  | `bun run type-check`、`bun run lint`（AC10）、`bun run test:smoke`                                          |
| 批次 2 后 | `bun run tauri dev` 下真实签到一次（AC4）                                                                   |
| 批次 5 后 | `rg --files -g '*.vue' src/views/checkin src/views/CheckinView.vue src/components/CheckinProgressModal.vue` |
| 交付前    | `just frontend-check-quick`、`bun run lint:ci`、8 个 CheckIn smoke 测试全通过（AC11）                       |

## 交付门（父任务视图门的一部分）

- [ ] AC1–AC11 全部满足。
- [ ] AC3 的 8 条核心操作路径逐条验证并记录：账号添加、账号编辑、OAuth 向导完整走通、手动签到、批量签到、余额查询、签到记录浏览、Provider 配置。
- [ ] `oauth-wizard-branches.md` 落盘，向导每步与每分支逐个验证（AC5）。
- [ ] WAF bypass 真实签到验证通过（AC4）。
- [ ] 8 个 CheckIn smoke 测试通过（AC11）。
- [ ] 脱敏断言通过（AC6），订阅解绑断言通过（AC7）。
- [ ] `src/api`、`src/types`、`src-tauri` git diff 为空。
- [ ] 余额查询队列的状态归属判定记录落盘。

## 回滚点

六个批次各自独立提交。批次 2（OAuth 向导）内可按向导步骤分多次提交。

每批次含该批次的测试重写，因此回滚一个批次同时回滚其测试，不留悬空测试。

## 协同点

| 编号 | 内容                                          | 对方                          | 时机         |
| ---- | --------------------------------------------- | ----------------------------- | ------------ |
| D    | `checkin-ux-contracts.md` 与 8 个测试的重写稿 | `08-22-test-contract-rebuild` | 前置与每批次 |
| I    | i18n 调用形式                                 | `08-22-i18n-port`             | 全程         |
| —    | `checkin-shared.css` 落位                     | `08-22-design-system`         | 前置         |
| —    | 进度事件走全局桥接层                          | `08-22-state-logic-port`      | 批次 4       |
| M    | WAF 局部事件登记到统一前端事件 inventory      | `08-22-test-contract-rebuild` | 批次 3       |
| —    | WAF bypass 在打包产物上复验                   | `08-22-regression-release`    | 交付后       |
