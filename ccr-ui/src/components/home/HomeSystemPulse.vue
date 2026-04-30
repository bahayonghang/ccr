<template>
  <section
    class="home-system"
    data-home-hero
  >
    <div class="home-system__intro">
      <p class="home-system__eyebrow">
        {{ t('home.workbenchEyebrow') }}
      </p>
      <h1 class="home-system__title">
        {{ t('home.workbenchTitle') }}
      </h1>
      <p class="home-system__description">
        {{ t('home.workbenchDescription') }}
      </p>
    </div>

    <div class="home-system__metrics">
      <div
        v-for="metric in metrics"
        :key="metric.label"
        class="home-system-metric"
        :data-tone="metric.tone"
      >
        <span class="home-system-metric__label">{{ metric.label }}</span>
        <strong class="home-system-metric__value">{{ metric.value }}</strong>
        <span class="home-system-metric__hint">{{ metric.hint }}</span>
      </div>
    </div>

    <div
      class="home-system__actions"
      data-home-actions
    >
      <RouterLink
        v-for="action in actions"
        :key="action.path"
        :to="action.path"
        class="home-action"
        :class="`home-action--${action.tone}`"
      >
        <span class="home-action__icon">
          <SIcon
            :name="action.icon"
            size="w-4 h-4"
          />
        </span>
        <span class="home-action__copy">
          <strong>{{ action.title }}</strong>
          <span>{{ action.desc }}</span>
        </span>
        <SIcon
          name="ArrowRight"
          size="w-4 h-4"
          class="home-action__arrow"
        />
      </RouterLink>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type { SystemInfo } from '@/types'
import type { HomeUsageOverviewResponse } from '@/types/usage'
import type { HomeQuickAction } from './types'

const props = defineProps<{
  systemInfo: SystemInfo | null
  installedCliCount: number
  runtimeCliCount: number
  overview: HomeUsageOverviewResponse | null
  usageLoading: boolean
  actions: HomeQuickAction[]
}>()

const { t } = useI18n()

const formatPercent = (value?: number) => {
  if (typeof value !== 'number' || Number.isNaN(value)) return '...'
  return `${value.toFixed(1)}%`
}

const formatNumber = (value?: number) => {
  if (typeof value !== 'number') return '...'
  return new Intl.NumberFormat(undefined, { notation: 'compact', maximumFractionDigits: 1 }).format(value)
}

const formatFixed = (value?: number) => {
  if (typeof value !== 'number' || Number.isNaN(value)) return '...'
  return value.toFixed(1)
}

const usageStatus = computed(() => {
  if (props.usageLoading) return t('home.usagePreparing')
  if (!props.overview) return t('home.usageUnavailable')
  if (props.overview.empty_reason) return t('home.usageMissing')
  return formatNumber(props.overview.summary.total_requests)
})

const metrics = computed(() => [
  {
    label: t('home.cpuUsage'),
    value: formatPercent(props.systemInfo?.cpu_usage),
    hint: t('home.systemMetricHost', { host: props.systemInfo?.hostname ?? t('home.systemMetricUnknown') }),
    tone: 'neutral',
  },
  {
    label: t('home.memoryUsage'),
    value: formatPercent(props.systemInfo?.memory_usage_percent),
    hint: props.systemInfo
      ? t('home.systemMetricMemory', {
          used: formatFixed(props.systemInfo.used_memory_gb),
          total: formatFixed(props.systemInfo.total_memory_gb),
        })
      : t('home.systemMetricPending'),
    tone: 'neutral',
  },
  {
    label: t('home.statusCliLabel'),
    value: `${props.installedCliCount}/${props.runtimeCliCount}`,
    hint: t('home.systemMetricCliHint'),
    tone: props.installedCliCount === props.runtimeCliCount ? 'success' : 'warning',
  },
  {
    label: t('home.usageMetricLabel'),
    value: usageStatus.value,
    hint: t('home.systemMetricUsageHint'),
    tone: props.overview?.empty_reason ? 'warning' : 'accent',
  },
])
</script>

<style scoped>
.home-system {
  display: grid;
  gap: 1rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 15%);
  border-radius: 14px;
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 96%), rgb(var(--color-bg-surface-rgb) / 88%));
  box-shadow: 0 18px 40px rgb(73 54 40 / 7%);
  padding: 1rem;
}

.home-system__intro {
  display: grid;
  gap: 0.4rem;
}

.home-system__eyebrow,
.home-system-metric__label {
  color: var(--color-text-muted);
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.home-system__title {
  margin: 0;
  color: var(--color-text-primary);
  font-size: 1.7rem;
  font-weight: 650;
  letter-spacing: 0;
  line-height: 1.15;
}

.home-system__description {
  max-width: 56rem;
  color: var(--color-text-secondary);
  font-size: 0.92rem;
  line-height: 1.7;
}

.home-system__metrics {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  overflow: hidden;
  border: 1px solid rgb(var(--color-border-default-rgb) / 12%);
  border-radius: 10px;
  background: rgb(var(--color-border-default-rgb) / 8%);
  gap: 1px;
}

.home-system-metric {
  display: grid;
  gap: 0.28rem;
  min-width: 0;
  background: rgb(var(--color-bg-elevated-rgb) / 92%);
  padding: 0.8rem;
}

.home-system-metric__value {
  color: var(--color-text-primary);
  font-family: var(--font-mono);
  font-size: 1.15rem;
  font-weight: 700;
}

.home-system-metric__hint {
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 0.74rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.home-system-metric[data-tone='success'] .home-system-metric__value {
  color: var(--accent-success);
}

.home-system-metric[data-tone='warning'] .home-system-metric__value {
  color: var(--accent-warning);
}

.home-system-metric[data-tone='accent'] .home-system-metric__value {
  color: var(--color-accent-primary);
}

.home-system__actions {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0.7rem;
}

.home-action {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.75rem;
  min-height: 4.25rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 13%);
  border-radius: 10px;
  background: rgb(var(--color-bg-surface-rgb) / 72%);
  color: var(--color-text-primary);
  padding: 0.75rem;
  transition:
    border-color 160ms ease,
    background-color 160ms ease,
    transform 160ms ease;
}

.home-action:hover,
.home-action:focus-visible {
  border-color: rgb(var(--color-accent-primary-rgb) / 28%);
  background: rgb(var(--color-bg-elevated-rgb) / 96%);
  transform: translateY(-1px);
}

.home-action__icon {
  display: grid;
  place-items: center;
  width: 2rem;
  height: 2rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 12%);
  border-radius: 8px;
  background: rgb(var(--color-bg-elevated-rgb) / 88%);
  color: var(--color-accent-primary);
}

.home-action--sync .home-action__icon {
  color: var(--color-accent-info);
}

.home-action--usage .home-action__icon {
  color: var(--accent-success);
}

.home-action--config .home-action__icon {
  color: var(--color-accent-secondary);
}

.home-action__copy {
  display: grid;
  gap: 0.18rem;
  min-width: 0;
}

.home-action__copy strong {
  overflow: hidden;
  color: var(--color-text-primary);
  font-size: 0.9rem;
  font-weight: 650;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.home-action__copy span {
  display: -webkit-box;
  overflow: hidden;
  color: var(--color-text-secondary);
  font-size: 0.74rem;
  line-height: 1.45;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.home-action__arrow {
  color: var(--color-text-muted);
}

@media (width <= 1100px) {
  .home-system__metrics,
  .home-system__actions {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (width <= 640px) {
  .home-system__metrics,
  .home-system__actions {
    grid-template-columns: 1fr;
  }
}

@media (prefers-reduced-motion: reduce) {
  .home-action {
    transition: none;
  }
}
</style>
