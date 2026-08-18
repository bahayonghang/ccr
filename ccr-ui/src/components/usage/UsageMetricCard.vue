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
        :class="`usage-metric-card__delta--${card.deltaSentiment}`"
      >
        {{ card.deltaLabel }}
      </span>
    </div>

    <p class="usage-metric-card__detail">
      {{ card.detail }}
    </p>

    <div class="usage-metric-card__sparkline">
      <Sparkline
        class="usage-metric-card__spark"
        :values="sparklineValues"
        :width="120"
        :height="38"
        :stroke-width="2.4"
        fill="currentColor"
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
import { computed } from 'vue'
import SIcon from '@/components/ui/SIcon.vue'
import Sparkline from '@/components/ui/Sparkline.vue'
import type { UsageSummaryCard } from '@/views/usage/usageSummaryCards'

interface Props {
  card: UsageSummaryCard
}

const props = defineProps<Props>()

// 统一 Sparkline 取数值序列；配色沿用卡片按 tone 设置的 --usage-metric-rgb（见 __spark 样式）
const sparklineValues = computed(() => props.card.sparkline.map((point) => point.value))
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
  border-radius: 0.75rem;
  border: 1px solid var(--color-border-subtle);
  padding: 0.92rem 1rem 0.95rem;
  background: var(--color-bg-surface);
}

.usage-metric-card--rose {
  --usage-metric-rgb: var(--color-accent-primary-rgb);
}

.usage-metric-card--sand {
  --usage-metric-rgb: var(--color-accent-secondary-rgb);
}

.usage-metric-card--sky {
  --usage-metric-rgb: var(--color-info-rgb);
}

.usage-metric-card--amber {
  --usage-metric-rgb: var(--color-warning-rgb);
}

.usage-metric-card::before,
.usage-metric-card::after {
  display: none;
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
  font-size: 0.8125rem;
  font-weight: 500;
  letter-spacing: 0;
}

.usage-metric-card__value {
  min-width: 0;
  color: var(--color-text-primary);
  font-size: 1.5rem;
  font-weight: 600;
  letter-spacing: 0;
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

.usage-metric-card__delta--positive {
  border-color: rgb(var(--color-success-rgb) / 18%);
  background: rgb(var(--color-success-rgb) / 10%);
  color: var(--color-success);
}

.usage-metric-card__delta--negative {
  border-color: rgb(var(--color-danger-rgb) / 16%);
  background: rgb(var(--color-danger-rgb) / 8%);
  color: var(--color-danger);
}

.usage-metric-card__delta--neutral {
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

.usage-metric-card__spark {
  width: 100%;
  height: 2.45rem;
  color: rgb(var(--usage-metric-rgb) / 92%);
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
