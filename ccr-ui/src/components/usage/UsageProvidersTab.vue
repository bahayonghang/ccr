<template>
  <section class="providers-tab">
    <article class="providers-tab__hero glass-panel">
      <div>
        <p class="providers-tab__eyebrow">
          {{ $t('usage.dashboard.providers.eyebrow') }}
        </p>
        <h3>{{ $t('usage.dashboard.providers.title') }}</h3>
        <p>{{ $t('usage.dashboard.providers.subtitle') }}</p>
      </div>

      <div class="providers-tab__total">
        <span>{{ $t('usage.dashboard.providers.totalCost') }}</span>
        <strong>{{ formatCost(totalCost) }}</strong>
        <small>{{ $t('usage.dashboard.providers.costBasis') }}</small>
      </div>
    </article>

    <AsyncStatePanel
      v-if="unsupported"
      state="empty"
      :title="$t('usage.dashboard.providers.unsupportedTitle')"
      :description="unsupportedDescription"
      icon="Database"
      compact
    />

    <article
      v-else
      class="providers-tab__workspace glass-panel"
    >
      <div class="providers-tab__head">
        <div>
          <h3>{{ $t('usage.dashboard.providers.tableTitle') }}</h3>
          <p>{{ $t('usage.dashboard.providers.tableSubtitle', { window: selectedWindowLabel }) }}</p>
        </div>
        <span>
          {{ providerRows.length.toLocaleString() }}
          {{ $t('usage.dashboard.providers.providers') }}
        </span>
      </div>

      <div
        v-if="providerRows.length > 0"
        class="providers-tab__summary-grid"
      >
        <article class="providers-tab__summary-card">
          <span>{{ $t('usage.dashboard.providers.summaryProviders') }}</span>
          <strong>{{ attributedProviderCount.toLocaleString() }}</strong>
          <small>{{ $t('usage.dashboard.providers.summaryProvidersDetail') }}</small>
        </article>
        <article class="providers-tab__summary-card">
          <span>{{ $t('usage.dashboard.providers.summaryTokens') }}</span>
          <strong>{{ formatTokens(totalTokens) }}</strong>
          <small>{{ tokenMixSummary }}</small>
        </article>
        <article class="providers-tab__summary-card">
          <span>{{ $t('usage.dashboard.providers.summaryRequests') }}</span>
          <strong>{{ totalRequests.toLocaleString() }}</strong>
          <small>{{ $t('usage.dashboard.providers.costBasis') }}</small>
        </article>
      </div>

      <div
        v-if="providerRows.length > 0"
        class="providers-tab__table-shell"
      >
        <table class="providers-tab__table">
          <colgroup>
            <col class="providers-tab__col providers-tab__col--provider">
            <col class="providers-tab__col providers-tab__col--requests">
            <col class="providers-tab__col providers-tab__col--tokens">
            <col class="providers-tab__col providers-tab__col--tokens">
            <col class="providers-tab__col providers-tab__col--tokens">
            <col class="providers-tab__col providers-tab__col--tokens">
            <col class="providers-tab__col providers-tab__col--cost">
            <col class="providers-tab__col providers-tab__col--cost">
            <col class="providers-tab__col providers-tab__col--share">
          </colgroup>
          <thead>
            <tr>
              <th>{{ $t('usage.dashboard.table.provider') }}</th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.requests') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.input') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.output') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.cacheRead') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.cacheWrite') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.costWithCache') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.costWithoutCache') }}
              </th>
              <th class="is-right">
                {{ $t('usage.dashboard.table.share') }}
              </th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="row in providerRows"
              :key="row.key"
            >
              <td>
                <div class="providers-tab__provider">
                  <span>{{ row.label }}</span>
                  <small v-if="row.isUnattributed">
                    {{ $t('usage.dashboard.providers.unattributedHint') }}
                  </small>
                </div>
              </td>
              <td class="is-right">
                {{ row.request_count.toLocaleString() }}
              </td>
              <td class="is-right">
                {{ formatTokens(row.input_tokens) }}
              </td>
              <td class="is-right">
                {{ formatTokens(row.output_tokens) }}
              </td>
              <td class="is-right">
                {{ formatTokens(row.cache_read_tokens) }}
              </td>
              <td class="is-right">
                {{ formatTokens(row.cache_creation_tokens) }}
              </td>
              <td class="is-right providers-tab__cost-cell">
                <strong>{{ formatCost(row.cost_with_cache_usd) }}</strong>
                <small>{{ $t('usage.dashboard.providers.costBasis') }}</small>
              </td>
              <td class="is-right providers-tab__cost-cell">
                <strong>{{ formatCost(row.cost_without_cache_usd) }}</strong>
                <small>{{ $t('usage.dashboard.providers.costBasis') }}</small>
              </td>
              <td class="is-right providers-tab__share-cell">
                <span>{{ formatShare(row.total_tokens) }}</span>
                <span class="providers-tab__bar">
                  <span :style="{ width: `${barWidth(row.total_tokens)}%` }" />
                </span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div
        v-else
        class="providers-tab__empty"
      >
        {{ $t('usage.dashboard.providers.empty') }}
      </div>
    </article>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import AsyncStatePanel from '@/components/ui/AsyncStatePanel.vue'
import type { ProviderBreakdown, UsageFeatureCapability } from '@/types/usage'

const props = defineProps<{
  providerStats: ProviderBreakdown[]
  providerCapability: UsageFeatureCapability | null
  selectedWindowLabel: string
  formatCost: (value: number) => string
  formatTokens: (value: number) => string
}>()

const { t } = useI18n()

type ProviderRow = ProviderBreakdown & {
  key: string
  label: string
  isUnattributed: boolean
}

const unsupported = computed(() => props.providerCapability?.supported === false)
const unsupportedDescription = computed(() => {
  const detail = props.providerCapability?.detail?.trim()
  const hint = t('usage.dashboard.providers.unsupportedHint')
  return detail
    ? `${detail} ${hint}`
    : hint
})

const providerRows = computed<ProviderRow[]>(() =>
  [...props.providerStats]
    .filter((item) => item.total_tokens > 0 || item.request_count > 0 || item.cost_with_cache_usd > 0)
    .sort((left, right) =>
      right.total_tokens - left.total_tokens ||
      right.cost_with_cache_usd - left.cost_with_cache_usd ||
      providerKey(left).localeCompare(providerKey(right)),
    )
    .map((item) => ({
      ...item,
      key: providerKey(item),
      label: item.provider?.trim() || t('usage.dashboard.providers.unattributed'),
      isUnattributed: !item.provider?.trim(),
    }))
)

const attributedProviderCount = computed(
  () => providerRows.value.filter((item) => !item.isUnattributed).length
)
const totalCost = computed(() =>
  providerRows.value.reduce((sum, item) => sum + item.cost_with_cache_usd, 0)
)
const totalTokens = computed(() =>
  providerRows.value.reduce((sum, item) => sum + item.total_tokens, 0)
)
const totalInputTokens = computed(() =>
  providerRows.value.reduce((sum, item) => sum + item.input_tokens, 0)
)
const totalOutputTokens = computed(() =>
  providerRows.value.reduce((sum, item) => sum + item.output_tokens, 0)
)
const totalRequests = computed(() =>
  providerRows.value.reduce((sum, item) => sum + item.request_count, 0)
)
const tokenMixSummary = computed(() =>
  t('usage.dashboard.providers.summaryTokenMix', {
    input: props.formatTokens(totalInputTokens.value),
    output: props.formatTokens(totalOutputTokens.value),
  })
)

const providerKey = (item: ProviderBreakdown) => item.provider?.trim() || '__unattributed__'
const formatShare = (tokens: number) => {
  if (totalTokens.value <= 0) return '0%'
  return `${Math.round((tokens / totalTokens.value) * 100)}%`
}
const barWidth = (tokens: number) => {
  if (totalTokens.value <= 0 || tokens <= 0) return 0
  return Math.max(4, Math.min(100, (tokens / totalTokens.value) * 100))
}
</script>

<style scoped>
.providers-tab {
  display: grid;
  gap: 1rem;
}

.providers-tab__hero,
.providers-tab__workspace {
  overflow: hidden;
  border-radius: 1.6rem;
  padding: 1rem;
}

.providers-tab__hero {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  background:
    linear-gradient(135deg, rgb(var(--color-bg-elevated-rgb) / 90%), rgb(var(--color-bg-surface-rgb) / 72%)),
    radial-gradient(circle at 100% 0%, rgb(var(--color-info-rgb) / 12%), transparent 42%);
}

.providers-tab__eyebrow {
  color: var(--color-text-muted);
  font-size: 0.66rem;
  font-weight: 760;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.providers-tab h3 {
  margin-top: 0.18rem;
  color: var(--color-text-primary);
  font-size: 1rem;
  font-weight: 730;
  letter-spacing: -0.02em;
}

.providers-tab__hero p:not(.providers-tab__eyebrow),
.providers-tab__head p {
  margin-top: 0.34rem;
  max-width: 50rem;
  color: var(--color-text-secondary);
  font-size: 0.82rem;
  line-height: 1.6;
}

.providers-tab__total {
  display: grid;
  min-width: 13rem;
  gap: 0.16rem;
  justify-items: end;
}

.providers-tab__total span,
.providers-tab__total small,
.providers-tab__head span {
  color: var(--color-text-muted);
  font-size: 0.7rem;
  font-weight: 700;
}

.providers-tab__total strong {
  color: var(--color-text-primary);
  font-size: 2rem;
  font-weight: 760;
  letter-spacing: 0;
  font-variant-numeric: tabular-nums;
}

.providers-tab__workspace {
  display: grid;
  gap: 1rem;
}

.providers-tab__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.providers-tab__head span {
  flex: none;
  border: 1px solid rgb(var(--color-border-default-rgb) / 13%);
  border-radius: 999px;
  padding: 0.26rem 0.55rem;
}

.providers-tab__summary-grid {
  display: grid;
  gap: 0.75rem;
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.providers-tab__summary-card {
  display: grid;
  min-width: 0;
  gap: 0.28rem;
  border-radius: 1rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 54%), rgb(var(--color-bg-surface-rgb) / 28%)),
    radial-gradient(circle at 100% 0%, rgb(var(--color-info-rgb) / 8%), transparent 46%);
  padding: 0.82rem 0.92rem;
}

.providers-tab__summary-card span {
  color: var(--color-text-muted);
  font-size: 0.66rem;
  font-weight: 740;
  letter-spacing: 0.09em;
  text-transform: uppercase;
}

.providers-tab__summary-card strong {
  color: var(--color-text-primary);
  font-size: 1.28rem;
  font-weight: 720;
  letter-spacing: -0.03em;
  font-variant-numeric: tabular-nums;
}

.providers-tab__summary-card small {
  overflow: hidden;
  color: var(--color-text-secondary);
  font-size: 0.76rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.providers-tab__table-shell {
  max-height: 38rem;
  overflow: auto;
  border-radius: 1.15rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 18%);
  background: rgb(var(--color-bg-elevated-rgb) / 44%);
}

.providers-tab__table {
  width: 100%;
  min-width: 88rem;
  border-collapse: separate;
  border-spacing: 0;
}

.providers-tab__table thead th {
  position: sticky;
  top: 0;
  z-index: 1;
  padding: 0.85rem 1rem;
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 18%);
  background: rgb(var(--color-bg-elevated-rgb) / 94%);
  color: var(--color-text-muted);
  font-size: 0.74rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-align: left;
  text-transform: uppercase;
}

.providers-tab__table tbody td {
  padding: 0.92rem 1rem;
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 12%);
  color: var(--color-text-secondary);
  font-size: 0.9rem;
  font-variant-numeric: tabular-nums;
}

.providers-tab__table tbody tr:hover {
  background: rgb(var(--color-accent-primary-rgb) / 6%);
}

.providers-tab__table .is-right {
  text-align: right;
}

.providers-tab__provider {
  display: grid;
  min-width: 0;
  gap: 0.18rem;
}

.providers-tab__provider span {
  overflow: hidden;
  color: var(--color-text-primary);
  font-weight: 680;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.providers-tab__provider small,
.providers-tab__cost-cell small {
  display: block;
  margin-top: 0.12rem;
  color: var(--color-text-muted);
  font-size: 0.72rem;
  line-height: 1.35;
}

.providers-tab__cost-cell strong {
  color: var(--color-text-primary);
  font-weight: 720;
}

.providers-tab__share-cell {
  min-width: 8rem;
}

.providers-tab__share-cell > span:first-child {
  display: block;
  margin-bottom: 0.32rem;
  color: var(--color-text-primary);
  font-weight: 700;
}

.providers-tab__bar {
  display: block;
  overflow: hidden;
  height: 0.34rem;
  border-radius: 999px;
  background: rgb(var(--color-border-default-rgb) / 18%);
}

.providers-tab__bar span {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(
    90deg,
    rgb(var(--color-info-rgb) / 82%),
    rgb(var(--color-accent-secondary-rgb) / 74%)
  );
}

.providers-tab__empty {
  display: grid;
  min-height: 16rem;
  place-items: center;
  border-radius: 1.2rem;
  border: 1px dashed rgb(var(--color-border-default-rgb) / 18%);
  color: var(--color-text-muted);
}

.providers-tab__col--provider {
  width: 22rem;
}

.providers-tab__col--requests,
.providers-tab__col--tokens,
.providers-tab__col--cost,
.providers-tab__col--share {
  width: 9rem;
}

@media (width < 980px) {
  .providers-tab__hero,
  .providers-tab__head {
    flex-direction: column;
  }

  .providers-tab__total {
    justify-items: start;
  }

  .providers-tab__summary-grid {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
