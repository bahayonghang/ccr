# 执行计划：应用外壳、路由与 Tauri 接线

> 父任务：`08-22-react-migration`（阶段 3，在 `08-22-state-logic-port` 之后）。
> 分支：`feature/react-migration/shell-port`，PR 目标 `feature/react-migration`。
>
> 本任务完成后七个 `08-22-views-*` 可并行推进。因此第 8 节的共享接口定稳是本任务的关键交付项。

## 前置确认

- [ ] `08-22-state-logic-port` AC1–AC8 全部满足，store 与 hooks 可用。
- [ ] `08-22-design-system` 的 `primitive-disposition.md` 与 `animation-disposition.md` 已落盘。
- [ ] `08-22-react-foundation` 的 `utils-disposition.md` 已标出 11 个「需接线」文件。
- [ ] 读 `confirm-interaction-contracts.md`（3.6 KB）。
- [ ] `git checkout -b feature/react-migration/shell-port feature/react-migration`

## 批次 0：三项未决项验证

在写路由表之前验证，避免结构定错后返工。

- [ ] `commands/:client?` 的可选参数匹配行为（`design.md` §11 第 1 项）。不等价则改两条路由。
- [ ] `<ScrollRestoration />` 对内部滚动容器是否生效。不生效则用 ref map + `useLayoutEffect`。
- [ ] tray 窗口是否需要独立 HTML 入口。需要则同步改 `vite.config.ts` 多入口与 Tailwind 源文件检测范围。

三项结论落盘为 `router-probe.md`。

## 批次 1：路由表

- [ ] 按 `design.md` §1 建 2 个顶层条目 + `MainLayout` 的 `children`（约 73 条）。
- [ ] `RouteHandle` 接口 + `useRouteHandle()` 读取函数，8 个字段名保留原名。
- [ ] smoke 测试遍历路由表，断言每条的 `handle` 字段名在允许集合内（补偿 `handle` 无编译期强制）。
- [ ] mcp / agents / plugins 路由沿用 `genericPlatformDescriptorList` 生成逻辑，输入按新 descriptor 的形状预留（`design.md` §2）。
- [ ] 22 处懒加载改路由级 `lazy`（`design.md` §5）。
- [ ] 路由清单比对表落盘，75 条路径逐条确认一致（AC2）。

验证：`bun run type-check`；全部 75 条路由可导航，无白屏（AC1）。此时页面内容为占位，视图迁移在阶段 5。

## 批次 2：导航守卫替代

- [ ] `beforeEach` 拆为 perf 埋点（`useEffect`）与 locale 预热（`loader`），按 `design.md` §3 的判据。
- [ ] `afterEach` 的 `recordRouteTiming` 改 `location` 变化监听。
- [ ] `usePageTransition` 的 `depth` / `group` 比较逻辑移到布局组件，用 ref 存上一次 handle。

验证：路由切换耗时可被 `perfTelemetry` 采集（`08-22-arch-quality-perf` 场景 5 的脚本可跑通）。

## 批次 3：缓存路由替代

- [ ] 按 `design.md` §4 的表落地 5 条路由的状态外提。
- [ ] `commands` 的流式累积缓冲：Zustand 按 client 键存数组，事件桥接层 append，卸载不清空。
- [ ] 缓冲上限与截断策略确定并实现。
- [ ] 滚动位置按批次 0 的结论落地。
- [ ] `MainLayout` 派生 `cacheKey` 数组的逻辑删除，`cacheKey` 字段保留在 `handle` 内。
- [ ] 逐条验证五项行为：数据、选中态、搜索词、筛选条件、滚动位置（AC4）。
- [ ] 表单草稿只验 store 侧读写（键为配置 id）。
- [ ] `commands` 的流式输出续读验证（AC4）。

表单草稿的界面级验证依赖 `configs` 视图，该视图在阶段 5 迁移。本批次先验证 store 侧的草稿读写，界面级验证归 AC11，由 `08-22-views-profiles-config` 批次 2 执行并在父任务视图门核对。本任务的责任是把该缺口显式记录并通知对方，**不作为本任务交付门的准出条件**。

## 批次 4：Tauri 与运行时接线

- [ ] `themeBootstrap` 在 React 挂载前执行。放 `main.tsx` 还是 `index.html` 内联按 `design.md` §9 的判据定（是否依赖 `@tauri-apps/api`）。
- [ ] 三层模型（`data-theme` / `data-flavor` / `data-accent`）的解析、写入、存储键读写行为不变（R6）。旧值可正常解析。
- [ ] `fontPreferences` 同期接线。
- [ ] `windowChrome` / `tauriWindow` / `nativeWindowAppearance` 接入 `Titlebar` 与启动流程。
- [ ] `startupRecovery` 在 React 挂载后接线。
- [ ] `runtimeState` / `tauriRuntime` 全局接线。
- [ ] `errorHandler` 接入两个 `ErrorBoundary`（`MainLayout` 内与 `/tray/codex`）+ `window.onerror` / `unhandledrejection`。
- [ ] `logger` 与 `logRedact` 接线，脱敏行为不变（R8）。
- [ ] `perfTelemetry` 的路由与启动埋点（批次 2 已部分完成）。

验证：AC5（主题切换与刷新后保留）、AC6（窗口六项操作）、AC7（启动恢复）、AC8（日志无未脱敏凭据，smoke 测试断言）。

## 批次 5：外壳组件

按 `design.md` §10 的依赖顺序，分多次提交。

- [ ] `ui/` 16 个原语的消费点落地（形态由 `08-22-design-system` 决定，位置为 `src/ui/`）。
- [ ] `common/` 12 文件 1,599 行：`BaseModal`、`GlobalConfirmDialog`、`ToastContainer`、`MasterDetailLayout`、`ListSearchHeader`、`MultiSelectFloatingBar`、`BulkDeleteDialog`、`ScrollToTopButton`、`AnimatedBackground`、`StageBackground`、`AgentIcons`、`MarketplacePagination`。
- [ ] 根级组件：`Titlebar`(448)、`PageHeaderCard`(274)、`ModuleSubnav`(122)、`UpdateModal`(514)、`ConfirmModal`(236)、`HistoryList`(242)、`VersionManager`(220)、`EnvironmentSwitcher`(208)、`EnvironmentBadge`(58)、`BackendStatusBanner`(49)、`ThemeToggle`(42)。
- [ ] `MainLayout`(589) 与 `App`(75)。`src/App.vue` 删除。
- [ ] 8 处 `Teleport` 中落在本任务范围的改 `createPortal`（R5）。
- [ ] 12 处 `Transition` 中落在本任务范围的改 `AnimatePresence`，与 `animation-disposition.md` 一致（R5）。
- [ ] `AnimatedBackground` / `StageBackground` 保留 CSS 驱动，遵循 reduced motion 降级。
- [ ] 8 处 `defineExpose` 中落在本任务范围的逐处判定并改写，优先受控属性（R9）。判定记录落盘。
- [ ] 9 个归属其他子任务的根级组件不动（PRD 表）。

验证：AC1（导航无报错）、AC10（`confirm-interaction-contracts.md` 行为通过）、AC9（`type-check` 与 `lint`）。

## 批次 6：共享接口定稳

本批次是阶段 4 → 5 门的准出项，不可省略。

- [ ] `MasterDetailLayout` 的 props 完整列表与类型公示。
- [ ] slot → children / render props 的映射表公示。
- [ ] 列表侧与详情侧的滚动、选中、空状态、加载态责任划分公示。
- [ ] `ui/` 16 个原语的同类信息公示。
- [ ] `shared-interfaces.md` 落盘，通知七个视图子任务。
- [ ] 首屏加载模块集合与迁移前对比记录落盘，未扩大（AC3）。

## 验证命令

| 时机            | 命令                                                     |
| --------------- | -------------------------------------------------------- |
| 每批次后        | `bun run type-check`、`bun run lint`                     |
| 批次 1、3、5 后 | `bun run test:smoke`                                     |
| 批次 4 后       | `bun run tauri dev`（窗口与主题行为需真实 Tauri 运行时） |
| 批次 6 后       | `bun run build`（首屏模块集合对比）                      |
| 交付前          | `just frontend-check-quick`、`just tauri-check`          |

## 交付门（父任务外壳门的另一半）

- [ ] AC1–AC10 全部满足。
- [ ] **AC11 不在本门**：`configs` 表单草稿的界面级验证依赖阶段 5 的视图，归 `08-22-views-profiles-config` 批次 2，在父任务视图门核对。本门只要求该缺口已记录并已通知对方。
- [ ] 三份记录落盘：`router-probe.md`、`shared-interfaces.md`、`defineExpose` 判定记录。
- [ ] 路由清单比对表 75 条逐条确认（AC2）。
- [ ] 首屏模块集合未扩大（AC3）。
- [ ] `MasterDetailLayout` 与 `src/ui/` 原语接口已公示，阶段 4a 之后不改。
- [ ] 通知 `08-22-test-contract-rebuild` 开始交付最小测试集（协同点 C）。

## 回滚点

| 批次 | 回滚方式                                            |
| ---- | --------------------------------------------------- |
| 0    | 只产出结论                                          |
| 1–2  | 路由表与守卫替代，单独提交                          |
| 3    | 缓存路由替代，单独提交                              |
| 4    | 接线按文件分多次提交，可精确回退某一项              |
| 5    | 按 `design.md` §10 的依赖顺序分多次提交，可按层回退 |
| 6    | 接口文档，revert 无代码影响                         |

批次 5 删除 `src/App.vue` 后 Vue 入口不再可用。此前的批次回滚仍可回到 Vue 可运行状态。

## 协同点

| 编号 | 内容                                                               | 对方                                  | 时机   |
| ---- | ------------------------------------------------------------------ | ------------------------------------- | ------ |
| H    | `MasterDetailLayout` 与 `ui/` 原语接口共同定稳                     | `08-22-design-system`、七个视图子任务 | 批次 6 |
| C    | 本任务完成即通知对方开始交付最小测试集，3 个工作日内               | `08-22-test-contract-rebuild`         | 交付时 |
| —    | `shellPreferences` / `themeBootstrap` / `fontPreferences` 接口对齐 | `08-22-state-logic-port`              | 批次 4 |
| —    | descriptor 扩展后替换路由生成的输入，路径不变                      | `08-22-platform-unify`                | 阶段 4 |
| —    | 表单草稿的界面级验证归对方（本任务 AC11）                          | `08-22-views-profiles-config`         | 阶段 5 |
| —    | `monitoring-log-contracts.md` 只保证接线，契约验证归对方           | `08-22-views-sync-tools`              | 批次 4 |
| —    | i18n 调用点先用临时方案，由对方统一切换                            | `08-22-i18n-port`                     | 批次 5 |
