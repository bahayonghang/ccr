import { useCallback, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  listGeminiAgents,
  addGeminiAgent as apiAddGeminiAgent,
  updateGeminiAgent as apiUpdateGeminiAgent,
  deleteGeminiAgent as apiDeleteGeminiAgent,
  toggleGeminiAgent as apiToggleGeminiAgent,
  listConfigs,
  getHistory,
  listAgents,
  getAgent as apiGetAgent,
  addAgent as apiAddAgent,
  updateAgent as apiUpdateAgent,
  deleteAgent as apiDeleteAgent,
  toggleAgent as apiToggleAgent,
} from '@/api'
import { genericPlatformDescriptors } from '@/config/platformDescriptors'
import { agentsKeys } from '@/features/agents/queries'
import { getErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'
import type { Agent, AgentRequest } from '@/types'

// agents 域 hook 的 React 迁移（08-22-state-logic-port 批次 5，服务端数据 → Query）。
// 原 loadAgents 的列表 + 系统信息辅助切片合并为单个 list 查询；CRUD 走
// useMutation + invalidateQueries（原实现为顺序 await loadAgents()）。
// 签名变化：返回对象中的 Ref<T> 改为普通值；loading 由 Query isPending 承载
// （消费方均为待迁移 .vue 视图）。

type GenericAgentModule = (typeof genericPlatformDescriptors)[keyof typeof genericPlatformDescriptors]['agents']['module']
type ModuleType = GenericAgentModule | 'agents'

interface AgentApi {
    list: () => Promise<{ agents: Agent[], folders?: string[] }>
    add: (req: AgentRequest) => Promise<unknown>
    update: (name: string, req: AgentRequest) => Promise<unknown>
    delete: (name: string) => Promise<string>
    toggle: (name: string) => Promise<unknown>
}

const apiMap: Record<ModuleType, AgentApi> = {
    gemini: {
        list: listGeminiAgents,
        add: apiAddGeminiAgent,
        update: apiUpdateGeminiAgent,
        delete: apiDeleteGeminiAgent,
        toggle: apiToggleGeminiAgent
    },
    agents: {
        list: listAgents,
        add: apiAddAgent,
        update: apiUpdateAgent,
        delete: apiDeleteAgent,
        toggle: apiToggleAgent
    }
}

interface AgentsPageData {
    agents: Agent[]
    folders: string[]
    currentConfig: string
    totalConfigs: number
    historyCount: number
}

/** 列表数据仅由显式 CRUD mutation 失效驱动；staleTime 0 保持挂载即拉取（原 loadAgents 行为）。 */
const AGENTS_STALE_TIME = 0

export function useAgents(module: ModuleType) {
    const api = apiMap[module]
    const queryClient = useQueryClient()
    const [currentAgent, setCurrentAgent] = useState<Agent | null>(null)

    const query = useQuery({
        queryKey: agentsKeys.list(module),
        queryFn: async (): Promise<AgentsPageData> => {
            try {
                const data = await api.list()

                // Load system info (optional, but kept for consistency with original views)
                let currentConfig = ''
                let totalConfigs = 0
                let historyCount = 0
                try {
                    const configData = await listConfigs()
                    currentConfig = configData.current_config
                    totalConfigs = configData.configs.length
                    const historyData = await getHistory()
                    historyCount = historyData.total
                } catch (err) {
                    logger.error('Failed to load system info', err)
                }

                return {
                    agents: data.agents || [],
                    folders: data.folders || [],
                    currentConfig,
                    totalConfigs,
                    historyCount
                }
            } catch (err) {
                logger.error(`Failed to load ${module} agents: ${getErrorMessage(err)}`, err)
                throw err
            }
        },
        staleTime: AGENTS_STALE_TIME,
    })

    const invalidateList = useCallback(
        () => queryClient.invalidateQueries({ queryKey: agentsKeys.list(module) }),
        [queryClient, module]
    )

    const addMutation = useMutation({ mutationFn: (req: AgentRequest) => api.add(req) })
    const updateMutation = useMutation({
        mutationFn: ({ name, req }: { name: string, req: AgentRequest }) => api.update(name, req)
    })
    const deleteMutation = useMutation({ mutationFn: (name: string) => api.delete(name) })
    const toggleMutation = useMutation({ mutationFn: (name: string) => api.toggle(name) })

    const runMutation = useCallback(
        (action: () => Promise<unknown>) => action().then(() => invalidateList()),
        [invalidateList]
    )

    const loadAgents = useCallback(async () => {
        await query.refetch()
    }, [query])

    const getAgent = useCallback(async (name: string): Promise<Agent> => {
        // For now, only Claude Code (agents module) has the getAgent API
        if (module === 'agents') {
            const fetchedAgent = await apiGetAgent(name)
            if (!fetchedAgent) {
                throw new Error(`Agent '${name}' not found`)
            }
            setCurrentAgent(fetchedAgent)
            return fetchedAgent
        }
        // For other platforms, find from loaded list
        const agent = (query.data?.agents ?? []).find(a => a.name === name)
        if (agent) {
            setCurrentAgent(agent)
            return agent
        }
        throw new Error(`Agent '${name}' not found`)
    }, [module, query.data])

    const addAgent = useCallback(async (req: AgentRequest) => {
        await runMutation(() => addMutation.mutateAsync(req))
    }, [addMutation, runMutation])
    const updateAgent = useCallback(async (name: string, req: AgentRequest) => {
        await runMutation(() => updateMutation.mutateAsync({ name, req }))
    }, [runMutation, updateMutation])
    const deleteAgent = useCallback(async (name: string) => {
        await runMutation(() => deleteMutation.mutateAsync(name))
    }, [deleteMutation, runMutation])
    const toggleAgent = useCallback(async (name: string) => {
        await runMutation(() => toggleMutation.mutateAsync(name))
    }, [runMutation, toggleMutation])

    return {
        agents: query.data?.agents ?? [],
        currentAgent,
        folders: query.data?.folders ?? [],
        loading: query.isPending,
        currentConfig: query.data?.currentConfig ?? '',
        totalConfigs: query.data?.totalConfigs ?? 0,
        historyCount: query.data?.historyCount ?? 0,
        loadAgents,
        getAgent,
        addAgent,
        updateAgent,
        deleteAgent,
        toggleAgent
    }
}
