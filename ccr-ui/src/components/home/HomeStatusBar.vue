<template>
  <section
    class="home-status"
    data-home-status
    :aria-label="t('home.workbenchEyebrow')"
  >
    <div
      v-for="chip in chips"
      :key="chip.id"
      class="home-status-chip"
      :data-tone="chip.tone"
    >
      <span class="home-status-chip__label">{{ chip.label }}</span>
      <span class="home-status-chip__value">{{ chip.value }}</span>
      <span
        v-if="chip.tone !== 'neutral'"
        class="home-status-chip__dot"
        aria-hidden="true"
      />
      <span
        v-if="chip.hint"
        class="home-status-chip__hint"
      >{{ chip.hint }}</span>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { hasTemplatePlaceholder, translateWithFallback } from '@/i18n/formatMessage'
import type { SystemInfo } from '@/types'
import type { HomeUsageOverviewResponse } from '@/types/usage'

const props = defineProps<{
  systemInfo: SystemInfo | null
  installedCliCount: number
  runtimeCliCount: number
  overview: HomeUsageOverviewResponse | null
  usageLoading: boolean
}>()

const { t } = useI18n()

type StatusTone = 'neutral' | 'success' | 'warning' | 'danger' | 'accent'

interface StatusChip {
  id: string
  label: string
  value: string
  hint?: string
  tone: StatusTone
}

const formatPercent = (value?: number) => {
  if (typeof value !== 'number' || Number.isNaN(value)) return '…'
  return `${value.toFixed(1)}%`
}

const formatNumber = (value?: number) => {
  if (typeof value !== 'number') return '…'
  return new Intl.NumberFormat(undefined, { notation: 'compact', maximumFractionDigits: 1 }).format(value)
}

const formatFixed = (value?: number) => {
  if (typeof value !== 'number' || Number.isNaN(value)) return '…'
  return value.toFixed(1)
}

const cpuTone = computed<StatusTone>(() => {
  const value = props.systemInfo?.cpu_usage
  if (typeof value !== 'number') return 'neutral'
  if (value >= 90) return 'danger'
  if (value >= 70) return 'warning'
  return 'neutral'
})

const memoryTone = computed<StatusTone>(() => {
  const value = props.systemInfo?.memory_usage_percent
  if (typeof value !== 'number') return 'neutral'
  if (value >= 92) return 'danger'
  if (value >= 78) return 'warning'
  return 'neutral'
})

const cliTone = computed<StatusTone>(() => {
  if (props.runtimeCliCount === 0) return 'neutral'
  if (props.installedCliCount === props.runtimeCliCount) return 'success'
  if (props.installedCliCount === 0) return 'warning'
  return 'warning'
})

const usageTone = computed<StatusTone>(() => {
  if (props.usageLoading) return 'neutral'
  if (!props.overview) return 'warning'
  if (props.overview.empty_reason) return 'warning'
  return 'accent'
})

const safeHostname = computed(() => {
  const hostname = props.systemInfo?.hostname?.trim()
  if (!hostname || hasTemplatePlaceholder(hostname)) {
    return t('home.systemMetricUnknown')
  }
  return hostname
})

const memoryHint = computed(() => {
  if (!props.systemInfo) return t('home.systemMetricPending')
  return translateWithFallback(
    t,
    'home.systemMetricMemory',
    '已用 {used} / {total} GB',
    {
      used: formatFixed(props.systemInfo.used_memory_gb),
      total: formatFixed(props.systemInfo.total_memory_gb),
    },
  )
})

const usageValue = computed(() => {
  if (props.usageLoading) return t('home.usagePreparing')
  if (!props.overview) return t('home.usageUnavailable')
  if (props.overview.empty_reason) return t('home.usageMissing')
  return formatNumber(props.overview.summary.total_requests)
})

const chips = computed<StatusChip[]>(() => [
  {
    id: 'cpu',
    label: t('home.cpuUsage'),
    value: formatPercent(props.systemInfo?.cpu_usage),
    hint: translateWithFallback(t, 'home.systemMetricHost', '主机：{host}', { host: safeHostname.value }),
    tone: cpuTone.value,
  },
  {
    id: 'memory',
    label: t('home.memoryUsage'),
    value: formatPercent(props.systemInfo?.memory_usage_percent),
    hint: memoryHint.value,
    tone: memoryTone.value,
  },
  {
    id: 'cli',
    label: t('home.statusCliLabel'),
    value: `${props.installedCliCount}/${props.runtimeCliCount}`,
    hint: t('home.systemMetricCliHint'),
    tone: cliTone.value,
  },
  {
    id: 'usage',
    label: t('home.usageMetricLabel'),
    value: usageValue.value,
    hint: t('home.systemMetricUsageHint'),
    tone: usageTone.value,
  },
])
</script>

<style scoped>
.home-status {
  display: flex;
  flex-wrap: nowrap;
  overflow-x: auto;
  gap: 0;
  border: 1px solid var(--home-border-hairline);
  border-radius: 10px;
  background: var(--home-surface-sunk);
  box-shadow: var(--home-elevation-sunk);
  padding: 0.55rem 0.4rem;
  scrollbar-width: none;
}

.home-status::-webkit-scrollbar {
  display: none;
}

.home-status-chip {
  display: inline-grid;
  grid-template:
    'label value dot' auto
    'hint  hint  hint' auto
    / auto auto auto;
  align-items: baseline;
  gap: 0.1rem 0.55rem;
  flex: 1 1 11rem;
  min-width: 11rem;
  padding: 0.25rem 0.85rem;
  border-right: 1px solid rgb(var(--color-border-default-rgb) / 10%);
}

.home-status-chip:last-child {
  border-right: 0;
}

.home-status-chip__label {
  grid-area: label;
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
  white-space: nowrap;
}

.home-status-chip__value {
  grid-area: value;
  color: var(--color-text-primary);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-mono);
  font-weight: 700;
  white-space: nowrap;
}

.home-status-chip__dot {
  grid-area: dot;
  align-self: center;
  width: 0.42rem;
  height: 0.42rem;
  border-radius: 999px;
  background: var(--color-text-muted);
}

.home-status-chip__hint {
  grid-area: hint;
  margin-top: 0.1rem;
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.home-status-chip[data-tone='success'] .home-status-chip__value,
.home-status-chip[data-tone='success'] .home-status-chip__dot {
  color: var(--color-success);
  background: var(--color-success);
}

.home-status-chip[data-tone='success'] .home-status-chip__value {
  background: transparent;
}

.home-status-chip[data-tone='warning'] .home-status-chip__value {
  color: var(--color-warning);
}

.home-status-chip[data-tone='warning'] .home-status-chip__dot {
  background: var(--color-warning);
}

.home-status-chip[data-tone='danger'] .home-status-chip__value {
  color: var(--color-danger);
}

.home-status-chip[data-tone='danger'] .home-status-chip__dot {
  background: var(--color-danger);
}

.home-status-chip[data-tone='accent'] .home-status-chip__value {
  color: var(--color-accent-primary);
}

.home-status-chip[data-tone='accent'] .home-status-chip__dot {
  background: var(--color-accent-primary);
}

@media (width <= 720px) {
  .home-status-chip {
    border-right: 0;
    border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 10%);
  }

  .home-status-chip:last-child {
    border-bottom: 0;
  }
}
</style>
