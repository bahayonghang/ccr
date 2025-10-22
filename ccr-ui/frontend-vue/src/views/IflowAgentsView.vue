<template>
  <div :style="{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }">
    <div class="mb-6">
      <Navbar />
      <StatusHeader :currentConfig="currentConfig" :totalConfigs="totalConfigs" :historyCount="historyCount" />
    </div>

    <div style="display: flex; gap: 0">
      <CollapsibleSidebar />

      <div :style="{ width: '180px', background: 'var(--bg-secondary)', borderRight: '1px solid var(--border-color)', padding: '12px 8px', overflowY: 'auto', height: 'calc(100vh - 40px)', flexShrink: 0 }">
        <h4 :style="{ color: 'var(--text-muted)', fontSize: '11px', fontWeight: '600', marginBottom: '8px', marginLeft: '8px', textTransform: 'uppercase', letterSpacing: '0.5px' }">文件夹</h4>
        <div v-for="folder in folderOptions" :key="folder.value" :style="{ padding: '6px 8px', borderRadius: '6px', cursor: 'pointer', marginBottom: '4px', background: selectedFolder === folder.value ? 'var(--accent-primary)' : 'transparent', color: selectedFolder === folder.value ? '#fff' : 'var(--text-primary)', display: 'flex', alignItems: 'center', gap: '6px', transition: 'all 0.2s', fontSize: '13px' }" @click="selectedFolder = folder.value" @mouseenter="(e) => folder.value !== selectedFolder && (e.currentTarget.style.background = 'var(--bg-tertiary)')" @mouseleave="(e) => folder.value !== selectedFolder && (e.currentTarget.style.background = 'transparent')">
          <component :is="folder.icon" class="w-3.5 h-3.5" />
          <span class="flex-1">{{ folder.label }}</span>
          <span :style="{ fontSize: '11px', opacity: 0.7 }">{{ folder.count }}</span>
        </div>
      </div>

      <div :style="{ flex: 1, minWidth: 0 }">
        <div class="max-w-[1600px] mx-auto">
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-4">
              <h2 class="text-2xl font-bold" :style="{ color: 'var(--text-primary)' }">
                <Bot class="inline-block w-7 h-7 mr-2" />iFlow Agents 管理
              </h2>
              <span class="px-3 py-1 rounded-full text-sm font-medium" :style="{ background: 'var(--accent-primary)', color: '#fff' }">{{ filteredAgents.length }}/{{ stats.total }}</span>
            </div>
            <div class="flex items-center gap-3">
              <RouterLink to="/iflow" class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors" :style="{ background: 'var(--bg-secondary)', color: 'var(--text-secondary)', border: '1px solid var(--border-color)' }">
                <Home class="w-4 h-4" /><span>返回首页</span>
              </RouterLink>
              <button class="px-4 py-2 rounded-lg font-medium transition-all hover:scale-105" :style="{ background: 'var(--accent-primary)', color: '#fff' }" @click="handleAdd">
                <Plus class="inline-block w-5 h-5 mr-2" />添加 Agent
              </button>
            </div>
          </div>

          <div class="mb-4">
            <div class="relative">
              <Search class="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5" :style="{ color: 'var(--text-muted)' }" />
              <input v-model="searchQuery" type="text" placeholder="搜索 agent 名称、系统提示或工具..." class="w-full pl-11 pr-10 py-3 rounded-lg transition-all focus:outline-none focus:ring-2" :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }" />
              <button v-if="searchQuery" class="absolute right-3 top-1/2 transform -translate-y-1/2 p-1 rounded-lg hover:bg-opacity-20 transition-all" :style="{ color: 'var(--text-muted)' }" @click="searchQuery = ''">
                <X class="w-4 h-4" />
              </button>
            </div>
            <p v-if="searchQuery" class="mt-2 text-sm" :style="{ color: 'var(--text-muted)' }">找到 <span :style="{ color: 'var(--accent-primary)', fontWeight: 'bold' }">{{ filteredAgents.length }}</span> 个匹配的 agents</p>
          </div>

          <div class="space-y-4">
            <div v-if="loading" class="text-center py-10" :style="{ color: 'var(--text-muted)' }">加载中...</div>
            <div v-else-if="agents.length === 0" class="text-center py-10" :style="{ color: 'var(--text-muted)' }">暂无 Agents 配置</div>
            <div v-else-if="filteredAgents.length === 0" class="text-center py-10" :style="{ color: 'var(--text-muted)' }">
              <Search class="w-12 h-12 mx-auto mb-3 opacity-50" /><p>未找到匹配的 agents</p><p class="text-sm mt-2">尝试使用其他关键词搜索或切换文件夹</p>
            </div>
            <div v-else v-for="agent in filteredAgents" :key="agent.name" class="group p-6 rounded-xl transition-all duration-300" :style="{ background: 'rgba(255, 255, 255, 0.9)', border: '1px solid rgba(99, 102, 241, 0.12)', outline: 'none', cursor: 'default' }" @mouseenter="(e) => onCardHover(e.currentTarget, true)" @mouseleave="(e) => onCardHover(e.currentTarget, false)">
              <div class="flex items-start justify-between">
                <div class="flex-1">
                  <div class="flex items-center gap-3 mb-2">
                    <h3 class="text-xl font-semibold" :style="{ color: 'var(--text-primary)' }">{{ agent.name }}</h3>
                    <span v-if="agent.folder" class="px-2 py-1 rounded text-xs font-medium" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-muted)' }">📁 {{ agent.folder }}</span>
                    <span v-if="agent.disabled" class="px-2 py-1 rounded text-xs font-medium" :style="{ background: '#fef3c7', color: '#92400e' }">已禁用</span>
                  </div>
                  <p class="mb-2" :style="{ color: 'var(--text-secondary)', fontSize: '14px' }"><strong>Model:</strong> {{ agent.model }}</p>
                  <p v-if="agent.tools && agent.tools.length > 0" class="mb-2 text-sm" :style="{ color: 'var(--text-muted)' }"><strong>Tools:</strong> {{ agent.tools.join(', ') }}</p>
                  <p v-if="agent.system_prompt" class="text-sm italic" :style="{ color: 'var(--text-muted)' }">{{ agent.system_prompt }}</p>
                </div>
                <div class="flex gap-2 ml-4">
                  <button class="p-2 rounded-lg transition-all hover:scale-110" :style="{ background: agent.disabled ? '#fef3c7' : '#d1fae5', color: agent.disabled ? '#92400e' : '#065f46' }" :title="agent.disabled ? '启用' : '禁用'" @click="handleToggle(agent.name)">
                    <PowerOff v-if="agent.disabled" class="w-5 h-5" /><Power v-else class="w-5 h-5" />
                  </button>
                  <button class="p-2 rounded-lg transition-all hover:scale-110" :style="{ background: 'var(--bg-tertiary)', color: 'var(--accent-primary)' }" title="编辑" @click="handleEdit(agent)">
                    <Edit2 class="w-5 h-5" />
                  </button>
                  <button class="p-2 rounded-lg transition-all hover:scale-110" :style="{ background: '#fee2e2', color: '#991b1b' }" title="删除" @click="handleDelete(agent.name)">
                    <Trash2 class="w-5 h-5" />
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="showAddForm" class="fixed inset-0 flex items-center justify-center z-50" :style="{ background: 'rgba(0, 0, 0, 0.5)' }" @click="showAddForm = false">
      <div class="p-8 rounded-2xl w-full max-w-2xl max-h-[80vh] overflow-y-auto" :style="{ background: 'var(--bg-secondary)' }" @click.stop>
        <h3 class="text-2xl font-bold mb-6" :style="{ color: 'var(--text-primary)' }">{{ editingAgent ? '编辑 Agent' : '添加 Agent' }}</h3>
        <div class="space-y-4">
          <div><label class="block mb-2 font-medium" :style="{ color: 'var(--text-secondary)' }">名称 *</label>
            <input v-model="formData.name" type="text" class="w-full px-4 py-2 rounded-lg focus:outline-none focus:ring-2" :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }" />
          </div>
          <div><label class="block mb-2 font-medium" :style="{ color: 'var(--text-secondary)' }">Model *</label>
            <select v-model="formData.model" class="w-full px-4 py-2 rounded-lg focus:outline-none focus:ring-2" :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }">
              <option value="claude-sonnet-4-5-20250929">Claude Sonnet 4.5</option>
              <option value="claude-opus-4-20250514">Claude Opus 4</option>
              <option value="claude-3-5-sonnet-20241022">Claude 3.5 Sonnet</option>
            </select>
          </div>
          <div><label class="block mb-2 font-medium" :style="{ color: 'var(--text-secondary)' }">Tools</label>
            <div class="flex gap-2 mb-2">
              <input v-model="toolInput" type="text" placeholder="输入工具名称" class="flex-1 px-4 py-2 rounded-lg focus:outline-none focus:ring-2" :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }" @keyup.enter="addTool" />
              <button class="px-4 py-2 rounded-lg font-medium text-white" :style="{ background: 'var(--accent-primary)' }" @click="addTool">添加</button>
            </div>
            <div class="flex flex-wrap gap-2">
              <span v-for="tool in formData.tools" :key="tool" class="px-3 py-1 rounded-full text-sm flex items-center gap-2" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)' }">
                {{ tool }}
                <button @click="removeTool(tool)"><X class="w-3 h-3" /></button>
              </span>
            </div>
          </div>
          <div><label class="block mb-2 font-medium" :style="{ color: 'var(--text-secondary)' }">System Prompt</label>
            <textarea v-model="formData.system_prompt" rows="6" class="w-full px-4 py-2 rounded-lg focus:outline-none focus:ring-2" :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }" />
          </div>
        </div>
        <div class="flex gap-3 mt-6">
          <button class="flex-1 px-6 py-3 rounded-lg font-medium transition-all hover:scale-105" :style="{ background: 'var(--accent-primary)', color: '#fff' }" @click="handleSubmit">{{ editingAgent ? '保存' : '添加' }}</button>
          <button class="flex-1 px-6 py-3 rounded-lg font-medium transition-all hover:scale-105" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }" @click="showAddForm = false">取消</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import { Bot, Plus, Edit2, Trash2, Power, PowerOff, Search, X, Folder, Home } from 'lucide-vue-next'
import { listIflowAgents, addIflowAgent, updateIflowAgent, deleteIflowAgent, toggleIflowAgent, listConfigs, getHistory } from '@/api/client'
import type { Agent, AgentRequest } from '@/types'
import Navbar from '@/components/Navbar.vue'
import StatusHeader from '@/components/StatusHeader.vue'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'

const agents = ref<Agent[]>([])
const folders = ref<string[]>([])
const loading = ref(true)
const currentConfig = ref('')
const totalConfigs = ref(0)
const historyCount = ref(0)
const selectedFolder = ref('')
const searchQuery = ref('')
const showAddForm = ref(false)
const editingAgent = ref<Agent | null>(null)
const formData = ref<AgentRequest>({ name: '', model: 'claude-sonnet-4-5-20250929', tools: [], system_prompt: '', disabled: false })
const toolInput = ref('')

const stats = computed(() => {
  const rootCount = agents.value.filter((a) => !a.folder || a.folder === '').length
  const folderCounts: Record<string, number> = {}
  folders.value.forEach((f) => { folderCounts[f] = agents.value.filter((a) => a.folder === f).length })
  return { rootCount, folderCounts, total: agents.value.length }
})

const folderOptions = computed(() => [
  { value: '', label: '全部', icon: Folder, count: stats.value.total },
  { value: '__root__', label: '根目录', icon: Home, count: stats.value.rootCount },
  ...folders.value.map((f) => ({ value: f, label: f, icon: Folder, count: stats.value.folderCounts[f] || 0 }))
])

const filteredAgents = computed(() => {
  let filtered = agents.value
  if (selectedFolder.value === '__root__') filtered = agents.value.filter((agent) => !agent.folder || agent.folder === '')
  else if (selectedFolder.value) filtered = agents.value.filter((agent) => agent.folder === selectedFolder.value)
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    filtered = filtered.filter((agent) => agent.name.toLowerCase().includes(query) || (agent.system_prompt && agent.system_prompt.toLowerCase().includes(query)) || (agent.tools && agent.tools.some(tool => tool.toLowerCase().includes(query))))
  }
  return filtered.sort((a, b) => a.name.localeCompare(b.name))
})

const loadAgents = async () => {
  try {
    loading.value = true
    const data = await listIflowAgents()
    agents.value = data.agents
    folders.value = data.folders || []
    try {
      const configData = await listConfigs()
      currentConfig.value = configData.current_config
      totalConfigs.value = configData.configs.length
      const historyData = await getHistory()
      historyCount.value = historyData.total
    } catch (err) { console.error('Failed to load system info:', err) }
  } catch (err) {
    console.error('Failed to load agents:', err)
    alert('加载 Agents 失败')
  } finally { loading.value = false }
}

onMounted(() => { loadAgents() })

const handleAdd = () => {
  showAddForm.value = true
  editingAgent.value = null
  formData.value = { name: '', model: 'claude-sonnet-4-5-20250929', tools: [], system_prompt: '', disabled: false }
  toolInput.value = ''
}

const handleEdit = (agent: Agent) => {
  editingAgent.value = agent
  showAddForm.value = true
  formData.value = { name: agent.name, model: agent.model, tools: agent.tools || [], system_prompt: agent.system_prompt || '', disabled: agent.disabled || false }
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
  if (!formData.value.name || !formData.value.model) { alert('请填写必填字段'); return }
  const request: AgentRequest = { ...formData.value, tools: formData.value.tools && formData.value.tools.length > 0 ? formData.value.tools : undefined, system_prompt: formData.value.system_prompt || undefined }
  try {
    if (editingAgent.value) await updateIflowAgent(editingAgent.value.name, request)
    else await addIflowAgent(request)
    showAddForm.value = false
    editingAgent.value = null
    loadAgents()
  } catch (err) { console.error('操作失败:', err); alert('操作失败') }
}

const handleDelete = async (name: string) => {
  if (!confirm(`确定要删除 agent "${name}" 吗？`)) return
  try { await deleteIflowAgent(name); loadAgents() } catch (err) { console.error('删除失败:', err); alert('删除失败') }
}

const handleToggle = async (name: string) => {
  try { await toggleIflowAgent(name); loadAgents() } catch (err) { console.error('切换状态失败:', err); alert('切换状态失败') }
}

const onCardHover = (el: HTMLElement, hover: boolean) => {
  if (hover) {
    el.style.background = 'rgba(255, 255, 255, 0.95)'
    el.style.borderColor = 'rgba(99, 102, 241, 0.24)'
    el.style.boxShadow = '0 10px 15px -3px rgba(0, 0, 0, 0.08), 0 4px 6px -4px rgba(0, 0, 0, 0.08)'
    el.style.transform = 'translateY(-2px)'
  } else {
    el.style.background = 'rgba(255, 255, 255, 0.9)'
    el.style.borderColor = 'rgba(99, 102, 241, 0.12)'
    el.style.boxShadow = 'none'
    el.style.transform = 'translateY(0)'
  }
}
</script>
