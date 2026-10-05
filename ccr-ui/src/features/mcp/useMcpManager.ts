import { useEffect, useMemo, useRef, useState } from 'react'
import type { McpGroup, McpPanelMode } from '@/types/mcpManager'
import type { UnifiedMcpServer } from '@/types/unifiedMcp'
import { useFuzzySearch } from './useFuzzySearch'
import { useUnifiedMcp } from './useUnifiedMcp'

function createGroup(name: string, items: UnifiedMcpServer[]): McpGroup {
  const sortedItems = [...items].sort((a, b) => {
    const order = { local: 0, project: 1, user: 2 } as Record<string, number>
    const scopeOrder = (order[String(a.scope)] ?? 9) - (order[String(b.scope)] ?? 9)
    if (scopeOrder !== 0) return scopeOrder
    return String(a.platform).localeCompare(String(b.platform))
  })
  const first = sortedItems.find((item) => item.effective !== false && !item.hidden_by) ?? sortedItems[0]
  const isHttp = !!first.url
  return {
    name,
    transportType: isHttp ? 'http' : 'stdio',
    transportLabel: isHttp ? (first.url ?? '') : (first.command ?? ''),
    items: sortedItems,
    platforms: [...new Set(sortedItems.map((s) => s.platform))],
    effectiveItem: first,
    scopes: [...new Set(sortedItems.map((s) => String(s.scope ?? 'global')))],
    hiddenCount: sortedItems.filter((s) => s.effective === false || !!s.hidden_by).length,
  }
}

export function useMcpManager() {
  const mcp = useUnifiedMcp()
  const [panelMode, setPanelMode] = useState<McpPanelMode>({ type: 'empty' })
  const [selectedKeys, setSelectedKeys] = useState<Set<string>>(new Set())
  const [isMultiSelectMode, setIsMultiSelectMode] = useState(false)

  const allGroupedServers = useMemo<McpGroup[]>(() => {
    const map = new Map<string, UnifiedMcpServer[]>()
    for (const server of mcp.servers) {
      const existing = map.get(server.name) ?? []
      map.set(server.name, [...existing, server])
    }
    return Array.from(map.entries()).map(([name, items]) => createGroup(name, items))
  }, [mcp.servers])

  const groupedServers = useMemo<McpGroup[]>(() => {
    const filter = mcp.filterScope
    if (filter === 'effective') {
      return allGroupedServers.filter((group) => group.items.some((item) => item.effective !== false && !item.hidden_by))
    }
    if (filter === 'hidden') {
      return allGroupedServers.filter((group) => group.items.some((item) => item.effective === false || !!item.hidden_by))
    }
    return allGroupedServers.filter((group) => group.items.some((item) => item.scope === filter))
  }, [allGroupedServers, mcp.filterScope])

  const { query: searchQuery, setQuery: setSearchQuery, results: filteredGroups } = useFuzzySearch<McpGroup>(
    groupedServers,
    [
      { name: 'name', weight: 2 },
      { name: 'transportLabel', weight: 1 },
      { name: 'platforms', weight: 0.5 },
    ],
    { threshold: 0.4, includeScore: true },
  )

  const activeGroup = useMemo<McpGroup | null>(() => {
    if (panelMode.type === 'detail' && 'groupName' in panelMode) {
      return allGroupedServers.find((g) => g.name === panelMode.groupName) ?? null
    }
    if (panelMode.type === 'edit' && 'groupName' in panelMode) {
      return allGroupedServers.find((g) => g.name === panelMode.groupName) ?? null
    }
    if (panelMode.type === 'empty' && filteredGroups.length > 0) return filteredGroups[0]
    return null
  }, [panelMode, allGroupedServers, filteredGroups])

  const effectiveSelectedKeys = useMemo<Set<string>>(() => {
    if (selectedKeys.size > 0) return selectedKeys
    if (activeGroup && !isMultiSelectMode) return new Set([activeGroup.name])
    return new Set()
  }, [selectedKeys, activeGroup, isMultiSelectMode])

  const selectedGroups = useMemo(
    () => allGroupedServers.filter((g) => selectedKeys.has(g.name)),
    [allGroupedServers, selectedKeys],
  )

  function selectGroup(name: string) {
    if (isMultiSelectMode) {
      const next = new Set(selectedKeys)
      if (next.has(name)) next.delete(name)
      else next.add(name)
      setSelectedKeys(next)
      return
    }
    setSelectedKeys(new Set([name]))
    setPanelMode({ type: 'detail', groupName: name })
  }

  function openCreate() {
    setSelectedKeys(new Set())
    setPanelMode({ type: 'create' })
    const filter = mcp.filterScope
    const scope = filter === 'local' || filter === 'project' || filter === 'user' ? filter : 'user'
    mcp.openAddForm('claude', scope)
  }

  function openImport() {
    setSelectedKeys(new Set())
    setPanelMode({ type: 'import' })
  }

  function openEdit(groupName: string) {
    const group = allGroupedServers.find((g) => g.name === groupName)
    if (group && group.items.length > 0) {
      mcp.openEditForm(group.effectiveItem ?? group.items[0])
      setPanelMode({ type: 'edit', groupName })
    }
  }

  function closePanel() {
    setPanelMode(activeGroup ? { type: 'detail', groupName: activeGroup.name } : { type: 'empty' })
    mcp.closeForm()
  }

  function toggleMultiSelect() {
    if (isMultiSelectMode) setSelectedKeys(new Set())
    setIsMultiSelectMode(!isMultiSelectMode)
  }

  async function deleteGroup(group: McpGroup) {
    for (const item of group.items) await mcp.deleteServer(item)
  }

  async function bulkDelete() {
    for (const group of selectedGroups) await deleteGroup(group)
    setSelectedKeys(new Set())
    setIsMultiSelectMode(false)
  }

  const prevGroupsRef = useRef(groupedServers)
  useEffect(() => {
    const changed = prevGroupsRef.current !== groupedServers
    prevGroupsRef.current = groupedServers
    if (!changed) return
    if (panelMode.type === 'empty' && groupedServers.length > 0 && !isMultiSelectMode) {
      setPanelMode({ type: 'detail', groupName: groupedServers[0].name })
    }
  }, [groupedServers, panelMode.type, isMultiSelectMode])

  return {
    ...mcp,
    panelMode,
    searchQuery,
    setSearchQuery,
    selectedKeys,
    isMultiSelectMode,
    groupedServers,
    filteredGroups,
    activeGroup,
    effectiveSelectedKeys,
    selectedGroups,
    selectGroup,
    openCreate,
    openImport,
    openEdit,
    closePanel,
    toggleMultiSelect,
    deleteGroup,
    bulkDelete,
    refresh: mcp.loadServers,
  }
}
