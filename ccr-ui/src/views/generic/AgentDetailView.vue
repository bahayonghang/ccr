<!-- -->
<template>
  <PageShell class="agent-detail-view">
    <template #header>
      <PageHeader
        :title="agent?.name || $t('common.loading')"
        :description="agent?.folder || undefined"
      >
        <template #status>
          <span
            v-if="agent?.model"
            class="inline-flex items-center rounded-md border border-border-default bg-bg-surface px-3 py-1 text-xs text-text-secondary"
          >
            {{ agent.model }}
          </span>
          <span
            v-if="agent"
            class="inline-flex items-center rounded-md border px-3 py-1 text-xs font-medium"
            :class="agent.disabled ? 'border-border-default bg-bg-surface text-text-muted' : 'border-border-default bg-bg-elevated text-text-secondary'"
          >
            {{ agent.disabled ? $t('agents.disabledBadge') : $t('agents.enabledBadge') }}
          </span>
        </template>
        <template #actions>
          <RouterLink
            to="/agents"
            class="inline-flex min-h-[44px] items-center gap-2 rounded-lg border border-border-default bg-bg-elevated px-4 py-2 text-sm font-medium text-text-secondary"
          >
            <SIcon
              name="ArrowLeft"
              size="w-4 h-4"
            />
            {{ $t('common.back') }}
          </RouterLink>
          <button
            v-if="agent"
            class="px-4 py-2 rounded-lg font-medium text-sm transition-colors flex items-center gap-2"
            :class="agent.disabled ? 'bg-bg-elevated text-text-secondary' : 'bg-bg-surface text-text-muted'"
            @click="handleToggle"
          >
            <SIcon
              :name="agent.disabled ? 'PowerOff' : 'Power'"
              size="w-4 h-4"
            />
            {{ agent.disabled ? $t('agents.enable') : $t('agents.disable') }}
          </button>
          <button
            v-if="agent"
            class="px-4 py-2 rounded-lg font-medium text-sm transition-colors bg-bg-elevated text-text-secondary flex items-center gap-2"
            @click="handleEdit"
          >
            <SIcon
              name="Edit2"
              size="w-4 h-4"
            />
            {{ $t('common.edit') }}
          </button>
          <button
            v-if="agent"
            class="px-4 py-2 rounded-lg font-medium text-sm transition-colors bg-accent-danger/10 text-accent-danger flex items-center gap-2"
            @click="handleDelete"
          >
            <SIcon
              name="Trash2"
              size="w-4 h-4"
            />
            {{ $t('common.delete') }}
          </button>
        </template>
      </PageHeader>
    </template>

    <template #subnav>
      <ModuleSubnav module="claude-code" />
    </template>

    <!-- Loading State -->
    <div
      v-if="loading"
      class="text-center py-20 text-text-muted"
    >
      <div class="loading-spinner mx-auto mb-4 w-8 h-8 border-accent-secondary/30 border-t-accent-secondary" />
      {{ $t('common.loading') }}
    </div>

    <!-- Error State -->
    <div
      v-else-if="error"
      class="text-center py-20"
    >
      <div class="bg-accent-danger/10 w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
        <SIcon
          name="AlertCircle"
          size="w-10 h-10"
          class="text-accent-danger"
        />
      </div>
      <p class="text-lg font-medium text-text-primary">
        {{ $t('agents.loadError') }}
      </p>
      <p class="text-sm mt-2 text-text-muted">
        {{ error }}
      </p>
      <RouterLink
        to="/agents"
        class="mt-4 inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium bg-bg-elevated hover:bg-bg-surface transition-colors"
      >
        <SIcon
          name="ArrowLeft"
          size="w-4 h-4"
        />
        {{ $t('common.back') }}
      </RouterLink>
    </div>

    <!-- Agent Detail -->
    <div v-else-if="agent">
      <!-- Tools Section -->
      <div
        v-if="agent.tools && agent.tools.length > 0"
        class="rounded-xl p-6 mb-6 border border-border-default/25 bg-bg-surface"
      >
        <h2 class="text-lg font-bold text-text-primary flex items-center gap-2 mb-4">
          <SIcon
            name="Wrench"
            size="w-5 h-5"
            class="text-accent-secondary"
          />
          {{ $t('agents.toolsLabel') }}
          <span class="text-sm font-normal text-text-muted">({{ agent.tools.length }})</span>
        </h2>
        <div class="flex flex-wrap gap-2">
          <span
            v-for="tool in agent.tools"
            :key="tool"
            class="px-3 py-1.5 rounded-lg text-sm bg-bg-surface border border-border-default/50 text-text-primary"
          >
            {{ tool }}
          </span>
        </div>
      </div>

      <!-- System Prompt Section -->
      <div class="rounded-xl p-6 border border-border-default/25 bg-bg-surface">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-bold text-text-primary flex items-center gap-2">
            <SIcon
              name="FileText"
              size="w-5 h-5"
              class="text-accent-secondary"
            />
            {{ $t('agents.systemPromptLabel') }}
          </h2>
          <button
            v-if="agent.system_prompt"
            class="px-3 py-1.5 rounded-lg text-xs font-medium transition-colors bg-bg-surface hover:bg-bg-elevated text-text-secondary flex items-center gap-1.5"
            @click="copySystemPrompt"
          >
            <SIcon
              name="Copy"
              size="w-3.5 h-3.5"
            />
            {{ copied ? $t('common.copied') : $t('common.copy') }}
          </button>
        </div>

        <div
          v-if="agent.system_prompt"
          class="relative"
        >
          <pre class="bg-bg-elevated rounded-xl p-4 overflow-auto max-h-[600px] border border-border-default/30">
              <code class="text-sm font-mono text-text-primary whitespace-pre-wrap break-words leading-relaxed">{{ agent.system_prompt }}</code>
            </pre>
        </div>
        <div
          v-else
          class="text-center py-12 text-text-muted"
        >
          <SIcon
            name="FileText"
            size="w-12 h-12"
            class="mx-auto mb-3 opacity-30"
          />
          <p>{{ $t('agents.noSystemPrompt') }}</p>
        </div>
      </div>
    </div>

    <!-- Edit Modal -->
    <div
      v-if="showEditModal"
      class="fixed inset-0 flex items-center justify-center z-50 bg-black/20 backdrop-blur-md transition-colors p-4"
      @click="showEditModal = false"
    >
      <div
        class="bg-bg-surface p-8 rounded-xl w-full max-w-2xl max-h-[85vh] overflow-y-auto border border-border-default/30 relative"
        @click.stop
      >
        <button
          class="absolute top-4 right-4 p-2 rounded-full hover:bg-bg-surface text-text-muted transition-colors"
          @click="showEditModal = false"
        >
          <SIcon
            name="X"
            size="w-5 h-5"
          />
        </button>

        <h3 class="text-2xl font-bold mb-8 text-text-primary flex items-center">
          <div class="w-10 h-10 rounded-xl bg-accent-secondary/10 flex items-center justify-center mr-3 text-accent-secondary">
            <SIcon
              name="Edit2"
              size="w-5 h-5"
            />
          </div>
          {{ $t('agents.editAgent') }}
        </h3>

        <div class="space-y-6">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <label class="block mb-2 text-xs font-medium text-text-secondary">{{ $t('agents.nameLabel') }}</label>
              <input
                :value="agent?.name"
                type="text"
                disabled
                class="w-full px-4 py-3 rounded-xl bg-bg-elevated border border-border-default opacity-60 cursor-not-allowed"
              >
            </div>

            <div>
              <label class="block mb-2 text-xs font-medium text-text-secondary">{{ $t('agents.modelLabel') }}</label>
              <div class="relative">
                <select
                  v-model="formData.model"
                  class="w-full px-4 py-3 rounded-xl bg-bg-surface border border-border-default focus:border-accent-secondary focus:ring-4 focus:ring-accent-secondary/10 outline-none transition-colors appearance-none"
                >
                  <option
                    v-for="option in defaultAgentModelOptions"
                    :key="option.value"
                    :value="option.value"
                  >
                    {{ option.label }}
                  </option>
                </select>
                <div class="absolute right-4 top-1/2 -translate-y-1/2 pointer-events-none text-text-muted">
                  <SIcon
                    name="ChevronDown"
                    size="w-4 h-4"
                  />
                </div>
              </div>
            </div>
          </div>

          <div>
            <label class="block mb-2 text-xs font-medium text-text-secondary">{{ $t('agents.toolsLabel') }}</label>
            <div class="flex gap-2 mb-3">
              <input
                v-model="toolInput"
                type="text"
                :placeholder="$t('agents.toolPlaceholder')"
                class="flex-1 px-4 py-3 rounded-xl bg-bg-surface border border-border-default focus:border-accent-secondary focus:ring-4 focus:ring-accent-secondary/10 outline-none transition-colors"
                @keyup.enter="addTool"
              >
              <button
                class="px-6 py-3 rounded-lg font-medium text-[color:var(--color-accent-primary-contrast)] bg-accent-secondary hover:bg-accent-secondary/90 transition-colors"
                @click="addTool"
              >
                {{ $t('agents.addTool') }}
              </button>
            </div>
            <div class="flex flex-wrap gap-2 min-h-[50px] p-4 rounded-xl bg-bg-base border border-border-default/50 border-dashed">
              <span
                v-if="!formData.tools || formData.tools.length === 0"
                class="text-sm text-text-muted italic w-full text-center py-2"
              >{{ $t('agents.noTools') }}</span>
              <span
                v-for="tool in (formData.tools || [])"
                :key="tool"
                class="px-3 py-1.5 rounded-lg text-sm flex items-center gap-2 bg-bg-elevated border border-border-default shadow-sm text-text-primary group"
              >
                {{ tool }}
                <button
                  class="text-text-muted group-hover:text-accent-danger transition-colors"
                  @click="removeTool(tool)"
                ><SIcon
                  name="X"
                  size="w-3.5 h-3.5"
                /></button>
              </span>
            </div>
          </div>

          <div>
            <label class="block mb-2 text-xs font-medium text-text-secondary">{{ $t('agents.systemPromptLabel') }}</label>
            <textarea
              v-model="formData.system_prompt"
              rows="8"
              class="w-full px-4 py-3 rounded-xl bg-bg-elevated border border-border-default focus:border-accent-secondary focus:ring-4 focus:ring-accent-secondary/10 outline-none transition-colors resize-y font-mono text-sm leading-relaxed"
              :placeholder="$t('agents.systemPromptPlaceholder')"
            />
          </div>
        </div>

        <div class="flex gap-4 mt-10 pt-6 border-t border-border-default/50">
          <button
            class="flex-1 px-6 py-3.5 rounded-xl font-bold transition-colors bg-bg-elevated text-text-secondary hover:bg-bg-surface border border-border-default"
            @click="showEditModal = false"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            class="flex-1 px-6 py-3.5 rounded-lg font-medium transition-colors bg-accent-secondary text-[color:var(--color-accent-primary-contrast)]"
            :disabled="saving"
            @click="handleSave"
          >
            {{ saving ? $t('common.saving') : $t('common.save') }}
          </button>
        </div>
      </div>
    </div>
  </PageShell>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted } from 'vue'
import { useRoute, useRouter, RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import { useAgents } from '@/composables/useAgents'
import type { Agent, AgentRequest } from '@/types'
import { extractStringParam } from '@/types/router'
import { getErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'
import { copyText } from '@/utils/clipboard'
import { useUIStore } from '@/stores/ui'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const uiStore = useUIStore()
const defaultAgentModelOptions = [
  { value: 'claude-sonnet-4-5-20250929', label: 'Claude Sonnet 4.5' },
  { value: 'claude-opus-4-20250514', label: 'Claude Opus 4' },
  { value: 'claude-3-5-sonnet-20241022', label: 'Claude 3.5 Sonnet' },
]

// 使用 agents module (Claude Code)
const { getAgent, updateAgent, deleteAgent, toggleAgent, loading } = useAgents('agents')

const agent = ref<Agent | null>(null)
const error = ref<string | null>(null)
const showEditModal = ref(false)
const formData = ref<AgentRequest>({ name: '', model: '', tools: [], system_prompt: '', disabled: false })
const toolInput = ref('')
const saving = ref(false)
const copied = ref(false)

onMounted(async () => {
  const name = extractStringParam(route.params.name)
  if (name) {
    try {
      agent.value = await getAgent(name)
    } catch (err: unknown) {
      logger.error('Failed to load agent:', err)
      error.value = getErrorMessage(err) || 'Failed to load agent'
    }
  } else {
    error.value = 'Invalid agent name parameter'
  }
})

const handleEdit = () => {
  if (agent.value) {
    formData.value = {
      name: agent.value.name,
      model: agent.value.model || 'claude-sonnet-4-5-20250929',
      tools: [...(agent.value.tools || [])],
      system_prompt: agent.value.system_prompt || '',
      disabled: agent.value.disabled || false
    }
    toolInput.value = ''
    showEditModal.value = true
  }
}

const addTool = () => {
  if (!formData.value.tools) {
    formData.value.tools = []
  }
  if (toolInput.value.trim() && !formData.value.tools.includes(toolInput.value.trim())) {
    formData.value.tools.push(toolInput.value.trim())
    toolInput.value = ''
  }
}

const removeTool = (tool: string) => {
  if (formData.value.tools) {
    formData.value.tools = formData.value.tools.filter(t => t !== tool)
  }
}

const handleSave = async () => {
  if (!agent.value) return

  saving.value = true
  try {
    const request: AgentRequest = {
      ...formData.value,
      tools: (formData.value.tools && formData.value.tools.length > 0) ? formData.value.tools : undefined,
      system_prompt: formData.value.system_prompt || undefined
    }
    await updateAgent(agent.value.name, request)

    // Update local state
    agent.value = {
      ...agent.value,
      model: formData.value.model,
      tools: formData.value.tools || [],
      system_prompt: formData.value.system_prompt
    }
    showEditModal.value = false
  } catch (err) {
    logger.error('Failed to update agent:', err)
    uiStore.showError(t('common.operationFailed'))
  } finally {
    saving.value = false
  }
}

const handleToggle = async () => {
  if (!agent.value) return

  try {
    await toggleAgent(agent.value.name)
    agent.value.disabled = !agent.value.disabled
  } catch (err) {
    logger.error('Failed to toggle agent:', err)
    uiStore.showError(t('common.operationFailed'))
  }
}

const handleDelete = async () => {
  if (!agent.value) return

  const confirmed = await uiStore.requestConfirm({
    title: t('common.delete'),
    message: t('agents.deleteConfirm', { name: agent.value.name }),
    confirmText: t('common.delete'),
    cancelText: t('common.cancel'),
    type: 'danger'
  })
  if (!confirmed) return

  try {
    await deleteAgent(agent.value.name)
    router.push('/agents')
  } catch (err) {
    logger.error('Failed to delete agent:', err)
    uiStore.showError(t('common.deleteFailed'))
  }
}

const copySystemPrompt = async () => {
  if (!agent.value?.system_prompt) return

  try {
    if (!(await copyText(agent.value.system_prompt))) throw new Error('clipboard copy failed')
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (err) {
    logger.error('Failed to copy:', err)
  }
}
</script>
