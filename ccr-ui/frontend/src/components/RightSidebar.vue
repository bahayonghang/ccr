<template>
  <aside
    class="sticky top-6 h-fit rounded-2xl transition-all duration-300 sidebar-container overflow-hidden"
    :class="collapsed ? 'w-12 p-2' : 'p-3'"
  >
    <!-- Collapse Toggle Button (Top Bar Style) -->
    <div 
      v-if="!collapsed"
      class="flex justify-end mb-2"
    >
      <button
        class="p-1.5 rounded-lg text-text-muted hover:text-accent-primary hover:bg-bg-surface/80 transition-all duration-200"
        title="收起侧边栏"
        @click="$emit('toggleCollapse')"
      >
        <PanelLeftClose class="w-4 h-4" />
      </button>
    </div>

    <!-- Collapsed View -->
    <template v-if="collapsed">
      <div class="flex flex-col items-center space-y-3 pt-1">
        <button
          class="p-2 rounded-lg text-text-muted hover:text-accent-primary hover:bg-bg-surface/50 transition-colors"
          title="展开侧边栏"
          @click="$emit('toggleCollapse')"
        >
          <PanelLeftOpen class="w-4 h-4" />
        </button>
        
        <div class="w-8 h-[1px] bg-border-subtle" />

        <button
          class="p-2 rounded-lg hover:bg-bg-surface/50 transition-colors"
          :title="'搜索'"
        >
          <Search class="w-4 h-4 text-text-muted" />
        </button>
        <button
          class="p-2 rounded-lg hover:bg-bg-surface/50 transition-colors"
          :title="'筛选'"
        >
          <Layers class="w-4 h-4 text-emerald-500" />
        </button>
        
        <div class="w-8 h-[1px] bg-border-subtle" />
        
        <!-- Config icons (collapsed) -->
        <div 
          v-for="config in filteredConfigs.slice(0, 8)" 
          :key="config.name"
          class="w-8 h-8 flex items-center justify-center rounded-lg cursor-pointer hover:bg-bg-surface/50 transition-all duration-200 group relative"
          :class="config.is_current ? 'bg-accent-primary/10' : ''"
          @click="$emit('configClick', config.name)"
        >
          <div 
            class="w-5 h-5 rounded text-[10px] font-bold flex items-center justify-center"
            :class="config.is_current ? 'bg-accent-primary text-white' : 'bg-bg-surface text-text-muted group-hover:text-text-primary'"
          >
            {{ config.name[0]?.toUpperCase() }}
          </div>
          
          <!-- Tooltip (Left side) -->
          <div class="absolute left-full ml-2 px-2 py-1 bg-bg-overlay border border-border-subtle rounded text-xs whitespace-nowrap opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity z-50">
            {{ config.name }}
          </div>
        </div>
      </div>
    </template>

    <!-- Expanded View -->
    <template v-else>
      <!-- 🔍 标题和统计 -->
      <div class="flex items-center justify-between mb-4 px-1">
        <h2
          class="text-sm font-bold flex items-center gap-2 text-primary"
        >
          <Layers class="w-4 h-4 text-emerald-500" />
          配置列表
        </h2>
        <div class="flex items-center gap-1.5 text-xs">
          <span class="text-text-secondary font-mono">{{ filteredConfigs.length }}</span>
          <span class="text-border-default">/</span>
          <span class="text-text-muted font-mono">{{ configs.length }}</span>
        </div>
      </div>

      <!-- 🔍 搜索框 -->
      <div class="relative mb-4">
        <Search
          class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-text-muted"
        />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="搜索配置..."
          class="w-full pl-9 pr-3 py-1.5 text-xs rounded-lg bg-bg-surface/50 border border-border-default focus:border-accent-primary/50 focus:bg-bg-surface transition-all duration-200 outline-none placeholder:text-text-disabled"
        >
      </div>

      <!-- 📊 分类标签快速筛选 -->
      <div
        class="grid grid-cols-2 gap-2 mb-4 category-filter-section"
      >
        <button
          v-for="category in categories"
          :key="category.key"
          class="relative flex items-center justify-between px-2.5 py-2 rounded-lg text-xs border transition-all duration-200 hover:scale-[1.01] active:scale-[0.99] cursor-pointer overflow-hidden group"
          :class="expandedCategory === category.key ? 'shadow-md' : 'hover:border-border-default hover:bg-bg-surface/30'"
          :style="{
            background: expandedCategory === category.key ? category.activeBackground : 'transparent',
            borderColor: expandedCategory === category.key ? category.activeBorder : 'var(--color-border-subtle)',
            color: expandedCategory === category.key ? category.activeColor : 'var(--text-secondary)'
          }"
          @click="toggleCategory(category.key)"
        >
          <div class="flex items-center gap-1.5 min-w-0">
            <component 
              :is="category.icon" 
              class="w-3.5 h-3.5 shrink-0"
              :class="expandedCategory === category.key ? '' : category.iconColor"
            />
            <span class="truncate font-medium whitespace-nowrap">{{ category.label }}</span>
          </div>
          <span 
            class="text-[10px] font-bold font-mono ml-1 px-1.5 py-0.5 rounded shrink-0"
            :style="{
              background: expandedCategory === category.key ? 'rgba(255,255,255,0.2)' : 'var(--bg-surface)',
              color: expandedCategory === category.key ? 'currentColor' : 'var(--text-muted)'
            }"
          >{{ category.count }}</span>
        </button>
      </div>

      <!-- 📋 配置列表 - 按分类分组 -->
      <div class="space-y-3">
        <!-- 官方中转分类 -->
        <div v-if="officialRelayConfigs.length > 0 && (expandedCategory === 'all' || expandedCategory === 'official_relay')">
          <div
            class="flex items-center gap-1.5 mb-2 cursor-pointer select-none group"
            @click="toggleCategory('official_relay')"
          >
            <ChevronDown
              class="w-3 h-3 transition-transform text-cyan-400"
              :class="{ 'rotate-[-90deg]': expandedCategory !== 'all' && expandedCategory !== 'official_relay' }"
            />
            <Zap class="w-3.5 h-3.5 text-cyan-400" />
            <span class="text-[10px] font-bold uppercase tracking-wide text-cyan-400 group-hover:text-cyan-300 transition-colors">
              官方中转
            </span>
            <span class="text-[9px] px-1.5 py-0.5 rounded bg-cyan-500/15 text-cyan-400 font-semibold">
              {{ officialRelayConfigs.length }}
            </span>
          </div>
          <div class="space-y-1 ml-1">
            <ConfigItem
              v-for="config in officialRelayConfigs"
              :key="config.name"
              :config="config"
              @click="$emit('configClick', config.name)"
            />
          </div>
        </div>

        <!-- 第三方模型分类 -->
        <div v-if="thirdPartyConfigs.length > 0 && (expandedCategory === 'all' || expandedCategory === 'third_party_model')">
          <div
            class="flex items-center gap-1.5 mb-2 cursor-pointer select-none group"
            @click="toggleCategory('third_party_model')"
          >
            <ChevronDown
              class="w-3 h-3 transition-transform text-violet-400"
              :class="{ 'rotate-[-90deg]': expandedCategory !== 'all' && expandedCategory !== 'third_party_model' }"
            />
            <Cpu class="w-3.5 h-3.5 text-violet-400" />
            <span class="text-[10px] font-bold uppercase tracking-wide text-violet-400 group-hover:text-violet-300 transition-colors">
              第三方模型
            </span>
            <span class="text-[9px] px-1.5 py-0.5 rounded bg-violet-500/15 text-violet-400 font-semibold">
              {{ thirdPartyConfigs.length }}
            </span>
          </div>
          <div class="space-y-1 ml-1">
            <ConfigItem
              v-for="config in thirdPartyConfigs"
              :key="config.name"
              :config="config"
              @click="$emit('configClick', config.name)"
            />
          </div>
        </div>

        <!-- 未分类 -->
        <div v-if="uncategorizedConfigs.length > 0 && (expandedCategory === 'all' || expandedCategory === 'uncategorized')">
          <div
            class="flex items-center gap-1.5 mb-2 cursor-pointer select-none group"
            @click="toggleCategory('uncategorized')"
          >
            <ChevronDown
              class="w-3 h-3 transition-transform text-amber-400"
              :class="{ 'rotate-[-90deg]': expandedCategory !== 'all' && expandedCategory !== 'uncategorized' }"
            />
            <HelpCircle class="w-3.5 h-3.5 text-amber-400" />
            <span class="text-[10px] font-bold uppercase tracking-wide text-amber-400 group-hover:text-amber-300 transition-colors">
              未分类
            </span>
            <span class="text-[9px] px-1.5 py-0.5 rounded bg-amber-500/15 text-amber-400 font-semibold">
              {{ uncategorizedConfigs.length }}
            </span>
          </div>
          <div class="space-y-1 ml-1">
            <ConfigItem
              v-for="config in uncategorizedConfigs"
              :key="config.name"
              :config="config"
              @click="$emit('configClick', config.name)"
            />
          </div>
        </div>
      </div>

      <!-- 空状态 -->
      <div
        v-if="filteredConfigs.length === 0"
        class="text-center py-6"
      >
        <Search
          class="w-5 h-5 mx-auto mb-2 text-muted"
        />
        <p
          class="text-xs text-muted"
        >
          未找到配置
        </p>
      </div>
    </template>
  </aside>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ChevronDown, Search, Layers, LayoutGrid, Zap, Cpu, HelpCircle, PanelLeftOpen, PanelLeftClose } from 'lucide-vue-next'
import type { ConfigItem as ConfigItemType } from '@/types'
import ConfigItem from './ConfigItem.vue'

type FilterType = 'all' | 'official_relay' | 'third_party_model' | 'uncategorized'

interface Props {
  configs: ConfigItemType[]
  currentFilter: FilterType
  collapsed?: boolean
}

const props = defineProps<Props>()

defineEmits<{
  configClick: [configName: string]
  toggleCollapse: []
}>()

// 当前展开的分类
const expandedCategory = ref<FilterType>('all')

// 🔍 搜索查询
const searchQuery = ref('')

// 监听外部筛选器变化
watch(() => props.currentFilter, (newFilter) => {
  expandedCategory.value = newFilter
})

// 切换分类展开状态
const toggleCategory = (category: FilterType) => {
  if (expandedCategory.value === category) {
    expandedCategory.value = 'all'
  } else {
    expandedCategory.value = category
  }
}

// 根据当前筛选器和搜索过滤配置
const filteredConfigs = computed(() => {
  let filtered: ConfigItemType[]
  
  if (props.currentFilter === 'all') {
    filtered = props.configs
  } else if (props.currentFilter === 'official_relay') {
    filtered = props.configs.filter(
      c => c.provider_type === 'OfficialRelay' || c.provider_type === 'official_relay'
    )
  } else if (props.currentFilter === 'third_party_model') {
    filtered = props.configs.filter(
      c => c.provider_type === 'ThirdPartyModel' || c.provider_type === 'third_party_model'
    )
  } else if (props.currentFilter === 'uncategorized') {
    filtered = props.configs.filter(c => !c.provider_type)
  } else {
    filtered = props.configs
  }
  
  // 🔍 应用搜索过滤
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase().trim()
    filtered = filtered.filter(c => 
      c.name.toLowerCase().includes(query) ||
      c.provider?.toLowerCase().includes(query) ||
      c.model?.toLowerCase().includes(query)
    )
  }
  
  return filtered
})

// 按分类分组的配置
const officialRelayConfigs = computed(() => 
  filteredConfigs.value.filter(
    c => c.provider_type === 'OfficialRelay' || c.provider_type === 'official_relay'
  )
)

const thirdPartyConfigs = computed(() => 
  filteredConfigs.value.filter(
    c => c.provider_type === 'ThirdPartyModel' || c.provider_type === 'third_party_model'
  )
)

const uncategorizedConfigs = computed(() => 
  filteredConfigs.value.filter(c => !c.provider_type)
)

// 分类信息 - 使用图标和增强样式
const categories = computed(() => [
  {
    key: 'all' as FilterType,
    label: '全部',
    count: props.configs.length,
    icon: LayoutGrid,
    iconColor: 'text-emerald-400',
    activeBackground: 'linear-gradient(135deg, rgba(var(--color-success-rgb), 0.15), rgba(var(--color-cyan-rgb), 0.15))',
    activeColor: 'var(--color-success)',
    activeBorder: 'rgba(var(--color-success-rgb), 0.3)'
  },
  {
    key: 'official_relay' as FilterType,
    label: '官方中转',
    count: props.configs.filter(c => c.provider_type === 'OfficialRelay' || c.provider_type === 'official_relay').length,
    icon: Zap,
    iconColor: 'text-cyan-400',
    activeBackground: 'linear-gradient(135deg, rgba(var(--color-cyan-rgb), 0.15), rgba(var(--color-info-rgb), 0.15))',
    activeColor: 'var(--color-cyan)',
    activeBorder: 'rgba(var(--color-cyan-rgb), 0.3)'
  },
  {
    key: 'third_party_model' as FilterType,
    label: '第三方',
    count: props.configs.filter(c => c.provider_type === 'ThirdPartyModel' || c.provider_type === 'third_party_model').length,
    icon: Cpu,
    iconColor: 'text-violet-400',
    activeBackground: 'linear-gradient(135deg, rgba(var(--color-teal-rgb), 0.15), rgba(var(--color-success-rgb), 0.15))',
    activeColor: 'var(--color-teal)',
    activeBorder: 'rgba(var(--color-teal-rgb), 0.3)'
  },
  {
    key: 'uncategorized' as FilterType,
    label: '未分类',
    count: props.configs.filter(c => !c.provider_type).length,
    icon: HelpCircle,
    iconColor: 'text-amber-400',
    activeBackground: 'rgba(var(--color-gray-rgb), 0.12)',
    activeColor: 'var(--text-muted)',
    activeBorder: 'rgba(var(--color-gray-rgb), 0.25)'
  }
])
</script>

<style scoped>
/* 侧边栏 - 轻量玻璃容器 */
.sidebar-container {
  background: var(--glass-bg-light);
  backdrop-filter: var(--glass-blur-sm);
  max-height: calc(100vh - 160px);
  overflow-y: auto;
}

/* Text utility classes */
.text-primary {
  color: var(--text-primary);
}

.text-muted {
  color: var(--text-muted);
}

/* Category filter section */
.category-filter-section {
  border-bottom: 1px solid var(--color-border-subtle);
  padding-bottom: 0.75rem;
}

/* 自定义滚动条 */
aside::-webkit-scrollbar {
  width: 3px;
}

aside::-webkit-scrollbar-track {
  background: transparent;
}

aside::-webkit-scrollbar-thumb {
  background: var(--color-border-subtle);
  border-radius: 2px;
}

aside::-webkit-scrollbar-thumb:hover {
  background: var(--color-border-default);
}
</style>
