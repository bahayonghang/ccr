<template>
  <div class="min-h-screen p-6 transition-colors duration-300">
    <div class="max-w-7xl mx-auto">
      <!-- Breadcrumb -->
      <Breadcrumb
        :items="breadcrumbItems"
        module-color="var(--color-accent-primary)"
        class="mb-6 animate-fade-in"
      />

      <!-- Hero Banner -->
      <div class="relative mb-8 p-8 md:p-10 rounded-3xl overflow-hidden glass-effect border border-[var(--color-border-default)] shadow-lg text-center">
        <div class="absolute top-0 left-1/4 w-72 h-72 bg-[var(--color-accent-primary)]/10 rounded-full blur-3xl pointer-events-none" />
        <div class="absolute bottom-0 right-1/4 w-64 h-64 bg-[var(--color-accent-secondary)]/10 rounded-full blur-3xl pointer-events-none" />

        <div class="relative z-10">
          <div class="flex items-center justify-center gap-3 mb-3">
            <div class="p-3 bg-gradient-to-br from-[var(--color-accent-primary)]/20 to-[var(--color-accent-secondary)]/20 rounded-2xl border border-[var(--color-accent-primary)]/30">
              <ShoppingBag class="w-7 h-7 text-[var(--color-accent-primary)]" />
            </div>
            <h1 class="text-3xl font-bold text-[var(--color-text-primary)]">
              {{ $t('market.title') }}
            </h1>
            <span class="px-2.5 py-0.5 rounded-full text-xs font-bold bg-[var(--color-accent-primary)]/15 text-[var(--color-accent-primary)] border border-[var(--color-accent-primary)]/30">
              {{ $t('market.badge') }}
            </span>
          </div>
          <p class="text-[var(--color-text-secondary)] mb-6 max-w-xl mx-auto">
            {{ $t('market.description') }}
          </p>

          <!-- Search -->
          <div class="max-w-lg mx-auto relative group">
            <Search class="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-[var(--color-text-muted)] group-focus-within:text-[var(--color-accent-primary)] transition-colors" />
            <input
              v-model="searchQuery"
              type="text"
              :placeholder="$t('market.searchPlaceholder')"
              class="w-full pl-12 pr-4 py-3.5 rounded-xl bg-[var(--color-bg-elevated)] border border-[var(--color-border-default)] focus:border-[var(--color-accent-primary)] focus:ring-4 focus:ring-[var(--color-accent-primary)]/10 outline-none transition-all text-[var(--color-text-primary)] placeholder-[var(--color-text-muted)]"
            >
          </div>
        </div>
      </div>

      <!-- Stats Row -->
      <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-6">
        <button
          v-for="stat in statCards"
          :key="stat.tab"
          class="glass-effect rounded-xl p-4 border transition-all text-left hover:-translate-y-0.5"
          :class="activeTab === stat.tab
            ? 'border-[var(--color-accent-primary)]/50 shadow-md shadow-[var(--color-accent-primary)]/10'
            : 'border-[var(--color-border-default)] hover:border-[var(--color-accent-primary)]/30'"
          @click="switchTab(stat.tab)"
        >
          <div class="flex items-center gap-2 mb-1">
            <component
              :is="stat.icon"
              class="w-4 h-4 text-[var(--color-accent-primary)]"
            />
            <span class="text-xs font-medium text-[var(--color-text-muted)]">{{ $t(stat.label) }}</span>
          </div>
          <span class="text-2xl font-bold text-[var(--color-text-primary)]">{{ stat.value() }}</span>
        </button>
      </div>

      <!-- Tabs -->
      <div class="flex gap-2 mb-6 overflow-x-auto pb-2 scrollbar-hide">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="px-4 py-2.5 rounded-xl font-medium whitespace-nowrap transition-all border flex items-center gap-2 text-sm"
          :class="activeTab === tab.id
            ? 'bg-[var(--color-accent-primary)]/15 text-[var(--color-accent-primary)] border-[var(--color-accent-primary)]/30'
            : 'bg-[var(--color-bg-surface)] text-[var(--color-text-secondary)] border-transparent hover:border-[var(--color-border-default)]'"
          @click="switchTab(tab.id)"
        >
          <component
            :is="tab.icon"
            class="w-4 h-4"
          />
          {{ $t(`market.tabs.${tab.id}`) }}
          <span
            class="px-1.5 py-0.5 rounded-md text-xs font-bold"
            :class="activeTab === tab.id
              ? 'bg-[var(--color-accent-primary)]/20'
              : 'bg-[var(--color-bg-overlay)]'"
          >
            {{ tab.count() }}
          </span>
        </button>
      </div>

      <!-- Error State -->
      <div
        v-if="error"
        class="mb-6 p-4 rounded-xl bg-[var(--color-danger)]/10 border border-[var(--color-danger)]/20 flex items-center gap-3"
      >
        <AlertCircle class="w-5 h-5 text-[var(--color-danger)] flex-shrink-0" />
        <span class="text-sm text-[var(--color-danger)]">{{ error }}</span>
        <button
          class="ml-auto text-xs text-[var(--color-danger)] hover:underline font-medium"
          @click="retryFetch"
        >
          {{ $t('common.retry') || 'Retry' }}
        </button>
      </div>

      <!-- Loading State -->
      <div
        v-if="loading"
        class="text-center py-24"
      >
        <div class="w-12 h-12 rounded-full border-4 border-[var(--color-accent-primary)]/30 border-t-[var(--color-accent-primary)] animate-spin mx-auto mb-4" />
        <p class="text-[var(--color-text-muted)]">
          {{ $t('common.loading') }}
        </p>
      </div>

      <!-- Content Grid -->
      <div
        v-else-if="filteredItems.length > 0"
        class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-5"
      >
        <GuofengCard
          v-for="item in filteredItems"
          :key="item.id"
          variant="glass"
          interactive
          glow-color="primary"
          class="h-full flex flex-col group"
        >
          <!-- Header -->
          <div class="flex items-start justify-between mb-3">
            <div class="flex items-center gap-3">
              <div
                class="w-12 h-12 rounded-xl flex items-center justify-center text-2xl shadow-sm transition-transform group-hover:scale-110 duration-300"
                :class="getCategoryColor(item.category)"
              >
                {{ getCategoryIcon(item) }}
              </div>
              <div>
                <h3 class="font-bold text-[var(--color-text-primary)] group-hover:text-[var(--color-accent-primary)] transition-colors">
                  {{ item.name }}
                </h3>
                <div class="flex items-center gap-2 text-xs text-[var(--color-text-muted)] mt-0.5">
                  <span class="px-1.5 py-0.5 rounded bg-[var(--color-bg-surface)] border border-[var(--color-border-default)] font-medium">
                    {{ $t(`market.categories.${item.category}`) }}
                  </span>
                  <span v-if="item.author">by {{ item.author }}</span>
                </div>
              </div>
            </div>
            <span
              v-if="item.source"
              class="text-xs font-medium px-2 py-0.5 rounded-full"
              :class="getSourceColor(item.source)"
            >
              {{ $t(`market.sources.${item.source}`) }}
            </span>
          </div>

          <!-- Description -->
          <p class="text-sm text-[var(--color-text-secondary)] mb-3 flex-1 line-clamp-3 leading-relaxed">
            {{ item.description }}
          </p>

          <!-- Tags -->
          <div
            v-if="item.tags?.length"
            class="flex flex-wrap gap-1.5 mb-3"
          >
            <span
              v-for="tag in item.tags.slice(0, 4)"
              :key="tag"
              class="px-2 py-0.5 text-xs rounded-md bg-[var(--color-bg-surface)] text-[var(--color-text-muted)] border border-[var(--color-border-subtle)]"
            >
              {{ tag }}
            </span>
          </div>

          <!-- API Key hint -->
          <div
            v-if="item.requires_api_key"
            class="flex items-center gap-1.5 mb-3 text-xs text-[var(--color-warning)]"
          >
            <Key class="w-3.5 h-3.5" />
            <span>{{ $t('market.requiresApiKey') }} {{ item.api_key_env }}</span>
          </div>

          <!-- Footer -->
          <div class="mt-auto pt-3 border-t border-[var(--color-border-subtle)] flex items-center justify-between">
            <div class="flex gap-0.5">
              <template
                v-for="i in 5"
                :key="i"
              >
                <Star
                  class="w-3.5 h-3.5"
                  :class="i <= (item.rating || 0) ? 'text-[var(--color-warning)] fill-[var(--color-warning)]' : 'text-[var(--color-text-disabled)]'"
                />
              </template>
            </div>
            <div class="flex items-center gap-2">
              <button
                v-if="item.installed && item.source !== 'builtin'"
                class="px-3 py-1.5 rounded-lg text-xs font-bold transition-all flex items-center gap-1.5 bg-[var(--color-danger)]/10 text-[var(--color-danger)] hover:bg-[var(--color-danger)]/20 border border-[var(--color-danger)]/20"
                :disabled="isInstalling(item.id)"
                @click="handleUninstall(item)"
              >
                <Trash2 class="w-3.5 h-3.5" />
                {{ $t('market.uninstall') }}
              </button>
              <button
                class="px-4 py-1.5 rounded-lg text-xs font-bold transition-all flex items-center gap-1.5"
                :class="item.installed
                  ? 'bg-[var(--color-success)]/10 text-[var(--color-success)] border border-[var(--color-success)]/20 cursor-default'
                  : isInstalling(item.id)
                    ? 'bg-[var(--color-accent-primary)]/50 text-white cursor-wait'
                    : 'bg-gradient-to-r from-[var(--color-accent-primary)] to-[var(--color-accent-secondary)] text-white shadow-md shadow-[var(--color-accent-primary)]/20 hover:shadow-lg hover:-translate-y-0.5'"
                :disabled="item.installed || isInstalling(item.id)"
                @click="!item.installed && !isInstalling(item.id) && onInstallClick(item)"
              >
                <component
                  :is="isInstalling(item.id) ? Loader2 : (item.installed ? Check : Download)"
                  class="w-3.5 h-3.5"
                  :class="{ 'animate-spin': isInstalling(item.id) }"
                />
                {{ isInstalling(item.id) ? $t('market.installing') : (item.installed ? $t('market.installed') : $t('market.install')) }}
              </button>
            </div>
          </div>
        </GuofengCard>
      </div>

      <!-- Empty State -->
      <div
        v-else-if="!loading"
        class="text-center py-24"
      >
        <div class="bg-[var(--color-bg-surface)] w-24 h-24 rounded-full flex items-center justify-center mx-auto mb-6">
          <Search class="w-10 h-10 text-[var(--color-text-muted)] opacity-30" />
        </div>
        <h3 class="text-lg font-bold text-[var(--color-text-primary)] mb-2">
          {{ $t('market.noResults') }}
        </h3>
        <p class="text-[var(--color-text-secondary)]">
          {{ $t('market.noResultsHint') }}
        </p>
      </div>
    </div>

    <!-- Install Modal -->
    <Teleport to="body">
      <div
        v-if="showInstallModal"
        class="fixed inset-0 z-50 flex items-center justify-center p-4"
      >
        <div
          class="absolute inset-0 bg-black/40 backdrop-blur-sm"
          @click="showInstallModal = false"
        />
        <div class="relative bg-[var(--color-bg-elevated)] rounded-2xl shadow-2xl w-full max-w-md p-6 animate-fade-in border border-[var(--color-border-default)]">
          <h3 class="text-lg font-bold text-[var(--color-text-primary)] mb-1">
            {{ $t('market.installModal.title') }}
          </h3>
          <p class="text-sm text-[var(--color-text-secondary)] mb-5">
            {{ installTarget?.name }}
          </p>

          <!-- Platform selection -->
          <div class="mb-5">
            <label class="block text-sm font-medium text-[var(--color-text-primary)] mb-2">
              {{ $t('market.installModal.platforms') }}
            </label>
            <div class="flex flex-wrap gap-2">
              <label
                v-for="p in availablePlatforms"
                :key="p.id"
                class="flex items-center gap-2 px-3 py-2 rounded-lg border cursor-pointer transition-all text-sm"
                :class="selectedPlatforms.includes(p.id)
                  ? 'border-[var(--color-accent-primary)] bg-[var(--color-accent-primary)]/10 text-[var(--color-accent-primary)]'
                  : 'border-[var(--color-border-default)] text-[var(--color-text-secondary)] hover:border-[var(--color-accent-primary)]/30'"
              >
                <input
                  v-model="selectedPlatforms"
                  type="checkbox"
                  :value="p.id"
                  class="sr-only"
                >
                <span>{{ p.icon }} {{ p.label }}</span>
              </label>
            </div>
          </div>

          <!-- API Key input -->
          <div
            v-if="installTarget?.requires_api_key"
            class="mb-5"
          >
            <label class="block text-sm font-medium text-[var(--color-text-primary)] mb-2">
              {{ installTarget.api_key_env }}
            </label>
            <input
              v-model="apiKeyInput"
              type="password"
              :placeholder="$t('market.installModal.apiKeyPlaceholder')"
              class="w-full px-4 py-2.5 rounded-lg border border-[var(--color-border-default)] bg-[var(--color-bg-surface)] focus:border-[var(--color-accent-primary)] focus:ring-2 focus:ring-[var(--color-accent-primary)]/10 outline-none text-sm text-[var(--color-text-primary)] placeholder-[var(--color-text-muted)]"
            >
          </div>

          <!-- Actions -->
          <div class="flex justify-end gap-3">
            <button
              class="px-4 py-2 rounded-lg text-sm font-medium text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-surface)] transition-colors"
              @click="showInstallModal = false"
            >
              {{ $t('common.cancel') || 'Cancel' }}
            </button>
            <button
              class="px-5 py-2 rounded-lg text-sm font-bold bg-gradient-to-r from-[var(--color-accent-primary)] to-[var(--color-accent-secondary)] text-white shadow-md hover:shadow-lg transition-all"
              :disabled="selectedPlatforms.length === 0"
              @click="confirmInstall"
            >
              {{ $t('market.install') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  ShoppingBag, Home, Search, Download, Star, Check, Loader2, Key, Trash2, AlertCircle,
  Sparkles, BookOpen, Plug, CheckCircle
} from 'lucide-vue-next'
import GuofengCard from '@/components/common/GuofengCard.vue'
import Breadcrumb from '@/components/ui/Breadcrumb.vue'
import { useMarketplace, type MarketItem, type MarketItemCategory } from '@/composables/useMarketplace'

const { t } = useI18n()
const { items, loading, error, fetchMarketItems, fetchInstalledItems, installItem, uninstallItem, isInstalling } = useMarketplace()

const breadcrumbItems = computed(() => [
  { label: t('market.breadcrumb.home'), path: '/', icon: Home },
  { label: t('market.breadcrumb.claude'), path: '/claude-code' },
  { label: t('market.breadcrumb.market'), path: '', icon: ShoppingBag }
])

const searchQuery = ref('')
const activeTab = ref('featured')

// 缓存全量数据用于统计
const allItems = ref<MarketItem[]>([])

const stats = computed(() => ({
  total: allItems.value.length,
  installed: allItems.value.filter(i => i.installed).length,
  skills: allItems.value.filter(i => i.category === 'skill').length,
  mcp: allItems.value.filter(i => i.category === 'mcp').length,
}))

const statCards = computed(() => [
  { tab: 'featured', icon: ShoppingBag, label: 'market.stats.total', value: () => stats.value.total },
  { tab: 'installed', icon: CheckCircle, label: 'market.stats.installed', value: () => stats.value.installed },
  { tab: 'skills', icon: BookOpen, label: 'market.stats.skills', value: () => stats.value.skills },
  { tab: 'mcp', icon: Plug, label: 'market.stats.mcp', value: () => stats.value.mcp },
])

const tabs = computed(() => [
  { id: 'featured', icon: Sparkles, count: () => stats.value.total },
  { id: 'skills', icon: BookOpen, count: () => stats.value.skills },
  { id: 'mcp', icon: Plug, count: () => stats.value.mcp },
  { id: 'installed', icon: CheckCircle, count: () => stats.value.installed },
])

// Install modal state
const showInstallModal = ref(false)
const installTarget = ref<MarketItem | null>(null)
const selectedPlatforms = ref<string[]>(['claude'])
const apiKeyInput = ref('')

const availablePlatforms = [
  { id: 'claude', label: 'Claude Code', icon: '🤖' },
  { id: 'codex', label: 'Codex', icon: '💻' },
  { id: 'gemini', label: 'Gemini', icon: '✨' },
  { id: 'qwen', label: 'Qwen', icon: '🌐' },
  { id: 'iflow', label: 'iFlow', icon: '🔄' },
]

const categoryMap: Record<string, MarketItemCategory> = {
  'skills': 'skill',
  'mcp': 'mcp',
}

// 加载市场数据
onMounted(async () => {
  await fetchMarketItems()
  allItems.value = [...items.value]
})

const switchTab = (tabId: string) => {
  activeTab.value = tabId
  if (tabId === 'featured') {
    fetchMarketItems()
  } else if (tabId === 'installed') {
    fetchInstalledItems()
  } else if (categoryMap[tabId]) {
    fetchMarketItems(categoryMap[tabId])
  }
}

const retryFetch = () => {
  switchTab(activeTab.value)
}

const filteredItems = computed(() => {
  let result = items.value
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(item =>
      item.name.toLowerCase().includes(query) ||
      item.description.toLowerCase().includes(query) ||
      item.tags?.some(tag => tag.toLowerCase().includes(query))
    )
  }
  return result
})

const getCategoryColor = (category: string) => {
  switch (category.toLowerCase()) {
    case 'skill': return 'bg-[var(--color-info)]/15 text-[var(--color-info)]'
    case 'mcp': return 'bg-[var(--color-success)]/15 text-[var(--color-success)]'
    case 'plugin': return 'bg-[var(--color-accent-secondary)]/15 text-[var(--color-accent-secondary)]'
    case 'command': return 'bg-[var(--color-warning)]/15 text-[var(--color-warning)]'
    default: return 'bg-[var(--color-bg-surface)] text-[var(--color-text-muted)]'
  }
}

const getCategoryIcon = (item: MarketItem) => {
  switch (item.category) {
    case 'skill': return '📚'
    case 'mcp': return '🔌'
    case 'plugin': return '🧩'
    case 'command': return '⌨️'
    default: return '📦'
  }
}

const getSourceColor = (source: string) => {
  switch (source) {
    case 'builtin': return 'bg-[var(--color-info)]/10 text-[var(--color-info)]'
    case 'local': return 'bg-[var(--color-success)]/10 text-[var(--color-success)]'
    default: return 'bg-[var(--color-bg-surface)] text-[var(--color-text-muted)]'
  }
}

const onInstallClick = (item: MarketItem) => {
  if (item.category === 'mcp') {
    installTarget.value = item
    selectedPlatforms.value = ['claude']
    apiKeyInput.value = ''
    showInstallModal.value = true
  } else {
    handleInstall(item)
  }
}

const handleInstall = async (item: MarketItem) => {
  await installItem(item)
}

const confirmInstall = async () => {
  if (!installTarget.value) return
  const env: Record<string, string> = {}
  if (installTarget.value.requires_api_key && installTarget.value.api_key_env && apiKeyInput.value) {
    env[installTarget.value.api_key_env] = apiKeyInput.value
  }
  showInstallModal.value = false
  await installItem(installTarget.value, selectedPlatforms.value, env)
}

const handleUninstall = async (item: MarketItem) => {
  await uninstallItem(item)
}
</script>
