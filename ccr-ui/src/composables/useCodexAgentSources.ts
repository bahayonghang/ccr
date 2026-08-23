import { useCallback, useEffect, useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  acceptLocalCodexSourceInstall,
  addCodexAgentSource,
  forceSyncCodexSourceInstall,
  installCodexSourceAgent,
  removeCodexAgentSource,
  syncCodexAgentSource,
  syncCodexSourceInstall,
  untrackCodexSourceInstall,
} from '@/api'
import {
  codexKeys,
  fetchCodexAgentSourceCatalog,
  fetchCodexAgentSources,
} from '@/features/codex/queries'
import type { CodexAgentSourceRecord } from '@/types'

// Codex agent source 管理的 React 迁移（08-22-state-logic-port 批次 5，
// 服务端数据 → Query）。sources / catalog 为查询；selectedSourceId 为选中瞬态
// （useState），catalog 派生自 selectedSourceId（原 loadCatalog 的显式清空语义
// 由「无选中 → catalog=null」的派生规则承担）。
// 签名变化：返回对象中的 Ref<T> 改为普通值；loading/mutating 由 Query
// fetchStatus 与本地 useState 承载（消费方均为待迁移 .vue 视图）。

export function useCodexAgentSources() {
  const [selectedSourceId, setSelectedSourceId] = useState<string | null>(null)
  const [mutating, setMutating] = useState(false)

  // 原实现每次进面板都全量刷新、无 TTL → staleTime 0
  const sourcesQuery = useQuery({
    queryKey: codexKeys.agentSources.list(),
    queryFn: async () => {
      const response = await fetchCodexAgentSources()
      return (response.sources ?? []) as CodexAgentSourceRecord[]
    },
    staleTime: 0,
  })

  const sources = sourcesQuery.data

  // 原 refreshSources 内联的自动选中：列表就绪且未选中时选第一项
  useEffect(() => {
    if (!selectedSourceId && sources && sources.length > 0) {
      setSelectedSourceId(sources[0]!.id)
    }
  }, [selectedSourceId, sources])

  const catalogQuery = useQuery({
    queryKey: codexKeys.agentSources.catalog(selectedSourceId),
    queryFn: () => fetchCodexAgentSourceCatalog(selectedSourceId!),
    enabled: selectedSourceId !== null,
    staleTime: 0,
  })

  const selectedSource = useMemo(
    () => sources?.find((source) => source.id === selectedSourceId) ?? null,
    [sources, selectedSourceId]
  )

  const loading = sourcesQuery.isFetching || catalogQuery.isFetching

  /** 原刷新入口：重拉列表；选中态保持（自动选中由 effect 承担）。 */
  const refreshSources = useCallback(async () => {
    await sourcesQuery.refetch()
  }, [sourcesQuery])

  /**
   * 原 loadCatalog：切换选中即触发 catalog 拉取（key 变化自动 refetch）；
   * 无目标时清空选中，catalog 派生为 null（等价原 catalog.value = null 分支）。
   */
  const loadCatalog = useCallback(async (sourceId?: string | null) => {
    setSelectedSourceId(sourceId ?? null)
  }, [])

  const refreshSelectedSourceLifecycle = useCallback(async (options: {
    sourceId?: string | null
    sync?: boolean
    reloadSources?: boolean
  } = {}) => {
    const targetId = options.sourceId ?? selectedSourceId
    if (!targetId) {
      setSelectedSourceId(null)
      return
    }

    if (options.sync) {
      await syncCodexAgentSource(targetId)
    }
    if (options.reloadSources ?? true) {
      await sourcesQuery.refetch()
    }
    setSelectedSourceId(targetId)
  }, [selectedSourceId, sourcesQuery])

  const addSource = useCallback(async (url: string) => {
    setMutating(true)
    try {
      const source = await addCodexAgentSource(url)
      await refreshSelectedSourceLifecycle({ sourceId: source.id })
    } finally {
      setMutating(false)
    }
  }, [refreshSelectedSourceLifecycle])

  const removeSource = useCallback(async (sourceId: string) => {
    setMutating(true)
    try {
      await removeCodexAgentSource(sourceId)
      if (selectedSourceId === sourceId) {
        setSelectedSourceId(null)
      }
      await sourcesQuery.refetch()
      await refreshSelectedSourceLifecycle({ reloadSources: false })
    } finally {
      setMutating(false)
    }
  }, [refreshSelectedSourceLifecycle, selectedSourceId, sourcesQuery])

  const syncSource = useCallback(async (sourceId: string) => {
    setMutating(true)
    try {
      await refreshSelectedSourceLifecycle({ sourceId, sync: true })
    } finally {
      setMutating(false)
    }
  }, [refreshSelectedSourceLifecycle])

  const runMutation = useCallback(
    async (action: () => Promise<unknown>) => {
      setMutating(true)
      try {
        return await action()
      } finally {
        setMutating(false)
      }
    },
    []
  )

  const installAgent = useCallback((payload: {
    sourceId: string
    agentId: string
    targetName?: string | null
    conflictMode?: string | null
  }) => runMutation(async () => {
    const result = await installCodexSourceAgent(payload)
    setSelectedSourceId(payload.sourceId)
    return result
  }), [runMutation])

  const syncInstall = useCallback((installId: string) => runMutation(async () => {
    const result = await syncCodexSourceInstall(installId)
    await refreshSelectedSourceLifecycle({ sync: true })
    return result
  }), [refreshSelectedSourceLifecycle, runMutation])

  const forceSyncInstall = useCallback((installId: string) => runMutation(async () => {
    const result = await forceSyncCodexSourceInstall(installId)
    await refreshSelectedSourceLifecycle({ sync: true })
    return result
  }), [refreshSelectedSourceLifecycle, runMutation])

  const acceptLocalInstall = useCallback((installId: string) => runMutation(async () => {
    const result = await acceptLocalCodexSourceInstall(installId)
    await refreshSelectedSourceLifecycle({ reloadSources: false })
    return result
  }), [refreshSelectedSourceLifecycle, runMutation])

  const untrackInstall = useCallback((installId: string) => runMutation(async () => {
    const result = await untrackCodexSourceInstall(installId)
    await refreshSelectedSourceLifecycle({ reloadSources: false })
    return result
  }), [refreshSelectedSourceLifecycle, runMutation])

  return {
    sources: sources ?? [],
    selectedSourceId,
    selectedSource,
    catalog: selectedSourceId !== null ? catalogQuery.data ?? null : null,
    loading,
    mutating,
    refreshSources,
    loadCatalog,
    addSource,
    removeSource,
    syncSource,
    installAgent,
    syncInstall,
    forceSyncInstall,
    acceptLocalInstall,
    untrackInstall,
  }
}
