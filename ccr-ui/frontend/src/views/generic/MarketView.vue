<template>
  <div class="min-h-screen p-6 transition-colors duration-300">
    <div class="max-w-7xl mx-auto">
      <!-- Breadcrumbs & Navigation -->
      <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-8 animate-fade-in">
        <nav class="flex items-center text-sm text-[var(--color-text-secondary)]">
          <RouterLink
            to="/"
            class="hover:text-[var(--color-danger)] transition-colors flex items-center gap-1"
          >
            <Home class="w-3.5 h-3.5" />
            {{ $t('market.breadcrumb.home') }}
          </RouterLink>
          <ChevronRight class="w-4 h-4 mx-2 text-[var(--color-text-muted)]" />
          <RouterLink
            to="/claude-code"
            class="hover:text-[var(--color-danger)] transition-colors"
          >
            {{ $t('market.breadcrumb.claude') }}
          </RouterLink>
          <ChevronRight class="w-4 h-4 mx-2 text-[var(--color-text-muted)]" />
          <span class="font-medium text-[var(--color-text-primary)] bg-[var(--color-bg-elevated)] px-2 py-0.5 rounded-md">
            {{ $t('market.breadcrumb.market') }}
          </span>
        </nav>

        <div class="flex items-center gap-3">
          <RouterLink
            to="/claude-code"
            class="group inline-flex items-center gap-2 px-4 py-2 rounded-xl font-medium text-sm transition-all bg-white/50 hover:bg-white border border-[var(--color-border-default)] hover:border-[var(--color-danger)]/30 hover:shadow-sm backdrop-blur-sm"
          >
            <ArrowLeft class="w-4 h-4 group-hover:-translate-x-0.5 transition-transform" />
            <span>{{ $t('market.backToClaude') }}</span>
          </RouterLink>
          <RouterLink
            to="/"
            class="group inline-flex items-center gap-2 px-4 py-2 rounded-xl font-medium text-sm transition-all bg-white/50 hover:bg-white border border-[var(--color-border-default)] hover:border-[var(--color-danger)]/30 hover:shadow-sm backdrop-blur-sm"
          >
            <Home class="w-4 h-4" />
            <span>{{ $t('market.backToHome') }}</span>
          </RouterLink>
        </div>
      </div>

      <!-- Hero Header -->
      <div class="relative mb-10 p-8 rounded-3xl overflow-hidden glass-effect border-0 shadow-lg">
        <div class="absolute top-0 right-0 w-64 h-64 bg-gradient-to-bl from-red-500/5 to-orange-500/5 rounded-full blur-3xl -mr-12 -mt-12 pointer-events-none" />
        
        <div class="relative z-10 flex flex-col md:flex-row items-start md:items-center justify-between gap-6">
          <div class="flex items-center gap-5">
            <div class="p-4 bg-gradient-to-br from-[var(--color-danger)] to-orange-500 rounded-2xl shadow-lg text-white transform rotate-3 hover:rotate-0 transition-transform duration-300">
              <ShoppingBag class="w-8 h-8" />
            </div>
            <div>
              <h1 class="text-3xl font-bold text-[var(--color-text-primary)] mb-2 flex items-center gap-3">
                {{ $t('market.title') }}
                <span class="px-2.5 py-0.5 text-xs font-bold bg-red-100 text-red-600 rounded-full border border-red-200 tracking-wide uppercase">Beta</span>
              </h1>
              <p class="text-[var(--color-text-secondary)] text-lg max-w-xl">
                {{ $t('market.description') }}
              </p>
            </div>
          </div>
          
          <div class="w-full md:w-auto min-w-[300px]">
            <div class="relative group">
              <Search class="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-[var(--color-text-muted)] group-focus-within:text-[var(--color-danger)] transition-colors" />
              <input
                v-model="searchQuery"
                type="text"
                :placeholder="$t('market.searchPlaceholder')"
                class="w-full pl-12 pr-4 py-3.5 rounded-xl bg-white/80 border border-[var(--color-border-default)] focus:border-[var(--color-danger)] focus:ring-4 focus:ring-[var(--color-danger)]/10 outline-none transition-all shadow-sm"
              >
            </div>
          </div>
        </div>
      </div>

      <!-- Tabs -->
      <div class="flex gap-2 mb-8 overflow-x-auto pb-2 scrollbar-hide">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="px-5 py-2.5 rounded-xl font-medium whitespace-nowrap transition-all border"
          :class="activeTab === tab.id 
            ? 'bg-[var(--color-danger)] text-white border-[var(--color-danger)] shadow-md shadow-[var(--color-danger)]/20' 
            : 'bg-white/50 text-[var(--color-text-secondary)] border-transparent hover:bg-white hover:border-[var(--color-border-default)]'"
          @click="activeTab = tab.id"
        >
          {{ $t(`market.tabs.${tab.id}`) }}
        </button>
      </div>

      <!-- Loading State -->
      <div
        v-if="loading"
        class="text-center py-24"
      >
        <div class="bg-[var(--color-bg-elevated)] w-24 h-24 rounded-full flex items-center justify-center mx-auto mb-6">
          <Loader2 class="w-10 h-10 opacity-50 animate-spin" />
        </div>
        <h3 class="text-lg font-bold text-[var(--color-text-primary)] mb-2">
          {{ $t('common.loading') }}
        </h3>
      </div>

      <!-- Content Grid -->
      <div
        v-else-if="!loading"
        class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6"
      >
        <GuofengCard
          v-for="item in filteredItems"
          :key="item.id"
          variant="glass"
          interactive
          class="h-full flex flex-col group hover:-translate-y-1 transition-transform duration-300"
        >
          <div class="flex items-start justify-between mb-4">
            <div class="flex items-center gap-4">
              <div 
                class="w-14 h-14 rounded-2xl flex items-center justify-center text-3xl shadow-sm transition-transform group-hover:scale-110 duration-300"
                :class="getCategoryColor(item.category)"
              >
                {{ getCategoryIcon(item) }}
              </div>
              <div>
                <h3 class="font-bold text-lg text-[var(--color-text-primary)] group-hover:text-[var(--color-danger)] transition-colors">
                  {{ item.name }}
                </h3>
                <div class="flex items-center gap-2 text-xs text-[var(--color-text-muted)] mt-0.5">
                  <span class="px-1.5 py-0.5 rounded bg-[var(--color-bg-surface)] border border-[var(--color-border-default)]/50 font-medium">
                    {{ item.category }}
                  </span>
                  <span>by {{ item.author || 'Unknown' }}</span>
                </div>
              </div>
            </div>
            <div class="flex items-center gap-1 text-xs font-medium text-[var(--color-text-secondary)] bg-[var(--color-bg-surface)] px-2.5 py-1 rounded-full">
              <Download class="w-3.5 h-3.5" /> {{ formatDownloads(item.downloads) }}
            </div>
          </div>

          <p class="text-sm text-[var(--color-text-secondary)] mb-5 flex-1 line-clamp-3 leading-relaxed">
            {{ item.description }}
          </p>

          <div class="mt-auto pt-4 border-t border-[var(--color-border-default)]/50 flex items-center justify-between">
            <div class="flex gap-0.5">
              <Star
                v-for="i in 5"
                :key="i"
                class="w-3.5 h-3.5"
                :class="i <= (item.rating || 0) ? 'text-yellow-400 fill-yellow-400' : 'text-gray-200'"
              />
            </div>
            <button 
              class="px-4 py-2 rounded-lg text-xs font-bold transition-all flex items-center gap-1.5"
              :class="item.installed 
                ? 'bg-[var(--color-bg-surface)] text-[var(--color-text-muted)] cursor-default' 
                : isInstalling(item.id)
                  ? 'bg-[var(--color-danger)]/50 text-white cursor-wait'
                  : 'bg-[var(--color-danger)] text-white shadow-md shadow-[var(--color-danger)]/20 hover:shadow-lg hover:shadow-[var(--color-danger)]/30 hover:-translate-y-0.5'"
              :disabled="item.installed || isInstalling(item.id)"
              @click="!item.installed && !isInstalling(item.id) && handleInstall(item)"
            >
              <component
                :is="isInstalling(item.id) ? Loader2 : (item.installed ? Check : Download)"
                class="w-3.5 h-3.5"
                :class="{ 'animate-spin': isInstalling(item.id) }"
              />
              {{ isInstalling(item.id) ? $t('common.installing') : (item.installed ? $t('market.installed') : $t('market.install')) }}
            </button>
          </div>
        </GuofengCard>
      </div>
      
      <!-- Empty State -->
      <div
        v-if="filteredItems.length === 0"
        class="text-center py-24"
      >
        <div class="bg-[var(--color-bg-elevated)] w-24 h-24 rounded-full flex items-center justify-center mx-auto mb-6 animate-pulse">
          <Search class="w-10 h-10 opacity-30" />
        </div>
        <h3 class="text-lg font-bold text-[var(--color-text-primary)] mb-2">
          {{ $t('market.noResults') }}
        </h3>
        <p class="text-[var(--color-text-secondary)]">
          Try adjusting your search or filters
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { RouterLink } from 'vue-router'
import { ShoppingBag, Home, Search, Download, Star, Check, ChevronRight, ArrowLeft, Loader2 } from 'lucide-vue-next'
import GuofengCard from '@/components/common/GuofengCard.vue'
import { useMarketplace, type MarketItem, type MarketItemCategory } from '@/composables/useMarketplace'

const { items, loading, fetchMarketItems, installItem, isInstalling } = useMarketplace()

const searchQuery = ref('')
const activeTab = ref('featured')

const tabs = [
  { id: 'featured' },
  { id: 'skills' },
  { id: 'plugins' },
  { id: 'mcp' }
]

// 加载市场数据
onMounted(() => {
  fetchMarketItems()
})

// 监听 Tab 变化重新加载数据
watch(activeTab, (newTab) => {
  if (newTab === 'featured') {
    fetchMarketItems()
  } else {
    const categoryMap: Record<string, MarketItemCategory> = {
      'skills': 'skill',
      'plugins': 'plugin',
      'mcp': 'mcp'
    }
    fetchMarketItems(categoryMap[newTab])
  }
})

// 计算过滤后的项目
const filteredItems = computed(() => {
  let result = items.value
  
  // Filter by Tab (已在 API 层面处理，但需要处理 featured)
  if (activeTab.value === 'featured') {
    // Featured shows high rated items
    result = result.filter(item => (item.rating || 0) >= 4)
  }

  // Filter by Search
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

// 获取分类对应的颜色
const getCategoryColor = (category: string) => {
  switch (category.toLowerCase()) {
    case 'skill': return 'bg-blue-100 text-blue-600'
    case 'mcp': return 'bg-green-100 text-green-600'
    case 'plugin': return 'bg-purple-100 text-purple-600'
    case 'command': return 'bg-orange-100 text-orange-600'
    default: return 'bg-gray-100 text-gray-600'
  }
}

// 获取分类图标
const getCategoryIcon = (item: MarketItem) => {
  switch (item.category) {
    case 'skill': return '📚'
    case 'mcp': return '🔌'
    case 'plugin': return '🧩'
    case 'command': return '⌨️'
    default: return '📦'
  }
}

// 格式化下载数
const formatDownloads = (downloads?: number) => {
  if (!downloads) return '0'
  if (downloads >= 1000) {
    return `${(downloads / 1000).toFixed(1)}k`
  }
  return downloads.toString()
}

// 处理安装
const handleInstall = async (item: MarketItem) => {
  const success = await installItem(item)
  if (success) {
    // 可以添加成功提示
    console.log(`${item.name} installed successfully`)
  }
}
</script>
