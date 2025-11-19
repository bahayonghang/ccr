<template>
  <div class="min-h-screen relative">
    <!-- 🎨 液态玻璃背景 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-10 right-10 w-[600px] h-[600px] rounded-full opacity-20 blur-3xl animate-pulse"
        :style="{ 
          background: 'linear-gradient(135deg, var(--accent-primary) 0%, var(--accent-secondary) 50%, var(--accent-tertiary) 100%)',
          animation: 'pulse 8s ease-in-out infinite'
        }"
      />
      <div
        class="absolute bottom-10 left-10 w-[500px] h-[500px] rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, var(--accent-success) 0%, var(--accent-info) 50%, var(--accent-primary) 100%)',
          animation: 'pulse 10s ease-in-out infinite',
          animationDelay: '2s'
        }"
      />
    </div>
    <div class="relative z-10 p-6">
      <div class="max-w-[1800px] mx-auto">
        <Navbar />
        <StatusHeader
          :current-config="currentConfig"
          :total-configs="totalConfigs"
          :history-count="historyCount"
        />

        <!-- Breadcrumb Navigation -->
        <Breadcrumb
          :items="[
            { label: $t('mcp.breadcrumb.home'), path: '/', icon: Home },
            { label: $t('mcp.breadcrumb.claudeCode'), path: '/claude-code', icon: Code2 },
            { label: $t('mcp.breadcrumb.mcp'), path: '/mcp', icon: Server }
          ]"
          module-color="#6366f1"
        />

        <div class="grid grid-cols-[auto_1fr] gap-6">
          <CollapsibleSidebar module="claude-code" />

          <main
            class="p-8 transition-all duration-300"
            :style="{
              background: 'rgba(255, 255, 255, 0.6)',
              backdropFilter: 'blur(20px) saturate(180%)',
              WebkitBackdropFilter: 'blur(20px) saturate(180%)',
              border: '1px solid rgba(255, 255, 255, 0.3)',
              borderRadius: '24px',
              boxShadow: '0 8px 32px rgba(0, 0, 0, 0.08), inset 0 1px 0 0 rgba(255, 255, 255, 0.5)'
            }"
          >
            <div class="flex items-center justify-between mb-8">
              <div class="flex items-center gap-4">
                <div
                  class="p-4 rounded-2xl"
                  :style="{ background: 'rgba(99, 102, 241, 0.15)' }"
                >
                  <Server
                    class="w-8 h-8"
                    :style="{ color: 'var(--accent-primary)' }"
                  />
                </div>
                <div>
                  <div class="flex items-center gap-3">
                    <h1
                      class="text-3xl font-bold"
                      :style="{ color: 'var(--text-primary)' }"
                    >
                      {{ $t('mcp.title') }}
                    </h1>
                    <span
                      class="px-3 py-1 rounded-full text-xs font-bold"
                      :style="{
                        background: 'rgba(99, 102, 241, 0.15)',
                        color: 'var(--accent-primary)'
                      }"
                    >
                      🔌 {{ $t('mcp.badge') }}
                    </span>
                  </div>
                  <p
                    class="text-sm mt-2"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('mcp.subtitle') }}
                  </p>
                </div>
              </div>
              <div class="flex items-center gap-3">
                <button
                  class="px-5 py-2.5 rounded-xl font-bold text-sm text-white flex items-center gap-2 transition-all duration-300 hover:scale-105"
                  :style="{
                    background: 'linear-gradient(135deg, #6366f1, #8b5cf6)',
                    boxShadow: '0 4px 16px rgba(99, 102, 241, 0.3)'
                  }"
                  @click="handleAdd"
                >
                  <Plus class="w-5 h-5" />
                  {{ $t('mcp.addServer') }}
                </button>
              </div>
            </div>

            <div
              v-if="loading"
              class="flex justify-center py-20"
            >
              <div
                class="w-12 h-12 rounded-full border-4 border-transparent animate-spin"
                :style="{
                  borderTopColor: 'var(--accent-primary)',
                  borderRightColor: 'var(--accent-secondary)'
                }"
              />
            </div>

            <div
              v-else
              class="space-y-3"
            >
              <div
                v-if="!servers || servers.length === 0"
                class="text-center py-10"
                :style="{ color: 'var(--text-muted)' }"
              >
                {{ $t('mcp.noServers') }}
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
                      <h3
                        class="text-lg font-bold font-mono"
                        :style="{ color: 'var(--text-primary)' }"
                      >
                        {{ server.name }}
                      </h3>
                      <span
                        v-if="server.disabled"
                        class="px-2 py-0.5 rounded text-xs font-semibold uppercase"
                        :style="{ background: 'var(--accent-danger)', color: 'white' }"
                      >
                        {{ $t('mcp.disabled') }}
                      </span>
                    </div>
                    <div class="space-y-2 text-sm">
                      <div>
                        <span :style="{ color: 'var(--text-muted)' }">{{ $t('mcp.command') }}:</span>
                        <code
                          class="ml-2 px-2 py-1 rounded font-mono"
                          :style="{ background: 'var(--bg-secondary)', color: 'var(--accent-primary)' }"
                        >
                          {{ server.command }}
                        </code>
                      </div>
                      <div>
                        <span :style="{ color: 'var(--text-muted)' }">{{ $t('mcp.args') }}:</span>
                        <code
                          class="ml-2 px-2 py-1 rounded font-mono"
                          :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }"
                        >
                          {{ server.args.join(' ') }}
                        </code>
                      </div>
                      <div v-if="server.env && Object.keys(server.env).length > 0">
                        <span :style="{ color: 'var(--text-muted)' }">{{ $t('mcp.envVars') }}:</span>
                        <div class="ml-2 mt-1 space-y-1">
                          <div
                            v-for="[key, value] in Object.entries(server.env)"
                            :key="key"
                            class="text-xs font-mono px-2 py-1 rounded"
                            :style="{ background: 'var(--bg-secondary)' }"
                          >
                            <span :style="{ color: 'var(--accent-secondary)' }">{{ key }}</span>=<span
                              :style="{ color: 'var(--text-primary)' }"
                            >{{ value }}</span>
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
                      :title="server.disabled ? $t('mcp.enable') : $t('mcp.disable')"
                      @click="handleToggle(server.name)"
                    >
                      <Power
                        v-if="server.disabled"
                        class="w-4 h-4"
                      />
                      <PowerOff
                        v-else
                        class="w-4 h-4"
                      />
                    </button>
                    <button
                      class="p-2 rounded-lg transition-all hover:scale-110"
                      :style="{
                        background: 'var(--bg-secondary)',
                        border: '1px solid var(--border-color)',
                        color: 'var(--accent-primary)'
                      }"
                      :title="$t('mcp.edit')"
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
                      :title="$t('mcp.delete')"
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
                <h2
                  class="text-xl font-bold mb-4"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ editingServer ? $t('mcp.editServer') : $t('mcp.addServer') }}
                </h2>

                <div class="space-y-4">
                  <div>
                    <label
                      class="block text-sm font-semibold mb-1"
                      :style="{ color: 'var(--text-secondary)' }"
                    >
                      {{ $t('mcp.serverName') }} {{ $t('mcp.required') }}
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
                      :placeholder="$t('mcp.namePlaceholder')"
                    >
                  </div>

                  <div>
                    <label
                      class="block text-sm font-semibold mb-1"
                      :style="{ color: 'var(--text-secondary)' }"
                    >
                      {{ $t('mcp.command') }} {{ $t('mcp.required') }}
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
                      :placeholder="$t('mcp.commandPlaceholder')"
                    >
                  </div>

                  <div>
                    <label
                      class="block text-sm font-semibold mb-1"
                      :style="{ color: 'var(--text-secondary)' }"
                    >
                      {{ $t('mcp.args') }} {{ $t('mcp.required') }}
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
                      :placeholder="$t('mcp.argsPlaceholder')"
                    >
                    <div
                      class="text-xs mt-1"
                      :style="{ color: 'var(--text-muted)' }"
                    >
                      {{ $t('mcp.argsHint') }}
                    </div>
                  </div>

                  <div>
                    <label
                      class="block text-sm font-semibold mb-1"
                      :style="{ color: 'var(--text-secondary)' }"
                    >
                      {{ $t('mcp.envVars') }}
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
                        :placeholder="$t('mcp.envKey')"
                      >
                      <input
                        v-model="envValue"
                        type="text"
                        class="flex-1 px-3 py-2 rounded-lg font-mono"
                        :style="{
                          background: 'var(--bg-tertiary)',
                          border: '1px solid var(--border-color)',
                          color: 'var(--text-primary)'
                        }"
                        :placeholder="$t('mcp.envValue')"
                      >
                      <button
                        class="px-4 py-2 rounded-lg font-semibold text-sm text-white"
                        :style="{ background: 'var(--accent-primary)' }"
                        @click="addEnvVar"
                      >
                        {{ $t('mcp.addEnv') }}
                      </button>
                    </div>
                    <div class="space-y-1">
                      <div
                        v-for="[key, value] in Object.entries(formData.env || {})"
                        :key="key"
                        class="flex items-center justify-between px-3 py-2 rounded"
                        :style="{ background: 'var(--bg-secondary)' }"
                      >
                        <code
                          class="text-sm font-mono"
                          :style="{ color: 'var(--text-primary)' }"
                        >
                          {{ key }}={{ value }}
                        </code>
                        <button
                          class="text-xs"
                          :style="{ color: 'var(--accent-danger)' }"
                          @click="removeEnvVar(key)"
                        >
                          {{ $t('mcp.removeEnv') }}
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
                    >
                    <label
                      for="disabled"
                      class="text-sm"
                      :style="{ color: 'var(--text-secondary)' }"
                    >
                      {{ $t('mcp.disableServer') }}
                    </label>
                  </div>
                </div>

                <div class="flex gap-3 mt-6">
                  <button
                    class="flex-1 px-4 py-2 rounded-lg font-semibold text-white"
                    :style="{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))' }"
                    @click="handleSubmit"
                  >
                    {{ editingServer ? $t('mcp.update') : $t('mcp.add') }}
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
                    {{ $t('mcp.cancel') }}
                  </button>
                </div>
              </div>
            </div>
          </main>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Server, Plus, Edit2, Trash2, Power, PowerOff, Home, Code2 } from 'lucide-vue-next'
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
import Breadcrumb from '@/components/Breadcrumb.vue'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'

const { t } = useI18n({ useScope: 'global' })

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
    alert(t('mcp.loadFailed'))
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
    alert(t('mcp.fillRequired'))
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
      alert(t('mcp.updateSuccess'))
    } else {
      await addMcpServer(request)
      alert(t('mcp.addSuccess'))
    }
    showAddForm.value = false
    await loadServers()
  } catch (err) {
    alert(`${t('mcp.operationFailed')}: ${err instanceof Error ? err.message : t('commands.unknownError')}`)
  }
}

const handleDelete = async (name: string) => {
  if (!confirm(t('mcp.deleteConfirm', { name }))) return

  try {
    await deleteMcpServer(name)
    alert(t('mcp.deleteSuccess'))
    await loadServers()
  } catch (err) {
    alert(`${t('mcp.deleteFailed')}: ${err instanceof Error ? err.message : t('commands.unknownError')}`)
  }
}

const handleToggle = async (name: string) => {
  try {
    await toggleMcpServer(name)
    await loadServers()
  } catch (err) {
    alert(`${t('mcp.toggleFailed')}: ${err instanceof Error ? err.message : t('commands.unknownError')}`)
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
