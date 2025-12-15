import { ref } from 'vue'
import {
    listCodexAgents, addCodexAgent, updateCodexAgent, deleteCodexAgent, toggleCodexAgent,
    listGeminiAgents, addGeminiAgent, updateGeminiAgent, deleteGeminiAgent, toggleGeminiAgent,
    listQwenAgents, addQwenAgent, updateQwenAgent, deleteQwenAgent, toggleQwenAgent,
    listIflowAgents, addIflowAgent, updateIflowAgent, deleteIflowAgent, toggleIflowAgent,
    listConfigs, getHistory,
    listAgents, addAgent, updateAgent, deleteAgent, toggleAgent
} from '@/api/client'
import type { Agent, AgentRequest } from '@/types'

type ModuleType = 'codex' | 'gemini' | 'qwen' | 'iflow' | 'agents'

interface AgentApi {
    list: () => Promise<{ agents: Agent[], folders?: string[] }>
    add: (req: AgentRequest) => Promise<string>
    update: (name: string, req: AgentRequest) => Promise<string>
    delete: (name: string) => Promise<string>
    toggle: (name: string) => Promise<string>
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
                const configData = await listConfigs()
                currentConfig.value = configData.current_config
                totalConfigs.value = configData.configs.length
                const historyData = await getHistory()
                historyCount.value = historyData.total
            } catch (err) {
                console.error('Failed to load system info:', err)
            }
        } catch (err) {
            console.error(`Failed to load ${module} agents:`, err)
            // alert(t(`${module}.agents.messages.loadFailed`)) // Let the view handle alerts
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
        folders,
        loading,
        currentConfig,
        totalConfigs,
        historyCount,
        loadAgents,
        addAgent,
        updateAgent,
        deleteAgent,
        toggleAgent
    }
}
