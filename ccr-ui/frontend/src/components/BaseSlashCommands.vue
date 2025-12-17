<template>
  <div
    :class="themeClasses.container"
    :style="themeStyles.container"
  >
    <!-- 面包屑导航 (可选) -->
    <Breadcrumb
      v-if="props.config.features.breadcrumb"
      :items="breadcrumbItems"
      :module-color="themeColors.module"
      class="mb-6"
    />

    <div :class="themeClasses.layout">
      <!-- 左侧折叠边栏 -->
      <CollapsibleSidebar :module="props.config.route.module" />

      <!-- 文件夹侧边栏 -->
      <FolderSidebar
        :folders="folderOptions"
        :selected-folder="selectedFolder"
        :stats="stats"
        :theme="config.theme"
        @folder-selected="selectedFolder = $event"
      />

      <!-- 主内容区 -->
      <div :class="themeClasses.mainContent">
        <div class="max-w-[1800px] mx-auto">
          <!-- 页面标题 -->
          <PageHeader
            :title="pageTitle"
            :subtitle="pageSubtitle"
            :count="filteredCommands.length"
            :total="stats.total"
            :home-path="config.route.homePath"
            :theme="config.theme"
            :theme-colors="themeColors"
          />

          <!-- 搜索和操作栏 -->
          <SearchAndActions
            v-model="searchQuery"
            :loading="loading"
            :theme="config.theme"
            :theme-colors="themeColors"
            @add-command="showAddModal = true"
            @refresh="loadData"
          />

          <!-- 命令列表 -->
          <CommandList
            :commands="filteredCommands"
            :loading="loading"
            :selected-folder="selectedFolder"
            :theme="config.theme"
            :theme-colors="themeColors"
            :config="config"
            @edit="handleEdit"
            @delete="handleDelete"
            @toggle="handleToggle"
          />

          <!-- 空状态 -->
          <EmptyState
            v-if="!loading && filteredCommands.length === 0"
            :search-query="searchQuery"
            :selected-folder="selectedFolder"
            :theme="config.theme"
            :theme-colors="themeColors"
          />
        </div>
      </div>
    </div>

    <!-- 添加/编辑命令模态框 -->
    <CommandFormModal
      v-model:visible="showAddModal"
      v-model:editing-command="editingCommand"
      :folders="availableFolders"
      :theme="config.theme"
      :theme-colors="themeColors"
      @submit="handleSubmit"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Command, Home, Code2 } from 'lucide-vue-next'

// 组件导入
import Breadcrumb from './Breadcrumb.vue'
import CollapsibleSidebar from './CollapsibleSidebar.vue'
import FolderSidebar from './FolderSidebar.vue'
import PageHeader from './PageHeader.vue'
import SearchAndActions from './SearchAndActions.vue'
import CommandList from './CommandList.vue'
import EmptyState from './EmptyState.vue'
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
    { label: t(`${props.config.i18n.prefix}.folders.all`), value: 'all', icon: Command, count: commands.value.length }
  ]

  availableFolders.value.forEach(folder => {
    const count = commands.value.filter(cmd => cmd.folder === folder).length
    options.push({
      label: folder,
      value: folder,
      icon: Command,
      count
    })
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

  return filtered
})

const breadcrumbItems = computed(() => {
  if (!props.config.features.breadcrumb || !props.config.i18n.breadcrumb) {
    return []
  }

  return [
    {
      label: props.config.i18n.breadcrumb.home,
      path: '/',
      icon: Home
    },
    {
      label: props.config.i18n.breadcrumb.platform,
      path: props.config.route.homePath,
      icon: Code2
    },
    {
      label: props.config.i18n.breadcrumb.current,
      path: '',
      icon: Command
    }
  ]
})

const pageTitle = computed(() => {
  return t(`${props.config.i18n.prefix}.pageTitle`)
})

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
      layout: 'grid grid-cols-[auto_1fr] gap-6',
      mainContent: 'flex gap-6 min-w-0'
    }
  } else {
    return {
      container: '',
      layout: 'display: flex; gap: 0',
      mainContent: 'flex: 1; min-width: 0'
    }
  }
})

const themeStyles = computed(() => {
  if (props.config.theme === 'claude-code') {
    return {
      container: {}
    }
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

const themeColors = computed(() => {
  if (props.config.theme === 'claude-code') {
    return {
      module: '#f59e0b',
      primary: 'text-guofeng-text-primary',
      secondary: 'text-guofeng-text-secondary',
      muted: 'text-guofeng-text-muted',
      bg: 'bg-guofeng-bg-primary',
      bgSecondary: 'bg-guofeng-bg-secondary',
      bgTertiary: 'bg-guofeng-bg-tertiary',
      accent: 'text-guofeng-amber',
      accentBg: 'bg-guofeng-amber',
      accentBorder: 'border-guofeng-amber'
    }
  } else {
    return {
      module: 'var(--accent-primary)',
      primary: 'var(--text-primary)',
      secondary: 'var(--text-secondary)',
      muted: 'var(--text-muted)',
      bg: 'var(--bg-primary)',
      bgSecondary: 'var(--bg-secondary)',
      bgTertiary: 'var(--bg-tertiary)',
      accent: '#fff',
      accentBg: 'var(--accent-primary)',
      accentBorder: 'var(--accent-primary)'
    }
  }
})

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
  loadData()
})
</script>

<style scoped>
/* Claude Code 主题样式 */
.claude-code-theme {
  /* 组件特定的样式将在这里 */
}

/* CSS 变量主题样式 */
.css-variable-theme {
  /* 组件特定的样式将在这里 */
}
</style>