# T07 实施报告

日期：2026-09-28。分支：dev。基线：34d8a85e0e48b793733835e0304c8ed33940fcee。
用户已批准实施；T11 command/runtime 前置契约由主代理确认独立通过。
本任务没有创建提交、推送、归档或更改任务生命周期。最终独立检查由 check_t07 执行。

## 已实现行为

- 既有 useCommandsStreamStore 成为外壳生命周期内的唯一命令任务 owner。页面选择 job、状态、输出、取消操作和历史，不再建立三组页面级监听。目录继续由 Query 管理。
- start 在首次 await 前同步设置 submitting；同步双击和提交期间重入只能产生一次启动调用。提前到达的终态按 start 响应返回的 job ID 接纳；旧任务事件不能选择新任务。
- 连续 seq 增量幂等合并。由于现行 snapshot 不含 seq watermark，发生缺口、主动对账或 cancel 响应后，后续增量触发权威 snapshot 读取，避免重复拼接已被快照包含的输出。
- shell 使用固定首事件 250 ms 窗口批量提交；owner 合并在途读取并使用固定 250 ms 对账窗口。终态立即提交缓冲，不因连续事件延迟。输出保留每通道 500 行和 512 KiB UTF-8 上限。
- terminal 单调。generation 和 request 编号隔离旧 status/cancel 响应。cancel acknowledgement 不伪造 cancelled；任务运行或提交期间不能清空任务。
- 回页按保留 ID 查询状态。精确 missing-job 错误显示快照过期、保留最后输出并禁用取消；其他读取错误保留任务并提供 Retry。
- 命令监听拒绝显示订阅降级，运行任务通过 status 查询恢复；成功读取后以 1 秒间隔继续，终态和过期停止，读取失败保留 Retry。仅命令区使用该降级处理。
- 历史写入区分 pending/saved/failed。拒绝保留终态、未确认持久化摘要和可见错误，跨路由仍可见。刷新仅重读。相同 job 在本次会话最多提交一次非幂等写入。
- 迟到 listen Promise 仍触发 unlisten；已清理外壳的旧 command callback 不再写 store。

## AC 与证据

| 验收 | 实施证据 | 行为证据 |
| --- | --- | --- |
| AC1：跨路由恢复任务和取消能力 | useCommandsPage.ts 从 store 选择任务，并在挂载时 reconcile；commandJobStore.ts:140、:196 | 实际 CommandsView/MemoryRouter：运行中离页返回恢复输出并取消原 job；离页完成；丢失终态后按 status 恢复 |
| AC2：终态/历史幂等，旧任务隔离，提交防重 | commandJobState.ts:50、:63；commandJobStore.ts:77、:169 | terminal-before-start、同步双击、重复/乱序/缺口、连续输出/在途读取、旧 status/cancel/event、取消非终态、history 读乱序和写拒绝 |
| AC3：异步监听清理 | eventBridge.ts:124、:158、:177 | immediate/deferred/StrictMode unlisten；旧外壳 callback 在迟到 resolve 前也不能更新 store；真实路由重入只保留一组 command listener |
| CP-T07-01 | reportListener 与可见降级反馈；commandJobStore.ts:102、:140、:219 | 仅 finished 订阅拒绝，正常 seq 0 进入 running；不离开路由，status 查询恢复 success 并停止后续查询 |
| CP-T07-02 | pending/saved/failed summary 与 HistoryNotice | addRecentItem 拒绝后没有 saved 提示；保留摘要/输出；重复终态和只读刷新不产生第二次写入 |

公开 start admission 阻止前一次 start 尚未返回时再启动新 job。因此没有声称单独复现“新 job 已启动后更早的 start 响应才到达”的不可达普通调用路径；实际覆盖旧 status/cancel 响应、旧事件和 terminal-before-start。

## 实际检查结果

日志均位于本任务 research/。

| 检查 | 结果 | 日志 |
| --- | --- | --- |
| 初始实际页面反例 | 0 通过 / 3 失败 | initial-red.log |
| 第一轮修复 | 3 / 3 通过 | first-green.log |
| listener/history 失败恢复与相关回归 | 4 文件 / 34 通过 | failure-recovery-tests.log |
| 最终 commands + shell + API 边界/runtime | 19 文件 / 98 通过 | commands-shell-final.log |
| bun run type-check | 通过 | typecheck.log |
| CCR_SKIP_ICON_GENERATION=1 bun run build | 通过；现有大 chunk warning 保留 | build.log |
| bun run lint:ci | 未通过；仅原有 5 条受保护临时脚本 no-console | formal-lint.log |
| bun run test:i18n | 24 / 24；4511 叶子 key | i18n.log |
| bun run check:arch-boundaries | 4 个禁止依赖夹具全部按预期被拒绝 | arch-boundaries.log |
| bun run check:cycles | 734 文件，0 环 | cycles.log |
| bun run lint:style | 通过 | style.log |
| bun run check:style-lines | 通过 | style-lines.log |
| 相关路径 git diff --check | 通过 | diff-check.log |

首次 type-check 暴露新增 historyFailed 与旧文案同名；已改为 historySaveFailed 并通过最终检查。失败工具输出摘录保存在 typecheck-initial-excerpt.txt。

正式 lint 的 5 条错误：.tmp-desktop-probe.mjs:20、24、92；.tmp-insights-visual.mjs:236、381。两个文件的 SHA-256 均与任务基线一致。本任务没有修改、删除或忽略这些文件。局部检查不替代正式 lint。

## 文件范围

- 新增 commandJobState.ts、commandJobStore.ts；扩展原 stores.ts 导出入口并保留 view preferences。
- 修改 useCommandsPage.ts、CommandsComposer.tsx、CommandsLedger.tsx、CommandsView.tsx、queries.ts。既有配置参数下拉经 T03 确认固定 Claude 范围，显式使用 listConfigs('claude')。
- 修改 shell/eventBridge.ts 的命令监听、批处理和清理路径；新增可选 track 结果回调。Usage 监听语义未变。
- 新增 tests/commands 两个 smoke 文件；修改 shell/cache-route 与 event-bridge-leak。
- 增加双语 commands 的 5 个 key，更新 i18n 数量断言。没有修改 Rust DTO、registry、ACL、资源策略或生成客户端。
- 新增 frontend/command-job-lifecycle.md，更新 index 与本任务 implement/check JSONL。

产品和测试于本轮 09:46 冻结。随后 T09 接手共享 locale/i18n 和 eventBridge 环境事件分支；该后续修改不属于 T07 命令区。implementation-evidence.json 记录自有路径 SHA-256、共享路径采集时 SHA-256、保护文件校验和检查结果。

独立检查接手后发现：shell suspend 后迟到的 start/cancel 响应可能重新调度恢复 timer。check_t07 已直接加入 recoveryActive/resume 约束和 2 个真实 Shell 卸载用例，并报告 0/2→2/2。该修复及最终版本检查以 check-report 为准；本报告的 98 项和源码指纹保留为实现者冻结版本证据，不冒充检查者修复后重新运行的结果。

## 明确边界

- 现有 addRecentItem 每次产生新 UUID，没有 job 幂等键。网络响应丢失时可能已提交；前端不会盲重试。本实现保证可见状态和本会话至多一次提交，不承诺数据库不可用时成功保存或跨 IPC exactly-once。
- 未增加跨应用重启恢复，也未持久化 command snapshot、输出或新的秘密字段。
- 本轮没有运行原生 Tauri WebView、实际浏览器视觉验收或全仓 just ci。实际页面交互证据来自 jsdom/MemoryRouter 和模拟 IPC；原生和视觉边界交父任务集成验收。
- 独立检查尚待 check_t07 最终报告；本报告不据此标记任务完成或归档。
