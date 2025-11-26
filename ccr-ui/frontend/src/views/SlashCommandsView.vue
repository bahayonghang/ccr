<template>
  <div class="min-h-screen p-6 transition-colors duration-300">
    <div class="max-w-[1800px] mx-auto">
      <Navbar />
      
      <!-- Breadcrumb Navigation -->
      <Breadcrumb
        :items="[
          { label: $t('slashCommands.breadcrumb.home'), path: '/', icon: Home },
          { label: $t('slashCommands.breadcrumb.claudeCode'), path: '/claude-code', icon: Code2 },
          { label: $t('slashCommands.breadcrumb.slashCommands'), path: '/slash-commands', icon: Command }
        ]"
        module-color="#f59e0b"
        class="mb-6"
      />

      <div class="grid grid-cols-[auto_1fr] gap-6">
        <CollapsibleSidebar module="claude-code" />

        <div class="flex gap-6 min-w-0">
          <!-- Folder Sidebar -->
          <div class="w-64 flex-shrink-0 hidden lg:block sticky top-6 self-start">
            <div class="glass-effect rounded-2xl p-4 border border-white/20 shadow-sm">
              <h3 class="text-xs font-bold text-guofeng-text-muted uppercase tracking-wider mb-3 px-2 flex items-center justify-between">
                {{ $t('slashCommands.folders.title') }}
                <span class="bg-guofeng-bg-tertiary px-1.5 py-0.5 rounded text-[10px]">{{ stats.total }}</span>
              </h3>
              
              <div class="space-y-1">
                <div
                  v-for="folder in folderOptions"
                  :key="folder.value"
                  class="flex items-center gap-2 px-3 py-2 rounded-xl cursor-pointer text-sm transition-all duration-200 group"
                  :class="[
                    selectedFolder === folder.value 
                      ? 'bg-guofeng-amber/10 text-guofeng-amber font-medium shadow-sm border border-guofeng-amber/20' 
                      : 'text-guofeng-text-secondary hover:bg-guofeng-bg-tertiary hover:text-guofeng-text-primary'
                  ]"
                  @click="selectedFolder = folder.value"
                >
                  <component 
                    :is="folder.icon" 
                    class="w-4 h-4 transition-transform group-hover:scale-110" 
                    :class="selectedFolder === folder.value ? 'text-guofeng-amber' : 'text-guofeng-text-muted'"
                  />
                  <span class="flex-1 truncate">{{ folder.label }}</span>
                  <span 
                    class="text-xs px-1.5 py-0.5 rounded-md transition-colors"
                    :class="selectedFolder === folder.value ? 'bg-guofeng-amber/20 text-guofeng-amber' : 'bg-guofeng-bg-tertiary text-guofeng-text-muted'"
                  >
                    {{ folder.count }}
                  </span>
                </div>
              </div>
            </div>
          </div>

          <!-- Main Content -->
          <div class="flex-1 min-w-0">
            <!-- Header -->
            <div class="glass-effect rounded-2xl p-6 mb-6 border border-white/20 flex flex-col md:flex-row items-center justify-between gap-4 sticky top-6 z-20 backdrop-blur-xl shadow-sm">
              <div class="flex items-center gap-4 w-full md:w-auto">
                <div class="p-3 rounded-xl bg-guofeng-amber/10 text-guofeng-amber">
                  <Command class="w-6 h-6" />
                </div>
                <div class="min-w-0">
                  <div class="flex items-center gap-3">
                    <h1 class="text-2xl font-bold text-guofeng-text-primary truncate">
                      {{ $t('slashCommands.title') }}
                    </h1>
                    <span class="px-2.5 py-0.5 rounded-full text-xs font-bold bg-guofeng-amber/10 text-guofeng-amber border border-guofeng-amber/20 flex-shrink-0">
                      Claude Code
                    </span>
                  </div>
                  <p class="text-sm mt-1 text-guofeng-text-secondary truncate">
                    {{ $t('slashCommands.subtitle') }}
                  </p>
                </div>
              </div>
              
              <div class="flex items-center gap-3 w-full md:w-auto justify-end">
                <div class="relative flex-1 md:w-64">
                  <Search class="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-guofeng-text-muted" />
                  <input
                    v-model="searchQuery"
                    type="text"
                    :placeholder="$t('slashCommands.search.placeholder')"
                    class="w-full pl-10 pr-10 py-2.5 rounded-xl transition-all focus:outline-none focus:ring-2 focus:ring-guofeng-amber/20 bg-guofeng-bg-tertiary/50 border border-guofeng-border hover:bg-guofeng-bg-tertiary text-guofeng-text-primary placeholder-guofeng-text-muted text-sm"
                  >
                  <button
                    v-if="searchQuery"
                    class="absolute right-3 top-1/2 transform -translate-y-1/2 p-0.5 rounded-full hover:bg-black/10 text-guofeng-text-muted transition-all"
                    @click="searchQuery = ''"
                  >
                    <X class="w-3 h-3" />
                  </button>
                </div>
                <button
                  class="px-5 py-2.5 rounded-xl font-bold text-sm text-white flex items-center gap-2 transition-all hover:scale-105 bg-guofeng-amber shadow-lg shadow-guofeng-amber/20 hover:shadow-guofeng-amber/30 flex-shrink-0"
                  @click="handleAdd"
                >
                  <Plus class="w-5 h-5" />
                  {{ $t('slashCommands.addCommand') }}
                </button>
              </div>
            </div>

            <!-- Content -->
            <div
              v-if="loading"
              class="flex justify-center py-20"
            >
              <div class="w-10 h-10 rounded-full border-4 border-guofeng-amber/30 border-t-guofeng-amber animate-spin" />
            </div>

            <div
              v-else-if="filteredCommands.length === 0"
              class="text-center py-16 glass-effect rounded-3xl border border-white/20 border-dashed"
            >
              <div class="bg-guofeng-bg-secondary w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
                <Search class="w-10 h-10 opacity-30 text-guofeng-text-muted" />
              </div>
              <p class="text-lg font-bold text-guofeng-text-primary">{{ $t('slashCommands.noMatches') }}</p>
              <p class="text-sm mt-2 text-guofeng-text-muted">{{ $t('slashCommands.tryOtherKeywords') }}</p>
              <button 
                class="mt-6 px-4 py-2 text-sm text-guofeng-amber hover:bg-guofeng-amber/5 rounded-lg transition-colors"
                @click="searchQuery = ''; selectedFolder = ''"
              >
                {{ $t('slashCommands.tryOtherKeywords') }}
              </button>
            </div>

            <div
              v-else
              class="space-y-4"
            >
              <div
                v-for="cmd in filteredCommands"
                :key="cmd.name"
                class="group glass-effect rounded-2xl p-5 border border-white/20 transition-all duration-300 hover:shadow-md hover:border-guofeng-amber/30"
              >
                <div class="flex items-start justify-between">
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-3 mb-2">
                      <h3 class="text-lg font-bold text-guofeng-text-primary group-hover:text-guofeng-amber transition-colors truncate">
                        {{ cmd.name }}
                      </h3>
                      <span
                        v-if="cmd.folder"
                        class="px-2 py-0.5 rounded text-xs font-medium bg-guofeng-bg-tertiary text-guofeng-text-muted border border-guofeng-border/50 flex items-center gap-1"
                      >
                        <Folder class="w-3 h-3" /> {{ cmd.folder }}
                      </span>
                      <span
                        v-if="cmd.disabled"
                        class="px-2 py-0.5 rounded text-xs font-semibold uppercase bg-guofeng-red/10 text-guofeng-red border border-guofeng-red/20"
                      >
                        {{ $t('slashCommands.disabled') }}
                      </span>
                    </div>
                    
                    <div class="mb-2 font-mono text-sm text-guofeng-text-secondary bg-guofeng-bg-tertiary/50 px-3 py-1.5 rounded-lg inline-block border border-guofeng-border/30">
                      {{ cmd.command }}
                    </div>
                    
                    <p class="text-sm text-guofeng-text-muted line-clamp-2">
                      {{ cmd.description }}
                    </p>
                  </div>
                  
                  <div class="flex gap-2 ml-4 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                    <button
                      class="p-2 rounded-lg transition-all hover:scale-110 border border-transparent"
                      :class="cmd.disabled ? 'text-guofeng-text-muted hover:text-guofeng-green hover:bg-guofeng-green/10' : 'text-guofeng-green hover:text-guofeng-text-muted hover:bg-guofeng-bg-tertiary'"
                      :title="cmd.disabled ? $t('slashCommands.enable') : $t('slashCommands.disable')"
                      @click="handleToggle(cmd.name)"
                    >
                      <PowerOff v-if="cmd.disabled" class="w-5 h-5" />
                      <Power v-else class="w-5 h-5" />
                    </button>
                    <button
                      class="p-2 rounded-lg transition-all hover:scale-110 text-guofeng-text-secondary hover:text-guofeng-amber hover:bg-guofeng-amber/10"
                      :title="$t('slashCommands.edit')"
                      @click="handleEdit(cmd)"
                    >
                      <Edit2 class="w-5 h-5" />
                    </button>
                    <button
                      class="p-2 rounded-lg transition-all hover:scale-110 text-guofeng-text-secondary hover:text-guofeng-red hover:bg-guofeng-red/10"
                      :title="$t('slashCommands.delete')"
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
    </div>

    <!-- Add/Edit Form Modal -->
    <div
      v-if="showAddForm"
      class="fixed inset-0 bg-guofeng-ink/20 backdrop-blur-sm flex items-center justify-center p-4 z-50 transition-all"
      @click="showAddForm = false"
    >
      <div
        class="glass-effect rounded-3xl p-8 max-w-2xl w-full max-h-[90vh] overflow-y-auto shadow-2xl border border-white/30 relative"
        @click.stop
      >
        <button 
          class="absolute top-4 right-4 p-2 rounded-full hover:bg-guofeng-bg-tertiary text-guofeng-text-muted transition-colors"
          @click="showAddForm = false"
        >
          <X class="w-5 h-5" />
        </button>

        <h3 class="text-2xl font-bold mb-6 text-guofeng-text-primary flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-guofeng-amber/10 flex items-center justify-center text-guofeng-amber">
            <component :is="editingCommand ? Edit2 : Plus" class="w-5 h-5" />
          </div>
          {{ editingCommand ? $t('slashCommands.editCommand') : $t('slashCommands.addCommand') }}
        </h3>

        <div class="space-y-5">
          <div>
            <label class="block text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider mb-2">
              {{ $t('slashCommands.form.name') }} <span class="text-guofeng-red">*</span>
            </label>
            <input
              v-model="formData.name"
              type="text"
              class="w-full px-4 py-3 rounded-xl bg-white/50 border border-guofeng-border focus:border-guofeng-amber focus:ring-4 focus:ring-guofeng-amber/10 outline-none transition-all"
            >
          </div>
          <div>
            <label class="block text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider mb-2">
              {{ $t('slashCommands.form.script') }} <span class="text-guofeng-red">*</span>
            </label>
            <input
              v-model="formData.command"
              type="text"
              class="w-full px-4 py-3 rounded-xl font-mono text-sm bg-white/50 border border-guofeng-border focus:border-guofeng-amber focus:ring-4 focus:ring-guofeng-amber/10 outline-none transition-all"
            >
          </div>
          <div>
            <label class="block text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider mb-2">
              {{ $t('slashCommands.form.description') }}
            </label>
            <textarea
              v-model="formData.description"
              rows="4"
              class="w-full px-4 py-3 rounded-xl bg-white/50 border border-guofeng-border focus:border-guofeng-amber focus:ring-4 focus:ring-guofeng-amber/10 outline-none transition-all"
            />
          </div>
        </div>

        <div class="flex gap-4 mt-8 pt-6 border-t border-guofeng-border/50">
          <button
            class="flex-1 px-6 py-3.5 rounded-xl font-bold transition-all bg-white text-guofeng-text-secondary hover:bg-guofeng-bg-tertiary border border-guofeng-border"
            @click="showAddForm = false"
          >
            {{ $t('slashCommands.form.cancel') }}
          </button>
          <button
            class="flex-1 px-6 py-3.5 rounded-xl font-bold transition-all bg-guofeng-amber text-white shadow-lg shadow-guofeng-amber/20 hover:shadow-xl hover:shadow-guofeng-amber/30 hover:-translate-y-0.5"
            @click="handleSubmit"
          >
            {{ editingCommand ? $t('slashCommands.form.save') : $t('slashCommands.form.add') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import { Command, Plus, Edit2, Trash2, Power, PowerOff, Search, X, Folder, Home, Code2 } from 'lucide-vue-next'
import Breadcrumb from '@/components/Breadcrumb.vue'
import { listSlashCommands, addSlashCommand, updateSlashCommand, deleteSlashCommand, toggleSlashCommand } from '@/api/client'
import type { SlashCommand, SlashCommandRequest } from '@/types'
import Navbar from '@/components/Navbar.vue'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'

const { t } = useI18n({ useScope: 'global' })

const commands = ref<SlashCommand[]>([])
const folders = ref<string[]>([])
const loading = ref(true)
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
  { value: '', label: t('slashCommands.folders.all'), icon: Folder, count: stats.value.total },
  { value: '__root__', label: t('slashCommands.folders.root'), icon: Home, count: stats.value.rootCount },
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
    const data = await listSlashCommands()
    commands.value = data.commands
    folders.value = data.folders || []
  } catch (err) {
    console.error('Failed to load slash commands:', err)
    alert(t('slashCommands.loadFailed'))
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
  if (!formData.value.name || !formData.value.command) { alert(t('slashCommands.fillRequired')); return }
  const request: SlashCommandRequest = { ...formData.value, args: formData.value.args && formData.value.args.length > 0 ? formData.value.args : undefined }
  try {
    if (editingCommand.value) await updateSlashCommand(editingCommand.value.name, request)
    else await addSlashCommand(request)
    showAddForm.value = false
    editingCommand.value = null
    loadCommands()
  } catch (err) { console.error('操作失败:', err); alert(t('slashCommands.operationFailed')) }
}

const handleDelete = async (name: string) => {
  if (!confirm(t('slashCommands.deleteConfirm', { name }))) return
  try { await deleteSlashCommand(name); loadCommands() } catch (err) { console.error('删除失败:', err); alert(t('slashCommands.deleteFailed')) }
}

const handleToggle = async (name: string) => {
  try { await toggleSlashCommand(name); loadCommands() } catch (err) { console.error('切换状态失败:', err); alert(t('slashCommands.toggleFailed')) }
}
</script>
