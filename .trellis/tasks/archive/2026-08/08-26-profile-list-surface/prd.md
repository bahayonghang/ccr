# Profile 列表呈现层：页壳、统计条、筛选与双视图

父任务：`08-26-profile-design-language`
依赖：`08-26-profile-registry-tokens`

## Goal

实现统一 Profile 列表页面的全部呈现层：面包屑、页头、Off 横幅、四卡统计条、QuickRail、筛选栏、卡片视图、表格视图、Inspector 右栏、空态、source mode，以及承载它们的页面装配组件与状态 hook。本子任务不接线平台视图，交付可被 rollout 直接组装的组件。

骨架顺序与既有能力按 `profiles-page-contracts.md` 的共享骨架契约执行：父任务决策 5 已确定不删除任何已规格化能力。

## Requirements

- R1：页壳。面包屑（`{平台名} / Profiles` + 环境徽标 + 配置文件名徽标）与页头（平台 glyph 方块 + 标题 + 配置路径 + 主次操作按钮）。
- R2：骨架顺序。`ProfilesPageHeader` → Off 横幅 → `ProfilesStatStrip` → `ProfilesQuickRail` → `ProfilesToolbar` → 主列表 → `ProfilesInspector` 右栏，与 `profiles-page-contracts.md` 一致。
- R3：Off 横幅。仅在 `canOff === true` 时渲染，位于 Header 与 StatStrip 之间，确认框 `type=warning`。不得移入 Header 溢出菜单。`canOff` 由 props 传入，本任务不改 `ProfilesConfig`。
- R4：统计条。四卡等宽网格——总数（含供应商去重计数副行）、运行中（accent 高亮）、标签分布、认证方式。后两卡为 chip + 计数列表。数值从 `ProfileDisplayRecord[]` 派生。
- R5：QuickRail 接入。`ProfilesQuickRail` 置于 StatStrip 与 Toolbar 之间，`⌘/Ctrl+1..8` 绑定、`ccr:profiles:pinned:{platform}` / `ccr:profiles:recent:{platform}` 持久化与稳定编号语义不变。
- R6：命令面板接入。`ProfilesCommandPalette` 保留 `__off` 条目，由 `features.commandPalette` 控制。
- R7：筛选栏。单行布局，含搜索框（快捷键提示跟随实际绑定）、标签 pill 单选组、视图切换段控件；Filters 弹层保留 provider 维度与排序维度及其焦点陷阱逻辑。
- R8：搜索。对 `ProfileDisplayRecord.searchText` 做小写包含匹配，覆盖名称、描述、Base URL、标签。
- R9：卡片视图。三列网格；Inspector 展开时降为两列。单卡含状态点、名称、描述、状态徽章、`badges`、2×2 字段网格、底栏（标签 + 编辑 + 应用/停用）。
- R10：表格视图。六列固定网格 `216px minmax(200px,1fr) 176px 104px 136px 132px`，容器 `min-width: 1024px` 且 `overflow-x: auto`。列定义来自 `presentation.fieldSlots`。
- R11：Inspector 右栏。`ProfilesInspector` 系列与 `ProfileDiffRows` 接入，descriptor 由平台侧以 props 注入，组件内不写平台分支。
- R12：空态。区分「该平台无任何 profile」与「当前筛选无结果」，后者提供「清除筛选」动作。
- R13：状态 hook。`useProfilesSurface` 接收 `ProfileDisplayRecord[]` 与 `current`，持有 `query` / `tagFilter` / `providerFilter` / `sortBy` / `viewMode` / `editorTarget` / `sourceMode`，派生 `filtered` 与四项统计。不发起请求。
- R14：视图模式持久化。按平台 key 持久化到 `localStorage`（key `ccr:profiles:view:{platform}`），复用 `features/profiles/stores.ts` 的 Zustand + 手动读写模式。读写用 `try/catch` 包裹；storage 抛错时降级为纯内存 state，当前会话内仍可切换视图。不得因存储不可用而放弃需求。
- R15：source mode。`ProfilesSurface` 持有 source mode 状态并挂载 `ProfilesRawEditorPanel`，由 props 传入的 raw-source capability 驱动，逐项满足 `raw-config-editor-contracts.md` 的明文警告、`conflict` 只给重载/取消、`activation_conflict` 需显式危险确认后带 `force: true` 重试、保存后先清 dirty 退出 source mode 再执行全量刷新四项。capability 缺席时不渲染入口。
- R16：页面装配。`ProfilesSurface` 把上述组件装配为完整页面，编辑器入口以 props 预留（`onAdd` / `onEdit` 透传），本任务不实现编辑器。
- R17：无平台名分支。`components/profiles/**` 与 `features/platform/profiles/**` 不得比较平台名字面量。
- R18：测试落位。新增测试位于 `ccr-ui/tests/*.smoke.test.ts(x)`。

## Acceptance Criteria

- [ ] AC1（R16、R13）：`ProfilesSurface` 可用任意 `ProfileDisplayRecord[]` + `ProfilePresentation` 组合渲染出完整列表页，平台差异仅来自入参。
- [ ] AC2（R2、R3）：DOM 顺序断言 Header → Off 横幅 → StatStrip → QuickRail → Toolbar → 列表 → Inspector；`canOff === false` 时 Off 横幅不存在，且 Header 溢出菜单中也不存在 Off 项。
- [ ] AC3（R4）：统计条四卡数值正确——总数等于记录长度；供应商数等于 `vendorKey` 非 null 后的去重数量；标签与认证分布计数与记录一致。
- [ ] AC4（R4）：运行中卡在有当前 profile 时显示其名称并使用 accent 高亮，无当前 profile 时显示未应用文案且不高亮。
- [ ] AC5（R5）：`tests/profiles-quick-switch.smoke.test.ts`、`tests/profiles-hotkeys.smoke.test.ts`、`tests/profiles-quick-rail.smoke.test.ts` 在 QuickRail 接入新页面后仍通过。
- [ ] AC6（R7、R8）：搜索对四字段生效；标签 pill、Filters 弹层的 provider 与排序维度均可用，三者与搜索可叠加。
- [ ] AC7（R9、R10）：卡片与表格两个视图展示同一组数据，运行中态在两个视图中的高亮一致。
- [ ] AC8（R10）：在 900×800 viewport 下，表格容器 `scrollWidth > clientWidth` 且 `document.body.scrollWidth <= document.body.clientWidth`，测试断言两个数值关系。
- [ ] AC9（R11）：Inspector 右栏可展开，descriptor 经 props 注入后渲染出对应行；展开时卡片网格为两列。
- [ ] AC10（R12）：无结果时展示带「清除筛选」的空态，点击后恢复全量列表；记录为空时展示另一套文案且不显示「清除筛选」。
- [ ] AC11（R14）：视图选择在组件卸载重挂载后保持；claude 与 codex 两个平台 key 互不影响；`localStorage.setItem` 抛错时不崩溃且视图仍可切换。三条各一测试。
- [ ] AC12（R15）：source mode 四项断言——进入前调用 `requestConfirm`；`conflict` 只渲染重载与取消；`activation_conflict` 先要求危险确认再以 `force: true` 重试同一 content 与 token；保存成功后调用顺序为清 dirty → 退出 source mode → `refreshAll()`。capability 未传入时入口不渲染。
- [ ] AC13（R17）：`tests/platform-surface-unify.smoke.test.ts` 的无平台名分支断言覆盖本任务新增文件并通过。
- [ ] AC14（R1-R16）：Profile 相关 `.tsx` 与 `.css` 中无硬编码 hex，`rg` 结果为空。
- [ ] AC15（R1-R16）：新增文案在 `zh-CN` 与 `en-US` 中均存在，`bun run check:i18n` 通过。
- [ ] AC16（R18）：新增测试文件为 `tests/profiles-surface.smoke.test.tsx`、`tests/profiles-view-mode.smoke.test.ts`、`tests/profiles-raw-source.smoke.test.tsx`；`rg -l "smoke.test" ccr-ui/src` 为空。
- [ ] AC17（R1-R16）：新组件在 1440×900 与 900×800 两个 viewport、`light|dark` × `neutral|clay` 四种组合下按 `tests/fixtures/profiles.ts` 夹具走查通过。
- [ ] AC18（R1-R16）：本任务重写 `profiles-shared.css` 后，仍由 `BaseProfiles` 渲染的 Claude 与 Codex 现有页面在上述同一组条件下走查无回归，结论记入 `notes.md`。
- [ ] AC19（R6）：`features.commandPalette` 为真时命令面板可打开，条目含 `__off`；Header 溢出菜单中不含 Off 项。
- [ ] AC20（R1-R18）：`just frontend-check-quick` 通过。

## Constraints

- 不删除任何现有文件。`ProfileListRow`、`GrokProfileCard` 等待退役组件在本任务保持原样，删除由 rollout 执行。
- 不改路由，不改 `src/configs/profiles.ts` 已有字段，不改 Tauri 命令签名。
- 不改 `features/{claude,codex,grok}/*ProfilesView.tsx`，接线由 rollout 执行。
- 样式只写 `components/profiles/profiles-shared.css`，不写 `profile-editor-shell.css`（归 `08-26-profile-editor`）。
- `profiles-shared.css` 仍被 `ProfileListRow`、`ProfilesHeader` 等在线组件 import。重写时保留这些组件使用的类名，直到 rollout 删除它们。AC18 是该约束的验收。
- 不移除 `ProfilesToolbar` 的 provider 维度、排序维度与焦点陷阱逻辑（父任务决策 5）。
- 动效遵循仓库既有 reduced motion 降级约定。

## Notes

- 结构、尺寸、层级的权威来源是 `../08-26-profile-design-language/research/design-source.md`。
- Inspector 右栏在设计稿中不存在，卡片网格因此从三列降为两列。这是对设计稿的已知偏差，来自父任务决策 5，需追加到 `research/design-source.md` 的偏差表。
- `ProfilesStatStrip` 与 `ProfilesToolbar` 是改造对象，不是从零新建。
- 本任务是审阅报告 TPR-06、TPR-07、TPR-10、TPR-14 的主要落地入口。
