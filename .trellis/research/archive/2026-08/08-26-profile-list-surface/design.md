# Profile 列表呈现层 — 技术设计

## 文件边界

新增：

- `components/profiles/ProfilesPageHeader.tsx` — 面包屑 + 页头
- `components/profiles/ProfilesOffBanner.tsx` — Off 横幅
- `components/profiles/ProfileCardGrid.tsx` — 卡片视图
- `components/profiles/ProfileTable.tsx` — 表格视图
- `components/profiles/ProfilesEmptyState.tsx` — 两种空态
- `features/platform/profiles/useProfilesSurface.ts` — 呈现层状态与派生
- `features/platform/profiles/resolveRowState.ts` — 行状态纯函数
- `features/platform/profiles/ProfilesSurface.tsx` — 页面装配
- `ccr-ui/tests/profiles-surface.smoke.test.tsx`
- `ccr-ui/tests/profiles-view-mode.smoke.test.ts`
- `ccr-ui/tests/profiles-raw-source.smoke.test.tsx`

改造：

- `components/profiles/ProfilesStatStrip.tsx` — 换槽位契约
- `components/profiles/ProfilesToolbar.tsx` — 按设计稿重排，保留 Filters 弹层
- `components/profiles/ProfilesQuickRail.tsx` — 适配新 props 类型
- `components/profiles/ProfilesInspector*.tsx` — 适配 `ProfileDisplayRecord`
- `components/profiles/profiles-shared.css` — 按新结构重写，保留在线旧类名
- `components/profiles/index.ts` — 导出新组件
- `ccr-ui/tests/fixtures/profiles.ts` — 补 `ProfileDisplayRecord` 夹具（typed 部分由 registry-tokens 建立）
- `ccr-ui/src/i18n/locales/zh-CN/*`、`en-US/*`

不动：`ProfileListRow.tsx`、`ProfilesRawEditorPanel.tsx`（只挂载，不改其 props 契约）、`features/{claude,codex,grok}/**`、`src/configs/profiles.ts`。

## useProfilesSurface

```ts
export function useProfilesSurface(args: {
  platformKey: string;
  presentation: ProfilePresentation;
  records: readonly ProfileDisplayRecord[];
  current: string | null;
}): {
  query: string;
  tagFilter: string | null;
  providerFilter: string | null;
  sortBy: "name" | "usage";
  viewMode: "card" | "table";
  filtered: readonly ProfileDisplayRecord[];
  stats: ProfilesStats;
  allTags: readonly string[];
  allProviders: readonly string[];
  // setters …
};
```

不发起请求。数据由平台控制器加载、剥离凭据、投影后传入（父任务 `design.md`「列表状态」）。

派生规则：

- `filtered`：`record.searchText.includes(query.trim().toLowerCase())`，且 `tagFilter === null || tags.includes(tagFilter)`，且 `providerFilter === null || vendorKey === providerFilter`，再按 `sortBy` 排序。
- `stats.total`：`records.length`，全量而非过滤后。统计条描述的是该平台整体。
- `stats.vendorCount`：`new Set(records.map(r => r.vendorKey).filter(Boolean)).size`。canonical key 由 `08-26-profile-registry-tokens` 的 `toVendorKey()` 在投影阶段算好，本任务不重复实现规范化。
- `stats.tagCounts` / `stats.authCounts`：对全量记录计数，`Record<string, number>`。

派生全部走 `useMemo`，依赖数组按 `react-rerender-discipline.md` 的约束填写。

## 视图模式持久化

复用 `features/profiles/stores.ts` 的既有模式：Zustand store + 手动 `localStorage` 读写，不引入 `zustand/persist`。

```ts
// features/profiles/stores.ts 内扩展，不新建 store 文件
const VIEW_KEY_PREFIX = "ccr:profiles:view:";
// viewByPlatform: Record<string, 'card' | 'table'>
// setView(platform, mode) → 写 state 并 try/catch 写 localStorage
```

模块加载时按前缀扫描水合，与该文件现有的 pinned / recent 水合方式一致。`readNames` / `writeNames` 的 `try/catch` 降级语义直接沿用：storage 抛错时只丢持久化，state 仍更新，当前会话内视图切换正常。

`ProfilesSurface` 卸载不清 state，因此路由离开再回来仍保持选择（AC11 的第一条）。

## 组件契约

### ProfilesPageHeader

```ts
{
  presentation: ProfilePresentation
  environmentLabel: string
  environmentOk: boolean
  loading: boolean
  onAdd(): void
  onReload(): void
  onExport?(): void
  onEditSource?(): void     // rawSource capability 存在时才传
}
```

设计稿页头右侧是「导入」+「新建 Profile」。仓库实际能力是导出而非导入，故次操作按钮渲染为刷新 / 导出 / 原始编辑三项，主按钮为「新建 Profile」。

Off **不进** Header：`profiles-page-contracts.md` 明确要求 Off 横幅位于 Header 与 StatStrip 之间且不得放入 Header 溢出菜单。

### ProfilesOffBanner

```ts
{ canOff: boolean; currentName: string | null; onOff(): Promise<void> }
```

`canOff === false` 时返回 `null`。确认走 `surfaceNotify.confirm({ type: 'warning' })`。

### ProfilesStatStrip（改造）

现签名 `{ current, total, labels, secondary, health, onHealthClick }` 全部替换为：

```ts
{
  current: string | null
  stats: { total: number; vendorCount: number; tagCounts: Record<string, number>; authCounts: Record<string, number> }
  labels: { ... }
}
```

四卡中前两卡是数值卡，后两卡是 chip 列表卡，用同一个卡片外壳 + 两种内容 slot 实现。`health` 槽位取消：设计稿无对应位置，且现有 health 计算无消费方。

`profiles-page-contracts.md` 记录的「StatStrip 特色槽」平台差异（Claude 的 Auth 分布、Codex 的 Config mode）由第四卡「认证方式」承载——两者本质都是认证维度分布，`authKey` / `authLabelKey` 已由 `project()` 按平台产出。该合并需同步更新规格中的平台差异表。

### ProfilesToolbar（改造）

按设计稿重排为单行：搜索框 + 标签 pill 组 + 视图切换段控件。Filters 弹层保留，内含 provider 下拉与排序下拉，焦点陷阱与方向键导航（`trapTab` / `moveByArrow`）一并保留。父任务决策 5：不删除已规格化能力。

搜索框快捷键提示：设计稿写 `⌘K`，仓库既有绑定用 `/`，且 `⌘K` 在 Windows 上不成立。实施时读取现有热键绑定，提示文案跟随实际绑定，不照抄设计稿。

### ProfileCardGrid / ProfileTable

两者共享行状态计算，抽为纯函数：

```ts
export function resolveRowState(
  record: ProfileDisplayRecord,
  presentation: ProfilePresentation,
): {
  dotTone: "active" | "idle";
  badge: { textKey: string; tone: "accent" | "neutral" };
  applyLabelKey: string;
  applyTone: "accent-soft" | "neutral";
  emphasized: boolean;
};
```

`current` 已在 `ProfileDisplayRecord.current` 上，不再单独传。卡片与表格各自只负责布局。

表格网格宽度按设计稿固定：`216px minmax(200px,1fr) 176px 104px 136px 132px`，容器 `min-width: 1024px`，外层 `overflow-x: auto`。col3 与 col4 的表头文案取 `presentation.fieldSlots[1].labelKey` 与 `[2].labelKey`。

卡片网格列数：Inspector 收起为 `repeat(3, 1fr)`，展开为 `repeat(2, 1fr)`。

### ProfilesInspector（改造）

descriptor 由平台侧注入（`utils/{platform}Profiles.ts` 现有构造函数），组件只按 descriptor 渲染。入参的记录类型从旧的平台 DTO 换为 `ProfileDisplayRecord` + 平台注入的 descriptor 行数据，组件内不写平台分支。

### ProfilesEmptyState

```ts
{ variant: 'no-profiles' | 'no-results'; query: string; tagFilter: string | null; providerFilter: string | null; onClear(): void; onAdd(): void }
```

`no-results` 的提示行按设计稿拼接查询词与筛选条件；`no-profiles` 不显示「清除筛选」。

## source mode

`ProfilesSurface` 持有 `sourceMode: boolean`，capability 由 props 传入：

```ts
rawSource?: {
  getRaw(): Promise<RawFileGetResult>
  saveRaw(content: string, token: string, force?: boolean): Promise<RawProfilesSaveResult>
  refreshAll(): Promise<void>
}
```

流程逐项对应 `raw-config-editor-contracts.md`：

| 契约要求                       | 落地                                                                                 |
| ------------------------------ | -------------------------------------------------------------------------------------- |
| 进入前明文警告                 | `useUIStore.requestConfirm`，拒绝则不进入 source mode                                   |
| version token                  | `getRaw()` 返回的 token 存在 `ProfilesSurface` state，`saveRaw` 原样回传                |
| `conflict`                     | 只渲染「重载」与「取消」两个动作，不静默刷新 token                                       |
| `activation_conflict`          | `surfaceNotify.confirm({ type: 'danger' })` 通过后以同一 content 与 token 重试 `force: true` |
| `invalid`                      | 透传 `kind` / `message` / `line` / `column` 到 `ProfilesRawEditorPanel` 的 `errorMarker` |
| 保存后顺序                     | 清 dirty → `setSourceMode(false)` → `await refreshAll()`                                |

`rawSource` 未传入时 `ProfilesPageHeader` 不接收 `onEditSource`，入口不渲染（Grok 场景）。

`ProfilesRawEditorPanel` 的 props 契约不改，本任务只提供调用方。

## 样式

`profiles-shared.css` 当前服务旧结构，按新组件结构重写。类名前缀沿用 `cp-`。

重写约束：`ProfileListRow.tsx`、`ProfilesHeader.tsx` 等仍在线的组件直接 import 该文件。实施第一步先 `rg` 列出这些组件用到的类名，重写时保留，直到 rollout 删除它们。AC18 走查现有 Claude / Codex 页面是该约束的验收。

所有颜色走 token，平台色用 `08-26-profile-registry-tokens` 新增的四角色。「运行中」高亮使用全局 accent（`--color-accent-primary` 系列），不用平台色。

## 测试

| 文件                                  | 覆盖                                                                                      |
| ------------------------------------- | ------------------------------------------------------------------------------------------- |
| `tests/profiles-surface.smoke.test.tsx` | AC1 AC2 AC3 AC4 AC6 AC7 AC8 AC9 AC10：装配、骨架顺序、统计、筛选叠加、双视图、滚动、Inspector、空态 |
| `tests/profiles-view-mode.smoke.test.ts` | AC11：跨卸载保持、按平台隔离、storage 抛错降级                                            |
| `tests/profiles-raw-source.smoke.test.tsx` | AC12：警告、conflict、activation_conflict、保存后顺序、capability 缺席                  |

AC8 的滚动断言在 jsdom 中通过 mock `clientWidth` / `scrollWidth` 完成；真实浏览器测量归 rollout 的走查步骤。

focused 命令：

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/profiles-surface.smoke.test.tsx tests/profiles-view-mode.smoke.test.ts tests/profiles-raw-source.smoke.test.tsx
```

QuickRail 相关回归：

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/profiles-quick-switch.smoke.test.ts tests/profiles-hotkeys.smoke.test.ts tests/profiles-quick-rail.smoke.test.ts
```

## 规格同步

本任务改变两处 `profiles-page-contracts.md` 的既有描述，需在同一任务内更新该文档：

1. StatStrip 的平台特色槽合并为统一的「认证方式」卡。
2. 骨架中新增卡片 / 表格双视图与 source mode 入口。

不改动的条款：Off 横幅位置与 `type=warning`、QuickSwitch 持久化与稳定编号、Filters 的 provider 与排序维度、Inspector 右栏存在性。

## 风险

- `profiles-shared.css` 重写影响仍在引用它的在线组件。缓解见「样式」一节与 AC18。
- Inspector 接入使卡片网格降为两列，与设计稿的三列不同。已记为决策 5 的已知偏差，需写进 `research/design-source.md` 的偏差表，不作为缺陷处理。
- `ProfilesInspector` 的入参类型改动可能牵动 `utils/{platform}Profiles.ts` 的 descriptor 构造函数签名。若需改动，只改签名不改语义，并在 `notes.md` 记录。
