<template>
  <div :style="{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }">
    <div class="max-w-[1800px] mx-auto">
      <Navbar />
      <StatusHeader
        :currentConfig="currentConfig"
        :totalConfigs="totalConfigs"
        :historyCount="historyCount"
      />

      <div class="grid grid-cols-[auto_1fr] gap-4">
        <CollapsibleSidebar />

        <main
          class="rounded-xl p-6 glass-effect"
          :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
        >
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-3">
              <Server class="w-6 h-6" :style="{ color: 'var(--accent-primary)' }" />
              <h1 class="text-2xl font-bold" :style="{ color: 'var(--text-primary)' }">
                MCP 服务器管理
              </h1>
            </div>
            <div class="flex items-center gap-3">
              <RouterLink
                to="/"
                class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors"
                :style="{
                  background: 'var(--bg-secondary)',
                  color: 'var(--text-secondary)',
                  border: '1px solid var(--border-color)'
                }"
              >
                <Home class="w-4 h-4" />
                <span>返回首页</span>
              </RouterLink>
              <button
                class="px-4 py-2 rounded-lg font-semibold text-sm text-white flex items-center gap-2"
                :style="{
                  background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
                  boxShadow: '0 0 20px var(--glow-primary)'
                }"
                @click="handleAdd"
              >
                <Plus class="w-4 h-4" />
                添加 MCP 服务器
              </button>
            </div>
          </div>

          <div v-if="loading" class="flex justify-center py-20">
            <div
              class="w-12 h-12 rounded-full border-4 border-transparent animate-spin"
              :style="{
                borderTopColor: 'var(--accent-primary)',
                borderRightColor: 'var(--accent-secondary)'
              }"
            />
          </div>

          <div v-else class="space-y-3">
            <div
              v-if="!servers || servers.length === 0"
              class="text-center py-10"
              :style="{ color: 'var(--text-muted)' }"
            >
              暂无 MCP 服务器配置
            </div>

            <div
              v-for="server in servers"
              :key="server.name"
              class="group rounded-lg p-4 transition-all duration-300"
              :style="{
                background: 'rgba(255, 255, 255, 0.7)',
                border: '1px solid rgba(99, 102, 241, 0.12)',
                outline: 'none',
                cursor: 'default'
              }"
              @mouseenter="(e) => onCardHover(e.currentTarget, true)"
              @mouseleave="(e) => onCardHover(e.currentTarget, false)"
            >
              <div class="flex items-start justify-between">
                <div class="flex-1">
                  <div class="flex items-center gap-2 mb-2">
                    <h3 class="text-lg font-bold font-mono" :style="{ color: 'var(--text-primary)' }">
                      {{ server.name }}
                    </h3>
                    <span
                      v-if="server.disabled"
                      class="px-2 py-0.5 rounded text-xs font-semibold uppercase"
                      :style="{ background: 'var(--accent-danger)', color: 'white' }"
                    >
                      已禁用
                    </span>
                  </div>
                  <div class="space-y-2 text-sm">
                    <div>
                      <span :style="{ color: 'var(--text-muted)' }">命令:</span>
                      <code
                        class="ml-2 px-2 py-1 rounded font-mono"
                        :style="{ background: 'var(--bg-secondary)', color: 'var(--accent-primary)' }"
                      >
                        {{ server.command }}
                      </code>
                    </div>
                    <div>
                      <span :style="{ color: 'var(--text-muted)' }">参数:</span>
                      <code
                        class="ml-2 px-2 py-1 rounded font-mono"
                        :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }"
                      >
                        {{ server.args.join(' ') }}
                      </code>
                    </div>
                    <div v-if="server.env && Object.keys(server.env).length > 0">
                      <span :style="{ color: 'var(--text-muted)' }">环境变量:</span>
                      <div class="ml-2 mt-1 space-y-1">
                        <div
                          v-for="[key, value] in Object.entries(server.env)"
                          :key="key"
                          class="text-xs font-mono px-2 py-1 rounded"
                          :style="{ background: 'var(--bg-secondary)' }"
                        >
                          <span :style="{ color: 'var(--accent-secondary)' }">{{ key }}</span>=<span
                            :style="{ color: 'var(--text-primary)' }"
                            >{{ value }}</span
                          >
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
                <div class="flex gap-2">
                  <button
                    class="p-2 rounded-lg transition-all hover:scale-110"
                    :style="{
                      background: 'var(--bg-secondary)',
                      border: '1px solid var(--border-color)',
                      color: server.disabled ? 'var(--accent-success)' : 'var(--accent-warning)'
                    }"
                    :title="server.disabled ? '启用' : '禁用'"
                    @click="handleToggle(server.name)"
                  >
                    <Power v-if="server.disabled" class="w-4 h-4" />
                    <PowerOff v-else class="w-4 h-4" />
                  </button>
                  <button
                    class="p-2 rounded-lg transition-all hover:scale-110"
                    :style="{
                      background: 'var(--bg-secondary)',
                      border: '1px solid var(--border-color)',
                      color: 'var(--accent-primary)'
                    }"
                    title="编辑"
                    @click="handleEdit(server)"
                  >
                    <Edit2 class="w-4 h-4" />
                  </button>
                  <button
                    class="p-2 rounded-lg transition-all hover:scale-110"
                    :style="{
                      background: 'var(--bg-secondary)',
                      border: '1px solid var(--border-color)',
                      color: 'var(--accent-danger)'
                    }"
                    title="删除"
                    @click="handleDelete(server.name)"
                  >
                    <Trash2 class="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Add/Edit Form Modal -->
          <div
            v-if="showAddForm"
            class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50"
          >
            <div
              class="rounded-xl p-6 max-w-2xl w-full max-h-[90vh] overflow-y-auto"
              :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)' }"
            >
              <h2 class="text-xl font-bold mb-4" :style="{ color: 'var(--text-primary)' }">
                {{ editingServer ? '编辑 MCP 服务器' : '添加 MCP 服务器' }}
              </h2>

              <div class="space-y-4">
                <div>
                  <label class="block text-sm font-semibold mb-1" :style="{ color: 'var(--text-secondary)' }">
                    服务器名称 *
                  </label>
                  <input
                    v-model="formData.name"
                    type="text"
                    class="w-full px-3 py-2 rounded-lg"
                    :style="{
                      background: 'var(--bg-tertiary)',
                      border: '1px solid var(--border-color)',
                      color: 'var(--text-primary)'
                    }"
                    placeholder="例如: filesystem-server"
                  />
                </div>

                <div>
                  <label class="block text-sm font-semibold mb-1" :style="{ color: 'var(--text-secondary)' }">
                    命令 *
                  </label>
                  <input
                    v-model="formData.command"
                    type="text"
                    class="w-full px-3 py-2 rounded-lg font-mono"
                    :style="{
                      background: 'var(--bg-tertiary)',
                      border: '1px solid var(--border-color)',
                      color: 'var(--text-primary)'
                    }"
                    placeholder="例如: npx 或 node"
                  />
                </div>

                <div>
                  <label class="block text-sm font-semibold mb-1" :style="{ color: 'var(--text-secondary)' }">
                    参数 *
                  </label>
                  <input
                    v-model="argInput"
                    type="text"
                    class="w-full px-3 py-2 rounded-lg font-mono"
                    :style="{
                      background: 'var(--bg-tertiary)',
                      border: '1px solid var(--border-color)',
                      color: 'var(--text-primary)'
                    }"
                    placeholder="例如: -y @modelcontextprotocol/server-filesystem /path"
                  />
                  <div class="text-xs mt-1" :style="{ color: 'var(--text-muted)' }">用空格分隔多个参数</div>
                </div>

                <div>
                  <label class="block text-sm font-semibold mb-1" :style="{ color: 'var(--text-secondary)' }">
                    环境变量
                  </label>
                  <div class="flex gap-2 mb-2">
                    <input
                      v-model="envKey"
                      type="text"
                      class="flex-1 px-3 py-2 rounded-lg font-mono"
                      :style="{
                        background: 'var(--bg-tertiary)',
                        border: '1px solid var(--border-color)',
                        color: 'var(--text-primary)'
                      }"
                      placeholder="KEY"
                    />
                    <input
                      v-model="envValue"
                      type="text"
                      class="flex-1 px-3 py-2 rounded-lg font-mono"
                      :style="{
                        background: 'var(--bg-tertiary)',
                        border: '1px solid var(--border-color)',
                        color: 'var(--text-primary)'
                      }"
                      placeholder="VALUE"
                    />
                    <button
                      class="px-4 py-2 rounded-lg font-semibold text-sm text-white"
                      :style="{ background: 'var(--accent-primary)' }"
                      @click="addEnvVar"
                    >
                      添加
                    </button>
                  </div>
                  <div class="space-y-1">
                    <div
                      v-for="[key, value] in Object.entries(formData.env || {})"
                      :key="key"
                      class="flex items-center justify-between px-3 py-2 rounded"
                      :style="{ background: 'var(--bg-secondary)' }"
                    >
                      <code class="text-sm font-mono" :style="{ color: 'var(--text-primary)' }">
                        {{ key }}={{ value }}
                      </code>
                      <button class="text-xs" :style="{ color: 'var(--accent-danger)' }" @click="removeEnvVar(key)">
                        删除
                      </button>
                    </div>
                  </div>
                </div>

                <div class="flex items-center gap-2">
                  <input
                    id="disabled"
                    v-model="formData.disabled"
                    type="checkbox"
                    class="w-4 h-4"
                  />
                  <label for="disabled" class="text-sm" :style="{ color: 'var(--text-secondary)' }">
                    禁用此服务器
                  </label>
                </div>
              </div>

              <div class="flex gap-3 mt-6">
                <button
                  class="flex-1 px-4 py-2 rounded-lg font-semibold text-white"
                  :style="{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))' }"
                  @click="handleSubmit"
                >
                  {{ editingServer ? '更新' : '添加' }}
                </button>
                <button
                  class="flex-1 px-4 py-2 rounded-lg font-semibold"
                  :style="{
                    background: 'var(--bg-tertiary)',
                    color: 'var(--text-primary)',
                    border: '1px solid var(--border-color)'
                  }"
                  @click="showAddForm = false"
                >
                  取消
                </button>
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
import { Server, Plus, Edit2, Trash2, Power, PowerOff, Home } from 'lucide-vue-next'
import {
  listMcpServers,
  addMcpServer,
  updateMcpServer,
  deleteMcpServer,
  toggleMcpServer,
  listConfigs,
  getHistory
} from '@/api/client'
import type { McpServer, McpServerRequest } from '@/types'
import Navbar from '@/components/Navbar.vue'
import StatusHeader from '@/components/StatusHeader.vue'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'

const servers = ref<McpServer[]>([])
const loading = ref(true)
const currentConfig = ref<string>('')
const totalConfigs = ref(0)
const historyCount = ref(0)
const showAddForm = ref(false)
const editingServer = ref<McpServer | null>(null)
const formData = ref<McpServerRequest>({
  name: '',
  command: '',
  args: [],
  env: {},
  disabled: false
})
const argInput = ref('')
const envKey = ref('')
const envValue = ref('')

const loadServers = async () => {
  try {
    loading.value = true
    const data = await listMcpServers()
    servers.value = data || []

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
    console.error('Failed to load MCP servers:', err)
    servers.value = []
    alert('加载 MCP 服务器失败')
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadServers()
})

const handleAdd = () => {
  showAddForm.value = true
  editingServer.value = null
  formData.value = {
    name: '',
    command: 'npx',
    args: [],
    env: {},
    disabled: false
  }
  argInput.value = ''
}

const handleEdit = (server: McpServer) => {
  editingServer.value = server
  showAddForm.value = true
  formData.value = {
    name: server.name,
    command: server.command,
    args: server.args,
    env: server.env || {},
    disabled: server.disabled || false
  }
  argInput.value = server.args.join(' ')
}

const handleSubmit = async () => {
  if (!formData.value.name || !formData.value.command) {
    alert('请填写必填字段')
    return
  }

  const args = argInput.value
    .split(' ')
    .filter((a) => a.trim())
  const request: McpServerRequest = {
    ...formData.value,
    args
  }

  try {
    if (editingServer.value) {
      await updateMcpServer(editingServer.value.name, request)
      alert('✓ MCP 服务器更新成功')
    } else {
      await addMcpServer(request)
      alert('✓ MCP 服务器添加成功')
    }
    showAddForm.value = false
    await loadServers()
  } catch (err) {
    alert(`操作失败: ${err instanceof Error ? err.message : 'Unknown error'}`)
  }
}

const handleDelete = async (name: string) => {
  if (!confirm(`确定删除 MCP 服务器 "${name}" 吗？`)) return

  try {
    await deleteMcpServer(name)
    alert('✓ MCP 服务器删除成功')
    await loadServers()
  } catch (err) {
    alert(`删除失败: ${err instanceof Error ? err.message : 'Unknown error'}`)
  }
}

const handleToggle = async (name: string) => {
  try {
    await toggleMcpServer(name)
    await loadServers()
  } catch (err) {
    alert(`切换失败: ${err instanceof Error ? err.message : 'Unknown error'}`)
  }
}

const addEnvVar = () => {
  if (envKey.value && envValue.value) {
    formData.value.env = { ...formData.value.env, [envKey.value]: envValue.value }
    envKey.value = ''
    envValue.value = ''
  }
}

const removeEnvVar = (key: string) => {
  const newEnv = { ...formData.value.env }
  delete newEnv[key]
  formData.value.env = newEnv
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
