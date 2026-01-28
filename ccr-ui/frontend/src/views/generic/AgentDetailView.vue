<template>
  <div class="min-h-screen p-5 transition-colors duration-300">
    <div class="max-w-[1200px] mx-auto">
      <!-- Breadcrumb -->
      <Breadcrumb :items="breadcrumbs" />

      <!-- Loading State -->
      <div
        v-if="loading"
        class="text-center py-20 text-guofeng-text-muted"
      >
        <div class="loading-spinner mx-auto mb-4 w-8 h-8 border-guofeng-jade/30 border-t-guofeng-jade" />
        {{ $t('common.loading') }}
      </div>

      <!-- Error State -->
      <div
        v-else-if="error"
        class="text-center py-20"
      >
        <div class="bg-guofeng-red/10 w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
          <AlertCircle class="w-10 h-10 text-guofeng-red" />
        </div>
        <p class="text-lg font-medium text-guofeng-text-primary">
          {{ $t('agents.loadError') }}
        </p>
        <p class="text-sm mt-2 text-guofeng-text-muted">
          {{ error }}
        </p>
        <RouterLink
          to="/agents"
          class="mt-4 inline-flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium bg-guofeng-bg-secondary hover:bg-guofeng-bg-tertiary transition-colors"
        >
          <ArrowLeft class="w-4 h-4" />
          {{ $t('common.back') }}
        </RouterLink>
      </div>

      <!-- Agent Detail -->
      <div v-else-if="agent">
        <!-- Header -->
        <div class="glass-effect rounded-2xl p-6 mb-6 border border-white/20 shadow-sm">
          <div class="flex items-start justify-between gap-4">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-3 mb-2">
                <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-guofeng-jade/10 to-guofeng-blue/10 flex items-center justify-center text-xl shadow-sm border border-white/20">
                  🤖
                </div>
                <div>
                  <h1 class="text-2xl font-bold text-guofeng-text-primary">
                    {{ agent.name }}
                  </h1>
                  <div class="flex items-center gap-2 mt-1">
                    <span
                      v-if="agent.folder"
                      class="flex items-center gap-1 text-xs text-guofeng-text-muted bg-guofeng-bg-tertiary px-2 py-0.5 rounded border border-guofeng-border/50"
                    >
                      <Folder class="w-3 h-3" /> {{ agent.folder }}
                    </span>
                    <span
                      v-if="agent.model"
                      class="text-xs text-guofeng-indigo bg-guofeng-indigo/10 px-2 py-0.5 rounded"
                    >
                      {{ agent.model }}
                    </span>
                    <span
                      :class="agent.disabled ? 'bg-guofeng-text-muted/20 text-guofeng-text-muted' : 'bg-guofeng-jade/10 text-guofeng-jade'"
                      class="text-xs px-2 py-0.5 rounded font-medium"
                    >
                      {{ agent.disabled ? $t('agents.disabledBadge') : $t('agents.enabledBadge') }}
                    </span>
                  </div>
                </div>
              </div>
            </div>

            <div class="flex items-center gap-2">
              <button
                class="px-4 py-2 rounded-lg font-medium text-sm transition-all flex items-center gap-2"
                :class="agent.disabled ? 'bg-guofeng-jade/10 text-guofeng-jade hover:bg-guofeng-jade/20' : 'bg-guofeng-text-muted/10 text-guofeng-text-muted hover:bg-guofeng-text-muted/20'"
                @click="handleToggle"
              >
                <Power
                  v-if="!agent.disabled"
                  class="w-4 h-4"
                />
                <PowerOff
                  v-else
                  class="w-4 h-4"
                />
                {{ agent.disabled ? $t('agents.enable') : $t('agents.disable') }}
              </button>
              <button
                class="px-4 py-2 rounded-lg font-medium text-sm transition-all bg-guofeng-blue/10 text-guofeng-blue hover:bg-guofeng-blue/20 flex items-center gap-2"
                @click="handleEdit"
              >
                <Edit2 class="w-4 h-4" />
                {{ $t('common.edit') }}
              </button>
              <button
                class="px-4 py-2 rounded-lg font-medium text-sm transition-all bg-guofeng-red/10 text-guofeng-red hover:bg-guofeng-red/20 flex items-center gap-2"
                @click="handleDelete"
              >
                <Trash2 class="w-4 h-4" />
                {{ $t('common.delete') }}
              </button>
            </div>
          </div>
        </div>

        <!-- Tools Section -->
        <div
          v-if="agent.tools && agent.tools.length > 0"
          class="glass-effect rounded-2xl p-6 mb-6 border border-white/20 shadow-sm"
        >
          <h2 class="text-lg font-bold text-guofeng-text-primary flex items-center gap-2 mb-4">
            <Wrench class="w-5 h-5 text-guofeng-jade" />
            {{ $t('agents.toolsLabel') }}
            <span class="text-sm font-normal text-guofeng-text-muted">({{ agent.tools.length }})</span>
          </h2>
          <div class="flex flex-wrap gap-2">
            <span
              v-for="tool in agent.tools"
              :key="tool"
              class="px-3 py-1.5 rounded-lg text-sm bg-guofeng-bg-tertiary border border-guofeng-border/50 text-guofeng-text-primary"
            >
              {{ tool }}
            </span>
          </div>
        </div>

        <!-- System Prompt Section -->
        <div class="glass-effect rounded-2xl p-6 border border-white/20 shadow-sm">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-lg font-bold text-guofeng-text-primary flex items-center gap-2">
              <FileText class="w-5 h-5 text-guofeng-jade" />
              {{ $t('agents.systemPromptLabel') }}
            </h2>
            <button
              v-if="agent.system_prompt"
              class="px-3 py-1.5 rounded-lg text-xs font-medium transition-all bg-guofeng-bg-tertiary hover:bg-guofeng-bg-secondary text-guofeng-text-secondary flex items-center gap-1.5"
              @click="copySystemPrompt"
            >
              <Copy class="w-3.5 h-3.5" />
              {{ copied ? $t('common.copied') : $t('common.copy') }}
            </button>
          </div>

          <div
            v-if="agent.system_prompt"
            class="relative"
          >
            <pre class="bg-guofeng-bg-tertiary/50 rounded-xl p-4 overflow-auto max-h-[600px] border border-guofeng-border/30">
              <code class="text-sm font-mono text-guofeng-text-primary whitespace-pre-wrap break-words leading-relaxed">{{ agent.system_prompt }}</code>
            </pre>
          </div>
          <div
            v-else
            class="text-center py-12 text-guofeng-text-muted"
          >
            <FileText class="w-12 h-12 mx-auto mb-3 opacity-30" />
            <p>{{ $t('agents.noSystemPrompt') }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- Edit Modal -->
    <div
      v-if="showEditModal"
      class="fixed inset-0 flex items-center justify-center z-50 bg-guofeng-ink/20 backdrop-blur-sm transition-all p-4"
      @click="showEditModal = false"
    >
      <div
        class="glass-effect p-8 rounded-3xl w-full max-w-2xl max-h-[85vh] overflow-y-auto shadow-2xl border border-white/30 relative"
        @click.stop
      >
        <button
          class="absolute top-4 right-4 p-2 rounded-full hover:bg-guofeng-bg-tertiary text-guofeng-text-muted transition-colors"
          @click="showEditModal = false"
        >
          <X class="w-5 h-5" />
        </button>

        <h3 class="text-2xl font-bold mb-8 text-guofeng-text-primary flex items-center">
          <div class="w-10 h-10 rounded-xl bg-guofeng-jade/10 flex items-center justify-center mr-3 text-guofeng-jade">
            <Edit2 class="w-5 h-5" />
          </div>
          {{ $t('agents.editAgent') }}
        </h3>

        <div class="space-y-6">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <label class="block mb-2 text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider">{{ $t('agents.nameLabel') }}</label>
              <input
                :value="agent?.name"
                type="text"
                disabled
                class="w-full px-4 py-3 rounded-xl bg-guofeng-bg-tertiary/50 border border-guofeng-border opacity-60 cursor-not-allowed"
              >
            </div>

            <div>
              <label class="block mb-2 text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider">{{ $t('agents.modelLabel') }}</label>
              <div class="relative">
                <select
                  v-model="formData.model"
                  class="w-full px-4 py-3 rounded-xl bg-white/50 border border-guofeng-border focus:border-guofeng-jade focus:ring-4 focus:ring-guofeng-jade/10 outline-none transition-all appearance-none"
                >
                  <option value="claude-sonnet-4-5-20250929">
                    Claude Sonnet 4.5
                  </option>
                  <option value="claude-opus-4-20250514">
                    Claude Opus 4
                  </option>
                  <option value="claude-3-5-sonnet-20241022">
                    Claude 3.5 Sonnet
                  </option>
                </select>
                <div class="absolute right-4 top-1/2 -translate-y-1/2 pointer-events-none text-guofeng-text-muted">
                  <ChevronDown class="w-4 h-4" />
                </div>
              </div>
            </div>
          </div>

          <div>
            <label class="block mb-2 text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider">{{ $t('agents.toolsLabel') }}</label>
            <div class="flex gap-2 mb-3">
              <input
                v-model="toolInput"
                type="text"
                :placeholder="$t('agents.toolPlaceholder')"
                class="flex-1 px-4 py-3 rounded-xl bg-white/50 border border-guofeng-border focus:border-guofeng-jade focus:ring-4 focus:ring-guofeng-jade/10 outline-none transition-all"
                @keyup.enter="addTool"
              >
              <button
                class="px-6 py-3 rounded-xl font-bold text-white bg-guofeng-jade hover:bg-guofeng-jade/90 transition-colors shadow-lg shadow-guofeng-jade/20"
                @click="addTool"
              >
                {{ $t('agents.addTool') }}
              </button>
            </div>
            <div class="flex flex-wrap gap-2 min-h-[50px] p-4 rounded-xl bg-guofeng-bg-secondary/50 border border-guofeng-border/50 border-dashed">
              <span
                v-if="!formData.tools || formData.tools.length === 0"
                class="text-sm text-guofeng-text-muted italic w-full text-center py-2"
              >{{ $t('agents.noTools') }}</span>
              <span
                v-for="tool in (formData.tools || [])"
                :key="tool"
                class="px-3 py-1.5 rounded-lg text-sm flex items-center gap-2 bg-white border border-guofeng-border shadow-sm text-guofeng-text-primary group"
              >
                {{ tool }}
                <button
                  class="text-guofeng-text-muted group-hover:text-guofeng-red transition-colors"
                  @click="removeTool(tool)"
                ><X class="w-3.5 h-3.5" /></button>
              </span>
            </div>
          </div>

          <div>
            <label class="block mb-2 text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider">{{ $t('agents.systemPromptLabel') }}</label>
            <textarea
              v-model="formData.system_prompt"
              rows="8"
              class="w-full px-4 py-3 rounded-xl bg-white/50 border border-guofeng-border focus:border-guofeng-jade focus:ring-4 focus:ring-guofeng-jade/10 outline-none transition-all resize-y font-mono text-sm leading-relaxed"
              :placeholder="$t('agents.systemPromptPlaceholder')"
            />
          </div>
        </div>

        <div class="flex gap-4 mt-10 pt-6 border-t border-guofeng-border/50">
          <button
            class="flex-1 px-6 py-3.5 rounded-xl font-bold transition-all bg-white text-guofeng-text-secondary hover:bg-guofeng-bg-tertiary border border-guofeng-border"
            @click="showEditModal = false"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            class="flex-1 px-6 py-3.5 rounded-xl font-bold transition-all bg-guofeng-jade text-white shadow-lg shadow-guofeng-jade/20 hover:shadow-xl hover:shadow-guofeng-jade/30 hover:-translate-y-0.5"
            :disabled="saving"
            @click="handleSave"
          >
            {{ saving ? $t('common.saving') : $t('common.save') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter, RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  Edit2, Trash2, ArrowLeft, Folder, FileText, Copy, X,
  AlertCircle, Home, Bot, Power, PowerOff, Wrench, ChevronDown
} from 'lucide-vue-next'
import Breadcrumb from '@/components/common/Breadcrumb.vue'
import { useAgents } from '@/composables/useAgents'
import type { Agent, AgentRequest } from '@/types'
import { extractStringParam } from '@/types/router'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()

// 使用 agents module (Claude Code)
const { getAgent, updateAgent, deleteAgent, toggleAgent, loading } = useAgents('agents')

const agent = ref<Agent | null>(null)
const error = ref<string | null>(null)
const showEditModal = ref(false)
const formData = ref<AgentRequest>({ name: '', model: '', tools: [], system_prompt: '', disabled: false })
const toolInput = ref('')
const saving = ref(false)
const copied = ref(false)

const breadcrumbs = computed(() => [
  { label: t('common.home'), to: '/', icon: Home },
  { label: t('agents.pageTitle'), to: '/agents', icon: Bot },
  { label: agent.value?.name || t('common.loading') }
])

onMounted(async () => {
  const name = extractStringParam(route.params.name)
  if (name) {
    try {
      agent.value = await getAgent(name)
    } catch (err: any) {
      console.error('Failed to load agent:', err)
      error.value = err.message || 'Failed to load agent'
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
    console.error('Failed to update agent:', err)
    alert(t('common.operationFailed'))
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
    console.error('Failed to toggle agent:', err)
    alert(t('common.operationFailed'))
  }
}

const handleDelete = async () => {
  if (!agent.value) return

  if (!confirm(t('agents.deleteConfirm', { name: agent.value.name }))) return

  try {
    await deleteAgent(agent.value.name)
    router.push('/agents')
  } catch (err) {
    console.error('Failed to delete agent:', err)
    alert(t('common.deleteFailed'))
  }
}

const copySystemPrompt = async () => {
  if (!agent.value?.system_prompt) return

  try {
    await navigator.clipboard.writeText(agent.value.system_prompt)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}
</script>
