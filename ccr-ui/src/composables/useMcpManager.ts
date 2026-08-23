/**
 * useMcpManager - MCP Manager 页面 Composable
 *
 * 在 useUnifiedMcp 基础上增加:
 * - master-detail 面板状态管理
 * - Fuse.js 模糊搜索
 * - 按名称聚合分组
 * - 多选模式 + 批量操作
 */

import { computed, onMounted, ref, watch } from 'vue'
import { useFuzzySearch } from '@/composables/useFuzzySearch'
import { useUnifiedMcp } from '@/composables/useUnifiedMcp'
import type { UnifiedMcpServer } from '@/types/unifiedMcp'
import type { McpGroup, McpPanelMode } from '@/types/mcpManager'

/**
 * 过渡期适配（批次 5b-ii）：useUnifiedMcp 已转为 React hook（普通值返回），
 * 本 composable 的 `.value` 读取点已同步改为直接属性访问；整体 React 重写归批次 5c。
 */
export function useMcpManager() {
  const mcp = useUnifiedMcp()

  // ============ 面板状态 ============

  const panelMode = ref<McpPanelMode>({ type: 'empty' })
  const selectedKeys = ref<Set<string>>(new Set())
  const isMultiSelectMode = ref(false)

  // ============ 分组逻辑 ============

  function createGroup(name: string, items: UnifiedMcpServer[]): McpGroup {
    const sortedItems = [...items].sort((a, b) => {
      const order = { local: 0, project: 1, user: 2 } as Record<string, number>
      const scopeOrder = (order[String(a.scope)] ?? 9) - (order[String(b.scope)] ?? 9)
      if (scopeOrder !== 0) return scopeOrder
      return String(a.platform).localeCompare(String(b.platform))
    })
    const first = sortedItems.find(item => item.effective !== false && !item.hidden_by) ?? sortedItems[0]
    const isHttp = !!first.url
    return {
      name,
      transportType: isHttp ? 'http' as const : 'stdio' as const,
      transportLabel: isHttp ? (first.url ?? '') : (first.command ?? ''),
      items: sortedItems,
      platforms: [...new Set(sortedItems.map(s => s.platform))],
      effectiveItem: first,
      scopes: [...new Set(sortedItems.map(s => String(s.scope ?? 'global')))],
      hiddenCount: sortedItems.filter(s => s.effective === false || !!s.hidden_by).length,
    }
  }

  /** 按名称聚合: 同名 MCP 跨平台归为一组，保留完整 precedence stack */
  const allGroupedServers = computed<McpGroup[]>(() => {
    const map = new Map<string, UnifiedMcpServer[]>()
    for (const server of mcp.servers) {
      const existing = map.get(server.name) ?? []
      map.set(server.name, [...existing, server])
    }
    return Array.from(map.entries()).map(([name, items]) => createGroup(name, items))
  })

  const groupedServers = computed<McpGroup[]>(() => {
    const filter = mcp.filterScope
    if (filter === 'effective') {
      return allGroupedServers.value.filter(group =>
        group.items.some(item => item.effective !== false && !item.hidden_by),
      )
    }
    if (filter === 'hidden') {
      return allGroupedServers.value.filter(group =>
        group.items.some(item => item.effective === false || !!item.hidden_by),
      )
    }
    return allGroupedServers.value.filter(group =>
      group.items.some(item => item.scope === filter),
    )
  })

  // ============ Fuse.js 搜索 ============

  const { query: searchQuery, results: filteredGroups } = useFuzzySearch<McpGroup>(
    groupedServers,
    [
      { name: 'name', weight: 2 },
      { name: 'transportLabel', weight: 1 },
      { name: 'platforms', weight: 0.5 },
    ],
    { threshold: 0.4, includeScore: true },
  )

  // ============ 选中状态 ============

  /** 当前激活的 group (detail 面板显示) */
  const activeGroup = computed<McpGroup | null>(() => {
    if (panelMode.value.type === 'detail' && 'groupName' in panelMode.value) {
      const targetName = (panelMode.value as { groupName: string }).groupName
      return allGroupedServers.value.find(g => g.name === targetName) ?? null
    }
    if (panelMode.value.type === 'edit' && 'groupName' in panelMode.value) {
      const targetName = (panelMode.value as { groupName: string }).groupName
      return allGroupedServers.value.find(g => g.name === targetName) ?? null
    }
    // 默认选中第一个
    if (panelMode.value.type === 'empty' && filteredGroups.value.length > 0) {
      return filteredGroups.value[0]
    }
    return null
  })

  /** ListBox 高亮 keys */
  const effectiveSelectedKeys = computed<Set<string>>(() => {
    if (selectedKeys.value.size > 0) return selectedKeys.value
    if (activeGroup.value && !isMultiSelectMode.value) {
      return new Set([activeGroup.value.name])
    }
    return new Set()
  })

  /** 多选模式下选中的 groups */
  const selectedGroups = computed(() =>
    allGroupedServers.value.filter(g => selectedKeys.value.has(g.name)),
  )

  // ============ 操作 ============

  function selectGroup(name: string) {
    if (isMultiSelectMode.value) {
      const next = new Set(selectedKeys.value)
      if (next.has(name)) {
        next.delete(name)
      } else {
        next.add(name)
      }
      selectedKeys.value = next
    } else {
      selectedKeys.value = new Set([name])
      panelMode.value = { type: 'detail', groupName: name }
    }
  }

  function openCreate() {
    selectedKeys.value = new Set()
    panelMode.value = { type: 'create' }
    const filter = mcp.filterScope
    const scope = filter === 'local' || filter === 'project' || filter === 'user'
      ? filter
      : 'user'
    mcp.openAddForm('claude', scope)
  }

  function openImport() {
    selectedKeys.value = new Set()
    panelMode.value = { type: 'import' }
  }

  function openEdit(groupName: string) {
    const group = allGroupedServers.value.find(g => g.name === groupName)
    if (group && group.items.length > 0) {
      mcp.openEditForm(group.effectiveItem ?? group.items[0])
      panelMode.value = { type: 'edit', groupName }
    }
  }

  function closePanel() {
    panelMode.value = activeGroup.value
      ? { type: 'detail', groupName: activeGroup.value.name }
      : { type: 'empty' }
    mcp.closeForm()
  }

  function toggleMultiSelect() {
    isMultiSelectMode.value = !isMultiSelectMode.value
    if (!isMultiSelectMode.value) {
      selectedKeys.value = new Set()
    }
  }

  async function deleteGroup(group: McpGroup) {
    for (const item of group.items) {
      await mcp.deleteServer(item)
    }
  }

  async function bulkDelete() {
    for (const group of selectedGroups.value) {
      await deleteGroup(group)
    }
    selectedKeys.value = new Set()
    isMultiSelectMode.value = false
  }

  // ============ 刷新 ============

  async function refresh() {
    await mcp.loadServers()
  }

  // 首次加载后，如有数据自动选中第一个
  watch(groupedServers, (groups) => {
    if (
      panelMode.value.type === 'empty'
      && groups.length > 0
      && !isMultiSelectMode.value
    ) {
      panelMode.value = { type: 'detail', groupName: groups[0].name }
    }
  })

  onMounted(() => {
    void refresh()
  })

  // ============ 返回 ============

  return {
    // 来自 useUnifiedMcp (透传需要的部分)
    servers: mcp.servers,
    loading: mcp.loading,
    error: mcp.error,
    capabilities: mcp.capabilities,
    diagnostics: mcp.diagnostics,
    sourceDiagnostics: mcp.sourceDiagnostics,
    showForm: mcp.showForm,
    editingServer: mcp.editingServer,
    isHttpMode: mcp.isHttpMode,
    formData: mcp.formData,
    argInput: mcp.argInput,
    envKey: mcp.envKey,
    envValue: mcp.envValue,
    headerKey: mcp.headerKey,
    headerValue: mcp.headerValue,
    includeToolInput: mcp.includeToolInput,
    currentCapability: mcp.currentCapability,
    filterScope: mcp.filterScope,
    scopeCounts: mcp.scopeCounts,
    filteredServers: mcp.filteredServers,
    submitForm: mcp.submitForm,
    addEnvVar: mcp.addEnvVar,
    removeEnvVar: mcp.removeEnvVar,
    addHeader: mcp.addHeader,
    removeHeader: mcp.removeHeader,
    toggleServer: mcp.toggleServer,
    supportsFeature: mcp.supportsFeature,
    PLATFORM_META: mcp.PLATFORM_META,
    ALL_PLATFORMS: mcp.ALL_PLATFORMS,

    // Manager 新增
    panelMode,
    searchQuery,
    selectedKeys,
    isMultiSelectMode,
    groupedServers,
    filteredGroups,
    activeGroup,
    effectiveSelectedKeys,
    selectedGroups,

    // 操作
    selectGroup,
    openCreate,
    openImport,
    openEdit,
    closePanel,
    toggleMultiSelect,
    deleteGroup,
    bulkDelete,
    refresh,
  }
}
