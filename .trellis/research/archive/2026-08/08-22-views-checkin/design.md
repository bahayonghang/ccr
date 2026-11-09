# 技术设计：CheckIn 视图迁移

> 父任务：`08-22-react-migration`。本域不进统一层，8,607 行全部迁移。测试密度全仓最高（8 个 smoke 测试），且含两处自动化无法完全覆盖的流程。

## 1. 本批次共性转换

与 `08-22-views-claude` 的 `design.md` §1 同表，不重复。前置阅读同。

## 2. 范围

13 个文件，8,607 行。四个最大文件：`AccountFormModal`(1,054)、`OAuthWizardModal`(1,050)、`CheckinAccountDashboardView`(1,048)、`CheckinProvidersTab`(1,007)，以及 `CheckinView`(1,452)。

本域不受 `08-22-platform-unify` 影响（其「前置与后续」明确），范围表不回填。

`CheckinProgressModal.vue`(294) 原位于 `src/components/` 根目录，迁移后归入 CheckIn 域（PRD Notes）。新位置按 `path-mapping.md` 的映射。

## 3. OAuth 向导的状态机（`OAuthWizardModal` 1,050 行）

R7：多步流程、状态机与错误分支行为不变。

设计：

- 状态机用 `useReducer`，不用多个 `useState`。理由是步骤转移有前置条件（如「已获取授权码」才能进「交换 token」），`useReducer` 让转移集中在一处可审查，多个 `useState` 会把转移条件散到各处。
- 每个错误分支对应一个 reducer action。错误分支清单从现有实现抽出，逐条对应（AC5 要求逐步与逐分支验证）。
- 步骤间的表单数据用单个 react-hook-form 实例跨步骤共享。
- 等待外部事件的步骤（WAF bypass、OAuth 回调）的超时与取消由 reducer 的超时 action 处理，不用裸 `setTimeout` 散在组件内。

**抽取错误分支清单是本节的前置动作**：不先抽出清单，迁移后无法逐条验证（AC5）。清单落盘为 `oauth-wizard-branches.md`。

## 4. WAF WebView bypass（R6）

前端侧的事件等待逻辑语义不变（对应 `checkin-waf-event-wait.smoke.test.ts`）。`src-tauri/src/commands/waf.rs` 不改动。

设计要点：

- 事件等待走 `08-22-state-logic-port` 的事件桥接层还是组件级 `listen()`：WAF 等待是一次性的、与特定向导实例绑定的等待，不是全局数据流。因此用组件级 `listen()` + `useEffect` cleanup，不进全局桥接层。
- **必须登记到前端事件 inventory**（协同点 M）。不进桥接层不等于不登记：`08-22-test-contract-rebuild` AC6 的「全部 Tauri Event 名」断言若只扫 `shell/eventBridge.ts`，本处事件会被漏掉。登记字段：事件名、所有者（本组件路径）、生命周期（一次性）、对应的 Rust `emit` 位置（`src-tauri/src/commands/waf.rs`）。本任务产出局部部分，14 合并。
- **取消协议**：`listen()` 返回 `Promise<UnlistenFn>`，组件级用法同样适用 `08-22-state-logic-port` §3 的 `disposed` + 迟到 unlisten 立即调用协议。向导弹层的挂载卸载比全局桥接层频繁，该时序更容易命中。
- StrictMode 下 effect 双调用会建立两次等待。等待逻辑必须幂等——同一事件到达时只推进状态机一次（reducer 的转移条件天然提供幂等：已在下一步时重复 action 无效）。
- 超时与取消路径保留。

WAF bypass 依赖 WebView 的实际行为，自动化测试无法完全覆盖（PRD Notes）。因此需两次人工验证：本任务一次（AC4），`08-22-regression-release` 在打包产物上再一次（其 AC6）。

## 5. 凭据与 Cookie 脱敏（R8）

界面与日志均不显示明文凭据与 Cookie。

实现路径：`logRedact.ts`（框架无关，原样复用）。本任务只保证不绕过——不在 React 侧新增直接 `console.log` 凭据对象的路径。

AC6 由 smoke 测试断言。断言方式：注入含凭据字段的对象，检查 `logger` 输出与渲染结果中无明文。

`AccountFormModal`(1,054) 是凭据输入的主要位置，其字段的掩码显示逐个核对。

## 6. 签到进度事件（R9）

事件名不变，页面卸载后订阅正确解绑（AC7：挂载卸载 100 次后监听器数量回到基线）。

进度事件是持续推送的数据流，与第 4 节的一次性等待不同。因此走全局事件桥接层 + Query 的 `setQueryData`（payload 含进度数据）。`CheckinProgressModal` 订阅该 Query。

## 7. 余额查询队列（R10）

行为不变（对应 `checkin-balance-queue.smoke.test.ts`）。

队列是并发控制逻辑（限制同时查询的账号数）。判定其归属：若队列状态跨组件共享，进 Zustand；若只在一个视图内，用 `useRef` 持有队列 + `useState` 持有可见状态。归属判定应已在 `08-22-arch-quality-perf` 的 `state-disposition.md` 中（若队列实现在某个 composable 内）。若不在，本任务按上述判据判定并记录。

队列的并发上限值与重试策略不变。

## 8. 测试与实现同步推进

PRD Notes：本批次含 8 个 CheckIn 相关 smoke 测试，是全仓测试密度最高的域。测试重写与实现迁移需同步推进，不宜先迁实现后补测试。

8 个测试：`checkin-accounts-tab`、`checkin-balance-queue`、`checkin-cookie-fix`、`checkin-progress-modal`、`checkin-records-api`、`checkin-runtime-coverage`、`checkin-state`、`checkin-waf-event-wait`。

同步推进的操作形态：每个提交批次内包含该批次涉及的测试重写。测试重写稿由 `08-22-test-contract-rebuild` 提供（协同点 D），本任务负责让其通过。

## 9. 契约

`checkin-ux-contracts.md`（7.2 KB）定义的交互行为在迁移后成立（R5）。重写稿由 `08-22-test-contract-rebuild` 提供。

## 10. 不变量

- IPC 调用点沿用现有 wrapper（R4）。`git diff --stat src/api src/types` 须为空（AC9）。
- `src-tauri/src/commands/checkin.rs` 与 `waf.rs` 不改（AC9 的 `src-tauri` diff 为空）。
- `crates/ccr-checkin/` 不改。
- 加密与凭据存储实现（`crates/ccr-db`）不改。
- `src/styles/checkin-shared.css`（1,136 字节）由 `08-22-design-system` 落位，本任务不改该文件。

## 11. 未决项

- 余额查询队列的状态归属（第 7 节），按 `state-disposition.md` 或本任务判据确定。
- OAuth 向导的错误分支数量，抽取后确定。
- WAF 等待的幂等性在 StrictMode 下的实际表现，实施时验证。
