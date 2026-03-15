<template>
  <div class="min-h-screen relative p-6">
    <AnimatedBackground complex />

    <div class="max-w-[1800px] mx-auto space-y-6">
      <!-- Breadcrumb & Nav Header -->
      <div class="flex items-center justify-between">
        <Breadcrumb
          :items="[
            { label: $t('configs.breadcrumb.home'), path: '/', icon: 'Home' },
            { label: $t('configs.breadcrumb.configs'), path: '/configs', icon: 'Settings' }
          ]"
        />
        <EnvironmentBadge />
      </div>

      <!-- Module Navigation (Glass Pills) -->
      <nav class="flex flex-wrap gap-2 p-1.5 rounded-full bg-white/5/40 backdrop-blur-md border border-white/10 w-fit">
        <RouterLink
          v-for="navItem in moduleNavItems"
          :key="navItem.path"
          :to="navItem.path"
          class="flex items-center gap-2 px-4 py-2 rounded-full text-sm font-medium transition-colors duration-300 border border-transparent"
          :class="$route.path === navItem.path 
            ? 'bg-accent-primary/20 text-accent-primary border-accent-primary/20 shadow-glow-primary' 
            : 'text-white/80 hover:text-white hover:bg-white/5'"
        >
          <SIcon
            :name="navItem.icon || ''"
            size="w-4 h-4"
          />
          <span>{{ navItem.label }}</span>
        </RouterLink>
      </nav>

      <!-- Main Content Layout -->
      <div 
        class="grid grid-cols-1 gap-6 transition-all duration-300"
        :class="sidebarCollapsed ? 'lg:grid-cols-[48px_1fr]' : 'lg:grid-cols-[280px_1fr]'"
      >
        <!-- Left Panel: Sidebar -->
        <div class="lg:order-first">
          <RightSidebar
            :configs="configs"
            :current-filter="currentFilter"
            :collapsed="sidebarCollapsed"
            @config-click="handleConfigClick"
            @toggle-collapse="sidebarCollapsed = !sidebarCollapsed"
          />
        </div>

        <!-- Right Panel: Main Content -->
        <Card
          variant="glass"
          glow
          class="p-6 h-fit min-h-[600px] flex flex-col lg:order-last"
        >
          <!-- Tab Navigation -->
          <div class="flex gap-4 border-b border-white/5 pb-4 mb-6">
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
                :name="tab.icon || ''"
                size="w-4 h-4"
              />
              {{ tab.label }}
            </button>
          </div>

          <!-- Configs View -->
          <div
            v-show="activeTab === 'configs'"
            class="space-y-6"
          >
            <!-- Actions & Filters -->
            <div class="flex flex-col md:flex-row md:items-center justify-between gap-4">
              <ConfigFilters
                v-model:current-filter="currentFilter"
                v-model:current-sort="currentSort"
                @show-provider-stats="showProviderModal = true"
                @add-config="isAddModalOpen = true"
              />
               
              <div class="flex gap-2">
                <Button
                  size="sm"
                  variant="ghost"
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

            <!-- List -->
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

          <!-- History View -->
          <div v-show="activeTab === 'history'">
            <HistoryList
              :entries="historyEntries"
              :loading="historyLoading"
            />
          </div>
        </Card>
      </div>
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
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import { Breadcrumb } from '@/components/ui'
import EnvironmentBadge from '@/components/EnvironmentBadge.vue'
import HistoryList from '@/components/HistoryList.vue'
import RightSidebar from '@/components/RightSidebar.vue'
import ConfigFilters from '@/components/configs/ConfigFilters.vue'
import ConfigList from '@/components/configs/ConfigList.vue'
import EditConfigModal from '@/components/EditConfigModal.vue'
import AddConfigModal from '@/components/AddConfigModal.vue'
import ProviderStatsModal from '@/components/configs/ProviderStatsModal.vue'

// API Imports
import {
  listConfigs, switchConfig,
  getHistory, deleteConfig, enableConfig, disableConfig
} from '@/api'
import { getProviderUsage } from '@/api'
import type { ConfigItem, ConfigListResponse, HistoryEntry, HistoryResponse } from '@/types'
import { useUIStore } from '@/stores/ui'
import { logger } from '@/utils/logger'

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
const sidebarCollapsed = ref(false)

// Modals
const isEditModalOpen = ref(false)
const editingConfigName = ref('')
const isAddModalOpen = ref(false)
const showProviderModal = ref(false)
const providerUsage = ref<Record<string, number>>({})
const providerLoading = ref(false)
const providerError = ref<string | null>(null)
const providerSortMode = ref<SortMode>('count_desc')

const tabs: Array<{ id: TabId; label: string; icon: string | string }> = [
  { id: 'configs', label: t('configs.tabs.configList'), icon: 'Settings' },
  { id: 'history', label: t('configs.tabs.history'), icon: 'History' },
]

const moduleNavItems = [
  { path: '/configs', label: 'Configs', icon: 'Settings' },
  { path: '/sync', label: 'Sync', icon: 'Cloud' },
  { path: '/mcp', label: 'MCP', icon: 'Server' },
  { path: '/slash-commands', label: 'Slash', icon: 'Command' },
  { path: '/agents', label: 'Agents', icon: 'Bot' },
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

  if (currentSort.value === 'usage_count') {
    list.sort((a, b) => (b.usage_count || 0) - (a.usage_count || 0))
  } else if (currentSort.value === 'recent') {
    list.sort((a, _b) => (a.is_current ? -1 : 1))
  } else {
    list.sort((a, b) => a.name.localeCompare(b.name))
  }
  
  return list
})

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

// Handlers (Simplified for brevity, logic same as before)
const handleSwitch = async (name: string) => {
  if (confirm(`Switch to ${name}?`)) {
    try {
      await switchConfig(name)
      uiStore.showSuccess(`Switched to configuration ${name}`)
      refreshData()
    } catch (e: unknown) {
      uiStore.showError(e instanceof Error ? e.message : 'Failed to switch configuration')
    }
  }
}

const handleEdit = (name: string) => { editingConfigName.value = name; isEditModalOpen.value = true }

const handleDelete = async (name: string) => { 
  if(confirm('Delete?')) { 
    try {
      await deleteConfig(name); 
      uiStore.showSuccess(`Configuration ${name} deleted`)
      refreshData() 
    } catch (e: unknown) {
      uiStore.showError(e instanceof Error ? e.message : 'Failed to delete configuration')
    }
  } 
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
