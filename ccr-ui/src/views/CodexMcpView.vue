<template>
  <div
    class="min-h-full p-5 transition-colors duration-300"
    :style="{ background: 'var(--bg-primary)' }"
  >
    <div class="max-w-[1800px] mx-auto">
      <div class="space-y-4">
        <ModuleSubnav module="codex" />

        <main
          class="rounded-xl p-6 glass-effect"
          :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
        >
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-3">
              <SIcon
                name="Server"
                size="w-6 h-6"
                :style="{ color: 'var(--accent-primary)' }"
              />
              <h1
                class="text-2xl font-bold"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ $t('codex.mcp.title') }}
              </h1>
            </div>
            <div class="flex items-center gap-3">
              <RouterLink
                to="/codex"
                class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors"
                :style="{ background: 'var(--bg-secondary)', color: 'var(--text-secondary)', border: '1px solid var(--border-color)' }"
              >
                <SIcon
                  name="ArrowLeft"
                  size="w-4 h-4"
                /><span>{{ $t('codex.mcp.backToCodex') }}</span>
              </RouterLink>
              <button
                class="px-4 py-2 rounded-lg font-semibold text-sm text-white flex items-center gap-2"
                :style="{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))', boxShadow: '0 0 20px var(--glow-primary)' }"
                :disabled="submitting"
                @click="handleAdd"
              >
                <SIcon
                  name="Plus"
                  size="w-4 h-4"
                />{{ $t('codex.mcp.addServer') }}
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
              {{ $t('codex.mcp.emptyState') }}
            </div>

            <div
              v-for="server in servers"
              :key="server.name"
              class="group rounded-lg p-4 transition-[color,background-color,border-color,box-shadow] duration-300"
              :style="{ background: 'rgba(255, 255, 255, 0.7)', border: '1px solid rgba(var(--color-accent-primary-rgb), 0.12)', outline: 'none', cursor: 'default' }"
              @mouseenter="(e) => onCardHover(e.currentTarget as HTMLElement, true)"
              @mouseleave="(e) => onCardHover(e.currentTarget as HTMLElement, false)"
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
                      v-if="server.url"
                      class="px-2 py-0.5 rounded text-xs font-semibold"
                      :style="{ background: 'var(--accent-secondary)', color: 'white' }"
                    >{{ $t('codex.mcp.types.http') }}</span>
                    <span
                      v-else
                      class="px-2 py-0.5 rounded text-xs font-semibold"
                      :style="{ background: 'var(--accent-primary)', color: 'white' }"
                    >{{ $t('codex.mcp.types.stdio') }}</span>
                  </div>
                  <div class="space-y-2 text-sm">
                    <div v-if="server.command">
                      <span :style="{ color: 'var(--text-muted)' }">{{ $t('codex.mcp.commandLabel') }}</span>
                      <code
                        class="ml-2 px-2 py-1 rounded font-mono"
                        :style="{ background: 'var(--bg-secondary)', color: 'var(--accent-primary)' }"
                      >{{ server.command }}</code>
                    </div>
                    <div v-if="server.url">
                      <span :style="{ color: 'var(--text-muted)' }">{{ $t('codex.mcp.urlLabel') }}</span>
                      <code
                        class="ml-2 px-2 py-1 rounded font-mono"
                        :style="{ background: 'var(--bg-secondary)', color: 'var(--accent-primary)' }"
                      >{{ server.url }}</code>
                    </div>
                    <div v-if="server.args && server.args.length > 0">
                      <span :style="{ color: 'var(--text-muted)' }">{{ $t('codex.mcp.argsLabel') }}</span>
                      <code
                        class="ml-2 px-2 py-1 rounded font-mono"
                        :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }"
                      >{{ server.args.join(' ') }}</code>
                    </div>
                    <div v-if="server.env && Object.keys(server.env).length > 0">
                      <span :style="{ color: 'var(--text-muted)' }">{{ $t('codex.mcp.envLabel') }}</span>
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
                    class="p-2 rounded-lg transition-transform hover:scale-110"
                    :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-primary)' }"
                    :title="$t('codex.actions.edit')"
                    :disabled="submitting"
                    @click="handleEdit(server)"
                  >
                    <SIcon
                      name="Edit2"
                      size="w-4 h-4"
                    />
                  </button>
                  <button
                    class="p-2 rounded-lg transition-transform hover:scale-110"
                    :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-danger)' }"
                    :title="$t('codex.actions.delete')"
                    :disabled="submitting"
                    @click="handleDelete(server.name)"
                  >
                    <SIcon
                      :name="deletingName === server.name ? 'RefreshCw' : 'Trash2'"
                      size="w-4 h-4"
                      :class="{ 'animate-spin': deletingName === server.name }"
                    />
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
                {{ editingServer ? $t('codex.mcp.editServer') : $t('codex.mcp.addServer') }}
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
                  >{{ $t('codex.mcp.httpServerLabel') }}</span>
                </label>
              </div>

              <div class="space-y-4">
                <div>
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--text-secondary)' }"
                  >Name</label>
                  <input
                    v-model="formData.name"
                    type="text"
                    class="w-full px-3 py-2 rounded-lg"
                    :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                    placeholder="Server Name (e.g. context-mcp)"
                  >
                </div>

                <div v-if="isHttpServer">
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--text-secondary)' }"
                  >{{ $t('codex.mcp.serverUrl') }}</label>
                  <input
                    v-model="formData.url"
                    type="text"
                    class="w-full px-3 py-2 rounded-lg"
                    :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                    :placeholder="$t('codex.mcp.placeholders.url')"
                  >
                </div>

                <div v-else>
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--text-secondary)' }"
                  >{{ $t('codex.mcp.commandField') }}</label>
                  <input
                    v-model="formData.command"
                    type="text"
                    class="w-full px-3 py-2 rounded-lg font-mono"
                    :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                    :placeholder="$t('codex.mcp.placeholders.command')"
                  >
                </div>

                <div v-if="!isHttpServer">
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--text-secondary)' }"
                  >{{ $t('codex.mcp.args') }}</label>
                  <input
                    v-model="argInput"
                    type="text"
                    class="w-full px-3 py-2 rounded-lg font-mono"
                    :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                    :placeholder="$t('codex.mcp.placeholders.args')"
                  >
                  <div
                    class="text-xs mt-1"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t('codex.mcp.argsSeparatorHint') }}
                  </div>
                </div>

                <div>
                  <label
                    class="block text-sm font-semibold mb-1"
                    :style="{ color: 'var(--text-secondary)' }"
                  >{{ $t('codex.mcp.env') }}</label>
                  <div class="flex gap-2 mb-2">
                    <input
                      v-model="envKey"
                      type="text"
                      class="flex-1 px-3 py-2 rounded-lg font-mono"
                      :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                      :placeholder="$t('codex.mcp.placeholders.envKey')"
                    >
                    <input
                      v-model="envValue"
                      type="text"
                      class="flex-1 px-3 py-2 rounded-lg font-mono"
                      :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                      :placeholder="$t('codex.mcp.placeholders.envValue')"
                    >
                    <button
                      class="px-4 py-2 rounded-lg font-semibold text-sm text-white"
                      :style="{ background: 'var(--accent-primary)' }"
                      @click="addEnvVar"
                    >
                      {{ $t('codex.actions.add') }}
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
                        {{ $t('codex.actions.delete') }}
                      </button>
                    </div>
                  </div>
                </div>
              </div>

              <div class="flex gap-3 mt-6">
                <button
                  class="flex-1 px-4 py-2 rounded-lg font-semibold text-white"
                  :style="{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))' }"
                  :disabled="submitting"
                  @click="handleSubmit"
                >
                  {{ editingServer ? $t('codex.mcp.updateServer') : $t('codex.mcp.addServer') }}
                </button>
                <button
                  class="flex-1 px-4 py-2 rounded-lg font-semibold"
                  :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)' }"
                  @click="showAddForm = false"
                >
                  {{ $t('codex.actions.cancel') }}
                </button>
              </div>
            </div>
          </div>

          <ConfirmModal
            v-model:is-open="showDeleteModal"
            type="danger"
            :title="$t('codex.actions.delete')"
            :message="$t('codex.mcp.deleteConfirm', { name: deletingName || '' })"
            :confirm-text="$t('codex.actions.delete')"
            :cancel-text="$t('codex.actions.cancel')"
            @confirm="confirmDelete"
          />
        </main>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { onActivated, onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { listCodexMcpServers, addCodexMcpServer, updateCodexMcpServer, deleteCodexMcpServer } from '@/api'
import type {
  CodexMcpServer,
  CodexMcpServersResponse,
  CodexMcpServerRequest
} from '@/types'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import { useI18n } from 'vue-i18n'
import { logger } from '@/utils/logger'
import ConfirmModal from '@/components/ConfirmModal.vue'
import { useUIStore } from '@/stores/ui'

defineOptions({ name: 'CodexMcpView' })

const { t } = useI18n()
const uiStore = useUIStore()

const servers = ref<CodexMcpServer[]>([])
const loading = ref(true)
const showAddForm = ref(false)
const editingServer = ref<CodexMcpServer | null>(null)
const isHttpServer = ref(false)
const formData = ref<CodexMcpServerRequest>({ name: '', command: undefined, url: undefined, args: [], env: {} })
const argInput = ref('')
const envKey = ref('')
const envValue = ref('')
const submitting = ref(false)
const deletingName = ref<string | null>(null)
const showDeleteModal = ref(false)
const lastLoadedAt = ref(0)

const REFRESH_TTL_MS = 30_000

const loadServers = async () => {
  try {
    loading.value = true
    const data = await listCodexMcpServers<CodexMcpServersResponse>()
    servers.value = Array.isArray(data.servers) ? data.servers : []
    lastLoadedAt.value = Date.now()
  } catch (err) {
    logger.error('Failed to load Codex MCP servers:', err)
    servers.value = []
    uiStore.showError(t('codex.mcp.messages.loadFailed'))
  } finally { loading.value = false }
}

const ensureLoaded = async (force = false) => {
  if (loading.value) return
  if (!force && lastLoadedAt.value && Date.now() - lastLoadedAt.value < REFRESH_TTL_MS) {
    return
  }
  await loadServers()
}

onMounted(() => {
  void ensureLoaded(true)
})

onActivated(() => {
  void ensureLoaded(false)
})

const handleAdd = () => {
  showAddForm.value = true
  editingServer.value = null
  isHttpServer.value = false
  formData.value = { name: '', command: '', url: undefined, args: [], env: {} }
  argInput.value = ''
}

const handleEdit = (server: CodexMcpServer) => {
  editingServer.value = server
  showAddForm.value = true
  isHttpServer.value = !!server.url
  formData.value = { name: server.name, command: server.command, url: server.url, args: server.args || [], env: server.env || {} }
  argInput.value = server.args?.join(' ') || ''
}

const handleSubmit = async () => {
  if (!isHttpServer.value && !formData.value.command) { uiStore.showError(t('codex.mcp.validation.commandRequired')); return }
  if (isHttpServer.value && !formData.value.url) { uiStore.showError(t('codex.mcp.validation.urlRequired')); return }

  const args = argInput.value.split(' ').filter((a) => a.trim())
  const request: CodexMcpServerRequest = { ...formData.value, args }
  if (!request.name) request.name = undefined
  if (isHttpServer.value) request.command = undefined
  else request.url = undefined

  try {
    submitting.value = true
    if (editingServer.value) {
      await updateCodexMcpServer(editingServer.value.name, request)
      uiStore.showSuccess(t('codex.mcp.messages.updateSuccess'))
    } else {
      await addCodexMcpServer(request)
      uiStore.showSuccess(t('codex.mcp.messages.addSuccess'))
    }
    showAddForm.value = false
    await loadServers()
  } catch (err) {
    uiStore.showError(t('codex.mcp.messages.operationFailed', { error: err instanceof Error ? err.message : 'Unknown error' }))
  } finally {
    submitting.value = false
  }
}

const handleDelete = async (name: string) => {
  deletingName.value = name
  showDeleteModal.value = true
}

const confirmDelete = async () => {
  if (!deletingName.value) return
  try {
    submitting.value = true
    await deleteCodexMcpServer(deletingName.value)
    uiStore.showSuccess(t('codex.mcp.messages.deleteSuccess'))
    await loadServers()
  } catch (err) {
    uiStore.showError(t('codex.mcp.messages.deleteFailed', { error: err instanceof Error ? err.message : 'Unknown error' }))
  } finally {
    submitting.value = false
    showDeleteModal.value = false
    deletingName.value = null
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
    el.style.borderColor = 'rgba(var(--color-accent-primary-rgb), 0.24)'
    el.style.boxShadow = '0 4px 6px -1px rgba(0, 0, 0, 0.08), 0 2px 4px -2px rgba(0, 0, 0, 0.08)'
    el.style.transform = 'translateY(-2px)'
  } else {
    el.style.background = 'rgba(255, 255, 255, 0.7)'
    el.style.borderColor = 'rgba(var(--color-accent-primary-rgb), 0.12)'
    el.style.boxShadow = 'none'
    el.style.transform = 'translateY(0)'
  }
}
</script>
