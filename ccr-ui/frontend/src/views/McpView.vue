<template>
  <div class="min-h-screen p-6 transition-colors duration-300">
    <!-- Enhanced Animated Background -->
    <AnimatedBackground
      variant="spotlight"
      spotlight-color="secondary"
    />

    <div class="max-w-[1800px] mx-auto">
      <Navbar />

      <!-- Breadcrumb Navigation -->
      <Breadcrumb
        :items="[
          { label: $t('mcp.breadcrumb.home'), path: '/', icon: Home },
          { label: $t('mcp.breadcrumb.claudeCode'), path: '/claude-code', icon: Code2 },
          { label: $t('mcp.breadcrumb.mcp'), path: '/mcp', icon: Server }
        ]"
        module-color="#6366f1"
        class="mb-6"
      />

      <div class="grid grid-cols-[auto_1fr] gap-6">
        <CollapsibleSidebar module="claude-code" />

        <main class="min-w-0">
          <!-- Header -->
          <div class="glass-effect rounded-2xl p-6 mb-6 border border-white/20 flex flex-col md:flex-row items-center justify-between gap-4 sticky top-6 z-20 backdrop-blur-xl shadow-sm">
            <div class="flex items-center gap-4">
              <div class="p-3 rounded-xl bg-gradient-to-br from-violet-500/20 to-purple-600/20 border border-violet-500/30">
                <Server class="w-6 h-6 text-violet-400" />
              </div>
              <div>
                <div class="flex items-center gap-3">
                  <h1 class="text-2xl font-bold text-gradient-purple">
                    {{ $t('mcp.title') }}
                  </h1>
                  <span class="px-2.5 py-0.5 rounded-full text-xs font-bold bg-violet-500/15 text-violet-400 border border-violet-500/30">
                    {{ $t('mcp.badge') }}
                  </span>
                </div>
                <p class="text-sm mt-1 text-text-secondary">
                  {{ $t('mcp.subtitle') }}
                </p>
              </div>
            </div>
            
            <button
              class="px-5 py-2.5 rounded-xl font-bold text-sm text-white flex items-center gap-2 transition-all hover:scale-105 bg-gradient-to-r from-violet-500 to-purple-600 shadow-lg shadow-violet-500/25 hover:shadow-violet-500/40"
              @click="handleAdd"
            >
              <Plus class="w-5 h-5" />
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
            class="flex justify-center py-20"
          >
            <div class="w-10 h-10 rounded-full border-4 border-violet-500/30 border-t-violet-500 animate-spin" />
          </div>

          <div
            v-else
            class="space-y-4"
          >
            <div
              v-if="!servers || servers.length === 0"
              class="text-center py-16 glass-effect rounded-3xl border border-white/20 border-dashed"
            >
              <div class="bg-bg-surface w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
                <Server class="w-10 h-10 opacity-30 text-text-muted" />
              </div>
              <p class="text-lg font-bold text-text-primary">
                {{ $t('mcp.noServers') }}
              </p>
            </div>

            <div
              v-for="server in servers"
              :key="server.name"
              class="group glass-effect rounded-2xl p-5 border border-white/20 transition-all duration-300 hover:shadow-lg hover:shadow-violet-500/10 hover:border-violet-500/30"
            >
              <div class="flex items-start justify-between">
                <div class="flex-1">
                  <div class="flex items-center gap-3 mb-3">
                    <h3 class="text-lg font-bold font-mono text-text-primary group-hover:text-violet-400 transition-colors">
                      {{ server.name }}
                    </h3>
                    <span
                      v-if="server.disabled"
                      class="px-2 py-0.5 rounded text-xs font-semibold uppercase bg-danger/15 text-danger border border-danger/30"
                    >
                      {{ $t('mcp.disabled') }}
                    </span>
                  </div>

                  <div class="space-y-2 text-sm">
                    <div class="flex items-center gap-2">
                      <span class="text-text-muted w-20">{{ $t('mcp.command') }}:</span>
                      <code class="px-2 py-1 rounded font-mono bg-bg-surface text-violet-400 border border-border-subtle">
                        {{ server.command }}
                      </code>
                    </div>
                    <div class="flex items-start gap-2">
                      <span class="text-text-muted w-20 mt-1">{{ $t('mcp.args') }}:</span>
                      <code class="px-2 py-1 rounded font-mono bg-bg-surface text-text-primary border border-border-subtle break-all">
                        {{ server.args.join(' ') }}
                      </code>
                    </div>
                    <div
                      v-if="server.env && Object.keys(server.env).length > 0"
                      class="flex items-start gap-2"
                    >
                      <span class="text-text-muted w-20 mt-1">{{ $t('mcp.envVars') }}:</span>
                      <div class="space-y-1">
                        <div
                          v-for="[key, value] in Object.entries(server.env)"
                          :key="key"
                          class="text-xs font-mono px-2 py-1 rounded bg-bg-surface border border-border-subtle"
                        >
                          <span class="text-violet-400">{{ key }}</span>=<span class="text-text-primary">{{ value }}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                  <button
                    class="p-2 rounded-lg transition-all hover:scale-110 border border-transparent"
                    :class="server.disabled ? 'text-text-muted hover:text-success hover:bg-success/10' : 'text-success hover:text-text-muted hover:bg-bg-surface'"
                    :title="server.disabled ? $t('mcp.enable') : $t('mcp.disable')"
                    @click="handleToggle(server.name)"
                  >
                    <Power
                      v-if="!server.disabled"
                      class="w-4 h-4"
                    />
                    <PowerOff
                      v-else
                      class="w-4 h-4"
                    />
                  </button>
                  <button
                    class="p-2 rounded-lg transition-all hover:scale-110 text-text-secondary hover:text-violet-400 hover:bg-violet-500/10"
                    :title="$t('mcp.edit')"
                    @click="handleEdit(server)"
                  >
                    <Edit2 class="w-4 h-4" />
                  </button>
                  <button
                    class="p-2 rounded-lg transition-all hover:scale-110 text-text-secondary hover:text-danger hover:bg-danger/10"
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
            class="fixed inset-0 bg-black/40 backdrop-blur-sm flex items-center justify-center p-4 z-50 transition-all"
            @click="showAddForm = false"
          >
            <div
              class="glass-effect rounded-3xl p-8 max-w-2xl w-full max-h-[90vh] overflow-y-auto shadow-2xl border border-white/30 relative"
              @click.stop
            >
              <button
                class="absolute top-4 right-4 p-2 rounded-full hover:bg-bg-surface text-text-muted transition-colors"
                @click="showAddForm = false"
              >
                <X class="w-5 h-5" />
              </button>

              <h2 class="text-2xl font-bold mb-6 text-text-primary flex items-center gap-3">
                <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-violet-500/20 to-purple-600/20 flex items-center justify-center text-violet-400 border border-violet-500/30">
                  <component
                    :is="editingServer ? Edit2 : Plus"
                    class="w-5 h-5"
                  />
                </div>
                <span class="text-gradient-purple">{{ editingServer ? $t('mcp.editServer') : $t('mcp.addServer') }}</span>
              </h2>

              <div class="space-y-5">
                <div>
                  <label class="block text-xs font-bold text-text-secondary uppercase tracking-wider mb-2">
                    {{ $t('mcp.serverName') }} <span class="text-danger">*</span>
                  </label>
                  <input
                    v-model="formData.name"
                    type="text"
                    class="w-full px-4 py-3 rounded-xl bg-bg-surface border border-border-default focus:border-violet-500 focus:ring-4 focus:ring-violet-500/10 outline-none transition-all text-text-primary"
                    :placeholder="$t('mcp.namePlaceholder')"
                  >
                </div>

                <div>
                  <label class="block text-xs font-bold text-text-secondary uppercase tracking-wider mb-2">
                    {{ $t('mcp.command') }} <span class="text-danger">*</span>
                  </label>
                  <input
                    v-model="formData.command"
                    type="text"
                    class="w-full px-4 py-3 rounded-xl font-mono text-sm bg-bg-surface border border-border-default focus:border-violet-500 focus:ring-4 focus:ring-violet-500/10 outline-none transition-all text-text-primary"
                    :placeholder="$t('mcp.commandPlaceholder')"
                  >
                </div>

                <div>
                  <label class="block text-xs font-bold text-text-secondary uppercase tracking-wider mb-2">
                    {{ $t('mcp.args') }} <span class="text-danger">*</span>
                  </label>
                  <input
                    v-model="argInput"
                    type="text"
                    class="w-full px-4 py-3 rounded-xl font-mono text-sm bg-bg-surface border border-border-default focus:border-violet-500 focus:ring-4 focus:ring-violet-500/10 outline-none transition-all text-text-primary"
                    :placeholder="$t('mcp.argsPlaceholder')"
                  >
                  <div class="text-xs mt-1.5 text-text-muted">
                    {{ $t('mcp.argsHint') }}
                  </div>
                </div>

                <div>
                  <label class="block text-xs font-bold text-text-secondary uppercase tracking-wider mb-2">
                    {{ $t('mcp.envVars') }}
                  </label>
                  <div class="flex gap-2 mb-3">
                    <input
                      v-model="envKey"
                      type="text"
                      class="flex-1 px-4 py-3 rounded-xl font-mono text-sm bg-bg-surface border border-border-default focus:border-violet-500 focus:ring-4 focus:ring-violet-500/10 outline-none transition-all text-text-primary"
                      :placeholder="$t('mcp.envKey')"
                    >
                    <input
                      v-model="envValue"
                      type="text"
                      class="flex-1 px-4 py-3 rounded-xl font-mono text-sm bg-bg-surface border border-border-default focus:border-violet-500 focus:ring-4 focus:ring-violet-500/10 outline-none transition-all text-text-primary"
                      :placeholder="$t('mcp.envValue')"
                    >
                    <button
                      class="px-4 py-2 rounded-xl font-bold text-sm text-white bg-gradient-to-r from-violet-500 to-purple-600 hover:opacity-90 transition-opacity shadow-lg shadow-violet-500/20"
                      @click="addEnvVar"
                    >
                      {{ $t('mcp.addEnv') }}
                    </button>
                  </div>
                  <div class="space-y-2">
                    <div
                      v-for="[key, value] in Object.entries(formData.env || {})"
                      :key="key"
                      class="flex items-center justify-between px-4 py-2 rounded-lg bg-bg-surface border border-border-subtle"
                    >
                      <code class="text-sm font-mono text-text-primary">
                        <span class="text-violet-400">{{ key }}</span>=<span>{{ value }}</span>
                      </code>
                      <button
                        class="text-text-muted hover:text-danger transition-colors"
                        @click="removeEnvVar(key)"
                      >
                        <X class="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                </div>

                <div class="flex items-center gap-3 p-4 rounded-xl bg-bg-surface/50 border border-border-subtle">
                  <input
                    id="disabled"
                    v-model="formData.disabled"
                    type="checkbox"
                    class="w-5 h-5 rounded text-violet-500 focus:ring-violet-500/20 border-border-default"
                  >
                  <label
                    for="disabled"
                    class="text-sm font-medium text-text-secondary cursor-pointer"
                  >
                    {{ $t('mcp.disableServer') }}
                  </label>
                </div>
              </div>

              <div class="flex gap-4 mt-8 pt-6 border-t border-border-subtle">
                <button
                  class="flex-1 px-6 py-3.5 rounded-xl font-bold transition-all bg-bg-surface text-text-secondary hover:bg-bg-overlay border border-border-default"
                  @click="showAddForm = false"
                >
                  {{ $t('mcp.cancel') }}
                </button>
                <button
                  class="flex-1 px-6 py-3.5 rounded-xl font-bold transition-all bg-gradient-to-r from-violet-500 to-purple-600 text-white shadow-lg shadow-violet-500/25 hover:shadow-xl hover:shadow-violet-500/35 hover:-translate-y-0.5"
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
      :message="$t('mcp.deleteConfirmMessage', { name: serverToDelete })"
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
      :message="serverToToggle.currentlyDisabled ? $t('mcp.enableConfirmMessage', { name: serverToToggle.name }) : $t('mcp.disableConfirmMessage', { name: serverToToggle.name })"
      :confirm-text="serverToToggle.currentlyDisabled ? $t('mcp.enable') : $t('mcp.disable')"
      :cancel-text="$t('common.cancel')"
      @confirm="confirmToggle"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Server, Plus, Edit2, Trash2, Power, PowerOff, Home, Code2, X } from 'lucide-vue-next'
import {
  listMcpServers,
  addMcpServer,
  updateMcpServer,
  deleteMcpServer,
  toggleMcpServer
} from '@/api/client'
import type { McpServer, McpServerRequest } from '@/types'
import Navbar from '@/components/Navbar.vue'
import { Breadcrumb } from '@/components/ui'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'
import ConfirmModal from '@/components/ConfirmModal.vue'
import McpPresetsPanel from '@/components/McpPresetsPanel.vue'
import McpSyncPanel from '@/components/McpSyncPanel.vue'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'

const { t } = useI18n({ useScope: 'global' })

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

const loadServers = async () => {
  try {
    loading.value = true
    const data = await listMcpServers()
    servers.value = data || []
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

const handleDelete = (name: string) => {
  serverToDelete.value = name
  showDeleteModal.value = true
}

const confirmDelete = async () => {
  if (!serverToDelete.value) return
  
  try {
    await deleteMcpServer(serverToDelete.value)
    alert(t('mcp.deleteSuccess'))
    await loadServers()
  } catch (err) {
    alert(`${t('mcp.deleteFailed')}: ${err instanceof Error ? err.message : t('commands.unknownError')}`)
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
    alert(`${t('mcp.toggleFailed')}: ${err instanceof Error ? err.message : t('commands.unknownError')}`)
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
