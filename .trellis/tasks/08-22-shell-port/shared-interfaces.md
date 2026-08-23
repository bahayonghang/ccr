# 共享接口公示（批次 6）

阶段 4a 之后本文件中的接口不改。消费方：七个 `08-22-views-*`。

## 1. `MasterDetailLayout`（`src/ui/master-detail-layout.tsx`）

```ts
interface MasterDetailLayoutProps {
  list: ReactNode
  detail: ReactNode
  listWidth?: string // 默认 '20rem'
  className?: string
}
```

| Vue | React |
| --- | --- |
| `#list` | `list` |
| `#detail` | `detail` |
| `listWidth` | `listWidth` |

责任划分：

- 布局：分栏、默认列表宽度、窄屏改为上下堆叠。
- 消费方：列表滚动、选中、空态、加载、搜索。布局不持有选中态。

## 2. 页壳与页头

`PageShell`：`header` / `subnav` / `children`（原 `#header` `#subnav` 默认槽）。

`PageHeader`：`title` `eyebrow` `description` `eyebrowLang`；`leading` `status` `actions` 对应具名槽。

`PageHeaderCard`：`title` `icon` `description` `badge` `tone`；`meta` `actions` `children`。

## 3. 列表辅助

`ListSearchHeader`：`searchValue` + `onSearchValueChange`（原 `v-model:searchValue`）；`children` 为右侧动作。

`MultiSelectFloatingBar`：`selectedCount` `totalCount` `showDelete` `onDelete`；`children` 为额外动作。

`ScrollToTopButton`：受控 `visible` + `onClick`。内容区滚动由 `MainLayout` 持有。

`MarketplacePagination`：`currentPage` `totalItems` `pageSize` `onPageChange`。

`AgentIcons`：`agents` `compact` `maxVisible`。

`BulkDeleteDialog`：受控 `isOpen`；`onConfirm` / `onCancel`。内部走 `BaseModal`。

`ConfirmModal`：受控 `isOpen`；`onConfirm` / `onCancel` / `onOpenChange`。全局确认走 `useUIStore.requestConfirm` + `GlobalConfirmDialog`。

`HistoryList`：`entries` `loading`；文案可选覆盖。不读 store。

## 4. `src/ui/` 原语

| 文件 | 形态 | 备注 |
| --- | --- | --- |
| `dialog` / `base-modal` | Radix Dialog | 弹层四项行为唯一实现 |
| `popover` `dropdown-menu` `tooltip` `tabs` `combobox` `select` `switch` `checkbox` | shadcn/Radix | 已在 design-system 落地 |
| `s-icon` | Iconify 薄包 | 全站图标入口 |
| `page-header` `page-shell` | 保留版式 | 本任务落地 |
| `empty-state` `async-state-panel` `pill-toggle-group` `stat-tile` `spinner` | 保留并消费 token | 本任务落地 |
| Button / Card / Input / Badge | shadcn，由视图子任务改调用 | 本任务不造第二套 |
| Breadcrumb | 零调用，按需接入 | 不移植 Vue 实现 |
| NavItem | MainLayout 用 `.nav-item` 类，不保留原语 | |
| IconWrapper | 零调用，不进入 `src/ui/` | |
| Sparkline | 归 `08-22-views-usage` | 未在本任务移植 |

约束：`src/ui/` 不得导入 `features/`、`api/`、store。

## 5. 通知

已写入七个视图子任务与 `08-22-test-contract-rebuild` 的 `implement.md`。
`configs` 表单草稿界面级验证（AC11）归 `08-22-views-profiles-config` 批次 2。
