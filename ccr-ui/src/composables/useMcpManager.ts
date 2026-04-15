/**
 * useMcpManager - MCP Manager 页面 Composable
 *
 * 在 useUnifiedMcp 基础上增加:
 * - master-detail 面板状态管理
 * - Fuse.js 模糊搜索
 * - 按名称聚合分组
 * - 多选模式 + 批量操作
 */

import Fuse from 'fuse.js'
import { computed, onMounted, ref, watch } from 'vue'
import { useUnifiedMcp } from '@/composables/useUnifiedMcp'
import type { UnifiedMcpServer } from '@/types/unifiedMcp'
import type { McpGroup, McpPanelMode } from '@/types/mcpManager'

export function useMcpManager() {
  const mcp = useUnifiedMcp()

  // ============ 面板状态 ============

  const panelMode = ref<McpPanelMode>({ type: 'empty' })
  const searchQuery = ref('')
  const selectedKeys = ref<Set<string>>(new Set())
  const isMultiSelectMode = ref(false)

  // ============ 分组逻辑 ============

  /** 按名称聚合: 同名 MCP 跨平台归为一组 */
  const groupedServers = computed<McpGroup[]>(() => {
    const map = new Map<string, UnifiedMcpServer[]>()
    for (const server of mcp.servers.value) {
      const existing = map.get(server.name) ?? []
      map.set(server.name, [...existing, server])
    }
    return Array.from(map.entries()).map(([name, items]) => {
      const first = items[0]
      const isHttp = !!first.url
      return {
        name,
        transportType: isHttp ? 'http' as const : 'stdio' as const,
        transportLabel: isHttp ? (first.url ?? '') : (first.command ?? ''),
        items,
        platforms: [...new Set(items.map(s => s.platform))],
      }
    })
  })

  // ============ Fuse.js 搜索 ============

  const fuse = computed(() =>
    new Fuse(groupedServers.value, {
      keys: [
        { name: 'name', weight: 2 },
        { name: 'transportLabel', weight: 1 },
        { name: 'platforms', weight: 0.5 },
      ],
      threshold: 0.4,
      includeScore: true,
    }),
  )

  const filteredGroups = computed<McpGroup[]>(() => {
    if (!searchQuery.value.trim()) return groupedServers.value
    return fuse.value.search(searchQuery.value).map(r => r.item)
  })

  // ============ 选中状态 ============

  /** 当前激活的 group (detail 面板显示) */
  const activeGroup = computed<McpGroup | null>(() => {
    if (panelMode.value.type === 'detail' && 'groupName' in panelMode.value) {
      const targetName = (panelMode.value as { groupName: string }).groupName
      return groupedServers.value.find(g => g.name === targetName) ?? null
    }
    if (panelMode.value.type === 'edit' && 'groupName' in panelMode.value) {
      const targetName = (panelMode.value as { groupName: string }).groupName
      return groupedServers.value.find(g => g.name === targetName) ?? null
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
    groupedServers.value.filter(g => selectedKeys.value.has(g.name)),
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
    mcp.openAddForm()
  }

  function openImport() {
    selectedKeys.value = new Set()
    panelMode.value = { type: 'import' }
  }

  function openEdit(groupName: string) {
    const group = groupedServers.value.find(g => g.name === groupName)
    if (group && group.items.length > 0) {
      mcp.openEditForm(group.items[0])
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
