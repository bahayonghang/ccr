<template>
  <div
    :class="themeClasses.container"
    :style="themeStyles.container"
  >
    <!-- 面包屑导航 (可选) -->
    <Breadcrumb
      v-if="props.config.features.breadcrumb"
      :items="breadcrumbItems"
      class="mb-6"
    />

    <div :class="themeClasses.layout">
      <!-- 左侧折叠边栏 -->
      <CollapsibleSidebar :module="props.config.route.module" />

      <!-- 主内容区 -->
      <main class="min-w-0">
        <!-- Sticky Header: 标题 + 添加按钮 -->
        <div class="glass-effect rounded-2xl p-6 mb-6 border border-white/20 flex flex-col md:flex-row items-center justify-between gap-4 sticky top-6 z-20 backdrop-blur-xl shadow-sm">
          <div class="flex items-center gap-4">
            <div
              class="p-3 rounded-xl border"
              :style="{
                background: 'color-mix(in srgb, var(--accent-primary) 15%, transparent)',
                borderColor: 'color-mix(in srgb, var(--accent-primary) 30%, transparent)'
              }"
            >
              <Command
                class="w-6 h-6"
                :style="{ color: 'var(--accent-primary)' }"
              />
            </div>
            <div>
              <div class="flex items-center gap-3">
                <h1
                  class="text-2xl font-bold"
                  :style="{
                    background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
                    WebkitBackgroundClip: 'text',
                    WebkitTextFillColor: 'transparent',
                    backgroundClip: 'text'
                  }"
                >
                  {{ pageTitle }}
                </h1>
                <span
                  class="px-2.5 py-0.5 rounded-full text-xs font-bold border"
                  :style="{
                    background: 'color-mix(in srgb, var(--accent-primary) 15%, transparent)',
                    color: 'var(--accent-primary)',
                    borderColor: 'color-mix(in srgb, var(--accent-primary) 30%, transparent)'
                  }"
                >
                  {{ filteredCommands.length }}/{{ stats.total }}
                </span>
              </div>
              <p
                class="text-sm mt-1"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ pageSubtitle }}
              </p>
            </div>
          </div>

          <div class="flex items-center gap-3">
            <!-- 刷新按钮 -->
            <button
              class="px-4 py-2.5 rounded-xl font-bold text-sm flex items-center gap-2 transition-all hover:scale-105"
              :style="{
                background: 'var(--bg-tertiary)',
                color: 'var(--text-primary)',
                border: '1px solid var(--border-color)'
              }"
              :disabled="loading"
              @click="loadData"
            >
              <RefreshCw
                class="w-4 h-4"
                :class="loading ? 'animate-spin' : ''"
              />
              {{ t('common.refresh') }}
            </button>
            <!-- 添加按钮 -->
            <button
              class="px-5 py-2.5 rounded-xl font-bold text-sm text-white flex items-center gap-2 transition-all hover:scale-105 shadow-lg"
              :style="{
                background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
                boxShadow: '0 0 20px var(--glow-primary)'
              }"
              @click="showAddModal = true"
            >
              <Plus class="w-5 h-5" />
              {{ t('common.add') }}
            </button>
          </div>
        </div>

        <!-- Toolbar: 搜索 + 文件夹Tab + 排序 + 视图 + 过滤 -->
        <div
          class="rounded-2xl p-4 mb-6 border"
          :style="{
            background: 'var(--bg-secondary)',
            borderColor: 'var(--border-color)'
          }"
        >
          <!-- 第一行: 搜索框 + 视图控制 -->
          <div class="flex items-center gap-3 flex-wrap">
            <!-- 搜索框 -->
            <div class="relative flex-1 min-w-[200px]">
              <Search
                class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4"
                :style="{ color: 'var(--text-muted)' }"
              />
              <input
                v-model="searchQuery"
                type="text"
                :placeholder="t('common.search')"
                class="w-full pl-10 pr-4 py-2 rounded-xl text-sm focus:outline-none focus:ring-2"
                :style="{
                  border: '1px solid var(--border-color)',
                  background: 'var(--bg-primary)',
                  color: 'var(--text-primary)',
                  '--tw-ring-color': 'var(--accent-primary)'
                }"
              >
            </div>

            <!-- 排序 -->
            <select
              v-model="viewStore.sortKey"
              class="px-3 py-2 text-sm rounded-xl focus:outline-none focus:ring-2"
              :style="{
                border: '1px solid var(--border-color)',
                background: 'var(--bg-primary)',
                color: 'var(--text-primary)',
                '--tw-ring-color': 'var(--accent-primary)'
              }"
            >
              <option value="name">
                {{ t('slashCommands.viewControls.sortByName') }}
              </option>
              <option value="usage">
                {{ t('slashCommands.viewControls.sortByUsage') }}
              </option>
              <option value="modified">
                {{ t('slashCommands.viewControls.sortByModified') }}
              </option>
            </select>
            <button
              class="p-2 rounded-xl transition-colors"
              :style="{
                background: 'var(--bg-primary)',
                border: '1px solid var(--border-color)',
                color: 'var(--text-secondary)'
              }"
              :title="viewStore.sortDir === 'asc' ? t('slashCommands.viewControls.sortAsc') : t('slashCommands.viewControls.sortDesc')"
              @click="viewStore.toggleSortDir()"
            >
              <ArrowUpDown
                :size="16"
                :class="viewStore.sortDir === 'desc' ? 'rotate-180' : ''"
                class="transition-transform"
              />
            </button>

            <!-- 视图模式 -->
            <div
              class="flex rounded-xl overflow-hidden border"
              :style="{ borderColor: 'var(--border-color)' }"
            >
              <button
                class="p-2 transition-colors"
                :style="{
                  background: viewStore.viewMode === 'flat' ? 'var(--accent-primary)' : 'var(--bg-primary)',
                  color: viewStore.viewMode === 'flat' ? '#fff' : 'var(--text-secondary)'
                }"
                :title="t('slashCommands.viewControls.flatView')"
                @click="viewStore.setViewMode('flat')"
              >
                <List :size="16" />
              </button>
              <button
                class="p-2 transition-colors"
                :style="{
                  background: viewStore.viewMode === 'tree' ? 'var(--accent-primary)' : 'var(--bg-primary)',
                  color: viewStore.viewMode === 'tree' ? '#fff' : 'var(--text-secondary)'
                }"
                :title="t('slashCommands.viewControls.treeView')"
                @click="viewStore.setViewMode('tree')"
              >
                <FolderTree :size="16" />
              </button>
            </div>

            <!-- 废弃过滤 -->
            <button
              class="flex items-center gap-1.5 px-3 py-2 text-sm rounded-xl transition-colors border"
              :style="{
                background: viewStore.showDeprecated ? 'var(--bg-primary)' : 'var(--accent-primary)',
                color: viewStore.showDeprecated ? 'var(--text-secondary)' : '#fff',
                borderColor: viewStore.showDeprecated ? 'var(--border-color)' : 'var(--accent-primary)'
              }"
              @click="viewStore.toggleShowDeprecated()"
            >
              <EyeOff :size="14" />
              <span class="hidden sm:inline">{{ viewStore.showDeprecated ? t('slashCommands.viewControls.hideDeprecated') : t('slashCommands.viewControls.showDeprecated') }}</span>
            </button>
          </div>

          <!-- 第二行: 文件夹 Tab 标签 -->
          <div
            v-if="folderOptions.length > 1"
            class="flex items-center gap-2 mt-3 pt-3 flex-wrap"
            :style="{ borderTop: '1px solid var(--border-color)' }"
          >
            <button
              v-for="folder in folderOptions"
              :key="folder.value"
              class="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-full transition-all"
              :style="{
                background: selectedFolder === folder.value
                  ? 'color-mix(in srgb, var(--accent-primary) 15%, transparent)'
                  : 'transparent',
                color: selectedFolder === folder.value
                  ? 'var(--accent-primary)'
                  : 'var(--text-secondary)',
                border: selectedFolder === folder.value
                  ? '1px solid color-mix(in srgb, var(--accent-primary) 30%, transparent)'
                  : '1px solid transparent',
                fontWeight: selectedFolder === folder.value ? '600' : '400'
              }"
              @click="selectedFolder = folder.value"
            >
              <span>{{ folder.label }}</span>
              <span
                class="text-xs px-1.5 py-0.5 rounded-full"
                :style="{
                  background: selectedFolder === folder.value
                    ? 'color-mix(in srgb, var(--accent-primary) 20%, transparent)'
                    : 'var(--bg-tertiary)',
                  color: selectedFolder === folder.value
                    ? 'var(--accent-primary)'
                    : 'var(--text-muted)'
                }"
              >{{ folder.count }}</span>
            </button>
          </div>
        </div>

        <!-- Loading -->
        <div
          v-if="loading"
          class="flex justify-center py-20"
        >
          <div
            class="w-10 h-10 rounded-full border-4 animate-spin"
            :style="{
              borderColor: 'color-mix(in srgb, var(--accent-primary) 30%, transparent)',
              borderTopColor: 'var(--accent-primary)'
            }"
          />
        </div>

        <!-- 命令列表 -->
        <template v-else>
          <div v-if="viewStore.viewMode === 'tree'">
            <div
              v-for="folder in groupedCommands"
              :key="folder.name"
              class="mb-4"
            >
              <button
                class="w-full flex items-center gap-2 px-4 py-2.5 rounded-xl transition-colors text-left border"
                :style="{
                  background: 'var(--bg-secondary)',
                  borderColor: 'var(--border-color)'
                }"
                @click="viewStore.toggleFolder(folder.name)"
              >
                <FolderTree
                  :size="16"
                  :style="{ color: 'var(--accent-primary)' }"
                />
                <span
                  class="font-medium"
                  :style="{ color: 'var(--text-primary)' }"
                >{{ folder.name }}</span>
                <span
                  class="text-sm"
                  :style="{ color: 'var(--text-muted)' }"
                >({{ folder.commands.length }})</span>
                <ChevronDown
                  :size="14"
                  :class="viewStore.expandedFolders.includes(folder.name) ? 'rotate-180' : ''"
                  class="ml-auto transition-transform"
                  :style="{ color: 'var(--text-muted)' }"
                />
              </button>
              <div
                v-if="viewStore.expandedFolders.includes(folder.name)"
                class="mt-2"
              >
                <CommandList
                  :commands="folder.commands"
                  :loading="loading"
                  @edit="handleEdit"
                  @delete="handleDelete"
                  @toggle="handleToggle"
                />
              </div>
            </div>
          </div>
          <CommandList
            v-else
            :commands="filteredCommands"
            :loading="loading"
            @edit="handleEdit"
            @delete="handleDelete"
            @toggle="handleToggle"
          />

          <!-- 空状态 -->
          <EmptyState
            v-if="!loading && filteredCommands.length === 0"
            :title="emptyStateTitle"
            :description="emptyStateDescription"
            :action-text="emptyStateActionText"
            :on-action="emptyStateAction"
          />
        </template>
      </main>
    </div>

    <!-- 添加/编辑命令模态框 -->
    <CommandFormModal
      v-model:visible="showAddModal"
      v-model:editing-command="editingCommand"
      :folders="availableFolders"
      @submit="handleSubmit"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Command, Home, Code2, ArrowUpDown, List, FolderTree, EyeOff, Search, RefreshCw, Plus, ChevronDown } from 'lucide-vue-next'
import { useCommandsViewStore } from '@/stores/commandsView'

// 组件导入
import { Breadcrumb, EmptyState } from '@/components/ui'
import CollapsibleSidebar from './CollapsibleSidebar.vue'
import CommandList from './CommandList.vue'
import CommandFormModal from './CommandFormModal.vue'

// 类型定义
interface SlashCommand {
  name: string
  command: string
  description: string
  folder: string
  enabled: boolean
}

interface SlashCommandRequest {
  name: string
  command: string
  description: string
  folder: string
}

interface PlatformConfig {
  api: {
    list: () => Promise<{ commands: SlashCommand[], folders: string[] }>
    add: (cmd: SlashCommandRequest) => Promise<void>
    update: (name: string, cmd: SlashCommandRequest) => Promise<void>
    delete: (name: string) => Promise<void>
    toggle: (name: string) => Promise<void>
  }
  i18n: {
    prefix: string
    breadcrumb?: {
      home: string
      platform: string
      current: string
    }
  }
  theme: 'claude-code' | 'css-variable'
  route: {
    homePath: string
    module: string
  }
  platform: {
    name: string
    displayName: string
  }
  features: {
    breadcrumb: boolean
    glassEffect: boolean
  }
}

// Props
interface Props {
  config: PlatformConfig
}

const props = defineProps<Props>()

// 状态管理
const { t } = useI18n()
const viewStore = useCommandsViewStore()

const loading = ref(false)
const commands = ref<SlashCommand[]>([])
const folders = ref<string[]>([])
const selectedFolder = ref('all')
const searchQuery = ref('')
const showAddModal = ref(false)
const editingCommand = ref<SlashCommand | null>(null)

// 计算属性
const availableFolders = computed(() => {
  const folderSet = new Set(folders.value)
  commands.value.forEach(cmd => folderSet.add(cmd.folder))
  return Array.from(folderSet).filter(Boolean)
})

const folderOptions = computed(() => {
  const options = [
    { label: t(`${props.config.i18n.prefix}.folders.all`), value: 'all', count: commands.value.length }
  ]

  availableFolders.value.forEach(folder => {
    const count = commands.value.filter(cmd => cmd.folder === folder).length
    options.push({ label: folder, value: folder, count })
  })

  return options
})

const stats = computed(() => ({
  total: commands.value.length,
  enabled: commands.value.filter(cmd => cmd.enabled).length,
  disabled: commands.value.filter(cmd => !cmd.enabled).length,
  byFolder: availableFolders.value.reduce((acc, folder) => {
    acc[folder] = commands.value.filter(cmd => cmd.folder === folder).length
    return acc
  }, {} as Record<string, number>)
}))

const filteredCommands = computed(() => {
  let filtered = commands.value

  // 废弃命令过滤
  if (!viewStore.showDeprecated) {
    filtered = filtered.filter(cmd => !cmd.description?.toLowerCase().includes('deprecated'))
  }

  // 文件夹过滤
  if (selectedFolder.value !== 'all') {
    filtered = filtered.filter(cmd => cmd.folder === selectedFolder.value)
  }

  // 搜索过滤
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase().trim()
    filtered = filtered.filter(cmd =>
      cmd.name.toLowerCase().includes(query) ||
      cmd.command.toLowerCase().includes(query) ||
      cmd.description.toLowerCase().includes(query)
    )
  }

  // 排序
  filtered.sort((a, b) => {
    let comparison = 0
    if (viewStore.sortKey === 'name') {
      comparison = a.name.localeCompare(b.name)
    } else if (viewStore.sortKey === 'usage') {
      comparison = (a.command?.length || 0) - (b.command?.length || 0)
    } else if (viewStore.sortKey === 'modified') {
      comparison = 0 // 需要后端提供修改时间
    }
    return viewStore.sortDir === 'asc' ? comparison : -comparison
  })

  return filtered
})

const groupedCommands = computed(() => {
  const groups = new Map<string, SlashCommand[]>()

  filteredCommands.value.forEach(cmd => {
    const folder = cmd.folder || t(`${props.config.i18n.prefix}.folders.root`)
    if (!groups.has(folder)) {
      groups.set(folder, [])
    }
    groups.get(folder)!.push(cmd)
  })

  return Array.from(groups.entries()).map(([name, commands]) => ({
    name,
    commands
  }))
})

const breadcrumbItems = computed(() => {
  if (!props.config.features.breadcrumb || !props.config.i18n.breadcrumb) {
    return []
  }

  return [
    { label: t(props.config.i18n.breadcrumb.home), path: '/', icon: Home },
    { label: t(props.config.i18n.breadcrumb.platform), path: props.config.route.homePath, icon: Code2 },
    { label: t(props.config.i18n.breadcrumb.current), path: '', icon: Command }
  ]
})

const pageTitle = computed(() => t(`${props.config.i18n.prefix}.pageTitle`))

const pageSubtitle = computed(() => {
  return t(`${props.config.i18n.prefix}.pageSubtitle`, {
    platform: props.config.platform.displayName
  })
})

// 主题相关计算属性
const themeClasses = computed(() => {
  if (props.config.theme === 'claude-code') {
    return {
      container: 'min-h-screen p-6 transition-colors duration-300',
      layout: 'grid grid-cols-[auto_1fr] gap-6'
    }
  } else {
    return {
      container: '',
      layout: 'display: flex; gap: 0'
    }
  }
})

const themeStyles = computed(() => {
  if (props.config.theme === 'claude-code') {
    return { container: {} }
  } else {
    return {
      container: {
        background: 'var(--bg-primary)',
        minHeight: '100vh',
        padding: '20px'
      }
    }
  }
})

// 空状态计算属性
const emptyStateTitle = computed(() => {
  if (searchQuery.value.trim()) return t('slashCommands.noSearchResults')
  if (selectedFolder.value !== 'all') return t('slashCommands.noCommandsInFolder')
  return t('slashCommands.noCommands')
})

const emptyStateDescription = computed(() => {
  if (searchQuery.value.trim()) return t('slashCommands.tryDifferentSearch')
  if (selectedFolder.value !== 'all') return t('slashCommands.tryDifferentFolder')
  return t('slashCommands.addFirstCommand')
})

const emptyStateActionText = computed(() => {
  if (searchQuery.value.trim()) return t('common.clearSearch')
  if (selectedFolder.value !== 'all') return t('common.showAll')
  return t('slashCommands.addFirst')
})

const emptyStateAction = () => {
  if (searchQuery.value.trim()) searchQuery.value = ''
  else if (selectedFolder.value !== 'all') selectedFolder.value = 'all'
  else showAddModal.value = true
}

// 方法
const loadData = async () => {
  loading.value = true
  try {
    const result = await props.config.api.list()
    commands.value = result.commands
    folders.value = result.folders
  } catch (error) {
    console.error('Failed to load slash commands:', error)
  } finally {
    loading.value = false
  }
}

const handleEdit = (command: SlashCommand) => {
  editingCommand.value = { ...command }
  showAddModal.value = true
}

const handleDelete = async (name: string) => {
  if (!confirm(t(`${props.config.i18n.prefix}.confirmDelete`, { name }))) {
    return
  }

  try {
    await props.config.api.delete(name)
    await loadData()
  } catch (error) {
    console.error('Failed to delete slash command:', error)
  }
}

const handleToggle = async (name: string) => {
  try {
    await props.config.api.toggle(name)
    await loadData()
  } catch (error) {
    console.error('Failed to toggle slash command:', error)
  }
}

const handleSubmit = async (data: SlashCommandRequest) => {
  try {
    if (editingCommand.value) {
      await props.config.api.update(editingCommand.value.name, data)
    } else {
      await props.config.api.add(data)
    }

    showAddModal.value = false
    editingCommand.value = null
    await loadData()
  } catch (error) {
    console.error('Failed to save slash command:', error)
  }
}

// 生命周期
onMounted(() => {
  viewStore.restore()
  loadData()
})
</script>
