# Tauri S2 Monitoring consumer 收据

## 边界与冻结

基线 HEAD `998dd9f2204abe99a3f468a857c362fb9e6b8120`。复用既有父任务；本子批仅拥有 React consumer、直接测试、两套文案及必要严格词条计数。没有修改 Rust、生成客户端/DTO、IPC/domain wrapper、bootstrap 或13 destinations。Backend/生成由 performance_fixture owner 持有。

Frontend 生产/测试/locale 输入已于 **2026-11-06 18:28:21 UTC** 明确冻结给主会话完整CI。最后生产改动约18:27:25 UTC，随后只有只读检查、spec/research及自有 localhost 清理。未提交。

## 文件与行为

- `ccr-ui/src/features/monitoring/BenchmarkS2Panel.tsx`：独立子树，只有 `isTauriRuntime()` 且 bootstrap boolean严格为true 才挂载。普通Web、Local、旧Fixture、S0/S1均零S2控件/零S2请求。三动作直接导入已生成的 `api/domains/benchmark`；未使用临时生产stub或冻结的tauri facade。
- `MonitoringView.tsx:5,164`：仅导入与挂载新面板，保留现有监控布局/日志/用量。
- `BenchmarkS2Panel.tsx:44`：同一已知run且严格更新revision才采纳数据。挂载/重连先完成listener注册再query；离页只detach，迟到注册立即detach，不取消producer。状态由进程owner恢复，终态没有Reset/Restart。
- `BenchmarkS2Panel.tsx:59`：query成功恢复连接错误与payload采纳分开；相等的not_started revision0也可恢复。query/action均捕获action generation；旧成功/失败不能清除或覆盖新动作错误。数据的旧revision/异run拒绝规则保留。
- `BenchmarkS2Panel.tsx:89`：Start仅not_started，并复用 `surfaceNotify.confirm`→GlobalConfirmDialog/ConfirmModal的warning确认，说明fixture写入和每进程一次。只有明确确认后调用Start。取消只发当前run_id请求，Cleaning继续等待实际owner终态，不提前宣布取消成功。
- `BenchmarkS2Panel.tsx:202`：稳定failure_code映射安全文案，未知值统一fallback，不显示raw error/path。超预算是历史事实，不能因为终态仍保留flag就声称完成仍在等待。
- `BenchmarkS2Panel.tsx:222`：固定requested1000/s、60s、queue128没有输入；三项actual rate仅显示DTO，不在前端重算。attempted分母为source active time，ring delivered/DB committed分母为包含drain/flush/cleanup的总time，并有可见/tooltip说明。null显示不可用，真实0保留0。scheduler/queue/drop/abandon/ring/evict/DB/log-emit/unacknowledged分别显示；日志emit不冒充状态通知。
- 新 `tests/monitoring/benchmark-s2.smoke.test.tsx`，两locale的 `monitoring.s2` 54个string leaves，严格词条基数同步；spec与backend owner协调，独立追加 `monitoring-log-contracts.md` 的S2 frontend段。

## 实际缺陷与红绿

1. **取消确认后焦点落body**：真实Ego浏览器首先证明初始Keep seed焦点与Enter零Start，但取消后焦点落body。仅把pending推迟到确认后仍不能恢复。新headless返回焦点断言稳定红；源码追到Radix modal Content的close autofocus只focus DialogTrigger，GlobalConfirmDialog是imperative路径，没有Trigger。consumer取消后下一animation frame恢复原Start，要求mounted、相同opening/action generation、原trigger仍连接、焦点仍在body且无其他dialog；新dialog、用户自行移焦或离页都不能被抢焦。原红测及这些防偷焦回归全部绿。没有改sharedmodal、timeout、动画或焦点陷阱。
2. **相同revision恢复仍锁死**（独立checker发现）：not_started0→Start准入/传输拒绝→reconnect status成功仍为0，原accept的`<=`早退也跳过清错误，Start永远disabled。新 `clears a refusal after successful recovery of the same not-started revision` 原红→绿；错误恢复单独按本次query generation/same-run判断，不采纳重复payload。
3. **旧query覆盖新动作状态**：新增 `does not let an old recovery failure overwrite a newer cancellation result` 与 `does not let an older successful recovery clear a newer cancellation refusal` 分别稳定红→绿。数据采纳函数现在只更新快照并返回是否采纳；错误状态由fresh listener/query/action单独管理。没有为了重连去放松revision/run边界。
4. **terminal仍显示pending**：浏览器harness保留历史cleanup flag发布terminal，暴露无条件warning同时宣称“Completion is still pending”。随后用真实DTO允许的failed+cleanup_over_budget+稳定failure_code复现红测。两locale把warning改为纯事实“Cleanup exceeded its time budget.”；是否仍在等待由phase说明，红→绿。词条数不变。

## 最终只读验证

| 检查 | 实际结果 |
| --- | --- |
| `bun run type-check` | 最终候选退出0 |
| scoped no-fix ESLint：Panel、MonitoringView、两locale、S2test、两i18n计数文件 | 最终候选退出0；没有规则disable/阈值调整 |
| `bunx vitest run --config vitest.smoke.config.ts tests/monitoring/benchmark-s2.smoke.test.tsx tests/ui/confirm-interaction.smoke.test.tsx tests/quality/monitoring-log-sanitize.smoke.test.tsx` | **25/25 PASS**：21项S2（含6个拒绝source分支）+旧confirm2/logsanitize2；最后一轮13:27:27本地时间，退出0 |
| `bun run check:i18n` | 最后候选两端**4564**个完全相同leaf，退出0；既有非阻断历史key告警保留 |
| `bun run test:i18n` | 最后候选24/24 + key-leak self-test PASS，退出0 |
| `git diff --check` | own范围退出0 |
| `bun run build` | 前一个候选退出0（MonitoringView产物27.28kB）；随后最终recovery/copy修复由上述tsc/lint/25focused验证，最终整包由主会话正在运行的完整CI消费，不能把先前build算作最终候选build收据 |

i18n初次失败是严格基数陈旧，不是两locale缺失：测试基数4510与当时基线一致，check脚本仍4501，已少记9项。本批新增54项，最终两常量都4564，保留严格等值/全集对账，不改>=或skip。中间4562候选已经淘汰。

生产build先因既有uv cache的sandbox权限失败，获授权复用既有cache后完整退出0；标准icons:ensure重生成的资源在git中无变化。没有手改品牌/图标或新建cache。构建有既有lightningcss `@tailwind components`提示，不构成本批源码失败。

## 真实 localhost 与截图限制

按 `ccr-ui-visual-workflow` 和已安装Ego2.2.0指南使用自有localhost测试页面。忽略路径 `ccr-ui/.tmp/s2-preview-20261106.html` 直接挂载真实Panel+真实GlobalConfirmDialog+正式样式，闭合mock只允许三S2动作和event订阅；页面显著标注 **UI test fixture · IPC mocked · synthetic counters · not a performance result**。没有启动Tauri/GPUI窗口，没有账户/SSH/WSL网络或真实S2 producer。

端口5187和标准5173最初EACCES。只读Windows排除表证明二者都在5117..5216保留范围；改用仓库配置的15173。正常sandbox启动后health loopback被拒，复用授权loopback后health `/` 345ms成功。没有改系统端口/防火墙或开发脚本。

单一Ego TaskSpace7的真实页面结果：

- 初始三rate均Not available，requested1000独立标注；展开计数可读，无rate/duration输入。
- 初始确认focus **Keep seed**；实际键盘Enter后 **Start调用0次**，modal关闭；等待两个requestAnimationFrame完全settle后focus仍为 `benchmark-s2-start`。
- 明确Start trial后mock Start仅1次，渲染synthetic 950/930/895，requested仍1000。
- 当前1639px viewport内panel宽976.8px，无横向overflow；light实测background rgb(242,247,245)/heading rgb(30,41,50)，dark切换为rgb(23,34,37)/rgb(240,248,245)，仍无横向overflow。没有扩展尺寸/DPI矩阵。
- Request stop仅1次；保持Cleaning且超预算，不出现Start；发布harness terminal后显示Cancelled after cleanup且仍不能Restart。该最后状态同时暴露了上述历史flag文案问题，最终文案修复由headless红绿覆盖，未重启已结束的浏览器space。

**没有成功截图。** Ego默认screenshot、`raw/fullPage:false`及最后受支持 `Page.captureScreenshot(fromSurface:false)`均返回 `CdpRequestTimeoutError: Page.captureScreenshot`；DOM/keyboard依然正常。已按主会话要求停止截图排查。不能把DOM/token读数称为截图或完整视觉通过，更不能称native/performance通过。原始脚本在忽略的 `outputs/desktop/s2-preview-20261106/round-01..07.mjs`，页面操作仅本任务自有surface。

TaskSpace已`finish({keep:[]})`成功关闭，Vite自有session已停止，精确15173 listener查询无匹配。最终源码继续冻结；完整CI与独立review由主会话拥有。
