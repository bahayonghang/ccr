<template>
  <div class="min-h-full relative p-6">
    <AnimatedBackground
      contained
      variant="minimal"
    />

    <div class="relative z-10 mx-auto max-w-[1800px] space-y-6">
      <div class="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
        <ModuleSubnav
          module="claude-code"
          class="flex-1"
        />
        <div class="flex justify-end">
          <EnvironmentBadge />
        </div>
      </div>

      <Card
        surface="workspace"
        :elevation="2"
        motion="subtle"
        density="compact"
        glow
        class="p-6 h-fit min-h-[600px] flex flex-col"
      >
        <div class="space-y-6">
          <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
            <button
              v-for="summary in configSummary"
              :key="summary.key"
              type="button"
              class="rounded-2xl border px-4 py-4 text-left transition-[border-color,background-color,transform] duration-200 hover:-translate-y-0.5"
              :class="summary.key === currentFilter ? summary.activeClass : summary.idleClass"
              @click="currentFilter = summary.key"
            >
              <p class="text-xs font-semibold uppercase tracking-[0.18em] opacity-70">
                {{ summary.label }}
              </p>
              <div class="mt-3 flex items-end justify-between gap-3">
                <span class="text-3xl font-bold leading-none">{{ summary.count }}</span>
                <SIcon
                  :name="summary.icon"
                  size="w-5 h-5"
                  class="shrink-0 opacity-80"
                />
              </div>
            </button>
          </div>

          <div class="rounded-2xl border border-border-default/50 bg-bg-elevated/70 p-4 backdrop-blur-md">
            <div class="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
              <div class="space-y-2">
                <p class="text-xs font-semibold uppercase tracking-[0.18em] text-text-muted">
                  {{ t('configs.description') }}
                </p>
                <div class="flex flex-wrap items-center gap-2 text-sm text-text-secondary">
                  <span class="rounded-full border border-accent-primary/20 bg-accent-primary/10 px-3 py-1 font-medium text-accent-primary">
                    {{ t('configs.currentConfig') }}: {{ currentConfigName }}
                  </span>
                  <span class="rounded-full border border-border-default/50 bg-bg-surface/70 px-3 py-1 font-medium">
                    {{ filteredConfigs.length }} / {{ configs.length }} {{ t('configs.availableConfigs') }}
                  </span>
                </div>
              </div>

              <label class="relative block w-full xl:max-w-md">
                <SIcon
                  name="Search"
                  size="w-4 h-4"
                  class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-text-muted"
                />
                <input
                  v-model="searchQuery"
                  type="text"
                  :placeholder="t('common.search')"
                  class="w-full rounded-xl border border-border-default/60 bg-bg-primary/80 py-2.5 pl-10 pr-4 text-sm text-text-primary outline-none transition-[border-color,box-shadow] duration-200 placeholder:text-text-muted focus:border-accent-primary/50 focus:ring-2 focus:ring-accent-primary/20"
                >
              </label>
            </div>

            <div
              v-if="quickJumpConfigs.length > 0"
              class="mt-4 flex flex-wrap gap-2"
            >
              <button
                v-for="config in quickJumpConfigs"
                :key="config.name"
                type="button"
                class="inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-sm transition-[border-color,background-color,color] duration-200"
                :class="config.is_current
                  ? 'border-accent-primary/30 bg-accent-primary/10 text-accent-primary'
                  : 'border-border-default/50 bg-bg-surface/60 text-text-secondary hover:border-accent-primary/20 hover:text-text-primary'"
                @click="handleConfigClick(config.name)"
              >
                <span class="truncate max-w-[180px]">{{ config.name }}</span>
                <span
                  class="rounded-full px-1.5 py-0.5 text-[11px] font-semibold"
                  :class="config.is_current ? 'bg-accent-primary/15 text-accent-primary' : 'bg-bg-elevated/80 text-text-muted'"
                >
                  {{ config.usage_count || 0 }}
                </span>
              </button>
            </div>
          </div>

          <div class="flex gap-4 border-b border-white/5 pb-4">
            <button
              v-for="tab in tabs"
              :key="tab.id"
              class="flex items-center gap-2 pb-2 px-2 text-sm font-bold border-b-2 transition-colors duration-300"
              :class="activeTab === tab.id
                ? 'border-accent-primary text-accent-primary'
                : 'border-transparent text-white/50 hover:text-white/80'"
              @click="activeTab = tab.id"
            >
              <SIcon
                :name="tab.icon"
                size="w-4 h-4"
              />
              {{ tab.label }}
            </button>
          </div>

          <div
            v-show="activeTab === 'configs'"
            class="space-y-6"
          >
            <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
              <ConfigFilters
                v-model:current-filter="currentFilter"
                v-model:current-sort="currentSort"
                @show-provider-stats="showProviderModal = true"
                @add-config="isAddModalOpen = true"
              />

              <div class="flex gap-2">
                <Button
                  variant="ghost"
                  density="compact"
                  surface="status"
                  motion="subtle"
                  @click="refreshData"
                >
                  <SIcon
                    name="RefreshCw"
                    size="w-4 h-4"
                    :class="{ 'animate-spin': loading }"
                  />
                </Button>
              </div>
            </div>

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

          <div v-show="activeTab === 'history'">
            <HistoryList
              :entries="historyEntries"
              :loading="historyLoading"
            />
          </div>
        </div>
      </Card>
    </div>

    <!-- Modals -->
    <EditConfigModal
      :is-open="isEditModalOpen"
      :config-name="editingConfigName"
      @close="isEditModalOpen = false"
      @saved="refreshData"
    />
    <AddConfigModal
      :is-open="isAddModalOpen"
      @close="isAddModalOpen = false"
      @saved="refreshData"
    />
    <ProviderStatsModal
      v-model:sort-mode="providerSortMode"
      :visible="showProviderModal"
      :provider-usage="providerUsage"
      :loading="providerLoading"
      :error="providerError"
      @close="showProviderModal = false"
      @refresh="loadProviderUsage"
    />
    <ConfirmModal
      v-model:is-open="showConfirmModal"
      :type="confirmDialog.type"
      :title="confirmDialog.title"
      :message="confirmDialog.message"
      :confirm-text="confirmDialog.confirmText"
      :cancel-text="$t('common.cancel')"
      @confirm="executeConfirmedAction"
    />
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, reactive, computed, onMounted, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import EnvironmentBadge from '@/components/EnvironmentBadge.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import HistoryList from '@/components/HistoryList.vue'
import ConfigFilters from '@/components/configs/ConfigFilters.vue'
import ConfigList from '@/components/configs/ConfigList.vue'
import { translateWithFallback } from '@/i18n/formatMessage'
import EditConfigModal from '@/components/EditConfigModal.vue'
import AddConfigModal from '@/components/AddConfigModal.vue'
import ProviderStatsModal from '@/components/configs/ProviderStatsModal.vue'
import ConfirmModal from '@/components/ConfirmModal.vue'

// API Imports
import {
  listConfigs, switchConfig,
  getHistory, deleteConfig, enableConfig, disableConfig
} from '@/api'
import { getProviderUsage } from '@/api'
import type { ConfigItem, ConfigListResponse, HistoryEntry, HistoryResponse } from '@/types'
import { useUIStore } from '@/stores/ui'
import { logger } from '@/utils/logger'

defineOptions({ name: 'ConfigsView' })

type FilterType = 'all' | 'official_relay' | 'third_party_model' | 'uncategorized'
type SortType = 'name' | 'usage_count' | 'recent'
type SortMode = 'count_desc' | 'count_asc' | 'name_asc'
type TabId = 'configs' | 'history'

const { t } = useI18n()
const uiStore = useUIStore()

// Data
const configs = ref<ConfigItem[]>([])
const historyEntries = ref<HistoryEntry[]>([])
const loading = ref(true)
const historyLoading = ref(false)
const error = ref<string | null>(null)
const activeTab = ref<TabId>('configs')
const currentFilter = ref<FilterType>('all')
const currentSort = ref<SortType>('name')
const searchQuery = ref('')

// Modals
const isEditModalOpen = ref(false)
const editingConfigName = ref('')
const isAddModalOpen = ref(false)
const showProviderModal = ref(false)
const providerUsage = ref<Record<string, number>>({})
const providerLoading = ref(false)
const providerError = ref<string | null>(null)
const providerSortMode = ref<SortMode>('count_desc')
const showConfirmModal = ref(false)
const confirmDialog = reactive<{
  title: string
  message: string
  confirmText: string
  type: 'danger' | 'info' | 'warning'
}>({
  title: '',
  message: '',
  confirmText: '',
  type: 'warning',
})
let confirmedAction: (() => Promise<void>) | null = null

const tabs: Array<{ id: TabId; label: string; icon: string | string }> = [
  { id: 'configs', label: t('configs.tabs.configList'), icon: 'Settings' },
  { id: 'history', label: t('configs.tabs.history'), icon: 'History' },
]

// Computed
const filteredConfigs = computed(() => {
  let list = [...configs.value]
  
  if (currentFilter.value !== 'all') {
    list = list.filter(c => {
       if (currentFilter.value === 'official_relay') return c.provider_type?.toLowerCase().includes('official')
       if (currentFilter.value === 'third_party_model') return c.provider_type?.toLowerCase().includes('third')
       return true
    })
  }

  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase().trim()
    list = list.filter(config =>
      config.name.toLowerCase().includes(query) ||
      config.provider?.toLowerCase().includes(query) ||
      config.model?.toLowerCase().includes(query)
    )
  }

  if (currentSort.value === 'usage_count') {
    list.sort((a, b) => (b.usage_count || 0) - (a.usage_count || 0))
  } else if (currentSort.value === 'recent') {
    list.sort((a, _b) => (a.is_current ? -1 : 1))
  } else {
    list.sort((a, b) => a.name.localeCompare(b.name))
  }
  
  return list
})

const currentConfigName = computed(() => {
  return configs.value.find(config => config.is_current)?.name ?? t('configs.noCurrentConfig')
})

const quickJumpConfigs = computed(() => {
  const current = filteredConfigs.value.filter(config => config.is_current)
  const rest = filteredConfigs.value.filter(config => !config.is_current)
  return [...current, ...rest].slice(0, 8)
})

const configSummary = computed(() => [
  {
    key: 'all' as FilterType,
    label: t('configs.filters.all'),
    count: configs.value.length,
    icon: 'LayoutGrid',
    activeClass: 'border-emerald-400/30 bg-emerald-400/10 text-emerald-300',
    idleClass: 'border-border-default/50 bg-bg-surface/60 text-text-secondary hover:border-emerald-400/20 hover:text-text-primary',
  },
  {
    key: 'official_relay' as FilterType,
    label: t('configs.filters.officialRelay'),
    count: configs.value.filter(config => config.provider_type?.toLowerCase().includes('official')).length,
    icon: 'Zap',
    activeClass: 'border-cyan-400/30 bg-cyan-400/10 text-cyan-300',
    idleClass: 'border-border-default/50 bg-bg-surface/60 text-text-secondary hover:border-cyan-400/20 hover:text-text-primary',
  },
  {
    key: 'third_party_model' as FilterType,
    label: t('configs.filters.thirdPartyModel'),
    count: configs.value.filter(config => config.provider_type?.toLowerCase().includes('third')).length,
    icon: 'Cpu',
    activeClass: 'border-violet-400/30 bg-violet-400/10 text-violet-300',
    idleClass: 'border-border-default/50 bg-bg-surface/60 text-text-secondary hover:border-violet-400/20 hover:text-text-primary',
  },
  {
    key: 'uncategorized' as FilterType,
    label: t('configs.filters.uncategorized'),
    count: configs.value.filter(config => !config.provider_type).length,
    icon: 'HelpCircle',
    activeClass: 'border-amber-400/30 bg-amber-400/10 text-amber-300',
    idleClass: 'border-border-default/50 bg-bg-surface/60 text-text-secondary hover:border-amber-400/20 hover:text-text-primary',
  },
])

// Methods
const loadConfigs = async () => {
  loading.value = true
  try {
    const data = await listConfigs<ConfigListResponse>()
    configs.value = data.configs
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : String(e)
    error.value = message
    uiStore.showError(`${t('configs.operationFailed')}: ${message}`)
  }
  finally { loading.value = false }
}

const loadHistory = async () => {
  historyLoading.value = true
  try {
    const data = await getHistory<HistoryResponse>()
    historyEntries.value = data.entries
  } catch (e: unknown) {
    logger.error('Failed to load history', e)
    const message = e instanceof Error ? e.message : String(e)
    uiStore.showError(`Failed to load history: ${message}`)
  }
  finally { historyLoading.value = false }
}

const loadProviderUsage = async () => {
  providerLoading.value = true
  try {
    providerUsage.value = (await getProviderUsage<Record<string, number>>()) || {}
  } catch (e: unknown) {
    providerError.value = e instanceof Error ? e.message : String(e)
  }
  finally { providerLoading.value = false }
}

const refreshData = async () => {
  await loadConfigs()
  await loadProviderUsage()
  if (activeTab.value === 'history') await loadHistory()
}

const openConfirmDialog = (options: {
  title: string
  message: string
  confirmText: string
  type: 'danger' | 'info' | 'warning'
  action: () => Promise<void>
}) => {
  confirmDialog.title = options.title
  confirmDialog.message = options.message
  confirmDialog.confirmText = options.confirmText
  confirmDialog.type = options.type
  confirmedAction = options.action
  showConfirmModal.value = true
}

const executeConfirmedAction = async () => {
  if (!confirmedAction) return
  try {
    await confirmedAction()
  } finally {
    confirmedAction = null
  }
}

// Handlers (Simplified for brevity, logic same as before)
const handleSwitch = async (name: string) => {
  openConfirmDialog({
    title: t('configs.switchConfig'),
    message: translateWithFallback(
      t,
      'configs.confirmSwitch',
      '确定切换到配置 "{name}" 吗？',
      { name },
    ),
    confirmText: t('configs.switchConfig'),
    type: 'warning',
    action: async () => {
      try {
        await switchConfig(name)
        uiStore.showSuccess(`Switched to configuration ${name}`)
        refreshData()
      } catch (e: unknown) {
        uiStore.showError(e instanceof Error ? e.message : 'Failed to switch configuration')
      }
    },
  })
}

const handleEdit = (name: string) => { editingConfigName.value = name; isEditModalOpen.value = true }

const handleDelete = async (name: string) => {
  openConfirmDialog({
    title: t('common.delete'),
    message: translateWithFallback(
      t,
      'configs.confirmDelete',
      '确认删除配置 "{name}" 吗？',
      { name },
    ),
    confirmText: t('common.delete'),
    type: 'danger',
    action: async () => {
      try {
        await deleteConfig(name)
        uiStore.showSuccess(`Configuration ${name} deleted`)
        refreshData()
      } catch (e: unknown) {
        uiStore.showError(e instanceof Error ? e.message : 'Failed to delete configuration')
      }
    },
  })
}

const handleEnable = async (name: string) => { 
  try {
    await enableConfig(name); 
    uiStore.showSuccess(`Configuration ${name} enabled`)
    refreshData() 
  } catch (e: unknown) {
    uiStore.showError(e instanceof Error ? e.message : 'Failed to enable configuration')
  }
}

const handleDisable = async (name: string) => { 
  try {
    await disableConfig(name); 
    uiStore.showSuccess(`Configuration ${name} disabled`)
    refreshData() 
  } catch (e: unknown) {
    uiStore.showError(e instanceof Error ? e.message : 'Failed to disable configuration')
  }
}
const handleConfigClick = async (name: string) => {
  await nextTick()

  const targetCard = document.querySelector(`[data-config-name="${name}"]`)
  if (targetCard) {
    targetCard.scrollIntoView({ behavior: 'smooth', block: 'nearest' })

    // 添加高亮动画
    targetCard.classList.add('highlight-pulse')
    setTimeout(() => targetCard.classList.remove('highlight-pulse'), 1500)
  }
}

watch(activeTab, (val) => { if (val === 'history') loadHistory() })
onMounted(refreshData)
</script>
