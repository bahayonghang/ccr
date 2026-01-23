<template>
  <div class="min-h-screen relative">
    <!-- 🎨 增强的液态玻璃背景 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <!-- 主渐变球 1 -->
      <div
        class="absolute top-10 right-10 w-[600px] h-[600px] rounded-full opacity-20 blur-3xl animate-pulse"
        :style="{ 
          background: 'var(--gradient-primary)',
          animation: 'pulse 8s ease-in-out infinite'
        }"
      />
      <!-- 主渐变球 2 -->
      <div
        class="absolute bottom-10 left-10 w-[500px] h-[500px] rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{
          background: 'var(--gradient-brand)',
          animation: 'pulse 10s ease-in-out infinite',
          animationDelay: '2s'
        }"
      />
      <!-- 辅助渐变球 -->
      <div
        class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[400px] h-[400px] rounded-full opacity-10 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, var(--accent-warning), var(--accent-danger))',
          animation: 'pulse 12s ease-in-out infinite',
          animationDelay: '4s'
        }"
      />
    </div>

    <div class="relative z-10 p-6">
      <div class="max-w-[1800px] mx-auto">
        <!-- Breadcrumb Navigation -->
        <Breadcrumb
          :items="[
            { label: $t('configs.breadcrumb.home'), path: '/', icon: Home },
            { label: $t('configs.breadcrumb.claudeCode'), path: '/claude-code', icon: Code2 },
            { label: $t('configs.breadcrumb.configs'), path: '/configs', icon: Settings }
          ]"
          module-color="var(--platform-gemini)"
        />

        <!-- Environment Badge -->
        <div class="mb-4">
          <EnvironmentBadge />
        </div>

        <!-- 🔗 功能模块快速导航 -->
        <nav
          class="flex flex-wrap gap-2 mb-6 p-3 rounded-2xl"
          :style="{
            background: 'rgba(255, 255, 255, 0.4)',
            backdropFilter: 'blur(12px)',
            border: '1px solid rgba(255, 255, 255, 0.3)',
            boxShadow: 'inset 0 1px 0 rgba(255, 255, 255, 0.5)'
          }"
        >
          <RouterLink
            v-for="navItem in moduleNavItems"
            :key="navItem.path"
            :to="navItem.path"
            class="flex items-center gap-2 px-4 py-2 rounded-full text-sm font-medium transition-all duration-200 hover:scale-105"
            :class="$route.path === navItem.path ? 'module-nav-active' : 'module-nav-inactive'"
          >
            <component
              :is="navItem.icon"
              class="w-4 h-4"
            />
            <span>{{ navItem.label }}</span>
          </RouterLink>
        </nav>

        <!-- 操作按钮栏（已移到 Navbar，保留此处作为备用） -->
        <div
          v-if="false"
          class="flex flex-wrap gap-3 mb-5"
        >
          <button
            class="flex items-center px-4 py-2 rounded-lg text-sm font-semibold transition-all hover:scale-105"
            :style="{
              background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
              color: 'white',
              boxShadow: '0 0 20px var(--glow-primary)'
            }"
            @click="refreshData"
          >
            <RefreshCw class="w-4 h-4 mr-2" />
            {{ $t('configs.buttons.refresh') }}
          </button>
        
          <button
            class="flex items-center px-4 py-2 rounded-lg text-sm font-semibold transition-all hover:scale-105"
            :style="{
              background: 'var(--accent-success)',
              color: 'white'
            }"
            @click="handleValidate"
          >
            <CheckCircle class="w-4 h-4 mr-2" />
            {{ $t('configs.buttons.validate') }}
          </button>

          <button
            class="flex items-center px-4 py-2 rounded-lg text-sm font-semibold transition-all hover:scale-105"
            :style="{
              background: 'var(--accent-warning)',
              color: 'white'
            }"
            @click="handleClean"
          >
            <Trash2 class="w-4 h-4 mr-2" />
            {{ $t('configs.buttons.clean') }}
          </button>

          <button
            class="flex items-center px-4 py-2 rounded-lg text-sm font-semibold transition-all hover:scale-105"
            :style="{
              background: 'var(--bg-tertiary)',
              color: 'var(--text-primary)',
              border: '1px solid var(--border-color)'
            }"
            @click="handleImport"
          >
            <Upload class="w-4 h-4 mr-2" />
            {{ $t('configs.buttons.import') }}
          </button>

          <button
            class="flex items-center px-4 py-2 rounded-lg text-sm font-semibold transition-all hover:scale-105"
            :style="{
              background: 'var(--bg-tertiary)',
              color: 'var(--text-primary)',
              border: '1px solid var(--border-color)'
            }"
            @click="handleExport"
          >
            <Download class="w-4 h-4 mr-2" />
            {{ $t('configs.buttons.export') }}
          </button>
        </div>

        <!-- 两列布局 (删除左侧子导航) -->
        <div class="grid grid-cols-[1fr_320px] gap-6">
          <!-- 主内容区 - 液态玻璃效果 -->
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
            <!-- Tab 导航 - 翡翠绿配色 -->
            <div
              class="flex gap-3 mb-6 p-2 rounded-2xl"
              :style="{ 
                background: 'rgba(255, 255, 255, 0.7)',
                backdropFilter: 'blur(12px)',
                border: '1px solid rgba(var(--color-success-rgb), 0.15)',
                boxShadow: '0 4px 16px rgba(0, 0, 0, 0.04)'
              }"
            >
              <button
                class="flex-1 py-3 px-6 rounded-xl text-sm font-bold transition-all duration-300 flex items-center justify-center gap-2"
                :style="{
                  background: activeTab === 'configs'
                    ? 'var(--gradient-primary)'
                    : 'rgba(255, 255, 255, 0.6)',
                  color: activeTab === 'configs' ? 'white' : 'var(--text-secondary)',
                  boxShadow: activeTab === 'configs' 
                    ? '0 4px 16px rgba(var(--color-success-rgb), 0.35)' 
                    : '0 2px 8px rgba(0, 0, 0, 0.04)',
                  border: activeTab === 'configs' 
                    ? '1px solid rgba(var(--color-success-rgb), 0.4)'
                    : '1px solid rgba(0, 0, 0, 0.06)',
                  transform: activeTab === 'configs' ? 'scale(1.02)' : 'scale(1)'
                }"
                @click="activeTab = 'configs'"
              >
                <Settings class="w-4 h-4" />
                <span>{{ $t('configs.tabs.configList') }}</span>
              </button>
              <button
                class="flex-1 py-3 px-6 rounded-xl text-sm font-bold transition-all duration-300 flex items-center justify-center gap-2"
                :style="{
                  background: activeTab === 'history'
                    ? 'var(--gradient-primary)'
                    : 'rgba(255, 255, 255, 0.6)',
                  color: activeTab === 'history' ? 'white' : 'var(--text-secondary)',
                  boxShadow: activeTab === 'history' 
                    ? '0 4px 16px rgba(var(--color-success-rgb), 0.35)' 
                    : '0 2px 8px rgba(0, 0, 0, 0.04)',
                  border: activeTab === 'history' 
                    ? '1px solid rgba(var(--color-success-rgb), 0.4)'
                    : '1px solid rgba(0, 0, 0, 0.06)',
                  transform: activeTab === 'history' ? 'scale(1.02)' : 'scale(1)'
                }"
                @click="activeTab = 'history'"
              >
                <History class="w-4 h-4" />
                <span>{{ $t('configs.tabs.history') }}</span>
              </button>
            </div>

            <!-- 配置列表 Tab -->
            <div v-if="activeTab === 'configs'">
              <!-- 筛选和排序控制栏 -->
              <ConfigFilters
                v-model:current-filter="currentFilter"
                v-model:current-sort="currentSort"
                @show-provider-stats="showProviderModal = true"
                @add-config="isAddModalOpen = true"
              />

              <!-- 配置列表（含加载/错误/空状态） -->
              <ConfigList
                :configs="filteredConfigs"
                :loading="loading"
                :error="error"
                @switch="handleSwitch"
                @edit="handleEdit"
                @delete="handleDelete"
                @enable="handleEnable"
                @disable="handleDisable"
              />
            </div>

            <!-- 历史记录 Tab -->
            <div v-if="activeTab === 'history'">
              <HistoryList
                :entries="historyEntries"
                :loading="historyLoading"
              />
            </div>
          </main>

          <!-- 右侧边栏：配置导航 -->
          <RightSidebar
            :configs="configs"
            :current-filter="currentFilter"
            @config-click="handleConfigClick"
          />
        </div>
      </div>
    </div>

    <!-- 编辑配置模态框 -->
    <EditConfigModal
      :is-open="isEditModalOpen"
      :config-name="editingConfigName"
      @close="handleEditModalClose"
      @saved="handleEditSaved"
    />

    <!-- 添加配置模态框 -->
    <AddConfigModal
      :is-open="isAddModalOpen"
      @close="handleAddModalClose"
      @saved="handleAddSaved"
    />

    <!-- 提供商统计模态框 -->
    <ProviderStatsModal
      v-model:sort-mode="providerSortMode"
      :visible="showProviderModal"
      :provider-usage="providerUsage"
      :loading="providerLoading"
      :error="providerError"
      @close="showProviderModal = false"
      @refresh="loadProviderUsage"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  Code2,
  Settings,
  Home,
  Cloud,
  Server,
  Command,
  Bot,
  Puzzle,
  History,
} from 'lucide-vue-next'
import { RouterLink } from 'vue-router'
import type { ConfigItem, HistoryEntry } from '@/types'
import {
  listConfigs,
  switchConfig,
  validateConfigs as apiValidateConfigs,
  getHistory,
  deleteConfig,
  enableConfig,
  disableConfig,
} from '@/api'
import { getProviderUsage } from '@/api/client'
import HistoryList from '@/components/HistoryList.vue'
import RightSidebar from '@/components/RightSidebar.vue'
import Breadcrumb from '@/components/Breadcrumb.vue'
import EnvironmentBadge from '@/components/EnvironmentBadge.vue'
import EditConfigModal from '@/components/EditConfigModal.vue'
import AddConfigModal from '@/components/AddConfigModal.vue'
import ProviderStatsModal from '@/components/configs/ProviderStatsModal.vue'
import ConfigFilters from '@/components/configs/ConfigFilters.vue'
import ConfigList from '@/components/configs/ConfigList.vue'

const { t } = useI18n({ useScope: 'global' })

// 🔗 模块导航项配置
const moduleNavItems = [
  { path: '/configs', label: '配置管理', icon: Settings },
  { path: '/sync', label: '云同步', icon: Cloud },
  { path: '/mcp', label: 'MCP 服务器', icon: Server },
  { path: '/slash-commands', label: 'Slash Commands', icon: Command },
  { path: '/agents', label: 'Agents', icon: Bot },
  { path: '/plugins', label: '插件管理', icon: Puzzle },
]

type FilterType = 'all' | 'official_relay' | 'third_party_model' | 'uncategorized'
type SortType = 'name' | 'usage_count' | 'recent'

const configs = ref<ConfigItem[]>([])
const currentConfig = ref<string>('')
const historyEntries = ref<HistoryEntry[]>([])
const loading = ref(true)
const historyLoading = ref(false)
const error = ref<string | null>(null)
const currentFilter = ref<FilterType>('all')
const currentSort = ref<SortType>('name') // 📊 排序方式
const activeTab = ref<'configs' | 'history'>('configs')
const isEditModalOpen = ref(false)
const editingConfigName = ref('')
const isAddModalOpen = ref(false)

const providerUsage = ref<Record<string, number>>({})
const providerLoading = ref(false)
const providerError = ref<string | null>(null)
const providerSortMode = ref<'count_desc' | 'count_asc' | 'name_asc'>('count_desc')
const showProviderModal = ref(false)

// 根据当前筛选器过滤和排序配置
const filteredConfigs = computed(() => {
  // 📌 第一步：筛选
  let filtered: ConfigItem[]
  if (currentFilter.value === 'all') {
    filtered = configs.value
  } else if (currentFilter.value === 'official_relay') {
    filtered = configs.value.filter(
      c => c.provider_type === 'OfficialRelay' || c.provider_type === 'official_relay'
    )
  } else if (currentFilter.value === 'third_party_model') {
    filtered = configs.value.filter(
      c => c.provider_type === 'ThirdPartyModel' || c.provider_type === 'third_party_model'
    )
  } else if (currentFilter.value === 'uncategorized') {
    filtered = configs.value.filter(c => !c.provider_type)
  } else {
    filtered = configs.value
  }

  // 📊 第二步：排序
  const sorted = [...filtered] // 创建副本以避免修改原数组

  if (currentSort.value === 'usage_count') {
    // 按使用次数降序排序（使用多的在前）
    sorted.sort((a, b) => {
      const countA = a.usage_count || 0
      const countB = b.usage_count || 0
      return countB - countA
    })
  } else if (currentSort.value === 'recent') {
    // 按最近使用排序（当前配置在前，然后按使用次数）
    sorted.sort((a, b) => {
      if (a.is_current) return -1
      if (b.is_current) return 1
      const countA = a.usage_count || 0
      const countB = b.usage_count || 0
      return countB - countA
    })
  } else {
    // 按名称排序（默认）
    sorted.sort((a, b) => a.name.localeCompare(b.name))
  }

  return sorted
})

// 加载配置列表
const loadConfigs = async () => {
  try {
    loading.value = true
    error.value = null
    const data = await listConfigs()
    configs.value = data.configs
    currentConfig.value = data.current_config

    // 加载历史记录数量
    const historyData = await getHistory()
    historyEntries.value = historyData.entries
  } catch (err) {
    error.value = err instanceof Error ? err.message : t('configs.loadFailed')
    console.error('Error loading configs:', err)
  } finally {
    loading.value = false
  }
}

// 加载历史记录
const loadHistory = async () => {
  try {
    historyLoading.value = true
    const historyData = await getHistory()
    historyEntries.value = historyData.entries
  } catch (err) {
    console.error('Failed to load history:', err)
  } finally {
    historyLoading.value = false
  }
}

const loadProviderUsage = async () => {
  try {
    providerLoading.value = true
    providerError.value = null
    const data = await getProviderUsage()
    providerUsage.value = data || {}
  } catch (err) {
    providerError.value = err instanceof Error ? err.message : t('configs.provider.loadFailed')
    console.error('Error loading provider usage:', err)
  } finally {
    providerLoading.value = false
  }
}

// 切换配置
const handleSwitch = async (configName: string) => {
  if (!confirm(t('configs.confirmSwitch', { name: configName }))) return

  try {
    await switchConfig(configName)
    alert(t('configs.switchSuccess', { name: configName }))
    await loadConfigs()
    if (activeTab.value === 'history') {
      await loadHistory()
    }
  } catch (err) {
    alert(
      `${t('configs.switchFailed')}: ${err instanceof Error ? err.message : 'Unknown error'}`
    )
  }
}

// 验证配置
const handleValidate = async () => {
  try {
    await apiValidateConfigs()
    alert(t('configs.validateSuccess'))
  } catch (err) {
    alert(
      `${t('configs.validateFailed')}: ${err instanceof Error ? err.message : 'Unknown error'}`
    )
  }
}

// 编辑配置
const handleEdit = (configName: string) => {
  editingConfigName.value = configName
  isEditModalOpen.value = true
}

// 关闭编辑模态框
const handleEditModalClose = () => {
  isEditModalOpen.value = false
  editingConfigName.value = ''
}

// 编辑保存后
const handleEditSaved = async () => {
  await loadConfigs()
}

// 关闭添加模态框
const handleAddModalClose = () => {
  isAddModalOpen.value = false
}

// 添加保存后
const handleAddSaved = async () => {
  await loadConfigs()
}

// 删除配置
const handleDelete = async (configName: string) => {
  if (!confirm(`${t('configs.confirmDelete', { name: configName })} ${t('configs.deleteWarning')}`)) return

  try {
    await deleteConfig(configName)
    alert(t('configs.deleteSuccess'))
  } catch (err) {
    alert(
      `${t('configs.deleteFailed')}: ${err instanceof Error ? err.message : 'Unknown error'}`
    )
  }
}

// 📊 启用配置
const handleEnable = async (configName: string) => {
  if (!confirm(t('configs.confirmEnable', { name: configName }))) return

  try {
    await enableConfig(configName)
    alert(t('configs.enableSuccess', { name: configName }))
  } catch (err) {
    alert(
      `${t('configs.enableFailed')}: ${err instanceof Error ? err.message : 'Unknown error'}`
    )
  }
}

// 📊 禁用配置
const handleDisable = async (configName: string) => {
  if (!confirm(t('configs.confirmDisable', { name: configName }))) return

  try {
    await disableConfig(configName)
    alert(t('configs.disableSuccess', { name: configName }))
  } catch (err) {
    alert(
      `${t('configs.disableFailed')}: ${err instanceof Error ? err.message : 'Unknown error'}`
    )
  }
}

// 清理备份
const handleClean = () => {
  alert(t('configs.cleanComingSoon'))
}

// 导入配置
const handleImport = () => {
  alert(t('configs.importComingSoon'))
}

// 导出配置
const handleExport = () => {
  alert(t('configs.exportComingSoon'))
}

// 刷新数据
const refreshData = async () => {
  await loadConfigs()
  await loadProviderUsage()
  if (activeTab.value === 'history') {
    await loadHistory()
  }
}

// 配置快速跳转
const handleConfigClick = (name: string) => {
  const element = document.getElementById(`config-${name}`)
  if (element) {
    element.scrollIntoView({ behavior: 'smooth', block: 'center' })
    // 闪烁效果
    element.style.transform = 'scale(1.02)'
    setTimeout(() => {
      element.style.transform = ''
    }, 300)
  }
}

// 监听 Tab 切换
watch(activeTab, (newTab) => {
  if (newTab === 'history') {
    loadHistory()
  }
})

onMounted(async () => {
  await loadConfigs()
  await loadProviderUsage()
})
</script>

<style scoped>
/* 🔗 模块导航链接样式 - 翡翠绿配色 */
.module-nav-active {
  background: var(--gradient-primary);
  color: white;
  box-shadow: 0 4px 12px rgb(var(--color-success-rgb), 0.35);
  border: 1px solid rgb(var(--color-success-rgb), 0.4);
}

.module-nav-inactive {
  background: rgb(255 255 255 / 60%);
  color: var(--text-secondary);
  border: 1px solid rgb(0 0 0 / 6%);
}

.module-nav-inactive:hover {
  background: rgb(255 255 255 / 85%);
  color: var(--text-primary);
  box-shadow: 0 2px 8px rgb(0 0 0 / 8%);
  border-color: rgb(var(--color-success-rgb), 0.2);
}
</style>