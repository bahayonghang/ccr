<template>
  <div class="flex gap-4 mb-6 items-center">
    <!-- 筛选按钮组 -->
    <div
      class="flex gap-3 flex-1 p-2 rounded-2xl"
      :style="{
        background: 'var(--glass-bg)',
        backdropFilter: 'blur(10px)',
        border: '1px solid var(--border-color)'
      }"
    >
      <button
        v-for="filter in filters"
        :key="filter.type"
        class="flex-1 py-3 px-5 rounded-xl text-sm font-bold transition-all duration-300 hover:scale-[1.02]"
        :style="filterButtonStyle(filter.type)"
        @click="$emit('update:currentFilter', filter.type)"
      >
        {{ filter.label }}
      </button>
    </div>

    <!-- 排序和操作区 -->
    <div class="flex items-center gap-3">
      <!-- 排序下拉菜单 -->
      <div class="flex items-center gap-2">
        <label
          class="text-sm font-medium whitespace-nowrap"
          :style="{ color: 'var(--text-secondary)' }"
        >
          {{ $t('configs.sort.label') }}
        </label>
        <select
          :value="currentSort"
          class="px-4 py-3 rounded-xl text-sm font-semibold transition-all cursor-pointer outline-none"
          :style="{
            background: 'var(--bg-secondary)',
            backdropFilter: 'blur(10px)',
            border: '1px solid var(--border-color)',
            color: 'var(--text-primary)'
          }"
          @change="$emit('update:currentSort', ($event.target as HTMLSelectElement).value as SortType)"
        >
          <option value="name">
            📝 {{ $t('configs.sort.name') }}
          </option>
          <option value="usage_count">
            📊 {{ $t('configs.sort.usageCount') }}
          </option>
          <option value="recent">
            🕒 {{ $t('configs.sort.recent') }}
          </option>
        </select>
      </div>

      <!-- 提供商统计按钮 -->
      <button
        class="px-3 py-2 rounded-xl text-xs font-semibold flex items-center gap-1 transition"
        :style="{
          border: '1px solid var(--border-color)',
          background: 'var(--bg-secondary)',
          color: 'var(--text-primary)'
        }"
        @click="$emit('showProviderStats')"
      >
        <BarChart3 class="w-4 h-4" />
        <span>{{ $t('configs.provider.title') }}</span>
      </button>

      <!-- 添加配置按钮 -->
      <button
        class="px-4 py-2 rounded-xl text-sm font-semibold flex items-center gap-2 text-white transition-all hover:scale-105"
        :style="{
          background: 'var(--gradient-primary)',
          boxShadow: '0 4px 12px color-mix(in srgb, var(--accent-primary) 30%, transparent)'
        }"
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
import { BarChart3, PlusCircle } from 'lucide-vue-next'

type FilterType = 'all' | 'official_relay' | 'third_party_model' | 'uncategorized'
type SortType = 'name' | 'usage_count' | 'recent'

interface Props {
  currentFilter: FilterType
  currentSort: SortType
}

const props = defineProps<Props>()

interface Emits {
  (e: 'update:currentFilter', value: FilterType): void
  (e: 'update:currentSort', value: SortType): void
  (e: 'showProviderStats'): void
  (e: 'addConfig'): void
}

defineEmits<Emits>()

const { t } = useI18n()

// 筛选选项
const filters = [
  { type: 'all' as FilterType, label: `📋 ${t('configs.filters.all')}` },
  { type: 'official_relay' as FilterType, label: `🔄 ${t('configs.filters.officialRelay')}` },
  { type: 'third_party_model' as FilterType, label: `🤖 ${t('configs.filters.thirdPartyModel')}` },
  { type: 'uncategorized' as FilterType, label: `❓ ${t('configs.filters.uncategorized')}` },
]

// 筛选按钮样式
const filterButtonStyle = (filterType: FilterType) => {
  const isActive = props.currentFilter === filterType
  return {
    background: isActive
      ? 'var(--gradient-primary)'
      : 'var(--glass-bg)',
    border: isActive
      ? '1px solid color-mix(in srgb, var(--accent-primary) 30%, transparent)'
      : '1px solid transparent',
    color: isActive ? 'white' : 'var(--text-secondary)',
    boxShadow: isActive
      ? '0 4px 16px color-mix(in srgb, var(--accent-primary) 25%, transparent)'
      : '0 2px 8px rgba(0, 0, 0, 0.03)'
  }
}
</script>
