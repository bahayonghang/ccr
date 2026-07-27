import { computed, ref } from 'vue'
import {
  addCodexAgentSource,
  acceptLocalCodexSourceInstall,
  forceSyncCodexSourceInstall,
  getCodexAgentSourceCatalog,
  installCodexSourceAgent,
  listCodexAgentSources,
  removeCodexAgentSource,
  syncCodexAgentSource,
  syncCodexSourceInstall,
  untrackCodexSourceInstall,
} from '@/api'
import type {
  CodexAgentSourceCatalogResponse,
  CodexAgentSourceRecord,
} from '@/types'

export function useCodexAgentSources() {
  const sources = ref<CodexAgentSourceRecord[]>([])
  const selectedSourceId = ref<string | null>(null)
  const catalog = ref<CodexAgentSourceCatalogResponse | null>(null)
  const loading = ref(false)
  const mutating = ref(false)

  const selectedSource = computed(() => {
    return sources.value.find((source) => source.id === selectedSourceId.value) ?? null
  })

  async function refreshSources() {
    loading.value = true
    try {
      const response = await listCodexAgentSources()
      sources.value = response.sources ?? []
      if (!selectedSourceId.value && sources.value.length > 0) {
        selectedSourceId.value = sources.value[0]!.id
      }
    } finally {
      loading.value = false
    }
  }

  async function loadCatalog(sourceId?: string | null) {
    const targetId = sourceId ?? selectedSourceId.value
    if (!targetId) {
      catalog.value = null
      return
    }

    loading.value = true
    try {
      selectedSourceId.value = targetId
      catalog.value = await getCodexAgentSourceCatalog(targetId)
    } finally {
      loading.value = false
    }
  }

  async function refreshSelectedSourceLifecycle(options: {
    sourceId?: string | null
    sync?: boolean
    reloadSources?: boolean
  } = {}) {
    const sourceId = options.sourceId ?? selectedSourceId.value
    if (!sourceId) {
      catalog.value = null
      return
    }

    if (options.sync) {
      await syncCodexAgentSource(sourceId)
    }
    if (options.reloadSources ?? true) {
      await refreshSources()
    }
    await loadCatalog(sourceId)
  }

  async function addSource(url: string) {
    mutating.value = true
    try {
      const source = await addCodexAgentSource(url)
      selectedSourceId.value = source.id
      await refreshSelectedSourceLifecycle({ sourceId: source.id })
    } finally {
      mutating.value = false
    }
  }

  async function removeSource(sourceId: string) {
    mutating.value = true
    try {
      await removeCodexAgentSource(sourceId)
      if (selectedSourceId.value === sourceId) {
        selectedSourceId.value = null
        catalog.value = null
      }
      await refreshSources()
      await refreshSelectedSourceLifecycle({ reloadSources: false })
    } finally {
      mutating.value = false
    }
  }

  async function syncSource(sourceId: string) {
    mutating.value = true
    try {
      await refreshSelectedSourceLifecycle({ sourceId, sync: true })
    } finally {
      mutating.value = false
    }
  }

  async function installAgent(payload: {
    sourceId: string
    agentId: string
    targetName?: string | null
    conflictMode?: string | null
  }) {
    mutating.value = true
    try {
      const result = await installCodexSourceAgent(payload)
      await loadCatalog(payload.sourceId)
      return result
    } finally {
      mutating.value = false
    }
  }

  async function syncInstall(installId: string) {
    mutating.value = true
    try {
      const result = await syncCodexSourceInstall(installId)
      await refreshSelectedSourceLifecycle({ sync: true })
      return result
    } finally {
      mutating.value = false
    }
  }

  async function forceSyncInstall(installId: string) {
    mutating.value = true
    try {
      const result = await forceSyncCodexSourceInstall(installId)
      await refreshSelectedSourceLifecycle({ sync: true })
      return result
    } finally {
      mutating.value = false
    }
  }

  async function acceptLocalInstall(installId: string) {
    mutating.value = true
    try {
      const result = await acceptLocalCodexSourceInstall(installId)
      await refreshSelectedSourceLifecycle({ reloadSources: false })
      return result
    } finally {
      mutating.value = false
    }
  }

  async function untrackInstall(installId: string) {
    mutating.value = true
    try {
      const result = await untrackCodexSourceInstall(installId)
      await refreshSelectedSourceLifecycle({ reloadSources: false })
      return result
    } finally {
      mutating.value = false
    }
  }

  return {
    sources,
    selectedSourceId,
    selectedSource,
    catalog,
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
