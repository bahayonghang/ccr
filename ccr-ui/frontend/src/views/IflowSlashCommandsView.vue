<template>
  <div :style="{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }">
    <div class="mb-6" />

    <div style="display: flex; gap: 0">
      <CollapsibleSidebar module="iflow" />

      <div :style="{ width: '180px', background: 'var(--bg-secondary)', borderRight: '1px solid var(--border-color)', padding: '12px 8px', overflowY: 'auto', height: 'calc(100vh - 40px)', flexShrink: 0 }">
        <h4 :style="{ color: 'var(--text-muted)', fontSize: '11px', fontWeight: '600', marginBottom: '8px', marginLeft: '8px', textTransform: 'uppercase', letterSpacing: '0.5px' }">
          {{ $t('iflow.slashCommands.folders.title') }}
        </h4>
        <div
          v-for="folder in folderOptions"
          :key="folder.value"
          :style="{ padding: '6px 8px', borderRadius: '6px', cursor: 'pointer', marginBottom: '4px', background: selectedFolder === folder.value ? 'var(--accent-primary)' : 'transparent', color: selectedFolder === folder.value ? '#fff' : 'var(--text-primary)', display: 'flex', alignItems: 'center', gap: '6px', transition: 'all 0.2s', fontSize: '13px' }"
          @click="selectedFolder = folder.value"
          @mouseenter="(e) => folder.value !== selectedFolder && ((e.currentTarget as HTMLElement).style.background = 'var(--bg-tertiary)')"
          @mouseleave="(e) => folder.value !== selectedFolder && ((e.currentTarget as HTMLElement).style.background = 'transparent')"
        >
          <component
            :is="folder.icon"
            class="w-3.5 h-3.5"
          />
          <span class="flex-1">{{ folder.label }}</span>
          <span :style="{ fontSize: '11px', opacity: 0.7 }">{{ folder.count }}</span>
        </div>
      </div>

      <div :style="{ flex: 1, minWidth: 0 }">
        <div class="max-w-[1600px] mx-auto">
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-4">
              <h2
                class="text-2xl font-bold"
                :style="{ color: 'var(--text-primary)' }"
              >
                <Command class="inline-block w-7 h-7 mr-2" />{{ $t('iflow.slashCommands.title') }}
              </h2>
              <span
                class="px-3 py-1 rounded-full text-sm font-medium"
                :style="{ background: 'var(--accent-primary)', color: '#fff' }"
              >{{ filteredCommands.length }}/{{ stats.total }}</span>
            </div>
            <div class="flex items-center gap-3">
              <RouterLink
                to="/iflow"
                class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors"
                :style="{ background: 'var(--bg-secondary)', color: 'var(--text-secondary)', border: '1px solid var(--border-color)' }"
              >
                <Home class="w-4 h-4" /><span>{{ $t('common.backHome') }}</span>
              </RouterLink>
              <button
                class="px-4 py-2 rounded-lg font-medium transition-all hover:scale-105"
                :style="{ background: 'var(--accent-primary)', color: '#fff' }"
                @click="handleAdd"
              >
                <Plus class="inline-block w-5 h-5 mr-2" />{{ $t('iflow.slashCommands.addCommand') }}
              </button>
            </div>
          </div>

          <div class="mb-4">
            <div class="relative">
              <Search
                class="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5"
                :style="{ color: 'var(--text-muted)' }"
              />
              <input
                v-model="searchQuery"
                type="text"
                :placeholder="$t('iflow.slashCommands.searchPlaceholder')"
                class="w-full pl-11 pr-10 py-3 rounded-lg transition-all focus:outline-none focus:ring-2"
                :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
              >
              <button
                v-if="searchQuery"
                class="absolute right-3 top-1/2 transform -translate-y-1/2 p-1 rounded-lg hover:bg-opacity-20 transition-all"
                :style="{ color: 'var(--text-muted)' }"
                @click="searchQuery = ''"
              >
                <X class="w-4 h-4" />
              </button>
            </div>
            <p
              v-if="searchQuery"
              class="mt-2 text-sm"
              :style="{ color: 'var(--text-muted)' }"
              v-html="$t('iflow.slashCommands.searchResults', { count: filteredCommands.length })"
            />
          </div>

          <div class="space-y-4">
            <div
              v-if="loading"
              class="text-center py-10"
              :style="{ color: 'var(--text-muted)' }"
            >
              {{ $t('common.loading') }}
            </div>
            <div
              v-else-if="commands.length === 0"
              class="text-center py-10"
              :style="{ color: 'var(--text-muted)' }"
            >
              {{ $t('iflow.slashCommands.emptyState') }}
            </div>
            <div
              v-else-if="filteredCommands.length === 0"
              class="text-center py-10"
              :style="{ color: 'var(--text-muted)' }"
            >
              <Search class="w-12 h-12 mx-auto mb-3 opacity-50" /><p>{{ $t('iflow.slashCommands.noResults') }}</p><p class="text-sm mt-2">
                {{ $t('iflow.slashCommands.tryDifferentKeywords') }}
              </p>
            </div>
            <div
              v-for="cmd in filteredCommands"
              v-else
              :key="cmd.name"
              class="group p-6 rounded-xl transition-all duration-300"
              :style="{ background: 'rgba(255, 255, 255, 0.9)', border: '1px solid rgba(99, 102, 241, 0.12)', outline: 'none', cursor: 'default' }"
              @mouseenter="(e) => onCardHover(e.currentTarget as HTMLElement, true)"
              @mouseleave="(e) => onCardHover(e.currentTarget as HTMLElement, false)"
            >
              <div class="flex items-start justify-between">
                <div class="flex-1">
                  <div class="flex items-center gap-3 mb-2">
                    <h3
                      class="text-xl font-semibold"
                      :style="{ color: 'var(--text-primary)' }"
                    >
                      {{ cmd.name }}
                    </h3>
                    <span
                      v-if="cmd.folder"
                      class="px-2 py-1 rounded text-xs font-medium"
                      :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-muted)' }"
                    >📁 {{ cmd.folder }}</span>
                    <span
                      v-if="cmd.disabled"
                      class="px-2 py-1 rounded text-xs font-medium"
                      :style="{ background: '#fef3c7', color: '#92400e' }"
                    >{{ $t('common.disabled') }}</span>
                  </div>
                  <p
                    class="mb-2"
                    :style="{ color: 'var(--text-secondary)', fontSize: '14px' }"
                  >
                    <strong>Command:</strong> {{ cmd.command }}
                  </p>
                  <p
                    class="text-sm"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ cmd.description }}
                  </p>
                </div>
                <div class="flex gap-2 ml-4">
                  <button
                    class="p-2 rounded-lg transition-all hover:scale-110"
                    :style="{ background: cmd.disabled ? '#fef3c7' : '#d1fae5', color: cmd.disabled ? '#92400e' : '#065f46' }"
                    :title="cmd.disabled ? $t('common.enable') : $t('common.disable')"
                    @click="handleToggle(cmd.name)"
                  >
                    <PowerOff
                      v-if="cmd.disabled"
                      class="w-5 h-5"
                    /><Power
                      v-else
                      class="w-5 h-5"
                    />
                  </button>
                  <button
                    class="p-2 rounded-lg transition-all hover:scale-110"
                    :style="{ background: 'var(--bg-tertiary)', color: 'var(--accent-primary)' }"
                    :title="$t('common.edit')"
                    @click="handleEdit(cmd)"
                  >
                    <Edit2 class="w-5 h-5" />
                  </button>
                  <button
                    class="p-2 rounded-lg transition-all hover:scale-110"
                    :style="{ background: '#fee2e2', color: '#991b1b' }"
                    :title="$t('common.delete')"
                    @click="handleDelete(cmd.name)"
                  >
                    <Trash2 class="w-5 h-5" />
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div
      v-if="showAddForm"
      class="fixed inset-0 flex items-center justify-center z-50"
      :style="{ background: 'rgba(0, 0, 0, 0.5)' }"
      @click="showAddForm = false"
    >
      <div
        class="p-8 rounded-2xl w-full max-w-2xl max-h-[80vh] overflow-y-auto"
        :style="{ background: 'var(--bg-secondary)' }"
        @click.stop
      >
        <h3
          class="text-2xl font-bold mb-6"
          :style="{ color: 'var(--text-primary)' }"
        >
          {{ editingCommand ? $t('iflow.slashCommands.editCommand') : $t('iflow.slashCommands.addCommand') }}
        </h3>
        <div class="space-y-4">
          <div>
            <label
              class="block mb-2 font-medium"
              :style="{ color: 'var(--text-secondary)' }"
            >{{ $t('iflow.slashCommands.name') }} *</label>
            <input
              v-model="formData.name"
              type="text"
              class="w-full px-4 py-2 rounded-lg focus:outline-none focus:ring-2"
              :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
            >
          </div>
          <div>
            <label
              class="block mb-2 font-medium"
              :style="{ color: 'var(--text-secondary)' }"
            >{{ $t('iflow.slashCommands.command') }} *</label>
            <input
              v-model="formData.command"
              type="text"
              class="w-full px-4 py-2 rounded-lg focus:outline-none focus:ring-2"
              :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
            >
          </div>
          <div>
            <label
              class="block mb-2 font-medium"
              :style="{ color: 'var(--text-secondary)' }"
            >{{ $t('iflow.slashCommands.description') }}</label>
            <textarea
              v-model="formData.description"
              rows="4"
              class="w-full px-4 py-2 rounded-lg focus:outline-none focus:ring-2"
              :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
            />
          </div>
        </div>
        <div class="flex gap-3 mt-6">
          <button
            class="flex-1 px-6 py-3 rounded-lg font-medium transition-all hover:scale-105"
            :style="{ background: 'var(--accent-primary)', color: '#fff' }"
            @click="handleSubmit"
          >
            {{ editingCommand ? $t('common.save') : $t('common.add') }}
          </button>
          <button
            class="flex-1 px-6 py-3 rounded-lg font-medium transition-all hover:scale-105"
            :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }"
            @click="showAddForm = false"
          >
            {{ $t('common.cancel') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import { Command, Plus, Edit2, Trash2, Power, PowerOff, Search, X, Folder, Home
} from 'lucide-vue-next'
import { listIflowSlashCommands, addIflowSlashCommand, updateIflowSlashCommand, deleteIflowSlashCommand, toggleIflowSlashCommand, listConfigs, getHistory } from '@/api/client'
import type { SlashCommand, SlashCommandRequest } from '@/types'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const commands = ref<SlashCommand[]>([])
const folders = ref<string[]>([])
const loading = ref(true)
const currentConfig = ref('')
const totalConfigs = ref(0)
const historyCount = ref(0)
const selectedFolder = ref('')
const searchQuery = ref('')
const showAddForm = ref(false)
const editingCommand = ref<SlashCommand | null>(null)
const formData = ref<SlashCommandRequest>({ name: '', description: '', command: '', args: [], disabled: false })

const stats = computed(() => {
  const rootCount = commands.value.filter((c) => !c.folder || c.folder === '').length
  const folderCounts: Record<string, number> = {}
  folders.value.forEach((f) => { folderCounts[f] = commands.value.filter((c) => c.folder === f).length })
  return { rootCount, folderCounts, total: commands.value.length }
})

const folderOptions = computed(() => [
  { value: '', label: t('iflow.slashCommands.folders.all'), icon: Folder, count: stats.value.total },
  { value: '__root__', label: t('iflow.slashCommands.folders.root'), icon: Home, count: stats.value.rootCount },
  ...folders.value.map((f) => ({ value: f, label: f, icon: Folder, count: stats.value.folderCounts[f] || 0 }))
])

const filteredCommands = computed(() => {
  let filtered = commands.value
  if (selectedFolder.value === '__root__') filtered = commands.value.filter((cmd) => !cmd.folder || cmd.folder === '')
  else if (selectedFolder.value) filtered = commands.value.filter((cmd) => cmd.folder === selectedFolder.value)
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    filtered = filtered.filter((cmd) => cmd.name.toLowerCase().includes(query) || cmd.description.toLowerCase().includes(query) || cmd.command.toLowerCase().includes(query))
  }
  return filtered.sort((a, b) => a.name.localeCompare(b.name))
})

const loadCommands = async () => {
  try {
    loading.value = true
    const data = await listIflowSlashCommands()
    commands.value = data.commands
    folders.value = data.folders || []
    try {
      const configData = await listConfigs()
      currentConfig.value = configData.current_config
      totalConfigs.value = configData.configs.length
      const historyData = await getHistory()
      historyCount.value = historyData.total
    } catch (err) { console.error('Failed to load system info:', err) }
  } catch (err) {
    console.error('Failed to load slash commands:', err)
    alert(t('iflow.slashCommands.loadFailed'))
  } finally { loading.value = false }
}

onMounted(() => { loadCommands() })

const handleAdd = () => {
  showAddForm.value = true
  editingCommand.value = null
  formData.value = { name: '', description: '', command: '', args: [], disabled: false }
}

const handleEdit = (cmd: SlashCommand) => {
  editingCommand.value = cmd
  showAddForm.value = true
  formData.value = { name: cmd.name, description: cmd.description, command: cmd.command, args: cmd.args || [], disabled: cmd.disabled || false }
}

const handleSubmit = async () => {
  if (!formData.value.name || !formData.value.command) { alert(t('iflow.slashCommands.validation.requiredFields')); return }
  const request: SlashCommandRequest = { ...formData.value, args: formData.value.args && formData.value.args.length > 0 ? formData.value.args : undefined }
  try {
    if (editingCommand.value) await updateIflowSlashCommand(editingCommand.value.name, request)
    else await addIflowSlashCommand(request)
    showAddForm.value = false
    editingCommand.value = null
    loadCommands()
  } catch (err) { console.error('操作失败:', err); alert(t('iflow.slashCommands.operationFailed')) }
}

const handleDelete = async (name: string) => {
  if (!confirm(t('iflow.slashCommands.confirmDelete', { name }))) return
  try { await deleteIflowSlashCommand(name); loadCommands() } catch (err) { console.error('删除失败:', err); alert(t('iflow.slashCommands.deleteFailed')) }
}

const handleToggle = async (name: string) => {
  try { await toggleIflowSlashCommand(name); loadCommands() } catch (err) { console.error('切换状态失败:', err); alert(t('iflow.slashCommands.toggleFailed')) }
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
