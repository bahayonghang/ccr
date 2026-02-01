<template>
  <aside
    class="sticky top-6 h-fit rounded-2xl p-4 transition-all duration-300 glass-sidebar"
  >
    <!-- 🔍 标题和统计 -->
    <div class="flex items-center justify-between mb-4">
      <h2
        class="text-base font-bold flex items-center gap-2 text-primary"
      >
        <Layers class="w-4 h-4 text-emerald-500" />
        快速导航
      </h2>
      <div class="flex items-center gap-1.5 text-xs">
        <span
          class="px-2 py-0.5 rounded-full font-semibold stat-badge-success"
        >
          {{ filteredConfigs.length }}
        </span>
        <span class="text-muted">/</span>
        <span
          class="px-2 py-0.5 rounded-full font-semibold stat-badge-info"
        >
          {{ configs.length }}
        </span>
      </div>
    </div>

    <!-- 🔍 搜索框 -->
    <div class="relative mb-4">
      <Search
        class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted"
      />
      <input
        v-model="searchQuery"
        type="text"
        placeholder="搜索配置..."
        class="w-full pl-9 pr-3 py-2 text-sm rounded-xl transition-all duration-200 outline-none search-input"
      >
    </div>

    <!-- 📊 分类标签快速筛选 -->
    <div
      class="grid grid-cols-2 gap-2 mb-4 pb-4 category-filter-section"
    >
      <button
        v-for="category in categories"
        :key="category.key"
        class="category-btn px-3 py-2 rounded-xl text-xs font-semibold transition-all duration-200 hover:scale-[1.02] cursor-pointer"
        :class="{ 'category-btn-active': expandedCategory === category.key }"
        :style="{
          background: expandedCategory === category.key
            ? category.activeBackground
            : 'var(--glass-bg-light)',
          color: expandedCategory === category.key
            ? category.activeColor
            : 'var(--text-secondary)',
          border: expandedCategory === category.key
            ? `1px solid ${category.activeBorder}`
            : '1px solid var(--glass-border-light)',
          boxShadow: expandedCategory === category.key
            ? `0 4px 12px ${category.activeBorder}40`
            : 'none'
        }"
        @click="toggleCategory(category.key)"
      >
        <div class="flex items-center justify-between gap-1.5">
          <div class="flex items-center gap-1.5">
            <component
              :is="category.icon"
              class="w-3.5 h-3.5"
              :class="expandedCategory === category.key ? '' : category.iconColor"
            />
            <span>{{ category.label }}</span>
          </div>
          <span
            class="px-1.5 py-0.5 rounded-md text-[10px] font-bold"
            :style="{
              background: expandedCategory === category.key
                ? 'rgba(255, 255, 255, 0.25)'
                : 'rgba(0, 0, 0, 0.08)'
            }"
          >
            {{ category.count }}
          </span>
        </div>
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
  </aside>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ChevronDown, Search, Layers, LayoutGrid, Zap, Cpu, HelpCircle } from 'lucide-vue-next'
import type { ConfigItem as ConfigItemType } from '@/types'
import ConfigItem from './ConfigItem.vue'

type FilterType = 'all' | 'official_relay' | 'third_party_model' | 'uncategorized'

interface Props {
  configs: ConfigItemType[]
  currentFilter: FilterType
}

const props = defineProps<Props>()

defineEmits<{
  configClick: [configName: string]
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
/* Glass sidebar effect */
.glass-sidebar {
  background: var(--glass-bg-medium);
  backdrop-filter: blur(16px) saturate(180%);
  border: 1px solid var(--glass-border-medium);
  box-shadow: var(--shadow-lg), inset 0 1px 0 var(--glass-border-light);
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

.text-info {
  color: var(--accent-info);
}

.text-purple {
  color: var(--color-purple);
}

/* Stat badges */
.stat-badge-success {
  background: linear-gradient(135deg, rgb(var(--color-success-rgb), 0.15), rgb(var(--color-cyan-rgb), 0.15));
  color: var(--accent-success);
  border: 1px solid rgb(var(--color-success-rgb), 0.2);
}

.stat-badge-info {
  background: linear-gradient(135deg, rgb(var(--color-cyan-rgb), 0.15), rgb(var(--color-info-rgb), 0.15));
  color: var(--color-cyan);
  border: 1px solid rgb(var(--color-cyan-rgb), 0.2);
}

/* Search input */
.search-input {
  background: var(--glass-bg-light);
  border: 1px solid rgb(var(--color-success-rgb), 0.2);
  color: var(--text-primary);
}

.search-input:focus {
  border-color: rgb(var(--color-success-rgb), 0.5);
}

/* Category filter section */
.category-filter-section {
  border-bottom: 1px solid rgb(var(--color-success-rgb), 0.12);
}

/* 自定义滚动条 */
aside::-webkit-scrollbar {
  width: 4px;
}

aside::-webkit-scrollbar-track {
  background: var(--bg-tertiary);
  border-radius: 2px;
}

aside::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 2px;
}

aside::-webkit-scrollbar-thumb:hover {
  background: var(--text-muted);
}
</style>
