# WS4.5 CodexAuthView 深度拆分至 ≤1000 — 拆分计划与结构图

## 现状

`src/views/CodexAuthView.vue` = **3320 行**（template 1-1315 / script 1316-2666 / style 2668-3320）。
前序 WS4.5（commit d0c4c5f0）已抽出双 Tab 模板（CodexAuthAccountsTab / CodexAuthProvidersTab），
script 逻辑全部保留在主视图，故仍远超 ≤1000。

## 为什么不宜在多任务会话中急做

- 安全网仅 `codex-auth-view.smoke.test.ts` 6 个用例，覆盖薄。
- 共享响应式状态 ~30+ 个 ref/reactive，add-account/oauth 子系统与 `handleRefresh` /
  `applyMutationSuccess` / `extractErrorMessage` / `tf` / `uiStore` 深度耦合。
- 安全拆法是「整子系统迁出」而非穿 30 个 props。属于专门的高风险重构，应独立会话执行。

## 三个 BaseModal（template 边界）

| Modal                  | template 行            | 状态/handler（script）                                                                                                                                                                                                                                  | 迁出目标                                                                    |
| ---------------------- | ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| Save current session   | 318-473（~155）        | `showSaveForm` `saveForm` `processWarning` `handleSave/CloseSaveForm/ConfirmSave`（1944-1999）                                                                                                                                                          | `SaveCodexSessionModal.vue`，props: `currentInfo`/`canManage`，emit `saved` |
| **Add account wizard** | 475-1193（~718，最大） | `showAddAccountModal` `activeAddMethod` `importForm` `apiKeyForm` `providerForm` `addAccountDraft` `oauth*` + 全部 add\* computed（1572-1635…）+ handlers（openAddAccountModal/close/switchAddMethod/handleStartOauth/import/apiKey/local，2179-~2500） | `AddCodexAccountModal.vue`（整子系统迁入，对外仅 emit `added`/`close`）     |
| Rename                 | 1205-1310（~105）      | `showRenameDialog` `renameSubmitting` `renameError` `renameForm` `canSubmitRename` `handleRename/Close/ConfirmRename`（2078-2157）                                                                                                                      | `RenameCodexAccountModal.vue`，emit `renamed`                               |

## OAuth composable（PRD 点名）

`useCodexOAuthFlow()`：`oauthLoginId/AuthUrl/CallbackUrl/Pending/PortBusy/TimeoutMessage` +
`oauthUnlisteners` + `resetOauthState/refreshOauthPortStatus/handleReleaseOauthPort/handleStartOauth/`
`codexOAuthSubmitCallbackUrl` 监听清理（onBeforeUnmount）。随 AddCodexAccountModal 一同迁出，
由该 modal 内部使用。

## 建议执行顺序（每步 smoke+type-check+lint+commit）

1. Rename → `RenameCodexAccountModal.vue`（最小、最自洽，验证模式）。
2. Save form → `SaveCodexSessionModal.vue`。
3. `useCodexOAuthFlow` composable 抽出（script-only，无模板风险）。
4. Add account → `AddCodexAccountModal.vue`（整子系统 + 用 useCodexOAuthFlow）。
5. 残余：把 accounts/providers 相关 handler 下沉到对应 Tab 或 `useCodexAuthManager` composable，
   主视图收敛为「编排 + 数据加载 + 共享 confirm」，目标 ≤1000。

## 验收

主文件 ≤1000 行；smoke 6/6；OAuth / 4 种添加 / Provider CRUD / 配额手工回归通过。

> 状态（2026-06-14）：✅ 已完成。专注会话按 step0-4 执行（step0 全局样式层先行，规避 Vue scoped
> 不作用于子组件嵌套节点的潜在断样）：
> - step0 抽 `codex-auth-shared.css` 全局层（274171db）
> - step1 `RenameCodexAccountModal`（89e3e060）
> - step2 `SaveCodexSessionModal`（09e57a84）
> - step3 `useCodexOAuthFlow` composable（ab4e6adf）
> - step4 `AddCodexAccountModal` 整子系统（67084f04）→ 主文件 3320 → **982 行（≤1000，AC#8 达成）**
>
> step5（把残余 provider/data 处理再下沉到 composable）未执行：≤1000 已在 step4 达成，主视图已收敛为
> 「编排 + 数据加载 + Provider CRUD + 共享 confirm」，按「简洁优先 / 外科手术式改动」未追加大重构。
> 全程验证：type-check 0 · eslint 0 error（866 warn 债务基线不变）· stylelint 0 · smoke 351/351 · i18n 23/23。
> 偏离记录：PRD 原列 OAuth composable 与 Add 弹窗为两步，实施时仍分两步提交；Add 弹窗账号变更改为可
> await 的 `refreshOnMutation` 回调（替代 emit），保持「刷新完成→关闭」时序、零回归通过 smoke。
