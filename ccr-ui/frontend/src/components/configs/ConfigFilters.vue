<template>
  <div class="flex gap-4 mb-6 items-center">
    <!-- 筛选按钮组 - Enhanced Glass Style -->
    <div class="flex gap-2 flex-1 p-1.5 rounded-2xl glass-filter-container">
      <button
        v-for="filter in filters"
        :key="filter.type"
        class="filter-btn flex-1 py-2.5 px-4 rounded-xl text-sm font-semibold transition-all duration-300 flex items-center justify-center gap-2"
        :class="{ 'filter-btn-active': currentFilter === filter.type }"
        @click="$emit('update:currentFilter', filter.type)"
      >
        <component
          :is="filter.icon"
          class="w-4 h-4"
          :class="currentFilter === filter.type ? 'text-white' : filter.iconColor"
        />
        <span>{{ filter.label }}</span>
      </button>
    </div>

    <!-- 排序和操作区 -->
    <div class="flex items-center gap-3">
      <!-- 排序下拉菜单 -->
      <div class="flex items-center gap-2">
        <label class="text-sm font-medium whitespace-nowrap text-text-secondary">
          {{ $t('configs.sort.label') }}
        </label>
        <div class="relative">
          <select
            :value="currentSort"
            class="sort-select appearance-none pl-9 pr-8 py-2.5 rounded-xl text-sm font-semibold transition-all cursor-pointer outline-none"
            @change="$emit('update:currentSort', ($event.target as HTMLSelectElement).value as SortType)"
          >
            <option value="name">
              {{ $t('configs.sort.name') }}
            </option>
            <option value="usage_count">
              {{ $t('configs.sort.usageCount') }}
            </option>
            <option value="recent">
              {{ $t('configs.sort.recent') }}
            </option>
          </select>
          <component
            :is="sortIcons[currentSort]"
            class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-accent-primary pointer-events-none"
          />
          <ChevronDown class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-text-muted pointer-events-none" />
        </div>
      </div>

      <!-- 提供商统计按钮 -->
      <button
        class="stats-btn px-3 py-2.5 rounded-xl text-xs font-semibold flex items-center gap-1.5 transition-all duration-200"
        @click="$emit('showProviderStats')"
      >
        <BarChart3 class="w-4 h-4" />
        <span>{{ $t('configs.provider.title') }}</span>
      </button>

      <!-- 添加配置按钮 -->
      <button
        class="add-btn px-4 py-2.5 rounded-xl text-sm font-bold flex items-center gap-2 text-white transition-all duration-200 hover:scale-105"
        @click="$emit('addConfig')"
      >
        <PlusCircle class="w-4 h-4" />
        <span>{{ $t('configs.buttons.add') }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import {
  BarChart3,
  PlusCircle,
  ChevronDown,
  LayoutGrid,
  Zap,
  Cpu,
  HelpCircle,
  FileText,
  TrendingUp,
  Clock
} from 'lucide-vue-next'
import { computed } from 'vue'

type FilterType = 'all' | 'official_relay' | 'third_party_model' | 'uncategorized'
type SortType = 'name' | 'usage_count' | 'recent'

interface Props {
  currentFilter: FilterType
  currentSort: SortType
}

defineProps<Props>()

interface Emits {
  (e: 'update:currentFilter', value: FilterType): void
  (e: 'update:currentSort', value: SortType): void
  (e: 'showProviderStats'): void
  (e: 'addConfig'): void
}

defineEmits<Emits>()

const { t } = useI18n()

// Sort icons mapping
const sortIcons = {
  name: FileText,
  usage_count: TrendingUp,
  recent: Clock
}

// 筛选选项 - 使用 SVG 图标替代 emoji
const filters = computed(() => [
  {
    type: 'all' as FilterType,
    label: t('configs.filters.all'),
    icon: LayoutGrid,
    iconColor: 'text-emerald-400'
  },
  {
    type: 'official_relay' as FilterType,
    label: t('configs.filters.officialRelay'),
    icon: Zap,
    iconColor: 'text-cyan-400'
  },
  {
    type: 'third_party_model' as FilterType,
    label: t('configs.filters.thirdPartyModel'),
    icon: Cpu,
    iconColor: 'text-violet-400'
  },
  {
    type: 'uncategorized' as FilterType,
    label: t('configs.filters.uncategorized'),
    icon: HelpCircle,
    iconColor: 'text-amber-400'
  },
])
</script>

<style scoped>
/* Glass filter container */
.glass-filter-container {
  background: var(--glass-bg-light);
  backdrop-filter: blur(12px);
  border: 1px solid var(--glass-border-light);
  box-shadow: var(--shadow-sm);
}

/* Filter button base */
.filter-btn {
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid transparent;
}

.filter-btn:hover:not(.filter-btn-active) {
  background: var(--glass-bg-medium);
  color: var(--text-primary);
}

/* Filter button active state */
.filter-btn-active {
  background: linear-gradient(135deg, var(--color-accent-primary), var(--color-accent-secondary));
  color: white;
  border-color: transparent;
  box-shadow:
    0 4px 16px rgb(var(--color-accent-primary-rgb) / 30%),
    inset 0 1px 0 rgb(255 255 255 / 20%);
}

/* Sort select */
.sort-select {
  background: var(--glass-bg-light);
  backdrop-filter: blur(10px);
  border: 1px solid var(--glass-border-light);
  color: var(--text-primary);
  min-width: 140px;
}

.sort-select:hover {
  border-color: var(--color-accent-primary);
}

.sort-select:focus {
  border-color: var(--color-accent-primary);
  box-shadow: 0 0 0 3px rgb(var(--color-accent-primary-rgb) / 15%);
}

/* Stats button */
.stats-btn {
  background: var(--glass-bg-light);
  border: 1px solid var(--glass-border-light);
  color: var(--text-secondary);
}

.stats-btn:hover {
  background: var(--glass-bg-medium);
  border-color: var(--color-accent-primary);
  color: var(--color-accent-primary);
}

/* Add button */
.add-btn {
  background: linear-gradient(135deg, var(--color-accent-primary), var(--color-accent-secondary));
  box-shadow: 0 4px 16px rgb(var(--color-accent-primary-rgb) / 30%);
}

.add-btn:hover {
  box-shadow: 0 6px 24px rgb(var(--color-accent-primary-rgb) / 40%);
}
</style>
