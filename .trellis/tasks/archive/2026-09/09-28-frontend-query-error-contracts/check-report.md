# T09 独立检查报告

日期：2026-09-28。检查角色：`/root/check_t09`。基线：`34d8a85e0e48b793733835e0304c8ed33940fcee`。本轮没有提交，`fix_commit=null`。

## 结论

T09 的 R1–R3 / AC1–AC3 通过当前范围的源码与自动化独立检查。最终检查没有发现尚未处理的本任务产品阻断项。前端正式 lint 仍因两份受保护的原有脚本失败；额外后端 all-targets Clippy 有 4 项既有测试 lint。原生 WebView、真实 SSH/WSL 保存、完整 bindings guard 和父任务完整 CI 不在本报告的已通过结论内。

已读取指定任务的 PRD、design、implement、check.jsonl 和全部注入规范；没有沿用后来可能切换的 current-task 指针。检查覆盖真实 Settings/Auth/Configs 消费链、环境事件桥、Claude typed IPC 参数、后端环境选择和受控双环境行为。前端和后端分别冻结后执行独立检查。

## Findings (fixed)

### 1. Claude Settings 缺少环境入场约束，远端更新会重复选取 active

- File：`ccr-ui/src-tauri/src/commands/claude.rs`、`claude_settings.rs`、`handler_registry.rs`；`ccr-ui/src/configs/settings-claude.ts`、`src/api/domains/claude.ts`。
- Issue：原更新逻辑在分支选择、读取、写入分别获取当前环境。切换可使 A 的内容写入 B。前端单独查询环境无法关闭查询完成到 IPC 入场之间的窗口。源证据见 `research/check-environment-owner-matrix.md`。
- Fix：主线程分派的后端 owner 为两个 Settings 命令增加兼容的可选 `expectedEnvironmentId`；在 registry 读锁中比对并捕获同一个执行环境 Arc；后续远端读取和写入绑定该对象。Local 路径保留 `SettingsManager::update_atomic_async`。前端编辑器明确传递已确认的环境 ID；generated client 和 registry 保持一致。
- Evidence：owner 的双 Barrier 旧反例实际失败，A 写次数为 0；检查者阅读该源码快照和日志。最终独立执行 22 项 Claude 行为测试全部通过，其中 6 项覆盖入场 mismatch 的零 I/O 拒绝、读写固定目标、A→B→A 对象替换、空 registry 兼容和 Local 原子写。

### 2. Auth 探针恢复没有刷新仍处于生产缓存有效期内的会话

- File：`ccr-ui/src/features/platform/auth/BaseAuth.tsx`。
- Issue：只将 dependent query 的 enabled 设为 false 再恢复 true，不能保证刷新生产 `staleTime: 30_000` 内的旧会话。显式 Retry 可清除错误，却继续显示旧 signed-in 状态。
- Fix：前端 owner 在重试探针前将对应 session cache 标记 stale，使用 `refetchType: 'none'`，探针成功后只启用一次实际会话重读。
- Evidence：检查者新增 `tests/platforms/auth-probe-recovery-check.smoke.test.tsx`。生产缓存条件下实际 1 项失败，修复后通过；断言第二次 load 与 signed-out，不只检查探针次数。日志：`research/check-auth-probe-production-cache.log`、`check-auth-probe-fixed.log`。早期 staleTime 为 0 的通过结果不足以证明该场景，保留但未用作结论。

### 3. 返回原环境时，快照重读完成前可提交旧草稿

- File：`ccr-ui/src/features/platform/settings/useSettingsSession.ts`、`BaseSettings.tsx`。
- Issue：A 草稿切换到 B 后返回 A，环境 identity 先恢复，A 的 values Query 仍可处于无数据的 pending 状态。原 UI 已开放保存。
- Fix：前端 owner 引入 `snapshotReady`；UI Save 和实际执行 guard 同时要求当前 probe/read 已成功且空闲。保留原草稿，读到外部变化后进入明确冲突提示。
- Evidence：检查者新增 `tests/platforms/settings-return-cache-check.smoke.test.tsx`。返回后挂起的原环境读取实际复现保存按钮错误开放；修复后 2 项通过，覆盖直接 form submit 不调用 mutation，以及原 cache 被回收后等值响应不产生伪冲突。日志：`research/check-settings-return-pending.log`、`check-settings-return-pending-submit.log`。

### 4. 原文草稿、环境 pending 和平台能力边界补齐

- File：`SettingsSource.tsx`、`ConfigSourcePanel.tsx`、`useSettingsSession.ts`、`settings-codex.ts`、`settings-opencode.ts`。
- Issue：环境 pending 的 UI 需要订阅 Query fetching 状态；原文 probe 后续 unsupported 不能卸载已打开 editor；固定本地路径的 Codex/OpenCode typed Settings 不能按远程能力呈现。
- Fix：前端 owner 显式订阅环境状态，保留已打开的真实 CodeMirror 与原 token，在 probe 错误/unsupported 时设为只读并提供重试。Codex/OpenCode 使用 Local-only capability/probe。快照采用结构比较，避免 Query GC 后新对象误报服务器变更。
- Evidence：最终独立测试包含真实 CodeMirror、环境事件桥、pending/error、返回原环境、明确 discard、迟到读取、原文 probe 两类失败和 Local-only 零平台读取。没有把源码推断描述为原生复现。

## 逐项验收

| 验收 | 最终核对机制 | 独立测试证据 |
| --- | --- | --- |
| AC1 / R1 | probe/load error 优先；Grok backend unsupported 保留；旧成功数据显示 stale；off 在确认前同步占用，失败保留旧状态 | `query-contract-baseline`、`auth-query-contract`、`auth-probe-recovery-check`；实际 Grok adapter 和生产缓存条件 |
| AC2 / R2 | 服务端快照、已确认 baseline 和 draft 分离；环境 id/type Query key；pending/error/change 冻结；明确 discard；返回原环境重读；迟到结果检查 generation/revision；Claude 后端绑定目标 | `settings-session`、`settings-return-cache-check`、`settings-source-session`、`settings-capabilities`；后端 `commands::claude::environment_tests` |
| AC3 / R3 | Configs 消费者使用响应式翻译，memo 依赖翻译身份 | `configs-actions` 在同一 page/card DOM 和同一 Query 数据引用中执行 zh-CN→en-US→zh-CN，检查 tabs、summary、badges、卡片操作文案，list_configs 只调用一次 |

## Findings (not fixed)

- **正式前端 lint 基线失败**：`ccr-ui/.tmp-desktop-probe.mjs` 第 20、24、92 行及 `.tmp-insights-visual.mjs` 第 236、381 行的 5 条 `no-console`。这些文件由主线程明确保护，检查者未删除、修改或添加忽略规则。两个 SHA256 与原始保护记录一致。
- **额外 all-targets Clippy 基线失败**：owner 执行记录有 `main.rs` 和 `state.rs` 的 `items_after_test_module`、Claude 旧测试的 `bool_assert_comparison`、Codex 旧测试的 `await_holding_lock`。检查者对照 HEAD 核实相关原代码已存在。本轮未扩大到无关测试整理；正式 `--bin ccr-desktop` strict Clippy 独立通过。
- **能力与验证边界**：可选 expected ID 的缺省调用保留兼容；ID 绑定逻辑目标，不是跨请求 revision CAS。已入场操作可继续完成原目标，前端负责拒绝旧响应。真实 SSH/WSL、原生 WebView、完整 bindings guard 和父任务 CI 尚未由本检查执行。

## Verification

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| Claude 行为测试，跳过 binding export | **22/22 passed**，含新增 6 项环境行为；附属 test target 的 0 匹配不计入通过数量 | `research/check-backend-behavior.log` |
| handler registry | **21/21 passed** | `research/check-backend-registry.log` |
| 正式 backend bin strict Clippy | **pass** | `research/check-backend-clippy.log` |
| Tauri fmt check | **pass** | `research/check-backend-fmt.log` |
| 前端相关 smoke | **67 files / 279 tests passed** | `research/check-frontend-smoke.log` |
| TypeCheck | **pass** | `research/check-frontend-type.log` |
| 正式 `bun run lint:ci` | **fail**，仅上述 5 条受保护脚本错误；后续 `&&` 样式步骤没有执行 | `research/check-frontend-lint.log` |
| 单独 stylelint | **pass**，不替代正式 lint 结论 | `research/check-frontend-style.log` |
| i18n | **24/24 passed**，4517 个叶子 key；泄漏检查自测通过 | `research/check-frontend-i18n.log` |
| 循环依赖 / 边界 | **pass**，737 个文件；4 项违规 fixture 正确拒绝 | `research/check-frontend-cycles.log`、`check-frontend-boundaries.log` |
| `git diff --check` | **pass**；输出保留行尾提示 | `research/check-diff.log` |

精确命令和退出码：`research/check-backend-results.json`、`research/check-frontend-results.json`。机器可读结论：`research/check-summary.json`。100 个相关源码/测试/保护文件在独立检查前后 SHA256 不变，见 `research/check-source-before.json`、`check-source-after.json`。

已核对更新后的 platform-surface、raw-config-editor、react-rerender 规范与实现一致。独立检查者新增两个行为回归文件和本报告/证据，未覆盖实现者在途源码，未更改任务生命周期、原有 Insights 任务或保护脚本。父任务可继续 T10 集成；本报告不将范围内通过扩大为全仓、发布或原生验收通过。
