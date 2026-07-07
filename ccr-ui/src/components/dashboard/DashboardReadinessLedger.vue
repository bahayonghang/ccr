<template>
  <section
    class="dashboard-ledger"
    data-dashboard-readiness
    :data-status="readiness.status"
    :aria-label="t('dashboard.readiness.label')"
  >
    <p class="dashboard-ledger__eyebrow">
      <span
        class="dashboard-ledger__status-dot"
        aria-hidden="true"
      />
      {{ t(readiness.labelKey) }}
    </p>
    <h2 class="dashboard-ledger__title">
      {{ t(readiness.titleKey) }}
    </h2>
    <p class="dashboard-ledger__description">
      {{ t(readiness.descriptionKey) }}
    </p>

    <ul class="dashboard-ledger__reasons">
      <li
        v-for="reason in readiness.reasons"
        :key="reason.key"
        class="dashboard-ledger__reason"
        :data-ok="reason.ok"
      >
        <SIcon
          :name="reason.ok ? 'Check' : 'AlertTriangle'"
          size="w-3.5 h-3.5"
          class="dashboard-ledger__reason-icon"
        />
        {{ stripTrailingPeriod(t(reason.key)) }}
      </li>
    </ul>

    <div class="dashboard-ledger__metrics">
      <article
        v-for="metric in statusMetrics"
        :key="metric.id"
        class="dashboard-ledger-metric"
        :data-tone="metric.tone"
      >
        <span class="dashboard-ledger-metric__label">{{ t(metric.labelKey) }}</span>
        <strong class="dashboard-ledger-metric__value">
          {{ resolveValue(metric) }}
        </strong>
        <span class="dashboard-ledger-metric__hint">
          {{ resolveHint(metric) }}
        </span>
      </article>
    </div>
  </section>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type {
  DashboardReadiness,
  DashboardStatusMetric,
} from '@/views/dashboard/dashboardPresentation'

defineProps<{
  readiness: DashboardReadiness
  statusMetrics: DashboardStatusMetric[]
}>()

const { t } = useI18n()

const resolveValue = (metric: DashboardStatusMetric) => {
  if (metric.valueKey) return t(metric.valueKey)
  return metric.value ?? '…'
}

const resolveHint = (metric: DashboardStatusMetric) => {
  if (metric.hint) return metric.hint
  if (metric.hintKey) return t(metric.hintKey)
  return ''
}

// 清单行结尾不需要句号，翻译文案沿用既有句子即可，这里统一裁掉中英文句号
const stripTrailingPeriod = (text: string) => text.replace(/[。.]$/, '')
</script>

<style scoped>
.dashboard-ledger {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  height: 100%;
  padding: var(--home-card-pad);
  border: 1px solid var(--home-border-card);
  border-radius: var(--home-card-radius);
  background: var(--home-surface-card);
  box-shadow: var(--home-elevation-raised);
  overflow: hidden;
}

.dashboard-ledger::before {
  content: '';
  position: absolute;
  inset: 0;
  border-top: 2px solid rgb(var(--color-accent-primary-rgb) / 32%);
  pointer-events: none;
}

.dashboard-ledger__eyebrow {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  margin: 0;
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 800;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.dashboard-ledger__status-dot {
  width: 0.48rem;
  height: 0.48rem;
  border-radius: 999px;
  background: var(--color-text-muted);
}

.dashboard-ledger[data-status='ready'] .dashboard-ledger__status-dot {
  background: var(--color-success);
}

.dashboard-ledger[data-status='attention'] .dashboard-ledger__status-dot {
  background: var(--color-warning);
}

.dashboard-ledger[data-status='web-preview'] .dashboard-ledger__status-dot {
  background: var(--color-accent-primary);
}

.dashboard-ledger__title {
  margin: 0;
  color: var(--color-text-primary);
  font-family: var(--font-brand);
  font-size: var(--home-text-section);
  font-weight: 640;
  letter-spacing: var(--home-tracking-display);
  line-height: 1.2;
}

.dashboard-ledger__description {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: var(--home-text-meta);
  letter-spacing: var(--home-tracking-body);
  line-height: var(--home-leading-body);
}

.dashboard-ledger__reasons {
  display: grid;
  align-content: start;
  gap: 0.3rem;
  margin: 0;
  padding: 0;
  list-style: none;
}

.dashboard-ledger__reason {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  color: var(--color-text-secondary);
  font-size: var(--home-text-meta);
  line-height: 1.45;
}

.dashboard-ledger__reason-icon {
  flex-shrink: 0;
  color: var(--color-text-disabled);
}

.dashboard-ledger__reason[data-ok='true'] .dashboard-ledger__reason-icon {
  color: var(--color-success);
}

.dashboard-ledger__reason[data-ok='false'] .dashboard-ledger__reason-icon {
  color: var(--color-warning);
}

.dashboard-ledger__metrics {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.5rem;
  padding-top: 0.15rem;
}

.dashboard-ledger-metric {
  display: grid;
  gap: 0.16rem;
  min-width: 0;
  padding: 0.68rem 0.72rem;
  border: 1px solid var(--home-border-hairline);
  border-radius: 10px;
  background: rgb(var(--color-bg-surface-rgb) / 62%);
}

.dashboard-ledger-metric__label {
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 800;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.dashboard-ledger-metric__value {
  overflow: hidden;
  color: var(--color-text-primary);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-mono-lg);
  font-weight: 800;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dashboard-ledger-metric__hint {
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dashboard-ledger-metric[data-tone='success'] .dashboard-ledger-metric__value {
  color: var(--color-success);
}

.dashboard-ledger-metric[data-tone='warning'] .dashboard-ledger-metric__value {
  color: var(--color-warning);
}

.dashboard-ledger-metric[data-tone='danger'] .dashboard-ledger-metric__value {
  color: var(--color-danger);
}

.dashboard-ledger-metric[data-tone='accent'] .dashboard-ledger-metric__value {
  color: var(--color-accent-primary);
}

@media (width <= 640px) {
  .dashboard-ledger__metrics {
    grid-template-columns: 1fr;
  }
}
</style>

