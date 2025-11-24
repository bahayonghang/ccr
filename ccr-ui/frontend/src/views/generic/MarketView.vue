<template>
  <div class="min-h-screen p-6 transition-colors duration-300">
    <div class="max-w-7xl mx-auto">
      <!-- Breadcrumbs & Navigation -->
      <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-8 animate-fade-in">
        <nav class="flex items-center text-sm text-guofeng-text-secondary">
          <RouterLink to="/" class="hover:text-guofeng-red transition-colors flex items-center gap-1">
            <Home class="w-3.5 h-3.5" />
            {{ $t('market.breadcrumb.home') }}
          </RouterLink>
          <ChevronRight class="w-4 h-4 mx-2 text-guofeng-text-muted" />
          <RouterLink to="/claude-code" class="hover:text-guofeng-red transition-colors">
            {{ $t('market.breadcrumb.claude') }}
          </RouterLink>
          <ChevronRight class="w-4 h-4 mx-2 text-guofeng-text-muted" />
          <span class="font-medium text-guofeng-text-primary bg-guofeng-bg-secondary px-2 py-0.5 rounded-md">
            {{ $t('market.breadcrumb.market') }}
          </span>
        </nav>

        <div class="flex items-center gap-3">
          <RouterLink
            to="/claude-code"
            class="group inline-flex items-center gap-2 px-4 py-2 rounded-xl font-medium text-sm transition-all bg-white/50 hover:bg-white border border-guofeng-border hover:border-guofeng-red/30 hover:shadow-sm backdrop-blur-sm"
          >
            <ArrowLeft class="w-4 h-4 group-hover:-translate-x-0.5 transition-transform" />
            <span>{{ $t('market.backToClaude') }}</span>
          </RouterLink>
          <RouterLink
            to="/"
            class="group inline-flex items-center gap-2 px-4 py-2 rounded-xl font-medium text-sm transition-all bg-white/50 hover:bg-white border border-guofeng-border hover:border-guofeng-red/30 hover:shadow-sm backdrop-blur-sm"
          >
            <Home class="w-4 h-4" />
            <span>{{ $t('market.backToHome') }}</span>
          </RouterLink>
        </div>
      </div>

      <!-- Hero Header -->
      <div class="relative mb-10 p-8 rounded-3xl overflow-hidden glass-effect border-0 shadow-lg">
        <div class="absolute top-0 right-0 w-64 h-64 bg-gradient-to-bl from-red-500/5 to-orange-500/5 rounded-full blur-3xl -mr-12 -mt-12 pointer-events-none"></div>
        
        <div class="relative z-10 flex flex-col md:flex-row items-start md:items-center justify-between gap-6">
          <div class="flex items-center gap-5">
            <div class="p-4 bg-gradient-to-br from-guofeng-red to-orange-500 rounded-2xl shadow-lg text-white transform rotate-3 hover:rotate-0 transition-transform duration-300">
              <ShoppingBag class="w-8 h-8" />
            </div>
            <div>
              <h1 class="text-3xl font-bold text-guofeng-text-primary mb-2 flex items-center gap-3">
                {{ $t('market.title') }}
                <span class="px-2.5 py-0.5 text-xs font-bold bg-red-100 text-red-600 rounded-full border border-red-200 tracking-wide uppercase">Beta</span>
              </h1>
              <p class="text-guofeng-text-secondary text-lg max-w-xl">
                {{ $t('market.description') }}
              </p>
            </div>
          </div>
          
          <div class="w-full md:w-auto min-w-[300px]">
            <div class="relative group">
              <Search class="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-guofeng-text-muted group-focus-within:text-guofeng-red transition-colors" />
              <input
                v-model="searchQuery"
                type="text"
                :placeholder="$t('market.searchPlaceholder')"
                class="w-full pl-12 pr-4 py-3.5 rounded-xl bg-white/80 border border-guofeng-border focus:border-guofeng-red focus:ring-4 focus:ring-guofeng-red/10 outline-none transition-all shadow-sm"
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
            ? 'bg-guofeng-red text-white border-guofeng-red shadow-md shadow-guofeng-red/20' 
            : 'bg-white/50 text-guofeng-text-secondary border-transparent hover:bg-white hover:border-guofeng-border'"
          @click="activeTab = tab.id"
        >
          {{ $t(`market.tabs.${tab.id}`) }}
        </button>
      </div>

      <!-- Content Grid -->
      <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
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
                {{ item.icon }}
              </div>
              <div>
                <h3 class="font-bold text-lg text-guofeng-text-primary group-hover:text-guofeng-red transition-colors">{{ item.name }}</h3>
                <div class="flex items-center gap-2 text-xs text-guofeng-text-muted mt-0.5">
                  <span class="px-1.5 py-0.5 rounded bg-guofeng-bg-tertiary border border-guofeng-border/50 font-medium">
                    {{ item.category }}
                  </span>
                  <span>by {{ item.author }}</span>
                </div>
              </div>
            </div>
            <div class="flex items-center gap-1 text-xs font-medium text-guofeng-text-secondary bg-guofeng-bg-tertiary px-2.5 py-1 rounded-full">
              <Download class="w-3.5 h-3.5" /> {{ item.downloads }}
            </div>
          </div>

          <p class="text-sm text-guofeng-text-secondary mb-5 flex-1 line-clamp-3 leading-relaxed">
            {{ item.description }}
          </p>

          <div class="mt-auto pt-4 border-t border-guofeng-border/50 flex items-center justify-between">
            <div class="flex gap-0.5">
              <Star v-for="i in 5" :key="i" class="w-3.5 h-3.5" :class="i <= item.rating ? 'text-yellow-400 fill-yellow-400' : 'text-gray-200'" />
            </div>
            <button 
              class="px-4 py-2 rounded-lg text-xs font-bold transition-all flex items-center gap-1.5"
              :class="item.installed 
                ? 'bg-guofeng-bg-tertiary text-guofeng-text-muted cursor-default' 
                : 'bg-guofeng-red text-white shadow-md shadow-guofeng-red/20 hover:shadow-lg hover:shadow-guofeng-red/30 hover:-translate-y-0.5'"
              @click="!item.installed && handleInstall(item)"
            >
              <component :is="item.installed ? Check : Download" class="w-3.5 h-3.5" />
              {{ item.installed ? $t('market.installed') : $t('market.install') }}
            </button>
          </div>
        </GuofengCard>
      </div>
      
      <!-- Empty State -->
      <div v-if="filteredItems.length === 0" class="text-center py-24">
        <div class="bg-guofeng-bg-secondary w-24 h-24 rounded-full flex items-center justify-center mx-auto mb-6 animate-pulse">
          <Search class="w-10 h-10 opacity-30" />
        </div>
        <h3 class="text-lg font-bold text-guofeng-text-primary mb-2">{{ $t('market.noResults') }}</h3>
        <p class="text-guofeng-text-secondary">Try adjusting your search or filters</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { RouterLink } from 'vue-router'
import { ShoppingBag, Home, Search, Download, Star, Check, ChevronRight, ArrowLeft } from 'lucide-vue-next'
import GuofengCard from '@/components/common/GuofengCard.vue'

const searchQuery = ref('')
const activeTab = ref('featured')

const tabs = [
  { id: 'featured' },
  { id: 'skills' },
  { id: 'plugins' },
  { id: 'mcp' }
]

// Mock Data
const items = [
  {
    id: 1,
    name: 'Git Expert',
    category: 'Skill',
    icon: '🐙',
    author: 'Anthropic',
    description: 'Advanced Git operations helper. Can handle complex merges, rebases, and explain commit history.',
    downloads: '12k',
    rating: 5,
    installed: true
  },
  {
    id: 2,
    name: 'Postgres MCP',
    category: 'MCP',
    icon: '🐘',
    author: 'Community',
    description: 'Connect Claude to your PostgreSQL database to query data, analyze schemas, and optimize queries.',
    downloads: '8.5k',
    rating: 4,
    installed: false
  },
  {
    id: 3,
    name: 'Code Formatter',
    category: 'Plugin',
    icon: '✨',
    author: 'Prettier',
    description: 'Automatically format code in your project using Prettier rules. Supports multiple languages.',
    downloads: '25k',
    rating: 5,
    installed: false
  },
  {
    id: 4,
    name: 'React Component Gen',
    category: 'Skill',
    icon: '⚛️',
    author: 'ReactTeam',
    description: 'Generate production-ready React components with Tailwind CSS styling and TypeScript interfaces.',
    downloads: '15k',
    rating: 5,
    installed: false
  },
  {
    id: 5,
    name: 'Filesystem MCP',
    category: 'MCP',
    icon: '📂',
    author: 'Anthropic',
    description: 'Allow Claude to read and write files on your local system. Essential for coding tasks.',
    downloads: '50k',
    rating: 5,
    installed: true
  },
  {
    id: 6,
    name: 'Jira Integration',
    category: 'Plugin',
    icon: '🎫',
    author: 'Atlassian',
    description: 'Create, update, and query Jira tickets directly from Claude Code. Keep your project management in sync.',
    downloads: '3k',
    rating: 3,
    installed: false
  }
]

const filteredItems = computed(() => {
  let result = items
  
  // Filter by Tab
  if (activeTab.value !== 'featured') {
    const categoryMap: Record<string, string> = {
      'skills': 'Skill',
      'plugins': 'Plugin',
      'mcp': 'MCP'
    }
    result = result.filter(item => item.category === categoryMap[activeTab.value])
  } else {
    // Featured shows high rated items
    result = result.filter(item => item.rating >= 4)
  }

  // Filter by Search
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(item => 
      item.name.toLowerCase().includes(query) || 
      item.description.toLowerCase().includes(query)
    )
  }

  return result
})

const getCategoryColor = (category: string) => {
  switch (category) {
    case 'Skill': return 'bg-blue-100 text-blue-600'
    case 'MCP': return 'bg-green-100 text-green-600'
    case 'Plugin': return 'bg-purple-100 text-purple-600'
    default: return 'bg-gray-100 text-gray-600'
  }
}

const handleInstall = (item: any) => {
  alert(`Install feature for ${item.name} coming soon!`)
}
</script>
