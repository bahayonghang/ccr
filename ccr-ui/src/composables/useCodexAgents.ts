import { useCallback, useEffect, useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  addCodexAgent,
  copyCodexAgent,
  deleteCodexAgent,
  renameCodexAgent,
  updateCodexAgent,
  validateCodexAgentToml,
} from '@/api'
import {
  codexKeys,
  fetchCodexAgents,
  fetchCodexDashboardOverview,
  fetchCodexModels,
} from '@/features/codex/queries'
import type { CodexDashboardOverview } from '@/api'
import { getErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'
import type {
  CodexAgentContext,
  CodexAgentContextRequest,
  CodexAgentUpsertRequest,
  CodexAgentsResponse,
} from '@/types'

// Codex agents 管理的 React 迁移（08-22-state-logic-port 批次 5，服务端数据 → Query）。
// agents/diagnostics/context 来自 list 查询（key 含 context mode/projectRoot）；
// models 与 runtime summary（复用 dashboard overview 缓存）为独立查询；
// lastProjectRoot 为 localStorage 偏好（useState 初始化 + 同步写回，原语义）。
//
// 签名变化：返回对象中的 Ref<T> 改为普通值；原 refresh(context) 的「显式上下文
// 单次拉取」映射为 requestedContext 状态切换 → key 变化自动拉取（消费方均为
// 待迁移 .vue 视图）。

const LAST_PROJECT_ROOT_KEY = 'ccr.codexAgents.lastProjectRoot'

function readLastProjectRoot(): string | null {
  if (typeof window === 'undefined') {
    return null
  }
  return window.localStorage.getItem(LAST_PROJECT_ROOT_KEY)
}

function writeLastProjectRoot(path: string | null) {
  if (typeof window === 'undefined') {
    return
  }
  if (path) {
    window.localStorage.setItem(LAST_PROJECT_ROOT_KEY, path)
  } else {
    window.localStorage.removeItem(LAST_PROJECT_ROOT_KEY)
  }
}

export const builtInCodexAgents = [
  {
    name: 'default',
    description: 'General-purpose fallback agent',
  },
  {
    name: 'worker',
    description: 'Execution-focused agent for implementation and fixes',
  },
  {
    name: 'explorer',
    description: 'Read-heavy codebase exploration agent',
  },
] as const

/** 列表仅由显式刷新与 mutation 失效驱动；原实现无 TTL → staleTime 0。 */
const CODEX_AGENTS_STALE_TIME = 0

const GLOBAL_CONTEXT_REQUEST: CodexAgentContextRequest = { mode: 'global' }

/** 返回对象的查询数据切片默认值（抽离以控制主 hook 复杂度）。 */
function toCodexAgentsState(options: {
  listData: CodexAgentsResponse | undefined
  modelsData: { models?: string[] } | undefined
  overviewData: CodexDashboardOverview | undefined
  listFetching: boolean
}) {
  const { listData, modelsData, overviewData, listFetching } = options
  return {
    agents: listData?.agents ?? [],
    diagnostics: listData?.diagnostics ?? [],
    availableModels: modelsData?.models ?? [],
    loading: listFetching,
    sessionsTotal: overviewData?.inventory?.sessions_total ?? null,
  }
}

export function useCodexAgents() {
  const [activeContext, setActiveContext] = useState<CodexAgentContext | null>(null)
  // 查询的显式上下文（初始 global，与原 activeContext=null 的派生一致）
  const [requestedContext, setRequestedContext] = useState<CodexAgentContextRequest>(GLOBAL_CONTEXT_REQUEST)
  const [lastProjectRoot, setLastProjectRoot] = useState<string | null>(readLastProjectRoot())
  const [mutating, setMutating] = useState(false)

  // 原 computed currentContextRequest：来源为 activeContext.mode / projectRoot
  const currentContextRequest = useMemo<CodexAgentContextRequest>(
    () => (activeContext?.mode === 'project' && activeContext.projectRoot
      ? { mode: 'project', projectRoot: activeContext.projectRoot }
      : { mode: 'global' }),
    [activeContext?.mode, activeContext?.projectRoot]
  )

  const hasProjectShortcut = useMemo(() => !!lastProjectRoot, [lastProjectRoot])
  const isProjectMode = useMemo(() => activeContext?.mode === 'project', [activeContext?.mode])
  const contextLabel = useMemo(() => activeContext?.label ?? 'Global', [activeContext])
  const activeMode = useMemo(() => activeContext?.mode ?? 'global', [activeContext?.mode])

  const listQuery = useQuery({
    queryKey: codexKeys.agents.list(
      requestedContext.mode ?? 'global',
      requestedContext.mode === 'project' && requestedContext.projectRoot ? requestedContext.projectRoot : null
    ),
    queryFn: () => fetchCodexAgents(requestedContext),
    staleTime: CODEX_AGENTS_STALE_TIME,
  })

  // 原 refresh() 内联的状态回写：response.context 驱动 activeContext 与项目根记忆
  useEffect(() => {
    const response = listQuery.data
    if (!response) return
    setActiveContext(response.context)
    if (response.context?.mode === 'project' && response.context.projectRoot) {
      setLastProjectRoot(response.context.projectRoot)
      writeLastProjectRoot(response.context.projectRoot)
    }
  }, [listQuery.data])

  const modelsQuery = useQuery({
    queryKey: codexKeys.agents.models(),
    queryFn: fetchCodexModels,
    staleTime: CODEX_AGENTS_STALE_TIME,
    // 原 loadModels 失败仅记日志、保留旧值
    retry: false,
    throwOnError: false,
  })

  const overviewQuery = useQuery({
    queryKey: codexKeys.dashboard.overview(),
    queryFn: fetchCodexDashboardOverview,
    staleTime: CODEX_AGENTS_STALE_TIME,
    retry: false,
    throwOnError: false,
  })

  const loadModels = useCallback(async () => {
    await modelsQuery.refetch()
  }, [modelsQuery])

  const loadRuntimeSummary = useCallback(async () => {
    await overviewQuery.refetch()
  }, [overviewQuery])

  /** 原 refresh(context?)：带上下文时切换查询上下文（key 变化自动拉取），否则重拉当前。 */
  const refresh = useCallback(async (context?: CodexAgentContextRequest) => {
    if (context) {
      setRequestedContext(context)
      return
    }
    await listQuery.refetch()
  }, [listQuery])

  const chooseProjectContext = useCallback(async () => {
    const initialValue = lastProjectRoot ?? ''
    const path = typeof window !== 'undefined' && typeof window.prompt === 'function'
      ? window.prompt('Enter Codex project root path', initialValue)
      : null
    if (!path) {
      return false
    }

    setLastProjectRoot(path)
    writeLastProjectRoot(path)
    await refresh({
      mode: 'project',
      projectRoot: path,
    })
    return true
  }, [lastProjectRoot, refresh])

  const switchToProjectContext = useCallback(async (projectRoot?: string | null) => {
    const path = projectRoot ?? lastProjectRoot
    if (!path) {
      return false
    }

    setLastProjectRoot(path)
    writeLastProjectRoot(path)
    await refresh({
      mode: 'project',
      projectRoot: path,
    })
    return true
  }, [lastProjectRoot, refresh])

  const switchToGlobalContext = useCallback(async () => {
    await refresh({ mode: 'global' })
  }, [refresh])

  const runMutation = useCallback(
    async (action: () => Promise<unknown>) => {
      setMutating(true)
      try {
        await action()
        await listQuery.refetch()
      } finally {
        setMutating(false)
      }
    },
    [listQuery]
  )

  const createAgent = useCallback((request: CodexAgentUpsertRequest) => runMutation(async () => {
    const { name, ...rest } = request
    await addCodexAgent({
      name,
      ...rest,
      context: currentContextRequest,
    })
  }), [currentContextRequest, runMutation])

  const updateAgentRecord = useCallback((name: string, request: CodexAgentUpsertRequest) => runMutation(async () => {
    const { name: requestedName, ...rest } = request
    await updateCodexAgent({
      name,
      ...(requestedName ? { name: requestedName } : {}),
      ...rest,
      context: currentContextRequest,
    })
  }), [currentContextRequest, runMutation])

  const renameAgentRecord = useCallback((name: string, newName: string) => runMutation(async () => {
    await renameCodexAgent({
      name,
      newName,
      context: currentContextRequest,
    })
  }), [currentContextRequest, runMutation])

  const deleteAgentRecord = useCallback((name: string) => runMutation(async () => {
    await deleteCodexAgent({
      name,
      context: currentContextRequest,
    })
  }), [currentContextRequest, runMutation])

  const validateAgentRecord = useCallback(async (name: string) => {
    return validateCodexAgentToml({
      name,
      context: currentContextRequest,
    })
  }, [currentContextRequest])

  const copyAgentRecord = useCallback((
    name: string,
    targetContext: CodexAgentContextRequest,
    targetName?: string
  ) => runMutation(async () => {
    await copyCodexAgent({
      name,
      targetName,
      sourceContext: currentContextRequest,
      targetContext,
    })
  }), [currentContextRequest, runMutation])

  const refreshAll = useCallback(async () => {
    await Promise.all([
      refresh(),
      loadModels(),
      loadRuntimeSummary(),
    ])
  }, [loadModels, loadRuntimeSummary, refresh])

  // models/overview 失败走日志降级（原 loadModels/loadRuntimeSummary catch 分支）
  useEffect(() => {
    if (modelsQuery.error) {
      logger.error(`Failed to load Codex models: ${getErrorMessage(modelsQuery.error)}`, modelsQuery.error)
    }
  }, [modelsQuery.error])
  useEffect(() => {
    if (overviewQuery.error) {
      logger.error(`Failed to load Codex runtime summary: ${getErrorMessage(overviewQuery.error)}`, overviewQuery.error)
    }
  }, [overviewQuery.error])


  const state = toCodexAgentsState({
    listData: listQuery.data,
    modelsData: modelsQuery.data,
    overviewData: overviewQuery.data,
    listFetching: listQuery.isFetching,
  })

  return {
    ...state,
    activeContext,
    context: activeContext,
    activeMode,
    builtInCodexAgents,
    contextLabel,
    currentContextRequest,
    hasProjectShortcut,
    isProjectMode,
    lastProjectRoot,
    mutating,
    refresh,
    loadAgents: refresh,
    refreshAll,
    chooseProjectContext,
    pickProjectRoot: chooseProjectContext,
    switchToProjectContext,
    reopenLastProjectRoot: switchToProjectContext,
    switchToGlobalContext,
    returnToGlobal: switchToGlobalContext,
    createAgent,
    addAgent: createAgent,
    updateAgentRecord,
    saveAgent: updateAgentRecord,
    renameAgentRecord,
    renameAgent: renameAgentRecord,
    deleteAgentRecord,
    removeAgent: deleteAgentRecord,
    validateAgentRecord,
    validateAgent: validateAgentRecord,
    copyAgentRecord,
    copyAgent: copyAgentRecord,
  }
}
