<template>
  <article class="platform-usage-rank">
    <div class="platform-usage-rank__head">
      <p>{{ eyebrow }}</p>
      <h3>{{ title }}</h3>
    </div>

    <div
      v-if="rows.length"
      class="platform-usage-rank__list"
    >
      <div
        v-for="row in rows"
        :key="row.id"
        class="platform-usage-rank__row"
      >
        <div class="platform-usage-rank__row-copy">
          <strong :title="row.title">{{ row.label }}</strong>
          <span>{{ row.detail }}</span>
        </div>
        <div class="platform-usage-rank__row-value">
          <span>{{ row.displayValue }}</span>
          <i>
            <b :style="{ width: `${row.share}%` }" />
          </i>
        </div>
      </div>
    </div>

    <div
      v-else
      class="platform-usage-rank__empty"
    >
      {{ emptyLabel }}
    </div>
  </article>
</template>

<script setup lang="ts">
import type { PlatformUsageRankRow } from '@/types/platformUsageInsight'

defineProps<{
  title: string
  eyebrow: string
  rows: PlatformUsageRankRow[]
  emptyLabel: string
}>()
</script>

<style scoped>
.platform-usage-rank {
  display: grid;
  gap: 0.88rem;
  min-width: 0;
  border: 1px solid rgb(var(--color-border-default-rgb) / 13%);
  border-radius: 1.2rem;
  background: var(--color-bg-surface);
  padding: 1rem;
}

.platform-usage-rank__head {
  display: grid;
  gap: 0.18rem;
}

.platform-usage-rank__head p {
  color: var(--color-text-muted);
  font-size: 0.66rem;
  font-weight: 760;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.platform-usage-rank__head h3 {
  color: var(--color-text-primary);
  font-size: 0.98rem;
  font-weight: 720;
  letter-spacing: -0.02em;
}

.platform-usage-rank__list {
  display: grid;
  gap: 0.72rem;
}

.platform-usage-rank__row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(6rem, 8rem);
  gap: 0.72rem;
  align-items: center;
}

.platform-usage-rank__row-copy {
  min-width: 0;
}

.platform-usage-rank__row-copy strong,
.platform-usage-rank__row-copy span {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.platform-usage-rank__row-copy strong {
  color: var(--color-text-primary);
  font-size: 0.86rem;
  font-weight: 680;
}

.platform-usage-rank__row-copy span {
  margin-top: 0.12rem;
  color: var(--color-text-muted);
  font-size: 0.72rem;
}

.platform-usage-rank__row-value {
  display: grid;
  gap: 0.32rem;
  min-width: 0;
}

.platform-usage-rank__row-value span {
  color: var(--color-text-secondary);
  font-size: 0.76rem;
  font-variant-numeric: tabular-nums;
  font-weight: 690;
  text-align: right;
}

.platform-usage-rank__row-value i {
  display: block;
  overflow: hidden;
  height: 0.36rem;
  border-radius: 999px;
  background: rgb(var(--color-border-default-rgb) / 16%);
}

.platform-usage-rank__row-value b {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: rgb(var(--color-accent-primary-rgb) / 50%);
}

.platform-usage-rank__empty {
  min-height: 7.5rem;
  display: grid;
  place-items: center;
  border: 1px dashed rgb(var(--color-border-default-rgb) / 16%);
  border-radius: 1rem;
  color: var(--color-text-muted);
  font-size: 0.82rem;
  text-align: center;
}

@media (width < 640px) {
  .platform-usage-rank__row {
    grid-template-columns: 1fr;
  }

  .platform-usage-rank__row-value span {
    text-align: left;
  }
}
</style>
