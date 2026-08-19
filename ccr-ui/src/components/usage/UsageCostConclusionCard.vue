<template>
  <article
    class="usage-cost-conclusion"
    :class="`usage-cost-conclusion--${card.tone}`"
  >
    <div class="usage-cost-conclusion__head">
      <span class="usage-cost-conclusion__icon">
        <SIcon
          :name="card.icon"
          size="w-4 h-4"
        />
      </span>
      <span class="usage-cost-conclusion__label">{{ card.label }}</span>
    </div>

    <div class="usage-cost-conclusion__value-row">
      <strong class="usage-cost-conclusion__value">{{ card.value }}</strong>
      <span
        class="usage-cost-conclusion__delta"
        :class="`usage-cost-conclusion__delta--${card.deltaSentiment}`"
      >
        {{ card.deltaLabel }}
        <small>{{ $t('usage.dashboard.cards.periodOverPeriod') }}</small>
      </span>
    </div>

    <p class="usage-cost-conclusion__detail">
      {{ card.detail }}
    </p>

    <div
      v-if="$slots.default"
      class="usage-cost-conclusion__embed"
    >
      <slot />
    </div>
  </article>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import type { UsageSummaryCard } from '@/views/usage/usageSummaryCards'

defineProps<{
  card: UsageSummaryCard
}>()
</script>

<style scoped>
.usage-cost-conclusion {
  --usage-metric-rgb: var(--color-accent-secondary-rgb);

  position: relative;
  isolation: isolate;
  overflow: hidden;
  display: grid;
  gap: 0.75rem;
  height: 100%;
  border-radius: 1.35rem;
  border: 1px solid var(--color-border-subtle);
  padding: 1.05rem 1.15rem 1.1rem;
  background: var(--color-bg-surface);
}

.usage-cost-conclusion--rose {
  --usage-metric-rgb: var(--color-accent-primary-rgb);
}

.usage-cost-conclusion--sand {
  --usage-metric-rgb: var(--color-accent-secondary-rgb);
}

.usage-cost-conclusion--sky {
  --usage-metric-rgb: var(--color-info-rgb);
}

.usage-cost-conclusion--amber {
  --usage-metric-rgb: var(--color-warning-rgb);
}

.usage-cost-conclusion::before {
  display: none;
}

.usage-cost-conclusion__head {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.usage-cost-conclusion__icon {
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

.usage-cost-conclusion__label {
  color: var(--color-text-muted);
  font-size: 0.7rem;
  font-weight: 750;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.usage-cost-conclusion__value-row {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 0.6rem;
}

.usage-cost-conclusion__value {
  min-width: 0;
  color: var(--color-text-primary);
  font-size: 1.75rem;
  font-weight: 600;
  letter-spacing: 0;
  line-height: 1;
  font-variant-numeric: tabular-nums;
}

.usage-cost-conclusion__delta {
  display: inline-flex;
  align-items: baseline;
  gap: 0.34rem;
  border-radius: 9999px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  background: rgb(var(--color-bg-elevated-rgb) / 64%);
  padding: 0.26rem 0.6rem;
  color: var(--color-text-secondary);
  font-size: 0.8rem;
  font-weight: 720;
  font-variant-numeric: tabular-nums;
}

.usage-cost-conclusion__delta small {
  color: var(--color-text-muted);
  font-size: 0.66rem;
  font-weight: 650;
  font-variant-numeric: initial;
}

.usage-cost-conclusion__delta--positive {
  border-color: rgb(var(--color-success-rgb) / 18%);
  background: rgb(var(--color-success-rgb) / 10%);
  color: var(--color-success);
}

.usage-cost-conclusion__delta--negative {
  border-color: rgb(var(--color-danger-rgb) / 16%);
  background: rgb(var(--color-danger-rgb) / 8%);
  color: var(--color-danger);
}

.usage-cost-conclusion__delta--neutral {
  color: var(--color-text-muted);
}

.usage-cost-conclusion__detail {
  color: var(--color-text-secondary);
  font-size: 0.82rem;
  line-height: 1.4;
}

.usage-cost-conclusion__embed {
  margin-top: 0.15rem;
}
</style>
