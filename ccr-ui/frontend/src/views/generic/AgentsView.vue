<template>
  <div class="min-h-screen p-6 transition-colors duration-300">
    <div class="max-w-[1800px] mx-auto">
      <!-- Breadcrumbs -->
      <Breadcrumb :items="breadcrumbs" />

      <div class="flex gap-6 items-start">
        <!-- Left Sidebar (Folders) -->
        <div class="w-64 flex-shrink-0 space-y-4 hidden lg:block sticky top-6">
          <div class="glass-effect rounded-2xl p-4 border border-white/20 shadow-sm">
            <h3 class="text-xs font-bold text-guofeng-text-muted uppercase tracking-wider mb-3 px-2 flex items-center justify-between">
              {{ $t(`${tPrefix}.folders.label`) }}
              <span class="bg-guofeng-bg-tertiary px-1.5 py-0.5 rounded text-[10px]">{{ stats.total }}</span>
            </h3>
             
            <div class="space-y-1">
              <div
                v-for="folder in folderOptions"
                :key="folder.value"
                class="flex items-center gap-2 px-3 py-2 rounded-xl cursor-pointer text-sm transition-all duration-200 group"
                :class="[
                  selectedFolder === folder.value 
                    ? 'bg-guofeng-jade/10 text-guofeng-jade font-medium shadow-sm border border-guofeng-jade/20' 
                    : 'text-guofeng-text-secondary hover:bg-guofeng-bg-tertiary hover:text-guofeng-text-primary'
                ]"
                @click="selectedFolder = folder.value"
              >
                <component 
                  :is="folder.icon" 
                  class="w-4 h-4 transition-transform group-hover:scale-110" 
                  :class="selectedFolder === folder.value ? 'text-guofeng-jade' : 'text-guofeng-text-muted'"
                />
                <span class="flex-1 truncate">{{ folder.label }}</span>
                <span 
                  class="text-xs px-1.5 py-0.5 rounded-md transition-colors"
                  :class="selectedFolder === folder.value ? 'bg-guofeng-jade/20 text-guofeng-jade' : 'bg-guofeng-bg-tertiary text-guofeng-text-muted'"
                >
                  {{ folder.count }}
                </span>
              </div>
            </div>
          </div>

          <!-- Stats Card -->
          <div class="glass-effect rounded-2xl p-5 border border-white/20 shadow-sm relative overflow-hidden group">
            <div class="absolute top-0 right-0 w-24 h-24 bg-guofeng-jade/10 rounded-full blur-2xl -mr-8 -mt-8 transition-all group-hover:bg-guofeng-jade/20" />
            <h4 class="text-sm font-bold text-guofeng-text-primary mb-1">
              Agent Status
            </h4>
            <div class="flex items-center gap-2 mt-3">
              <div class="flex-1 bg-guofeng-bg-tertiary rounded-lg p-2 text-center">
                <div class="text-lg font-bold text-guofeng-jade">
                  {{ stats.active }}
                </div>
                <div class="text-[10px] text-guofeng-text-muted uppercase">
                  Active
                </div>
              </div>
              <div class="flex-1 bg-guofeng-bg-tertiary rounded-lg p-2 text-center">
                <div class="text-lg font-bold text-guofeng-text-muted">
                  {{ stats.disabled }}
                </div>
                <div class="text-[10px] text-guofeng-text-muted uppercase">
                  Disabled
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Main Content -->
        <div class="flex-1 min-w-0">
          <!-- Header Bar -->
          <div class="glass-effect rounded-2xl p-4 mb-6 border border-white/20 flex flex-col md:flex-row items-center justify-between gap-4 sticky top-6 z-20 backdrop-blur-xl shadow-sm">
            <div class="flex items-center gap-3 w-full md:w-auto">
              <div class="relative flex-1 md:w-80">
                <Search class="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-guofeng-text-muted" />
                <input
                  v-model="searchQuery"
                  type="text"
                  :placeholder="$t(`${tPrefix}.searchPlaceholder`)"
                  class="w-full pl-10 pr-10 py-2.5 rounded-xl transition-all focus:outline-none focus:ring-2 focus:ring-guofeng-jade/20 bg-guofeng-bg-tertiary/50 border border-guofeng-border hover:bg-guofeng-bg-tertiary text-guofeng-text-primary placeholder-guofeng-text-muted text-sm"
                >
                <button
                  v-if="searchQuery"
                  class="absolute right-3 top-1/2 transform -translate-y-1/2 p-0.5 rounded-full hover:bg-black/10 text-guofeng-text-muted transition-all"
                  @click="searchQuery = ''"
                >
                  <X class="w-3 h-3" />
                </button>
              </div>
            </div>

            <div class="flex items-center gap-3 w-full md:w-auto justify-end">
              <button
                class="px-4 py-2.5 rounded-xl font-medium transition-all hover:scale-105 bg-guofeng-jade text-white shadow-lg shadow-guofeng-jade/20 hover:shadow-guofeng-jade/30 flex items-center text-sm"
                @click="handleAdd"
              >
                <Plus class="w-4 h-4 mr-2" />{{ $t(`${tPrefix}.addAgent`) }}
              </button>
            </div>
          </div>

          <!-- Agent Grid -->
          <div
            v-if="loading"
            class="text-center py-20 text-guofeng-text-muted"
          >
            <div class="loading-spinner mx-auto mb-4 w-8 h-8 border-guofeng-jade/30 border-t-guofeng-jade" />
            {{ $t(`${tPrefix}.loading`) }}
          </div>
           
          <div
            v-else-if="filteredAgents.length === 0"
            class="text-center py-24 glass-effect rounded-3xl border border-white/20 border-dashed"
          >
            <div class="bg-guofeng-bg-secondary w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
              <Search class="w-10 h-10 opacity-30 text-guofeng-text-muted" />
            </div>
            <p class="text-lg font-bold text-guofeng-text-primary">
              {{ $t(`${tPrefix}.noResults`) }}
            </p>
            <p class="text-sm mt-2 text-guofeng-text-muted">
              {{ $t(`${tPrefix}.noResultsHint`) }}
            </p>
            <button 
              class="mt-6 px-4 py-2 text-sm text-guofeng-jade hover:bg-guofeng-jade/5 rounded-lg transition-colors"
              @click="searchQuery = ''; selectedFolder = ''"
            >
              {{ $t(`${tPrefix}.tryOtherKeywords`) }}
            </button>
          </div>

          <div
            v-else
            class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 gap-5"
          >
            <GuofengCard
              v-for="agent in filteredAgents"
              :key="agent.name"
              variant="glass"
              interactive
              pattern
              class="h-full flex flex-col group"
              @click="handleEdit(agent)"
            >
              <div class="relative z-10 flex flex-col h-full">
                <div class="flex items-start justify-between mb-3">
                  <div class="flex items-center gap-3 overflow-hidden">
                    <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-guofeng-jade/10 to-guofeng-blue/10 flex items-center justify-center text-lg shadow-sm border border-white/20 group-hover:scale-110 transition-transform duration-300">
                      🤖
                    </div>
                    <div class="min-w-0">
                      <h3 class="text-base font-bold text-guofeng-text-primary group-hover:text-guofeng-jade transition-colors truncate">
                        {{ agent.name }}
                      </h3>
                      <div class="flex items-center gap-1.5 mt-0.5">
                        <span
                          v-if="agent.folder"
                          class="flex items-center gap-1 text-[10px] text-guofeng-text-muted bg-guofeng-bg-tertiary px-1.5 py-0.5 rounded border border-guofeng-border/50"
                        >
                          <Folder class="w-3 h-3" /> {{ agent.folder }}
                        </span>
                      </div>
                    </div>
                  </div>
                   
                  <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                    <button
                      class="p-1.5 rounded-lg transition-colors hover:bg-guofeng-bg-tertiary"
                      :class="agent.disabled ? 'text-guofeng-text-muted hover:text-guofeng-jade' : 'text-guofeng-jade hover:text-guofeng-text-muted'"
                      :title="agent.disabled ? $t(`${tPrefix}.enable`) : $t(`${tPrefix}.disable`)"
                      @click.stop="handleToggle(agent.name)"
                    >
                      <PowerOff
                        v-if="agent.disabled"
                        class="w-4 h-4"
                      />
                      <Power
                        v-else
                        class="w-4 h-4"
                      />
                    </button>
                    <button
                      class="p-1.5 rounded-lg text-guofeng-text-secondary hover:text-guofeng-blue hover:bg-guofeng-blue/10 transition-colors"
                      :title="$t('common.edit')"
                      @click.stop="handleEdit(agent)"
                    >
                      <Edit2 class="w-4 h-4" />
                    </button>
                    <button
                      class="p-1.5 rounded-lg text-guofeng-text-secondary hover:text-guofeng-red hover:bg-guofeng-red/10 transition-colors"
                      :title="$t('common.delete')"
                      @click.stop="handleDelete(agent.name)"
                    >
                      <Trash2 class="w-4 h-4" />
                    </button>
                  </div>
                </div>

                <div class="flex-1 space-y-3">
                  <div
                    v-if="agent.system_prompt"
                    class="relative"
                  >
                    <div class="absolute left-0 top-0 bottom-0 w-0.5 bg-guofeng-jade/30 rounded-full" />
                    <p class="pl-3 text-xs text-guofeng-text-secondary line-clamp-3 leading-relaxed italic">
                      {{ agent.system_prompt }}
                    </p>
                  </div>
                  <div
                    v-else
                    class="text-xs text-guofeng-text-muted italic pl-3"
                  >
                    No system prompt configured
                  </div>
                </div>
                 
                <div class="mt-4 pt-3 border-t border-guofeng-border/30 flex items-center justify-between gap-2">
                  <div class="flex items-center gap-1.5 text-[10px] text-guofeng-text-muted bg-guofeng-bg-tertiary/50 px-2 py-1 rounded-md border border-guofeng-border/30">
                    <span class="w-1.5 h-1.5 rounded-full bg-guofeng-indigo/50" />
                    <span class="truncate max-w-[100px]">{{ agent.model }}</span>
                  </div>

                  <div
                    v-if="agent.tools && agent.tools.length > 0"
                    class="flex -space-x-1.5"
                  >
                    <div
                      v-for="(tool, i) in agent.tools.slice(0, 3)"
                      :key="i" 
                      class="w-6 h-6 rounded-full bg-white border border-guofeng-border flex items-center justify-center text-[10px] shadow-sm text-guofeng-text-secondary"
                      :title="tool"
                    >
                      {{ tool.charAt(0).toUpperCase() }}
                    </div>
                    <div
                      v-if="agent.tools.length > 3"
                      class="w-6 h-6 rounded-full bg-guofeng-bg-tertiary border border-guofeng-border flex items-center justify-center text-[9px] font-medium text-guofeng-text-muted"
                    >
                      +{{ agent.tools.length - 3 }}
                    </div>
                  </div>
                </div>
              </div>
               
              <!-- Disabled Overlay -->
              <div
                v-if="agent.disabled"
                class="absolute inset-0 bg-guofeng-bg/40 backdrop-blur-[2px] flex items-center justify-center z-20 rounded-xl border border-guofeng-text-muted/10"
              >
                <span class="px-3 py-1 bg-guofeng-text-muted/80 text-white text-xs font-bold rounded-full shadow-sm uppercase tracking-wider backdrop-blur-md">
                  {{ $t(`${tPrefix}.disabledBadge`) }}
                </span>
              </div>
            </GuofengCard>
          </div>
        </div>
      </div>
    </div>

    <!-- Add/Edit Modal -->
    <div
      v-if="showAddForm"
      class="fixed inset-0 flex items-center justify-center z-50 bg-guofeng-ink/20 backdrop-blur-sm transition-all p-4"
      @click="showAddForm = false"
    >
      <div
        class="glass-effect p-8 rounded-3xl w-full max-w-2xl max-h-[85vh] overflow-y-auto shadow-2xl border border-white/30 relative"
        @click.stop
      >
        <button 
          class="absolute top-4 right-4 p-2 rounded-full hover:bg-guofeng-bg-tertiary text-guofeng-text-muted transition-colors"
          @click="showAddForm = false"
        >
          <X class="w-5 h-5" />
        </button>

        <h3 class="text-2xl font-bold mb-8 text-guofeng-text-primary flex items-center">
          <div class="w-10 h-10 rounded-xl bg-guofeng-jade/10 flex items-center justify-center mr-3 text-guofeng-jade">
            <component
              :is="editingAgent ? Edit2 : Plus"
              class="w-5 h-5"
            />
          </div>
          {{ editingAgent ? $t(`${tPrefix}.editAgent`) : $t(`${tPrefix}.addAgent`) }}
        </h3>

        <div class="space-y-6">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <label class="block mb-2 text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider">{{ $t(`${tPrefix}.nameLabel`) }}</label>
              <input
                v-model="formData.name"
                type="text"
                class="w-full px-4 py-3 rounded-xl bg-white/50 border border-guofeng-border focus:border-guofeng-jade focus:ring-4 focus:ring-guofeng-jade/10 outline-none transition-all"
                :placeholder="$t(`${tPrefix}.namePlaceholder` || 'Agent Name')"
              >
            </div>

            <div>
              <label class="block mb-2 text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider">{{ $t(`${tPrefix}.modelLabel`) }}</label>
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
            <label class="block mb-2 text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider">{{ $t(`${tPrefix}.toolsLabel`) }}</label>
            <div class="flex gap-2 mb-3">
              <input
                v-model="toolInput"
                type="text"
                :placeholder="$t(`${tPrefix}.toolPlaceholder`)"
                class="flex-1 px-4 py-3 rounded-xl bg-white/50 border border-guofeng-border focus:border-guofeng-jade focus:ring-4 focus:ring-guofeng-jade/10 outline-none transition-all"
                @keyup.enter="addTool"
              >
              <button
                class="px-6 py-3 rounded-xl font-bold text-white bg-guofeng-jade hover:bg-guofeng-jade/90 transition-colors shadow-lg shadow-guofeng-jade/20"
                @click="addTool"
              >
                {{ $t(`${tPrefix}.addTool`) }}
              </button>
            </div>
            <div class="flex flex-wrap gap-2 min-h-[50px] p-4 rounded-xl bg-guofeng-bg-secondary/50 border border-guofeng-border/50 border-dashed">
              <span
                v-if="!formData.tools || formData.tools.length === 0"
                class="text-sm text-guofeng-text-muted italic w-full text-center py-2"
              >No tools added</span>
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
            <label class="block mb-2 text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider">{{ $t(`${tPrefix}.systemPromptLabel`) }}</label>
            <textarea
              v-model="formData.system_prompt"
              rows="6"
              class="w-full px-4 py-3 rounded-xl bg-white/50 border border-guofeng-border focus:border-guofeng-jade focus:ring-4 focus:ring-guofeng-jade/10 outline-none transition-all resize-y font-mono text-sm leading-relaxed"
              :placeholder="$t(`${tPrefix}.systemPromptPlaceholder` || 'Enter system prompt...')"
            />
          </div>
        </div>

        <div class="flex gap-4 mt-10 pt-6 border-t border-guofeng-border/50">
          <button
            class="flex-1 px-6 py-3.5 rounded-xl font-bold transition-all bg-white text-guofeng-text-secondary hover:bg-guofeng-bg-tertiary border border-guofeng-border"
            @click="showAddForm = false"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            class="flex-1 px-6 py-3.5 rounded-xl font-bold transition-all bg-guofeng-jade text-white shadow-lg shadow-guofeng-jade/20 hover:shadow-xl hover:shadow-guofeng-jade/30 hover:-translate-y-0.5"
            @click="handleSubmit"
          >
            {{ editingAgent ? $t(`${tPrefix}.save`) : $t(`${tPrefix}.add`) }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Plus, Edit2, Trash2, Power, PowerOff, Search, X, Folder, Home, ChevronDown, Code2, Sparkles, Workflow } from 'lucide-vue-next'
import Breadcrumb from '@/components/common/Breadcrumb.vue'
import GuofengCard from '@/components/common/GuofengCard.vue'
import { useAgents } from '@/composables/useAgents'
import type { Agent, AgentRequest } from '@/types'

const props = defineProps<{
  module: 'codex' | 'gemini' | 'qwen' | 'iflow' | 'agents'
}>()

const { t } = useI18n()
const tPrefix = computed(() => props.module === 'agents' ? 'agents' : `${props.module}.agents`)
const {
  agents,
  folders,
  loading,
  loadAgents,
  addAgent,
  updateAgent,
  deleteAgent,
  toggleAgent
} = useAgents(props.module)

const selectedFolder = ref('')
const searchQuery = ref('')
const showAddForm = ref(false)
const editingAgent = ref<Agent | null>(null)
const formData = ref<AgentRequest>({ name: '', model: 'claude-sonnet-4-5-20250929', tools: [], system_prompt: '', disabled: false })
const toolInput = ref('')

// Breadcrumbs
const breadcrumbs = computed(() => {
  const items: { label: string; to?: string; icon?: any }[] = [
    { label: t('common.home'), to: '/', icon: Home }
  ]

  if (props.module === 'agents') {
    items.push({ label: t('claudeCode.title'), to: '/claude-code', icon: Code2 })
  } else if (props.module === 'codex') {
    items.push({ label: t('nav.codex'), to: '/codex', icon: Code2 })
  } else if (props.module === 'gemini') {
    items.push({ label: t('nav.gemini'), to: '/gemini-cli', icon: Sparkles })
  } else if (props.module === 'qwen') {
    items.push({ label: t('nav.qwen'), to: '/qwen', icon: Sparkles })
  } else if (props.module === 'iflow') {
    items.push({ label: t('nav.iflow'), to: '/iflow', icon: Workflow })
  }

  items.push({ label: t(`${tPrefix.value}.pageTitle`) })
  return items
})

// Reload agents when module changes
watch(() => props.module, () => {
  loadAgents()
  selectedFolder.value = ''
  searchQuery.value = ''
})

onMounted(() => {
  loadAgents()
})

const stats = computed(() => {
  const rootCount = agents.value.filter((a) => !a.folder || a.folder === '').length
  const folderCounts: Record<string, number> = {}
  folders.value.forEach((f) => { folderCounts[f] = agents.value.filter((a) => a.folder === f).length })
  
  const active = agents.value.filter(a => !a.disabled).length
  const disabled = agents.value.filter(a => a.disabled).length
  
  return { rootCount, folderCounts, total: agents.value.length, active, disabled }
})

const folderOptions = computed(() => [
  { value: '', label: t(`${tPrefix.value}.folders.all`), icon: Folder, count: stats.value.total },
  { value: '__root__', label: t(`${tPrefix.value}.folders.root`), icon: Home, count: stats.value.rootCount },
  ...folders.value.map((f) => ({ value: f, label: f, icon: Folder, count: stats.value.folderCounts[f] || 0 }))
])

const filteredAgents = computed(() => {
  let filtered = agents.value
  if (selectedFolder.value === '__root__') filtered = agents.value.filter((agent) => !agent.folder || agent.folder === '')
  else if (selectedFolder.value) filtered = agents.value.filter((agent) => agent.folder === selectedFolder.value)
  
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    filtered = filtered.filter((agent) => 
      agent.name.toLowerCase().includes(query) || 
      (agent.system_prompt && agent.system_prompt.toLowerCase().includes(query)) || 
      (agent.tools && agent.tools.some(tool => tool.toLowerCase().includes(query)))
    )
  }
  return filtered.sort((a, b) => a.name.localeCompare(b.name))
})

const handleAdd = () => {
  showAddForm.value = true
  editingAgent.value = null
  formData.value = { name: '', model: 'claude-sonnet-4-5-20250929', tools: [], system_prompt: '', disabled: false }
  toolInput.value = ''
}

const handleEdit = (agent: Agent) => {
  editingAgent.value = agent
  showAddForm.value = true
  formData.value = { 
    name: agent.name, 
    model: agent.model, 
    tools: [...(agent.tools || [])], 
    system_prompt: agent.system_prompt || '', 
    disabled: agent.disabled || false 
  }
  toolInput.value = ''
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

const handleSubmit = async () => {
  if (!formData.value.name || !formData.value.model) {
    alert(t(`${tPrefix.value}.validation.required`))
    return
  }

  const request: AgentRequest = {
    ...formData.value,
    tools: (formData.value.tools && formData.value.tools.length > 0) ? formData.value.tools : undefined,
    system_prompt: formData.value.system_prompt || undefined
  }
  
  try {
    if (editingAgent.value) {
      await updateAgent(editingAgent.value.name, request)
    } else {
      await addAgent(request)
    }
    showAddForm.value = false
    editingAgent.value = null
  } catch (err) {
    console.error('Operation failed:', err)
    alert(t(`${tPrefix.value}.messages.operationFailed`, { error: err instanceof Error ? err.message : 'Unknown error' }))
  }
}

const handleDelete = async (name: string) => {
  if (!confirm(t(`${tPrefix.value}.deleteConfirm`, { name }))) return
  try {
    await deleteAgent(name)
  } catch (err) {
    console.error('Delete failed:', err)
    alert(t(`${tPrefix.value}.messages.deleteFailed`, { error: err instanceof Error ? err.message : 'Unknown error' }))
  }
}

const handleToggle = async (name: string) => {
  try {
    await toggleAgent(name)
  } catch (err) {
    console.error('Toggle failed:', err)
    alert(t(`${tPrefix.value}.messages.toggleFailed`, { error: err instanceof Error ? err.message : 'Unknown error' }))
  }
}
</script>

<style scoped>
/* Custom Scrollbar for the folder list */
::-webkit-scrollbar {
  width: 4px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: rgba(0,0,0,0.1);
  border-radius: 4px;
}
::-webkit-scrollbar-thumb:hover {
  background: rgba(0,0,0,0.2);
}
</style>
