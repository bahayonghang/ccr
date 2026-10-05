# Profiles 共享层接口公示（批次 1）

阶段 4a 交付。消费方：`08-22-platform-unify` 批次 4 的 `BaseProfiles`，以及 Claude / Codex / Grok Profiles 薄壳。

落点：`ccr-ui/src/components/profiles/*.tsx`（另有 `profiles-shared.css`、`profile-editor-shell.css`）。
描述符类型单源：`ccr-ui/src/utils/profileDescriptors.ts`（utils 不得导入 components/）。

Vue `v-model` / 事件 → React 受控属性与回调。具名 slot → `children` 或具名 render prop。

## 1. 位置与分层

父任务 `design.md` §2 希望域聚合到 `features/profiles/`。本批次仍放在 `src/components/profiles/`（`legacy-feature`），原因：

- `features/platform` 不得导入 `features/profiles`（跨域禁止；只允许导入 `platform`）。
- `features/*` 不得导入 `src/shell`（i18n `useShellT`、`useUIStore`）。
- `utils` 不得导入 `features` 或 `components`。

`08-22-platform-unify` 批次 4 若把 `BaseProfiles` 放进 `features/platform`，应再把本共享层迁到 `features/platform/profiles/`（同域），或经 `features/platform` 再导出。本任务不提前搬迁（Out of Scope：复用不改造）。

## 2. 状态责任

共享层持有的瞬态：

| 组件 | 内部状态 |
| --- | --- |
| `ProfilesHeader` | 溢出菜单开合与焦点 |
| `ProfilesToolbar` | Filters 弹层开合与焦点 |
| `ProfilesQuickRail` | roving tabindex 下标 |
| `ProfilesCommandPalette` | 搜索词、高亮行 |
| `ProfilesRawEditorPanel` | 加载/保存、内容、token、冲突、环境、error marker |

调用方持有并传入：profile 列表、当前名、预览目标、筛选/排序/视图模式、`quickSwitch`、descriptor、i18n 前缀、busy/disabled、sessionWriteAt、selectedTag。

不读 Pinia；`ProfilesRawEditorPanel` 经选择器读 `useUIStore` 的 `requestConfirm` / toast。`ProfilesQuickRail` 不读 store，只消费传入的 `ProfilesQuickSwitch`。

## 3. 组件接口

### `ProfileDiffRows`

```ts
interface ProfileDiffRowsProps {
  rows: ProfileDiffRow[]
  placeholder?: string // 默认 '—'
}
```

无 slot。无内部状态。

### `ProfileListRow`

```ts
interface ProfileListRowProps<T extends ProfileRowProfile> {
  profile: T
  descriptor: ProfileRowDescriptor<T>
  isCurrent: boolean
  disabled?: boolean
  busyAction?: 'apply' | 'delete' | null
  onApply: (name: string) => void
  onEdit: (name: string) => void
  onDelete: (name: string) => void
}
```

| Vue | React |
| --- | --- |
| `@apply` | `onApply` |
| `@edit` | `onEdit` |
| `@delete` | `onDelete` |

类型 `ProfileRowProfile` / `ProfileRowDescriptor` 定义在 `utils/profileDescriptors.ts`，组件再导出。

### `ProfilesSection`

```ts
interface ProfilesSectionProps {
  title: string
  count: number
  children?: ReactNode
}
```

| Vue | React |
| --- | --- |
| 默认 slot | `children` |

### `ProfilesStatStrip`

```ts
interface ProfilesStatStripProps {
  current: string | null
  total: number
  labels: ProfilesStatStripLabels
  secondary: ProfilesStatStripSecondary
  health: ProfilesStatStripHealth
  onHealthClick: () => void
}
```

| Vue | React |
| --- | --- |
| `@healthClick` | `onHealthClick` |

### `ProfilesHeader`

```ts
interface ProfilesHeaderProps {
  icon: string
  backTo: string
  labels: ProfilesHeaderLabels
  loading?: boolean
  exporting?: boolean
  palette?: ProfilesHeaderPalette | null
  paletteOpen?: boolean
  sourceDisabled?: boolean
  sourceTitle?: string
  onAdd: () => void
  onExport: () => void
  onReload: () => void
  onOpenPalette: () => void
  onEditSource: () => void
}
```

| Vue | React |
| --- | --- |
| `RouterLink :to` | `react-router` `Link to` |
| `@add` `@export` `@reload` `@openPalette` `@editSource` | `onAdd` `onExport` `onReload` `onOpenPalette` `onEditSource` |

PageHeader 原 `#leading` / `#actions` 在 React 原语上已是 `leading` / `actions` props，本组件内部接线。

### `ProfilesQuickRail`

```ts
interface ProfilesQuickRailProps<T extends QuickRailProfile> {
  profiles: T[]
  currentName: string | null
  i18nPrefix: string
  disabled?: boolean
  busyName?: string | null
  quickSwitch: ProfilesQuickSwitch // 已是 React 值，不是 Vue Ref
  moreCount?: number
  onApply: (name: string) => void
  onMore: () => void
}
```

| Vue | React |
| --- | --- |
| `@apply` `@more` | `onApply` `onMore` |
| `quickSwitch.pinned.value` | `quickSwitch.pinned` |

空列表时不渲染（原 `v-show`）。

### `ProfilesToolbar`

```ts
interface ProfilesToolbarProps {
  query: string
  statusFilter: ProfilesStatusFilter
  tagFilter: string | null
  sortBy: ProfilesSortBy
  viewMode: ProfilesViewMode // 'card' | 'list'
  resultCount: number
  total: number
  allTags: string[]
  i18nPrefix: string
  providerFilter?: string | null
  allProviders?: ProviderOption[]
  onUpdateQuery: (value: string) => void
  onUpdateStatusFilter: (value: ProfilesStatusFilter) => void
  onUpdateTagFilter: (value: string | null) => void
  onUpdateProviderFilter: (value: string | null) => void
  onUpdateSortBy: (value: ProfilesSortBy) => void
  onUpdateViewMode: (value: ProfilesViewMode) => void
}

interface ProfilesToolbarHandle {
  focusSearch: () => void
}
```

| Vue | React |
| --- | --- |
| `v-model:query` 等 | `query` + `onUpdateQuery` 等同名对 |
| `defineExpose({ focusSearch })` | `forwardRef` + `ProfilesToolbarHandle` |

搜索框保持受控；用 `onInput` 对齐 Vue `@input`。

### `ProfilesInspector`

```ts
interface ProfilesInspectorProps<T extends ProfilesInspectorProfile> {
  profiles: T[]
  previewProfile: T | null
  currentProfile: T | null
  i18nPrefix: string
  descriptor: ProfilesInspectorDescriptor<T>
  sessionWriteAt?: string | null
  selectedTag?: string | null
  onEdit: (name: string) => void
  onLocate: (name: string) => void
  onTagSelect: (tag: string) => void
}
```

| Vue | React |
| --- | --- |
| `@edit` `@locate` `@tag-select` | `onEdit` `onLocate` `onTagSelect` |
| `descriptor.useInsights(Ref<P[]>)` | `descriptor.useInsights(P[])`（与 `utils/profilesInsights` 纯函数一致） |

实现拆成 `ProfilesInspector.tsx` + `ProfilesInspectorPreview.tsx` + `ProfilesInspectorAudit.tsx` + `ProfilesInspectorDistribution.tsx`（`max-lines` 500 / complexity 16）。对外只导出 `ProfilesInspector`。

### `ProfilesCommandPalette`

```ts
interface ProfilesCommandPaletteProps<T extends { name: string }> {
  open: boolean
  profiles: T[]
  descriptor: ProfilesCommandPaletteDescriptor<T>
  actions: ProfilesCommandPaletteAction[]
  i18nPrefix: string
  onUpdateOpen: (value: boolean) => void
  onApply: (name: string) => void
}
```

| Vue | React |
| --- | --- |
| `v-model:open` | `open` + `onUpdateOpen` |
| `@apply` | `onApply` |
| BaseModal `#header` / `#footer` | `header` render prop / `footer` |

弹层走 `src/ui` `BaseModal`。

### `ProfilesRawEditorPanel`

```ts
interface ProfilesRawEditorPanelProps {
  getRaw: () => Promise<RawFileGetResult>
  saveRaw: (content: string, token: string, force?: boolean) => Promise<RawProfilesSaveResult>
  onSaved: () => void
  onClose: () => void
  onDirtyChange?: (dirty: boolean) => void
  renderEditor?: (props: ProfilesRawEditorRenderProps) => ReactNode
}
```

| Vue | React |
| --- | --- |
| `@saved` `@close` `@dirty-change` | `onSaved` `onClose` `onDirtyChange` |
| `onBeforeRouteLeave` | `useBlocker(dirty)` |
| `CodeSourceEditor` | 可选 `renderEditor`；缺省 textarea（见缺陷） |

须挂在 data router 下（`useBlocker`）。`getRaw` / `saveRaw` 仍由调用方注入，不新增 IPC。

## 4. CSS 与 `0.75rem`

- 原 10 个 SFC scoped style 合并为 `profiles-shared.css`，类名 `cp-*` / `profiles-raw-*` 不变。
- `profile-editor-shell.css` 路径不变，供仍未迁移的编辑器模态 `import`。
- 密排元数据字阶 `0.75rem` 原样保留（labels / chips / diff / stats / kbd）。

## 5. 登记缺陷（本批次不改）

1. **跨域落点**：共享层仍在 `components/profiles/`。`BaseProfiles` 若在 `features/platform`，批次 4 需同域搬迁或再导出。
2. **Header / Toolbar 弹层**：仍用手写 popover，未换 `DropdownMenu` / `Popover`。原因：`profiles-page-contracts.md` 的 `≤720px` 底部全宽面板 + Filters 选中后保持打开，与当前 Radix 原语不完全同构。
3. **原始编辑器桥接**：`ProfilesRawEditorPanel` 缺省 textarea。CodeMirror 桥接属 `08-22-views-sync-tools`。对方交付后经 `renderEditor` 注入。
4. **未迁移 Vue 视图**：`ClaudeCodeProfilesView.vue` / `CodexProfilesView.vue` / `GrokProfilesView.vue` 仍 import 已删除的 `.vue`。由对应视图子任务改接到本 React 模块。
5. **`@media (prefers-reduced-motion)`**：共享 CSS 仍用媒体查询，与 Vue 源一致。迁到 `[data-reduced-motion='true']` 留给视图/收口批次。
