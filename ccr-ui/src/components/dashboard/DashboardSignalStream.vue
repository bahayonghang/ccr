<template>
  <section
    class="dashboard-signals"
    data-dashboard-signals
  >
    <header class="dashboard-signals__header">
      <div class="dashboard-signals__lede">
        <p class="dashboard-signals__eyebrow">
          {{ t('dashboard.signals.eyebrow') }}
        </p>
        <h2 class="dashboard-signals__title">
          {{ t('dashboard.signals.title') }}
        </h2>
      </div>
      <PillToggleGroup
        :options="filterOptions"
        :model-value="filter"
        v-bind="{ ariaLabel: t('dashboard.signals.title') }"
        @update:model-value="filter = $event"
      />
    </header>

    <ol
      v-if="visibleEntries.length > 0"
      class="dashboard-signals__list"
    >
      <li
        v-for="entry in visibleEntries"
        :key="entry.id"
        class="dashboard-signal"
        :data-level="entry.level"
      >
        <span class="dashboard-signal__time">{{ formatTime(entry.timestamp) }}</span>
        <span
          class="dashboard-signal__dot"
          :aria-label="entry.level"
        />
        <span class="dashboard-signal__channel">{{ entry.channel }}</span>
        <span class="dashboard-signal__message-group">
          <p
            class="dashboard-signal__message"
            :title="entry.message"
          >
            {{ entry.message }}
          </p>
          <span
            v-if="entry.count > 1"
            class="dashboard-signal__count"
          >×{{ entry.count }}</span>
        </span>
      </li>
    </ol>

    <div
      v-else
      class="dashboard-signals__empty"
    >
      <span class="dashboard-signals__empty-icon">
        <SIcon
          name="History"
          size="w-5 h-5"
        />
      </span>
      <div>
        <h3>{{ t('dashboard.signals.emptyTitle') }}</h3>
        <p>{{ t('dashboard.signals.emptyDescription') }}</p>
      </div>
      <RouterLink
        to="/monitoring"
        class="dashboard-signals__empty-cta"
      >
        {{ t('dashboard.signals.viewAll') }}
        <SIcon
          name="ArrowRight"
          size="w-4 h-4"
        />
      </RouterLink>
    </div>

    <footer
      v-if="visibleEntries.length > 0"
      class="dashboard-signals__footer"
    >
      <RouterLink to="/monitoring">
        {{ t('dashboard.signals.openMonitoring') }}
        <SIcon
          name="ArrowRight"
          size="w-4 h-4"
        />
      </RouterLink>
    </footer>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import PillToggleGroup from '@/components/ui/PillToggleGroup.vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { MonitoringEntry } from '@/composables/useMonitoringFeed'

const props = withDefaults(defineProps<{
  entries: MonitoringEntry[]
  limit?: number
}>(), {
  limit: 6,
})

const { t } = useI18n()

type FilterId = 'all' | 'warn' | 'error'
type AggregatedEntry = MonitoringEntry & { count: number }

const filter = ref<FilterId>('all')

// 按时间稳定倒序：不再按 severity 优先排序，避免更早的错误盖过更新的普通事件。
const sortedEntries = computed(() => {
  return [...props.entries].sort((left, right) => (
    new Date(right.timestamp).getTime() - new Date(left.timestamp).getTime()
  ))
})

// 相邻且频道/级别/文案完全相同的事件聚合为一条，附带出现次数，避免刷屏。
const aggregatedEntries = computed<AggregatedEntry[]>(() => {
  const result: AggregatedEntry[] = []

  for (const entry of sortedEntries.value) {
    const last = result[result.length - 1]
    if (last && last.message === entry.message && last.channel === entry.channel && last.level === entry.level) {
      last.count += 1
    } else {
      result.push({ ...entry, count: 1 })
    }
  }

  return result
})

const matchesFilter = (entry: MonitoringEntry, id: FilterId) => {
  if (id === 'all') return true
  if (id === 'warn') return entry.level === 'warn' || entry.level === 'error'
  return entry.level === 'error'
}

const filteredEntries = computed(() => aggregatedEntries.value.filter((entry) => matchesFilter(entry, filter.value)))

const visibleEntries = computed(() => filteredEntries.value.slice(0, props.limit))

const filterOptions = computed(() => ([
  { value: 'all' as const, label: `${t('dashboard.signals.filterAll')} ${aggregatedEntries.value.length}` },
  { value: 'warn' as const, label: `${t('dashboard.signals.filterWarn')} ${aggregatedEntries.value.filter((entry) => matchesFilter(entry, 'warn')).length}` },
  { value: 'error' as const, label: `${t('dashboard.signals.filterError')} ${aggregatedEntries.value.filter((entry) => matchesFilter(entry, 'error')).length}` },
]))

const formatTime = (timestamp: string) => {
  const date = new Date(timestamp)
  if (Number.isNaN(date.getTime())) return t('dashboard.signals.unknownTime')

  return new Intl.DateTimeFormat(undefined, {
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}
</script>

<style scoped>
.dashboard-signals {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  height: 100%;
  padding: var(--home-card-pad);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--home-card-radius);
  background: var(--color-bg-surface);
}

.dashboard-signals__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.75rem;
}

.dashboard-signals__lede {
  display: grid;
  gap: 0.18rem;
}

.dashboard-signals__eyebrow {
  margin: 0;
  color: var(--color-text-muted);
  font-size: 0.8125rem;
  font-weight: 500;
  line-height: 1.24;
  letter-spacing: 0;
}

.dashboard-signals__title {
  margin: 0;
  color: var(--color-text-primary);
  font-size: 1.0625rem;
  font-weight: 600;
  line-height: 1.3;
  letter-spacing: 0;
}

.dashboard-signals__list {
  display: grid;
  gap: 0.15rem;
  flex: 1;
  min-height: 0;
  margin: 0;
  padding: 0;
  list-style: none;
}

.dashboard-signal {
  display: grid;
  grid-template-columns: auto auto auto minmax(0, 1fr);
  align-items: start;
  gap: 0.5rem;
  padding: 0.38rem 0.4rem;
  border-radius: 8px;
}

.dashboard-signal__time,
.dashboard-signal__dot,
.dashboard-signal__channel {
  margin-top: 0.15rem;
}

.dashboard-signal:hover {
  background: rgb(var(--color-border-default-rgb) / 6%);
}

.dashboard-signal__time {
  color: var(--color-text-muted);
  font-size: 0.75rem;
  font-variant-numeric: tabular-nums;
  font-weight: 500;
  letter-spacing: 0;
  min-width: 2.8rem;
}

.dashboard-signal__dot {
  width: 0.42rem;
  height: 0.42rem;
  border-radius: 999px;
  background: var(--color-info);
}

.dashboard-signal[data-level='warn'] .dashboard-signal__dot {
  background: var(--color-warning);
}

.dashboard-signal[data-level='error'] .dashboard-signal__dot {
  background: var(--color-danger);
}

.dashboard-signal[data-level='debug'] .dashboard-signal__dot {
  background: var(--color-text-disabled);
}

.dashboard-signal__channel {
  color: var(--color-text-disabled);
  font-size: 0.6875rem;
  font-weight: 500;
  letter-spacing: 0;
}

.dashboard-signal__message-group {
  display: flex;
  align-items: flex-start;
  gap: 0.4rem;
  min-width: 0;
}

.dashboard-signal__message {
  display: -webkit-box;
  overflow: hidden;
  flex: 1;
  min-width: 0;
  margin: 0;
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  line-height: 1.4;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.dashboard-signal__count {
  flex-shrink: 0;
  color: var(--color-text-muted);
  font-size: 0.75rem;
  font-variant-numeric: tabular-nums;
  font-weight: 500;
}

.dashboard-signals__empty {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  align-items: center;
  gap: 0.85rem;
  flex: 1;
  min-height: 8rem;
  padding: 1rem;
  border: 1px dashed var(--color-border-subtle);
  border-radius: 8px;
}

.dashboard-signals__empty-icon {
  display: grid;
  place-items: center;
  width: 2.25rem;
  height: 2.25rem;
  border-radius: 8px;
  background: var(--color-bg-elevated);
  color: var(--color-text-muted);
}

.dashboard-signals__empty h3 {
  margin: 0;
  color: var(--color-text-primary);
  font-size: 0.875rem;
  font-weight: 600;
}

.dashboard-signals__empty p {
  margin: 0.15rem 0 0;
  color: var(--color-text-secondary);
  font-size: 0.75rem;
  line-height: 1.5;
}

.dashboard-signals__empty-cta {
  grid-column: 1 / -1;
  display: inline-flex;
  align-items: center;
  justify-self: start;
  gap: 0.35rem;
  padding: 0.35rem 0.65rem;
  border: 1px solid var(--color-border-subtle);
  border-radius: 8px;
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-size: 0.8125rem;
  font-weight: 500;
  letter-spacing: 0;
  text-decoration: none;
}

.dashboard-signals__empty-cta:hover,
.dashboard-signals__empty-cta:focus-visible {
  color: var(--color-text-primary);
  outline: 0;
}

.dashboard-signals__footer {
  display: flex;
  justify-content: flex-end;
  padding-top: 0.4rem;
  border-top: 1px solid var(--color-border-subtle);
}

.dashboard-signals__footer a {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  color: var(--color-text-muted);
  font-size: 0.8125rem;
  font-weight: 500;
  letter-spacing: 0;
  text-decoration: none;
}

.dashboard-signals__footer a:hover,
.dashboard-signals__footer a:focus-visible {
  color: var(--color-text-primary);
  outline: 0;
}

@media (width <= 720px) {
  .dashboard-signals__header {
    flex-direction: column;
  }
}
</style>
