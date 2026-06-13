<template>
  <div class="grid gap-4 xl:grid-cols-[320px_minmax(0,1fr)]">
    <Card
      variant="glass"
      class="p-5"
    >
      <div class="mb-3 flex items-center gap-2 text-sm font-semibold text-text-primary">
        <SIcon
          name="Github"
          size="w-4 h-4"
        />GitHub Sources
      </div>

      <div class="space-y-3">
        <label class="block space-y-2 text-sm text-text-secondary">
          <span class="font-medium text-text-primary">Repository URL</span>
          <input
            v-model="sourceUrl"
            type="text"
            class="w-full rounded-2xl border border-border-default/60 bg-bg-surface/70 px-3 py-2 text-sm text-text-primary"
            placeholder="https://github.com/VoltAgent/awesome-codex-subagents"
          >
        </label>
        <button
          type="button"
          class="inline-flex min-h-11 items-center gap-2 rounded-2xl bg-accent-primary px-4 py-2 text-sm font-semibold text-white"
          :disabled="mutating || !sourceUrl.trim()"
          @click="handleAddSource"
        >
          <SIcon
            name="Plus"
            size="w-4 h-4"
          />Add Source
        </button>
      </div>

      <div
        v-if="loading && sources.length === 0"
        class="py-10 text-sm text-text-muted"
      >
        Loading sources...
      </div>

      <div
        v-else
        class="mt-4 space-y-3"
      >
        <button
          v-for="source in sources"
          :key="source.id"
          type="button"
          class="w-full rounded-2xl border px-3 py-3 text-left transition-colors"
          :class="selectedSourceId === source.id ? 'border-accent-primary/40 bg-accent-primary/10' : 'border-border-default/60 bg-bg-surface/55 hover:border-accent-primary/25'"
          @click="handleSelectSource(source.id)"
        >
          <div class="flex items-start justify-between gap-3">
            <div class="min-w-0">
              <div class="truncate text-sm font-semibold text-text-primary">
                {{ source.owner }}/{{ source.repo }}
              </div>
              <div class="mt-1 truncate text-xs text-text-muted">
                {{ source.repoUrl }}
              </div>
            </div>
            <span
              class="rounded-full px-2 py-0.5 text-[11px] font-medium uppercase"
              :class="statusTone(source.status)"
            >
              {{ source.status }}
            </span>
          </div>
          <div class="mt-3 flex items-center justify-between text-xs text-text-secondary">
            <span>{{ source.agentCount }} agents</span>
            <span>{{ source.isStale ? 'stale cache' : source.scanComplete ? 'complete' : 'partial' }}</span>
          </div>
          <div
            v-if="source.lastError"
            class="mt-3 rounded-xl border border-amber-400/25 bg-amber-500/10 px-3 py-2 text-xs text-amber-100"
          >
            {{ source.lastError }}
          </div>
        </button>
      </div>
    </Card>

    <div class="space-y-4">
      <Card
        variant="glass"
        class="p-5"
      >
        <div class="flex flex-wrap items-start justify-between gap-3">
          <div>
            <div class="text-sm font-semibold text-text-primary">
              {{ catalog?.source.owner && catalog?.source.repo ? `${catalog.source.owner}/${catalog.source.repo}` : 'Select a source' }}
            </div>
            <div
              v-if="catalog?.source.repoUrl"
              class="mt-1 break-all text-xs text-text-muted"
            >
              {{ catalog.source.repoUrl }}
            </div>
          </div>
          <div
            v-if="catalog"
            class="flex flex-wrap gap-2"
          >
            <button
              type="button"
              class="inline-flex min-h-10 items-center gap-2 rounded-2xl border border-border-default/60 bg-bg-surface/70 px-3 py-2 text-sm text-text-primary"
              :disabled="mutating"
              @click="handleSyncSource(catalog.source.id)"
            >
              <SIcon
                name="RefreshCcw"
                size="w-4 h-4"
              />Rescan
            </button>
            <button
              type="button"
              class="inline-flex min-h-10 items-center gap-2 rounded-2xl border border-rose-400/35 bg-rose-500/10 px-3 py-2 text-sm text-rose-100"
              :disabled="mutating"
              @click="handleRemoveSource(catalog.source.id)"
            >
              <SIcon
                name="Trash2"
                size="w-4 h-4"
              />Remove
            </button>
          </div>
        </div>

        <div
          v-if="catalog"
          class="mt-4 grid gap-3 md:grid-cols-4"
        >
          <div class="rounded-2xl border border-border-default/60 bg-bg-surface/60 px-3 py-3">
            <div class="text-[11px] uppercase tracking-wider text-text-muted">
              Status
            </div>
            <div class="mt-1 text-base font-semibold text-text-primary">
              {{ catalog.source.status }}
            </div>
          </div>
          <div class="rounded-2xl border border-border-default/60 bg-bg-surface/60 px-3 py-3">
            <div class="text-[11px] uppercase tracking-wider text-text-muted">
              Agents
            </div>
            <div class="mt-1 text-base font-semibold text-text-primary">
              {{ catalog.agents.length }}
            </div>
          </div>
          <div class="rounded-2xl border border-border-default/60 bg-bg-surface/60 px-3 py-3">
            <div class="text-[11px] uppercase tracking-wider text-text-muted">
              Diagnostics
            </div>
            <div class="mt-1 text-base font-semibold text-text-primary">
              {{ catalog.diagnostics.length }}
            </div>
          </div>
          <div class="rounded-2xl border border-border-default/60 bg-bg-surface/60 px-3 py-3">
            <div class="text-[11px] uppercase tracking-wider text-text-muted">
              Tracked
            </div>
            <div class="mt-1 text-base font-semibold text-text-primary">
              {{ catalog.installs.length }}
            </div>
          </div>
        </div>
        <div
          v-if="catalog?.source.isStale"
          class="mt-4 rounded-2xl border border-sky-400/25 bg-sky-500/10 px-4 py-3 text-sm text-sky-100"
        >
          This catalog is being served from cache and may be stale. Use <span class="font-semibold">Rescan</span> to force a fresh GitHub scan.
        </div>
        <div
          v-if="catalog?.source.lastError"
          class="mt-4 rounded-2xl border border-amber-400/25 bg-amber-500/10 px-4 py-3 text-sm text-amber-100"
        >
          {{ catalog.source.lastError }}
        </div>
      </Card>

      <Card
        v-if="catalog?.installs.length"
        variant="glass"
        class="p-5"
      >
        <div class="mb-3 flex items-center gap-2 text-sm font-semibold text-text-primary">
          <SIcon
            name="History"
            size="w-4 h-4"
          />Tracked Installs
        </div>
        <div class="space-y-3">
          <article
            v-for="install in catalog.installs"
            :key="install.id"
            class="rounded-2xl border border-border-default/60 bg-bg-surface/55 p-4"
          >
            <div class="flex flex-wrap items-start justify-between gap-3">
              <div class="min-w-0">
                <div class="text-sm font-semibold text-text-primary">
                  {{ install.installedName }}
                </div>
                <div class="mt-1 break-all text-xs text-text-muted">
                  {{ install.targetPath }}
                </div>
                <div class="mt-2 break-all text-xs text-text-secondary">
                  {{ install.sourcePath }}
                </div>
              </div>
              <div class="flex items-center gap-2">
                <span
                  class="rounded-full px-2 py-0.5 text-[11px] font-medium uppercase"
                  :class="statusTone(install.status)"
                >
                  {{ install.status }}
                </span>
                <button
                  type="button"
                  class="inline-flex min-h-9 items-center gap-2 rounded-xl border border-border-default/60 bg-bg-surface/70 px-3 py-2 text-sm text-text-primary"
                  :disabled="mutating || !(install.hasUpstreamUpdate && !install.hasLocalChanges)"
                  @click="handleSyncInstall(install.id)"
                >
                  <SIcon
                    name="Download"
                    size="w-4 h-4"
                  />Sync
                </button>
                <button
                  v-if="install.status === 'conflict'"
                  type="button"
                  class="inline-flex min-h-9 items-center gap-2 rounded-xl border border-rose-400/35 bg-rose-500/10 px-3 py-2 text-sm text-rose-100"
                  :disabled="mutating"
                  @click="handleForceSyncInstall(install.id)"
                >
                  <SIcon
                    name="AlertOctagon"
                    size="w-4 h-4"
                  />Overwrite
                </button>
                <button
                  v-if="install.status === 'local-modified' || install.status === 'conflict'"
                  type="button"
                  class="inline-flex min-h-9 items-center gap-2 rounded-xl border border-border-default/60 bg-bg-surface/70 px-3 py-2 text-sm text-text-primary"
                  :disabled="mutating"
                  @click="handleAcceptLocalInstall(install.id)"
                >
                  <SIcon
                    name="CheckCheck"
                    size="w-4 h-4"
                  />Accept Local
                </button>
                <button
                  v-if="install.status === 'broken'"
                  type="button"
                  class="inline-flex min-h-9 items-center gap-2 rounded-xl border border-border-default/60 bg-bg-surface/70 px-3 py-2 text-sm text-text-primary"
                  :disabled="mutating"
                  @click="handleUntrackInstall(install.id)"
                >
                  <SIcon
                    name="Unlink2"
                    size="w-4 h-4"
                  />Untrack
                </button>
              </div>
            </div>
            <div
              v-if="install.lastError"
              class="mt-3 rounded-xl border border-amber-400/25 bg-amber-500/10 px-3 py-2 text-xs text-amber-100"
            >
              {{ install.lastError }}
            </div>
          </article>
        </div>
      </Card>

      <Card
        v-if="catalog?.diagnostics.length"
        variant="glass"
        class="p-5"
      >
        <div class="mb-3 flex items-center gap-2 text-sm font-semibold text-text-primary">
          <SIcon
            name="AlertTriangle"
            size="w-4 h-4"
          />Scan Diagnostics
        </div>
        <div class="space-y-3">
          <div
            v-for="diagnostic in catalog.diagnostics"
            :key="`${diagnostic.path}:${diagnostic.message}`"
            class="rounded-2xl border border-amber-400/25 bg-amber-500/10 px-3 py-3 text-sm text-amber-100"
          >
            <div class="break-all font-medium">
              {{ diagnostic.path }}
            </div>
            <div class="mt-1 text-xs">
              {{ diagnostic.message }}
            </div>
          </div>
        </div>
      </Card>

      <template v-if="catalog">
        <Card
          v-for="group in groupedAgents"
          :key="group.category"
          variant="glass"
          class="p-5"
        >
          <div class="mb-4 flex items-center justify-between gap-3">
            <div class="text-sm font-semibold text-text-primary">
              {{ group.label }}
            </div>
            <div class="text-xs text-text-muted">
              {{ group.items.length }} agents
            </div>
          </div>
          <div class="space-y-3">
            <article
              v-for="agent in group.items"
              :key="agent.id"
              class="rounded-2xl border border-border-default/60 bg-bg-surface/55 p-4"
            >
              <div class="flex flex-wrap items-start justify-between gap-3">
                <div class="min-w-0 flex-1">
                  <div class="flex flex-wrap items-center gap-2">
                    <div class="text-sm font-semibold text-text-primary">
                      {{ agent.name }}
                    </div>
                    <span
                      v-if="agent.parseError"
                      class="rounded-full border border-amber-400/30 bg-amber-500/10 px-2 py-0.5 text-[11px] text-amber-100"
                    >
                      invalid
                    </span>
                    <span
                      v-if="agent.model"
                      class="rounded-full border border-sky-400/25 bg-sky-500/10 px-2 py-0.5 text-[11px] text-sky-100"
                    >
                      {{ agent.model }}
                    </span>
                  </div>
                  <div class="mt-1 text-sm text-text-secondary">
                    {{ agent.description || 'No description' }}
                  </div>
                  <div class="mt-2 break-all text-xs text-text-muted">
                    {{ agent.sourcePath }}
                  </div>
                </div>
                <div class="flex items-center gap-2">
                  <button
                    type="button"
                    class="inline-flex min-h-9 items-center gap-2 rounded-xl border border-border-default/60 bg-bg-surface/70 px-3 py-2 text-sm text-text-primary"
                    @click="previewAgent(agent)"
                  >
                    <SIcon
                      name="FileCode2"
                      size="w-4 h-4"
                    />Preview
                  </button>
                  <button
                    type="button"
                    class="inline-flex min-h-9 items-center gap-2 rounded-xl bg-accent-primary px-3 py-2 text-sm font-semibold text-white"
                    :disabled="!!agent.parseError || mutating"
                    @click="openInstallModal(agent)"
                  >
                    <SIcon
                      name="Download"
                      size="w-4 h-4"
                    />Install
                  </button>
                </div>
              </div>
            </article>
          </div>
        </Card>
      </template>
    </div>

    <BaseModal
      :model-value="previewOpen"
      title="Remote Agent Preview"
      size="xl"
      @update:model-value="previewOpen = $event"
    >
      <div class="space-y-3">
        <div
          v-if="previewTarget"
          class="rounded-2xl border border-border-default/60 bg-bg-surface/60 px-4 py-3 text-sm text-text-secondary"
        >
          <div class="font-semibold text-text-primary">
            {{ previewTarget.name }}
          </div>
          <div class="mt-1 break-all text-xs">
            {{ previewTarget.sourcePath }}
          </div>
        </div>
        <textarea
          :value="previewTarget?.rawToml ?? ''"
          rows="22"
          readonly
          class="w-full rounded-2xl border border-border-default/60 bg-bg-surface/70 px-4 py-3 font-mono text-[13px] text-text-primary"
        />
      </div>
    </BaseModal>

    <BaseModal
      :model-value="installOpen"
      title="Install Remote Agent"
      size="md"
      @update:model-value="installOpen = $event"
    >
      <div class="space-y-4">
        <div class="rounded-2xl border border-border-default/60 bg-bg-surface/60 px-4 py-3 text-sm text-text-secondary">
          <div class="font-semibold text-text-primary">
            {{ installTarget?.name }}
          </div>
          <div class="mt-1 break-all text-xs">
            {{ installTarget?.sourcePath }}
          </div>
        </div>
        <label class="block space-y-2 text-sm text-text-secondary">
          <span class="font-medium text-text-primary">Target name</span>
          <input
            v-model="installTargetName"
            type="text"
            class="w-full rounded-2xl border border-border-default/60 bg-bg-surface/70 px-3 py-2 text-sm text-text-primary"
            placeholder="Leave empty to keep the source name"
          >
        </label>
        <label class="block space-y-2 text-sm text-text-secondary">
          <span class="font-medium text-text-primary">Conflict policy</span>
          <select
            v-model="installConflictMode"
            class="w-full rounded-2xl border border-border-default/60 bg-bg-surface/70 px-3 py-2 text-sm text-text-primary"
          >
            <option value="">
              Abort if same-name target exists
            </option>
            <option value="replace">
              Replace existing tracked remote install
            </option>
          </select>
        </label>
      </div>
      <template #footer>
        <button
          type="button"
          class="inline-flex min-h-10 items-center rounded-2xl border border-border-default/60 px-4 py-2 text-sm text-text-secondary"
          @click="installOpen = false"
        >
          Cancel
        </button>
        <button
          type="button"
          class="inline-flex min-h-10 items-center rounded-2xl bg-accent-primary px-4 py-2 text-sm font-semibold text-white"
          :disabled="mutating || !installTarget"
          @click="handleInstall"
        >
          Install
        </button>
      </template>
    </BaseModal>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { getErrorMessage } from '@/utils/errorHandler'
import BaseModal from '@/components/common/BaseModal.vue'
import Card from '@/components/ui/Card.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { useCodexAgentSources } from '@/composables/useCodexAgentSources'
import { useUIStore } from '@/stores/ui'
import type { CodexRemoteAgentRecord } from '@/types'

const emit = defineEmits<{
  refreshInstalled: []
}>()

const uiStore = useUIStore()
const {
  sources,
  selectedSourceId,
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
} = useCodexAgentSources()

const sourceUrl = ref('')
const previewOpen = ref(false)
const previewTarget = ref<CodexRemoteAgentRecord | null>(null)
const installOpen = ref(false)
const installTarget = ref<CodexRemoteAgentRecord | null>(null)
const installTargetName = ref('')
const installConflictMode = ref('')

const groupedAgents = computed(() => {
  const groups = new Map<string, { category: string; label: string; items: CodexRemoteAgentRecord[] }>()

  for (const agent of catalog.value?.agents ?? []) {
    const existing = groups.get(agent.category)
    if (existing) {
      existing.items.push(agent)
      continue
    }

    groups.set(agent.category, {
      category: agent.category,
      label: agent.categoryLabel,
      items: [agent],
    })
  }

  return Array.from(groups.values())
})

onMounted(async () => {
  await refreshSources()
  if (selectedSourceId.value) {
    await loadCatalog(selectedSourceId.value)
  }
})

function statusTone(status: string) {
  switch (status) {
    case 'ok':
      return 'bg-emerald-500/10 text-emerald-200 border border-emerald-400/25'
    case 'error':
      return 'bg-rose-500/10 text-rose-100 border border-rose-400/25'
    case 'access-denied':
    case 'not-found':
      return 'bg-rose-500/10 text-rose-100 border border-rose-400/25'
    case 'rate-limited':
      return 'bg-amber-500/10 text-amber-100 border border-amber-400/25'
    case 'update-available':
      return 'bg-sky-500/10 text-sky-100 border border-sky-400/25'
    case 'conflict':
    case 'broken':
      return 'bg-rose-500/10 text-rose-100 border border-rose-400/25'
    case 'partial':
    case 'local-modified':
      return 'bg-amber-500/10 text-amber-100 border border-amber-400/25'
    default:
      return 'bg-bg-elevated/70 text-text-secondary border border-border-default/50'
  }
}

async function handleAddSource() {
  try {
    await addSource(sourceUrl.value.trim())
    sourceUrl.value = ''
    uiStore.showSuccess('Source added and scanned')
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}

async function handleSelectSource(sourceId: string) {
  try {
    await loadCatalog(sourceId)
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}

async function handleSyncSource(sourceId: string) {
  try {
    await syncSource(sourceId)
    uiStore.showSuccess('Source rescanned')
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}

async function handleRemoveSource(sourceId: string) {
  const confirmed = await uiStore.requestConfirm({
    title: 'Remove source',
    message: 'Remove this GitHub source? Existing installed agents will be kept.',
    confirmText: 'Remove',
    cancelText: 'Cancel',
    type: 'danger',
  })
  if (!confirmed) {
    return
  }

  try {
    await removeSource(sourceId)
    uiStore.showSuccess('Source removed')
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}

function previewAgent(agent: CodexRemoteAgentRecord) {
  previewTarget.value = agent
  previewOpen.value = true
}

function openInstallModal(agent: CodexRemoteAgentRecord) {
  installTarget.value = agent
  installTargetName.value = ''
  installConflictMode.value = ''
  installOpen.value = true
}

async function handleInstall() {
  if (!installTarget.value) {
    return
  }

  try {
    await installAgent({
      sourceId: installTarget.value.sourceId,
      agentId: installTarget.value.id,
      targetName: installTargetName.value.trim() || null,
      conflictMode: installConflictMode.value || null,
    })
    installOpen.value = false
    emit('refreshInstalled')
    uiStore.showSuccess('Remote agent installed')
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}

async function handleSyncInstall(installId: string) {
  try {
    await syncInstall(installId)
    emit('refreshInstalled')
    uiStore.showSuccess('Tracked install synced')
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}

async function handleForceSyncInstall(installId: string) {
  const confirmed = await uiStore.requestConfirm({
    title: 'Overwrite local changes',
    message: 'Replace the local tracked file with the upstream version?',
    confirmText: 'Overwrite',
    cancelText: 'Cancel',
    type: 'danger',
  })
  if (!confirmed) {
    return
  }

  try {
    await forceSyncInstall(installId)
    emit('refreshInstalled')
    uiStore.showSuccess('Upstream version applied')
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}

async function handleAcceptLocalInstall(installId: string) {
  try {
    await acceptLocalInstall(installId)
    emit('refreshInstalled')
    uiStore.showSuccess('Local changes accepted as baseline')
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}

async function handleUntrackInstall(installId: string) {
  const confirmed = await uiStore.requestConfirm({
    title: 'Stop tracking install',
    message: 'Remove provenance tracking for this install and keep the local file?',
    confirmText: 'Untrack',
    cancelText: 'Cancel',
    type: 'warning',
  })
  if (!confirmed) {
    return
  }

  try {
    await untrackInstall(installId)
    emit('refreshInstalled')
    uiStore.showSuccess('Tracking removed')
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}
</script>
