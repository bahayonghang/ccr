<template>
  <div class="min-h-full p-6 transition-colors duration-300">
    <div class="mx-auto max-w-[1800px]">
      <ModuleSubnav
        module="codex"
        class="mb-6"
      />

      <PageHeaderCard
        :title="$t('codex.agents.pageTitle')"
        icon="Bot"
        :badge="String(filteredAgents.length)"
        tone="primary"
        class="mb-6"
      >
        <template #meta>
          <span class="rounded-full border border-accent-primary/25 bg-accent-primary/10 px-3 py-1 text-sm font-medium text-accent-primary">{{ contextLabel }}</span>
          <span class="rounded-full border border-border-default/60 bg-bg-surface/70 px-3 py-1 text-sm text-text-secondary">{{ activeContext?.agentsDir ?? '~/.codex/agents/' }}</span>
        </template>
        <template #actions>
          <div class="flex flex-wrap justify-end gap-2">
            <button
              type="button"
              class="codex-agent-secondary-button"
              @click="handleRefresh"
            >
              <SIcon
                name="RefreshCcw"
                size="w-4 h-4"
              /><span>{{ $t('common.refresh') }}</span>
            </button>
            <button
              v-if="activePanel === 'installed'"
              type="button"
              class="codex-agent-secondary-button"
              @click="handleChooseProject"
            >
              <SIcon
                name="FolderSearch"
                size="w-4 h-4"
              /><span>Choose Project</span>
            </button>
            <button
              v-if="activePanel === 'installed' && hasProjectShortcut && !isProjectMode"
              type="button"
              class="codex-agent-secondary-button"
              @click="handleSwitchToSavedProject"
            >
              <SIcon
                name="FolderGit2"
                size="w-4 h-4"
              /><span>Open Last Project</span>
            </button>
            <button
              v-if="activePanel === 'installed' && isProjectMode"
              type="button"
              class="codex-agent-secondary-button"
              @click="handleBackToGlobal"
            >
              <SIcon
                name="ArrowLeftRight"
                size="w-4 h-4"
              /><span>Back To Global</span>
            </button>
            <button
              v-if="activePanel === 'installed'"
              type="button"
              class="codex-agent-primary-button"
              @click="openCreateModal"
            >
              <SIcon
                name="Plus"
                size="w-4 h-4"
              /><span>{{ $t('codex.agents.addAgent') }}</span>
            </button>
          </div>
        </template>
        <div class="mb-4 flex flex-wrap gap-2">
          <button
            type="button"
            class="rounded-2xl border px-4 py-2 text-sm font-medium transition-colors"
            :class="activePanel === 'installed' ? 'border-accent-primary/40 bg-accent-primary/10 text-accent-primary' : 'border-border-default/60 bg-bg-surface/60 text-text-secondary hover:bg-bg-surface'"
            @click="activePanel = 'installed'"
          >
            Installed
          </button>
          <button
            type="button"
            class="rounded-2xl border px-4 py-2 text-sm font-medium transition-colors"
            :class="activePanel === 'sources' ? 'border-accent-primary/40 bg-accent-primary/10 text-accent-primary' : 'border-border-default/60 bg-bg-surface/60 text-text-secondary hover:bg-bg-surface'"
            @click="activePanel = 'sources'"
          >
            Sources
          </button>
        </div>

        <div
          v-if="activePanel === 'installed'"
          class="grid gap-4 xl:grid-cols-[minmax(0,1.7fr)_minmax(320px,1fr)]"
        >
          <div class="space-y-4">
            <div class="grid gap-4 md:grid-cols-3">
              <div class="codex-agent-summary-card">
                <div class="codex-agent-summary-label">
                  Active Scope
                </div><div class="codex-agent-summary-value">
                  {{ activeContext?.mode === 'project' ? 'Project' : 'Global' }}
                </div><div class="codex-agent-summary-note">
                  Only one management context is active at a time.
                </div>
              </div>
              <div class="codex-agent-summary-card">
                <div class="codex-agent-summary-label">
                  Agents
                </div><div class="codex-agent-summary-value">
                  {{ agents.length }}
                </div><div class="codex-agent-summary-note">
                  {{ diagnostics.length }} diagnostics tracked
                </div>
              </div>
              <div class="codex-agent-summary-card">
                <div class="codex-agent-summary-label">
                  Sessions
                </div><div class="codex-agent-summary-value">
                  {{ sessionsTotal ?? '—' }}
                </div><div class="codex-agent-summary-note">
                  Inventory from Codex dashboard overview.
                </div>
              </div>
            </div>

            <div class="rounded-3xl border border-border-default/20 bg-bg-card/70 p-5 shadow-xl shadow-black/10 backdrop-blur-xl">
              <div class="mb-4 flex flex-wrap items-center gap-3">
                <div class="relative min-w-[260px] flex-1">
                  <SIcon
                    name="Search"
                    size="w-4 h-4"
                    class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-text-muted"
                  />
                  <input
                    v-model="searchQuery"
                    type="text"
                    :placeholder="$t('codex.agents.searchPlaceholder')"
                    class="w-full rounded-2xl border border-border-default/60 bg-bg-surface/70 py-3 pl-10 pr-4 text-sm text-text-primary"
                  >
                </div>
                <div class="flex flex-wrap gap-2">
                  <button
                    type="button"
                    class="codex-agent-secondary-button"
                    :disabled="selectedAgents.length === 0"
                    @click="handleBulkValidate"
                  >
                    <SIcon
                      name="ShieldCheck"
                      size="w-4 h-4"
                    /><span>Validate</span>
                  </button>
                  <button
                    type="button"
                    class="codex-agent-secondary-button"
                    :disabled="selectedAgents.length === 0"
                    @click="handleExportSelected"
                  >
                    <SIcon
                      name="Download"
                      size="w-4 h-4"
                    /><span>Export</span>
                  </button>
                  <button
                    type="button"
                    class="codex-agent-secondary-button"
                    :disabled="!canCopySelection"
                    @click="openBulkCopyModal"
                  >
                    <SIcon
                      name="Copy"
                      size="w-4 h-4"
                    /><span>Copy</span>
                  </button>
                  <button
                    type="button"
                    class="codex-agent-secondary-button"
                    @click="triggerImport"
                  >
                    <SIcon
                      name="Upload"
                      size="w-4 h-4"
                    /><span>Import</span>
                  </button>
                  <button
                    type="button"
                    class="codex-agent-secondary-button"
                    :disabled="selectedAgents.length === 0"
                    @click="openBulkRenameModal"
                  >
                    <SIcon
                      name="TextField"
                      size="w-4 h-4"
                    /><span>Rename</span>
                  </button>
                  <button
                    type="button"
                    class="codex-agent-danger-button"
                    :disabled="selectedAgents.length === 0"
                    @click="handleBulkDelete"
                  >
                    <SIcon
                      name="Trash2"
                      size="w-4 h-4"
                    /><span>Delete</span>
                  </button>
                </div>
              </div>

              <div
                v-if="loading"
                class="py-20 text-center text-text-muted"
              >
                <div class="mx-auto mb-4 h-8 w-8 animate-spin rounded-full border-2 border-accent-primary/20 border-t-accent-primary" />{{ $t('common.loading') }}
              </div>
              <div
                v-else-if="filteredAgents.length === 0"
                class="rounded-3xl border border-dashed border-border-default/20 bg-bg-surface/35 px-6 py-16 text-center"
              >
                <div class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-bg-elevated/70">
                  <SIcon
                    name="Bot"
                    size="w-8 h-8"
                    class="text-text-muted"
                  />
                </div>
                <div class="text-lg font-semibold text-text-primary">
                  {{ searchQuery ? $t('codex.agents.noResults') : $t('codex.agents.emptyState') }}
                </div>
                <div class="mt-2 text-sm text-text-muted">
                  {{ searchQuery ? $t('codex.agents.noResultsHint') : $t('codex.agents.emptyHint') }}
                </div>
              </div>
              <div
                v-else
                class="space-y-3"
              >
                <label class="mb-2 flex items-center gap-3 text-sm text-text-secondary"><input
                  :checked="allVisibleSelected"
                  type="checkbox"
                  class="h-4 w-4 rounded border-border-default/70 bg-transparent"
                  @change="toggleVisibleSelection(($event.target as HTMLInputElement).checked)"
                ><span>Select visible agents</span></label>
                <article
                  v-for="agent in filteredAgents"
                  :key="agent.path"
                  class="rounded-3xl border border-border-default/15 bg-bg-surface/55 p-4 transition-colors hover:border-accent-primary/30 hover:bg-bg-surface/75"
                >
                  <div class="flex flex-wrap items-start gap-3">
                    <input
                      :checked="selectedNames.includes(agent.name)"
                      type="checkbox"
                      class="mt-1 h-4 w-4 rounded border-border-default/70 bg-transparent"
                      @change="toggleSelection(agent.name, ($event.target as HTMLInputElement).checked)"
                    >
                    <div class="min-w-0 flex-1">
                      <div class="flex flex-wrap items-center gap-2">
                        <h3 class="text-base font-semibold text-text-primary">
                          {{ agent.name }}
                        </h3>
                        <span class="rounded-full border border-border-default/60 bg-bg-elevated/70 px-2.5 py-1 text-xs text-text-secondary">{{ agent.fileName }}</span>
                        <span
                          v-if="agent.parseError"
                          class="rounded-full border border-amber-400/30 bg-amber-500/10 px-2.5 py-1 text-xs text-amber-100"
                        >Invalid TOML</span>
                        <span
                          v-if="agent.model"
                          class="rounded-full border border-sky-400/20 bg-sky-500/10 px-2.5 py-1 text-xs text-sky-100"
                        >{{ agent.model }}</span>
                      </div>
                      <div class="mt-2 text-sm text-text-secondary">
                        {{ agent.description || 'No description' }}
                      </div>
                      <div class="mt-3 flex flex-wrap gap-2 text-xs text-text-muted">
                        <span
                          v-if="agent.nicknameCandidates?.length"
                          class="rounded-full border border-border-default/50 px-2.5 py-1"
                        >{{ agent.nicknameCandidates.length }} nicknames</span>
                        <span
                          v-if="agent.sandboxMode"
                          class="rounded-full border border-border-default/50 px-2.5 py-1"
                        >{{ agent.sandboxMode }}</span>
                        <span class="rounded-full border border-border-default/50 px-2.5 py-1">{{ agent.path }}</span>
                      </div>
                    </div>
                    <div class="flex flex-wrap gap-2">
                      <button
                        type="button"
                        class="codex-agent-icon-button"
                        title="Edit"
                        @click="openEditModal(agent)"
                      >
                        <SIcon
                          name="Edit2"
                          size="w-4 h-4"
                        />
                      </button>
                      <button
                        type="button"
                        class="codex-agent-icon-button"
                        title="Rename"
                        @click="openRenameModal(agent)"
                      >
                        <SIcon
                          name="PencilLine"
                          size="w-4 h-4"
                        />
                      </button>
                      <button
                        type="button"
                        class="codex-agent-icon-button"
                        title="Copy"
                        :disabled="!alternateContextRequest"
                        @click="openCopyModal(agent)"
                      >
                        <SIcon
                          name="Copy"
                          size="w-4 h-4"
                        />
                      </button>
                      <button
                        type="button"
                        class="codex-agent-icon-button"
                        title="Validate"
                        @click="handleValidateAgent(agent)"
                      >
                        <SIcon
                          name="ShieldCheck"
                          size="w-4 h-4"
                        />
                      </button>
                      <button
                        type="button"
                        class="codex-agent-icon-button"
                        title="Export"
                        @click="exportAgent(agent)"
                      >
                        <SIcon
                          name="Download"
                          size="w-4 h-4"
                        />
                      </button>
                      <button
                        type="button"
                        class="codex-agent-icon-button danger"
                        title="Delete"
                        @click="handleDeleteAgent(agent)"
                      >
                        <SIcon
                          name="Trash2"
                          size="w-4 h-4"
                        />
                      </button>
                    </div>
                  </div>
                </article>
              </div>
            </div>
          </div>

          <div class="space-y-4">
            <Card
              variant="glass"
              class="p-5"
            >
              <div class="mb-3 flex items-center gap-2 text-sm font-semibold text-text-primary">
                <SIcon
                  name="FolderTree"
                  size="w-4 h-4"
                />Context Control
              </div>
              <div class="space-y-3 text-sm text-text-secondary">
                <div>
                  <div class="font-medium text-text-primary">
                    Current
                  </div><div class="mt-1 break-all">
                    {{ activeContext?.agentsDir ?? '~/.codex/agents/' }}
                  </div>
                </div>
                <div v-if="lastProjectRoot">
                  <div class="font-medium text-text-primary">
                    Last Project
                  </div><div class="mt-1 break-all">
                    {{ lastProjectRoot }}
                  </div>
                </div>
              </div>
            </Card>
            <Card
              variant="glass"
              class="p-5"
            >
              <div class="mb-3 flex items-center gap-2 text-sm font-semibold text-text-primary">
                <SIcon
                  name="Boxes"
                  size="w-4 h-4"
                />Built-in Agents
              </div>
              <div class="space-y-3">
                <div
                  v-for="builtIn in builtInCodexAgents"
                  :key="builtIn.name"
                  class="rounded-2xl border border-border-default/60 bg-bg-surface/55 px-3 py-3"
                >
                  <div class="flex items-center justify-between gap-3">
                    <div class="font-medium text-text-primary">
                      {{ builtIn.name }}
                    </div><span class="rounded-full border border-border-default/50 px-2 py-0.5 text-[11px] text-text-muted">Read-only</span>
                  </div>
                  <div class="mt-1 text-sm text-text-secondary">
                    {{ builtIn.description }}
                  </div>
                </div>
              </div>
            </Card>
            <Card
              variant="glass"
              class="p-5"
            >
              <div class="mb-3 flex items-center gap-2 text-sm font-semibold text-text-primary">
                <SIcon
                  name="AlertTriangle"
                  size="w-4 h-4"
                />Diagnostics
              </div>
              <div
                v-if="diagnostics.length === 0"
                class="text-sm text-text-secondary"
              >
                No context-level diagnostics.
              </div>
              <div
                v-else
                class="space-y-3"
              >
                <div
                  v-for="diagnostic in diagnostics"
                  :key="`${diagnostic.path}:${diagnostic.message}`"
                  class="rounded-2xl border border-amber-400/25 bg-amber-500/10 px-3 py-3 text-sm text-amber-100"
                >
                  <div class="font-medium">
                    {{ diagnostic.fileName }}
                  </div>
                  <div class="mt-1 break-words text-xs">
                    {{ diagnostic.message }}
                  </div>
                </div>
              </div>
            </Card>
          </div>
        </div>
        <CodexAgentSourcesPanel
          v-else
          @refresh-installed="handleRefresh"
        />
      </PageHeaderCard>
    </div>

    <input
      ref="fileInputRef"
      type="file"
      class="hidden"
      accept=".toml,.json"
      multiple
      @change="handleImportFiles"
    >
    <CodexAgentEditorModal
      v-model="editorOpen"
      :agent="editingAgent"
      :available-models="availableModels"
      @save="handleSaveAgent"
    />
    <BaseModal
      :model-value="renameModalOpen"
      title="Rename Agent"
      size="md"
      @update:model-value="renameModalOpen = $event"
    >
      <label class="space-y-2 text-sm text-text-secondary"><span class="font-semibold text-text-primary">New name</span><input
        v-model="renameDraft"
        type="text"
        class="codex-agent-input"
      ></label>
      <template #footer>
        <button
          type="button"
          class="codex-agent-secondary-button"
          @click="renameModalOpen = false"
        >
          Cancel
        </button><button
          type="button"
          class="codex-agent-primary-button"
          @click="handleRenameAgent"
        >
          Rename
        </button>
      </template>
    </BaseModal>
    <BaseModal
      :model-value="copyModalOpen"
      title="Copy Agent"
      size="md"
      @update:model-value="copyModalOpen = $event"
    >
      <div class="space-y-4">
        <div class="rounded-2xl border border-border-default/60 bg-bg-surface/60 px-4 py-3 text-sm text-text-secondary">
          Target context: <span class="font-semibold text-text-primary">{{ copyTargetLabel }}</span>
        </div>
        <label class="space-y-2 text-sm text-text-secondary"><span class="font-semibold text-text-primary">Target name</span><input
          v-model="copyDraftName"
          type="text"
          class="codex-agent-input"
          placeholder="Leave blank to keep current name"
        ></label>
      </div>
      <template #footer>
        <button
          type="button"
          class="codex-agent-secondary-button"
          @click="copyModalOpen = false"
        >
          Cancel
        </button><button
          type="button"
          class="codex-agent-primary-button"
          :disabled="!alternateContextRequest"
          @click="handleCopyAgent"
        >
          Copy
        </button>
      </template>
    </BaseModal>
    <BaseModal
      :model-value="bulkRenameModalOpen"
      title="Bulk Rename Agents"
      size="md"
      @update:model-value="bulkRenameModalOpen = $event"
    >
      <div class="space-y-4">
        <label class="space-y-2 text-sm text-text-secondary">
          <span class="font-semibold text-text-primary">Prefix</span>
          <input
            v-model="bulkRenamePrefix"
            type="text"
            class="codex-agent-input"
            placeholder="feature-"
          >
        </label>
        <label class="space-y-2 text-sm text-text-secondary">
          <span class="font-semibold text-text-primary">Suffix</span>
          <input
            v-model="bulkRenameSuffix"
            type="text"
            class="codex-agent-input"
            placeholder="-review"
          >
        </label>
      </div>
      <template #footer>
        <button
          type="button"
          class="codex-agent-secondary-button"
          @click="bulkRenameModalOpen = false"
        >
          Cancel
        </button>
        <button
          type="button"
          class="codex-agent-primary-button"
          :disabled="selectedAgents.length === 0"
          @click="handleBulkRename"
        >
          Rename
        </button>
      </template>
    </BaseModal>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'
import Card from '@/components/ui/Card.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import PageHeaderCard from '@/components/PageHeaderCard.vue'
import SIcon from '@/components/ui/SIcon.vue'
import CodexAgentEditorModal from '@/components/codex/CodexAgentEditorModal.vue'
import CodexAgentSourcesPanel from '@/components/codex/CodexAgentSourcesPanel.vue'
import { useCodexAgents } from '@/composables/useCodexAgents'
import { useUIStore } from '@/stores/ui'
import type { CodexAgentContextRequest, CodexAgentRecord, CodexAgentUpsertRequest } from '@/types'

const { t } = useI18n()
const uiStore = useUIStore()
const {
  activeContext,
  agents,
  availableModels,
  builtInCodexAgents,
  chooseProjectContext,
  contextLabel,
  copyAgentRecord,
  createAgent,
  currentContextRequest,
  deleteAgentRecord,
  diagnostics,
  hasProjectShortcut,
  isProjectMode,
  lastProjectRoot,
  loading,
  refreshAll,
  renameAgentRecord,
  sessionsTotal,
  switchToGlobalContext,
  switchToProjectContext,
  updateAgentRecord,
  validateAgentRecord,
} = useCodexAgents()
const activePanel = ref<'installed' | 'sources'>('installed')
const searchQuery = ref('')
const selectedNames = ref<string[]>([])
const editorOpen = ref(false)
const editingAgent = ref<CodexAgentRecord | null>(null)
const renameModalOpen = ref(false)
const renameTarget = ref<CodexAgentRecord | null>(null)
const renameDraft = ref('')
const copyModalOpen = ref(false)
const copyTarget = ref<CodexAgentRecord | null>(null)
const copyDraftName = ref('')
const bulkRenameModalOpen = ref(false)
const bulkRenamePrefix = ref('')
const bulkRenameSuffix = ref('')
const fileInputRef = ref<HTMLInputElement | null>(null)

const filteredAgents = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) {
    return agents.value
  }

  return agents.value.filter((agent) => {
    return [
      agent.name,
      agent.description ?? '',
      agent.developerInstructions ?? '',
      agent.model ?? '',
      agent.path,
    ].some((value) => value.toLowerCase().includes(query))
  })
})
const selectedAgents = computed(() => {
  return filteredAgents.value.filter((agent) => selectedNames.value.includes(agent.name))
})
const allVisibleSelected = computed(() => {
  return filteredAgents.value.length > 0
    && filteredAgents.value.every((agent) => selectedNames.value.includes(agent.name))
})
const alternateContextRequest = computed<CodexAgentContextRequest | null>(() => {
  if (isProjectMode.value) {
    return { mode: 'global' }
  }

  return lastProjectRoot.value
    ? { mode: 'project', projectRoot: lastProjectRoot.value }
    : null
})
const copyTargetLabel = computed(() => {
  if (isProjectMode.value) {
    return 'Global'
  }

  return lastProjectRoot.value
    ? `Project: ${lastProjectRoot.value}`
    : 'Choose project first'
})
const canCopySelection = computed(() => {
  return selectedAgents.value.length > 0 && !!alternateContextRequest.value
})

onMounted(async () => {
  await refreshAll()
})

function resetSelection() {
  selectedNames.value = []
}

function toggleSelection(name: string, checked: boolean) {
  if (checked) {
    selectedNames.value = Array.from(new Set([...selectedNames.value, name]))
    return
  }

  selectedNames.value = selectedNames.value.filter((value) => value !== name)
}

function toggleVisibleSelection(checked: boolean) {
  if (checked) {
    selectedNames.value = Array.from(
      new Set([...selectedNames.value, ...filteredAgents.value.map((agent) => agent.name)]),
    )
    return
  }

  const visible = new Set(filteredAgents.value.map((agent) => agent.name))
  selectedNames.value = selectedNames.value.filter((name) => !visible.has(name))
}

async function handleRefresh() {
  await refreshAll()
}

async function handleChooseProject() {
  try {
    if (await chooseProjectContext()) {
      uiStore.showSuccess('Project context activated')
      resetSelection()
    }
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleSwitchToSavedProject() {
  try {
    if (await switchToProjectContext()) {
      uiStore.showSuccess('Project context activated')
      resetSelection()
    }
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleBackToGlobal() {
  try {
    await switchToGlobalContext()
    uiStore.showSuccess('Returned to global agents')
    resetSelection()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

function openCreateModal() {
  editingAgent.value = null
  editorOpen.value = true
}

function openEditModal(agent: CodexAgentRecord) {
  editingAgent.value = agent
  editorOpen.value = true
}

async function handleSaveAgent(payload: CodexAgentUpsertRequest) {
  try {
    if (editingAgent.value) {
      await updateAgentRecord(editingAgent.value.name, payload)
      uiStore.showSuccess('Codex agent updated')
    } else {
      await createAgent(payload)
      uiStore.showSuccess('Codex agent created')
    }

    editorOpen.value = false
    editingAgent.value = null
    resetSelection()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

function openRenameModal(agent: CodexAgentRecord) {
  renameTarget.value = agent
  renameDraft.value = agent.name
  renameModalOpen.value = true
}

async function handleRenameAgent() {
  if (!renameTarget.value) {
    return
  }

  try {
    await renameAgentRecord(renameTarget.value.name, renameDraft.value.trim())
    uiStore.showSuccess('Agent renamed')
    renameModalOpen.value = false
    renameTarget.value = null
    resetSelection()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

function openCopyModal(agent: CodexAgentRecord) {
  copyTarget.value = agent
  copyDraftName.value = ''
  copyModalOpen.value = true
}

function openBulkRenameModal() {
  bulkRenamePrefix.value = ''
  bulkRenameSuffix.value = ''
  bulkRenameModalOpen.value = true
}

function openBulkCopyModal() {
  if (selectedAgents.value.length === 1) {
    openCopyModal(selectedAgents.value[0]!)
    return
  }

  copyTarget.value = null
  copyDraftName.value = ''
  copyModalOpen.value = true
}

async function handleCopyAgent() {
  if (!alternateContextRequest.value) {
    uiStore.showWarning('Choose a project context first')
    return
  }

  try {
    const queue = copyTarget.value ? [copyTarget.value] : selectedAgents.value
    for (const agent of queue) {
      await copyAgentRecord(agent.name, alternateContextRequest.value, copyDraftName.value.trim() || undefined)
    }

    uiStore.showSuccess(`Copied ${queue.length} agent${queue.length > 1 ? 's' : ''}`)
    copyModalOpen.value = false
    copyTarget.value = null
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleBulkRename() {
  if (!bulkRenamePrefix.value && !bulkRenameSuffix.value) {
    uiStore.showWarning('Provide a prefix, suffix, or both')
    return
  }

  try {
    for (const agent of selectedAgents.value) {
      await renameAgentRecord(
        agent.name,
        `${bulkRenamePrefix.value}${agent.name}${bulkRenameSuffix.value}`,
      )
    }
    bulkRenameModalOpen.value = false
    uiStore.showSuccess(`Renamed ${selectedAgents.value.length} agent${selectedAgents.value.length > 1 ? 's' : ''}`)
    resetSelection()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleDeleteAgent(agent: CodexAgentRecord) {
  const confirmed = await uiStore.requestConfirm({
    title: t('common.delete'),
    message: t('codex.agents.deleteConfirm', { name: agent.name }),
    confirmText: t('common.delete'),
    cancelText: t('common.cancel'),
    type: 'danger',
  })
  if (!confirmed) {
    return
  }

  try {
    await deleteAgentRecord(agent.name)
    uiStore.showSuccess('Agent deleted')
    resetSelection()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleBulkDelete() {
  if (selectedAgents.value.length === 0) {
    return
  }

  const confirmed = await uiStore.requestConfirm({
    title: 'Delete selected agents',
    message: `Delete ${selectedAgents.value.length} selected agents from the active context?`,
    confirmText: t('common.delete'),
    cancelText: t('common.cancel'),
    type: 'danger',
  })
  if (!confirmed) {
    return
  }

  try {
    for (const agent of selectedAgents.value) {
      await deleteAgentRecord(agent.name)
    }
    uiStore.showSuccess('Selected agents deleted')
    resetSelection()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleValidateAgent(agent: CodexAgentRecord) {
  try {
    await validateAgentRecord(agent.name)
    uiStore.showSuccess(`Validated ${agent.name}`)
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handleBulkValidate() {
  try {
    for (const agent of selectedAgents.value) {
      await validateAgentRecord(agent.name)
    }
    uiStore.showSuccess(`Validated ${selectedAgents.value.length} agent${selectedAgents.value.length > 1 ? 's' : ''}`)
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

function triggerImport() {
  fileInputRef.value?.click()
}

async function handleImportFiles(event: Event) {
  const input = event.target as HTMLInputElement | null
  const files = Array.from(input?.files ?? [])
  if (files.length === 0) {
    return
  }

  try {
    for (const file of files) {
      const text = await file.text()
      if (file.name.endsWith('.toml')) {
        await createAgent({ rawToml: text })
        continue
      }

      if (file.name.endsWith('.json')) {
        const parsed = JSON.parse(text)
        const entries = Array.isArray(parsed)
          ? parsed
          : Array.isArray(parsed.agents)
            ? parsed.agents
            : []

        for (const entry of entries) {
          const record = entry as { name?: string; rawToml?: string | null }
          await createAgent({
            name: record.name,
            rawToml: record.rawToml ?? '',
          })
        }
      }
    }

    uiStore.showSuccess('Import completed')
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  } finally {
    if (input) {
      input.value = ''
    }
  }
}

function downloadFile(name: string, content: string, mime = 'text/plain;charset=utf-8') {
  const blob = new Blob([content], { type: mime })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = name
  anchor.click()
  URL.revokeObjectURL(url)
}

function exportAgent(agent: CodexAgentRecord) {
  downloadFile(
    agent.fileName || `${agent.name}.toml`,
    agent.rawToml ?? '',
    'application/toml;charset=utf-8',
  )
}

function handleExportSelected() {
  if (selectedAgents.value.length === 1) {
    exportAgent(selectedAgents.value[0]!)
    return
  }

  const payload = selectedAgents.value.map((agent) => ({
    name: agent.name,
    path: agent.path,
    context: currentContextRequest.value,
    rawToml: agent.rawToml ?? '',
  }))

  downloadFile(
    `codex-agents-${activeContext.value?.mode ?? 'global'}.json`,
    JSON.stringify(payload, null, 2),
    'application/json;charset=utf-8',
  )
}
</script>

<style scoped>
.codex-agent-summary-card {
  border: 1px solid rgb(var(--color-border-default-rgb) / 50%);
  border-radius: 1.5rem;
  background: rgb(var(--color-bg-surface-rgb) / 58%);
  padding: 1rem 1.125rem;
}

.codex-agent-summary-label {
  color: var(--color-text-muted);
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.codex-agent-summary-value {
  margin-top: 0.4rem;
  color: var(--color-text-primary);
  font-size: 1.5rem;
  font-weight: 700;
}

.codex-agent-summary-note {
  margin-top: 0.45rem;
  color: var(--color-text-secondary);
  font-size: 0.875rem;
}

.codex-agent-primary-button,
.codex-agent-secondary-button,
.codex-agent-icon-button,
.codex-agent-danger-button {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  min-height: 2.75rem;
  border-radius: 0.9rem;
  padding: 0.65rem 0.95rem;
  font-size: 0.875rem;
  transition:
    transform 150ms ease,
    background-color 150ms ease,
    border-color 150ms ease,
    color 150ms ease;
}

.codex-agent-primary-button {
  background: var(--color-accent-primary);
  color: white;
}

.codex-agent-primary-button:hover {
  transform: scale(1.02);
}

.codex-agent-secondary-button,
.codex-agent-icon-button {
  border: 1px solid rgb(var(--color-border-default-rgb) / 65%);
  background: rgb(var(--color-bg-surface-rgb) / 72%);
  color: var(--color-text-secondary);
}

.codex-agent-danger-button {
  border: 1px solid rgb(244 63 94 / 30%);
  background: rgb(244 63 94 / 12%);
  color: rgb(254 205 211);
}

.codex-agent-secondary-button:hover,
.codex-agent-icon-button:hover {
  background: rgb(var(--color-bg-surface-rgb) / 95%);
  color: var(--color-text-primary);
}

.codex-agent-danger-button:hover {
  background: rgb(244 63 94 / 20%);
}

.codex-agent-icon-button {
  justify-content: center;
  min-width: 2.75rem;
  padding-inline: 0.75rem;
}

.codex-agent-icon-button.danger:hover {
  border-color: rgb(244 63 94 / 40%);
  color: rgb(254 205 211);
  background: rgb(244 63 94 / 14%);
}

.codex-agent-secondary-button:disabled,
.codex-agent-icon-button:disabled,
.codex-agent-primary-button:disabled,
.codex-agent-danger-button:disabled {
  cursor: not-allowed;
  opacity: 0.45;
  transform: none;
}

.codex-agent-input {
  width: 100%;
  min-height: 2.75rem;
  border-radius: 0.875rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 65%);
  background: rgb(var(--color-bg-surface-rgb) / 72%);
  padding: 0.75rem 0.875rem;
  color: var(--color-text-primary);
}
</style>

