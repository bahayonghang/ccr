<template>
  <section class="diagnostics-tab glass-panel rounded-xl p-4">
    <div class="diagnostics-tab__head">
      <div>
        <h3 class="diagnostics-tab__title">
          {{ $t('usage.dashboard.logs.title') }}
        </h3>
        <p class="diagnostics-tab__subtitle">
          {{ $t('usage.dashboard.logs.subtitle') }}
        </p>
      </div>

      <div class="diagnostics-tab__filter-rail">
        <label class="diagnostics-tab__filter-field">
          <span>{{ $t('usage.dashboard.logs.filterLabel') }}</span>
          <input
            :value="logModelFilter"
            :placeholder="$t('usage.dashboard.logs.filterPlaceholder')"
            class="toolbar-select diagnostics-tab__filter-input"
            @input="updateLogModelFilter(($event.target as HTMLInputElement).value)"
            @keyup.enter="loadLogs('reset')"
          >
        </label>
        <button
          class="diagnostics-tab__filter-action"
          @click="loadLogs('reset')"
        >
          {{ $t('usage.dashboard.logs.search') }}
        </button>
      </div>
    </div>

    <div class="diagnostics-tab__summary">
      <article class="diagnostics-tab__summary-card">
        <span class="diagnostics-tab__summary-label">
          {{ $t('usage.dashboard.diagnostics.totalRecords') }}
        </span>
        <strong class="diagnostics-tab__summary-value">
          {{ diagnosticsSummary.totalRecords }}
        </strong>
      </article>
      <article class="diagnostics-tab__summary-card">
        <span class="diagnostics-tab__summary-label">
          {{ $t('usage.dashboard.diagnostics.latestRecord') }}
        </span>
        <strong class="diagnostics-tab__summary-value diagnostics-tab__summary-value--small">
          {{ diagnosticsSummary.latestRecordAt }}
        </strong>
      </article>
      <article class="diagnostics-tab__summary-card">
        <span class="diagnostics-tab__summary-label">
          {{ $t('usage.dashboard.diagnostics.dataHealth') }}
        </span>
        <div class="diagnostics-tab__health-row">
          <strong class="diagnostics-tab__summary-value diagnostics-tab__summary-value--small">
            {{ diagnosticsSummary.healthLabel }}
          </strong>
          <span
            class="diagnostics-tab__health-pill"
            :class="{
              'diagnostics-tab__health-pill--warning': diagnosticsSummary.repairRecommended,
            }"
          >
            {{ diagnosticsSummary.repairRecommended ? $t('usage.dashboard.diagnostics.needsAction') : $t('usage.dashboard.diagnostics.ready') }}
          </span>
        </div>
        <span class="diagnostics-tab__summary-detail">
          {{ diagnosticsSummary.healthDetail }}
        </span>
      </article>
    </div>

    <div
      v-if="diagnosticsSummary.repairRecommended"
      class="diagnostics-tab__repair-callout"
    >
      <div>
        <h4 class="diagnostics-tab__repair-title">
          {{ $t('usage.dashboard.diagnostics.repairTitle') }}
        </h4>
        <p class="diagnostics-tab__repair-detail">
          {{ diagnosticsSummary.healthDetail }}
        </p>
      </div>

      <button
        v-if="diagnosticsSummary.canRepairCodex"
        class="diagnostics-tab__repair-button"
        @click="repairCodexLogs"
      >
        {{ repairButtonLabel }}
      </button>
    </div>

    <div class="diagnostics-tab__ledger">
      <div class="diagnostics-tab__header diagnostics-tab__row">
        <div>
          {{ $t('usage.dashboard.table.time') }}
        </div>
        <div>
          {{ $t('usage.dashboard.table.platform') }}
        </div>
        <div>
          {{ $t('usage.dashboard.table.model') }}
        </div>
        <div class="is-right">
          {{ $t('usage.dashboard.table.input') }}
        </div>
        <div class="is-right">
          {{ $t('usage.dashboard.table.output') }}
        </div>
        <div class="is-right">
          {{ $t('usage.dashboard.table.cost') }}
        </div>
      </div>

      <div
        v-if="logsLoading"
        class="diagnostics-tab__state"
      >
        {{ $t('usage.states.loading') }}
      </div>

      <div
        v-else-if="logsRecords.length === 0"
        class="diagnostics-tab__state"
      >
        <strong class="diagnostics-tab__state-title">
          {{ diagnosticsEmptyMessage }}
        </strong>
        <span class="diagnostics-tab__state-detail">
          {{ diagnosticsEmptyDetail }}
        </span>
      </div>

      <div
        v-else
        class="diagnostics-tab__body"
      >
        <div
          v-for="record in logsRecords"
          :key="record.id"
          class="diagnostics-tab__row diagnostics-tab__row--item"
        >
          <div class="diagnostics-tab__cell diagnostics-tab__cell--time">
            {{ new Date(record.recorded_at).toLocaleString() }}
          </div>
          <div class="diagnostics-tab__cell">
            {{ record.platform }}
          </div>
          <div
            class="diagnostics-tab__cell diagnostics-tab__cell--model"
            :title="record.model || $t('usage.dashboard.diagnostics.unknownModel')"
          >
            {{ record.model || $t('usage.dashboard.diagnostics.unknownModel') }}
          </div>
          <div class="diagnostics-tab__cell is-right">
            {{ formatTokens(record.input_tokens) }}
          </div>
          <div class="diagnostics-tab__cell is-right">
            {{ formatTokens(record.output_tokens) }}
          </div>
          <div class="diagnostics-tab__cell is-right">
            {{ formatCost(record.cost_with_cache_usd) }}
          </div>
        </div>
      </div>
    </div>

    <div
      v-if="showPager"
      class="diagnostics-tab__pager"
    >
      <span class="diagnostics-tab__pager-status">
        {{ $t('usage.dashboard.logs.pageStatus', { page: logsPage, pages: hasLogsTotal ? logsTotalPages : '?' }) }}
      </span>
      <button
        class="diagnostics-tab__pager-button"
        :disabled="!canPrevLogs"
        @click="loadLogs('prev')"
      >
        {{ $t('usage.dashboard.logs.prev') }}
      </button>
      <span
        v-if="hasLogsTotal"
        class="diagnostics-tab__pager-text"
      >
        {{ logsPage }} / {{ logsTotalPages }}
      </span>
      <span
        v-else
        class="diagnostics-tab__pager-text"
      >
        {{ logsPage }}
      </span>
      <button
        class="diagnostics-tab__pager-button"
        :disabled="!canNextLogs"
        @click="loadLogs('next')"
      >
        {{ $t('usage.dashboard.logs.next') }}
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { UsageRecordV2 } from '@/types/usage'

type LogsDirection = 'reset' | 'next' | 'prev' | 'same'

type UsageDiagnosticsSummary = {
  totalRecords: string
  latestRecordAt: string
  healthLabel: string
  healthDetail: string
  repairRecommended: boolean
  canRepairCodex: boolean
}

interface Props {
  diagnosticsEmptyDetail: string
  diagnosticsEmptyMessage: string
  diagnosticsSummary: UsageDiagnosticsSummary
  logModelFilter: string
  logsLoading: boolean
  logsRecords: UsageRecordV2[]
  logsPage: number
  logsTotalPages: number
  canPrevLogs: boolean
  canNextLogs: boolean
  hasLogsTotal: boolean
  showPager: boolean
  repairButtonLabel: string
  formatCost: (value: number) => string
  formatTokens: (value: number) => string
  loadLogs: (direction?: LogsDirection) => void | Promise<void>
  repairCodexLogs: () => void | Promise<void>
  updateLogModelFilter: (value: string) => void
}

defineProps<Props>()
</script>

<style scoped>
.diagnostics-tab {
  display: grid;
  gap: 1rem;
  overflow: hidden;
}

.diagnostics-tab__head {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.diagnostics-tab__title {
  color: var(--color-text-primary);
  font-size: 1rem;
  font-weight: 650;
}

.diagnostics-tab__subtitle {
  margin-top: 0.35rem;
  color: var(--color-text-secondary);
  font-size: 0.82rem;
  line-height: 1.6;
}

.diagnostics-tab__filter-rail {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-end;
  gap: 0.65rem;
}

.diagnostics-tab__filter-field {
  display: grid;
  gap: 0.28rem;
}

.diagnostics-tab__filter-field span {
  color: var(--color-text-muted);
  font-size: 0.64rem;
  font-weight: 740;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.diagnostics-tab__filter-input {
  min-width: 18rem;
}

.diagnostics-tab__filter-action,
.diagnostics-tab__repair-button,
.diagnostics-tab__pager-button {
  display: inline-flex;
  min-height: 44px;
  align-items: center;
  justify-content: center;
  border-radius: 1rem;
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 24%);
  background: linear-gradient(135deg, rgb(var(--color-accent-primary-rgb) / 92%), rgb(var(--color-accent-secondary-rgb) / 84%));
  padding: 0 1rem;
  color: var(--color-text-inverted);
  font-size: 0.84rem;
  font-weight: 700;
  box-shadow: 0 12px 26px rgb(var(--color-accent-primary-rgb) / 18%);
  transition: transform var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.diagnostics-tab__filter-action:hover,
.diagnostics-tab__repair-button:hover,
.diagnostics-tab__pager-button:hover:not(:disabled) {
  transform: translateY(-1px);
}

.diagnostics-tab__summary {
  display: grid;
  gap: 0.85rem;
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.diagnostics-tab__summary-card {
  display: grid;
  gap: 0.35rem;
  min-width: 0;
  border-radius: 1rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 22%);
  background: rgb(var(--color-bg-elevated-rgb) / 44%);
  padding: 0.95rem 1rem;
}

.diagnostics-tab__summary-label {
  color: var(--color-text-muted);
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.diagnostics-tab__summary-value {
  color: var(--color-text-primary);
  font-size: 1.15rem;
  font-weight: 650;
  font-variant-numeric: tabular-nums;
}

.diagnostics-tab__summary-value--small {
  font-size: 0.96rem;
  line-height: 1.4;
}

.diagnostics-tab__summary-detail {
  color: var(--color-text-secondary);
  font-size: 0.8rem;
  line-height: 1.55;
}

.diagnostics-tab__health-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.55rem;
}

.diagnostics-tab__health-pill {
  display: inline-flex;
  min-height: 1.35rem;
  align-items: center;
  border-radius: 9999px;
  border: 1px solid rgb(var(--color-success-rgb) / 16%);
  background: rgb(var(--color-success-rgb) / 9%);
  padding: 0 0.48rem;
  color: var(--color-success);
  font-size: 0.66rem;
  font-weight: 760;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.diagnostics-tab__health-pill--warning {
  border-color: rgb(var(--color-warning-rgb) / 20%);
  background: rgb(var(--color-warning-rgb) / 10%);
  color: var(--color-warning);
}

.diagnostics-tab__repair-callout {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  border-radius: 1.15rem;
  border: 1px solid rgb(var(--color-warning-rgb) / 26%);
  background: linear-gradient(135deg, rgb(var(--color-warning-rgb) / 14%), rgb(var(--color-accent-primary-rgb) / 8%));
  padding: 1rem;
}

.diagnostics-tab__repair-title {
  color: var(--color-text-primary);
  font-size: 0.92rem;
  font-weight: 650;
}

.diagnostics-tab__repair-detail {
  margin-top: 0.25rem;
  color: var(--color-text-secondary);
  font-size: 0.82rem;
  line-height: 1.55;
}

.diagnostics-tab__ledger {
  overflow: hidden;
  border-radius: 1.15rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 18%);
  background: rgb(var(--color-bg-elevated-rgb) / 46%);
}

.diagnostics-tab__row {
  display: grid;
  grid-template-columns: minmax(12rem, 1.55fr) minmax(6rem, 0.72fr) minmax(14rem, 1.45fr) minmax(7rem, 0.8fr) minmax(7rem, 0.8fr) minmax(7rem, 0.78fr);
  min-width: 60rem;
}

.diagnostics-tab__header {
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 18%);
  background: rgb(var(--color-bg-elevated-rgb) / 92%);
}

.diagnostics-tab__header div {
  padding: 0.85rem 1rem;
  color: var(--color-text-muted);
  font-size: 0.74rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-align: left;
  text-transform: uppercase;
}

.diagnostics-tab__body {
  max-height: 32rem;
  overflow: auto;
}

.diagnostics-tab__row--item {
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 12%);
  transition: background-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.diagnostics-tab__row--item:hover {
  background: rgb(var(--color-accent-primary-rgb) / 6%);
}

.diagnostics-tab__cell {
  overflow: hidden;
  padding: 0.92rem 1rem;
  color: var(--color-text-secondary);
  font-size: 0.88rem;
  font-variant-numeric: tabular-nums;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.diagnostics-tab__cell--time,
.diagnostics-tab__cell--model {
  color: var(--color-text-primary);
}

.diagnostics-tab__cell--model {
  font-weight: 600;
}

.diagnostics-tab__row .is-right {
  text-align: right;
}

.diagnostics-tab__state {
  display: grid;
  gap: 0.35rem;
  min-height: 12rem;
  place-items: center;
  padding: 2rem 1rem;
  text-align: center;
}

.diagnostics-tab__state-title {
  color: var(--color-text-primary);
  font-size: 0.95rem;
  font-weight: 650;
}

.diagnostics-tab__state-detail {
  color: var(--color-text-secondary);
  font-size: 0.82rem;
  line-height: 1.55;
}

.diagnostics-tab__pager {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
}

.diagnostics-tab__pager-button {
  min-height: 40px;
  padding: 0 0.95rem;
  font-size: 0.8rem;
}

.diagnostics-tab__pager-button:disabled {
  cursor: not-allowed;
  opacity: 0.45;
  transform: none;
}

.diagnostics-tab__pager-text {
  color: var(--color-text-secondary);
  font-size: 0.82rem;
  font-variant-numeric: tabular-nums;
}

.diagnostics-tab__pager-status {
  flex-basis: 100%;
  color: var(--color-text-muted);
  font-size: 0.76rem;
  text-align: center;
  font-variant-numeric: tabular-nums;
}

@media (width < 1080px) {
  .diagnostics-tab__summary {
    grid-template-columns: minmax(0, 1fr);
  }
}

@media (width < 960px) {
  .diagnostics-tab__filter-input {
    min-width: 100%;
  }
}
</style>
