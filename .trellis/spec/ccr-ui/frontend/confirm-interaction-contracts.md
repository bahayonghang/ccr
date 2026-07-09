# Confirm Interaction Contracts

> 全站确认对话框与提示交互的执行契约。
> 提炼自 `07-07-ui-consistency-sweep`(R1 清除 7 文件 16 处原生 `confirm()/alert()`)。

---

## 1. Scope / Trigger

- 触发:任何需要用户确认的破坏性/大影响操作,或需要向用户提示结果的场景。
- 适用于 `ccr-ui/src` 全部 views / components / composables。

## 2. Signatures

- 闸门式确认(默认路径):`uiStore.requestConfirm({ title, message, confirmText?, cancelText?, type })`
  → Promise\<boolean\>,由 App.vue 挂载的 `GlobalConfirmDialog` 全局渲染 ConfirmModal。
- 局部确认(busy 态场景):`useConfirmAction` + 组件内 `ConfirmModal`(profiles 页模式)。
- toast:`uiStore.showError / showSuccess / showWarning`。
- 兜底扫描:`rg "\b(confirm|alert)\(" ccr-ui/src --glob '!**/*.test.*'`
  ——预期零实弹命中(仅允许注释;当前豁免:`composables/useAgents.ts` 注释、
  `composables/useConfirmAction.ts` 文档注释)。

## 3. Contracts

- **禁止原生 `confirm()` / `alert()` / `prompt()`**。确认走 `requestConfirm`,提示走 toast。
- 两套确认接入方式二选一,不再新增第三种:
  1. `requestConfirm`(默认)——同步决策 → 异步执行、按钮无需 busy 态;
  2. `useConfirmAction` + 局部 ConfirmModal——确认按钮需要 busy 态或对话框内容需自定义时。
- 语义分级(验收口径):

  | type      | 场景                                                          |
  | --------- | ------------------------------------------------------------- |
  | `danger`  | 删除/不可逆(删账号、删服务器组、删 agent、删斜杠命令、删插件) |
  | `warning` | 影响面大但可逆/可重试(project scope 写入与导入、切换官方账号) |
  | `info`    | 纯信息确认                                                    |

- 提示类分级:校验拦截用 `showWarning`,失败用 `showError`,成功用 `showSuccess`。
  多行失败清单不塞 toast——升级为界面内联结果列表(参照 `McpPresetsPanel` 安装结果)。
- **composable 不得触达对话框**:不 import ConfirmModal、不调 `requestConfirm`。
  `delete*` 类函数是纯执行器(执行 + 结果 toast + 刷新,返回 `Promise<boolean>`),
  确认决策上移到消费视图的事件处理器(参照 `PlatformMcpView.handleDeleteServer`)。
  toast 仍允许在 composable 内(消息通道,不是交互决策)。
- 页面局部手搓 modal(自绘 backdrop + panel)应迁 `BaseModal`
  (焦点陷阱/Esc/滚动锁/标准动效内置;参照 `ClaudeAuthView` 保存表单)。

## 4. Validation & Error Matrix

- 新增原生 confirm/alert → 兜底扫描出现新实弹命中,不接受。
- 删除类操作用了 `warning`/`info` → 语义分级违约;取消按钮必须真正不执行。
- composable 内出现 `requestConfirm` 或 ConfirmModal import → 边界违约,决策上移。
- 手搓 fixed-inset backdrop 新 modal → 缺焦点管理与滚动锁,改用 BaseModal。

## 5. Good/Base/Bad Cases

- Good:视图 handler `const ok = await uiStore.requestConfirm({ ..., type: 'danger' }); if (ok) await deleteX(...)`。
- Base:存量 `useConfirmAction` 局部模式(profiles 页)保留,不强迁。
- Bad:composable 里 `if (!confirm(msg)) return` 再执行删除。
- Bad:`uiStore.showError(title + '\n\n' + 多行失败清单)`——改内联列表。

## 6. Tests Required

- `rg "\b(confirm|alert)\(" ccr-ui/src --glob '!**/*.test.*'` 零实弹。
- `cd ccr-ui && bun run type-check && bun run lint`。
- 手测:确认对话框 danger/warning 语义正确、取消不执行、toast 出现。
