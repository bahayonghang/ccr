# 全站一致性清扫 — 技术设计

> 只覆盖需要跨会话统一口径的三块:确认交互约定、composable 决策上移、usage 遗留项落法。其余按 prd.md R2/R3 逐页施工。

## 1. 确认交互约定(R1 基准)

代码库已有两套 ConfirmModal 接入方式,按场景二选一,不再新增第三种:

1. **闸门式确认(默认路径)**:`await uiStore.requestConfirm({ title, message, confirmText?, type })` → `GlobalConfirmDialog`(App.vue 挂载)全局渲染 ConfirmModal。适用于"同步决策 → 异步执行、按钮无需 busy 态"的场景,是既定习惯用法(BudgetView / HooksView / generic AgentsView / CodexAgentsView 等 9 个文件在用)。**R1 全部站点走此路径。**
2. **useConfirmAction + 局部 ConfirmModal**(profiles 页模式):确认按钮需要 busy 态或对话框内容需要自定义时用。R1 无站点需要,保留给后续复杂场景。

语义分级(R3 验收口径):

| type | 场景 |
|---|---|
| `danger` | 删除/不可逆(删账号、删服务器组、删 agent、删斜杠命令、删插件) |
| `warning` | 影响面大但可逆/可重试(project scope 写入与导入、切换官方账号) |
| `info` | 纯信息确认(R1 无) |

提示类 `alert()` 一律改 toast:`uiStore.showError / showSuccess / showWarning`(校验拦截用 warning,失败用 error)。

## 2. composable 确认决策上移(usePlatformMcp / usePlatformPlugins)

现状:`deleteServer` / `deletePlugin` 在 composable 内部先 `confirm()` 再执行。各只有一个消费视图(PlatformMcpView / PlatformPluginsView),模板 `@click` 直呼。

设计约定:

- composable 的 `delete*` 去掉 confirm,变**纯执行器**(保留执行 + 结果 toast + `loadX()` 刷新,签名不变仍返回 `Promise<boolean>`)。
- 确认决策上移到消费视图的事件处理器,视图用 `requestConfirm` 弹 dialog:

```ts
// PlatformMcpView.vue(PlatformPluginsView 同构)
async function handleDeleteServer(server: PlatformMcpServer) {
  const name = getServerIdentifier(server)
  const confirmed = await uiStore.requestConfirm({
    title: t('common.confirmDeleteTitle'),   // 复用/新增双语 key,执行时对齐现有 requestConfirm 站点的标题习惯
    message: t(`${i18nPrefix}.deleteConfirm`, { name }),  // 复用既有 deleteConfirm 文案
    type: 'danger',
  })
  if (confirmed) await deleteServer(server)
}
```

- 模板 `@click="deleteServer(server)"` 改指向 `handleDeleteServer`。
- 固化规则:**composable 不得触达对话框**(不 import ConfirmModal、不调 `requestConfirm`);toast 仍允许(uiStore 的 toast 是消息通道,不是交互决策)。"composable 返回待确认意图"在本例中即"暴露纯执行函数,由视图决定何时执行"。

## 3. R2-6 usage 遗留三项落法

决策(2026-07-09,用户拍板):cost 涨=红(danger)/降=绿(success)。

- **cost delta 语义色**:`deltaTone`(usageSummaryCards.ts)保持方向语义(up/down/flat)不动,色彩映射按指标含义翻转——仅 cost 卡(UsageMetricCard 的 cost 实例 + UsageCostConclusionCard 的同名 delta 类)把 up→danger 色、down→success 色。实现取最小改动:给 cost 卡 draft 加一个语义倾向字段(如 `deltaSentiment`)或组件侧 invert 标志,二选一执行时定;requests/tokens 卡维持涨=绿。改 `usageSummaryCards.ts` / 展示结构前对照 dashboard-presentation-contracts。
- **animations 对齐**:`usageChartOptions.ts:261` 的 `buildChartAnimations` 改为导出;`UsageTokensTab.vue:259`、`UsageCostTab.vue:174` 的硬编码 `animations: { enabled: false }` 替换为 `buildChartAnimations()`。不得破坏 options 引用纪律(`redrawOnParentResize/redrawOnWindowResize: false` 保留,options 仍为引用稳定的 computed);完成后同步收敛 usage-chart-stability-contracts "已知偏差"一节。
- **sourcesHint 人话化**:`zh-CN.ts:2678` / `en-US.ts:2786` 的 `'live / missing / deleted'` 改为完整人话(zh 中文说明、en 完整短句),`UsageDiagnosticsDrawer.vue:41` 显示核对;跑 `bun run i18n` 保证 keys.txt 同步。

## 4. 范围外与回滚

- codex-auth-shared.css(658 行)迁移 → 独立子任务 07-09-ui-codex-auth-css-tokens;本任务 R2-2 不改该文件,两边并行安全。
- R2 各页不动信息架构,纯表面与交互对齐。
- 回滚粒度 = 提交粒度:R1 拆"对话框清除"与"composable 上移"两笔,R2 每页一笔,单笔 revert 即可回退。
