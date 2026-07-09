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
        />{{ t('codex.agents.sources.title') }}
      </div>

      <div class="space-y-3">
        <label class="block space-y-2 text-sm text-text-secondary">
          <span class="font-medium text-text-primary">{{ t('codex.agents.sources.repositoryUrl') }}</span>
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
          />{{ t('codex.agents.sources.addSource') }}
        </button>
      </div>

      <div
        v-if="loading && sources.length === 0"
        class="py-10 text-sm text-text-muted"
      >
        {{ t('codex.agents.sources.loading') }}
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
              {{ statusText(source.status) }}
            </span>
          </div>
          <div class="mt-3 flex items-center justify-between text-xs text-text-secondary">
            <span>{{ t('codex.agents.sources.agentCount', { count: source.agentCount }) }}</span>
            <span>{{ sourceFreshnessText(source) }}</span>
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
              {{ catalog?.source.owner && catalog?.source.repo ? `${catalog.source.owner}/${catalog.source.repo}` : t('codex.agents.sources.selectSource') }}
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
              />{{ t('codex.agents.sources.rescan') }}
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
              />{{ t('codex.agents.sources.remove') }}
            </button>
          </div>
        </div>

        <div
          v-if="catalog"
          class="mt-4 grid gap-3 md:grid-cols-4"
        >
          <div class="rounded-2xl border border-border-default/60 bg-bg-surface/60 px-3 py-3">
            <div class="text-[11px] uppercase tracking-wider text-text-muted">
              {{ t('codex.agents.sources.stats.status') }}
            </div>
            <div class="mt-1 text-base font-semibold text-text-primary">
              {{ statusText(catalog.source.status) }}
            </div>
          </div>
          <div class="rounded-2xl border border-border-default/60 bg-bg-surface/60 px-3 py-3">
            <div class="text-[11px] uppercase tracking-wider text-text-muted">
              {{ t('codex.agents.sources.stats.agents') }}
            </div>
            <div class="mt-1 text-base font-semibold text-text-primary">
              {{ catalog.agents.length }}
            </div>
          </div>
          <div class="rounded-2xl border border-border-default/60 bg-bg-surface/60 px-3 py-3">
            <div class="text-[11px] uppercase tracking-wider text-text-muted">
              {{ t('codex.agents.sources.stats.diagnostics') }}
            </div>
            <div class="mt-1 text-base font-semibold text-text-primary">
              {{ catalog.diagnostics.length }}
            </div>
          </div>
          <div class="rounded-2xl border border-border-default/60 bg-bg-surface/60 px-3 py-3">
            <div class="text-[11px] uppercase tracking-wider text-text-muted">
              {{ t('codex.agents.sources.stats.tracked') }}
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
          {{ t('codex.agents.sources.staleCatalogHint') }}
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
          />{{ t('codex.agents.sources.trackedInstalls') }}
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
                  {{ statusText(install.status) }}
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
                  />{{ t('codex.agents.sources.sync') }}
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
                  />{{ t('codex.agents.sources.overwrite') }}
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
                  />{{ t('codex.agents.sources.acceptLocal') }}
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
                  />{{ t('codex.agents.sources.untrack') }}
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
          />{{ t('codex.agents.sources.scanDiagnostics') }}
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
              {{ t('codex.agents.sources.agentCount', { count: group.items.length }) }}
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
                      {{ t('codex.agents.sources.invalid') }}
                    </span>
                    <span
                      v-if="agent.model"
                      class="rounded-full border border-sky-400/25 bg-sky-500/10 px-2 py-0.5 text-[11px] text-sky-100"
                    >
                      {{ agent.model }}
                    </span>
                  </div>
                  <div class="mt-1 text-sm text-text-secondary">
                    {{ agent.description || t('codex.agents.noDescription') }}
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
                    />{{ t('codex.agents.sources.preview') }}
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
                    />{{ t('codex.agents.sources.install') }}
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
      :title="t('codex.agents.sources.previewTitle')"
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
      :title="t('codex.agents.sources.installTitle')"
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
          <span class="font-medium text-text-primary">{{ t('codex.agents.sources.targetName') }}</span>
          <input
            v-model="installTargetName"
            type="text"
            class="w-full rounded-2xl border border-border-default/60 bg-bg-surface/70 px-3 py-2 text-sm text-text-primary"
            :placeholder="t('codex.agents.sources.targetNamePlaceholder')"
          >
        </label>
        <label class="block space-y-2 text-sm text-text-secondary">
          <span class="font-medium text-text-primary">{{ t('codex.agents.sources.conflictPolicy') }}</span>
          <select
            v-model="installConflictMode"
            class="w-full rounded-2xl border border-border-default/60 bg-bg-surface/70 px-3 py-2 text-sm text-text-primary"
          >
            <option value="">
              {{ t('codex.agents.sources.conflictAbort') }}
            </option>
            <option value="replace">
              {{ t('codex.agents.sources.conflictReplace') }}
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
          {{ t('common.cancel') }}
        </button>
        <button
          type="button"
          class="inline-flex min-h-10 items-center rounded-2xl bg-accent-primary px-4 py-2 text-sm font-semibold text-white"
          :disabled="mutating || !installTarget"
          @click="handleInstall"
        >
          {{ t('codex.agents.sources.install') }}
        </button>
      </template>
    </BaseModal>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
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

const { t } = useI18n()
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

function statusText(status: string) {
  switch (status) {
    case 'ok':
      return t('codex.agents.sources.status.ok')
    case 'error':
      return t('codex.agents.sources.status.error')
    case 'access-denied':
      return t('codex.agents.sources.status.accessDenied')
    case 'not-found':
      return t('codex.agents.sources.status.notFound')
    case 'rate-limited':
      return t('codex.agents.sources.status.rateLimited')
    case 'update-available':
      return t('codex.agents.sources.status.updateAvailable')
    case 'conflict':
      return t('codex.agents.sources.status.conflict')
    case 'broken':
      return t('codex.agents.sources.status.broken')
    case 'partial':
      return t('codex.agents.sources.status.partial')
    case 'local-modified':
      return t('codex.agents.sources.status.localModified')
    default:
      return status
  }
}

function sourceFreshnessText(source: { isStale: boolean; scanComplete: boolean }) {
  if (source.isStale) {
    return t('codex.agents.sources.freshness.staleCache')
  }
  return source.scanComplete
    ? t('codex.agents.sources.freshness.complete')
    : t('codex.agents.sources.freshness.partial')
}

async function handleAddSource() {
  try {
    await addSource(sourceUrl.value.trim())
    sourceUrl.value = ''
    uiStore.showSuccess(t('codex.agents.sources.messages.addSuccess'))
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
    uiStore.showSuccess(t('codex.agents.sources.messages.rescanSuccess'))
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}

async function handleRemoveSource(sourceId: string) {
  const confirmed = await uiStore.requestConfirm({
    title: t('codex.agents.sources.confirm.removeTitle'),
    message: t('codex.agents.sources.confirm.removeMessage'),
    confirmText: t('codex.agents.sources.remove'),
    cancelText: t('common.cancel'),
    type: 'danger',
  })
  if (!confirmed) {
    return
  }

  try {
    await removeSource(sourceId)
    uiStore.showSuccess(t('codex.agents.sources.messages.removeSuccess'))
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
    uiStore.showSuccess(t('codex.agents.sources.messages.installSuccess'))
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}

async function handleSyncInstall(installId: string) {
  try {
    await syncInstall(installId)
    emit('refreshInstalled')
    uiStore.showSuccess(t('codex.agents.sources.messages.syncSuccess'))
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}

async function handleForceSyncInstall(installId: string) {
  const confirmed = await uiStore.requestConfirm({
    title: t('codex.agents.sources.confirm.overwriteTitle'),
    message: t('codex.agents.sources.confirm.overwriteMessage'),
    confirmText: t('codex.agents.sources.overwrite'),
    cancelText: t('common.cancel'),
    type: 'danger',
  })
  if (!confirmed) {
    return
  }

  try {
    await forceSyncInstall(installId)
    emit('refreshInstalled')
    uiStore.showSuccess(t('codex.agents.sources.messages.overwriteSuccess'))
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}

async function handleAcceptLocalInstall(installId: string) {
  try {
    await acceptLocalInstall(installId)
    emit('refreshInstalled')
    uiStore.showSuccess(t('codex.agents.sources.messages.acceptLocalSuccess'))
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}

async function handleUntrackInstall(installId: string) {
  const confirmed = await uiStore.requestConfirm({
    title: t('codex.agents.sources.confirm.untrackTitle'),
    message: t('codex.agents.sources.confirm.untrackMessage'),
    confirmText: t('codex.agents.sources.untrack'),
    cancelText: t('common.cancel'),
    type: 'warning',
  })
  if (!confirmed) {
    return
  }

  try {
    await untrackInstall(installId)
    emit('refreshInstalled')
    uiStore.showSuccess(t('codex.agents.sources.messages.untrackSuccess'))
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}
</script>
