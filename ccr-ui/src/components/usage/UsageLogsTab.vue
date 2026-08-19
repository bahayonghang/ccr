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
            :value="ctx.logModelFilter"
            :placeholder="$t('usage.dashboard.logs.filterPlaceholder')"
            class="toolbar-select diagnostics-tab__filter-input"
            @input="ctx.updateLogModelFilter(($event.target as HTMLInputElement).value)"
            @keyup.enter="ctx.loadLogs('reset')"
          >
        </label>
        <button
          class="diagnostics-tab__filter-action"
          @click="ctx.loadLogs('reset')"
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
          {{ ctx.diagnosticsSummary.totalRecords }}
        </strong>
      </article>
      <article class="diagnostics-tab__summary-card">
        <span class="diagnostics-tab__summary-label">
          {{ $t('usage.dashboard.diagnostics.latestRecord') }}
        </span>
        <strong class="diagnostics-tab__summary-value diagnostics-tab__summary-value--small">
          {{ ctx.diagnosticsSummary.latestRecordAt }}
        </strong>
      </article>
      <article class="diagnostics-tab__summary-card">
        <span class="diagnostics-tab__summary-label">
          {{ $t('usage.dashboard.diagnostics.dataHealth') }}
        </span>
        <div class="diagnostics-tab__health-row">
          <strong class="diagnostics-tab__summary-value diagnostics-tab__summary-value--small">
            {{ ctx.diagnosticsSummary.healthLabel }}
          </strong>
          <span
            class="diagnostics-tab__health-pill"
            :class="{
              'diagnostics-tab__health-pill--warning': ctx.diagnosticsSummary.repairRecommended,
            }"
          >
            {{ ctx.diagnosticsSummary.repairRecommended ? $t('usage.dashboard.diagnostics.needsAction') : $t('usage.dashboard.diagnostics.ready') }}
          </span>
        </div>
        <span class="diagnostics-tab__summary-detail">
          {{ ctx.diagnosticsSummary.healthDetail }}
        </span>
      </article>
    </div>

    <div
      v-if="ctx.diagnosticsSummary.repairRecommended"
      class="diagnostics-tab__repair-callout"
    >
      <div>
        <h4 class="diagnostics-tab__repair-title">
          {{ $t('usage.dashboard.diagnostics.repairTitle') }}
        </h4>
        <p class="diagnostics-tab__repair-detail">
          {{ ctx.diagnosticsSummary.healthDetail }}
        </p>
      </div>

      <button
        v-if="ctx.diagnosticsSummary.canRepairCodex"
        class="diagnostics-tab__repair-button"
        @click="ctx.repairCodexLogs"
      >
        {{ ctx.repairCodexButtonLabel }}
      </button>
    </div>

    <!-- 骨架块本身 aria-hidden,加载状态经 aria-busy 暴露给辅助技术(含分页翻页)。 -->
    <div
      class="diagnostics-tab__ledger"
      :aria-busy="ctx.logsLoading"
    >
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
        v-if="ctx.logsLoading"
        class="diagnostics-tab__body"
        aria-hidden="true"
      >
        <div
          v-for="row in skeletonRowCount"
          :key="row"
          class="diagnostics-tab__row diagnostics-tab__row--item"
        >
          <div
            v-for="col in 6"
            :key="col"
            class="diagnostics-tab__cell"
            :class="{ 'is-right': col >= 4 }"
          >
            <span class="diagnostics-tab__skeleton" />
          </div>
        </div>
      </div>

      <div
        v-else-if="ctx.logsRecords.length === 0"
        class="diagnostics-tab__state"
      >
        <strong class="diagnostics-tab__state-title">
          {{ ctx.diagnosticsEmptyMessage }}
        </strong>
        <span class="diagnostics-tab__state-detail">
          {{ ctx.diagnosticsEmptyDetail }}
        </span>
      </div>

      <div
        v-else
        class="diagnostics-tab__body"
      >
        <div
          v-for="record in ctx.logsRecords"
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
            {{ ctx.formatTokens(record.input_tokens) }}
          </div>
          <div class="diagnostics-tab__cell is-right">
            {{ ctx.formatTokens(record.output_tokens) }}
          </div>
          <div class="diagnostics-tab__cell is-right">
            {{ ctx.formatCost(record.cost_with_cache_usd) }}
          </div>
        </div>
      </div>
    </div>

    <div
      v-if="ctx.showLogsPager"
      class="diagnostics-tab__pager"
    >
      <span class="diagnostics-tab__pager-status">
        {{ $t('usage.dashboard.logs.pageStatus', { page: ctx.logsPage, pages: ctx.hasLogsTotal ? ctx.logsTotalPages : '?' }) }}
      </span>
      <button
        class="diagnostics-tab__pager-button"
        :disabled="!ctx.canPrevLogs"
        @click="ctx.loadLogs('prev')"
      >
        {{ $t('usage.dashboard.logs.prev') }}
      </button>
      <span
        v-if="ctx.hasLogsTotal"
        class="diagnostics-tab__pager-text"
      >
        {{ ctx.logsPage }} / {{ ctx.logsTotalPages }}
      </span>
      <span
        v-else
        class="diagnostics-tab__pager-text"
      >
        {{ ctx.logsPage }}
      </span>
      <button
        class="diagnostics-tab__pager-button"
        :disabled="!ctx.canNextLogs"
        @click="ctx.loadLogs('next')"
      >
        {{ $t('usage.dashboard.logs.next') }}
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useUsageDashboardContext } from '@/views/usage/usageDashboardContext'

const ctx = useUsageDashboardContext()

// 骨架行数参考分页大小,但滚动容器 max-height 内只能看到约 12 行,超出部分渲染无意义。
const skeletonRowCount = computed(() => Math.min(ctx.logsPageSize, 12))
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
  border: 1px solid transparent;
  background: var(--color-accent-primary);
  padding: 0 1rem;
  color: var(--color-accent-primary-contrast);
  font-size: 0.84rem;
  font-weight: 600;
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
  background: var(--color-bg-surface);
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
  /* 唯一滚动容器:表头 sticky 依赖它,横向滚动时表头与行保持同步 */
  max-height: 35rem;
  overflow: auto;
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
  position: sticky;
  top: 0;
  z-index: 1;
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

.diagnostics-tab__row--item {
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 12%);
  transition: background-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.diagnostics-tab__skeleton {
  display: inline-block;
  width: 62%;
  height: 0.9rem;
  border-radius: 0.45rem;
  background: rgb(var(--color-border-default-rgb) / 26%);
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
