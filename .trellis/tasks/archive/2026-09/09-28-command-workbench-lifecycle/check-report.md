# T07 独立检查报告

- 检查日期：2026-09-28。
- 任务：命令工作台任务恢复与事件生命周期。
- 检查者：`check_t07`；使用 `.agents/skills/trellis-check/SKILL.md`。
- 结论：T07 的 AC1–AC3 在独立自动化行为验证中通过。两项早期反馈已由实现者修复；最终检查又复现并修复两项恢复调度缺陷。全局正式 lint 和 type-check 在检查时仍有任务外失败，因此不能据此宣布仓库门禁全部通过。
- 范围：命令页/store/reducer、shell 命令事件区、typed command API 调用链和对应规范。保留 T09 环境分支及其他并行修改；未改 Rust、DTO、registry、生成物、保护文件或任务生命周期。

## Findings (fixed)

### CP-T07-01 — 命令监听注册失败没有可见恢复入口

- 文件：`ccr-ui/src/shell/eventBridge.ts` 的 `trackCommand`；`ccr-ui/src/features/commands/commandJobStore.ts` 的 `reportListener`、`reconcile`、`start`；`CommandsLedger.tsx` 的 `JobNotice`。
- 问题：原检查版本只记录 listener rejection，正常连续 seq 不会触发 status 对账；终态监听失败时任务可保持 running 且没有恢复提示。
- 修复归属：实现者加入按事件保存的 listener failure、可见降级提示、现有 status 对账入口和成功读取后的 1 秒恢复查询。terminal/expired 停止查询；注册成功只清除自己的错误。
- 独立验证：真实 CommandsView + MemoryRouter + shell bridge 场景证明 finished listener 拒绝后，无需切换路由即可从 status 恢复终态，并且只提交一次 history。最终 102 项组合回归包含该场景。

### CP-T07-02 — 历史写入失败只有日志

- 文件：`ccr-ui/src/features/commands/commandJobStore.ts:79`，`ccr-ui/src/features/commands/CommandsLedger.tsx:20`。
- 问题：原检查版本在写入前永久认领 job ID；拒绝后没有可见状态。该限制来自旧实现。
- 修复归属：实现者区分 pending/saved/failed，保留失败摘要和真实终态；仅成功响应可以标记 saved。失败跨路由可见，刷新只读，不重复写入。
- 独立验证：受控 addRecentItem Promise 先 pending 后 reject，界面不显示 saved；重复终态、只读刷新和路由重入仍只提交一次。
- 已确认边界：history API 每次生成新 UUID，没有 job 幂等键。本任务保证会话内至多一次提交和诚实的失败状态，不承诺数据库故障下成功保存，也不承诺跨 IPC 持久 exactly-once。

### CP-T07-03 — Shell 卸载后迟到响应重启恢复查询

- 文件：`ccr-ui/src/features/commands/commandJobStore.ts:61`、`:105`、`:144`、`:241-242`；`ccr-ui/src/shell/eventBridge.ts:169`。
- 问题：原 suspend 只清除当时的 timer。挂起 start 在卸载后返回，会因 listener failure 重新调度；挂起 cancel 的 finally 也会调度。两个真实 Shell 卸载反例分别在卸载后触发 2 次和 1 次 status 请求。
- 检查者修复：增加 shell recovery 活跃标记及 resume/suspend；调度和对账均检查该标记。迟到 start 仍保留真实 job 身份，shell 重挂载时恢复对账，不丢弃尚未返回的已启动任务。修改只落在命令区。
- 红绿证据：`research/check-late-response-red.log` 为 0/2；`check-late-response-green.log` 为 2/2。回归还证明 cancel 迟到响应后重挂载恢复终态并只写一次 history。
- 测试：`ccr-ui/tests/commands/command-workbench-lifecycle.smoke.test.tsx:404`、`:447`。

### CP-T07-04 — 对账失败后已排队的定时器仍自动重试

- 文件：`ccr-ui/src/features/commands/commandJobStore.ts:155`。
- 问题：listener 失败先安排 250 ms 恢复；用户在定时器前手动 Retry，status 拒绝后旧定时器仍然存在。未收到新事件时，2 秒内产生 3 次 status 请求并覆盖首次错误。该行为不符合“失败后等新事件或明确重试”的契约。
- 检查者修复：当前 status 读取失败时同步取消已有恢复 timer，保留错误及快照；新事件或用户重试仍可继续恢复。
- 红绿证据：`research/check-retry-timer-red.log` 为 0/1；最终组合回归通过该反例，断言没有自动额外请求，显式 Retry 恢复终态。
- 测试：`ccr-ui/tests/commands/command-workbench-lifecycle.smoke.test.tsx:423`。
- 规范：同步 `.trellis/spec/ccr-ui/frontend/command-job-lifecycle.md` 的卸载、迟到响应和失败定时器约束。

## Findings (not fixed)

1. **全局 type-check 在检查时失败，属于 T09 在途改动。** `BaseSettings.tsx:77` 的 source panel props 尚未包含 environment；`useSettingsSession.ts:147` 的对象不满足 SettingsSnapshot。已向主代理报告，未修改其他 owner 的代码。保留 `check-typecheck.log`。该结果不能用实现者更早版本的 type-check 通过记录替代。
2. **正式 lint 在检查时失败 11 条。** 5 条来自受保护基线 `.tmp-desktop-probe.mjs:20,24,92` 和 `.tmp-insights-visual.mjs:236,381`。另 6 条属于 T09 在途 BaseAuth、ConfigSourcePanel、BaseSettings、useSettingsSession 的 complexity/exhaustive-deps。已报告主代理；未删除、忽略或修改相关文件。保留 `check-formal-lint.log`。局部 lint 通过不等于正式门禁通过。
3. **原生与浏览器验收仍需父任务集成。** 本轮没有操作 Tauri WebView 或真实配置，也没有执行实际浏览器交互/截图。真实 React 组件测试使用 jsdom/MemoryRouter 和模拟 IPC，不能替代原生/视觉验收。

T07 检查范围内没有另外保留的已确认代码缺陷。仓库全量 CI、T09 的最终修复与类型检查由对应 owner 和 T10 继续完成。

## Verification

| 项目 | 独立结果 | 证据 |
| --- | --- | --- |
| 新增 Shell 卸载反例 | 0/2 → 2/2 | check-late-response-red.log、check-late-response-green.log |
| 失败后的旧 timer 反例 | 0/1 → 最终组合通过 | check-retry-timer-red.log、check-final-regression.log |
| 最终 commands/shell/API facade boundary + coverage | 19 文件、102 测试通过 | check-final-regression.log |
| T07 scoped ESLint | 通过 | check-scoped-lint-final.log |
| 正式 lint:ci | 失败：基线 5 + T09 在途 6 | check-formal-lint.log |
| 全局 type-check | 失败：T09 在途 2 处 | check-typecheck.log |
| 架构边界夹具 | 4 项按预期拒绝 | check-boundaries.log |
| 循环依赖 | 737 文件，无环 | check-cycles.log |
| CCR_SKIP_ICON_GENERATION=1 bun run build | 通过；大 chunk warning 保留 | check-build.log |
| Scoped git diff --check | 通过；LF/CRLF 提示保留 | check-diff.log |

最终命令：

```text
bun run test:smoke -- tests/commands tests/shell tests/api/api-facade-boundary.smoke.test.ts tests/api/api-facade-coverage.smoke.test.ts
```

- AC1：运行中路由离开/返回保留 ID、状态和输出，取消仍指向原 job；离页完成由 shell 记录终态。
- AC2：terminal-before-start、同步重复提交、重复/旧 seq、gap、snapshot 无 watermark、持续输出固定窗口、旧 status/cancel/event、过期与瞬态失败、取消非终态、history 状态和一次提交均在最终组合内通过。
- AC3：即时/延迟 listen、StrictMode、清理后回调及重新挂载覆盖；新增迟到 start/cancel 反例确保卸载后不再调度。
- 公开 start admission 阻止前一 start 尚未返回时启动下一 job；不声称单独覆盖公开路径不可达的“新 job 生效后更旧 start 响应返回”。

## 版本与保护边界

`research/check-final-regression-evidence.json` 记录最终回归前后 16 个路径及 shell 命令区的 SHA-256；所有指纹一致。`research/check-results.json` 记录最终检查结果与报告生成时的源码指纹。早期/中间测试和实现者的 98 项结果均单独保留，不累加为独立唯一测试数量。

两份保护文件与基线一致：

- `.tmp-desktop-probe.mjs`：`02e1aea2ad36cfd1d56e62852f1462d0f9c96a439a1380eaf03e0d4b14912c85`。
- `.tmp-insights-visual.mjs`：`ac8fe1e9e9de9738fbc5b2ce44f4d66322eb7d83b83823a38168314c41f47d53`。

未执行 bindings/inventory 生成、commit、push、archive 或 task lifecycle 操作。
