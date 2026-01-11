<template>
  <div
    v-if="visible"
    class="fixed inset-0 z-50 flex items-center justify-center"
    :style="{ background: 'rgba(0, 0, 0, 0.4)' }"
    @click.self="$emit('close')"
  >
    <div
      class="w-full max-w-5xl mx-4 rounded-3xl p-6"
      :style="{
        background: 'var(--glass-bg-strong)',
        backdropFilter: 'blur(24px) saturate(180%)',
        WebkitBackdropFilter: 'blur(24px) saturate(180%)',
        border: '1px solid var(--border-color)',
        boxShadow: 'var(--shadow-xl)'
      }"
    >
      <!-- 标题栏 -->
      <div class="flex items-center justify-between mb-4 gap-4">
        <div>
          <h2
            class="text-lg font-bold flex items-center gap-2"
            :style="{ color: 'var(--text-primary)' }"
          >
            <span>🏢 {{ $t('configs.provider.stats') }}</span>
          </h2>
          <p
            class="mt-1 text-xs"
            :style="{ color: 'var(--text-muted)' }"
          >
            {{ $t('configs.provider.totalProviders', { count: providerEntries.length }) }} ·
            {{ $t('configs.provider.totalCalls', { count: totalUsage }) }}
          </p>
        </div>
        <div class="flex items-center gap-3">
          <!-- 排序选择 -->
          <div
            class="flex items-center gap-1 text-xs"
            :style="{ color: 'var(--text-muted)' }"
          >
            <span>{{ $t('configs.provider.sortLabel') }}</span>
            <select
              :value="sortMode"
              class="px-3 py-1.5 rounded-xl text-xs font-medium outline-none cursor-pointer"
              :style="{
                border: '1px solid var(--border-color)',
                background: 'var(--bg-secondary)',
                color: 'var(--text-primary)'
              }"
              @change="$emit('update:sortMode', ($event.target as HTMLSelectElement).value as SortMode)"
            >
              <option value="count_desc">
                {{ $t('configs.provider.sortCountDesc') }}
              </option>
              <option value="count_asc">
                {{ $t('configs.provider.sortCountAsc') }}
              </option>
              <option value="name_asc">
                {{ $t('configs.provider.sortNameAsc') }}
              </option>
            </select>
          </div>
          <!-- 刷新按钮 -->
          <button
            class="px-3 py-1.5 rounded-xl text-xs font-semibold flex items-center gap-1 transition"
            :style="{
              border: '1px solid var(--border-color)',
              background: 'var(--bg-secondary)',
              color: 'var(--text-primary)'
            }"
            :disabled="loading"
            @click="$emit('refresh')"
          >
            <RefreshCw
              class="w-3.5 h-3.5"
              :class="{ 'animate-spin': loading }"
            />
            <span>{{ $t('configs.provider.refreshStats') }}</span>
          </button>
          <!-- 关闭按钮 -->
          <button
            class="w-8 h-8 rounded-full flex items-center justify-center transition"
            :style="{ color: 'var(--text-muted)' }"
            @click="$emit('close')"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- 加载状态 -->
      <div
        v-if="loading"
        class="flex items-center justify-center py-10"
      >
        <div
          class="w-10 h-10 rounded-full border-4 border-transparent animate-spin"
          :style="{
            borderTopColor: 'var(--accent-primary)',
            borderRightColor: 'var(--accent-secondary)'
          }"
        />
      </div>

      <!-- 错误状态 -->
      <div
        v-else-if="error"
        class="text-xs rounded-lg px-3 py-2"
        :style="{
          border: '1px solid var(--accent-danger)',
          background: 'color-mix(in srgb, var(--accent-danger) 10%, transparent)',
          color: 'var(--accent-danger)'
        }"
      >
        {{ $t('configs.provider.loadFailed') }}: {{ error }}
      </div>

      <!-- 空状态 -->
      <div
        v-else-if="sortedEntries.length === 0"
        class="text-center text-xs py-8"
        :style="{ color: 'var(--text-muted)' }"
      >
        {{ $t('configs.provider.noData') }}
      </div>

      <!-- 图表内容 -->
      <div
        v-else
        class="flex flex-col h-[500px]"
      >
        <!-- 统计摘要 -->
        <div
          class="flex items-center gap-4 mb-6 p-4 rounded-2xl"
          :style="{
            background: 'color-mix(in srgb, var(--accent-primary) 10%, transparent)',
            border: '1px solid color-mix(in srgb, var(--accent-primary) 20%, transparent)'
          }"
        >
          <div
            class="p-3 rounded-xl"
            :style="{
              background: 'color-mix(in srgb, var(--accent-primary) 20%, transparent)',
              color: 'var(--accent-primary)'
            }"
          >
            <BarChart3 class="w-6 h-6" />
          </div>
          <div>
            <div
              class="text-sm font-medium"
              :style="{ color: 'var(--text-secondary)' }"
            >
              {{ $t('configs.provider.totalProviders', { count: providerEntries.length }) }}
            </div>
            <div
              class="text-2xl font-bold"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ $t('configs.provider.totalCalls', { count: totalUsage }) }}
            </div>
          </div>
          <div
            class="ml-auto text-right text-xs"
            :style="{ color: 'var(--text-muted)' }"
          >
            <div>{{ $t('configs.provider.currentSort', { label: sortLabel }) }}</div>
          </div>
        </div>

        <!-- 柱状图容器 -->
        <div class="flex-1 relative min-h-0 flex flex-col">
          <!-- Y轴网格线 -->
          <div class="absolute inset-0 flex flex-col justify-between pointer-events-none z-0 pb-8 pl-8">
            <div
              v-for="tick in yTicks.slice().reverse()"
              :key="tick.percent"
              class="relative w-full h-px"
              :style="{ background: 'var(--border-color)', opacity: 0.5 }"
            >
              <span
                class="absolute -left-8 -top-2 text-[10px] w-6 text-right"
                :style="{ color: 'var(--text-muted)' }"
              >
                {{ tick.value }}
              </span>
            </div>
          </div>

          <!-- 柱状图滚动区域 -->
          <div class="flex-1 overflow-x-auto overflow-y-hidden custom-scrollbar z-10 pl-8 pb-2">
            <div class="h-full flex items-end gap-4 min-w-max px-4 pt-4">
              <div
                v-for="([provider, count], index) in sortedEntries"
                :key="provider || index"
                class="group flex flex-col items-center gap-2 w-16"
                :style="{ animation: 'slideUp 0.4s ease ' + index * 0.05 + 's backwards' }"
              >
                <!-- 数值提示 (Hover显示) -->
                <div
                  class="opacity-0 group-hover:opacity-100 transition-opacity absolute -top-8 text-xs px-2 py-1 rounded shadow-lg pointer-events-none whitespace-nowrap z-20"
                  :style="{
                    background: 'var(--bg-tertiary)',
                    color: 'var(--text-primary)'
                  }"
                >
                  {{ count }}次 ({{ maxCount ? ((count / totalUsage) * 100).toFixed(1) : 0 }}%)
                </div>

                <!-- 柱子 -->
                <div class="relative w-full flex items-end justify-center h-[300px]">
                  <div
                    class="w-full rounded-t-lg transition-all duration-300 group-hover:brightness-110 relative overflow-hidden"
                    :style="{
                      height: Math.max((count / (maxCount || 1)) * 100, 4) + '%',
                      background: chartColors[index % chartColors.length],
                      boxShadow: `0 4px 12px ${chartColors[index % chartColors.length]}40`
                    }"
                  >
                    <!-- 玻璃光泽效果 -->
                    <div
                      class="absolute inset-0 opacity-50"
                      :style="{ background: 'linear-gradient(to bottom, rgba(255,255,255,0.3), transparent)' }"
                    />
                  </div>
                </div>

                <!-- 标签 -->
                <div class="text-center w-full">
                  <div
                    class="text-xs font-medium truncate w-full cursor-help transition-colors"
                    :style="{ color: 'var(--text-secondary)' }"
                    :title="provider || $t('configs.provider.unknown')"
                  >
                    {{ provider || $t('configs.provider.unknown') }}
                  </div>
                  <div
                    class="text-[10px] font-mono mt-0.5"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ count }}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { RefreshCw, X, BarChart3 } from 'lucide-vue-next'

type SortMode = 'count_desc' | 'count_asc' | 'name_asc'

interface Props {
  visible: boolean
  providerUsage: Record<string, number>
  loading: boolean
  error: string | null
  sortMode: SortMode
}

const props = defineProps<Props>()

interface Emits {
  (e: 'close'): void
  (e: 'refresh'): void
  (e: 'update:sortMode', value: SortMode): void
}

defineEmits<Emits>()

const { t } = useI18n()

// 图表颜色配置
const chartColors = [
  'var(--accent-success)',
  'var(--platform-gemini)',
  'var(--platform-codex)',
  'var(--accent-info)',
  'var(--accent-danger)'
]

// 计算属性
const providerEntries = computed(() => {
  return Object.entries(props.providerUsage || {})
})

const sortedEntries = computed(() => {
  const entries = [...providerEntries.value]
  if (props.sortMode === 'count_asc') {
    entries.sort((a, b) => a[1] - b[1])
  } else if (props.sortMode === 'name_asc') {
    entries.sort((a, b) => a[0].localeCompare(b[0]))
  } else {
    entries.sort((a, b) => b[1] - a[1])
  }
  return entries
})

const sortLabel = computed(() => {
  if (props.sortMode === 'count_asc') {
    return t('configs.provider.sortModes.countAsc')
  }
  if (props.sortMode === 'name_asc') {
    return t('configs.provider.sortModes.nameAsc')
  }
  return t('configs.provider.sortModes.countDesc')
})

const maxCount = computed(() => {
  const values = providerEntries.value.map(([, count]) => count)
  return values.length ? Math.max(...values) : 0
})

const totalUsage = computed(() => {
  return providerEntries.value.reduce((sum, [, count]) => sum + count, 0)
})

const yTicks = computed(() => {
  const max = maxCount.value || 0
  const percents = [0, 25, 50, 75, 100]
  if (max === 0) {
    return percents.map(percent => ({ percent, value: 0 }))
  }
  return percents.map(percent => ({
    percent,
    value: Math.round((max * percent) / 100),
  }))
})
</script>

<style scoped>
@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
