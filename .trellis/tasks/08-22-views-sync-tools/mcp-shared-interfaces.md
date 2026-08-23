# MCP 共享面板接口公示（批次 3 前半）

阶段 4a 交付。消费方：`08-22-platform-unify` 批次 5 的 `PlatformMcpView`，以及本任务批次 3 后半的 `McpManagerView`。

**落点**：`ccr-ui/src/features/mcp/`（父任务 `design.md` 按域聚合）。`src/components/mcp/*.vue` 已删除。feature 层不能导入 `shell/`，故 `t` 由父级注入（`useShellT()`），不在面板内调 i18n 运行时。

## 1. 文件与职责

| 模块 | 职责 | 状态所有权 |
| --- | --- | --- |
| `McpListPanel` | 搜索、多选、添加菜单、刷新、分组列表 | 父级持有 `groups` / 搜索词 / 选中集 / 多选开关 / loading |
| `McpDetailPanel` | 生效配置、优先级、密钥掩码、实例启停、诊断 | 父级持有 `group` 与 `diagnostics`；面板只读 |
| `McpCreatePanel` | 新增/编辑表单 | 父级持有 `useForm<UnifiedMcpRequest>`（`useUnifiedMcp.formApi`）与 STDIO/HTTP 瞬态字段 |
| `McpImportPanel` | JSON 导入、平台/scope、预览 | **面板本地** `useForm`；提交时把解析结果交给父级 |

无具名插槽。Vue 侧四个面板本身也不对外暴露 slot（`ListSearchHeader` 的右侧动作收在 List 面板内部）。

## 2. Vue → React 映射

### `McpListPanel`

| Vue | React |
| --- | --- |
| `groups` `searchQuery` `selectedKeys` `isMultiSelectMode` `loading` | 同名 props |
| `@update:search-query` | `onSearchQueryChange` |
| `@select` `@create` `@import` `@refresh` `@toggleMultiSelect` `@bulkDelete` | `onSelect` `onCreate` `onImport` `onRefresh` `onToggleMultiSelect` `onBulkDelete` |
| `useI18n()` | `t` |

列表项 `McpListItem` 为 `memo`；`key=group.name`。添加菜单走 `src/ui` `DropdownMenu`，不再手写浮层。

父级接线：`useMcpManager` 已透传 `setSearchQuery`。

### `McpDetailPanel`

| Vue | React |
| --- | --- |
| `group` `diagnostics?` | 同名 |
| `@edit`（groupName） | `onEdit(groupName)` |
| `@delete`（group） | `onDelete(group)` |
| `@toggle`（server） | `onToggle(server)` |
| `useI18n()` | `t` |

密钥展示继续走 `maskSecret`（与 Vue 相同规则）。诊断 `key` 用 `source_path + level + message`，不用数组下标。

### `McpCreatePanel`

| Vue | React |
| --- | --- |
| `isEditing` `formData` `isHttpMode` `argInput` `envKey` `envValue` `headerKey` `headerValue` `platforms` `platformMeta` | 同名 |
| `@update-field` | 面板内 `formApi.setValue`（需额外传 `formApi`） |
| 名称 / command / url 的受控 input | `formApi.register`（非受控，满足 rerender-discipline） |
| `@update:isHttpMode` `@update:argInput` `@update:envKey` … | `onIsHttpModeChange` `onArgInputChange` `onEnvKeyChange` … |
| `@submit` `@cancel` `@addEnv` `@removeEnv` `@addHeader` `@removeHeader` | `onSubmit` `onCancel` `onAddEnv` `onRemoveEnv` `onAddHeader` `onRemoveHeader` |
| 原生 `<select>` | `src/ui` `Select` |
| `useI18n()` | `t` |

`formApi` 是 Vue 没有的必填项：对应 `useUnifiedMcp().formApi`。`formData` 必须是 `form.watch()` 快照（platform/scope 条件与 env/headers 展示）。

`argInput` / `envKey` / `envValue` / `headerKey` / `headerValue` 仍在父级 `useState`（`useUnifiedMcp` 已如此）。面板用局部 `useForm({ values })` + `register` 同步这些瞬态字段，避免受控 `value+onChange`。后续 `McpManagerView` 批次可把它们折进主 `formApi`。

父级接线：`useMcpManager` 已透传 `formApi` 与 `setIsHttpMode` / `setArgInput` / `setEnvKey` / `setEnvValue` / `setHeaderKey` / `setHeaderValue`。

### `McpImportPanel`

| Vue | React |
| --- | --- |
| `platforms` `platformMeta` | 同名 |
| `@cancel` | `onCancel` |
| `@import(servers, platform, scope?)` | `onImport(servers, platform, scope?)` |
| 内部 `jsonInput` / `targetPlatform` / `targetScope` | 面板内 `useForm` |
| `useI18n()` | `t` |

解析函数 `parseMcpImportJson` 与面板同行为，供测试与后续视图复用。`ParsedMcpServer` 字段与 Vue `ParsedServer` 一致。

## 3. 状态责任划分

```
useUnifiedMcp / useMcpManager（父级，本批次不迁视图）
  ├─ Query：servers / capabilities / diagnostics
  ├─ RHF：UnifiedMcpRequest（formApi + formData 快照）
  ├─ useState：isHttpMode、argInput、env*、header*、panelMode、selectedKeys
  └─ useFuzzySearch：searchQuery
        │
        ▼
features/mcp 四个面板（展示 + 把用户输入写回上述所有者）
```

面板不调用 IPC，不导入 `@/api` / `@/composables` / `@/shell`。

## 4. 改造需求（不在本批次做）

- 把 `argInput` 与 kv 输入行折进 RHF，去掉 `defaultValue+value` 过渡写法。
- `PlatformMcpView` 接入这些面板（`08-22-platform-unify` 批次 5）。
- `McpManagerView` 从占位页换成真实视图（本任务批次 3 后半）。

已通知 `08-22-platform-unify/implement.md` 协同点 F2。
