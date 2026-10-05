# T09 前端实施报告

日期：2026-09-28。基线 commit：`34d8a85e0e48b793733835e0304c8ed33940fcee`。本报告覆盖前端 owner 的 AC1–AC3 实施；后台补充由独立 owner 完成，证据另列。所有改动仍在原共享工作区，未提交、推送或归档。

## 实施结果

### AC1：Auth 查询与退出

- `configs/auth.ts` 保留 `unsupported_environment`，不再把后端不支持结果转换为未登录。`BaseAuth.tsx` 区分 probe、首次 load、旧缓存刷新、未登录、已登录及不支持状态。
- probe 或首次 load 失败显示可重试错误。已有成功缓存的刷新失败保留会话，并显示旧数据及错误反馈。Retry 使用当前界面语言。
- probe 重试先将会话 Query 标记为失效，再允许成功 probe 触发读取；独立检查验证生产 `staleTime: 30_000` 下可以恢复。
- 退出操作在确认前取得同步提交锁，防止重复请求；失败保留会话并反馈错误。不支持响应不能显示退出成功。

### AC2：Settings 编辑会话与环境边界

- 使用既有 `['current-environment']` Query 作为环境身份来源。probe 和数据 key 包含配置、环境 id 和类型；未增加持久环境 store。
- `useSettingsSession.ts` 维护已经确认的环境和 snapshot。React Hook Form 维护草稿及 dirty 字段。后台刷新不重置既有会话；服务器 snapshot 变化时显示提示并冻结保存。
- snapshot 比较使用 `replaceEqualDeep`。等值数据或 Query GC 后的新引用不构成假冲突。只有成功保存或用户明确放弃后，才重新建立基线。
- 环境身份正在刷新、查询失败或环境改变时保留草稿并冻结写入。提供返回草稿原环境、明确放弃后重新读取。返回操作等到有效且空闲的 probe/data；页面按钮和直接表单提交均受检查。
- session generation 与环境 Query 的 `dataUpdateCount` 隔离异步响应，覆盖 A→B→A。读取和保存传入已经确认的环境 id，并检查完成后的身份。保存成功但随后读取失败时禁止重复保存，要求重新读取。
- Claude typed Settings 的实际 generated 调用发送 `expectedEnvironmentId`。后台 owner 在命令入口匹配环境并捕获同一个环境 Arc；前端成功不替代后台约束。Codex/OpenCode 的 typed Settings 准确标记既有 Local-only 能力。
- raw editor 在后续 probe 失败或不支持时保留同一个 CodeMirror 实例、内容和 CAS token，冻结保存并提供重试。过期响应不能覆盖内容、token、基线或显示成功；返回原环境不会重复确认明文或读取。
- 环境事件先刷新环境身份，取消旧 Settings 读取，再失效其他数据。T07 的命令事件处理和清理逻辑保持其 owner 的实现。
- 保留 T08 的 dirty-leaf/null、managed 锁定、未知值、unsupported-clear、原子保存和 raw CAS/确认契约。验证错误继续使用既有 Toast 通道。

### AC3：Configs 语言订阅

- Configs 页面、hook、过滤器、卡片、标记和子导航使用 `useAppT`；翻译结果的 memo 依赖包含 `t`。
- 行为测试在同一页面实例中切换 zh-CN→en-US→zh-CN，保持 Query 数据引用及已有卡片 DOM，检查 tab、summary、标记、说明和操作文案。列表读取次数保持为 1。
- 保留 T03 的显式 Claude 请求、版本化编辑草稿和启用流程。

## 文件与规范

当前文件 SHA256、共享文件说明和受保护文件校验见 `research/implementation-files.json`。hash 表示共享工作区在交接时的完整文件状态，不表示其他 owner 的变更属于 T09。

- Auth：`configs/auth.ts`、`features/platform/auth/BaseAuth.tsx`、`features/platform/SurfacePage.tsx`。
- Settings：`configs/environmentSession.ts`、Settings descriptors/types、Claude domain wrapper、`features/platform/settings/` 中本任务 session/form/source 文件、`features/platform/editor/ConfigSourcePanel.tsx`。
- Configs：页面、hook 及 6 个翻译消费者。共享变更还包括环境 event bridge、两种 locale 和 i18n key 数量。两种 locale 均为 4517 个叶子 key。
- 规范：`platform-surface-contracts.md`、`raw-config-editor-contracts.md`、`react-rerender-discipline.md`。
- 回归：Auth、Settings typed/raw session、Configs locale、实际 Claude IPC 参数、原有平台基线和能力用例。checker 拥有的两份回归文件在清单中单独标识。

## 前端 owner 执行的检查

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| `bun run test:smoke -- tests/platforms tests/configs tests/shell tests/api tests/editor tests/ui/confirm-interaction.smoke.test.tsx` | 66 文件、277/277 通过；47.91 秒 | `frontend-smoke-final.log` |
| `bun run type-check` | exit 0 | `frontend-typecheck-final.log` |
| `bun run check:cycles` | 737 文件，无循环 | `frontend-cycles.log` |
| `bun run check:arch-boundaries` | 4 个违规 fixture 均被拒绝 | `frontend-boundaries.log` |
| `bun run test:i18n` | 24/24；4517 key；key 泄漏自测通过 | `frontend-i18n.log` |
| `bun run lint:style` | exit 0 | `frontend-style.log` |
| `bun run lint` | exit 1；5 个原有错误、2 个原有 warning | `frontend-lint-final.log` |
| `CCR_SKIP_ICON_GENERATION=1 bun run build` | exit 0；Vite 2.01 秒；存在分块大小 warning | `frontend-build.log` |

正式 lint 仍失败。5 个错误全部位于两份受保护的原有 `.tmp` 文件（`no-console`）；2 个 warning 位于 `scripts/measure-distribution.mjs:44`、`tests/checkin/checkin-accounts-tab.smoke.test.tsx:82`。未删除、修改或排除这些文件。style 单独通过不改变正式 lint 状态。

先前中间失败日志保留：`frontend-smoke.log` 的 274/276、`frontend-typecheck.log` 的 QueryFn 参数错误及 lint complexity 诊断均由上述最终结果替代。修复保留行为断言，未加入 lint disable。

## 独立及后台证据

- checker 的 Auth 生产缓存用例先红后绿，证据为 `research/check-auth-probe-production-cache.log`、`research/check-auth-probe-fixed.log`。
- checker 的返回环境等待数据及直接提交用例先红后绿，证据为 `research/check-settings-return-pending.log`、`research/check-settings-return-pending-submit.log`。
- 后台 owner 实施和验证详见 `backend-implementation-report.md`、`research/backend-verification.json`。owner 报告 Claude 模块 39/39（其中 17 个 binding exports，6 个新增环境回归）、registry 21/21、正式 bin Clippy 和 fmt 通过；all-targets Clippy 的原有 4 个错误保持单独记录。
- 独立后台检查记录在 `research/check-backend-results.json`：22 个行为测试、21 个 registry 测试、正式 bin Clippy、fmt 通过。前端 owner 未执行这些 Cargo 命令。
- 独立前端检查及审查结论由 `/root/check_t09` 写入；本报告不预先判定独立检查完成。

## 验证边界与交接

root 已通过实际 Web preview 观察 `/grok/auth` 的 Tauri 不可用错误和 Retry 入口；此观察不能验证真实 native IPC。未执行原生桌面、真实账户、SSH/WSL transport 集成或真实用户配置写入。未执行完整 `just ci`。

前端源码与测试已冻结，生产构建通过。当前交接包括独立检查、父任务 requirement-to-evidence ledger 和 T10 跨域集成；任务状态、PRD 验收勾选及最终完整 gate 由 root 协调。
