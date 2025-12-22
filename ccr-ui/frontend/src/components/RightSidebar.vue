<template>
  <aside
    class="sticky top-6 h-fit rounded-xl p-4"
    :style="{
      background: 'var(--bg-secondary)',
      border: '1px solid var(--border-color)',
      boxShadow: 'var(--shadow-small)',
      maxHeight: 'calc(100vh - 200px)',
      overflowY: 'auto'
    }"
  >
    <!-- 标题和统计 -->
    <div class="flex items-center justify-between mb-3">
      <h2
        class="text-sm font-bold"
        :style="{ color: 'var(--text-primary)' }"
      >
        快速导航
      </h2>
      <div class="flex items-center gap-2 text-xs">
        <span
          class="px-1.5 py-0.5 rounded font-medium"
          :style="{
            background: 'rgba(99, 102, 241, 0.1)',
            color: '#6366f1'
          }"
        >
          {{ filteredConfigs.length }}
        </span>
        <span :style="{ color: 'var(--text-muted)' }">/</span>
        <span
          class="px-1.5 py-0.5 rounded font-medium"
          :style="{
            background: 'rgba(16, 185, 129, 0.1)',
            color: '#10b981'
          }"
        >
          {{ configs.length }}
        </span>
      </div>
    </div>

    <!-- 分类标签快速跳转 -->
    <div
      class="flex flex-wrap gap-1.5 mb-3 pb-3"
      :style="{ borderBottom: '1px solid var(--border-color)' }"
    >
      <button
        v-for="category in categories"
        :key="category.key"
        class="px-2 py-1 rounded text-[10px] font-medium transition-all duration-150 hover:scale-105"
        :style="{
          background: expandedCategory === category.key 
            ? category.activeBackground 
            : 'var(--bg-tertiary)',
          color: expandedCategory === category.key 
            ? category.activeColor 
            : 'var(--text-muted)',
          border: expandedCategory === category.key 
            ? `1px solid ${category.activeBorder}` 
            : '1px solid var(--border-color)'
        }"
        @click="toggleCategory(category.key)"
      >
        {{ category.label }} ({{ category.count }})
      </button>
    </div>

    <!-- 配置列表 - 按分类分组 -->
    <div class="space-y-2">
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
import { ChevronDown, Search } from 'lucide-vue-next'
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

// 根据当前筛选器过滤配置
const filteredConfigs = computed(() => {
  if (props.currentFilter === 'all') {
    return props.configs
  } else if (props.currentFilter === 'official_relay') {
    return props.configs.filter(
      c => c.provider_type === 'OfficialRelay' || c.provider_type === 'official_relay'
    )
  } else if (props.currentFilter === 'third_party_model') {
    return props.configs.filter(
      c => c.provider_type === 'ThirdPartyModel' || c.provider_type === 'third_party_model'
    )
  } else if (props.currentFilter === 'uncategorized') {
    return props.configs.filter(c => !c.provider_type)
  }
  return props.configs
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

// 分类信息
const categories = computed(() => [
  {
    key: 'all' as FilterType,
    label: '全部',
    count: props.configs.length,
    activeBackground: 'rgba(99, 102, 241, 0.15)',
    activeColor: '#6366f1',
    activeBorder: 'rgba(99, 102, 241, 0.3)'
  },
  {
    key: 'official_relay' as FilterType,
    label: '官方中转',
    count: props.configs.filter(c => c.provider_type === 'OfficialRelay' || c.provider_type === 'official_relay').length,
    activeBackground: 'rgba(59, 130, 246, 0.15)',
    activeColor: '#3b82f6',
    activeBorder: 'rgba(59, 130, 246, 0.3)'
  },
  {
    key: 'third_party_model' as FilterType,
    label: '第三方',
    count: props.configs.filter(c => c.provider_type === 'ThirdPartyModel' || c.provider_type === 'third_party_model').length,
    activeBackground: 'rgba(168, 85, 247, 0.15)',
    activeColor: '#a855f7',
    activeBorder: 'rgba(168, 85, 247, 0.3)'
  },
  {
    key: 'uncategorized' as FilterType,
    label: '未分类',
    count: props.configs.filter(c => !c.provider_type).length,
    activeBackground: 'rgba(107, 114, 128, 0.15)',
    activeColor: '#6b7280',
    activeBorder: 'rgba(107, 114, 128, 0.3)'
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
