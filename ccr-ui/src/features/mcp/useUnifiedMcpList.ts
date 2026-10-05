import { useMutation, useQuery } from '@tanstack/react-query'
import { useCallback, useMemo, useState } from 'react'
import {
  addUnifiedMcp,
  deleteUnifiedMcp,
  listUnifiedMcp,
  toggleUnifiedMcp,
  updateUnifiedMcp,
} from '@/api'
import { mcpKeys } from '@/features/mcp/queries'
import type { UnknownRecord } from '@/types/common'
import type {
  McpScopeFilter,
  PlatformMcpCapability,
  UnifiedMcpDiagnostic,
  UnifiedMcpListResponse,
  UnifiedMcpPlatform,
  UnifiedMcpRequest,
  UnifiedMcpServer,
} from '@/types/unifiedMcp'
import { logger } from '@/utils/logger'
import { ALL_PLATFORMS, toSuccessMessage } from './mcp-constants'
import { mcpNotify } from './notify'

export function useUnifiedMcpList() {
  const listQuery = useQuery({
    queryKey: mcpKeys.unifiedList(),
    staleTime: Infinity,
    queryFn: async () => {
      try {
        return await listUnifiedMcp<UnifiedMcpListResponse>()
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Unknown error'
        logger.error('Failed to load unified MCP servers', err)
        mcpNotify.error(`加载 MCP 服务器失败: ${msg}`)
        throw err
      }
    },
  })

  const servers: UnifiedMcpServer[] = useMemo(
    () => (Array.isArray(listQuery.data?.servers) ? listQuery.data.servers : []),
    [listQuery.data],
  )
  const capabilities: PlatformMcpCapability[] = useMemo(
    () => (Array.isArray(listQuery.data?.capabilities) ? listQuery.data.capabilities : []),
    [listQuery.data],
  )
  const diagnostics: UnifiedMcpDiagnostic[] = useMemo(
    () => (Array.isArray(listQuery.data?.diagnostics) ? listQuery.data.diagnostics : []),
    [listQuery.data],
  )

  const [filterPlatform, setFilterPlatform] = useState<UnifiedMcpPlatform | ''>('')
  const [filterKeyword, setFilterKeyword] = useState('')
  const [filterProtocol, setFilterProtocol] = useState<'all' | 'stdio' | 'http'>('all')
  const [filterScope, setFilterScope] = useState<McpScopeFilter>('effective')

  const filteredServers = useMemo(() => {
    let result = servers
    if (filterPlatform) result = result.filter((s) => s.platform === filterPlatform)
    if (filterProtocol === 'stdio') result = result.filter((s) => s.command && !s.url)
    else if (filterProtocol === 'http') result = result.filter((s) => s.url)
    if (filterScope === 'effective') result = result.filter((s) => s.effective !== false && !s.hidden_by)
    else if (filterScope === 'hidden') result = result.filter((s) => s.effective === false || !!s.hidden_by)
    else result = result.filter((s) => s.scope === filterScope)
    if (filterKeyword) {
      const kw = filterKeyword.toLowerCase()
      result = result.filter(
        (s) =>
          s.name.toLowerCase().includes(kw)
          || (s.command && s.command.toLowerCase().includes(kw))
          || (s.url && s.url.toLowerCase().includes(kw)),
      )
    }
    return result
  }, [filterKeyword, filterPlatform, filterProtocol, filterScope, servers])

  const scopeCounts = useMemo<Record<McpScopeFilter, number>>(
    () => ({
      effective: servers.filter((s) => s.effective !== false && !s.hidden_by).length,
      local: servers.filter((s) => s.scope === 'local').length,
      project: servers.filter((s) => s.scope === 'project').length,
      user: servers.filter((s) => s.scope === 'user').length,
      hidden: servers.filter((s) => s.effective === false || !!s.hidden_by).length,
    }),
    [servers],
  )

  const { refetch: refetchList } = listQuery
  const reloadServers = useCallback(async () => {
    await refetchList()
  }, [refetchList])

  const addMutation = useMutation({
    mutationFn: (request: UnifiedMcpRequest) => addUnifiedMcp<string | UnknownRecord>(request),
  })
  const updateMutation = useMutation({
    mutationFn: (input: { platform: string; name: string; request: UnifiedMcpRequest }) =>
      updateUnifiedMcp<string | UnknownRecord>(input.platform, input.name, input.request),
  })
  const deleteMutation = useMutation({
    mutationFn: (input: { platform: string; name: string; scope?: string }) =>
      deleteUnifiedMcp<string | UnknownRecord>(input.platform, input.name, input.scope),
  })
  const toggleMutation = useMutation({
    mutationFn: (input: { platform: string; name: string; disabled: boolean; scope?: string }) =>
      toggleUnifiedMcp(input.platform, input.name, input.disabled, input.scope),
  })

  const deleteServer = useCallback(async (server: UnifiedMcpServer) => {
    try {
      const message = await deleteMutation.mutateAsync({
        platform: server.platform,
        name: server.name,
        scope: typeof server.scope === 'string' ? server.scope : undefined,
      })
      mcpNotify.success(toSuccessMessage(message, '删除成功'))
      await reloadServers()
      return true
    } catch (err) {
      mcpNotify.error(`删除失败: ${err instanceof Error ? err.message : 'Unknown error'}`)
      return false
    }
  }, [deleteMutation, reloadServers])

  const toggleServer = useCallback(async (server: UnifiedMcpServer) => {
    try {
      const result = await toggleMutation.mutateAsync({
        platform: server.platform,
        name: server.name,
        disabled: !server.disabled,
        scope: typeof server.scope === 'string' ? server.scope : undefined,
      })
      mcpNotify.success(toSuccessMessage(result, '状态已更新'))
      await reloadServers()
      return true
    } catch (err) {
      mcpNotify.error(`切换状态失败: ${err instanceof Error ? err.message : 'Unknown error'}`)
      return false
    }
  }, [reloadServers, toggleMutation])

  const supportsFeature = useCallback(
    (platform: string, feature: keyof Omit<PlatformMcpCapability, 'platform'>): boolean => {
      const cap = capabilities.find((c) => c.platform === platform)
      return cap ? cap[feature] : false
    },
    [capabilities],
  )

  const platformCounts = useMemo(() => {
    const counts: Record<string, number> = {}
    for (const p of ALL_PLATFORMS) counts[p] = servers.filter((s) => s.platform === p).length
    return counts
  }, [servers])

  return {
    servers,
    capabilities,
    diagnostics,
    sourceDiagnostics: diagnostics,
    loading: listQuery.isFetching,
    error: listQuery.error ? (listQuery.error instanceof Error ? listQuery.error.message : 'Unknown error') : null,
    filterPlatform,
    filterKeyword,
    filterProtocol,
    filterScope,
    setFilterPlatform,
    setFilterKeyword,
    setFilterProtocol,
    setFilterScope,
    filteredServers,
    platformCounts,
    scopeCounts,
    loadServers: reloadServers,
    addMutation,
    updateMutation,
    deleteServer,
    toggleServer,
    supportsFeature,
    reloadServers,
  }
}

export type UnifiedMcpListApi = ReturnType<typeof useUnifiedMcpList>
