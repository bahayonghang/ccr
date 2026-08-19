<template>
  <div :class="themeClasses.container">
    <div :class="themeClasses.layout">
      <ModuleSubnav
        v-if="!hideChrome"
        :module="props.config.route.module"
      />

      <!-- 主内容区 -->
      <main class="min-w-0">
        <!-- Sticky Header: 标题 + 添加按钮 -->
        <div
          class="rounded-xl p-6 mb-6 border border-border-default/25 flex flex-col md:flex-row items-center justify-between gap-4 bg-bg-surface"
          :class="{ 'md:justify-end p-0 mb-4 border-0 bg-transparent': hideChrome }"
        >
          <div
            v-if="!hideChrome"
            class="flex items-center gap-4"
          >
            <div class="p-3 rounded-xl border border-border-default bg-bg-elevated">
              <SIcon
                name="Command"
                size="w-6 h-6"
                class="text-accent-primary"
              />
            </div>
            <div>
              <div class="flex items-center gap-3">
                <h1 class="text-2xl font-bold text-text-primary">
                  {{ pageTitle }}
                </h1>
                <span class="px-2.5 py-0.5 rounded-md text-xs font-medium border border-border-default bg-bg-elevated text-text-secondary">
                  {{ filteredCommands.length }}/{{ stats.total }}
                </span>
              </div>
              <p class="text-sm mt-1 text-text-secondary">
                {{ pageSubtitle }}
              </p>
            </div>
          </div>

          <div class="flex items-center gap-3">
            <!-- 刷新按钮 -->
            <button
              class="px-4 py-2.5 rounded-xl font-medium text-sm flex items-center gap-2 border border-border-default bg-bg-elevated text-text-primary"
              :disabled="loading"
              @click="loadData"
            >
              <SIcon
                name="RefreshCw"
                size="w-4 h-4"
                :class="loading ? 'animate-spin' : ''"
              />
              {{ t('common.refresh') }}
            </button>
            <!-- 添加按钮 -->
            <button
              class="px-5 py-2.5 rounded-lg font-medium text-sm text-[color:var(--color-accent-primary-contrast)] flex items-center gap-2 bg-accent-primary"
              @click="showAddModal = true"
            >
              <SIcon
                name="Plus"
                size="w-5 h-5"
              />
              {{ t('common.add') }}
            </button>
          </div>
        </div>

        <!-- Toolbar: 搜索 + 文件夹Tab + 排序 + 视图 + 过滤 -->
        <div class="rounded-2xl p-4 mb-6 border border-border-subtle bg-bg-surface">
          <!-- 第一行: 搜索框 + 视图控制 -->
          <div class="flex items-center gap-3 flex-wrap">
            <!-- 搜索框 -->
            <div class="relative flex-1 min-w-[200px]">
              <SIcon
                name="Search"
                size="w-4 h-4"
                class="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted"
              />
              <input
                v-model="searchQuery"
                type="text"
                :placeholder="t('common.search')"
                class="w-full pl-10 pr-4 py-2 rounded-xl text-sm border border-border-default bg-bg-base text-text-primary focus:outline-none focus:ring-2 focus:ring-accent-primary/25"
              >
            </div>

            <!-- 排序 -->
            <select
              v-model="viewStore.sortKey"
              class="px-3 py-2 text-sm rounded-xl border border-border-default bg-bg-base text-text-primary focus:outline-none focus:ring-2 focus:ring-accent-primary/25"
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
              class="p-2 rounded-xl border border-border-default bg-bg-base text-text-secondary"
              :title="viewStore.sortDir === 'asc' ? t('slashCommands.viewControls.sortAsc') : t('slashCommands.viewControls.sortDesc')"
              @click="viewStore.toggleSortDir()"
            >
              <SIcon
                name="ArrowUpDown"
                size="w-4 h-4"
                class="transition-transform"
                :class="viewStore.sortDir === 'desc' ? 'rotate-180' : ''"
              />
            </button>

            <!-- 视图模式 -->
            <div class="flex rounded-xl overflow-hidden border border-border-default">
              <button
                class="p-2 transition-colors"
                :class="viewStore.viewMode === 'flat'
                  ? 'bg-accent-primary text-[color:var(--color-accent-primary-contrast)]'
                  : 'bg-bg-base text-text-secondary'"
                :title="t('slashCommands.viewControls.flatView')"
                @click="viewStore.setViewMode('flat')"
              >
                <SIcon
                  name="List"
                  size="w-4 h-4"
                />
              </button>
              <button
                class="p-2 transition-colors"
                :class="viewStore.viewMode === 'tree'
                  ? 'bg-accent-primary text-[color:var(--color-accent-primary-contrast)]'
                  : 'bg-bg-base text-text-secondary'"
                :title="t('slashCommands.viewControls.treeView')"
                @click="viewStore.setViewMode('tree')"
              >
                <SIcon
                  name="FolderTree"
                  size="w-4 h-4"
                />
              </button>
            </div>

            <!-- 废弃过滤 -->
            <button
              class="flex items-center gap-1.5 px-3 py-2 text-sm rounded-xl transition-colors border"
              :class="viewStore.showDeprecated
                ? 'bg-bg-base text-text-secondary border-border-default'
                : 'bg-accent-primary text-[color:var(--color-accent-primary-contrast)] border-accent-primary'"
              @click="viewStore.toggleShowDeprecated()"
            >
              <SIcon
                name="EyeOff"
                size="w-3.5 h-3.5"
              />
              <span class="hidden sm:inline">{{ viewStore.showDeprecated ? t('slashCommands.viewControls.hideDeprecated') : t('slashCommands.viewControls.showDeprecated') }}</span>
            </button>
          </div>

          <!-- 第二行: 文件夹 Tab 标签 -->
          <div
            v-if="folderOptions.length > 1"
            class="flex items-center gap-2 mt-3 pt-3 flex-wrap border-t border-border-subtle"
          >
            <button
              v-for="folder in folderOptions"
              :key="folder.value"
              class="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-lg border transition-colors"
              :class="selectedFolder === folder.value
                ? 'border-accent-primary/30 bg-accent-primary/10 font-medium text-accent-primary'
                : 'border-transparent font-normal text-text-secondary'"
              @click="selectedFolder = folder.value"
            >
              <span>{{ folder.label }}</span>
              <span
                class="text-xs px-1.5 py-0.5 rounded-md"
                :class="selectedFolder === folder.value
                  ? 'bg-accent-primary/20 text-accent-primary'
                  : 'bg-bg-elevated text-text-muted'"
              >{{ folder.count }}</span>
            </button>
          </div>
        </div>

        <!-- Loading -->
        <div
          v-if="loading"
          class="flex justify-center py-20"
        >
          <div class="w-10 h-10 rounded-full border-4 border-accent-primary/30 border-t-accent-primary animate-spin" />
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
                class="w-full flex items-center gap-2 px-4 py-2.5 rounded-xl transition-colors text-left border border-border-default bg-bg-surface"
                @click="viewStore.toggleFolder(folder.name)"
              >
                <SIcon
                  name="FolderTree"
                  size="w-4 h-4"
                  class="text-accent-primary"
                />
                <span class="font-medium text-text-primary">{{ folder.name }}</span>
                <span class="text-sm text-text-muted">({{ folder.commands.length }})</span>
                <SIcon
                  name="ChevronDown"
                  size="w-3.5 h-3.5"
                  class="ml-auto text-text-muted transition-transform"
                  :class="viewStore.expandedFolders.includes(folder.name) ? 'rotate-180' : ''"
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
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useCommandsViewStore } from '@/stores/commandsView'
import { useUIStore } from '@/stores/ui'
import { logger } from '@/utils/logger'

// 组件导入
import { EmptyState } from '@/components/ui'
import ModuleSubnav from './ModuleSubnav.vue'
import CommandList from './CommandList.vue'
import CommandFormModal from './CommandFormModal.vue'
import type { SlashCommand, SlashCommandRequest, PlatformConfig } from '@/types/platform'

// Props
interface Props {
  config: PlatformConfig
  hideChrome?: boolean
}

const props = defineProps<Props>()

// 状态管理
const { t } = useI18n()
const viewStore = useCommandsViewStore()
const uiStore = useUIStore()

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
      container: 'min-h-full p-6 transition-colors duration-300',
      layout: 'space-y-6'
    }
  } else {
    return {
      container: 'min-h-full transition-colors duration-300',
      layout: 'space-y-4'
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
    logger.error('Failed to load slash commands:', error)
  } finally {
    loading.value = false
  }
}

const handleEdit = (command: SlashCommand) => {
  editingCommand.value = { ...command }
  showAddModal.value = true
}

const handleDelete = async (name: string) => {
  const confirmed = await uiStore.requestConfirm({
    title: t('common.delete'),
    message: t(`${props.config.i18n.prefix}.confirmDelete`, { name }),
    confirmText: t('common.delete'),
    cancelText: t('common.cancel'),
    type: 'danger',
  })
  if (!confirmed) {
    return
  }

  try {
    await props.config.api.delete(name)
    await loadData()
  } catch (error) {
    logger.error('Failed to delete slash command:', error)
  }
}

const handleToggle = async (name: string) => {
  try {
    await props.config.api.toggle(name)
    await loadData()
  } catch (error) {
    logger.error('Failed to toggle slash command:', error)
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
    logger.error('Failed to save slash command:', error)
  }
}

// 生命周期
onMounted(() => {
  viewStore.restore()
  loadData()
})
</script>

