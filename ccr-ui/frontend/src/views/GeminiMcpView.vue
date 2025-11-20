<template>
  <div :style="{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }">
    <div class="max-w-[1800px] mx-auto">
      <!-- Breadcrumb Navigation -->
      <Breadcrumb
        :items="[
          { label: $t('common.home'), path: '/', icon: Home },
          { label: 'Gemini CLI', path: '/gemini-cli', icon: Sparkles },
          { label: $t('gemini.mcp.title'), path: '/gemini-cli/mcp', icon: Server }
        ]"
        module-color="#8b5cf6"
      />
      <div class="grid grid-cols-[auto_1fr] gap-4">
        <CollapsibleSidebar module="gemini-cli" />

        <main
          class="rounded-xl p-6 glass-effect"
          :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
        >
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-3">
              <Server
                class="w-6 h-6"
                :style="{ color: 'var(--accent-primary)' }"
              />
              <h1
                class="text-2xl font-bold"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ $t('gemini.mcp.pageTitle') }}
              </h1>
            </div>
            <div class="flex items-center gap-3">
              <RouterLink
                to="/gemini-cli"
                class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors"
                :style="{ background: 'var(--bg-secondary)', color: 'var(--text-secondary)', border: '1px solid var(--border-color)' }"
              >
                <ArrowLeft class="w-4 h-4" /><span>{{ $t('common.back') }}</span>
              </RouterLink>
              <button
                class="px-4 py-2 rounded-lg font-semibold text-sm text-white flex items-center gap-2"
                :style="{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))', boxShadow: '0 0 20px var(--glow-primary)' }"
                @click="handleAdd"
              >
                <Plus class="w-4 h-4" />{{ $t('gemini.mcp.addServer') }}
              </button>
            </div>
          </div>

          <div
            v-if="loading"
            class="flex justify-center py-20"
          >
            <div
              class="w-12 h-12 rounded-full border-4 border-transparent animate-spin"
              :style="{ borderTopColor: 'var(--accent-primary)', borderRightColor: 'var(--accent-secondary)' }"
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
              {{ $t('gemini.mcp.emptyState') }}
            </div>

            <div
              v-for="server in servers"
              :key="server.command || server.url"
              class="group rounded-lg p-4 transition-all duration-300"
              :style="{ background: 'rgba(255, 255, 255, 0.7)', border: '1px solid rgba(99, 102, 241, 0.12)', outline: 'none', cursor: 'default' }"
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
                      {{ server.command || server.url }}
                    </h3>
                    <span
                      v-if="server.url"
                      class="px-2 py-0.5 rounded text-xs font-semibold"
                      :style="{ background: 'var(--accent-secondary)', color: 'white' }"
                    >HTTP</span>
                    <span
                      v-else
                      class="px-2 py-0.5 rounded text-xs font-semibold"
                      :style="{ background: 'var(--accent-primary)', color: 'white' }"
                    >STDIO</span>
                  </div>
                  <div class="space-y-2 text-sm">
                    <div v-if="server.command">
                      <span :style="{ color: 'var(--text-muted)' }">命令:</span>
                      <code
                        class="ml-2 px-2 py-1 rounded font-mono"
                        :style="{ background: 'var(--bg-secondary)', color: 'var(--accent-primary)' }"
                      >{{ server.command }}</code>
                    </div>
                    <div v-if="server.url">
                      <span :style="{ color: 'var(--text-muted)' }">URL:</span>
                      <code
                        class="ml-2 px-2 py-1 rounded font-mono"
                        :style="{ background: 'var(--bg-secondary)', color: 'var(--accent-primary)' }"
                      >{{ server.url }}</code>
                    </div>
                    <div v-if="server.args && server.args.length > 0">
                      <span :style="{ color: 'var(--text-muted)' }">参数:</span>
                      <code
                        class="ml-2 px-2 py-1 rounded font-mono"
                        :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }"
                      >{{ server.args.join(' ') }}</code>
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
                          <span :style="{ color: 'var(--accent-secondary)' }">{{ key }}</span>=<span :style="{ color: 'var(--text-primary)' }">{{ value }}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
                <div class="flex gap-2">
                  <button
                    class="p-2 rounded-lg transition-all hover:scale-110"
                    :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-primary)' }"
                    title="编辑"
                    @click="handleEdit(server)"
                  >
                    <Edit2 class="w-4 h-4" />
                  </button>
                  <button
                    class="p-2 rounded-lg transition-all hover:scale-110"
                    :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-danger)' }"
                    title="删除"
                    @click="handleDelete(server.command || server.url || '')"
                  >
                    <Trash2 class="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          </div>

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
                {{ editingServer ? $t('gemini.mcp.editServer') : $t('gemini.mcp.addServer') }}
              </h2>

              <div class="mb-4">
                <label class="flex items-center gap-2 cursor-pointer">
                  <input
                    v-model="isHttpServer"
                    type="checkbox"
                    class="w-4 h-4"
                  >
                  <span
                    class="text-sm font-semibold"
                    :style="{ color: 'var(--text-secondary)' }"
                  >{{ $t('gemini.mcp.httpServerHint') }}</span>
                </label>
              </div>

              <div class="space-y-4">
                <div v-if="isHttpServer">
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--text-secondary)' }"
                  >{{ $t('gemini.mcp.urlLabel') }}</label>
                  <input
                    v-model="formData.url"
                    type="text"
                    class="w-full px-3 py-2 rounded-lg"
                    :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                    :placeholder="$t('gemini.mcp.urlPlaceholder')"
                  >
                </div>

                <div v-else>
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--text-secondary)' }"
                  >{{ $t('gemini.mcp.commandLabel') }}</label>
                  <input
                    v-model="formData.command"
                    type="text"
                    class="w-full px-3 py-2 rounded-lg font-mono"
                    :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                    :placeholder="$t('gemini.mcp.commandPlaceholder')"
                  >
                </div>

                <div v-if="!isHttpServer">
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--text-secondary)' }"
                  >{{ $t('gemini.mcp.argsLabel') }}</label>
                  <input
                    v-model="argInput"
                    type="text"
                    class="w-full px-3 py-2 rounded-lg font-mono"
                    :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                    :placeholder="$t('gemini.mcp.argPlaceholder')"
                  >
                  <div
                    class="text-xs mt-1"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t('gemini.mcp.argsHint') }}
                  </div>
                </div>

                <div>
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--text-secondary)' }"
                  >{{ $t('gemini.mcp.envLabel') }}</label>
                  <div class="flex gap-2 mb-2">
                    <input
                      v-model="envKey"
                      type="text"
                      class="flex-1 px-3 py-2 rounded-lg font-mono"
                      :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                      :placeholder="$t('gemini.mcp.envKey')"
                    >
                    <input
                      v-model="envValue"
                      type="text"
                      class="flex-1 px-3 py-2 rounded-lg font-mono"
                      :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                      :placeholder="$t('gemini.mcp.envValue')"
                    >
                    <button
                      class="px-4 py-2 rounded-lg font-semibold text-sm text-white"
                      :style="{ background: 'var(--accent-primary)' }"
                      @click="addEnvVar"
                    >
                      {{ $t('gemini.mcp.addEnv') }}
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
                      >{{ key }}={{ value }}</code>
                      <button
                        class="text-xs"
                        :style="{ color: 'var(--accent-danger)' }"
                        @click="removeEnvVar(key)"
                      >
                        {{ $t('common.delete') }}
                      </button>
                    </div>
                  </div>
                </div>
              </div>

              <div class="flex gap-3 mt-6">
                <button
                  class="flex-1 px-4 py-2 rounded-lg font-semibold text-white"
                  :style="{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))' }"
                  @click="handleSubmit"
                >
                  {{ editingServer ? $t('gemini.mcp.save') : $t('gemini.mcp.add') }}
                </button>
                <button
                  class="flex-1 px-4 py-2 rounded-lg font-semibold"
                  :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)' }"
                  @click="showAddForm = false"
                >
                  {{ $t('common.cancel') }}
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
import { useI18n } from 'vue-i18n'
import { Server, Plus, Edit2, Trash2, ArrowLeft, Home, Sparkles } from 'lucide-vue-next'
import { listGeminiMcpServers, addGeminiMcpServer, updateGeminiMcpServer, deleteGeminiMcpServer, listConfigs, getHistory } from '@/api/client'
import type { GeminiMcpServer, GeminiMcpServerRequest } from '@/types'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'
import Breadcrumb from '@/components/Breadcrumb.vue'

const { t } = useI18n()

const servers = ref<GeminiMcpServer[]>([])
const loading = ref(true)
const currentConfig = ref<string>('')
const totalConfigs = ref(0)
const historyCount = ref(0)
const showAddForm = ref(false)
const editingServer = ref<GeminiMcpServer | null>(null)
const isHttpServer = ref(false)
const formData = ref<GeminiMcpServerRequest>({ command: undefined, url: undefined, args: [], env: {} })
const argInput = ref('')
const envKey = ref('')
const envValue = ref('')

const loadServers = async () => {
  try {
    loading.value = true
    const data = await listGeminiMcpServers()
    servers.value = data || []

    try {
      const configData = await listConfigs()
      currentConfig.value = configData.current_config
      totalConfigs.value = configData.configs.length
      const historyData = await getHistory()
      historyCount.value = historyData.total
    } catch (err) { console.error('Failed to load system info:', err) }
  } catch (err) {
    console.error('Failed to load Gemini MCP servers:', err)
    servers.value = []
    alert(t('gemini.mcp.messages.loadFailed'))
  } finally { loading.value = false }
}

onMounted(() => { loadServers() })

const handleAdd = () => {
  showAddForm.value = true
  editingServer.value = null
  isHttpServer.value = false
  formData.value = { command: '', url: undefined, args: [], env: {} }
  argInput.value = ''
}

const handleEdit = (server: GeminiMcpServer) => {
  editingServer.value = server
  showAddForm.value = true
  isHttpServer.value = !!server.url
  formData.value = { command: server.command, url: server.url, args: server.args || [], env: server.env || {} }
  argInput.value = server.args?.join(' ') || ''
}

const handleSubmit = async () => {
  if (!isHttpServer.value && !formData.value.command) { alert(t('gemini.mcp.validation.required')); return }
  if (isHttpServer.value && !formData.value.url) { alert(t('gemini.mcp.validation.required')); return }

  const args = argInput.value.split(' ').filter((a) => a.trim())
  const request: GeminiMcpServerRequest = { ...formData.value, args }
  if (isHttpServer.value) request.command = undefined
  else request.url = undefined

  try {
    const _name = (request.command || request.url)!
    if (editingServer.value) {
      await updateGeminiMcpServer(editingServer.value.command || editingServer.value.url || '', request)
      alert(t('gemini.mcp.messages.updateSuccess'))
    } else {
      await addGeminiMcpServer(request)
      alert(t('gemini.mcp.messages.addSuccess'))
    }
    showAddForm.value = false
    await loadServers()
  } catch (err) { alert(t('gemini.mcp.messages.operationFailed', { error: err instanceof Error ? err.message : 'Unknown error' })) }
}

const handleDelete = async (name: string) => {
  if (!confirm(t('gemini.mcp.deleteConfirm', { name }))) return
  try {
    await deleteGeminiMcpServer(name)
    alert(t('gemini.mcp.messages.deleteSuccess'))
    await loadServers()
  } catch (err) { alert(t('gemini.mcp.messages.deleteFailed', { error: err instanceof Error ? err.message : 'Unknown error' })) }
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
