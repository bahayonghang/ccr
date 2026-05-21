<template>
  <article
    class="usage-metric-card"
    :class="`usage-metric-card--${card.tone}`"
  >
    <div class="usage-metric-card__topline">
      <span class="usage-metric-card__icon">
        <SIcon
          :name="card.icon"
          size="w-4 h-4"
        />
      </span>
      <span class="usage-metric-card__label">{{ card.label }}</span>
    </div>

    <div class="usage-metric-card__body">
      <strong class="usage-metric-card__value">{{ card.value }}</strong>
      <span
        class="usage-metric-card__delta"
        :class="`usage-metric-card__delta--${card.deltaTone}`"
      >
        {{ card.deltaLabel }}
      </span>
    </div>

    <p class="usage-metric-card__detail">
      {{ card.detail }}
    </p>

    <div class="usage-metric-card__sparkline">
      <UsageSparkline
        :points="card.sparkline"
        :tone="card.tone"
        :label="card.sparklineLabel"
      />
    </div>

    <dl class="usage-metric-card__stats">
      <div>
        <dt>{{ $t('usage.dashboard.cards.average') }}</dt>
        <dd>{{ card.averageLabel }}</dd>
      </div>
      <div>
        <dt>{{ $t('usage.dashboard.cards.peak') }}</dt>
        <dd>{{ card.peakLabel }}</dd>
      </div>
    </dl>
  </article>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import UsageSparkline from './UsageSparkline.vue'
import type { UsageSummaryCard } from '@/views/usage/usageSummaryCards'

interface Props {
  card: UsageSummaryCard
}

defineProps<Props>()
</script>

<style scoped>
.usage-metric-card {
  --usage-metric-rgb: var(--color-accent-primary-rgb);

  position: relative;
  isolation: isolate;
  overflow: hidden;
  display: grid;
  gap: 0.62rem;
  min-height: 11.25rem;
  border-radius: 1.35rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  padding: 0.92rem 1rem 0.95rem;
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 90%), rgb(var(--color-bg-surface-rgb) / 74%)),
    radial-gradient(circle at 82% 0%, rgb(var(--usage-metric-rgb) / 8%), transparent 48%);
  box-shadow: var(--elevation-1), inset 0 1px 0 rgb(255 255 255 / 8%);
  transition:
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease),
    transform var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.usage-metric-card:hover {
  transform: translateY(-1px);
  border-color: rgb(var(--usage-metric-rgb) / 20%);
  box-shadow: var(--elevation-2), inset 0 1px 0 rgb(255 255 255 / 12%);
}

.usage-metric-card--rose {
  --usage-metric-rgb: var(--color-accent-primary-rgb);
}

.usage-metric-card--violet {
  --usage-metric-rgb: var(--color-accent-secondary-rgb);
}

.usage-metric-card--sky {
  --usage-metric-rgb: var(--color-info-rgb);
}

.usage-metric-card--amber {
  --usage-metric-rgb: var(--color-warning-rgb);
}

.usage-metric-card::before {
  content: '';
  position: absolute;
  inset: 0 auto 0 0;
  z-index: -1;
  width: 3px;
  background: linear-gradient(180deg, rgb(var(--usage-metric-rgb) / 82%), transparent);
}

.usage-metric-card::after {
  content: '';
  position: absolute;
  inset: auto -16% -36% auto;
  z-index: -1;
  width: 9rem;
  height: 9rem;
  border-radius: 9999px;
  background: radial-gradient(circle, rgb(var(--usage-metric-rgb) / 10%), transparent 62%);
}

.usage-metric-card__topline,
.usage-metric-card__body,
.usage-metric-card__stats {
  display: flex;
  align-items: center;
}

.usage-metric-card__topline,
.usage-metric-card__body {
  justify-content: space-between;
  gap: 0.7rem;
}

.usage-metric-card__icon {
  display: inline-flex;
  height: 1.9rem;
  width: 1.9rem;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  border-radius: 0.78rem;
  border: 1px solid rgb(var(--usage-metric-rgb) / 14%);
  background: rgb(var(--usage-metric-rgb) / 10%);
  color: rgb(var(--usage-metric-rgb));
}

.usage-metric-card__label {
  margin-right: auto;
  color: var(--color-text-muted);
  font-size: 0.68rem;
  font-weight: 750;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.usage-metric-card__value {
  min-width: 0;
  color: var(--color-text-primary);
  font-size: clamp(1.72rem, 1.6vw + 1rem, 2.35rem);
  font-weight: 760;
  letter-spacing: -0.04em;
  line-height: 1;
  font-variant-numeric: tabular-nums;
}

.usage-metric-card__delta {
  flex-shrink: 0;
  border-radius: 9999px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  padding: 0.24rem 0.52rem;
  background: rgb(var(--color-bg-elevated-rgb) / 64%);
  color: var(--color-text-secondary);
  font-size: 0.7rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.usage-metric-card__delta--up {
  border-color: rgb(var(--color-success-rgb) / 18%);
  background: rgb(var(--color-success-rgb) / 10%);
  color: var(--color-success);
}

.usage-metric-card__delta--down {
  border-color: rgb(var(--color-danger-rgb) / 16%);
  background: rgb(var(--color-danger-rgb) / 8%);
  color: var(--color-danger);
}

.usage-metric-card__delta--flat {
  color: var(--color-text-muted);
}

.usage-metric-card__detail {
  min-height: 2.1rem;
  color: var(--color-text-secondary);
  font-size: 0.78rem;
  line-height: 1.35;
}

.usage-metric-card__sparkline {
  min-width: 0;
  border-radius: 1rem;
  border: 1px solid rgb(var(--usage-metric-rgb) / 10%);
  background: rgb(var(--color-bg-elevated-rgb) / 28%);
  padding: 0.26rem 0.36rem;
}

.usage-metric-card__stats {
  justify-content: space-between;
  gap: 0.8rem;
  padding-top: 0.1rem;
}

.usage-metric-card__stats div {
  min-width: 0;
}

.usage-metric-card__stats dt {
  color: var(--color-text-muted);
  font-size: 0.65rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.usage-metric-card__stats dd {
  margin-top: 0.1rem;
  color: var(--color-text-primary);
  font-size: 0.78rem;
  font-weight: 650;
  font-variant-numeric: tabular-nums;
}
</style>
