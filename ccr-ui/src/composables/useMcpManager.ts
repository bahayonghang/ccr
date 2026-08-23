/**
 * useMcpManager - MCP Manager 页面 Hook（React）
 *
 * 在 useUnifiedMcp 基础上增加:
 * - master-detail 面板状态管理
 * - Fuse.js 模糊搜索
 * - 按名称聚合分组
 * - 多选模式 + 批量操作
 *
 * 08-22-state-logic-port 批次 5c：Vue → React（组件本地 + 服务端数据 SPLIT）。
 * 服务器数据来自 React 版 useUnifiedMcp（普通值，批次 5b-ii 的两处过渡期 `.value`
 * 适配随本重写一并消失）。签名变化（消费方为待迁移 McpManagerView.vue）：
 * 返回字段由 Ref/computed 改为普通值，动作函数名与语义不变。
 *
 * watch/onMounted 映射登记（classification §2）：
 * - onMounted(refresh)：初始拉取由 Query 挂载自动拉取覆盖（useUnifiedMcp 内
 *   mcpKeys.unifiedList），不再显式刷新；
 * - watch(groupedServers)（原 :196，无 immediate/deep，默认 flush pre）→
 *   useEffect + prev 引用比对：仅 groupedServers 实际变化时触发（对齐非 immediate），
 *   首次执行跳过；flush pre 以默认 effect 时序近似。
 */

import { useEffect, useMemo, useRef, useState } from 'react'
import { useFuzzySearch } from '@/composables/useFuzzySearch'
import { useUnifiedMcp } from '@/composables/useUnifiedMcp'
import type { UnifiedMcpServer } from '@/types/unifiedMcp'
import type { McpGroup, McpPanelMode } from '@/types/mcpManager'

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

export function useMcpManager() {
  const mcp = useUnifiedMcp()

  // ============ 面板状态 ============

  const [panelMode, setPanelMode] = useState<McpPanelMode>({ type: 'empty' })
  const [selectedKeys, setSelectedKeys] = useState<Set<string>>(new Set())
  const [isMultiSelectMode, setIsMultiSelectMode] = useState(false)

  // ============ 分组逻辑（原 computed(:117) allGroupedServers：来源 mcp.servers）============

  /** 按名称聚合: 同名 MCP 跨平台归为一组，保留完整 precedence stack */
  const allGroupedServers = useMemo<McpGroup[]>(
    () => {
      const map = new Map<string, UnifiedMcpServer[]>()
      for (const server of mcp.servers) {
        const existing = map.get(server.name) ?? []
        map.set(server.name, [...existing, server])
      }
      return Array.from(map.entries()).map(([name, items]) => createGroup(name, items))
    },
    [mcp.servers],
  )

  // 来源 allGroupedServers、mcp.filterScope
  const groupedServers = useMemo<McpGroup[]>(() => {
    const filter = mcp.filterScope
    if (filter === 'effective') {
      return allGroupedServers.filter(group =>
        group.items.some(item => item.effective !== false && !item.hidden_by),
      )
    }
    if (filter === 'hidden') {
      return allGroupedServers.filter(group =>
        group.items.some(item => item.effective === false || !!item.hidden_by),
      )
    }
    return allGroupedServers.filter(group =>
      group.items.some(item => item.scope === filter),
    )
  }, [allGroupedServers, mcp.filterScope])

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

  /** 当前激活的 group (detail 面板显示)；来源 panelMode、allGroupedServers、filteredGroups */
  const activeGroup = useMemo<McpGroup | null>(() => {
    if (panelMode.type === 'detail' && 'groupName' in panelMode) {
      const targetName = panelMode.groupName
      return allGroupedServers.find(g => g.name === targetName) ?? null
    }
    if (panelMode.type === 'edit' && 'groupName' in panelMode) {
      const targetName = panelMode.groupName
      return allGroupedServers.find(g => g.name === targetName) ?? null
    }
    // 默认选中第一个
    if (panelMode.type === 'empty' && filteredGroups.length > 0) {
      return filteredGroups[0]
    }
    return null
  }, [panelMode, allGroupedServers, filteredGroups])

  /** ListBox 高亮 keys */
  const effectiveSelectedKeys = useMemo<Set<string>>(() => {
    if (selectedKeys.size > 0) return selectedKeys
    if (activeGroup && !isMultiSelectMode) {
      return new Set([activeGroup.name])
    }
    return new Set()
  }, [selectedKeys, activeGroup, isMultiSelectMode])

  /** 多选模式下选中的 groups */
  const selectedGroups = useMemo(
    () => allGroupedServers.filter(g => selectedKeys.has(g.name)),
    [allGroupedServers, selectedKeys],
  )

  // ============ 操作 ============

  function selectGroup(name: string) {
    if (isMultiSelectMode) {
      const next = new Set(selectedKeys)
      if (next.has(name)) {
        next.delete(name)
      } else {
        next.add(name)
      }
      setSelectedKeys(next)
    } else {
      setSelectedKeys(new Set([name]))
      setPanelMode({ type: 'detail', groupName: name })
    }
  }

  function openCreate() {
    setSelectedKeys(new Set())
    setPanelMode({ type: 'create' })
    const filter = mcp.filterScope
    const scope = filter === 'local' || filter === 'project' || filter === 'user'
      ? filter
      : 'user'
    mcp.openAddForm('claude', scope)
  }

  function openImport() {
    setSelectedKeys(new Set())
    setPanelMode({ type: 'import' })
  }

  function openEdit(groupName: string) {
    const group = allGroupedServers.find(g => g.name === groupName)
    if (group && group.items.length > 0) {
      mcp.openEditForm(group.effectiveItem ?? group.items[0])
      setPanelMode({ type: 'edit', groupName })
    }
  }

  function closePanel() {
    setPanelMode(activeGroup
      ? { type: 'detail', groupName: activeGroup.name }
      : { type: 'empty' })
    mcp.closeForm()
  }

  function toggleMultiSelect() {
    if (isMultiSelectMode) {
      setSelectedKeys(new Set())
    }
    setIsMultiSelectMode(!isMultiSelectMode)
  }

  async function deleteGroup(group: McpGroup) {
    for (const item of group.items) {
      await mcp.deleteServer(item)
    }
  }

  async function bulkDelete() {
    for (const group of selectedGroups) {
      await deleteGroup(group)
    }
    setSelectedKeys(new Set())
    setIsMultiSelectMode(false)
  }

  // ============ 刷新 ============

  // 原 onMounted(() => void refresh()) 由 Query 挂载自动拉取覆盖（见文件头登记）
  const refresh = mcp.loadServers

  // 原 watch(:196)：首次加载后，如有数据自动选中第一个。
  // 仅 groupedServers 引用实际变化时触发（对齐非 immediate）；首次执行跳过。
  const prevGroupsRef = useRef(groupedServers)
  useEffect(() => {
    const changed = prevGroupsRef.current !== groupedServers
    prevGroupsRef.current = groupedServers
    if (!changed) return
    if (
      panelMode.type === 'empty'
      && groupedServers.length > 0
      && !isMultiSelectMode
    ) {
      setPanelMode({ type: 'detail', groupName: groupedServers[0].name })
    }
  }, [groupedServers, panelMode.type, isMultiSelectMode])

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
