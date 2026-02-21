<template>
  <div class="min-h-screen relative p-6">
    <AnimatedBackground complex />

    <div class="max-w-[1800px] mx-auto space-y-6">
      <!-- Breadcrumb & Nav Header -->
      <div class="flex items-center justify-between">
        <Breadcrumb
          :items="[
            { label: $t('configs.breadcrumb.home'), path: '/', icon: Home },
            { label: $t('configs.breadcrumb.configs'), path: '/configs', icon: Settings }
          ]"
        />
        <EnvironmentBadge />
      </div>

      <!-- Module Navigation (Glass Pills) -->
      <nav class="flex flex-wrap gap-2 p-1.5 rounded-full bg-bg-elevated/40 backdrop-blur-md border border-white/10 w-fit">
        <RouterLink
          v-for="navItem in moduleNavItems"
          :key="navItem.path"
          :to="navItem.path"
          class="flex items-center gap-2 px-4 py-2 rounded-full text-sm font-medium transition-all duration-300 border border-transparent"
          :class="$route.path === navItem.path 
            ? 'bg-accent-primary/20 text-accent-primary border-accent-primary/20 shadow-glow-primary' 
            : 'text-text-secondary hover:text-text-primary hover:bg-white/5'"
        >
          <component
            :is="navItem.icon"
            class="w-4 h-4"
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
          <div class="flex gap-4 border-b border-border-subtle pb-4 mb-6">
            <button
              v-for="tab in tabs"
              :key="tab.id"
              class="flex items-center gap-2 pb-2 px-2 text-sm font-bold border-b-2 transition-all duration-300"
              :class="activeTab === tab.id 
                ? 'border-accent-primary text-accent-primary' 
                : 'border-transparent text-text-muted hover:text-text-secondary'"
              @click="activeTab = tab.id as any"
            >
              <component
                :is="tab.icon"
                class="w-4 h-4"
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
                  <RefreshCw
                    class="w-4 h-4"
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
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  Settings, Home, Cloud, Server, Command, Bot, History,
  RefreshCw
} from 'lucide-vue-next'
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
import { getProviderUsage } from '@/api/client'
import type { ConfigItem, HistoryEntry } from '@/types'
import { useUIStore } from '@/stores/ui'

const { t } = useI18n()
const uiStore = useUIStore()

// Data
const configs = ref<ConfigItem[]>([])
const historyEntries = ref<HistoryEntry[]>([])
const loading = ref(true)
const historyLoading = ref(false)
const error = ref<string | null>(null)
const activeTab = ref<'configs' | 'history'>('configs')
const currentFilter = ref<any>('all')
const currentSort = ref('name') as any
const sidebarCollapsed = ref(false)

// Modals
const isEditModalOpen = ref(false)
const editingConfigName = ref('')
const isAddModalOpen = ref(false)
const showProviderModal = ref(false)
const providerUsage = ref({})
const providerLoading = ref(false)
const providerError = ref(null)
const providerSortMode = ref('count_desc') as any

const tabs = [
  { id: 'configs', label: t('configs.tabs.configList'), icon: Settings },
  { id: 'history', label: t('configs.tabs.history'), icon: History },
]

const moduleNavItems = [
  { path: '/configs', label: 'Configs', icon: Settings },
  { path: '/sync', label: 'Sync', icon: Cloud },
  { path: '/mcp', label: 'MCP', icon: Server },
  { path: '/slash-commands', label: 'Slash', icon: Command },
  { path: '/agents', label: 'Agents', icon: Bot },
]

// Computed
const filteredConfigs = computed(() => {
  let list = [...configs.value]
  
  if (currentFilter.value !== 'all') {
    list = list.filter(c => {
       if (currentFilter.value === 'official_relay') return c.provider_type?.toLowerCase().includes('official')
       if (currentFilter.value === 'third_party') return c.provider_type?.toLowerCase().includes('third')
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
    const data = await listConfigs()
    configs.value = data.configs
  } catch (e: any) { 
    error.value = e.message 
    uiStore.showError(t('configs.operationFailed') + ': ' + e.message)
  }
  finally { loading.value = false }
}

const loadHistory = async () => {
  historyLoading.value = true
  try {
    const data = await getHistory()
    historyEntries.value = data.entries
  } catch (e: any) { 
    console.error(e) 
    uiStore.showError('Failed to load history: ' + e.message)
  }
  finally { historyLoading.value = false }
}

const loadProviderUsage = async () => {
  providerLoading.value = true
  try {
    providerUsage.value = await getProviderUsage() || {}
  } catch (e: any) { providerError.value = e.message }
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
    } catch (e: any) {
      uiStore.showError(e.message || 'Failed to switch configuration')
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
    } catch (e: any) {
      uiStore.showError(e.message || 'Failed to delete configuration')
    }
  } 
}

const handleEnable = async (name: string) => { 
  try {
    await enableConfig(name); 
    uiStore.showSuccess(`Configuration ${name} enabled`)
    refreshData() 
  } catch (e: any) {
    uiStore.showError(e.message || 'Failed to enable configuration')
  }
}

const handleDisable = async (name: string) => { 
  try {
    await disableConfig(name); 
    uiStore.showSuccess(`Configuration ${name} disabled`)
    refreshData() 
  } catch (e: any) {
    uiStore.showError(e.message || 'Failed to disable configuration')
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