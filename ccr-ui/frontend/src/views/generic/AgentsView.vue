<template>
  <div class="min-h-screen p-5 bg-guofeng-bg text-guofeng-ink transition-colors duration-300">
    <div class="mb-6" />

    <div class="flex gap-0">
      <CollapsibleSidebar :module="module" />

      <!-- Sidebar: Folders -->
      <div class="w-[180px] bg-guofeng-bg-secondary border-r border-guofeng-border p-3 overflow-y-auto h-[calc(100vh-40px)] flex-shrink-0">
        <h4 class="text-guofeng-text-muted text-[11px] font-semibold mb-2 ml-2 uppercase tracking-wider">
          {{ $t(`${tPrefix}.folders.label`) }}
        </h4>
        <div
          v-for="folder in folderOptions"
          :key="folder.value"
          class="flex items-center gap-1.5 px-2 py-1.5 rounded-md cursor-pointer mb-1 text-[13px] transition-all duration-200"
          :class="[
            selectedFolder === folder.value 
              ? 'bg-guofeng-red text-white shadow-sm' 
              : 'text-guofeng-text-primary hover:bg-guofeng-bg-tertiary'
          ]"
          @click="selectedFolder = folder.value"
        >
          <component :is="folder.icon" class="w-3.5 h-3.5" />
          <span class="flex-1 truncate">{{ folder.label }}</span>
          <span class="text-[11px] opacity-70">{{ folder.count }}</span>
        </div>
      </div>

      <!-- Main Content -->
      <div class="flex-1 min-w-0 pl-6 pr-2">
        <div class="max-w-[1600px] mx-auto">
          <!-- Header -->
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-4">
              <h2 class="text-2xl font-bold text-guofeng-text-primary flex items-center">
                <Bot class="w-7 h-7 mr-2 text-guofeng-red" />
                {{ $t(`${tPrefix}.pageTitle`) }}
              </h2>
              <span class="px-3 py-1 rounded-full text-sm font-medium bg-guofeng-red/10 text-guofeng-red border border-guofeng-red/20">
                {{ filteredAgents.length }}/{{ stats.total }}
              </span>
            </div>
            <div class="flex items-center gap-3">
              <RouterLink
                :to="`/${module}`"
                class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors bg-guofeng-bg-secondary text-guofeng-text-secondary border border-guofeng-border hover:bg-guofeng-bg-tertiary"
              >
                <Home class="w-4 h-4" /><span>{{ $t(`${tPrefix}.backToHome`) }}</span>
              </RouterLink>
              <button
                class="px-4 py-2 rounded-lg font-medium transition-all hover:scale-105 bg-guofeng-red text-white shadow-md hover:shadow-lg flex items-center"
                @click="handleAdd"
              >
                <Plus class="w-5 h-5 mr-2" />{{ $t(`${tPrefix}.addAgent`) }}
              </button>
            </div>
          </div>

          <!-- Search -->
          <div class="mb-6 relative">
            <Search class="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-guofeng-text-muted" />
            <input
              v-model="searchQuery"
              type="text"
              :placeholder="$t(`${tPrefix}.searchPlaceholder`)"
              class="w-full pl-11 pr-10 py-3 rounded-lg transition-all focus:outline-none focus:ring-2 focus:ring-guofeng-red/20 bg-guofeng-bg-tertiary border border-guofeng-border text-guofeng-text-primary placeholder-guofeng-text-muted"
            >
            <button
              v-if="searchQuery"
              class="absolute right-3 top-1/2 transform -translate-y-1/2 p-1 rounded-lg hover:bg-black/5 text-guofeng-text-muted transition-all"
              @click="searchQuery = ''"
            >
              <X class="w-4 h-4" />
            </button>
          </div>

          <!-- Agent Grid -->
          <div v-if="loading" class="text-center py-20 text-guofeng-text-muted">
            <div class="loading-spinner mx-auto mb-4 w-8 h-8 border-guofeng-red/30 border-t-guofeng-red"></div>
            {{ $t(`${tPrefix}.loading`) }}
          </div>
          
          <div v-else-if="filteredAgents.length === 0" class="text-center py-20 text-guofeng-text-muted">
            <div class="bg-guofeng-bg-secondary w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
              <Search class="w-10 h-10 opacity-50" />
            </div>
            <p class="text-lg font-medium">{{ $t(`${tPrefix}.noResults`) }}</p>
            <p class="text-sm mt-2 opacity-70">{{ $t(`${tPrefix}.noResultsHint`) }}</p>
          </div>

          <div v-else class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-4">
            <GuofengCard
              v-for="agent in filteredAgents"
              :key="agent.name"
              interactive
              pattern
              @click="handleEdit(agent)"
            >
              <div class="relative z-10">
                <div class="flex items-start justify-between mb-3">
                  <div class="flex items-center gap-2">
                    <h3 class="text-lg font-bold text-guofeng-text-primary group-hover:text-guofeng-red transition-colors">
                      {{ agent.name }}
                    </h3>
                    <span v-if="agent.folder" class="px-1.5 py-0.5 rounded text-[10px] font-medium bg-guofeng-bg-tertiary text-guofeng-text-secondary border border-guofeng-border">
                      {{ agent.folder }}
                    </span>
                  </div>
                  <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                     <button
                      class="p-1.5 rounded-md transition-colors hover:bg-guofeng-bg-tertiary"
                      :class="agent.disabled ? 'text-guofeng-gold' : 'text-guofeng-jade'"
                      :title="agent.disabled ? $t(`${tPrefix}.enable`) : $t(`${tPrefix}.disable`)"
                      @click.stop="handleToggle(agent.name)"
                    >
                      <PowerOff v-if="agent.disabled" class="w-4 h-4" />
                      <Power v-else class="w-4 h-4" />
                    </button>
                    <button
                      class="p-1.5 rounded-md text-guofeng-blue hover:bg-guofeng-blue/10 transition-colors"
                      :title="$t('common.edit')"
                      @click.stop="handleEdit(agent)"
                    >
                      <Edit2 class="w-4 h-4" />
                    </button>
                    <button
                      class="p-1.5 rounded-md text-guofeng-red hover:bg-guofeng-red/10 transition-colors"
                      :title="$t('common.delete')"
                      @click.stop="handleDelete(agent.name)"
                    >
                      <Trash2 class="w-4 h-4" />
                    </button>
                  </div>
                </div>

                <div class="space-y-2 text-sm">
                  <div class="flex items-center gap-2 text-guofeng-text-secondary">
                    <span class="w-12 text-xs uppercase tracking-wide text-guofeng-text-muted font-semibold">Model</span>
                    <span class="font-mono text-xs bg-guofeng-bg-tertiary px-1.5 py-0.5 rounded">{{ agent.model }}</span>
                  </div>
                  
                  <div v-if="agent.tools && agent.tools.length > 0" class="flex items-start gap-2 text-guofeng-text-secondary">
                    <span class="w-12 text-xs uppercase tracking-wide text-guofeng-text-muted font-semibold mt-0.5">Tools</span>
                    <div class="flex flex-wrap gap-1 flex-1">
                      <span v-for="tool in agent.tools.slice(0, 3)" :key="tool" class="text-[10px] px-1.5 py-0.5 bg-guofeng-blue/5 text-guofeng-blue border border-guofeng-blue/20 rounded">
                        {{ tool }}
                      </span>
                      <span v-if="agent.tools.length > 3" class="text-[10px] px-1.5 py-0.5 text-guofeng-text-muted">
                        +{{ agent.tools.length - 3 }}
                      </span>
                    </div>
                  </div>

                  <div v-if="agent.system_prompt" class="mt-3 pt-3 border-t border-guofeng-border/50">
                    <p class="text-xs text-guofeng-text-muted line-clamp-2 italic leading-relaxed">
                      "{{ agent.system_prompt }}"
                    </p>
                  </div>
                </div>
              </div>
              
              <!-- Disabled Overlay -->
              <div v-if="agent.disabled" class="absolute inset-0 bg-guofeng-bg/60 backdrop-blur-[1px] flex items-center justify-center z-20 rounded-xl border-2 border-dashed border-guofeng-gold/30">
                <span class="px-3 py-1 bg-guofeng-gold text-white text-xs font-bold rounded-full shadow-sm uppercase tracking-wider">
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
      class="fixed inset-0 flex items-center justify-center z-50 bg-guofeng-ink/20 backdrop-blur-sm transition-all"
      @click="showAddForm = false"
    >
      <div
        class="bg-guofeng-bg p-8 rounded-2xl w-full max-w-2xl max-h-[85vh] overflow-y-auto shadow-2xl border border-guofeng-border relative"
        @click.stop
      >
        <button 
          class="absolute top-4 right-4 p-2 rounded-full hover:bg-guofeng-bg-tertiary text-guofeng-text-muted transition-colors"
          @click="showAddForm = false"
        >
          <X class="w-5 h-5" />
        </button>

        <h3 class="text-2xl font-bold mb-6 text-guofeng-text-primary flex items-center">
          <component :is="editingAgent ? Edit2 : Plus" class="w-6 h-6 mr-2 text-guofeng-red" />
          {{ editingAgent ? $t(`${tPrefix}.editAgent`) : $t(`${tPrefix}.addAgent`) }}
        </h3>

        <div class="space-y-5">
          <div>
            <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-secondary">{{ $t(`${tPrefix}.nameLabel`) }}</label>
            <input
              v-model="formData.name"
              type="text"
              class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all"
              :placeholder="$t(`${tPrefix}.namePlaceholder` || 'Agent Name')"
            >
          </div>

          <div>
            <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-secondary">{{ $t(`${tPrefix}.modelLabel`) }}</label>
            <select
              v-model="formData.model"
              class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all appearance-none"
            >
              <option value="claude-sonnet-4-5-20250929">Claude Sonnet 4.5</option>
              <option value="claude-opus-4-20250514">Claude Opus 4</option>
              <option value="claude-3-5-sonnet-20241022">Claude 3.5 Sonnet</option>
              <!-- Add more models as needed or fetch dynamically -->
            </select>
          </div>

          <div>
            <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-secondary">{{ $t(`${tPrefix}.toolsLabel`) }}</label>
            <div class="flex gap-2 mb-3">
              <input
                v-model="toolInput"
                type="text"
                :placeholder="$t(`${tPrefix}.toolPlaceholder`)"
                class="flex-1 px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all"
                @keyup.enter="addTool"
              >
              <button
                class="px-4 py-2 rounded-lg font-medium text-white bg-guofeng-blue hover:bg-guofeng-blue/90 transition-colors"
                @click="addTool"
              >
                {{ $t(`${tPrefix}.addTool`) }}
              </button>
            </div>
            <div class="flex flex-wrap gap-2 min-h-[40px] p-3 rounded-lg bg-guofeng-bg-secondary border border-guofeng-border border-dashed">
              <span v-if="formData.tools.length === 0" class="text-sm text-guofeng-text-muted italic w-full text-center py-1">No tools added</span>
              <span
                v-for="tool in formData.tools"
                :key="tool"
                class="px-3 py-1 rounded-full text-sm flex items-center gap-2 bg-white border border-guofeng-border shadow-sm text-guofeng-text-primary"
              >
                {{ tool }}
                <button @click="removeTool(tool)" class="text-guofeng-text-muted hover:text-guofeng-red transition-colors"><X class="w-3 h-3" /></button>
              </span>
            </div>
          </div>

          <div>
            <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-secondary">{{ $t(`${tPrefix}.systemPromptLabel`) }}</label>
            <textarea
              v-model="formData.system_prompt"
              rows="6"
              class="w-full px-4 py-3 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all resize-y font-mono text-sm"
              :placeholder="$t(`${tPrefix}.systemPromptPlaceholder` || 'Enter system prompt...')"
            />
          </div>
        </div>

        <div class="flex gap-4 mt-8 pt-6 border-t border-guofeng-border">
           <button
            class="flex-1 px-6 py-3 rounded-lg font-medium transition-all bg-guofeng-bg-tertiary text-guofeng-text-secondary hover:bg-guofeng-bg-secondary border border-guofeng-border"
            @click="showAddForm = false"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            class="flex-1 px-6 py-3 rounded-lg font-medium transition-all bg-guofeng-red text-white shadow-md hover:shadow-lg hover:-translate-y-0.5"
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
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Bot, Plus, Edit2, Trash2, Power, PowerOff, Search, X, Folder, Home } from 'lucide-vue-next'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'
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
  return { rootCount, folderCounts, total: agents.value.length }
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
  if (toolInput.value.trim() && !formData.value.tools.includes(toolInput.value.trim())) {
    formData.value.tools.push(toolInput.value.trim())
    toolInput.value = ''
  }
}

const removeTool = (tool: string) => {
  formData.value.tools = formData.value.tools.filter(t => t !== tool)
}

const handleSubmit = async () => {
  if (!formData.value.name || !formData.value.model) { 
    alert(t(`${tPrefix.value}.validation.required`))
    return 
  }
  
  const request: AgentRequest = { 
    ...formData.value, 
    tools: formData.value.tools.length > 0 ? formData.value.tools : undefined, 
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
