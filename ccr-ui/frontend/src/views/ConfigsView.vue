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
          module-color="#6366f1"
        />

        <!-- Environment Badge -->
        <div class="mb-4">
          <EnvironmentBadge />
        </div>

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
              <!-- 筛选和排序控制栏 -->
              <div class="flex gap-4 mb-6 items-center">
                <!-- 筛选按钮 - 液态玻璃风格 -->
                <div
                  class="flex gap-3 flex-1 p-2 rounded-2xl"
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

                <!-- 📊 排序下拉菜单 + 提供商统计按钮 -->
                <div class="flex items-center gap-3">
                  <div class="flex items-center gap-2">
                    <label
                      class="text-sm font-medium whitespace-nowrap"
                      :style="{ color: 'var(--text-secondary)' }"
                    >
                      排序:
                    </label>
                    <select
                      v-model="currentSort"
                      class="px-4 py-3 rounded-xl text-sm font-semibold transition-all cursor-pointer outline-none"
                      :style="{
                        background: 'rgba(255, 255, 255, 0.6)',
                        backdropFilter: 'blur(10px)',
                        border: '1px solid rgba(99, 102, 241, 0.3)',
                        color: 'var(--text-primary)',
                        boxShadow: '0 2px 8px rgba(99, 102, 241, 0.1)'
                      }"
                    >
                      <option value="name">
                        📝 名称
                      </option>
                      <option value="usage_count">
                        📊 使用次数
                      </option>
                      <option value="recent">
                        🕒 最近使用
                      </option>
                    </select>
                  </div>
                  <button
                    class="px-3 py-2 rounded-xl text-xs font-semibold flex items-center gap-1 border border-indigo-200/80 dark:border-indigo-500/70 text-indigo-700 dark:text-indigo-200 bg-white/70 dark:bg-slate-900/70 hover:bg-white dark:hover:bg-slate-800/80 transition shadow-sm"
                    @click="showProviderModal = true"
                  >
                    <svg
                      class="w-4 h-4"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M4 6h16M4 12h10M4 18h6"
                      />
                    </svg>
                    <span>提供商统计</span>
                  </button>
                </div>
              </div>
              <!-- 加载状态 -->
              <div
                v-if="loading"
                class="flex items-center justify-center py-20"
              >
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

              <!-- 提供商统计 + 配置卡片列表 -->
              <div
                v-else
                class="space-y-8"
              >
                <!-- 配置卡片列表 -->
                <div class="space-y-6">
                  <div
                    v-if="filteredConfigs.length === 0"
                    class="text-center py-10"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    当前分类下暂无配置
                  </div>
                  <ConfigCard
                    v-for="config in filteredConfigs"
                    v-else
                    :key="config.name"
                    :config="config"
                    @switch="handleSwitch"
                    @edit="handleEdit"
                    @delete="handleDelete"
                    @enable="handleEnable"
                    @disable="handleDisable"
                  />
                </div>
              </div>
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

    <!-- 提供商统计模态框 -->
    <div
      v-if="showProviderModal"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      @click.self="showProviderModal = false"
    >
      <div
        class="w-full max-w-5xl mx-4 rounded-3xl p-6"
        :style="{
          background: 'rgba(255, 255, 255, 0.95)',
          backdropFilter: 'blur(24px) saturate(180%)',
          WebkitBackdropFilter: 'blur(24px) saturate(180%)',
          border: '1px solid rgba(148, 163, 184, 0.6)',
          boxShadow: '0 20px 60px rgba(15, 23, 42, 0.4)'
        }"
      >
        <div class="flex items-center justify-between mb-4 gap-4">
          <div>
            <h2 class="text-lg font-bold text-slate-900 dark:text-slate-50 flex items-center gap-2">
              <span>🏢 提供商使用统计</span>
            </h2>
            <p class="mt-1 text-xs text-slate-500 dark:text-slate-400">
              共 {{ providerEntries.length }} 个提供商 · 总调用 {{ totalProviderUsage }} 次
            </p>
          </div>
          <div class="flex items-center gap-3">
            <div class="flex items-center gap-1 text-xs text-slate-500 dark:text-slate-400">
              <span>排序:</span>
              <select
                v-model="providerSortMode"
                class="px-3 py-1.5 rounded-xl text-xs font-medium border border-slate-200/70 dark:border-slate-600/70 bg-white/70 dark:bg-slate-900/70 text-slate-700 dark:text-slate-200 outline-none cursor-pointer"
              >
                <option value="count_desc">
                  使用次数 ↓
                </option>
                <option value="count_asc">
                  使用次数 ↑
                </option>
                <option value="name_asc">
                  供应商 A-Z
                </option>
              </select>
            </div>
            <button
              class="px-3 py-1.5 rounded-xl text-xs font-semibold flex items-center gap-1 text-slate-700 dark:text-slate-100 border border-slate-200/80 dark:border-slate-500/80 bg-white/70 dark:bg-slate-900/70 hover:bg-white dark:hover:bg-slate-800/80 transition"
              :disabled="providerLoading"
              @click="loadProviderUsage"
            >
              <svg
                class="w-3.5 h-3.5"
                :class="{ 'animate-spin': providerLoading }"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                />
              </svg>
              <span>刷新统计</span>
            </button>
            <button
              class="w-8 h-8 rounded-full flex items-center justify-center text-slate-500 hover:text-slate-900 dark:hover:text-slate-100 hover:bg-slate-100/70 dark:hover:bg-slate-800/70 transition"
              @click="showProviderModal = false"
            >
              ✕
            </button>
          </div>
        </div>

        <div
          v-if="providerLoading"
          class="flex items-center justify-center py-10"
        >
          <div class="w-10 h-10 rounded-full border-4 border-transparent border-t-indigo-500 border-r-fuchsia-500 animate-spin" />
        </div>
        <div
          v-else-if="providerError"
          class="text-xs rounded-lg px-3 py-2 border border-red-300/70 bg-red-50/80 text-red-700 dark:border-red-700/70 dark:bg-red-900/30 dark:text-red-100"
        >
          加载提供商统计失败: {{ providerError }}
        </div>
        <div
          v-else-if="sortedProviderEntries.length === 0"
          class="text-center text-xs text-slate-500 dark:text-slate-400 py-8"
        >
          暂无提供商使用数据，开始使用 AI API 后，这里会显示各提供商的调用次数
        </div>
        <div
          v-else
          class="provider-chart-container"
        >
          <div class="provider-chart-summary">
            共 {{ providerEntries.length }} 个提供商 · 总调用 {{ totalProviderUsage }} 次
            <div class="provider-chart-desc">
              Y 轴：使用次数（单位：次） · X 轴：提供商 · 当前排序：{{ providerSortLabel }}
            </div>
          </div>
          <div class="provider-chart">
            <div class="provider-chart-y-grid">
              <div
                v-for="tick in providerYTicks"
                :key="tick.percent"
                class="y-grid-line"
                :style="{ bottom: tick.percent + '%' }"
              >
                <span class="y-grid-label">{{ tick.value }}</span>
              </div>
            </div>
            <div class="provider-chart-y-axis-line" />
            <div class="provider-chart-x-axis-line" />
            <div class="provider-chart-bars">
              <div
                v-for="([provider, count], index) in sortedProviderEntries"
                :key="provider || index"
                class="provider-bar-column"
                :style="{ animation: 'configFadeIn 0.4s ease ' + index * 0.05 + 's backwards' }"
              >
                <div class="provider-bar-vertical-bg">
                  <div
                    class="provider-bar-vertical-fill"
                    :class="'bar-color-' + (index % 5)"
                    :style="{ height: Math.max((count / (maxProviderCount || 1)) * 100, 8) + '%' }"
                    :title="(provider || 'unknown') + ': ' + count + ' 次，占最高值的 ' + (maxProviderCount ? ((count / maxProviderCount) * 100).toFixed(1) : 0) + '%'"
                  />
                </div>
                <div class="provider-bar-value">
                  {{ count }} 次
                </div>
                <div
                  class="provider-bar-label"
                  :title="provider || 'unknown'"
                >
                  {{ provider || 'unknown' }}
                </div>
              </div>
            </div>
          </div>
          <div class="provider-chart-x-axis-label">
            X 轴：提供商
          </div>
        </div>
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
  getHistory,
  deleteConfig,
  enableConfig,
  disableConfig,
  isTauriEnvironment
} from '@/api'
import { getProviderUsage } from '@/api/client'
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

const providerUsage = ref<Record<string, number>>({})
const providerLoading = ref(false)
const providerError = ref<string | null>(null)
const providerSortMode = ref<'count_desc' | 'count_asc' | 'name_asc'>('count_desc')
const showProviderModal = ref(false)

const filters = [
  { type: 'all' as FilterType, label: '📋 全部配置' },
  { type: 'official_relay' as FilterType, label: '🔄 官方中转' },
  { type: 'third_party_model' as FilterType, label: '🤖 第三方模型' },
  { type: 'uncategorized' as FilterType, label: '❓ 未分类' }
]

const providerEntries = computed(() => {
  return Object.entries(providerUsage.value || {})
})

const sortedProviderEntries = computed(() => {
  const entries = [...providerEntries.value]
  if (providerSortMode.value === 'count_asc') {
    entries.sort((a, b) => a[1] - b[1])
  } else if (providerSortMode.value === 'name_asc') {
    entries.sort((a, b) => a[0].localeCompare(b[0]))
  } else {
    entries.sort((a, b) => b[1] - a[1])
  }
  return entries
})

const providerSortLabel = computed(() => {
  if (providerSortMode.value === 'count_asc') {
    return '使用次数（从低到高）'
  }
  if (providerSortMode.value === 'name_asc') {
    return '供应商名称（A → Z）'
  }
  return '使用次数（从高到低）'
})

const maxProviderCount = computed(() => {
  const values = providerEntries.value.map(([, count]) => count)
  return values.length ? Math.max(...values) : 0
})

const totalProviderUsage = computed(() => {
  return providerEntries.value.reduce((sum, [, count]) => sum + count, 0)
})

const providerYTicks = computed(() => {
  const max = maxProviderCount.value || 0
  const percents = [0, 25, 50, 75, 100]
  if (max === 0) {
    return percents.map(percent => ({ percent, value: 0 }))
  }
  return percents.map(percent => ({
    percent,
    value: Math.round((max * percent) / 100),
  }))
})

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

const loadProviderUsage = async () => {
  try {
    providerLoading.value = true
    providerError.value = null
    const data = await getProviderUsage()
    providerUsage.value = data || {}
  } catch (err) {
    providerError.value = err instanceof Error ? err.message : 'Failed to load provider usage'
    console.error('Error loading provider usage:', err)
  } finally {
    providerLoading.value = false
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

// 📊 启用配置
const handleEnable = async (configName: string) => {
  if (!confirm(`确定启用配置 "${configName}" 吗？`)) return

  try {
    await enableConfig(configName)
    alert(`✓ 成功启用配置 "${configName}"`)
    await loadConfigs()
  } catch (err) {
    alert(`启用失败: ${err instanceof Error ? err.message : 'Unknown error'}`)
  }
}

// 📊 禁用配置
const handleDisable = async (configName: string) => {
  if (!confirm(`确定禁用配置 "${configName}" 吗？禁用后该配置将无法使用。`)) return

  try {
    await disableConfig(configName)
    alert(`✓ 成功禁用配置 "${configName}"`)
    await loadConfigs()
  } catch (err) {
    alert(`禁用失败: ${err instanceof Error ? err.message : 'Unknown error'}`)
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