<template>
  <div
    class="min-h-screen relative"
    :style="{ background: 'var(--bg-primary)', padding: '20px' }"
  >
    <!-- 🎨 动态背景装饰 - 液态玻璃风格 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-20 right-20 w-96 h-96 rounded-full opacity-20 blur-3xl animate-pulse"
        :style="{ background: 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)' }"
      />
      <div
        class="absolute bottom-20 left-20 w-96 h-96 rounded-full opacity-20 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, #ec4899 0%, #f59e0b 100%)',
          animationDelay: '1s'
        }"
      />
    </div>

    <div class="max-w-[1800px] mx-auto relative z-10">
      <!-- Breadcrumb Navigation -->
      <Breadcrumb
        :items="[
          { label: '首页', path: '/', icon: Home },
          { label: 'Claude Code', path: '/claude-code', icon: Code2 },
          { label: '配置管理', path: '/configs', icon: Settings }
        ]"
        moduleColor="#6366f1"
      />

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

      <!-- 🌟 WebDAV 云同步功能入口 -->
      <RouterLink to="/sync" class="block mb-5">
        <div
          class="glass-card relative overflow-hidden p-6 cursor-pointer transition-all duration-300 hover:scale-[1.01] group"
          :style="{
            background: 'linear-gradient(135deg, rgba(99, 102, 241, 0.12), rgba(139, 92, 246, 0.08))',
            border: '1.5px solid rgba(99, 102, 241, 0.25)',
            boxShadow: 'var(--shadow-lg), inset 0 1px 0 0 rgba(255, 255, 255, 0.4)'
          }"
        >
          <div class="relative flex items-center justify-between">
            <div class="flex items-center gap-6">
              <!-- 图标区域 -->
              <div class="relative">
                <div
                  class="absolute inset-0 blur-lg opacity-30"
                  :style="{ background: '#6366f1' }"
                />
                <div
                  class="relative z-10 p-4 rounded-2xl"
                  :style="{ background: 'rgba(99, 102, 241, 0.15)' }"
                >
                  <Cloud
                    class="w-10 h-10 group-hover:scale-110 transition-transform"
                    :style="{ color: '#6366f1' }"
                  />
                </div>
                <Sparkles
                  class="w-5 h-5 absolute -top-1 -right-1 animate-pulse"
                  :style="{ color: '#f59e0b' }"
                />
              </div>

              <!-- 文字内容 -->
              <div>
                <div class="flex items-center gap-3 mb-2">
                  <h3
                    class="text-2xl font-bold"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    WebDAV 云同步
                  </h3>
                  <span
                    class="px-3 py-1 rounded-full text-xs font-bold"
                    :style="{
                      background: 'var(--accent-warning)',
                      color: 'white'
                    }"
                  >
                    NEW ✨
                  </span>
                </div>
                <p
                  class="text-sm font-medium mb-3"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  一键同步配置到云端 · 支持坚果云、Nextcloud、ownCloud 等 WebDAV 服务
                </p>
                <div class="flex items-center gap-4">
                  <div
                    class="flex items-center gap-1.5 text-xs"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    <div
                      class="w-1.5 h-1.5 rounded-full animate-pulse"
                      :style="{ background: 'var(--accent-success)' }"
                    />
                    <span>多设备同步</span>
                  </div>
                  <div
                    class="flex items-center gap-1.5 text-xs"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    <div
                      class="w-1.5 h-1.5 rounded-full animate-pulse"
                      :style="{ background: 'var(--accent-info)' }"
                    />
                    <span>自动备份</span>
                  </div>
                  <div
                    class="flex items-center gap-1.5 text-xs"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    <div
                      class="w-1.5 h-1.5 rounded-full animate-pulse"
                      :style="{ background: 'var(--accent-secondary)' }"
                    />
                    <span>安全加密</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- 右侧按钮 -->
            <div class="flex items-center gap-3">
              <div class="text-right mr-4">
                <div
                  class="text-sm font-medium"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  立即体验
                </div>
                <div
                  class="text-xs"
                  :style="{ color: 'var(--text-muted)' }"
                >
                  点击进入管理
                </div>
              </div>
              <div
                class="w-12 h-12 rounded-full flex items-center justify-center transition-all group-hover:scale-110"
                :style="{
                  background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
                  boxShadow: '0 0 20px var(--glow-primary)'
                }"
              >
                <ArrowRight
                  class="w-6 h-6 text-white group-hover:translate-x-1 transition-transform"
                />
              </div>
            </div>
          </div>
        </div>
      </RouterLink>

      <!-- 三列布局：侧边栏 + 主内容 + 右侧边栏 -->
      <div class="grid grid-cols-[auto_1fr_280px] gap-4">
        <!-- 可折叠侧边栏 -->
        <CollapsibleSidebar module="claude-code" />

        <!-- 主内容区 -->
        <main
          class="rounded-xl p-6 glass-effect"
          :style="{
            border: '1px solid var(--border-color)',
            boxShadow: 'var(--shadow-small)'
          }"
        >
          <!-- Tab 导航 -->
          <div
            class="flex gap-1.5 mb-5 p-1 rounded-lg"
            :style="{ background: 'var(--bg-tertiary)' }"
          >
            <button
              class="flex-1 py-2 px-4 rounded-md text-sm font-semibold transition-all"
              :style="{
                background: activeTab === 'configs' ? 'var(--accent-primary)' : 'transparent',
                color: activeTab === 'configs' ? 'white' : 'var(--text-secondary)'
              }"
              @click="activeTab = 'configs'"
            >
              配置列表
            </button>
            <button
              class="flex-1 py-2 px-4 rounded-md text-sm font-semibold transition-all"
              :style="{
                background: activeTab === 'history' ? 'var(--accent-primary)' : 'transparent',
                color: activeTab === 'history' ? 'white' : 'var(--text-secondary)'
              }"
              @click="activeTab = 'history'"
            >
              历史记录
            </button>
          </div>

          <!-- 配置列表 Tab -->
          <div v-if="activeTab === 'configs'">
            <!-- 筛选按钮 -->
            <div
              class="flex gap-2 mb-5 p-2 rounded-lg"
              :style="{
                background: 'var(--bg-tertiary)',
                border: '1px solid var(--border-color)'
              }"
            >
              <button
                v-for="filter in filters"
                :key="filter.type"
                class="flex-1 py-2.5 px-4 rounded-lg text-sm font-semibold transition-all hover:scale-105"
                :style="{
                  background: currentFilter === filter.type
                    ? 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))'
                    : 'transparent',
                  border: `1px solid ${
                    currentFilter === filter.type ? 'var(--accent-primary)' : 'var(--border-color)'
                  }`,
                  color: currentFilter === filter.type ? 'white' : 'var(--text-secondary)',
                  boxShadow: currentFilter === filter.type ? '0 0 15px var(--glow-primary)' : 'none'
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
  getHistory
} from '@/api/client'
import ConfigCard from '@/components/ConfigCard.vue'
import HistoryList from '@/components/HistoryList.vue'
import RightSidebar from '@/components/RightSidebar.vue'
import Navbar from '@/components/Navbar.vue'
import StatusHeader from '@/components/StatusHeader.vue'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'
import Breadcrumb from '@/components/Breadcrumb.vue'

type FilterType = 'all' | 'official_relay' | 'third_party_model' | 'uncategorized'

const configs = ref<ConfigItem[]>([])
const currentConfig = ref<string>('')
const historyEntries = ref<HistoryEntry[]>([])
const loading = ref(true)
const historyLoading = ref(false)
const error = ref<string | null>(null)
const currentFilter = ref<FilterType>('all')
const activeTab = ref<'configs' | 'history'>('configs')

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
  alert(`编辑功能开发中: ${configName}`)
}

// 删除配置
const handleDelete = (configName: string) => {
  if (confirm(`确定删除配置 "${configName}" 吗？此操作不可恢复！`)) {
    alert(`删除功能开发中: ${configName}`)
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