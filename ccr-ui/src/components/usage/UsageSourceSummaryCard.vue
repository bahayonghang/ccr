<template>
  <section class="source-card glass-panel">
    <div class="source-card__head">
      <div>
        <p class="source-card__eyebrow">
          {{ $t('usage.dashboard.sources.eyebrow') }}
        </p>
        <h3>{{ $t('usage.dashboard.sources.title') }}</h3>
        <p>{{ $t('usage.dashboard.sources.subtitle') }}</p>
      </div>
      <span class="source-card__count">
        {{ visibleSources.length }} {{ $t('usage.dashboard.sources.sources') }}
      </span>
    </div>

    <div
      v-if="visibleSources.length > 0"
      class="source-card__grid"
    >
      <button
        v-for="item in visibleSources"
        :key="item.source"
        type="button"
        class="source-card__item"
        :class="{ 'source-card__item--active': selectedPlatform === item.source }"
        @click="onSelectSource(item.source)"
      >
        <span class="source-card__source-row">
          <span class="source-card__source-name">{{ sourceLabel(item.source) }}</span>
          <span class="source-card__share">{{ formatPercent(item.share_tokens) }}</span>
        </span>

        <span class="source-card__metrics">
          <strong>{{ formatTokens(item.total_tokens) }}</strong>
          <span>{{ formatCost(item.total_cost) }}</span>
        </span>

        <span class="source-card__meta">
          {{ item.event_count.toLocaleString() }}
          {{ $t('usage.dashboard.sources.requests') }}
          ·
          {{ item.active_days.toLocaleString() }}
          {{ $t('usage.dashboard.sources.activeDays') }}
        </span>

        <span class="source-card__bar">
          <span :style="{ width: `${barWidth(item.share_tokens)}%` }" />
        </span>
      </button>
    </div>

    <div
      v-else
      class="source-card__empty"
    >
      {{ $t('usage.dashboard.table.noData') }}
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { UsagePlatform, SourceBreakdown } from '@/types/usage'
import { formatPercent } from '@/views/usage/usageSummaryCards'
import { isUsagePlatform, usageSourceFallbackLabel } from '@/views/usage/usageSources'

const props = defineProps<{
  sourceStats: SourceBreakdown[]
  selectedPlatform: string
  formatCost: (value: number) => string
  formatTokens: (value: number) => string
}>()

const emit = defineEmits<{
  'select-source': [source: UsagePlatform]
}>()

// wire 上 source 是 string，视图层在 emit 边界收窄为 UsagePlatform；
// 未知平台（理论上不出现）不触发选择。
const onSelectSource = (source: string) => {
  if (isUsagePlatform(source)) {
    emit('select-source', source)
  }
}

const visibleSources = computed(() =>
  [...props.sourceStats]
    .filter((item) => item.total_tokens > 0 || item.total_cost > 0 || item.event_count > 0)
    .sort((left, right) =>
      right.total_tokens - left.total_tokens ||
      right.total_cost - left.total_cost ||
      left.source.localeCompare(right.source),
    )
)

// 已知平台显示品牌名，未知值原样回显
const sourceLabel = (source: string) => usageSourceFallbackLabel(source)
const barWidth = (share: number) => Math.min(100, Math.max(share * 100, share > 0 ? 4 : 0))
</script>

<style scoped>
.source-card {
  display: grid;
  gap: 0.9rem;
  overflow: hidden;
  border-radius: 1.6rem;
  padding: 1rem;
}

.source-card__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.source-card__eyebrow {
  color: var(--color-text-muted);
  font-size: 0.66rem;
  font-weight: 760;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.source-card h3 {
  margin-top: 0.18rem;
  color: var(--color-text-primary);
  font-size: 1rem;
  font-weight: 730;
  letter-spacing: -0.02em;
}

.source-card__head p:not(.source-card__eyebrow) {
  margin-top: 0.28rem;
  max-width: 44rem;
  color: var(--color-text-secondary);
  font-size: 0.82rem;
  line-height: 1.55;
}

.source-card__count {
  flex: none;
  border-radius: 999px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  background: rgb(var(--color-bg-elevated-rgb) / 46%);
  padding: 0.3rem 0.62rem;
  color: var(--color-text-muted);
  font-size: 0.7rem;
  font-weight: 700;
}

.source-card__grid {
  display: grid;
  gap: 0.7rem;
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.source-card__item {
  display: grid;
  min-width: 0;
  gap: 0.48rem;
  border-radius: 1.15rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 58%), rgb(var(--color-bg-surface-rgb) / 34%));
  padding: 0.82rem 0.9rem;
  text-align: left;
  transition:
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    transform var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.source-card__item:hover,
.source-card__item--active {
  transform: translateY(-1px);
  border-color: rgb(var(--color-accent-secondary-rgb) / 28%);
  background:
    linear-gradient(180deg, rgb(var(--color-accent-secondary-rgb) / 10%), rgb(var(--color-bg-elevated-rgb) / 58%));
}

.source-card__source-row,
.source-card__metrics,
.source-card__meta {
  display: flex;
  min-width: 0;
  align-items: baseline;
  justify-content: space-between;
  gap: 0.6rem;
}

.source-card__source-name {
  overflow: hidden;
  color: var(--color-text-primary);
  font-size: 0.9rem;
  font-weight: 720;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.source-card__share,
.source-card__meta {
  color: var(--color-text-muted);
  font-size: 0.72rem;
  font-weight: 650;
}

.source-card__metrics strong {
  color: var(--color-text-primary);
  font-size: 1.24rem;
  font-weight: 760;
  letter-spacing: -0.03em;
  font-variant-numeric: tabular-nums;
}

.source-card__metrics span {
  flex: none;
  color: var(--color-text-secondary);
  font-size: 0.82rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.source-card__meta {
  justify-content: flex-start;
  line-height: 1.35;
}

.source-card__bar {
  overflow: hidden;
  height: 0.38rem;
  border-radius: 999px;
  background: rgb(var(--color-border-default-rgb) / 18%);
}

.source-card__bar span {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(
    90deg,
    rgb(var(--color-accent-secondary-rgb) / 86%),
    rgb(var(--color-warning-rgb) / 74%)
  );
}

.source-card__empty {
  display: grid;
  min-height: 8rem;
  place-items: center;
  border-radius: 1.1rem;
  border: 1px dashed rgb(var(--color-border-default-rgb) / 18%);
  color: var(--color-text-muted);
}

@media (width < 1180px) {
  .source-card__grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (width < 700px) {
  .source-card__head {
    flex-direction: column;
  }

  .source-card__grid {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
