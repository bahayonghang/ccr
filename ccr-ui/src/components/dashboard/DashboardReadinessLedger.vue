<template>
  <section
    class="dashboard-ledger"
    data-dashboard-readiness
    :data-status="readiness.status"
    :aria-label="t('dashboard.readiness.label')"
  >
    <div class="dashboard-ledger__verdict">
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
    </div>

    <ul class="dashboard-ledger__reasons">
      <li
        v-for="reasonKey in readiness.reasonKeys"
        :key="reasonKey"
        class="dashboard-ledger__reason"
      >
        {{ t(reasonKey) }}
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
</script>

<style scoped>
.dashboard-ledger {
  position: relative;
  display: grid;
  grid-template-columns: minmax(0, 1.05fr) minmax(16rem, 0.95fr);
  gap: 1rem;
  min-height: 100%;
  padding: clamp(1rem, 2vw, 1.35rem);
  border: 1px solid var(--home-border-card);
  border-radius: var(--home-card-radius);
  background:
    linear-gradient(135deg, rgb(var(--color-bg-elevated-rgb) / 90%), rgb(var(--color-bg-surface-rgb) / 78%)),
    radial-gradient(circle at top right, rgb(var(--color-accent-primary-rgb) / 8%), transparent 42%);
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

.dashboard-ledger__verdict {
  display: grid;
  align-content: start;
  gap: 0.45rem;
  min-width: 0;
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
  max-width: 42rem;
  margin: 0;
  color: var(--color-text-primary);
  font-family: var(--font-brand);
  font-size: clamp(1.65rem, 3.6vw, 3.05rem);
  font-weight: 640;
  letter-spacing: -0.055em;
  line-height: 0.96;
}

.dashboard-ledger__description {
  max-width: 44rem;
  margin: 0;
  color: var(--color-text-secondary);
  font-size: var(--home-text-body);
  letter-spacing: var(--home-tracking-body);
  line-height: var(--home-leading-body);
}

.dashboard-ledger__reasons {
  display: grid;
  align-content: start;
  gap: 0.45rem;
  margin: 0;
  padding: 0;
  list-style: none;
}

.dashboard-ledger__reason {
  padding: 0.48rem 0.65rem;
  border: 1px solid var(--home-border-hairline);
  border-radius: 10px;
  background: rgb(var(--color-bg-surface-rgb) / 58%);
  color: var(--color-text-secondary);
  font-size: var(--home-text-meta);
  line-height: 1.45;
}

.dashboard-ledger__metrics {
  grid-column: 1 / -1;
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
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

@media (width <= 1080px) {
  .dashboard-ledger {
    grid-template-columns: 1fr;
  }

  .dashboard-ledger__metrics {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (width <= 640px) {
  .dashboard-ledger__metrics {
    grid-template-columns: 1fr;
  }
}
</style>

