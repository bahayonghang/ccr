import { ref } from 'vue'
import { listCodexAgents, addCodexAgent, updateCodexAgent, deleteCodexAgent, toggleCodexAgent } from '@/api'
import { listGeminiAgents, addGeminiAgent, updateGeminiAgent, deleteGeminiAgent, toggleGeminiAgent } from '@/api'
import { listQwenAgents, addQwenAgent, updateQwenAgent, deleteQwenAgent, toggleQwenAgent } from '@/api'
import { listIflowAgents, addIflowAgent, updateIflowAgent, deleteIflowAgent, toggleIflowAgent } from '@/api'
import { listDroidAgents, addDroidAgent, updateDroidAgent, deleteDroidAgent } from '@/api'
import { listConfigs, getHistory } from '@/api'
import { listAgents, getAgent as apiGetAgent, addAgent, updateAgent, deleteAgent, toggleAgent } from '@/api'
import { logger } from '@/utils/logger'
import type { Agent, AgentRequest } from '@/types'
import type { ConfigListResponse, HistoryResponse } from '@/types'

type ModuleType = 'codex' | 'gemini' | 'qwen' | 'iflow' | 'droid' | 'agents'

interface AgentApi {
    list: () => Promise<{ agents: Agent[], folders?: string[] }>
    add: (req: AgentRequest) => Promise<string>
    update: (name: string, req: AgentRequest) => Promise<string>
    delete: (name: string) => Promise<string>
    toggle: (name: string) => Promise<string>
}

function getErrorMessage(err: unknown): string {
    return err instanceof Error ? err.message : String(err)
}

const apiMap: Record<ModuleType, AgentApi> = {
    codex: {
        list: listCodexAgents,
        add: addCodexAgent,
        update: updateCodexAgent,
        delete: deleteCodexAgent,
        toggle: toggleCodexAgent
    },
    gemini: {
        list: listGeminiAgents,
        add: addGeminiAgent,
        update: updateGeminiAgent,
        delete: deleteGeminiAgent,
        toggle: toggleGeminiAgent
    },
    qwen: {
        list: listQwenAgents,
        add: addQwenAgent,
        update: updateQwenAgent,
        delete: deleteQwenAgent,
        toggle: toggleQwenAgent
    },
    iflow: {
        list: listIflowAgents,
        add: addIflowAgent,
        update: updateIflowAgent,
        delete: deleteIflowAgent,
        toggle: toggleIflowAgent
    },
    droid: {
        list: listDroidAgents,
        add: addDroidAgent,
        update: updateDroidAgent,
        delete: deleteDroidAgent,
        toggle: async () => { throw new Error('Droid agents do not support toggle') }
    },
    agents: {
        list: listAgents,
        add: addAgent,
        update: updateAgent,
        delete: deleteAgent,
        toggle: toggleAgent
    }
}

export function useAgents(module: ModuleType) {
    const api = apiMap[module]

    const agents = ref<Agent[]>([])
    const currentAgent = ref<Agent | null>(null)
    const folders = ref<string[]>([])
    const loading = ref(true)
    const currentConfig = ref('')
    const totalConfigs = ref(0)
    const historyCount = ref(0)

    const loadAgents = async () => {
        try {
            loading.value = true
            const data = await api.list()
            agents.value = data.agents || []
            folders.value = data.folders || []

            // Load system info (optional, but kept for consistency with original views)
            try {
                const configData = await listConfigs<ConfigListResponse>()
                currentConfig.value = configData.current_config
                totalConfigs.value = configData.configs.length
                const historyData = await getHistory<HistoryResponse>()
                historyCount.value = historyData.total
            } catch (err) {
                logger.error('Failed to load system info', err)
            }
        } catch (err) {
            logger.error(`Failed to load ${module} agents: ${getErrorMessage(err)}`, err)
            // alert(t(`${module}.agents.messages.loadFailed`)) // Let the view handle alerts
        } finally {
            loading.value = false
        }
    }

    const getAgent = async (name: string): Promise<Agent> => {
        loading.value = true
        try {
            // For now, only Claude Code (agents module) has the getAgent API
            if (module === 'agents') {
                const fetchedAgent = await apiGetAgent<Agent>(name)
                if (!fetchedAgent) {
                    throw new Error(`Agent '${name}' not found`)
                }
                currentAgent.value = fetchedAgent
                return fetchedAgent
            }
            // For other platforms, find from loaded list
            const agent = agents.value.find(a => a.name === name)
            if (agent) {
                currentAgent.value = agent
                return agent
            }
            throw new Error(`Agent '${name}' not found`)
        } finally {
            loading.value = false
        }
    }

    const addAgent = async (req: AgentRequest) => {
        await api.add(req)
        await loadAgents()
    }

    const updateAgent = async (name: string, req: AgentRequest) => {
        await api.update(name, req)
        await loadAgents()
    }

    const deleteAgent = async (name: string) => {
        await api.delete(name)
        await loadAgents()
    }

    const toggleAgent = async (name: string) => {
        await api.toggle(name)
        await loadAgents()
    }

    return {
        agents,
        currentAgent,
        folders,
        loading,
        currentConfig,
        totalConfigs,
        historyCount,
        loadAgents,
        getAgent,
        addAgent,
        updateAgent,
        deleteAgent,
        toggleAgent
    }
}
