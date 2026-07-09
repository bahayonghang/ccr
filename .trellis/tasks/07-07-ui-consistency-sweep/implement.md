# 执行清单

> 约定见 design.md。R1 为首个执行会话范围;R2 按序每页独立验收,可跨会话。
> 前置阅读:`.trellis/spec/ccr-ui/frontend/theme-token-contracts.md`;做 R2-6 时加读 `usage-chart-stability-contracts.md` 与 `dashboard-presentation-contracts.md`。
> 完成一项即打勾并在该项下追加验证记录(日期 + 命令输出要点);不得预填。

## R1 原生对话框清除(7 文件,16 处实弹命中)

每完成一项跑余量核对,预期命中数递减(起点 16 实弹):

```bash
rg "\b(confirm|alert)\(" ccr-ui/src --glob '!**/*.test.*'
```

豁免(不计入实弹):`composables/useAgents.ts:72`(注释)、`composables/useConfirmAction.ts:2`(文档注释)。

- [x] R1-1 `views/ClaudeAuthView.vue`(2 处):471 切换官方账号 confirm → `requestConfirm`(**warning**);489 删除官方账号 → `requestConfirm`(**danger**)。文案沿用现有 `tt()` 双语串拆 title/message。
      验证:rg 余量 14 实弹。
      记录(2026-07-09):`rg "\b(confirm|alert)\(" ccr-ui/src --glob '!**/*.test.*'` = 16 命中(14 实弹 + 2 豁免注释)。title/confirmText 用 tt() 内联双语,未新增 i18n key。
- [x] R1-2 `views/mcp/McpManagerView.vue`(3 处):304 projectScopeWrite → **warning**;331 deleteGroup → **danger**;354 projectScopeImport → **warning**。`handleSubmit` / `handleImportServers` 改 async 闸门(scope !== 'project' 直通不弹窗)。
      验证:rg 余量 11 实弹。
      记录(2026-07-09):rg = 13 命中(11 实弹)。三个 handler 原本已 async,确认后继续 submitForm+closePanel / 导入流程,控制流不变。title/confirmText 复用 `common.warning`/`common.confirm`/`common.delete`/`common.cancel`。
- [x] R1-3 `views/generic/AgentDetailView.vue`(4 处):447/461 `alert(operationFailed)` → `uiStore.showError`;468 删除 confirm → `requestConfirm`(**danger**);475 `alert(deleteFailed)` → `uiStore.showError`。补 `useUIStore` import。
      验证:rg 余量 7 实弹。
      记录(2026-07-09):rg = 9 命中(7 实弹)。已补 `useUIStore` import + 实例。
- [x] R1-4 `components/BaseSlashCommands.vue`(1 处):557 删除 confirm → `requestConfirm`(**danger**)。补 uiStore。
      验证:rg 余量 6 实弹。
      记录(2026-07-09):rg = 8 命中(6 实弹)。message 沿用 `${props.config.i18n.prefix}.confirmDelete`。
- [x] R1-5 `components/McpPresetsPanel.vue`(4 处):339 apiKeyRequired → `showWarning`;361 installPartialFailed → `showError`(失败平台列表拼入消息,内联结果列表升级留 R2-4);363 installSuccess → `showSuccess`;370 installFailed → `showError`。补 uiStore。
      验证:rg 余量 2 实弹(仅剩 composables)。
      记录(2026-07-09):rg = 4 命中(2 实弹,均在 composables)。失败平台列表沿用原 `\n` 拼接进 showError 消息。
- [x] R1-6 `composables/usePlatformMcp.ts:221` + `views/generic/PlatformMcpView.vue`:composable `deleteServer` 去 confirm 变纯执行器;视图新增 `handleDeleteServer`(`requestConfirm` **danger**,message 用 `${i18nPrefix}.deleteConfirm`),模板 `@click` 改接线。
      验证:rg 余量 1 实弹。
      记录(2026-07-09):rg = 3 命中(1 实弹)。deleteServer 签名/返回 `Promise<boolean>` 与 toast+loadServers 保留;PostToolUse formatter hook 将该 composable 整文件重排(4 空格→2 空格),非手工重构。
- [x] R1-7 `composables/usePlatformPlugins.ts:138` + `views/generic/PlatformPluginsView.vue`:同 R1-6,`handleDeletePlugin`。
      验证:rg 实弹 0(全站仅剩 2 处豁免注释)。
      记录(2026-07-09):rg = 2 命中(0 实弹,仅剩 useAgents.ts:72 / useConfirmAction.ts:2 豁免注释)。name 用 `plugin.name || plugin.id`;formatter 同样整文件重排该 composable。`cd ccr-ui && bun run type-check` 通过;`bun run lint` 0 errors(1 个存量 warning:DashboardSignalStream.vue,非本次触碰)。未新增 i18n key。
- [x] R1-V 收尾验证(全绿才算过):
  - `rg "\b(confirm|alert)\(" ccr-ui/src --glob '!**/*.test.*'` 零实弹命中。
  - `cd ccr-ui && bun run type-check && bun run lint`。
  - `just frontend-check-quick`。
  - 若新增 i18n key(如确认框标题):`bun run i18n && bun run test:i18n`。
      记录(2026-07-09):rg 全扫 = 2 命中,均为豁免注释,零实弹 ✅;type-check 零报错 ✅;lint 0 errors(1 个存量 warning:DashboardSignalStream.vue,非本次触碰)✅;`just frontend-check-quick` 全绿(i18n 23/23、smoke 82 文件 372/372)✅;未新增 i18n key(复用 common.* 与既有 deleteConfirm/confirmDelete key、ClaudeAuthView 用 tt() 内联双语)。composable 语义 diff 经 `git diff -w` 复核:仅 confirm 闸门删除 + 注释更新,大 diff 为项目 formatter hook 的 4→2 空格重排。
- [ ] R1-M 手测(亮/暗各一轮):ClaudeAuth 切换/删除、MCP manager project scope 提交/删组/project scope 导入、Agent 详情删除、斜杠命令删除、presets 安装失败与成功路径、platform MCP 删除、platform 插件删除 —— 核对 dialog type 语义、取消不执行、toast 出现。留手测记录。
      状态(2026-07-09):本会话无运行中的 Tauri 应用,手测未执行;留待用户或下个带应用会话完成后补记录。静态面已由 R1-V 覆盖。
- [x] R1-C 提交(中文,[AI] 前缀,不 push,分两笔):① 视图/组件对话框清除(R1-1~5);② composable 确认决策上移(R1-6~7)。
      记录(2026-07-09):① `02789820` fix(ccr-ui) 清除视图与组件层原生 confirm/alert;② `94ff6f29` refactor(ccr-ui) platform MCP/插件删除确认决策上移到视图层。未 push。

## R2 旧页面对齐(每页一项,按序独立验收)

每页统一验收基线(R3):无原生对话框;危险操作 danger;表面/边框/文字走语义令牌,无新硬编码 hex/rgba(装饰性除外并注释);空态有引导动作(EmptyState);加载态骨架或统一 spinner;无新增 `backdrop-filter`;`prefers-reduced-motion` 下无常驻动画。

每页统一产出:**亮/暗截图各一** + 确认对话框/空态/加载态手测记录 + 该页文件 `rg "#[0-9a-fA-F]{6}"` 新增命中为零 + `cd ccr-ui && bun run type-check && bun run lint`。

- [x] R2-1 `McpManagerView` / `mcp/*`:卡片表面贴新令牌 + 空态 EmptyState(confirm 已在 R1 清除)。
      记录(2026-07-09):① `McpDetailPanel.vue` `.detail-section` 表面从 `rgb(--color-bg-surface-rgb / 54%)`+`--elevation-1` 贴换为 `--surface-card-{bg,border,shadow}` 语义令牌,`--effective` 变体渐变叠在 `--surface-card-bg` 上;② `McpListPanel.vue` 列表空态从纯文本 div 换为 `EmptyState` 组件(icon Server;非搜索空态带"添加服务器"引导动作 → emit create;搜索无结果不带动作),删除孤儿样式 `.mcp-list-panel__empty`;③ `McpManagerView.vue` 唯一 hex fallback `var(--color-danger, #ef4444)` 去掉字面量(令牌已在 tokens.css 定义)。未新增 i18n key(复用 list.empty/noSearchResults/actions.addServer)。验证:mcp 全部文件 `rg "#……"` 零命中;type-check 零报错;lint 0 errors(1 存量 warning 非本次触碰);`just frontend-check-quick` 全绿(smoke 372/372)。亮/暗截图与手测:本会话无运行中 Tauri 应用,与 R1-M 一并留待带应用会话补。detail 面板"未选中"占位为选择提示非数据空态,维持原样。
- [x] R2-2 `ClaudeAuthView` / codex tabs(Auth 相关):页面结构内表面类与交互对齐。**不动 `codex-auth-shared.css`**(已拆 07-09-ui-codex-auth-css-tokens)。
      记录(2026-07-09):① 手搓保存表单 modal(`__modal-backdrop`/`__modal`,含 `rgb(15 23 42 / 55%)` 遮罩与阴影字面量)整体迁移到全局 `BaseModal`(surface="solid",footer slot 放取消/保存按钮),获得焦点陷阱、Esc、body 滚动锁与标准动效;删除孤儿样式 `__modal-backdrop`/`__modal`/`__modal-title`/`__modal-actions` 及各分组选择器引用,顺带合并因删除产生的重复分组选择器;② freshness 三色 `rgb(16 185 129)/rgb(245 158 11)/rgb(239 68 68)` 字面量 → `var(--color-success/warning/danger)`;③ "无账号"空态换 `EmptyState`(icon User,带"保存当前登录"引导动作),loading 文本占位维持。`CodexAuthView.vue` 复查零 hex/裸 rgb,结构样式在 codex-auth-shared.css(未触碰,归 07-09)。验证:ClaudeAuthView `rg` hex/裸 rgb 零命中;type-check 零报错;lint 0 errors(1 存量 warning);`just frontend-check-quick` 全绿。亮/暗截图与手测(保存 modal、切换/删除 confirm、空态动作)留待带应用会话与 R1-M 一并补。
- [x] R2-3 `generic/AgentDetailView` + `AgentsView`:危险操作 danger 语义复查(AgentsView 已用 requestConfirm,核对 type)+ 表面对齐。
      记录(2026-07-09):危险语义复查:AgentsView `handleDelete` 已是 `requestConfirm type: 'danger'` ✅,AgentDetailView 的删除在 R1-3 已改 danger ✅。表面对齐:① `bg-white` 硬编码共 5 处(AgentDetailView 2:工具 chip、编辑弹窗取消按钮;AgentsView 3:工具头像、chip、取消按钮)→ `bg-bg-elevated`(暗色模式下原为刺眼纯白);② AgentDetailView `bg-bg-surface/700` 无效透明度笔误 → `/70`;③ AgentsView 滚动条 `rgb(0 0 0 / 10%/20%)` → `rgb(var(--color-border-default-rgb) / 45%/70%)`(暗色下黑色 thumb 不可见)。空态复查:AgentsView 无结果空态已有引导动作(清空筛选按钮),维持;两页手搓编辑弹窗为存量 glass(无新增 backdrop-filter),迁 BaseModal 超出"表面对齐"范围,不动。验证:两文件 bg-white/裸 rgb/hex 零命中;type-check 零报错;lint 0 errors(1 存量 warning);`just frontend-check-quick` 全绿。
- [x] R2-4 `McpPresetsPanel`:部分失败展示从 toast 升级为内联结果列表 + 表面对齐。
      记录(2026-07-09):① 部分失败流:新增 `installResults` ref,失败时弹窗不再关闭,在操作按钮上方内联渲染逐平台结果行(成功 accent-success / 失败 accent-danger,含消息 truncate + title 悬停全文),标题复用 `mcp.presets.installPartialFailed`;打开/关闭弹窗时重置;成功路径维持 toast + 关闭,两条路径都 emit('installed')。② 表面:取消按钮 `bg-white` → `bg-bg-elevated`;API key 输入框 `bg-bg-surface/700` 笔误 → `/70`。卡片 hover 的 `from-white/5` 渐变与遮罩为装饰性存量,不动。未新增 i18n key。验证:bg-white//700 零命中;type-check 零报错;lint 0 errors(1 存量 warning)。
- [x] R2-5 `SyncView` / `CheckinView` / `ConfigsView`:抽查硬编码颜色与旧卡片样式,贴新令牌(不重排版)。
      记录(2026-07-09):抽查结果干净,零代码改动。三视图 + `views/checkin/` 子树:hex/裸 rgb/rgba/bg-white/无效透明度零命中;liquid-glass/glass-blur/backdrop-filter 零命中;原生 confirm/alert 零命中。唯一命中 `CheckinProvidersTab.vue:913` 弹窗遮罩 `rgb(0 0 0 / 50%)`——与 BaseModal 自身 `bg-black/40~60` 遮罩惯例一致,判定为惯例性 scrim 留存(登记入 F-2 装饰性清单)。
- [x] R2-6 usage 遗留三项(设计见 design.md §3;对照 usage-chart-stability-contracts + dashboard-presentation-contracts):
  - a. cost delta 涨=红/降=绿:`usageSummaryCards.ts` + `UsageMetricCard.vue`(cost 实例)+ `UsageCostConclusionCard.vue`;requests/tokens 卡不动。
  - b. `UsageTokensTab.vue:259` / `UsageCostTab.vue:174` 硬编码 animations → 导出并复用 `buildChartAnimations()`;保持 options 引用纪律;顺带收敛契约"已知偏差"节。
  - c. `ops.sourcesHint`(zh-CN.ts:2678 / en-US.ts:2786)双语人话化;`UsageDiagnosticsDrawer` 显示核对。
  - 验证:type-check + lint + `bun run i18n && bun run test:i18n`;usage 页亮/暗截图;reduced-motion 下图表动画降级手测;tab 切换无 canvas 重建回归(节点身份口径见契约 §5)。
      记录(2026-07-09):a. 方向与好坏解耦——`UsageSummaryCard` 新增 `deltaSentiment: 'positive'|'negative'|'neutral'`(`resolveDeltaSentiment`:cost 卡 up→negative/down→positive,其余 up→positive;flat→neutral),两个卡组件 delta class 从 deltaTone 改绑 deltaSentiment,CSS 类名 up/down/flat → positive/negative/neutral(色值不变:positive=success 绿、negative=danger 红);`deltaTone` 字段保留方向语义。smoke 测试补 requests(up→positive)与 cost(up→negative)断言。b. `buildChartAnimations` 导出,两 tab 的 `animations: { enabled: false }` 改为 `buildChartAnimations()`(计入 options computed 依赖,reduced-motion 偏好切换自动重建,与工厂口径一致;redraw 双 false 保留);契约"已知偏差"节已收敛(animations 项划掉,ctx.trends 项保留)。c. sourcesHint:zh `按状态标注：正常在线 / 文件缺失 / 已删除`,en `Each source is marked live, missing, or deleted`;keys.txt 快照同步。顺手修存量 i18n 缺口:en-US 补 `claudeCode.observer.empty.loadError`(HEAD 上 check:i18n 即红,经 git stash 验证与本次无关,单独提交)。验证:`check:i18n` 3659/3659 一致 ✅;`test:i18n` 通过 ✅;type-check 零报错 ✅;lint 0 errors(1 存量 warning)✅;`just frontend-check-quick` 全绿(含新增 deltaSentiment 断言)✅。usage 页亮/暗截图、reduced-motion 手测、canvas 节点身份复测:无运行中 Tauri 应用,与 R1-M 一并留待带应用会话(引用纪律静态面已核:animations 为 computed 内纯函数调用,无新增构建期数据依赖)。

## 收尾(全部 R2 完成后)

- [ ] F-1 spec 更新:design.md §1/§2 确认交互约定沉淀到 spec(trellis-update-spec 评估:并入现有 ccr-ui frontend 契约或新建交互契约文档);usage-chart-stability-contracts "已知偏差"节收敛。
- [ ] F-2 PRD 验收清单逐项复核打勾;登记存量装饰性 hex 命中清单;确认独立子任务 07-09 不阻塞本任务归档。
- [ ] F-3 全量验证:`just frontend-check-quick` + 主题 smoke(`bunx vitest run --config vitest.smoke.config.ts tests/apple-glass-surface-contract.smoke.test.ts tests/theme-bootstrap.smoke.test.ts tests/app-settings.smoke.test.ts`)+ provider-templates smoke(`bun run test:smoke -- tests/provider-templates.smoke.test.ts`)。
- [ ] F-4 中文分批提交([AI] 前缀,R2 每页一笔);不 push。

## 回滚点

- R1 两笔提交独立可 revert;composable 上移若引回归,revert 第 ② 笔即恢复 confirm 内置行为。
- R2 每页一笔提交,单页回退不影响其他页。
