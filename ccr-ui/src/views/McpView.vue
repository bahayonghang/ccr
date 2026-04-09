<template>
  <div class="mcp-view">
    <div class="mcp-view__container">
      <div class="mcp-view__stack">
        <ModuleSubnav module="claude-code" />

        <main class="mcp-view__main">
          <!-- Header -->
          <div class="mcp-view__hero">
            <div class="mcp-view__hero-main">
              <div class="mcp-view__hero-icon-shell">
                <SIcon
                  name="Server"
                  size="w-6 h-6"
                  class="mcp-view__hero-icon"
                />
              </div>
              <div>
                <div class="mcp-view__hero-title-row">
                  <h1 class="mcp-view__hero-title">
                    {{ $t('mcp.title') }}
                  </h1>
                  <span class="mcp-view__hero-badge">
                    {{ $t('mcp.badge') }}
                  </span>
                </div>
                <p class="mcp-view__hero-subtitle">
                  {{ $t('mcp.subtitle') }}
                </p>
              </div>
            </div>
            
            <button
              class="mcp-view__primary-button"
              @click="handleAdd"
            >
              <SIcon
                name="Plus"
                size="w-5 h-5"
              />
              {{ $t('mcp.addServer') }}
            </button>
          </div>

          <!-- MCP Presets Panel -->
          <McpPresetsPanel @installed="loadServers" />

          <!-- MCP Sync Panel -->
          <McpSyncPanel @synced="loadServers" />

          <!-- Content -->
          <div
            v-if="loading"
            class="mcp-view__loading"
          >
            <div class="mcp-view__loading-spinner" />
          </div>

          <div
            v-else
            class="mcp-view__content"
          >
            <div
              v-if="!servers || servers.length === 0"
              class="mcp-view__empty"
            >
              <div class="mcp-view__empty-icon-shell">
                <SIcon
                  name="Server"
                  size="w-10 h-10"
                  class="mcp-view__empty-icon"
                />
              </div>
              <p class="mcp-view__empty-title">
                {{ $t('mcp.noServers') }}
              </p>
            </div>

            <div
              v-for="server in servers"
              :key="server.name"
              class="mcp-view__server-card"
            >
              <div class="mcp-view__server-card-header">
                <div class="mcp-view__server-card-body">
                  <div class="mcp-view__server-title-row">
                    <h3 class="mcp-view__server-title">
                      {{ server.name }}
                    </h3>
                    <span
                      v-if="server.disabled"
                      class="mcp-view__server-state"
                    >
                      {{ $t('mcp.disabled') }}
                    </span>
                  </div>

                  <div class="mcp-view__server-meta">
                    <div class="mcp-view__server-meta-row">
                      <span class="mcp-view__server-meta-label">{{ $t('mcp.command') }}:</span>
                      <code class="mcp-view__server-code mcp-view__server-code--accent">
                        {{ server.command }}
                      </code>
                    </div>
                    <div class="mcp-view__server-meta-row mcp-view__server-meta-row--top">
                      <span class="mcp-view__server-meta-label mcp-view__server-meta-label--top">{{ $t('mcp.args') }}:</span>
                      <code class="mcp-view__server-code mcp-view__server-code--wrap">
                        {{ (server.args || []).join(' ') }}
                      </code>
                    </div>
                    <div
                      v-if="server.env && Object.keys(server.env).length > 0"
                      class="mcp-view__server-meta-row mcp-view__server-meta-row--top"
                    >
                      <span class="mcp-view__server-meta-label mcp-view__server-meta-label--top">{{ $t('mcp.envVars') }}:</span>
                      <div class="mcp-view__server-env-list">
                        <div
                          v-for="[key, value] in Object.entries(server.env)"
                          :key="key"
                          class="mcp-view__server-env-item"
                        >
                          <span class="text-violet-400">{{ key }}</span>=<span class="text-white">{{ value }}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="mcp-view__server-actions">
                  <button
                    :class="[
                      'mcp-view__icon-button',
                      server.disabled
                        ? 'mcp-view__icon-button--enable'
                        : 'mcp-view__icon-button--disable',
                    ]"
                    :title="server.disabled ? $t('mcp.enable') : $t('mcp.disable')"
                    @click="handleToggle(server.name)"
                  >
                    <SIcon
                      v-if="!server.disabled"
                      name="Power"
                      size="w-4 h-4"
                    />
                    <SIcon
                      v-else
                      name="PowerOff"
                      size="w-4 h-4"
                    />
                  </button>
                  <button
                    class="mcp-view__icon-button mcp-view__icon-button--edit"
                    :title="$t('mcp.edit')"
                    @click="handleEdit(server)"
                  >
                    <SIcon
                      name="Edit2"
                      size="w-4 h-4"
                    />
                  </button>
                  <button
                    class="mcp-view__icon-button mcp-view__icon-button--delete"
                    :title="$t('mcp.delete')"
                    @click="handleDelete(server.name)"
                  >
                    <SIcon
                      name="Trash2"
                      size="w-4 h-4"
                    />
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Add/Edit Form Modal -->
          <div
            v-if="showAddForm"
            class="mcp-view__modal-overlay"
            @click="showAddForm = false"
          >
            <div
              class="mcp-view__modal"
              @click.stop
            >
              <button
                class="mcp-view__modal-close"
                @click="showAddForm = false"
              >
                <SIcon
                  name="X"
                  size="w-5 h-5"
                />
              </button>

              <h2 class="mcp-view__modal-title">
                <div class="mcp-view__modal-title-icon">
                  <SIcon
                    :name="editingServer ? 'Edit2' : 'Plus'"
                    size="w-5 h-5"
                  />
                </div>
                <span class="mcp-view__modal-title-text">{{ editingServer ? $t('mcp.editServer') : $t('mcp.addServer') }}</span>
              </h2>

              <div class="mcp-view__form">
                <div>
                  <label class="mcp-view__field-label">
                    {{ $t('mcp.serverName') }} <span class="text-danger">*</span>
                  </label>
                  <input
                    v-model="formData.name"
                    type="text"
                    class="mcp-view__input"
                    :placeholder="$t('mcp.namePlaceholder')"
                  >
                </div>

                <div>
                  <label class="mcp-view__field-label">
                    {{ $t('mcp.command') }} <span class="text-danger">*</span>
                  </label>
                  <input
                    v-model="formData.command"
                    type="text"
                    class="mcp-view__input mcp-view__input--mono"
                    :placeholder="$t('mcp.commandPlaceholder')"
                  >
                </div>

                <div>
                  <label class="mcp-view__field-label">
                    {{ $t('mcp.args') }} <span class="text-danger">*</span>
                  </label>
                  <input
                    v-model="argInput"
                    type="text"
                    class="mcp-view__input mcp-view__input--mono"
                    :placeholder="$t('mcp.argsPlaceholder')"
                  >
                  <div class="mcp-view__hint">
                    {{ $t('mcp.argsHint') }}
                  </div>
                </div>

                <div>
                  <label class="mcp-view__field-label">
                    {{ $t('mcp.envVars') }}
                  </label>
                  <div class="mcp-view__env-inputs">
                    <input
                      v-model="envKey"
                      type="text"
                      class="mcp-view__input mcp-view__input--mono"
                      :placeholder="$t('mcp.envKey')"
                    >
                    <input
                      v-model="envValue"
                      type="text"
                      class="mcp-view__input mcp-view__input--mono"
                      :placeholder="$t('mcp.envValue')"
                    >
                    <button
                      class="mcp-view__secondary-button"
                      @click="addEnvVar"
                    >
                      {{ $t('mcp.addEnv') }}
                    </button>
                  </div>
                  <div class="mcp-view__env-list">
                    <div
                      v-for="[key, value] in Object.entries(formData.env || {})"
                      :key="key"
                      class="mcp-view__env-row"
                    >
                      <code class="mcp-view__env-code">
                        <span class="text-violet-400">{{ key }}</span>=<span>{{ value }}</span>
                      </code>
                      <button
                        class="mcp-view__env-remove"
                        @click="removeEnvVar(key)"
                      >
                        <SIcon
                          name="X"
                          size="w-4 h-4"
                        />
                      </button>
                    </div>
                  </div>
                </div>

                <div class="mcp-view__toggle">
                  <input
                    id="disabled"
                    v-model="formData.disabled"
                    type="checkbox"
                    class="mcp-view__toggle-input"
                  >
                  <label
                    for="disabled"
                    class="mcp-view__toggle-label"
                  >
                    {{ $t('mcp.disableServer') }}
                  </label>
                </div>
              </div>

              <div class="mcp-view__modal-actions">
                <button
                  class="mcp-view__modal-button mcp-view__modal-button--ghost"
                  @click="showAddForm = false"
                >
                  {{ $t('mcp.cancel') }}
                </button>
                <button
                  class="mcp-view__modal-button mcp-view__modal-button--primary"
                  @click="handleSubmit"
                >
                  {{ editingServer ? $t('mcp.update') : $t('mcp.add') }}
                </button>
              </div>
            </div>
          </div>
        </main>
      </div>
    </div>
    
    <!-- Delete Confirmation Modal -->
    <ConfirmModal
      v-model:is-open="showDeleteModal"
      type="danger"
      :title="$t('mcp.deleteConfirmTitle')"
      :message="deleteConfirmMessage"
      :confirm-text="$t('common.delete')"
      :cancel-text="$t('common.cancel')"
      @confirm="confirmDelete"
    />
    
    <!-- Toggle (Enable/Disable) Confirmation Modal -->
    <ConfirmModal
      v-if="serverToToggle"
      v-model:is-open="showToggleModal"
      type="warning"
      :title="serverToToggle.currentlyDisabled ? $t('mcp.enableConfirmTitle') : $t('mcp.disableConfirmTitle')"
      :message="toggleConfirmMessage"
      :confirm-text="serverToToggle.currentlyDisabled ? $t('mcp.enable') : $t('mcp.disable')"
      :cancel-text="$t('common.cancel')"
      @confirm="confirmToggle"
    />
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  listMcpServers,
  addMcpServer,
  updateMcpServer,
  deleteMcpServer,
  toggleMcpServer
} from '@/api'
import type { McpServer, McpServerRequest } from '@/types'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import ConfirmModal from '@/components/ConfirmModal.vue'
import McpPresetsPanel from '@/components/McpPresetsPanel.vue'
import McpSyncPanel from '@/components/McpSyncPanel.vue'
import { translateWithFallback } from '@/i18n/formatMessage'
import { logger } from '@/utils/logger'
import { useUIStore } from '@/stores/ui'


const { t } = useI18n({ useScope: 'global' })
const uiStore = useUIStore()

const servers = ref<McpServer[]>([])
const loading = ref(true)
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
const showDeleteModal = ref(false)
const serverToDelete = ref('')
const showToggleModal = ref(false)
const serverToToggle = ref<{ name: string; currentlyDisabled: boolean } | null>(null)
const deleteConfirmMessage = computed(() => translateWithFallback(
  t,
  'mcp.deleteConfirmMessage',
  '确定删除 MCP 服务器 "{name}" 吗？',
  { name: serverToDelete.value || '' },
))
const toggleConfirmMessage = computed(() => {
  const target = serverToToggle.value
  if (!target) return ''

  return target.currentlyDisabled
    ? translateWithFallback(
        t,
        'mcp.enableConfirmMessage',
        '确定启用 MCP 服务器 "{name}" 吗？',
        { name: target.name },
      )
    : translateWithFallback(
        t,
        'mcp.disableConfirmMessage',
        '确定禁用 MCP 服务器 "{name}" 吗？禁用后该服务器将不可用。',
        { name: target.name },
      )
})

const loadServers = async () => {
  try {
    loading.value = true
    const data = await listMcpServers<McpServer[]>()
    servers.value = data || []
  } catch (err) {
    logger.error('Failed to load MCP servers:', err)
    servers.value = []
    uiStore.showError(t('mcp.loadFailed'))
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
  argInput.value = server.args?.join(' ') ?? ''
}

const handleSubmit = async () => {
  if (!formData.value.name || !formData.value.command) {
    uiStore.showError(t('mcp.fillRequired'))
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
      uiStore.showSuccess(t('mcp.updateSuccess'))
    } else {
      await addMcpServer(request)
      uiStore.showSuccess(t('mcp.addSuccess'))
    }
    showAddForm.value = false
    await loadServers()
  } catch (err) {
    uiStore.showError(`${t('mcp.operationFailed')}: ${err instanceof Error ? err.message : t('commands.unknownError')}`)
  }
}

const handleDelete = (name: string) => {
  serverToDelete.value = name
  showDeleteModal.value = true
}

const confirmDelete = async () => {
  if (!serverToDelete.value) return
  
  try {
    await deleteMcpServer(serverToDelete.value)
    uiStore.showSuccess(t('mcp.deleteSuccess'))
    await loadServers()
  } catch (err) {
    uiStore.showError(`${t('mcp.deleteFailed')}: ${err instanceof Error ? err.message : t('commands.unknownError')}`)
  } finally {
    showDeleteModal.value = false
    serverToDelete.value = ''
  }
}

const handleToggle = (name: string) => {
  const server = servers.value.find(s => s.name === name)
  if (!server) return

  serverToToggle.value = { name, currentlyDisabled: server.disabled || false }
  showToggleModal.value = true
}

const confirmToggle = async () => {
  if (!serverToToggle.value) return
  
  try {
    await toggleMcpServer(serverToToggle.value.name)
    await loadServers()
  } catch (err) {
    uiStore.showError(`${t('mcp.toggleFailed')}: ${err instanceof Error ? err.message : t('commands.unknownError')}`)
  } finally {
    showToggleModal.value = false
    serverToToggle.value = null
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
</script>

<style scoped>
.mcp-view {
  min-height: 100%;
  padding: 1.5rem;
  transition: color 0.3s ease, background-color 0.3s ease;
}

.mcp-view__container {
  max-width: 1800px;
  margin: 0 auto;
}

.mcp-view__stack {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.mcp-view__main {
  min-width: 0;
}

.mcp-view__hero {
  position: sticky;
  top: 1.5rem;
  z-index: 20;
  margin-bottom: 1.5rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  border: 1px solid rgb(255 255 255 / 20%);
  border-radius: 1rem;
  padding: 1.5rem;
  backdrop-filter: blur(12px);
  box-shadow: 0 1px 2px rgb(15 23 42 / 8%);
}

.mcp-view__hero-main {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.mcp-view__hero-icon-shell,
.mcp-view__modal-title-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid rgb(139 92 246 / 30%);
  border-radius: 0.75rem;
  background: linear-gradient(135deg, rgb(139 92 246 / 20%), rgb(147 51 234 / 20%));
  color: rgb(192 132 252 / 100%);
}

.mcp-view__hero-icon-shell {
  padding: 0.75rem;
}

.mcp-view__modal-title-icon {
  width: 2.5rem;
  height: 2.5rem;
}

.mcp-view__hero-icon {
  color: rgb(192 132 252 / 100%);
}

.mcp-view__hero-title-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.mcp-view__hero-title,
.mcp-view__modal-title {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  color: rgb(255 255 255 / 100%);
  font-weight: 700;
}

.mcp-view__hero-title {
  font-size: 1.5rem;
  line-height: 2rem;
}

.mcp-view__modal-title {
  margin-bottom: 1.5rem;
  font-size: 1.5rem;
  line-height: 2rem;
}

.mcp-view__modal-title-text {
  background: linear-gradient(90deg, rgb(216 180 254 / 100%), rgb(167 139 250 / 100%));
  background-clip: text;
  color: transparent;
}

.mcp-view__hero-badge {
  border: 1px solid rgb(139 92 246 / 30%);
  border-radius: 9999px;
  background: rgb(139 92 246 / 15%);
  padding: 0.125rem 0.625rem;
  color: rgb(192 132 252 / 100%);
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
}

.mcp-view__hero-subtitle {
  margin-top: 0.25rem;
  color: rgb(255 255 255 / 80%);
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.mcp-view__primary-button,
.mcp-view__secondary-button,
.mcp-view__modal-button--primary {
  background: linear-gradient(90deg, rgb(139 92 246 / 100%), rgb(147 51 234 / 100%));
  color: rgb(255 255 255 / 100%);
  box-shadow: 0 18px 40px rgb(139 92 246 / 22%);
}

.mcp-view__primary-button {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  border-radius: 0.75rem;
  padding: 0.625rem 1.25rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 700;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.mcp-view__primary-button:hover {
  transform: scale(1.05);
  box-shadow: 0 22px 44px rgb(139 92 246 / 32%);
}

.mcp-view__loading {
  display: flex;
  justify-content: center;
  padding: 5rem 0;
}

.mcp-view__loading-spinner {
  width: 2.5rem;
  height: 2.5rem;
  border: 4px solid rgb(139 92 246 / 30%);
  border-top-color: rgb(139 92 246 / 100%);
  border-radius: 9999px;
  animation: spin 1s linear infinite;
}

.mcp-view__content {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.mcp-view__empty,
.mcp-view__server-card,
.mcp-view__modal {
  border: 1px solid rgb(255 255 255 / 20%);
}

.mcp-view__empty {
  border-style: dashed;
  border-radius: 1.5rem;
  padding: 4rem 1.5rem;
  text-align: center;
}

.mcp-view__empty-icon-shell {
  display: inline-flex;
  width: 5rem;
  height: 5rem;
  align-items: center;
  justify-content: center;
  margin: 0 auto 1rem;
  border-radius: 9999px;
}

.mcp-view__empty-icon {
  color: rgb(255 255 255 / 50%);
  opacity: 0.3;
}

.mcp-view__empty-title {
  color: rgb(255 255 255 / 100%);
  font-size: 1.125rem;
  line-height: 1.75rem;
  font-weight: 700;
}

.mcp-view__server-card {
  border-radius: 1rem;
  padding: 1.25rem;
  transition: border-color 0.3s ease, box-shadow 0.3s ease;
}

.mcp-view__server-card:hover {
  border-color: rgb(139 92 246 / 30%);
  box-shadow: 0 18px 40px rgb(139 92 246 / 10%);
}

.mcp-view__server-card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
}

.mcp-view__server-card-body {
  flex: 1 1 auto;
}

.mcp-view__server-title-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.75rem;
}

.mcp-view__server-title {
  font-family: var(--font-mono, 'Maple Mono', monospace);
  color: rgb(255 255 255 / 100%);
  font-size: 1.125rem;
  line-height: 1.75rem;
  font-weight: 700;
  transition: color 0.2s ease;
}

.mcp-view__server-card:hover .mcp-view__server-title {
  color: rgb(192 132 252 / 100%);
}

.mcp-view__server-state {
  border: 1px solid rgb(var(--danger-rgb, 239 68 68) / 30%);
  border-radius: 0.25rem;
  background: rgb(var(--danger-rgb, 239 68 68) / 15%);
  padding: 0.125rem 0.5rem;
  color: var(--color-danger, #ef4444);
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 600;
  text-transform: uppercase;
}

.mcp-view__server-meta {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.mcp-view__server-meta-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.mcp-view__server-meta-row--top {
  align-items: flex-start;
}

.mcp-view__server-meta-label {
  width: 5rem;
  color: rgb(255 255 255 / 50%);
}

.mcp-view__server-meta-label--top {
  margin-top: 0.25rem;
}

.mcp-view__server-code,
.mcp-view__server-env-item,
.mcp-view__env-row,
.mcp-view__input,
.mcp-view__modal-button--ghost {
  border: 1px solid rgb(255 255 255 / 20%);
}

.mcp-view__server-code {
  border-color: rgb(255 255 255 / 5%);
  border-radius: 0.5rem;
  padding: 0.25rem 0.5rem;
  font-family: var(--font-mono, 'Maple Mono', monospace);
  color: rgb(255 255 255 / 100%);
}

.mcp-view__server-code--accent {
  color: rgb(192 132 252 / 100%);
}

.mcp-view__server-code--wrap {
  word-break: break-all;
}

.mcp-view__server-env-list,
.mcp-view__env-list,
.mcp-view__form {
  display: flex;
  flex-direction: column;
}

.mcp-view__server-env-list,
.mcp-view__env-list {
  gap: 0.5rem;
}

.mcp-view__server-env-item {
  border-color: rgb(255 255 255 / 5%);
  border-radius: 0.5rem;
  padding: 0.25rem 0.5rem;
  font-family: var(--font-mono, 'Maple Mono', monospace);
  font-size: 0.75rem;
  line-height: 1rem;
}

.mcp-view__server-actions {
  display: flex;
  gap: 0.5rem;
  opacity: 0;
  transition: opacity 0.2s ease;
}

.mcp-view__server-card:hover .mcp-view__server-actions {
  opacity: 1;
}

.mcp-view__icon-button {
  border: 1px solid transparent;
  border-radius: 0.5rem;
  padding: 0.5rem;
  color: rgb(255 255 255 / 80%);
  transition: transform 0.2s ease, color 0.2s ease, background-color 0.2s ease;
}

.mcp-view__icon-button:hover {
  transform: scale(1.1);
}

.mcp-view__icon-button--enable {
  color: rgb(255 255 255 / 50%);
}

.mcp-view__icon-button--enable:hover {
  color: var(--color-success, #22c55e);
  background: rgb(var(--success-rgb, 34 197 94) / 10%);
}

.mcp-view__icon-button--disable {
  color: var(--color-success, #22c55e);
}

.mcp-view__icon-button--disable:hover {
  color: rgb(255 255 255 / 50%);
  background: rgb(255 255 255 / 5%);
}

.mcp-view__icon-button--edit:hover {
  color: rgb(192 132 252 / 100%);
  background: rgb(139 92 246 / 10%);
}

.mcp-view__icon-button--delete:hover,
.mcp-view__env-remove:hover {
  color: var(--color-danger, #ef4444);
}

.mcp-view__modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
  background: rgb(0 0 0 / 40%);
  backdrop-filter: blur(12px);
}

.mcp-view__modal {
  position: relative;
  width: min(100%, 42rem);
  max-height: 90vh;
  overflow-y: auto;
  border-radius: 1.5rem;
  padding: 2rem;
  box-shadow: 0 32px 80px rgb(15 23 42 / 35%);
}

.mcp-view__modal-close {
  position: absolute;
  top: 1rem;
  right: 1rem;
  border-radius: 9999px;
  padding: 0.5rem;
  color: rgb(255 255 255 / 50%);
  transition: color 0.2s ease, background-color 0.2s ease;
}

.mcp-view__modal-close:hover {
  background: rgb(255 255 255 / 5%);
}

.mcp-view__form {
  gap: 1.25rem;
}

.mcp-view__field-label {
  display: block;
  margin-bottom: 0.5rem;
  color: rgb(255 255 255 / 80%);
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.mcp-view__input {
  width: 100%;
  border-radius: 0.75rem;
  padding: 0.75rem 1rem;
  color: rgb(255 255 255 / 100%);
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
  outline: none;
}

.mcp-view__input:focus {
  border-color: rgb(139 92 246 / 100%);
  box-shadow: 0 0 0 4px rgb(139 92 246 / 10%);
}

.mcp-view__input--mono,
.mcp-view__env-code {
  font-family: var(--font-mono, 'Maple Mono', monospace);
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.mcp-view__hint {
  margin-top: 0.375rem;
  color: rgb(255 255 255 / 50%);
  font-size: 0.75rem;
  line-height: 1rem;
}

.mcp-view__env-inputs {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}

.mcp-view__secondary-button {
  border-radius: 0.75rem;
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 700;
  transition: opacity 0.2s ease;
}

.mcp-view__secondary-button:hover {
  opacity: 0.9;
}

.mcp-view__env-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-color: rgb(255 255 255 / 5%);
  border-radius: 0.5rem;
  padding: 0.5rem 1rem;
}

.mcp-view__env-code {
  color: rgb(255 255 255 / 100%);
}

.mcp-view__env-remove {
  color: rgb(255 255 255 / 50%);
  transition: color 0.2s ease;
}

.mcp-view__toggle {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  border: 1px solid rgb(255 255 255 / 5%);
  border-radius: 0.75rem;
  background: rgb(255 255 255 / 5%);
  padding: 1rem;
}

.mcp-view__toggle-input {
  width: 1.25rem;
  height: 1.25rem;
  border-radius: 0.375rem;
  border: 1px solid rgb(255 255 255 / 10%);
  color: rgb(139 92 246 / 100%);
}

.mcp-view__toggle-label {
  cursor: pointer;
  color: rgb(255 255 255 / 80%);
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
}

.mcp-view__modal-actions {
  display: flex;
  gap: 1rem;
  margin-top: 2rem;
  padding-top: 1.5rem;
  border-top: 1px solid rgb(255 255 255 / 5%);
}

.mcp-view__modal-button {
  flex: 1 1 0%;
  border-radius: 0.75rem;
  padding: 0.875rem 1.5rem;
  font-weight: 700;
}

.mcp-view__modal-button--ghost {
  color: rgb(255 255 255 / 80%);
  transition: background-color 0.2s ease;
}

.mcp-view__modal-button--ghost:hover {
  background: rgb(255 255 255 / 10%);
}

.mcp-view__modal-button--primary {
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.mcp-view__modal-button--primary:hover {
  transform: translateY(-2px);
  box-shadow: 0 24px 48px rgb(139 92 246 / 32%);
}

@media (width <= 900px) {
  .mcp-view__hero {
    position: static;
    flex-direction: column;
    align-items: flex-start;
  }

  .mcp-view__server-card-header,
  .mcp-view__env-inputs,
  .mcp-view__modal-actions {
    flex-direction: column;
  }

  .mcp-view__server-actions {
    opacity: 1;
    margin-top: 1rem;
  }
}
</style>
