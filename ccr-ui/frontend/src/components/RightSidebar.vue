<template>
  <aside
    class="sticky top-6 h-fit rounded-2xl p-4 transition-all duration-300"
    :style="{
      background: 'rgba(255, 255, 255, 0.6)',
      backdropFilter: 'blur(16px) saturate(180%)',
      WebkitBackdropFilter: 'blur(16px) saturate(180%)',
      border: '1px solid rgba(255, 255, 255, 0.4)',
      boxShadow: '0 8px 32px rgba(0, 0, 0, 0.08), inset 0 1px 0 rgba(255, 255, 255, 0.5)',
      maxHeight: 'calc(100vh - 160px)',
      overflowY: 'auto'
    }"
  >
    <!-- 🔍 标题和统计 -->
    <div class="flex items-center justify-between mb-4">
      <h2
        class="text-base font-bold flex items-center gap-2"
        :style="{ color: '#0f172a' }"
      >
        <Layers class="w-4 h-4 text-emerald-500" />
        快速导航
      </h2>
      <div class="flex items-center gap-1.5 text-xs">
        <span
          class="px-2 py-0.5 rounded-full font-semibold"
          :style="{
            background: 'linear-gradient(135deg, rgba(16, 185, 129, 0.15), rgba(6, 182, 212, 0.15))',
            color: '#10b981',
            border: '1px solid rgba(16, 185, 129, 0.2)'
          }"
        >
          {{ filteredConfigs.length }}
        </span>
        <span :style="{ color: 'var(--text-muted)' }">/</span>
        <span
          class="px-2 py-0.5 rounded-full font-semibold"
          :style="{
            background: 'linear-gradient(135deg, rgba(6, 182, 212, 0.15), rgba(14, 165, 233, 0.15))',
            color: '#06b6d4',
            border: '1px solid rgba(6, 182, 212, 0.2)'
          }"
        >
          {{ configs.length }}
        </span>
      </div>
    </div>

    <!-- 🔍 搜索框 -->
    <div class="relative mb-4">
      <Search
        class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4"
        :style="{ color: 'var(--text-muted)' }"
      />
      <input
        v-model="searchQuery"
        type="text"
        placeholder="搜索配置..."
        class="w-full pl-9 pr-3 py-2 text-sm rounded-xl transition-all duration-200 outline-none"
        :style="{
          background: 'rgba(255, 255, 255, 0.6)',
          border: '1px solid rgba(16, 185, 129, 0.2)',
          color: '#0f172a'
        }"
        @focus="($event.target as HTMLInputElement).style.borderColor = 'rgba(16, 185, 129, 0.5)'"
        @blur="($event.target as HTMLInputElement).style.borderColor = 'rgba(16, 185, 129, 0.2)'"
      >
    </div>

    <!-- 📊 分类标签快速筛选 -->
    <div
      class="grid grid-cols-2 gap-2 mb-4 pb-4"
      :style="{ borderBottom: '1px solid rgba(16, 185, 129, 0.12)' }"
    >
      <button
        v-for="category in categories"
        :key="category.key"
        class="px-3 py-2 rounded-xl text-xs font-semibold transition-all duration-200 hover:scale-[1.02] cursor-pointer"
        :style="{
          background: expandedCategory === category.key 
            ? category.activeBackground 
            : 'rgba(255, 255, 255, 0.4)',
          color: expandedCategory === category.key 
            ? category.activeColor 
            : 'var(--text-secondary)',
          border: expandedCategory === category.key 
            ? `1px solid ${category.activeBorder}` 
            : '1px solid rgba(255, 255, 255, 0.3)',
          boxShadow: expandedCategory === category.key 
            ? `0 4px 12px ${category.activeBorder}40`
            : 'none'
        }"
        @click="toggleCategory(category.key)"
      >
        <div class="flex items-center justify-between">
          <span>{{ category.label }}</span>
          <span 
            class="ml-1 px-1.5 py-0.5 rounded-md text-[10px] font-bold"
            :style="{
              background: expandedCategory === category.key
                ? 'rgba(255, 255, 255, 0.25)'
                : 'rgba(0, 0, 0, 0.06)'
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
          class="flex items-center gap-1.5 mb-1.5 cursor-pointer select-none"
          @click="toggleCategory('official_relay')"
        >
          <ChevronDown
            class="w-3 h-3 transition-transform"
            :class="{ 'rotate-[-90deg]': expandedCategory !== 'all' && expandedCategory !== 'official_relay' }"
            :style="{ color: '#3b82f6' }"
          />
          <span
            class="text-[10px] font-bold uppercase tracking-wide"
            :style="{ color: '#3b82f6' }"
          >
            官方中转
          </span>
          <span
            class="text-[9px]"
            :style="{ color: 'var(--text-muted)' }"
          >
            ({{ officialRelayConfigs.length }})
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
          class="flex items-center gap-1.5 mb-1.5 cursor-pointer select-none"
          @click="toggleCategory('third_party_model')"
        >
          <ChevronDown
            class="w-3 h-3 transition-transform"
            :class="{ 'rotate-[-90deg]': expandedCategory !== 'all' && expandedCategory !== 'third_party_model' }"
            :style="{ color: '#a855f7' }"
          />
          <span
            class="text-[10px] font-bold uppercase tracking-wide"
            :style="{ color: '#a855f7' }"
          >
            第三方模型
          </span>
          <span
            class="text-[9px]"
            :style="{ color: 'var(--text-muted)' }"
          >
            ({{ thirdPartyConfigs.length }})
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
          class="flex items-center gap-1.5 mb-1.5 cursor-pointer select-none"
          @click="toggleCategory('uncategorized')"
        >
          <ChevronDown
            class="w-3 h-3 transition-transform"
            :class="{ 'rotate-[-90deg]': expandedCategory !== 'all' && expandedCategory !== 'uncategorized' }"
            :style="{ color: 'var(--text-muted)' }"
          />
          <span
            class="text-[10px] font-bold uppercase tracking-wide"
            :style="{ color: 'var(--text-muted)' }"
          >
            未分类
          </span>
          <span
            class="text-[9px]"
            :style="{ color: 'var(--text-muted)' }"
          >
            ({{ uncategorizedConfigs.length }})
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
        class="w-5 h-5 mx-auto mb-2"
        :style="{ color: 'var(--text-muted)' }"
      />
      <p
        class="text-xs"
        :style="{ color: 'var(--text-muted)' }"
      >
        未找到配置
      </p>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ChevronDown, Search, Layers } from 'lucide-vue-next'
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

// 分类信息 - 翡翠绿配色
const categories = computed(() => [
  {
    key: 'all' as FilterType,
    label: '全部',
    count: props.configs.length,
    activeBackground: 'linear-gradient(135deg, rgba(16, 185, 129, 0.15), rgba(6, 182, 212, 0.15))',
    activeColor: '#10b981',
    activeBorder: 'rgba(16, 185, 129, 0.3)'
  },
  {
    key: 'official_relay' as FilterType,
    label: '官方中转',
    count: props.configs.filter(c => c.provider_type === 'OfficialRelay' || c.provider_type === 'official_relay').length,
    activeBackground: 'linear-gradient(135deg, rgba(6, 182, 212, 0.15), rgba(14, 165, 233, 0.15))',
    activeColor: '#06b6d4',
    activeBorder: 'rgba(6, 182, 212, 0.3)'
  },
  {
    key: 'third_party_model' as FilterType,
    label: '第三方',
    count: props.configs.filter(c => c.provider_type === 'ThirdPartyModel' || c.provider_type === 'third_party_model').length,
    activeBackground: 'linear-gradient(135deg, rgba(20, 184, 166, 0.15), rgba(34, 197, 94, 0.15))',
    activeColor: '#14b8a6',
    activeBorder: 'rgba(20, 184, 166, 0.3)'
  },
  {
    key: 'uncategorized' as FilterType,
    label: '未分类',
    count: props.configs.filter(c => !c.provider_type).length,
    activeBackground: 'rgba(107, 114, 128, 0.12)',
    activeColor: '#64748b',
    activeBorder: 'rgba(107, 114, 128, 0.25)'
  }
])
</script>

<style scoped>
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
