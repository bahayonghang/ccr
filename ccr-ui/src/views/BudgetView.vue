<template>
  <PageShell class="budget-view">
    <template #header>
      <PageHeader
        :title="tt('预算管理', 'Budget Management')"
        :description="tt('管理成本预算限制和警告阈值', 'Manage spending limits and warning thresholds')"
      >
        <template #actions>
          <button
            type="button"
            :disabled="loading"
            class="budget-primary-button"
            @click="loadData"
          >
            <svg
              class="budget-primary-button__icon"
              :class="{ 'animate-spin': loading }"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
              />
            </svg>
            <span>{{ tt('刷新', 'Refresh') }}</span>
          </button>
        </template>
      </PageHeader>
    </template>

    <div
      v-if="loading"
      class="budget-shell budget-shell--loading"
      aria-live="polite"
    >
      <div class="budget-loading">
        <div class="budget-loading__spinner animate-spin" />
        <p class="budget-loading__text text-text-secondary">
          {{ tt('正在加载预算数据...', 'Loading budget data...') }}
        </p>
      </div>
    </div>

    <div
      v-if="error"
      class="budget-error"
      role="alert"
    >
      <div class="budget-error__layout">
        <svg
          class="budget-error__icon"
          fill="currentColor"
          viewBox="0 0 20 20"
        >
          <path
            fill-rule="evenodd"
            d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
            clip-rule="evenodd"
          />
        </svg>
        <div>
          <h2 class="budget-error__title">
            {{ tt('加载失败', 'Load failed') }}
          </h2>
          <p class="budget-error__message">
            {{ error }}
          </p>
        </div>
      </div>
    </div>

    <div
      v-if="!loading && !error && budgetStatus"
      class="budget-content"
    >
      <section class="budget-shell">
        <div class="budget-section-header">
          <div>
            <h2 class="budget-section-title">
              {{ tt('预算状态', 'Budget status') }}
            </h2>
            <p class="budget-section-copy">
              {{ tt('当前预算开关与各周期成本总览', 'Current budget switch plus period cost overview') }}
            </p>
          </div>

          <span
            class="budget-status-pill"
            :class="budgetStatus.enabled ? 'budget-status-pill--on' : 'budget-status-pill--off'"
          >
            {{ budgetStatus.enabled ? tt('已启用', 'Enabled') : tt('已禁用', 'Disabled') }}
          </span>
        </div>

        <div class="budget-overview-grid">
          <StatTile
            :label="tt('今日成本', 'Today cost')"
            :value="`$${budgetStatus.current_costs.today.toFixed(4)}`"
          />
          <StatTile
            :label="tt('本周成本', 'This week cost')"
            :value="`$${budgetStatus.current_costs.this_week.toFixed(4)}`"
          />
          <StatTile
            :label="tt('本月成本', 'This month cost')"
            :value="`$${budgetStatus.current_costs.this_month.toFixed(4)}`"
          />
        </div>

        <div class="budget-limits">
          <h3 class="budget-subsection-title text-text-primary">
            {{ tt('预算限制', 'Budget limits') }}
          </h3>
          <div class="budget-limit-grid">
            <div
              v-for="(limit, period) in budgetLimits"
              :key="period"
              class="budget-limit-row"
            >
              <span class="budget-limit-row__label text-text-secondary">
                {{ period }}
              </span>
              <span class="budget-limit-row__value text-text-primary">
                {{ formatLimitValue(limit) }}
              </span>
            </div>
          </div>
        </div>

        <div
          v-if="budgetStatus.warnings.length > 0"
          class="budget-warning-group"
        >
          <h3 class="budget-subsection-title text-accent-danger">
            {{ tt('预算警告', 'Budget warnings') }}
          </h3>
          <div
            v-for="(warning, index) in budgetStatus.warnings"
            :key="index"
            class="budget-warning-card"
          >
            <p class="budget-warning-card__text">
              {{ formatWarningSummary(warning) }}
            </p>
          </div>
        </div>
      </section>

      <section class="budget-shell">
        <div class="budget-section-header budget-section-header--compact">
          <div>
            <h2 class="budget-section-title">
              {{ tt('配置预算', 'Configure budget') }}
            </h2>
            <p class="budget-section-copy">
              {{ tt('调整预算开关、上限以及告警阈值', 'Adjust the budget switch, hard limits, and warning threshold') }}
            </p>
          </div>
        </div>

        <form
          class="budget-form"
          @submit.prevent="saveBudget"
        >
          <div class="budget-toggle-card">
            <label
              for="enabled"
              class="budget-toggle"
            >
              <input
                id="enabled"
                v-model="form.enabled"
                type="checkbox"
                class="budget-checkbox"
              >
              <div>
                <p class="budget-toggle__title text-text-primary">
                  {{ tt('启用预算控制', 'Enable budget control') }}
                </p>
                <p class="budget-toggle__copy text-text-secondary">
                  {{ tt('开启后将根据下方限制进行预算提醒', 'Enable reminders based on the limits below') }}
                </p>
              </div>
            </label>
          </div>

          <div class="budget-input-grid">
            <div>
              <label
                for="daily_limit"
                class="budget-label text-text-secondary"
              >
                {{ tt('每日限制 ($)', 'Daily limit ($)') }}
              </label>
              <input
                id="daily_limit"
                v-model.number="form.daily_limit"
                type="number"
                step="0.01"
                min="0"
                class="budget-input"
                placeholder="留空表示无限制"
              >
            </div>

            <div>
              <label
                for="weekly_limit"
                class="budget-label text-text-secondary"
              >
                {{ tt('每周限制 ($)', 'Weekly limit ($)') }}
              </label>
              <input
                id="weekly_limit"
                v-model.number="form.weekly_limit"
                type="number"
                step="0.01"
                min="0"
                class="budget-input"
                placeholder="留空表示无限制"
              >
            </div>

            <div>
              <label
                for="monthly_limit"
                class="budget-label text-text-secondary"
              >
                {{ tt('每月限制 ($)', 'Monthly limit ($)') }}
              </label>
              <input
                id="monthly_limit"
                v-model.number="form.monthly_limit"
                type="number"
                step="0.01"
                min="0"
                class="budget-input"
                placeholder="留空表示无限制"
              >
            </div>
          </div>

          <div>
            <label
              for="warn_threshold"
              class="budget-label text-text-secondary"
            >
              {{ tt('警告阈值 (%)', 'Warning threshold (%)') }}
            </label>
            <input
              id="warn_threshold"
              v-model.number="form.warn_threshold"
              type="number"
              min="0"
              max="100"
              class="budget-input"
            >
          </div>

          <div class="budget-form-actions">
            <button
              type="submit"
              :disabled="saving"
              class="budget-primary-button budget-primary-button--wide"
            >
              {{ saving ? tt('保存中...', 'Saving...') : tt('保存配置', 'Save settings') }}
            </button>
            <button
              type="button"
              :disabled="saving"
              class="budget-secondary-button"
              @click="handleReset"
            >
              {{ tt('重置所有限制', 'Reset all limits') }}
            </button>
          </div>
        </form>
      </section>
    </div>
  </PageShell>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
import StatTile from '@/components/ui/StatTile.vue'
import { getBudgetStatus, setBudget, resetBudget } from '@/api'
import { useUIStore } from '@/stores/ui'
import type { BudgetStatus, SetBudgetRequest } from '@/types'
import { getErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'

const { locale } = useI18n()
const uiStore = useUIStore()
const isZh = computed(() => locale.value.startsWith('zh'))
const tt = (zh: string, en: string) => (isZh.value ? zh : en)
const budgetStatus = ref<BudgetStatus | null>(null)
const loading = ref(false)
const saving = ref(false)
const error = ref<string | null>(null)

const form = ref<{
  enabled: boolean
  daily_limit: number | null
  weekly_limit: number | null
  monthly_limit: number | null
  warn_threshold: number
}>({
  enabled: false,
  daily_limit: null,
  weekly_limit: null,
  monthly_limit: null,
  warn_threshold: 80,
})

const budgetLimits = computed(() => {
  if (!budgetStatus.value) return {}
  return {
    [tt('每日', 'Daily')]: budgetStatus.value.daily_limit,
    [tt('每周', 'Weekly')]: budgetStatus.value.weekly_limit,
    [tt('每月', 'Monthly')]: budgetStatus.value.monthly_limit,
  }
})

const formatLimitValue = (limit: number | null | undefined) => (
  limit !== null && limit !== undefined ? `$${limit.toFixed(2)}` : tt('无限制', 'Unlimited')
)

const formatWarningSummary = (warning: BudgetStatus['warnings'][number]) => (
  isZh.value
    ? `${warning.period}: 当前成本 $${warning.current_cost.toFixed(2)} / 限制 $${warning.limit.toFixed(2)} (${warning.usage_percent.toFixed(1)}%)`
    : `${warning.period}: current cost $${warning.current_cost.toFixed(2)} / limit $${warning.limit.toFixed(2)} (${warning.usage_percent.toFixed(1)}%)`
)

const loadData = async () => {
  loading.value = true
  error.value = null

  try {
    const status = await getBudgetStatus()
    budgetStatus.value = status

    form.value.enabled = status.enabled
    form.value.daily_limit = status.daily_limit
    form.value.weekly_limit = status.weekly_limit
    form.value.monthly_limit = status.monthly_limit
    form.value.warn_threshold = status.warn_threshold
  } catch (e: unknown) {
    error.value = getErrorMessage(e) || tt('加载失败', 'Load failed')
    logger.error('Failed to load budget:', e)
  } finally {
    loading.value = false
  }
}

const saveBudget = async () => {
  saving.value = true

  try {
    const request: SetBudgetRequest = {
      enabled: form.value.enabled,
      daily_limit: form.value.daily_limit,
      weekly_limit: form.value.weekly_limit,
      monthly_limit: form.value.monthly_limit,
      warn_threshold: form.value.warn_threshold,
    }

    await setBudget(request)
    await loadData()

    uiStore.showSuccess(tt('配置已保存', 'Configuration saved'))
  } catch (e: unknown) {
    uiStore.showError(`${tt('保存失败', 'Save failed')}: ${getErrorMessage(e) || tt('未知错误', 'Unknown error')}`)
    logger.error('Failed to save budget:', e)
  } finally {
    saving.value = false
  }
}

const handleReset = async () => {
  const confirmed = await uiStore.requestConfirm({
    title: tt('重置预算限制', 'Reset budget limits'),
    message: tt('确定要重置所有预算限制吗？', 'Are you sure you want to reset all budget limits?'),
    confirmText: tt('重置', 'Reset'),
    cancelText: tt('取消', 'Cancel'),
    type: 'danger'
  })
  if (!confirmed) return

  saving.value = true

  try {
    await resetBudget()
    await loadData()

    uiStore.showSuccess(tt('预算限制已重置', 'Budget limits reset'))
  } catch (e: unknown) {
    uiStore.showError(`${tt('重置失败', 'Reset failed')}: ${getErrorMessage(e) || tt('未知错误', 'Unknown error')}`)
    logger.error('Failed to reset budget:', e)
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  loadData()
})
</script>

<style scoped>
.budget-view,
.budget-content,
.budget-loading,
.budget-form,
.budget-warning-group {
  display: flex;
  flex-direction: column;
}

.budget-view {
  min-width: 0;
}

.budget-content,
.budget-form,
.budget-warning-group {
  gap: 1.5rem;
}

.budget-shell {
  border: 1px solid var(--color-border-subtle);
  border-radius: 0.75rem;
  padding: 1.25rem;
  background: var(--color-bg-surface);
}

.budget-shell__header,
.budget-section-header,
.budget-limit-row,
.budget-toggle,
.budget-form-actions,
.budget-primary-button,
.budget-secondary-button {
  display: flex;
  align-items: center;
}

.budget-shell__header,
.budget-section-header,
.budget-limit-row {
  justify-content: space-between;
  gap: 1rem;
}

.budget-shell__header {
  flex-direction: column;
  align-items: flex-start;
}

.budget-shell__intro {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.budget-shell__title-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.budget-shell__icon {
  display: none;
}

.budget-shell__icon-svg,
.budget-primary-button__icon {
  width: 1.25rem;
  height: 1.25rem;
}

.budget-title {
  font-size: 1.5rem;
  font-weight: 700;
  line-height: 1.2;
}

.budget-subtitle,
.budget-section-copy,
.budget-loading__text,
.budget-label,
.budget-toggle__copy,
.budget-overview-card__label,
.budget-limit-row__label {
  font-size: 0.875rem;
}

.budget-primary-button,
.budget-secondary-button {
  justify-content: center;
  min-height: 44px;
  border-radius: 0.75rem;
  padding: 0.625rem 1rem;
  transition: transform 0.2s ease, box-shadow 0.2s ease, background-color 0.2s ease, color 0.2s ease,
    border-color 0.2s ease, opacity 0.2s ease;
}

.budget-primary-button {
  gap: 0.5rem;
  background: var(--color-accent-primary);
  color: var(--color-accent-primary-contrast);
  font-size: 0.875rem;
  font-weight: 600;
}

.budget-primary-button:hover:not(:disabled) {
  background: var(--color-accent-primary-hover);
}

.budget-primary-button--wide {
  padding-inline: 1.5rem;
}

.budget-secondary-button {
  border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-surface);
  color: var(--color-text-secondary);
  padding-inline: 1.5rem;
  font-weight: 500;
}

.budget-secondary-button:hover:not(:disabled) {
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
}

.budget-primary-button:disabled,
.budget-secondary-button:disabled {
  cursor: not-allowed;
  opacity: 0.5;
  transform: none;
}

.budget-shell--loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding-block: 4rem;
}

.budget-loading {
  align-items: center;
  gap: 1rem;
}

.budget-loading__spinner {
  width: 3rem;
  height: 3rem;
  border: 4px solid rgb(var(--color-accent-primary-rgb) / 15%);
  border-top-color: var(--color-accent-primary);
  border-radius: 9999px;
}

.budget-error {
  border: 1px solid rgb(var(--color-danger-rgb) / 30%);
  border-radius: 0.75rem;
  background: rgb(var(--color-danger-rgb) / 10%);
  padding: 1rem;
  color: var(--color-danger);
}

.budget-error__layout {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
}

.budget-error__icon {
  width: 1.25rem;
  height: 1.25rem;
  margin-top: 0.125rem;
  flex: none;
  color: var(--color-danger);
}

.budget-error__title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-danger);
}

.budget-error__message {
  margin-top: 0.25rem;
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.budget-section-header {
  margin-bottom: 1.5rem;
  flex-direction: column;
  align-items: flex-start;
}

.budget-section-header--compact {
  margin-bottom: 1.25rem;
}

.budget-section-title,
.budget-subsection-title,
.budget-toggle__title {
  font-weight: 600;
}

.budget-section-title {
  font-size: 1.25rem;
}

.budget-section-copy {
  margin-top: 0.25rem;
}

.budget-status-pill {
  display: inline-flex;
  align-items: center;
  min-height: 36px;
  border: 1px solid var(--color-border-subtle);
  border-radius: 9999px;
  padding: 0.25rem 0.75rem;
  font-size: 0.875rem;
  font-weight: 500;
}

.budget-status-pill--on {
  border-color: rgb(var(--color-success-rgb) / 30%);
  background: rgb(var(--color-success-rgb) / 10%);
  color: var(--color-success);
}

.budget-status-pill--off {
  background: var(--color-bg-surface);
  color: var(--color-text-secondary);
}

.budget-overview-grid,
.budget-limit-grid,
.budget-input-grid {
  display: grid;
  grid-template-columns: repeat(1, minmax(0, 1fr));
  gap: 1rem;
}

.budget-overview-grid {
  margin-bottom: 1.5rem;
}

.budget-overview-card,
.budget-limit-row,
.budget-toggle-card {
  border: 1px solid var(--color-border-default);
  border-radius: 1rem;
  background: var(--color-bg-surface);
}

.budget-overview-card {
  padding: 1rem;
}

.budget-overview-card__value {
  margin-top: 0.5rem;
  font-size: 1.5rem;
  font-weight: 700;
}

.budget-limits {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.budget-subsection-title {
  font-size: 1.125rem;
}

.budget-limit-row {
  gap: 0.75rem;
  padding: 1rem;
}

.budget-limit-row__label {
  font-weight: 500;
}

.budget-limit-row__value {
  font-size: 0.875rem;
  font-weight: 600;
}

.budget-warning-card {
  border: 1px solid rgb(var(--color-danger-rgb) / 25%);
  border-radius: 0.75rem;
  background: rgb(var(--color-danger-rgb) / 10%);
  padding: 1rem;
}

.budget-warning-card__text {
  font-size: 0.875rem;
  line-height: 1.75;
  color: var(--color-danger);
}

.budget-toggle-card {
  padding: 1rem;
  background: var(--color-bg-surface);
}

.budget-toggle {
  cursor: pointer;
  gap: 0.75rem;
}

.budget-checkbox {
  width: 1rem;
  height: 1rem;
  border-radius: 0.25rem;
}

.budget-toggle__copy {
  margin-top: 0.125rem;
}

.budget-label {
  display: block;
  font-weight: 500;
}

.budget-input {
  display: block;
  width: 100%;
  margin-top: 0.5rem;
  padding: 0.625rem 1rem;
  border: 1px solid var(--color-border-subtle);
  border-radius: 0.625rem;
  background: var(--color-bg-surface);
  color: var(--color-text-primary);
}

.budget-input::placeholder {
  color: var(--color-text-muted);
}

.budget-input:focus {
  outline: 2px solid var(--color-accent-primary);
  outline-offset: 2px;
}

.budget-form-actions {
  flex-direction: column;
  gap: 0.75rem;
}

@media (width >= 640px) {
  .budget-shell {
    padding: 1.5rem;
  }

  .budget-shell__header,
  .budget-section-header {
    flex-direction: row;
  }

  .budget-shell__header {
    align-items: flex-start;
  }

  .budget-section-header {
    align-items: center;
  }

  .budget-form-actions {
    flex-flow: row wrap;
  }
}

@media (width >= 768px) {
  .budget-overview-grid,
  .budget-limit-grid,
  .budget-input-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}
</style>
