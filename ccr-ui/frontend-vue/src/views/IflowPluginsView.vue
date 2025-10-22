<template>
  <div :style="{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }">
    <div class="max-w-[1800px] mx-auto">
      <Navbar />
      <StatusHeader :currentConfig="currentConfig" :totalConfigs="totalConfigs" :historyCount="historyCount" />

      <div class="grid grid-cols-[auto_1fr] gap-4">
        <CollapsibleSidebar />

        <main class="rounded-xl p-6 glass-effect" :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }">
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-3">
              <Puzzle class="w-6 h-6" :style="{ color: 'var(--accent-primary)' }" />
              <h1 class="text-2xl font-bold" :style="{ color: 'var(--text-primary)' }">iFlow 插件管理</h1>
              <span class="px-3 py-1 rounded-full text-sm font-medium" :style="{ background: 'var(--accent-primary)', color: '#fff' }">{{ plugins.length }}</span>
            </div>
            <div class="flex items-center gap-3">
              <RouterLink to="/iflow" class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors" :style="{ background: 'var(--bg-secondary)', color: 'var(--text-secondary)', border: '1px solid var(--border-color)' }">
                <Home class="w-4 h-4" /><span>返回首页</span>
              </RouterLink>
              <button class="px-4 py-2 rounded-lg font-semibold text-sm text-white flex items-center gap-2" :style="{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))', boxShadow: '0 0 20px var(--glow-primary)' }" @click="handleAdd">
                <Plus class="w-4 h-4" />添加插件
              </button>
            </div>
          </div>

          <div v-if="loading" class="flex justify-center py-20">
            <div class="w-12 h-12 rounded-full border-4 border-transparent animate-spin" :style="{ borderTopColor: 'var(--accent-primary)', borderRightColor: 'var(--accent-secondary)' }" />
          </div>

          <div v-else-if="!plugins || plugins.length === 0" class="text-center py-10" :style="{ color: 'var(--text-muted)' }">暂无插件配置</div>

          <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <div v-for="plugin in plugins" :key="plugin.id" class="group rounded-lg p-5 transition-all duration-300" :style="{ background: 'rgba(255, 255, 255, 0.7)', border: '1px solid rgba(99, 102, 241, 0.12)', outline: 'none', cursor: 'default' }" @mouseenter="(e) => onCardHover(e.currentTarget, true)" @mouseleave="(e) => onCardHover(e.currentTarget, false)">
              <div class="flex items-start justify-between mb-3">
                <div class="flex-1">
                  <h3 class="text-lg font-bold" :style="{ color: 'var(--text-primary)' }">{{ plugin.name }}</h3>
                  <p class="text-xs font-mono mt-1" :style="{ color: 'var(--text-muted)' }">ID: {{ plugin.id }}</p>
                </div>
                <span v-if="!plugin.enabled" class="px-2 py-0.5 rounded text-xs font-semibold uppercase" :style="{ background: 'var(--accent-danger)', color: 'white' }">已禁用</span>
              </div>

              <div class="mb-4">
                <p class="text-sm" :style="{ color: 'var(--text-secondary)' }"><strong>Version:</strong> {{ plugin.version }}</p>
                <p v-if="plugin.config" class="text-xs mt-2 font-mono p-2 rounded overflow-auto max-h-32" :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }">{{ JSON.stringify(plugin.config, null, 2) }}</p>
              </div>

              <div class="flex gap-2">
                <button class="flex-1 p-2 rounded-lg transition-all hover:scale-105 flex items-center justify-center gap-1 text-sm font-medium" :style="{ background: plugin.enabled ? '#fef3c7' : '#d1fae5', color: plugin.enabled ? '#92400e' : '#065f46' }" :title="plugin.enabled ? '禁用' : '启用'" @click="handleToggle(plugin.id)">
                  <PowerOff v-if="!plugin.enabled" class="w-4 h-4" /><Power v-else class="w-4 h-4" />
                  <span>{{ plugin.enabled ? '禁用' : '启用' }}</span>
                </button>
                <button class="p-2 rounded-lg transition-all hover:scale-110" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-primary)' }" title="编辑" @click="handleEdit(plugin)">
                  <Edit2 class="w-4 h-4" />
                </button>
                <button class="p-2 rounded-lg transition-all hover:scale-110" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-danger)' }" title="删除" @click="handleDelete(plugin.id)">
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>

          <div v-if="showAddForm" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
            <div class="rounded-xl p-6 max-w-2xl w-full max-h-[90vh] overflow-y-auto" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)' }">
              <h2 class="text-xl font-bold mb-4" :style="{ color: 'var(--text-primary)' }">{{ editingPlugin ? '编辑插件' : '添加插件' }}</h2>

              <div class="space-y-4">
                <div>
                  <label class="block text-sm font-semibold mb-1" :style="{ color: 'var(--text-secondary)' }">插件 ID *</label>
                  <input v-model="formData.id" type="text" class="w-full px-3 py-2 rounded-lg" :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }" placeholder="例如: my-plugin" />
                </div>

                <div>
                  <label class="block text-sm font-semibold mb-1" :style="{ color: 'var(--text-secondary)' }">插件名称 *</label>
                  <input v-model="formData.name" type="text" class="w-full px-3 py-2 rounded-lg" :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }" placeholder="例如: My Plugin" />
                </div>

                <div>
                  <label class="block text-sm font-semibold mb-1" :style="{ color: 'var(--text-secondary)' }">版本 *</label>
                  <input v-model="formData.version" type="text" class="w-full px-3 py-2 rounded-lg" :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }" placeholder="例如: 1.0.0" />
                </div>

                <div>
                  <label class="block text-sm font-semibold mb-1" :style="{ color: 'var(--text-secondary)' }">配置 JSON</label>
                  <textarea v-model="configJson" rows="10" class="w-full px-3 py-2 rounded-lg font-mono text-sm" :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }" placeholder='{"key": "value"}' />
                  <div class="text-xs mt-1" :style="{ color: 'var(--text-muted)' }">可选，输入合法的 JSON 格式配置</div>
                </div>

                <div class="flex items-center gap-2">
                  <input id="enabled" v-model="formData.enabled" type="checkbox" class="w-4 h-4" />
                  <label for="enabled" class="text-sm" :style="{ color: 'var(--text-secondary)' }">启用此插件</label>
                </div>
              </div>

              <div class="flex gap-3 mt-6">
                <button class="flex-1 px-4 py-2 rounded-lg font-semibold text-white" :style="{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))' }" @click="handleSubmit">{{ editingPlugin ? '更新' : '添加' }}</button>
                <button class="flex-1 px-4 py-2 rounded-lg font-semibold" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)' }" @click="showAddForm = false">取消</button>
              </div>
            </div>
          </div>
        </main>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import { Puzzle, Plus, Edit2, Trash2, Power, PowerOff, Home } from 'lucide-vue-next'
import { listIflowPlugins, addIflowPlugin, updateIflowPlugin, deleteIflowPlugin, toggleIflowPlugin, listConfigs, getHistory } from '@/api/client'
import type { Plugin as PluginType, PluginRequest } from '@/types'
import Navbar from '@/components/Navbar.vue'
import StatusHeader from '@/components/StatusHeader.vue'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'

const plugins = ref<PluginType[]>([])
const loading = ref(true)
const currentConfig = ref<string>('')
const totalConfigs = ref(0)
const historyCount = ref(0)
const showAddForm = ref(false)
const editingPlugin = ref<PluginType | null>(null)
const formData = ref<PluginRequest>({ id: '', name: '', version: '', enabled: true, config: undefined })
const configJson = ref('')

const loadPlugins = async () => {
  try {
    loading.value = true
    const data = await listIflowPlugins()
    plugins.value = data || []

    try {
      const configData = await listConfigs()
      currentConfig.value = configData.current_config
      totalConfigs.value = configData.configs.length

      const historyData = await getHistory()
      historyCount.value = historyData.total
    } catch (err) {
      console.error('Failed to load system info:', err)
    }
  } catch (err) {
    console.error('Failed to load plugins:', err)
    plugins.value = []
    alert('加载插件失败')
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadPlugins()
})

const handleAdd = () => {
  showAddForm.value = true
  editingPlugin.value = null
  formData.value = { id: '', name: '', version: '1.0.0', enabled: true, config: undefined }
  configJson.value = ''
}

const handleEdit = (plugin: PluginType) => {
  editingPlugin.value = plugin
  showAddForm.value = true
  formData.value = { id: plugin.id, name: plugin.name, version: plugin.version, enabled: plugin.enabled, config: plugin.config }
  configJson.value = plugin.config ? JSON.stringify(plugin.config, null, 2) : ''
}

const handleSubmit = async () => {
  if (!formData.value.id || !formData.value.name || !formData.value.version) {
    alert('请填写必填字段')
    return
  }

  let config = undefined
  if (configJson.value.trim()) {
    try {
      config = JSON.parse(configJson.value)
    } catch (err) {
      alert('配置 JSON 格式错误')
      return
    }
  }

  const request: PluginRequest = { ...formData.value, config }

  try {
    if (editingPlugin.value) {
      await updateIflowPlugin(editingPlugin.value.id, request)
      alert('✓ 插件更新成功')
    } else {
      await addIflowPlugin(request)
      alert('✓ 插件添加成功')
    }
    showAddForm.value = false
    await loadPlugins()
  } catch (err) {
    alert(`操作失败: ${err instanceof Error ? err.message : 'Unknown error'}`)
  }
}

const handleDelete = async (id: string) => {
  if (!confirm(`确定删除插件 "${id}" 吗？`)) return

  try {
    await deleteIflowPlugin(id)
    alert('✓ 插件删除成功')
    await loadPlugins()
  } catch (err) {
    alert(`删除失败: ${err instanceof Error ? err.message : 'Unknown error'}`)
  }
}

const handleToggle = async (id: string) => {
  try {
    await toggleIflowPlugin(id)
    await loadPlugins()
  } catch (err) {
    alert(`切换失败: ${err instanceof Error ? err.message : 'Unknown error'}`)
  }
}

const onCardHover = (el: HTMLElement, hover: boolean) => {
  if (hover) {
    el.style.background = 'rgba(255, 255, 255, 0.9)'
    el.style.borderColor = 'rgba(99, 102, 241, 0.24)'
    el.style.boxShadow = '0 4px 6px -1px rgba(0, 0, 0, 0.08), 0 2px 4px -2px rgba(0, 0, 0, 0.08)'
    el.style.transform = 'translateY(-2px)'
  } else {
    el.style.background = 'rgba(255, 255, 255, 0.7)'
    el.style.borderColor = 'rgba(99, 102, 241, 0.12)'
    el.style.boxShadow = 'none'
    el.style.transform = 'translateY(0)'
  }
}
</script>
