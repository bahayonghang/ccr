# 应用外壳、路由与 Tauri 接线迁移

> 父任务：`08-22-react-migration`

## Goal

迁移应用外壳、共享 UI 层与 75 条路由，重新接线窗口、主题引导与启动恢复，使 React 应用可完整启动并在各页面间导航。本任务完成后，`08-22-views-*` 七个子任务可并行推进。

## Scope

### 外壳与共享层

| 文件 / 目录 | 行数 |
|---|---|
| `src/App.vue` | 75 |
| `src/components/MainLayout.vue` | 589 |
| `src/components/layout/Titlebar.vue` | 448 |
| `src/components/PageHeaderCard.vue` | 274 |
| `src/components/ModuleSubnav.vue` | 122 |
| `src/components/UpdateModal.vue` | 514 |
| `src/components/ConfirmModal.vue` | 236 |
| `src/components/HistoryList.vue` | 242 |
| `src/components/VersionManager.vue` | 220 |
| `src/components/EnvironmentSwitcher.vue` | 208 |
| `src/components/EnvironmentBadge.vue` | 58 |
| `src/components/BackendStatusBanner.vue` | 49 |
| `src/components/ThemeToggle.vue` | 42 |
| `src/components/common/`（12 文件） | 1,599 |
| `src/components/ui/`（16 文件） | 2,201 |
| 小计 | 约 6,877 |

`src/components/common/` 含 `BaseModal.vue`、`GlobalConfirmDialog.vue`、`ToastContainer.vue`、`MasterDetailLayout.vue`、`ListSearchHeader.vue`、`MultiSelectFloatingBar.vue`、`BulkDeleteDialog.vue`、`ScrollToTopButton.vue`、`AnimatedBackground.vue`、`StageBackground.vue`、`AgentIcons.vue`、`MarketplacePagination.vue`。

`src/components/ui/` 的 16 个原语由 `08-22-design-system` 决定替换或保留，并迁到目标目录 `src/ui/`（父任务 `design.md` §2）。本任务负责落地其消费点。

### 归属其他子任务的根级组件

以下根级组件按业务域归入对应子任务，不在本任务范围：

| 文件 | 行数 | 归属 |
|---|---|---|
| `BaseSlashCommands.vue` | 507 | `08-22-views-sync-tools` |
| `McpPresetsPanel.vue` | 416 | `08-22-views-sync-tools` |
| `McpSyncPanel.vue` | 297 | `08-22-views-sync-tools` |
| `CommandFormModal.vue` | 247 | `08-22-views-sync-tools` |
| `CommandList.vue` | 163 | `08-22-views-sync-tools` |
| `EditConfigModal.vue` | 406 | `08-22-views-profiles-config` |
| `AddConfigModal.vue` | 342 | `08-22-views-profiles-config` |
| `ConfigCard.vue` | 331 | `08-22-views-profiles-config` |
| `CheckinProgressModal.vue` | 294 | `08-22-views-checkin` |

### 路由

75 条路由从 Vue Router 4.6.4 迁到 React Router 8.3.0（选型见父任务 `design.md` §1、映射方案见 §3）。

**结构现状**：`src/router/index.ts` 594 行，2 个顶层条目——`/tray/codex`（独立窗口，不套 MainLayout）与 `/`（布局父级，其 `children` 承载其余约 73 条）。映射为 React Router 的同构嵌套 + `<Outlet />`。

4 条动态参数路由：`commands/:client?`（可选参数）、`agents/:name`、`skills/:platform/:name`、`checkin/manage/:accountId`。

8 个自定义 `meta` 字段迁到路由对象的 `handle`，保留原名：`cache`、`cacheKey`、`hideGlobalBackground`、`stream`、`depth`、`group`、`hideSidebar`、`deferLocaleHydration`。

mcp / agents / plugins 路由由 `genericPlatformDescriptorList`（`src/config/platformDescriptors.ts` 50 行）程序化生成。生成逻辑保留，输入改为 `08-22-platform-unify` 的新 descriptor。

含 `defineAsyncComponent` 懒加载 22 处。

### 缓存路由替代

**决策**：状态外提到 store，不做组件常驻（见父任务 `design.md` §5）。

现状为 5 条路由带 `meta.cache: true`：

| 路由 | cacheKey | 需保留的状态 | 处理 |
|---|---|---|---|
| `dashboard` | `DashboardView` | 数据 | 由 TanStack Query 缓存承担 |
| `grok` | `GrokView` | 数据 + 选中态 | 数据走 Query，选中态入 Zustand |
| `commands/:client?` | `CommandsView` | 数据 + 流式输出（`stream: true`） | 数据走 Query，流式累积缓冲入 Zustand |
| `configs` | `ConfigsView` | 数据 + 选中态 + 搜索词 + 表单草稿 | 数据走 Query，其余入 Zustand（草稿键为配置 id） |
| `usage` | `UsageDashboardView` | 数据 + 时间范围 + 平台维度 | 数据走 Query，筛选条件入 Zustand |

滚动位置由 React Router 的滚动恢复机制处理，不入 store。

`MainLayout` 从 `meta.cache === true` 派生 `cacheKey` 数组的逻辑删除。`cacheKey` 字段保留，仅用于迁移映射表追溯。


### Tauri 与运行时接线

| 文件 | 内容 |
|---|---|
| `src/utils/windowChrome.ts` | 窗口 chrome |
| `src/utils/tauriWindow.ts` | 窗口 API 封装 |
| `src/utils/nativeWindowAppearance.ts` | 原生窗口外观 |
| `src/utils/themeBootstrap.ts` | 主题引导（`data-theme` / `data-flavor` / `data-accent` 解析与写入） |
| `src/utils/fontPreferences.ts` | 字体偏好 |
| `src/utils/startupRecovery.ts` | 启动恢复 |
| `src/utils/perfTelemetry.ts` | 性能遥测 |
| `src/utils/runtimeState.ts` | 运行时状态 |
| `src/utils/tauriRuntime.ts` | Tauri 运行时判定 |
| `src/utils/errorHandler.ts` | 全局错误处理 |
| `src/utils/logger.ts` | 日志 |

以上文件由 `08-22-react-foundation` 的 `src/utils` 判定清单标记为「需接线」，本任务完成接线。

## Requirements

- R1 React 应用可完整启动，Titlebar、侧边导航、内容区、Toast、全局确认弹层全部可用。
- R2 75 条路由全部可达，嵌套结构与路径不变。
- R3 22 处懒加载改为 React 等价机制，首屏加载的模块集合不扩大。
- R4 5 条 `meta.cache` 路由按「状态外提到 store」方案处理，逐条给出状态归属与恢复行为。滚动位置走路由库的滚动恢复。
- R5 8 处 `Teleport` 与 12 处 `Transition` 落在本任务范围内的部分完成改写。
- R6 `themeBootstrap` 在 React 侧生效，`data-theme` / `data-flavor` / `data-accent` 三层模型的解析、写入与存储键读写行为不变。
- R7 窗口 chrome、原生窗口外观、启动恢复三项行为与迁移前一致。
- R8 全局错误处理与日志脱敏（`logRedact.ts`）行为不变。
- R9 `defineExpose`（全仓 8 处）落在本任务范围内的改用 `useImperativeHandle` 或改为受控属性。

## Acceptance Criteria

- [x] AC1 应用启动后可在全部 75 条路由间导航，无白屏与控制台报错。
- [x] AC2 路由清单比对表落盘，75 条路径逐条确认一致。
- [x] AC3 首屏加载模块集合与迁移前对比记录落盘，未扩大。
- [x] AC4 5 条缓存路由逐条验证：离开后返回，数据、选中态、搜索词、筛选条件、滚动位置五项行为与迁移前一致。`commands` 的流式输出续读正常。**表单草稿项只验 store 侧读写**（键为配置 id），其界面级验证见 AC11。
- [x] AC5 切换明暗主题、flavor、accent 后界面正确响应，刷新后偏好保留。
- [x] AC6 窗口最小化、最大化、关闭、拖拽、原生外观切换五项行为验证通过。
- [x] AC7 启动恢复在异常退出后可正确恢复上次状态。
- [x] AC8 日志中无未脱敏的凭据字段，由 smoke 测试断言。
- [x] AC9 `bun run type-check` 与 `bun run lint` 退出码 0。
- [x] AC10 `confirm-interaction-contracts.md` 定义的确认交互行为通过验证。
- [ ] AC11 `configs` 表单草稿的界面级验证（离开后返回草稿仍在）。**该项依赖阶段 5 才迁移的 `ConfigsView`，不在本任务交付门核对**，由 `08-22-views-profiles-config` 批次 2 执行并在父任务视图门核对。本任务的责任是把缺口显式记录并通知对方，不是自行完成。

## 前置与后续

- 前置：`08-22-state-logic-port`。
- 后续：`08-22-views-claude`、`08-22-views-codex`、`08-22-views-secondary-platforms`、`08-22-views-checkin`、`08-22-views-usage`、`08-22-views-profiles-config`、`08-22-views-sync-tools`（七者可并行）。
- 本任务完成后，`08-22-test-contract-rebuild` 应开始交付最小测试集，缩短 IPC 行为回归的保护空窗（见父任务约束 C2）。

## Out of Scope

- 业务视图迁移。
- `src/components/ui/` 原语的形态决策（属 `08-22-design-system`）。
- 归属其他子任务的 9 个根级组件（见上表）。
- i18n 运行时接入（属 `08-22-i18n-port`）。本任务的文案调用点先用临时方案，由 `08-22-i18n-port` 统一切换。

## Notes

- `AnimatedBackground.vue` 与 `StageBackground.vue` 涉及动效，需遵循 reduced motion 降级要求。
- `MasterDetailLayout.vue` 被多个视图子任务消费，接口需在本任务中定稳，变更会波及七个并行子任务。
- `monitoring-log-contracts.md` 与 `logger.ts`、`ansiRenderer.ts` 相关，本任务只保证接线，契约验证属 `08-22-views-sync-tools`。
