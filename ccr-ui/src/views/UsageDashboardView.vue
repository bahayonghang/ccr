<!-- eslint-disable vue/no-template-shadow -->
<template>
  <div class="min-h-full p-6 lg:p-10 relative overflow-hidden">
    <AnimatedBackground variant="aurora" />

    <div class="max-w-[1600px] mx-auto flex flex-col gap-6 relative z-10">
      <!-- 顶部工具栏 -->
      <div class="flex flex-wrap items-center gap-3 justify-between">
        <div>
          <h1 class="text-2xl font-bold text-white">
            {{ $t('usage.title') }}
          </h1>
          <p class="text-sm text-white/50 mt-1">
            {{ $t('usage.subtitle') }}
          </p>
        </div>
        <div class="flex items-center gap-2">
          <select
            v-model="selectedPlatform"
            class="toolbar-select"
            @change="onFilterChange"
          >
            <option value="">
              {{ $t('usage.dashboard.allPlatforms') }}
            </option>
            <option value="claude">
              Claude
            </option>
            <option value="codex">
              Codex
            </option>
            <option value="gemini">
              Gemini
            </option>
          </select>
          <select
            v-model="selectedDays"
            class="toolbar-select"
            @change="onFilterChange"
          >
            <option :value="7">
              {{ $t('usage.dashboard.days7') }}
            </option>
            <option :value="30">
              {{ $t('usage.dashboard.days30') }}
            </option>
            <option :value="90">
              {{ $t('usage.dashboard.days90') }}
            </option>
            <option :value="365">
              {{ $t('usage.dashboard.days365') }}
            </option>
          </select>
          <button
            class="px-3 py-1.5 rounded-lg text-xs font-medium bg-accent-primary/20 text-accent-primary hover:bg-accent-primary/30 transition-colors"
            :disabled="store.importing"
            @click="doImport"
          >
            {{ importButtonLabel }}
          </button>
          <span
            v-if="store.lastUpdated"
            class="text-[10px] text-white/50"
          >
            {{ store.lastUpdated.toLocaleTimeString() }}
          </span>
        </div>
      </div>

      <!-- 标签页 -->
      <div class="flex gap-1 border-b border-white/10">
        <button
          v-for="t in tabKeys"
          :key="t"
          class="px-4 py-2 text-sm font-medium transition-colors border-b-2 -mb-px"
          :class="activeTab === t ? 'border-accent-primary text-accent-primary' : 'border-transparent text-white/50 hover:text-white'"
          @click="activeTab = t"
        >
          {{ $t(`usage.dashboard.tabs.${t}`) }}
        </button>
      </div>

      <!-- 加载/错误 -->
      <div
        v-if="store.loading"
        class="text-center py-12 text-white/50"
      >
        {{ $t('usage.states.loading') }}
      </div>
      <div
        v-else-if="store.error"
        class="text-center py-12 text-red-400"
      >
        {{ store.error }}
      </div>
      <div
        v-else
        class="space-y-4"
      >
        <div
          v-if="warningMessage"
          class="rounded-2xl border border-amber-400/30 bg-amber-400/10 px-4 py-3 text-sm text-amber-100"
        >
          <div class="font-medium">
            {{ warningMessage }}
          </div>
          <div
            v-for="detail in importDetails"
            :key="detail"
            class="mt-1 text-xs text-amber-100/75"
          >
            {{ detail }}
          </div>
        </div>

        <div
          v-if="showEmptyState"
          class="glass-panel rounded-2xl p-8 text-center"
        >
          <div class="text-lg font-semibold text-white">
            {{ emptyStateTitle }}
          </div>
          <div class="mt-2 text-sm text-white/60">
            {{ emptyStateDescription }}
          </div>
        </div>

        <!-- Overview -->
        <template v-else-if="activeTab === 'overview'">
        <!-- 汇总卡片 -->
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
          <div
            v-for="card in summaryCards"
            :key="card.label"
            class="glass-panel p-5 rounded-xl"
          >
            <div class="text-xs text-white/50 mb-1.5">
              {{ card.label }}
            </div>
            <div class="text-2xl font-bold text-white">
              {{ card.value }}
            </div>
          </div>
        </div>

        <!-- 趋势 + 饼图并排 -->
        <div class="grid lg:grid-cols-3 gap-4">
          <div class="lg:col-span-2 glass-panel p-4 rounded-xl">
            <h3 class="text-sm font-medium text-white/80 mb-3">
              {{ $t('usage.dashboard.chart.trendTitle') }}
            </h3>
            <apexchart
              v-if="shouldLoadCharts && trendSeries[0]?.data?.length"
              type="area"
              height="320"
              :options="trendOptions"
              :series="trendSeries"
            />
            <div
              v-else
              class="flex items-center justify-center h-[320px] text-white/50 text-sm"
            >
              {{ $t('usage.dashboard.chart.noTrend') }}
            </div>
          </div>
          <div class="glass-panel p-4 rounded-xl">
            <h3 class="text-sm font-medium text-white/80 mb-3">
              {{ $t('usage.dashboard.chart.costByModel') }}
            </h3>
            <apexchart
              v-if="shouldLoadCharts && store.modelStats.length"
              type="donut"
              height="320"
              :options="pieOptions"
              :series="pieSeries"
            />
            <div
              v-else
              class="flex items-center justify-center h-[320px] text-white/50 text-sm"
            >
              {{ $t('usage.dashboard.table.noData') }}
            </div>
          </div>
        </div>

        <!-- 模型 + 项目 Top5 并排 -->
        <div class="grid lg:grid-cols-2 gap-4">
          <div class="glass-panel rounded-xl overflow-hidden">
            <div class="px-4 py-3 border-b border-white/10/50 text-sm font-medium text-white/80">
              Top {{ $t('usage.dashboard.tabs.models') }}
            </div>
            <table class="w-full text-sm">
              <thead>
                <tr class="border-b border-white/10 text-white/50 text-left">
                  <th class="p-3">
                    {{ $t('usage.dashboard.table.model') }}
                  </th>
                  <th class="p-3 text-right">
                    {{ $t('usage.dashboard.table.requests') }}
                  </th>
                  <th class="p-3 text-right">
                    {{ $t('usage.dashboard.table.cost') }}
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="m in store.modelStats.slice(0, 5)"
                  :key="m.model"
                  class="border-b border-white/10/50 hover:bg-white/5 transition-colors"
                >
                  <td class="p-3 text-white font-medium">
                    {{ m.model }}
                  </td>
                  <td class="p-3 text-right text-white/80">
                    {{ m.request_count }}
                  </td>
                  <td class="p-3 text-right text-white/80">
                    {{ formatCost(m.total_cost) }}
                  </td>
                </tr>
              </tbody>
            </table>
            <div
              v-if="!store.modelStats.length"
              class="p-6 text-center text-white/50 text-sm"
            >
              {{ $t('usage.dashboard.table.noData') }}
            </div>
          </div>
          <div class="glass-panel rounded-xl overflow-hidden">
            <div class="px-4 py-3 border-b border-white/10/50 text-sm font-medium text-white/80">
              Top {{ $t('usage.dashboard.tabs.projects') }}
            </div>
            <table class="w-full text-sm">
              <thead>
                <tr class="border-b border-white/10 text-white/50 text-left">
                  <th class="p-3">
                    {{ $t('usage.dashboard.table.project') }}
                  </th>
                  <th class="p-3 text-right">
                    {{ $t('usage.dashboard.table.tokens') }}
                  </th>
                  <th class="p-3 text-right">
                    {{ $t('usage.dashboard.table.cost') }}
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="p in store.projectStats.slice(0, 5)"
                  :key="p.project_path"
                  class="border-b border-white/10/50 hover:bg-white/5 transition-colors"
                >
                  <td
                    class="p-3 text-white font-medium truncate max-w-[200px]"
                    :title="p.project_path"
                  >
                    {{ shortenPath(p.project_path) }}
                  </td>
                  <td class="p-3 text-right text-white/80">
                    {{ formatTokens(p.total_tokens) }}
                  </td>
                  <td class="p-3 text-right text-white/80">
                    {{ formatCost(p.total_cost) }}
                  </td>
                </tr>
              </tbody>
            </table>
            <div
              v-if="!store.projectStats.length"
              class="p-6 text-center text-white/50 text-sm"
            >
              {{ $t('usage.dashboard.table.noData') }}
            </div>
          </div>
        </div>
        </template>

        <!-- Models -->
        <template v-else-if="activeTab === 'models'">
        <div class="glass-panel p-4 rounded-xl">
          <h3 class="text-sm font-medium text-white/80 mb-3">
            {{ $t('usage.dashboard.chart.costByModel') }}
          </h3>
          <apexchart
            v-if="shouldLoadCharts && store.modelStats.length"
            type="donut"
            height="280"
            :options="pieOptions"
            :series="pieSeries"
          />
          <div
            v-else
            class="flex items-center justify-center h-[280px] text-white/50 text-sm"
          >
            {{ $t('usage.dashboard.table.noData') }}
          </div>
        </div>
        <div class="glass-panel rounded-xl overflow-hidden">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b border-white/10 text-white/50 text-left">
                <th class="p-3">
                  {{ $t('usage.dashboard.table.model') }}
                </th>
                <th class="p-3 text-right">
                  {{ $t('usage.dashboard.table.requests') }}
                </th>
                <th class="p-3 text-right">
                  {{ $t('usage.dashboard.table.tokens') }}
                </th>
                <th class="p-3 text-right">
                  {{ $t('usage.dashboard.table.cost') }}
                </th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="m in store.modelStats"
                :key="m.model"
                class="border-b border-white/10/50 hover:bg-white/5 transition-colors"
              >
                <td class="p-3 text-white font-medium">
                  {{ m.model }}
                </td>
                <td class="p-3 text-right text-white/80">
                  {{ m.request_count }}
                </td>
                <td class="p-3 text-right text-white/80">
                  {{ formatTokens(m.total_tokens) }}
                </td>
                <td class="p-3 text-right text-white/80">
                  {{ formatCost(m.total_cost) }}
                </td>
              </tr>
            </tbody>
          </table>
          <div
            v-if="!store.modelStats.length"
            class="p-6 text-center text-white/50 text-sm"
          >
            {{ $t('usage.dashboard.table.noData') }}
          </div>
        </div>
        </template>

        <!-- Projects -->
        <template v-else-if="activeTab === 'projects'">
        <div class="glass-panel rounded-xl overflow-hidden">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b border-white/10 text-white/50 text-left">
                <th class="p-3">
                  {{ $t('usage.dashboard.table.project') }}
                </th>
                <th class="p-3 text-right">
                  {{ $t('usage.dashboard.table.requests') }}
                </th>
                <th class="p-3 text-right">
                  {{ $t('usage.dashboard.table.tokens') }}
                </th>
                <th class="p-3 text-right">
                  {{ $t('usage.dashboard.table.cost') }}
                </th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="p in store.projectStats"
                :key="p.project_path"
                class="border-b border-white/10/50 hover:bg-white/5 transition-colors"
              >
                <td
                  class="p-3 text-white font-medium truncate max-w-xs"
                  :title="p.project_path"
                >
                  {{ shortenPath(p.project_path) }}
                </td>
                <td class="p-3 text-right text-white/80">
                  {{ p.request_count }}
                </td>
                <td class="p-3 text-right text-white/80">
                  {{ formatTokens(p.total_tokens) }}
                </td>
                <td class="p-3 text-right text-white/80">
                  {{ formatCost(p.total_cost) }}
                </td>
              </tr>
            </tbody>
          </table>
          <div
            v-if="!store.projectStats.length"
            class="p-6 text-center text-white/50 text-sm"
          >
            {{ $t('usage.dashboard.table.noData') }}
          </div>
        </div>
        </template>

        <!-- Logs -->
        <template v-else-if="activeTab === 'logs'">
        <div class="flex items-center gap-2">
          <input
            v-model="logModelFilter"
            :placeholder="$t('usage.dashboard.logs.filterPlaceholder')"
            class="toolbar-select flex-1 max-w-xs"
            @keyup.enter="loadLogs('reset')"
          >
          <button
            class="px-3 py-1.5 rounded-lg text-xs font-medium bg-accent-primary/20 text-accent-primary hover:bg-accent-primary/30 transition-colors"
            @click="loadLogs('reset')"
          >
            {{ $t('usage.dashboard.logs.search') }}
          </button>
        </div>
        <div class="glass-panel rounded-xl overflow-x-auto">
          <div class="grid grid-cols-[2fr,1fr,2fr,1fr,1fr,1fr] border-b border-white/10 text-white/50 text-left text-sm">
            <div class="p-3">
              {{ $t('usage.dashboard.table.time') }}
            </div>
            <div class="p-3">
              {{ $t('usage.dashboard.table.platform') }}
            </div>
            <div class="p-3">
              {{ $t('usage.dashboard.table.model') }}
            </div>
            <div class="p-3 text-right">
              {{ $t('usage.dashboard.table.input') }}
            </div>
            <div class="p-3 text-right">
              {{ $t('usage.dashboard.table.output') }}
            </div>
            <div class="p-3 text-right">
              {{ $t('usage.dashboard.table.cost') }}
            </div>
          </div>
          <div
            ref="logsScrollRef"
            class="max-h-[420px] overflow-auto"
          >
            <div
              class="relative"
              :style="{ height: `${logsVirtualizer.getTotalSize()}px` }"
            >
              <div
                v-for="virtualRow in logsVirtualizer.getVirtualItems()"
                :key="logsRecords[virtualRow.index]?.id ?? virtualRow.index"
                class="absolute left-0 right-0 grid grid-cols-[2fr,1fr,2fr,1fr,1fr,1fr] border-b border-white/10/50 hover:bg-white/5 transition-colors text-sm"
                :style="{ transform: `translateY(${virtualRow.start}px)` }"
              >
                <div class="p-3 text-white/50 text-xs whitespace-nowrap">
                  {{ new Date(logsRecords[virtualRow.index].recorded_at).toLocaleString() }}
                </div>
                <div class="p-3 text-white/80">
                  {{ logsRecords[virtualRow.index].platform }}
                </div>
                <div class="p-3 text-white font-medium truncate">
                  {{ logsRecords[virtualRow.index].model || '-' }}
                </div>
                <div class="p-3 text-right text-white/80">
                  {{ formatTokens(logsRecords[virtualRow.index].input_tokens) }}
                </div>
                <div class="p-3 text-right text-white/80">
                  {{ formatTokens(logsRecords[virtualRow.index].output_tokens) }}
                </div>
                <div class="p-3 text-right text-white/80">
                  {{ formatCost(logsRecords[virtualRow.index].cost_usd) }}
                </div>
              </div>
            </div>
          </div>
          <div
            v-if="!store.logs?.records?.length"
            class="p-6 text-center text-white/50 text-sm"
          >
            {{ $t('usage.dashboard.logs.noLogs') }}
          </div>
        </div>
        <!-- 分页 -->
        <div
          v-if="store.logs && (store.canPrevLogs || store.canNextLogs)"
          class="flex items-center justify-center gap-2"
        >
          <button
            class="px-3 py-1 rounded text-xs glass-surface text-white/80 hover:text-white disabled:opacity-40 transition-colors"
            :disabled="!store.canPrevLogs"
            @click="loadLogs('prev')"
          >
            {{ $t('usage.dashboard.logs.prev') }}
          </button>
          <span
            v-if="store.logs.total"
            class="text-xs text-white/50"
          >{{ store.logsPage }} / {{ store.logsTotalPages }}</span>
          <span
            v-else
            class="text-xs text-white/50"
          >{{ store.logsPage }}</span>
          <button
            class="px-3 py-1 rounded text-xs glass-surface text-white/80 hover:text-white disabled:opacity-40 transition-colors"
            :disabled="!store.canNextLogs"
            @click="loadLogs('next')"
          >
            {{ $t('usage.dashboard.logs.next') }}
          </button>
        </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, defineAsyncComponent } from 'vue'
import { useI18n } from 'vue-i18n'
import { useVirtualizer } from '@tanstack/vue-virtual'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import { useUsageStore } from '@/stores/usage'
import type { Platform } from '@/types/usage'

const { t } = useI18n()

// 图表仅在需要时按需加载，避免把 apexcharts 打进初始路由 chunk
const apexchart = defineAsyncComponent(async () => {
  const module = await import('vue3-apexcharts')
  return module.default
})

const store = useUsageStore()
const tabKeys = ['overview', 'models', 'projects', 'logs'] as const
const activeTab = ref<string>('overview')
const selectedPlatform = ref('')
const selectedDays = ref(30)
const logModelFilter = ref('')
const logsScrollRef = ref<HTMLElement | null>(null)
const shouldLoadCharts = computed(() => activeTab.value === 'overview' || activeTab.value === 'models')

// 格式化工具
const formatTokens = (n: number) => n >= 1e6 ? `${(n / 1e6).toFixed(1)}M` : n >= 1e3 ? `${(n / 1e3).toFixed(1)}K` : n.toString()
const formatCost = (n: number) => `$${n.toFixed(4)}`
const formatPercent = (n: number) => `${(n * 100).toFixed(1)}%`
const shortenPath = (p: string) => { const parts = p.replace(/\\/g, '/').split('/'); return parts.length > 2 ? `.../${parts.slice(-2).join('/')}` : p }

// 计算时间范围
function getTimeRange(days: number) {
  const end = new Date()
  const start = new Date(end.getTime() - days * 86400000)
  return { start: start.toISOString().slice(0, 10), end: end.toISOString().slice(0, 10) }
}

// 筛选变更
function onFilterChange() {
  const { start, end } = getTimeRange(selectedDays.value)
  store.setFilters({
    platform: (selectedPlatform.value || undefined) as Platform | undefined,
    start, end,
  })
}

// 导入
async function doImport() {
  await store.triggerImport(undefined, 'manual')
}

// 加载日志
function loadLogs(direction: 'reset' | 'next' | 'prev' | 'same' = 'reset') {
  store.logsModelFilter = logModelFilter.value || undefined
  store.fetchLogs(direction)
}

// 切换到 Logs 标签时自动加载
watch(activeTab, (tab) => {
  if (tab === 'logs' && !store.logs) loadLogs('reset')
})

function onVisibilityChange() {
  if (document.hidden) {
    store.stopAutoRefresh()
    return
  }
  store.startAutoRefresh()
  if (activeTab.value === 'logs') {
    loadLogs('same')
  }
}

// 汇总卡片
const summaryCards = computed(() => {
  const s = store.summary
  if (!s) return []
  return [
    { label: t('usage.dashboard.cards.totalRequests'), value: s.total_requests.toLocaleString() },
    { label: t('usage.dashboard.cards.totalTokens'), value: formatTokens(s.total_input_tokens + s.total_output_tokens) },
    { label: t('usage.dashboard.cards.totalCost'), value: formatCost(s.total_cost_usd) },
    { label: t('usage.dashboard.cards.cacheEfficiency'), value: formatPercent(s.cache_efficiency) },
  ]
})

// 主题色
const COLORS = ['#f472b6', '#fb923c', '#3b82f6']

// 趋势图配置
const trendSeries = computed(() => [
  { name: t('usage.dashboard.chart.input'), data: store.trends.map(d => d.input_tokens) },
  { name: t('usage.dashboard.chart.output'), data: store.trends.map(d => d.output_tokens) },
  { name: t('usage.dashboard.chart.cache'), data: store.trends.map(d => d.cache_read_tokens) },
])

const trendOptions = computed(() => ({
  chart: { background: 'transparent', toolbar: { show: false } },
  theme: { mode: 'dark' as const },
  colors: COLORS,
  xaxis: { categories: store.trends.map(d => d.date), labels: { style: { colors: '#94a3b8' } } },
  yaxis: { labels: { style: { colors: '#94a3b8' }, formatter: (v: number) => formatTokens(v) } },
  stroke: { curve: 'smooth' as const, width: 2 },
  fill: { type: 'gradient', gradient: { opacityFrom: 0.4, opacityTo: 0.05 } },
  dataLabels: { enabled: false },
  tooltip: { theme: 'dark' },
  grid: { borderColor: '#334155', strokeDashArray: 4 },
  legend: { labels: { colors: '#94a3b8' } },
}))

// 饼图配置
const pieSeries = computed(() => store.modelStats.map(m => m.total_cost))
const pieOptions = computed(() => ({
  chart: { background: 'transparent' },
  theme: { mode: 'dark' as const },
  colors: COLORS.concat(['#a78bfa', '#34d399', '#fbbf24']),
  labels: store.modelStats.map(m => m.model),
  legend: { position: 'bottom' as const, labels: { colors: '#94a3b8' } },
  dataLabels: { enabled: true, formatter: (_: number, opts: { seriesIndex: number; w: { globals: { series: number[] } } }) => formatCost(opts.w.globals.series[opts.seriesIndex]) },
  tooltip: { theme: 'dark' },
}))

const logsRecords = computed(() => store.logs?.records ?? [])
const logsVirtualizer = useVirtualizer(computed(() => ({
  count: logsRecords.value.length,
  getScrollElement: () => logsScrollRef.value,
  estimateSize: () => 44,
  overscan: 10,
})))

const importButtonLabel = computed(() => {
  if (store.isBootstrapping) return t('usage.dashboard.bootstrapping')
  if (store.importing) return t('usage.dashboard.importing')
  return t('usage.dashboard.import')
})

const importDetails = computed(() =>
  store.lastImportResults
    .filter(result => result.error)
    .map(result => `${result.platform}: ${result.error}`),
)

const warningMessage = computed(() => {
  if (!store.warning) return null
  return store.warning
})

const showEmptyState = computed(() => store.hasNoUsageData)

const emptyStateTitle = computed(() => {
  if (store.lastImportSummary && store.lastImportSummary.processed_files === 0 && store.lastImportSummary.imported_records === 0) {
    return t('usage.dashboard.status.noLogsTitle')
  }
  return t('usage.states.noData')
})

const emptyStateDescription = computed(() => {
  if (store.lastImportSummary && store.lastImportSummary.processed_files === 0 && store.lastImportSummary.imported_records === 0) {
    return t('usage.dashboard.status.noLogs')
  }
  return t('usage.states.noDataHint', { platform: selectedPlatform.value || 'AI' })
})

// 生命周期
onMounted(async () => {
  const { start, end } = getTimeRange(selectedDays.value)
  await store.initializeDashboard({
    platform: (selectedPlatform.value || undefined) as Platform | undefined,
    start,
    end,
  })
  store.startAutoRefresh()
  document.addEventListener('visibilitychange', onVisibilityChange)
})

onUnmounted(() => {
  store.stopAutoRefresh()
  document.removeEventListener('visibilitychange', onVisibilityChange)
})
</script>

<style scoped>
.glass-panel {
  background: rgb(255 255 255 / 5%);
  backdrop-filter: blur(4px);
  border: 1px solid var(--color-border-default, rgb(51 65 85 / 50%));
}

.toolbar-select {
  padding: 0.375rem 0.75rem;
  border-radius: 0.5rem;
  font-size: 0.75rem;
  background: var(--color-bg-elevated, #1e293b);
  border: 1px solid var(--color-border-default, #334155);
  color: var(--color-text-primary, #f8fafc);
}

.toolbar-select:focus {
  outline: none;
  border-color: var(--color-accent-primary, #f472b6);
}
</style>
