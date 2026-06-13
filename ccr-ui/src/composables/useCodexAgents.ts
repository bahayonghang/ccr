import { computed, ref } from 'vue'
import {
  addCodexAgent,
  copyCodexAgent,
  deleteCodexAgent,
  getCodexDashboardOverview,
  listCodexAgents,
  listCodexModels,
  renameCodexAgent,
  updateCodexAgent,
  validateCodexAgentToml,
} from '@/api'
import { logger } from '@/utils/logger'
import type {
  CodexAgentContext,
  CodexAgentContextRequest,
  CodexAgentDiagnostic,
  CodexAgentRecord,
  CodexAgentMutationResponse,
  CodexAgentUpsertRequest,
  CodexAgentsResponse,
  CodexModelsResponse,
} from '@/types'

const LAST_PROJECT_ROOT_KEY = 'ccr.codexAgents.lastProjectRoot'

function getErrorMessage(error: unknown) {
  return getErrorMessage(error)
}

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

export function useCodexAgents() {
  const agents = ref<CodexAgentRecord[]>([])
  const diagnostics = ref<CodexAgentDiagnostic[]>([])
  const activeContext = ref<CodexAgentContext | null>(null)
  const lastProjectRoot = ref<string | null>(readLastProjectRoot())
  const loading = ref(false)
  const mutating = ref(false)
  const availableModels = ref<string[]>([])
  const sessionsTotal = ref<number | null>(null)

  const currentContextRequest = computed<CodexAgentContextRequest>(() => {
    if (activeContext.value?.mode === 'project' && activeContext.value.projectRoot) {
      return {
        mode: 'project',
        projectRoot: activeContext.value.projectRoot,
      }
    }

    return { mode: 'global' }
  })

  const hasProjectShortcut = computed(() => !!lastProjectRoot.value)
  const isProjectMode = computed(() => activeContext.value?.mode === 'project')
  const contextLabel = computed(() => activeContext.value?.label ?? 'Global')

  const refresh = async (context?: CodexAgentContextRequest) => {
    loading.value = true
    try {
      const response = await listCodexAgents<CodexAgentsResponse>(context ?? currentContextRequest.value)
      agents.value = response.agents ?? []
      diagnostics.value = response.diagnostics ?? []
      activeContext.value = response.context
      if (response.context?.mode === 'project' && response.context.projectRoot) {
        lastProjectRoot.value = response.context.projectRoot
        writeLastProjectRoot(response.context.projectRoot)
      }
    } finally {
      loading.value = false
    }
  }

  const loadModels = async () => {
    try {
      const response = await listCodexModels<CodexModelsResponse>()
      availableModels.value = response.models ?? []
    } catch (error) {
      logger.error(`Failed to load Codex models: ${getErrorMessage(error)}`, error)
    }
  }

  const loadRuntimeSummary = async () => {
    try {
      const response = await getCodexDashboardOverview<{ inventory?: { sessions_total?: number } }>()
      sessionsTotal.value = response.inventory?.sessions_total ?? null
    } catch (error) {
      logger.error(`Failed to load Codex runtime summary: ${getErrorMessage(error)}`, error)
    }
  }

  const chooseProjectContext = async () => {
    const initialValue = lastProjectRoot.value ?? ''
    const path = typeof window !== 'undefined' && typeof window.prompt === 'function'
      ? window.prompt('Enter Codex project root path', initialValue)
      : null
    if (!path) {
      return false
    }

    lastProjectRoot.value = path
    writeLastProjectRoot(path)
    await refresh({
      mode: 'project',
      projectRoot: path,
    })
    return true
  }

  const switchToProjectContext = async (projectRoot?: string | null) => {
    const path = projectRoot ?? lastProjectRoot.value
    if (!path) {
      return false
    }

    lastProjectRoot.value = path
    writeLastProjectRoot(path)
    await refresh({
      mode: 'project',
      projectRoot: path,
    })
    return true
  }

  const switchToGlobalContext = async () => {
    await refresh({ mode: 'global' })
  }

  const createAgent = async (request: CodexAgentUpsertRequest) => {
    mutating.value = true
    try {
      const { name, ...rest } = request
      await addCodexAgent({
        name,
        ...rest,
        context: currentContextRequest.value,
      })
      await refresh()
    } finally {
      mutating.value = false
    }
  }

  const updateAgentRecord = async (name: string, request: CodexAgentUpsertRequest) => {
    mutating.value = true
    try {
      const { name: requestedName, ...rest } = request
      await updateCodexAgent({
        name,
        ...(requestedName ? { name: requestedName } : {}),
        ...rest,
        context: currentContextRequest.value,
      })
      await refresh()
    } finally {
      mutating.value = false
    }
  }

  const renameAgentRecord = async (name: string, newName: string) => {
    mutating.value = true
    try {
      await renameCodexAgent({
        name,
        newName,
        context: currentContextRequest.value,
      })
      await refresh()
    } finally {
      mutating.value = false
    }
  }

  const deleteAgentRecord = async (name: string) => {
    mutating.value = true
    try {
      await deleteCodexAgent({
        name,
        context: currentContextRequest.value,
      })
      await refresh()
    } finally {
      mutating.value = false
    }
  }

  const validateAgentRecord = async (name: string) => {
    return validateCodexAgentToml<CodexAgentMutationResponse>({
      name,
      context: currentContextRequest.value,
    })
  }

  const copyAgentRecord = async (name: string, targetContext: CodexAgentContextRequest, targetName?: string) => {
    mutating.value = true
    try {
      await copyCodexAgent({
        name,
        targetName,
        sourceContext: currentContextRequest.value,
        targetContext,
      })
      await refresh()
    } finally {
      mutating.value = false
    }
  }

  const refreshAll = async () => {
    await Promise.all([
      refresh(),
      loadModels(),
      loadRuntimeSummary(),
    ])
  }

  return {
    agents,
    diagnostics,
    activeContext,
    context: activeContext,
    activeMode: computed(() => activeContext.value?.mode ?? 'global'),
    availableModels,
    builtInCodexAgents,
    contextLabel,
    currentContextRequest,
    hasProjectShortcut,
    isProjectMode,
    lastProjectRoot,
    loading,
    mutating,
    sessionsTotal,
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
