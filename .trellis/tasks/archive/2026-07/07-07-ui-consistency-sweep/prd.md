# 全站一致性清扫:原生对话框与旧页面对齐

## Goal

清除全站残留的原生 `confirm()/alert()`,把尚未跟上"编辑式工作台"DNA 的旧页面逐个对齐到新材质与交互标准。本任务是清单驱动的收尾任务,在 07-07-ui-profiles-unify 抽出的 `useConfirmAction` 之上批量施工。

## Requirements

### R1 原生对话框清除(全部改 ConfirmModal/useConfirmAction + toast)

| 文件 | 命中 |
|---|---|
| views/ClaudeAuthView.vue | 471/489 confirm |
| views/mcp/McpManagerView.vue | 304/331/354 confirm |
| views/generic/AgentDetailView.vue | 447/461/475 alert+confirm |
| components/BaseSlashCommands.vue | 557 confirm |
| components/McpPresetsPanel.vue | 339/361/363/370 alert |
| composables/usePlatformMcp.ts | 221 confirm |
| composables/usePlatformPlugins.ts | 138 confirm |

composables 中的 confirm 需要把确认决策上移到调用视图层(composable 返回需确认的意图,视图弹 dialog),不允许 composable 直接依赖 UI 组件。

### R2 旧页面对齐(每页一个 checklist 项,按优先级)

1. McpManagerView / mcp/*:确认对话框 + 卡片表面贴新令牌 + 空态用 EmptyState。
2. ClaudeAuthView / codex/tabs(Auth 相关):同上;~~codex-auth-shared.css(658 行)中硬编码表面色迁移语义令牌~~(2026-07-09 拆出独立子任务 07-09-ui-codex-auth-css-tokens,本任务不动该文件)。
3. generic/AgentDetailView + AgentsView:对话框 + 危险操作 danger 语义。
4. McpPresetsPanel:alert 链改 toast + 部分失败展示改内联结果列表。
5. SyncView / CheckinView / ConfigsView:抽查硬编码颜色与旧卡片样式,贴新令牌(不重排版,纯表面对齐)。
6. usage 遗留三项(2026-07-09 决策并入):cost delta 语义色改"涨=红(danger)/降=绿(success)";UsageTokensTab/UsageCostTab 硬编码 `animations: { enabled: false }` 对齐 `buildChartAnimations()`;`ops.sourcesHint` 裸英文文案人话化(zh/en 双语)。

### R3 通用标准(每页验收基准)

- 无原生 confirm/alert;危险操作 danger 型确认;错误走 toast 或内联错误面板。
- 表面/边框/文字全部走语义令牌,无新硬编码 hex/rgba(装饰性除外并注释)。
- 空态有引导动作;加载态用骨架或统一 spinner;无新增 backdrop-filter。
- prefers-reduced-motion 下无常驻动画。

## Out of Scope

- 各旧页面的信息架构重排(只做表面与交互对齐,大重构另立任务)。
- HistoryList 虚拟滚动之外的新虚拟化改造(如后续发现长列表卡顿另立任务)。

## Acceptance Criteria

- [x] `rg "\\b(confirm|alert)\\(" ccr-ui/src --glob '!**/*.test.*'` 仅剩注释或明确标注的降级路径(目标零命中)。(2026-07-09 复核:2 命中均为注释豁免,零实弹)
- [ ] R2 清单每页:亮/暗截图 + 确认对话框/空态/加载态手测记录。(代码面全部完成;截图与手测因会话无运行中 Tauri 应用,统一留待带应用会话补,见 implement.md R1-M)
- [x] `rg "#[0-9a-fA-F]{6}" ccr-ui/src/views ccr-ui/src/components --glob '*.vue'` 新增命中为零(存量装饰性命中登记清单)。(2026-07-09 登记:AppSettingsView 15 处=主题/accent 预览色板数据;AgentIcons 4 处=平台品牌色 fallback;CommandList 4 处=令牌 fallback;TokenDetailTab 1 处=图表色 fallback;ClaudeCodeSettingsView 2 处 `#f87171`=存量待迁,非本任务触碰。另:CheckinProvidersTab 弹窗遮罩 `rgb(0 0 0 / 50%)` 为惯例性 scrim,与 BaseModal 黑遮罩一致)
- [x] usage 遗留三项按 2026-07-09 决策落地:cost 卡涨=红/降=绿;两处图表 animations 走 `buildChartAnimations()`;sourcesHint 双语人话化(`bun run check:i18n` 通过)。
- [x] `bun run type-check && bun run lint` + 主题 smoke + provider-templates smoke 通过。(2026-07-09,见 implement.md F-3 记录)

## Dependencies

- 依赖 07-07-ui-glass-tokens(令牌)与 07-07-ui-profiles-unify(useConfirmAction)。

## Notes

- 本任务按 R2 顺序可拆多次提交/多个会话执行,每页独立可验收;若单页工作量超预期(如 codex-auth-shared.css 迁移),允许把该页拆为独立子任务再挂到父任务下。
- 2026-07-09 规划决策(用户拍板):① usage 遗留三项并入本任务(R2 第 6 项);② cost delta 语义色定为涨=红/降=绿;③ codex-auth-shared.css 迁移拆为独立子任务 07-09-ui-codex-auth-css-tokens,本任务 R2-2 不动该文件。
