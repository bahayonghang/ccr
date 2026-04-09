<!-- -->
<template>
  <div class="min-h-full p-6 transition-colors duration-300">
    <div class="max-w-[1800px] mx-auto">
      <ModuleSubnav
        :module="moduleNavModule"
        class="mb-6"
      />

      <div class="flex gap-6 items-start">
        <!-- Left Sidebar (Folders) -->
        <div class="w-64 flex-shrink-0 space-y-4 hidden lg:block sticky top-6">
          <div class="glass-effect rounded-2xl p-4 border border-white/20 shadow-sm">
            <h3 class="text-xs font-bold text-text-muted uppercase tracking-wider mb-3 px-2 flex items-center justify-between">
              {{ $t(`${tPrefix}.folders.label`) }}
              <span class="bg-bg-surface px-1.5 py-0.5 rounded text-[10px]">{{ stats.total }}</span>
            </h3>
             
            <div class="space-y-1">
              <button
                v-for="folder in folderOptions"
                :key="folder.value"
                type="button"
                class="flex w-full items-center gap-2 rounded-xl px-3 py-2 text-left text-sm transition-colors duration-200 group min-h-[44px]"
                :class="[
                  selectedFolder === folder.value 
                    ? 'bg-accent-primary/10 text-accent-primary font-medium shadow-sm border border-accent-primary/20' 
                    : 'text-text-secondary hover:bg-bg-surface hover:text-text-primary'
                ]"
                @click="selectedFolder = folder.value"
              >
                <SIcon
                  :name="folder.icon"
                  size="w-4 h-4"
                  class="transition-transform group-hover:scale-110"
                  :class="selectedFolder === folder.value ? 'text-accent-primary' : 'text-text-muted'"
                />
                <span class="flex-1 truncate">{{ folder.label }}</span>
                <span 
                  class="text-xs px-1.5 py-0.5 rounded-md transition-colors"
                  :class="selectedFolder === folder.value ? 'bg-accent-primary/20 text-accent-primary' : 'bg-bg-surface text-text-muted'"
                >
                  {{ folder.count }}
                </span>
              </button>
            </div>
          </div>

          <!-- Stats Card -->
          <div class="glass-effect rounded-2xl p-5 border border-white/20 shadow-sm relative overflow-hidden group">
            <div class="absolute top-0 right-0 w-24 h-24 bg-accent-primary/10 rounded-full blur-2xl -mr-8 -mt-8 transition-colors group-hover:bg-accent-primary/20" />
            <h4 class="text-sm font-bold text-text-primary mb-1">
              Agent Status
            </h4>
            <div class="flex items-center gap-2 mt-3">
              <div class="flex-1 bg-bg-surface rounded-lg p-2 text-center">
                <div class="text-lg font-bold text-accent-primary">
                  {{ stats.active }}
                </div>
                <div class="text-[10px] text-text-muted uppercase">
                  Active
                </div>
              </div>
              <div class="flex-1 bg-bg-surface rounded-lg p-2 text-center">
                <div class="text-lg font-bold text-text-muted">
                  {{ stats.disabled }}
                </div>
                <div class="text-[10px] text-text-muted uppercase">
                  Disabled
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Main Content -->
        <div class="flex-1 min-w-0">
          <PageHeaderCard
            :title="$t(`${tPrefix}.pageTitle`)"
            icon="Bot"
            :badge="String(stats.total)"
            tone="primary"
            class="mb-6"
          >
            <template #meta>
              <span class="inline-flex items-center gap-2 rounded-full border border-accent-primary/20 bg-accent-primary/10 px-3 py-1 text-sm font-medium text-accent-primary">
                {{ stats.active }} Active
              </span>
              <span class="inline-flex items-center gap-2 rounded-full border border-border-default/50 bg-bg-surface/70 px-3 py-1 text-sm font-medium text-text-secondary">
                {{ stats.disabled }} Disabled
              </span>
            </template>

            <template #actions>
              <button
                class="min-h-[44px] px-4 py-2.5 rounded-xl font-medium transition-[color,background-color,border-color,transform] hover:scale-105 bg-accent-primary text-white shadow-lg shadow-accent-primary/20 hover:shadow-accent-primary/30 flex items-center text-sm"
                @click="handleAdd"
              >
                <SIcon
                  name="Plus"
                  size="w-4 h-4"
                  class="mr-2"
                />{{ $t(`${tPrefix}.addAgent`) }}
              </button>
            </template>

            <div class="relative max-w-md">
              <SIcon
                name="Search"
                size="w-4 h-4"
                class="absolute left-3 top-1/2 transform -translate-y-1/2 text-text-muted"
              />
              <input
                v-model="searchQuery"
                type="text"
                :placeholder="$t(`${tPrefix}.searchPlaceholder`)"
                class="w-full pl-10 pr-10 py-2.5 rounded-xl transition-colors focus:outline-none focus:ring-2 focus:ring-accent-primary/20 bg-bg-surface/50 border border-border-default hover:bg-bg-surface text-text-primary placeholder:text-text-muted text-sm"
              >
              <button
                v-if="searchQuery"
                class="absolute right-2 top-1/2 transform -translate-y-1/2 rounded-full hover:bg-black/10 text-text-muted transition-colors min-h-[44px] min-w-[44px] flex items-center justify-center"
                :aria-label="$t('common.clearSearch')"
                @click="searchQuery = ''"
              >
                <SIcon
                  name="X"
                  size="w-3 h-3"
                />
              </button>
            </div>
          </PageHeaderCard>

          <!-- Agent Grid -->
          <div
            v-if="loading"
            class="text-center py-20 text-text-muted"
          >
            <div class="loading-spinner mx-auto mb-4 w-8 h-8 border-accent-primary/30 border-t-accent-primary" />
            {{ $t(`${tPrefix}.loading`) }}
          </div>
           
          <div
            v-else-if="filteredAgents.length === 0"
            class="text-center py-24 glass-effect rounded-3xl border border-white/20 border-dashed"
          >
            <div class="bg-bg-elevated w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
              <SIcon
                name="Search"
                size="w-10 h-10"
                class="opacity-30 text-text-muted"
              />
            </div>
            <p class="text-lg font-bold text-text-primary">
              {{ $t(`${tPrefix}.noResults`) }}
            </p>
            <p class="text-sm mt-2 text-text-muted">
              {{ $t(`${tPrefix}.noResultsHint`) }}
            </p>
            <button 
              class="mt-6 min-h-[44px] px-4 py-2 text-sm text-accent-primary hover:bg-accent-primary/5 rounded-lg transition-colors"
              @click="searchQuery = ''; selectedFolder = ''"
            >
              {{ $t(`${tPrefix}.tryOtherKeywords`) }}
            </button>
          </div>

          <div
            v-else
          >
            <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 gap-5">
              <Card
                v-for="agent in paginatedAgents"
                :key="agent.name"
                variant="glass"
                pattern
                class="h-full flex flex-col group"
              >
                <div class="relative z-10 flex flex-col h-full">
                  <div class="flex items-start justify-between mb-3">
                    <div class="flex items-center gap-3 overflow-hidden">
                      <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-accent-primary/10 to-accent-info/10 flex items-center justify-center text-lg shadow-sm border border-white/20 group-hover:scale-110 transition-transform duration-300">
                        🤖
                      </div>
                      <div class="min-w-0">
                        <h3 class="text-base font-bold text-text-primary group-hover:text-accent-primary transition-colors truncate">
                          {{ agent.name }}
                        </h3>
                        <div class="flex items-center gap-1.5 mt-0.5">
                          <span
                            v-if="agent.folder"
                            class="flex items-center gap-1 text-[10px] text-text-muted bg-bg-surface px-1.5 py-0.5 rounded border border-border-default/50"
                          >
                            <SIcon
                              name="Folder"
                              size="w-3 h-3"
                            /> {{ agent.folder }}
                          </span>
                        </div>
                      </div>
                    </div>
                   
                    <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                      <button
                        v-if="!isQoderSubagents"
                        class="min-h-[44px] min-w-[44px] rounded-lg transition-colors hover:bg-bg-surface flex items-center justify-center"
                        :class="agent.disabled ? 'text-text-muted hover:text-accent-primary' : 'text-accent-primary hover:text-text-muted'"
                        :title="agent.disabled ? $t(`${tPrefix}.enable`) : $t(`${tPrefix}.disable`)"
                        @click.stop="handleToggle(agent)"
                      >
                        <SIcon
                          v-if="agent.disabled"
                          name="PowerOff"
                          size="w-4 h-4"
                        />
                        <SIcon
                          v-else
                          name="Power"
                          size="w-4 h-4"
                        />
                      </button>
                      <button
                        v-if="module === 'agents'"
                        class="min-h-[44px] min-w-[44px] rounded-lg text-text-secondary hover:text-accent-primary hover:bg-accent-primary/10 transition-colors flex items-center justify-center"
                        :title="$t('common.view')"
                        @click.stop="navigateToDetail(agent)"
                      >
                        <SIcon
                          name="Eye"
                          size="w-4 h-4"
                        />
                      </button>
                      <button
                        class="min-h-[44px] min-w-[44px] rounded-lg text-text-secondary hover:text-accent-info hover:bg-accent-info/10 transition-colors flex items-center justify-center"
                        :title="$t('common.edit')"
                        @click.stop="handleEdit(agent)"
                      >
                        <SIcon
                          name="Edit2"
                          size="w-4 h-4"
                        />
                      </button>
                      <button
                        class="min-h-[44px] min-w-[44px] rounded-lg text-text-secondary hover:text-accent-danger hover:bg-accent-danger/10 transition-colors flex items-center justify-center"
                        :title="$t('common.delete')"
                        @click.stop="handleDelete(agent)"
                      >
                        <SIcon
                          name="Trash2"
                          size="w-4 h-4"
                        />
                      </button>
                    </div>
                  </div>

                  <button
                    type="button"
                    class="flex h-full flex-col text-left rounded-2xl transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/25"
                    :aria-label="$t('common.view') + ': ' + agent.name"
                    @click="navigateToDetail(agent)"
                  >
                    <div class="flex-1 space-y-3">
                      <div
                        v-if="agent.system_prompt"
                        class="relative"
                      >
                        <div class="absolute left-0 top-0 bottom-0 w-0.5 bg-accent-primary/30 rounded-full" />
                        <p class="pl-3 text-xs text-text-secondary line-clamp-3 leading-relaxed italic">
                          {{ agent.system_prompt }}
                        </p>
                      </div>
                      <div
                        v-else
                        class="text-xs text-text-muted italic pl-3"
                      >
                        No system prompt configured
                      </div>
                    </div>
                   
                    <div class="mt-4 pt-3 border-t border-border-default/30 flex items-center justify-between gap-2">
                      <div class="flex items-center gap-1.5 text-[10px] text-text-muted bg-bg-surface/50 px-2 py-1 rounded-md border border-border-default/30">
                        <span class="w-1.5 h-1.5 rounded-full bg-accent-secondary/50" />
                        <span class="truncate max-w-[120px]">
                          {{ isQoderSubagents ? 'Subagent' : agent.model }}
                        </span>
                      </div>

                      <div
                        v-if="agent.tools && agent.tools.length > 0"
                        class="flex -space-x-1.5"
                      >
                        <div
                          v-for="(tool, i) in agent.tools.slice(0, 3)"
                          :key="i" 
                          class="w-6 h-6 rounded-full bg-white border border-border-default flex items-center justify-center text-[10px] shadow-sm text-text-secondary"
                          :title="tool"
                        >
                          {{ tool.charAt(0).toUpperCase() }}
                        </div>
                        <div
                          v-if="agent.tools.length > 3"
                          class="w-6 h-6 rounded-full bg-bg-surface border border-border-default flex items-center justify-center text-[9px] font-medium text-text-muted"
                        >
                          +{{ agent.tools.length - 3 }}
                        </div>
                      </div>
                    </div>
                  </button>
                </div>
               
                <!-- Disabled Overlay -->
                <div
                  v-if="agent.disabled"
                  class="absolute inset-0 bg-bg-base/40 backdrop-blur-[2px] flex items-center justify-center z-20 rounded-xl border border-text-muted/10"
                >
                  <span class="px-3 py-1 bg-text-muted/80 text-white text-xs font-bold rounded-full shadow-sm uppercase tracking-wider backdrop-blur-md">
                    {{ $t(`${tPrefix}.disabledBadge`) }}
                  </span>
                </div>
              </Card>
            </div>

            <MarketplacePagination
              :current-page="currentPage"
              :total-items="filteredAgents.length"
              :page-size="PAGE_SIZE"
              class="mt-6"
              @page-change="currentPage = $event"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Add/Edit Modal -->
    <div
      v-if="showAddForm"
      class="fixed inset-0 flex items-center justify-center z-50 bg-bg-overlay/20 backdrop-blur-md transition-colors p-4"
      @click="showAddForm = false"
    >
      <div
        class="glass-effect p-8 rounded-3xl w-full max-w-2xl max-h-[85vh] overflow-y-auto shadow-2xl border border-white/30 relative"
        @click.stop
      >
        <button 
          class="absolute top-4 right-4 p-2 rounded-full hover:bg-bg-surface text-text-muted transition-colors"
          @click="showAddForm = false"
        >
          <SIcon
            name="X"
            size="w-5 h-5"
          />
        </button>

        <h3 class="text-2xl font-bold mb-8 text-text-primary flex items-center">
          <div class="w-10 h-10 rounded-xl bg-accent-primary/10 flex items-center justify-center mr-3 text-accent-primary">
            <SIcon
              :name="editingAgent ? 'Edit2' : 'Plus'"
              size="w-5 h-5"
            />
          </div>
          {{ editingAgent ? $t(`${tPrefix}.editAgent`) : $t(`${tPrefix}.addAgent`) }}
        </h3>

        <div class="space-y-6">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <label class="block mb-2 text-xs font-bold text-text-secondary uppercase tracking-wider">{{ $t(`${tPrefix}.nameLabel`) }}</label>
              <input
                v-model="formData.name"
                type="text"
                class="w-full px-4 py-3 rounded-xl bg-white/50 border border-border-default focus:border-accent-primary focus:ring-4 focus:ring-accent-primary/10 outline-none transition-colors"
                :placeholder="$t(`${tPrefix}.namePlaceholder` || 'Agent Name')"
              >
            </div>

            <div v-if="!isQoderSubagents">
              <label class="block mb-2 text-xs font-bold text-text-secondary uppercase tracking-wider">{{ $t(`${tPrefix}.modelLabel`) }}</label>
              <div class="relative">
                <select
                  v-model="formData.model"
                  class="w-full px-4 py-3 rounded-xl bg-white/50 border border-border-default focus:border-accent-primary focus:ring-4 focus:ring-accent-primary/10 outline-none transition-colors appearance-none"
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
            <label class="block mb-2 text-xs font-bold text-text-secondary uppercase tracking-wider">{{ $t(`${tPrefix}.toolsLabel`) }}</label>
            <div class="flex gap-2 mb-3">
              <input
                v-model="toolInput"
                type="text"
                :placeholder="$t(`${tPrefix}.toolPlaceholder`)"
                class="flex-1 px-4 py-3 rounded-xl bg-white/50 border border-border-default focus:border-accent-primary focus:ring-4 focus:ring-accent-primary/10 outline-none transition-colors"
                @keyup.enter="addTool"
              >
              <button
                class="px-6 py-3 rounded-xl font-bold text-white bg-accent-primary hover:bg-accent-primary/90 transition-colors shadow-lg shadow-accent-primary/20"
                @click="addTool"
              >
                {{ $t(`${tPrefix}.addTool`) }}
              </button>
            </div>
            <div class="flex flex-wrap gap-2 min-h-[50px] p-4 rounded-xl bg-bg-elevated/50 border border-border-default/50 border-dashed">
              <span
                v-if="!formData.tools || formData.tools.length === 0"
                class="text-sm text-text-muted italic w-full text-center py-2"
              >No tools added</span>
              <span
                v-for="tool in (formData.tools || [])"
                :key="tool"
                class="px-3 py-1.5 rounded-lg text-sm flex items-center gap-2 bg-white border border-border-default shadow-sm text-text-primary group"
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
            <label class="block mb-2 text-xs font-bold text-text-secondary uppercase tracking-wider">{{ $t(`${tPrefix}.systemPromptLabel`) }}</label>
            <textarea
              v-model="formData.system_prompt"
              rows="6"
              class="w-full px-4 py-3 rounded-xl bg-white/50 border border-border-default focus:border-accent-primary focus:ring-4 focus:ring-accent-primary/10 outline-none transition-colors resize-y font-mono text-sm leading-relaxed"
              :placeholder="$t(`${tPrefix}.systemPromptPlaceholder` || 'Enter system prompt...')"
            />
          </div>
        </div>

        <div class="flex gap-4 mt-10 pt-6 border-t border-border-default/50">
          <button
            class="flex-1 px-6 py-3.5 rounded-xl font-bold transition-colors bg-white text-text-secondary hover:bg-bg-surface border border-border-default"
            @click="showAddForm = false"
          >
            {{ $t('common.cancel') }}
          </button>
          <button
            class="flex-1 px-6 py-3.5 rounded-xl font-bold transition-[color,background-color,border-color,transform] bg-accent-primary text-white shadow-lg shadow-accent-primary/20 hover:shadow-xl hover:shadow-accent-primary/30 hover:-translate-y-0.5"
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
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import Card from '@/components/ui/Card.vue'
import PageHeaderCard from '@/components/PageHeaderCard.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import MarketplacePagination from '@/components/skills/MarketplacePagination.vue'
import { useAgents } from '@/composables/useAgents'
import { useUIStore } from '@/stores/ui'
import type { Agent, AgentRequest } from '@/types'
import { logger } from '@/utils/logger'

const props = defineProps<{
  module: 'gemini' | 'qwen' | 'qoder' | 'agents' | 'droid'
}>()

const { t } = useI18n()
const router = useRouter()
const uiStore = useUIStore()
const tPrefix = computed(() => props.module === 'agents' ? 'agents' : `${props.module}.agents`)
const isQoderSubagents = computed(() => props.module === 'qoder')
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
const currentPage = ref(1)
const showAddForm = ref(false)
const editingAgent = ref<Agent | null>(null)
const defaultAgentRequest = (): AgentRequest => ({
  name: '',
  model: props.module === 'qoder' ? 'qoder-subagent' : 'claude-sonnet-4-5-20250929',
  tools: [],
  system_prompt: '',
  disabled: false,
})

const formData = ref<AgentRequest>(defaultAgentRequest())
const toolInput = ref('')
const PAGE_SIZE = 20

const moduleNavModule = computed(() => {
  if (props.module === 'agents') return 'claude-code'
  if (props.module === 'gemini') return 'gemini-cli'
  return props.module
})

// Reload agents when module changes
watch(() => props.module, () => {
  loadAgents()
  selectedFolder.value = ''
  searchQuery.value = ''
  currentPage.value = 1
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
  { value: '', label: t(`${tPrefix.value}.folders.all`), icon: 'Folder', count: stats.value.total },
  { value: '__root__', label: t(`${tPrefix.value}.folders.root`), icon: 'Home', count: stats.value.rootCount },
  ...folders.value.map((f) => ({ value: f, label: f, icon: 'Folder', count: stats.value.folderCounts[f] || 0 }))
])

const filteredAgents = computed(() => {
  let filtered = agents.value
  if (selectedFolder.value === '__root__') filtered = agents.value.filter((agent) => !agent.folder || agent.folder === '')
  else if (selectedFolder.value) filtered = agents.value.filter((agent) => agent.folder === selectedFolder.value)
  
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    filtered = filtered.filter((agent) => 
      agent.name.toLowerCase().includes(query) || 
      (agent.description && agent.description.toLowerCase().includes(query)) ||
      (agent.system_prompt && agent.system_prompt.toLowerCase().includes(query)) || 
      (agent.tools && agent.tools.some(tool => tool.toLowerCase().includes(query)))
    )
  }
  return filtered.sort((a, b) => a.name.localeCompare(b.name))
})

const paginatedAgents = computed(() => {
  const start = (currentPage.value - 1) * PAGE_SIZE
  return filteredAgents.value.slice(start, start + PAGE_SIZE)
})

watch([selectedFolder, searchQuery], () => {
  currentPage.value = 1
})

watch(filteredAgents, (agentsOnPage) => {
  const totalPages = Math.max(1, Math.ceil(agentsOnPage.length / PAGE_SIZE))
  if (currentPage.value > totalPages) {
    currentPage.value = totalPages
  }
})

const handleAdd = () => {
  showAddForm.value = true
  editingAgent.value = null
  formData.value = defaultAgentRequest()
  toolInput.value = ''
}

const handleEdit = (agent: Agent) => {
  editingAgent.value = agent
  showAddForm.value = true
  formData.value = { 
    name: agent.name, 
    model: agent.model || (props.module === 'qoder' ? 'qoder-subagent' : 'claude-sonnet-4-5-20250929'),
    description: agent.description,
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

const getAgentApiName = (agent: Agent) => {
  if (props.module !== 'droid') return agent.name
  return agent.folder ? `${agent.folder}/${agent.name}` : agent.name
}

const handleSubmit = async () => {
  if (!formData.value.name || (!isQoderSubagents.value && !formData.value.model)) {
    uiStore.showWarning(t(`${tPrefix.value}.validation.required`))
    return
  }

  const request: AgentRequest = {
    ...formData.value,
    tools: (formData.value.tools && formData.value.tools.length > 0) ? formData.value.tools : undefined,
    system_prompt: formData.value.system_prompt || undefined
  }
  
  try {
    if (editingAgent.value) {
      await updateAgent(getAgentApiName(editingAgent.value), request)
      uiStore.showSuccess(t('common.saveSuccess'))
    } else {
      await addAgent(request)
      uiStore.showSuccess(t(`${tPrefix.value}.addSuccess`))
    }
    showAddForm.value = false
    editingAgent.value = null
  } catch (err) {
    logger.error('Operation failed:', err)
    uiStore.showError(t(`${tPrefix.value}.messages.operationFailed`, { error: err instanceof Error ? err.message : 'Unknown error' }))
  }
}

const handleDelete = async (agent: Agent) => {
  const displayName = getAgentApiName(agent)
  const confirmed = await uiStore.requestConfirm({
    title: t('common.delete'),
    message: t(`${tPrefix.value}.deleteConfirm`, { name: displayName }),
    confirmText: t('common.delete'),
    cancelText: t('common.cancel'),
    type: 'danger'
  })
  if (!confirmed) return
  try {
    await deleteAgent(getAgentApiName(agent))
    uiStore.showSuccess(t(`${tPrefix.value}.deleteSuccess`))
  } catch (err) {
    logger.error('Delete failed:', err)
    uiStore.showError(t(`${tPrefix.value}.messages.deleteFailed`, { error: err instanceof Error ? err.message : 'Unknown error' }))
  }
}

const handleToggle = async (agent: Agent) => {
  try {
    await toggleAgent(getAgentApiName(agent))
    uiStore.showSuccess(
      agent.disabled
        ? t(`${tPrefix.value}.enableSuccess`, { name: agent.name })
        : t(`${tPrefix.value}.disableSuccess`, { name: agent.name })
    )
  } catch (err) {
    logger.error('Toggle failed:', err)
    uiStore.showError(t(`${tPrefix.value}.messages.toggleFailed`, { error: err instanceof Error ? err.message : 'Unknown error' }))
  }
}

const navigateToDetail = (agent: Agent) => {
  // Only navigate to detail for Claude Code agents module
  if (props.module === 'agents') {
    router.push(`/agents/${encodeURIComponent(agent.name)}`)
  } else {
    // For other platforms, open edit modal directly
    handleEdit(agent)
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
  background: rgb(0 0 0 / 10%);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgb(0 0 0 / 20%);
}
</style>
