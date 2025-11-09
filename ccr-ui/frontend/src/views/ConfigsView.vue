<template>
  <div class="min-h-screen relative">
    <!-- 🎨 增强的液态玻璃背景 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <!-- 主渐变球 1 -->
      <div
        class="absolute top-10 right-10 w-[600px] h-[600px] rounded-full opacity-20 blur-3xl animate-pulse"
        :style="{ 
          background: 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 50%, #ec4899 100%)',
          animation: 'pulse 8s ease-in-out infinite'
        }"
      />
      <!-- 主渐变球 2 -->
      <div
        class="absolute bottom-10 left-10 w-[500px] h-[500px] rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, #10b981 0%, #06b6d4 50%, #3b82f6 100%)',
          animation: 'pulse 10s ease-in-out infinite',
          animationDelay: '2s'
        }"
      />
      <!-- 辅助渐变球 -->
      <div
        class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[400px] h-[400px] rounded-full opacity-10 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, #f59e0b 0%, #ef4444 100%)',
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
          { label: '首页', path: '/', icon: Home },
          { label: 'Claude Code', path: '/claude-code', icon: Code2 },
          { label: '配置管理', path: '/configs', icon: Settings }
        ]"
        moduleColor="#6366f1"
      />

      <!-- Environment Badge -->
      <div class="mb-4">
        <EnvironmentBadge />
      </div>

      <!-- 操作按钮栏（已移到 Navbar，保留此处作为备用） -->
      <div v-if="false" class="flex flex-wrap gap-3 mb-5">
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
          刷新
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
          验证配置
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
          清理备份
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
          导入
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
          导出
        </button>
      </div>

      <!-- 三列布局 -->
      <div class="grid grid-cols-[auto_1fr_320px] gap-6">
        <!-- 可折叠侧边栏 -->
        <CollapsibleSidebar module="claude-code" />

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
          <!-- Tab 导航 - 增强版 -->
          <div
            class="flex gap-2 mb-6 p-1.5 rounded-2xl"
            :style="{ 
              background: 'rgba(99, 102, 241, 0.08)',
              backdropFilter: 'blur(10px)',
              border: '1px solid rgba(99, 102, 241, 0.15)'
            }"
          >
            <button
              class="flex-1 py-3 px-6 rounded-xl text-sm font-bold transition-all duration-300"
              :style="{
                background: activeTab === 'configs' 
                  ? 'linear-gradient(135deg, #6366f1, #8b5cf6)' 
                  : 'transparent',
                color: activeTab === 'configs' ? 'white' : 'var(--text-secondary)',
                boxShadow: activeTab === 'configs' 
                  ? '0 4px 12px rgba(99, 102, 241, 0.3)' 
                  : 'none',
                transform: activeTab === 'configs' ? 'scale(1.02)' : 'scale(1)'
              }"
              @click="activeTab = 'configs'"
            >
              📋 配置列表
            </button>
            <button
              class="flex-1 py-3 px-6 rounded-xl text-sm font-bold transition-all duration-300"
              :style="{
                background: activeTab === 'history' 
                  ? 'linear-gradient(135deg, #6366f1, #8b5cf6)' 
                  : 'transparent',
                color: activeTab === 'history' ? 'white' : 'var(--text-secondary)',
                boxShadow: activeTab === 'history' 
                  ? '0 4px 12px rgba(99, 102, 241, 0.3)' 
                  : 'none',
                transform: activeTab === 'history' ? 'scale(1.02)' : 'scale(1)'
              }"
              @click="activeTab = 'history'"
            >
              🕐 历史记录
            </button>
          </div>

          <!-- 配置列表 Tab -->
          <div v-if="activeTab === 'configs'">
            <!-- 筛选按钮 - 液态玻璃风格 -->
            <div
              class="flex gap-3 mb-6 p-2 rounded-2xl"
              :style="{
                background: 'rgba(255, 255, 255, 0.4)',
                backdropFilter: 'blur(10px)',
                border: '1px solid rgba(255, 255, 255, 0.3)',
                boxShadow: 'inset 0 1px 0 rgba(255, 255, 255, 0.5)'
              }"
            >
              <button
                v-for="filter in filters"
                :key="filter.type"
                class="flex-1 py-3 px-5 rounded-xl text-sm font-bold transition-all duration-300 hover:scale-105"
                :style="{
                  background: currentFilter === filter.type
                    ? 'linear-gradient(135deg, #6366f1, #8b5cf6)'
                    : 'rgba(255, 255, 255, 0.3)',
                  backdropFilter: currentFilter === filter.type ? 'blur(10px)' : 'none',
                  border: currentFilter === filter.type 
                    ? '1px solid rgba(99, 102, 241, 0.3)' 
                    : '1px solid rgba(255, 255, 255, 0.2)',
                  color: currentFilter === filter.type ? 'white' : 'var(--text-secondary)',
                  boxShadow: currentFilter === filter.type 
                    ? '0 4px 16px rgba(99, 102, 241, 0.25), inset 0 1px 0 rgba(255, 255, 255, 0.3)' 
                    : '0 2px 8px rgba(0, 0, 0, 0.05)'
                }"
                @click="currentFilter = filter.type"
              >
                {{ filter.label }}
              </button>
            </div>

            <!-- 加载状态 -->
            <div v-if="loading" class="flex items-center justify-center py-20">
              <div
                class="w-12 h-12 rounded-full border-4 border-transparent animate-spin"
                :style="{
                  borderTopColor: 'var(--accent-primary)',
                  borderRightColor: 'var(--accent-secondary)'
                }"
              />
            </div>

            <!-- 错误状态 -->
            <div
              v-else-if="error"
              class="rounded-lg p-4 flex items-center space-x-2"
              :style="{
                background: 'rgba(239, 68, 68, 0.1)',
                border: '1px solid var(--accent-danger)'
              }"
            >
              <AlertCircle :style="{ color: 'var(--accent-danger)' }" />
              <span :style="{ color: 'var(--accent-danger)' }">Error: {{ error }}</span>
            </div>

            <!-- 配置卡片列表 -->
            <div v-else class="space-y-6">
              <div v-if="filteredConfigs.length === 0" class="text-center py-10" :style="{ color: 'var(--text-muted)' }">
                当前分类下暂无配置
              </div>
              <ConfigCard
                v-else
                v-for="config in filteredConfigs"
                :key="config.name"
                :config="config"
                @switch="handleSwitch"
                @edit="handleEdit"
                @delete="handleDelete"
              />
            </div>
          </div>

          <!-- 历史记录 Tab -->
          <div v-if="activeTab === 'history'">
            <HistoryList :entries="historyEntries" :loading="historyLoading" />
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { RouterLink } from 'vue-router'
import { useConfigsStore, useUIStore } from '@/store'
import {
  Cloud,
  Sparkles,
  ArrowRight,
  AlertCircle,
  ArrowLeft,
  Code2,
  Settings,
} from 'lucide-vue-next'
import type { ConfigItem, HistoryEntry } from '@/types'
import {
  listConfigs,
  switchConfig,
  validateConfigs as apiValidateConfigs,
  getHistory,
  deleteConfig,
  isTauriEnvironment
} from '@/api'
import ConfigCard from '@/components/ConfigCard.vue'
import HistoryList from '@/components/HistoryList.vue'
import RightSidebar from '@/components/RightSidebar.vue'
import Navbar from '@/components/Navbar.vue'
import StatusHeader from '@/components/StatusHeader.vue'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'
import Breadcrumb from '@/components/Breadcrumb.vue'
import EnvironmentBadge from '@/components/EnvironmentBadge.vue'
import EditConfigModal from '@/components/EditConfigModal.vue'

type FilterType = 'all' | 'official_relay' | 'third_party_model' | 'uncategorized'

const configs = ref<ConfigItem[]>([])
const currentConfig = ref<string>('')
const historyEntries = ref<HistoryEntry[]>([])
const loading = ref(true)
const historyLoading = ref(false)
const error = ref<string | null>(null)
const currentFilter = ref<FilterType>('all')
const activeTab = ref<'configs' | 'history'>('configs')
const isEditModalOpen = ref(false)
const editingConfigName = ref('')

const filters = [
  { type: 'all' as FilterType, label: '📋 全部配置' },
  { type: 'official_relay' as FilterType, label: '🔄 官方中转' },
  { type: 'third_party_model' as FilterType, label: '🤖 第三方模型' },
  { type: 'uncategorized' as FilterType, label: '❓ 未分类' }
]

// 根据当前筛选器过滤配置
const filteredConfigs = computed(() => {
  if (currentFilter.value === 'all') {
    return configs.value
  } else if (currentFilter.value === 'official_relay') {
    return configs.value.filter(
      c => c.provider_type === 'OfficialRelay' || c.provider_type === 'official_relay'
    )
  } else if (currentFilter.value === 'third_party_model') {
    return configs.value.filter(
      c => c.provider_type === 'ThirdPartyModel' || c.provider_type === 'third_party_model'
    )
  } else if (currentFilter.value === 'uncategorized') {
    return configs.value.filter(c => !c.provider_type)
  }
  return configs.value
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
    error.value = err instanceof Error ? err.message : 'Failed to load configs'
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

// 切换配置
const handleSwitch = async (configName: string) => {
  if (!confirm(`确定切换到配置 "${configName}" 吗？`)) return

  try {
    await switchConfig(configName)
    alert(`✓ 成功切换到配置 "${configName}"`)
    await loadConfigs()
    if (activeTab.value === 'history') {
      await loadHistory()
    }
  } catch (err) {
    alert(`切换失败: ${err instanceof Error ? err.message : 'Unknown error'}`)
  }
}

// 验证配置
const handleValidate = async () => {
  try {
    await apiValidateConfigs()
    alert('✓ 配置验证通过')
  } catch (err) {
    alert(`验证失败: ${err instanceof Error ? err.message : 'Unknown error'}`)
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

// 删除配置
const handleDelete = async (configName: string) => {
  if (!confirm(`确定删除配置 "${configName}" 吗？此操作不可恢复！`)) return

  try {
    await deleteConfig(configName)
    alert(`✓ 成功删除配置 "${configName}"`)
    await loadConfigs()
  } catch (err) {
    alert(`删除失败: ${err instanceof Error ? err.message : 'Unknown error'}`)
  }
}

// 清理备份
const handleClean = () => {
  alert('清理备份功能开发中')
}

// 导入配置
const handleImport = () => {
  alert('导入功能开发中')
}

// 导出配置
const handleExport = () => {
  alert('导出功能开发中')
}

// 刷新数据
const refreshData = async () => {
  await loadConfigs()
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
})
</script>