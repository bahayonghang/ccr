<template>
  <section
    class="usage-token-strip"
    :aria-label="$t('usage.dashboard.tokenStrip.title')"
  >
    <div class="usage-token-strip__header">
      <div>
        <p class="usage-token-strip__eyebrow">
          {{ $t('usage.dashboard.tokenStrip.eyebrow') }}
        </p>
        <h2>{{ $t('usage.dashboard.tokenStrip.title') }}</h2>
      </div>
      <div class="usage-token-strip__efficiency">
        <span>{{ $t('usage.dashboard.tokenStrip.cacheEfficiency') }}</span>
        <strong>{{ cacheEfficiencyLabel }}</strong>
      </div>
    </div>

    <div class="usage-token-strip__track">
      <div
        v-for="item in visibleItems"
        :key="item.id"
        class="usage-token-strip__segment"
        :style="{ flexGrow: item.share, '--usage-token-rgb': `var(${item.color.rgbVar})` }"
        :title="`${item.label}: ${item.valueLabel}`"
      />
    </div>

    <div class="usage-token-strip__legend">
      <article
        v-for="item in items"
        :key="item.id"
        class="usage-token-strip__item"
        :class="`usage-token-strip__item--${item.id}`"
        :style="{ '--usage-token-rgb': `var(${item.color.rgbVar})` }"
      >
        <span class="usage-token-strip__dot" />
        <span class="usage-token-strip__label">{{ item.label }}</span>
        <strong class="usage-token-strip__value">{{ item.valueLabel }}</strong>
      </article>
    </div>

    <p class="usage-token-strip__note">
      {{ $t('usage.dashboard.tokenStrip.outputReasoningNote') }}
    </p>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { UsageSummary } from '@/types/usage'
import { formatPercent, formatTokens } from '@/views/usage/usageSummaryCards'
import { usageTokenCategoryColors, type UsageTokenCategory } from '@/views/usage/usageChartOptions'

interface Props {
  summary: UsageSummary
  cacheCreationTokens: number
}

const props = defineProps<Props>()
const { t } = useI18n()

type TokenStripItem = {
  id: Exclude<UsageTokenCategory, 'reasoning'>
  label: string
  value: number
  valueLabel: string
  share: number
  color: (typeof usageTokenCategoryColors)[UsageTokenCategory]
}

const total = computed(() =>
  Math.max(
    0,
    props.summary.total_input_tokens +
      props.summary.total_output_tokens +
      props.summary.total_cache_read_tokens +
      props.cacheCreationTokens,
  )
)

const buildItem = (
  id: Exclude<UsageTokenCategory, 'reasoning'>,
  labelKey: string,
  fallback: string,
  value: number,
) => ({
  id,
  label: t(labelKey) === labelKey ? fallback : t(labelKey),
  value,
  valueLabel: formatTokens(value),
  share: total.value > 0 ? Math.max(value / total.value, 0.015) : 1,
  color: usageTokenCategoryColors[id],
})

const items = computed<TokenStripItem[]>(() => [
  buildItem(
    'input',
    'usage.dashboard.tokenStrip.input',
    'Input',
    props.summary.total_input_tokens,
  ),
  buildItem(
    'output',
    'usage.dashboard.tokenStrip.output',
    'Output',
    props.summary.total_output_tokens,
  ),
  buildItem(
    'cacheRead',
    'usage.dashboard.tokenStrip.cacheRead',
    'Cache Read',
    props.summary.total_cache_read_tokens,
  ),
  buildItem(
    'cacheCreation',
    'usage.dashboard.tokenStrip.cacheCreation',
    'Cache Creation',
    props.cacheCreationTokens,
  ),
])

const visibleItems = computed(() => items.value.filter((item) => item.value > 0))
const cacheEfficiencyLabel = computed(() => formatPercent(props.summary.cache_efficiency))
</script>

<style scoped>
.usage-token-strip {
  display: grid;
  gap: 0.7rem;
  border-radius: 1.28rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 13%);
  background: var(--color-bg-surface);
  padding: 0.85rem 0.95rem;
  box-shadow: var(--elevation-1);
}

.usage-token-strip__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.8rem;
}

.usage-token-strip__eyebrow {
  color: var(--color-text-muted);
  font-size: 0.65rem;
  font-weight: 760;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.usage-token-strip h2 {
  margin-top: 0.12rem;
  color: var(--color-text-primary);
  font-size: 0.98rem;
  font-weight: 720;
  letter-spacing: -0.015em;
}

.usage-token-strip__efficiency {
  display: inline-flex;
  flex-shrink: 0;
  align-items: baseline;
  gap: 0.42rem;
  border-radius: 9999px;
  border: 1px solid rgb(var(--color-accent-secondary-rgb) / 16%);
  background: rgb(var(--color-accent-secondary-rgb) / 8%);
  padding: 0.34rem 0.62rem;
  color: var(--color-text-secondary);
  font-size: 0.72rem;
}

.usage-token-strip__efficiency strong {
  color: var(--color-text-primary);
  font-size: 0.82rem;
  font-variant-numeric: tabular-nums;
}

.usage-token-strip__track {
  overflow: hidden;
  display: flex;
  min-height: 0.52rem;
  gap: 0.18rem;
  border-radius: 9999px;
  background: rgb(var(--color-bg-overlay-rgb) / 56%);
  box-shadow: inset 0 0 0 1px rgb(var(--color-border-default-rgb) / 10%);
}

.usage-token-strip__segment {
  min-width: 0.35rem;
  border-radius: inherit;
  background: rgb(var(--usage-token-rgb));
}

.usage-token-strip__legend {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0.5rem;
}

.usage-token-strip__item {
  --usage-token-rgb: var(--color-accent-primary-rgb);

  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.42rem;
  border-radius: 0.95rem;
  border: 1px solid rgb(var(--usage-token-rgb) / 11%);
  background: rgb(var(--usage-token-rgb) / 5%);
  padding: 0.5rem 0.58rem;
  min-width: 0;
}

.usage-token-strip__dot {
  width: 0.48rem;
  height: 0.48rem;
  border-radius: 9999px;
  background: rgb(var(--usage-token-rgb));
}

.usage-token-strip__label {
  overflow: hidden;
  color: var(--color-text-secondary);
  font-size: 0.75rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.usage-token-strip__value {
  color: var(--color-text-primary);
  font-size: 0.78rem;
  font-variant-numeric: tabular-nums;
}

.usage-token-strip__item--input {
  --usage-token-rgb: var(--color-success-rgb);
}

.usage-token-strip__item--output {
  --usage-token-rgb: var(--color-info-rgb);
}

.usage-token-strip__item--cacheRead {
  --usage-token-rgb: var(--color-accent-secondary-rgb);
}

.usage-token-strip__item--cacheCreation {
  --usage-token-rgb: var(--color-warning-rgb);
}

.usage-token-strip__note {
  color: var(--color-text-muted);
  font-size: 0.73rem;
  line-height: 1.45;
}

@media (width < 980px) {
  .usage-token-strip__legend {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (width < 640px) {
  .usage-token-strip__header {
    flex-direction: column;
  }

  .usage-token-strip__legend {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
