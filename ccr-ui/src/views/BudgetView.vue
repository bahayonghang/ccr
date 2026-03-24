<template>
  <div class="budget-view">
    <div class="budget-shell glass-effect">
      <div class="budget-shell__header">
        <div class="budget-shell__intro">
          <div class="budget-shell__title-row">
            <div class="budget-shell__icon text-accent-primary">
              <svg
                class="budget-shell__icon-svg"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
            </div>
            <div>
              <h1 class="budget-title text-text-primary">
                预算管理
              </h1>
              <p class="budget-subtitle text-text-secondary">
                管理成本预算限制和警告阈值
              </p>
            </div>
          </div>
        </div>

        <button
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
          <span>刷新</span>
        </button>
      </div>
    </div>

    <div
      v-if="loading"
      class="budget-shell budget-shell--loading glass-effect"
      aria-live="polite"
    >
      <div class="budget-loading">
        <div class="budget-loading__spinner animate-spin" />
        <p class="budget-loading__text text-text-secondary">
          正在加载预算数据...
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
            加载失败
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
      <section class="budget-shell glass-effect">
        <div class="budget-section-header">
          <div>
            <h2 class="budget-section-title text-text-primary">
              预算状态
            </h2>
            <p class="budget-section-copy text-text-secondary">
              当前预算开关与各周期成本总览
            </p>
          </div>

          <span
            class="budget-status-pill"
            :class="budgetStatus.enabled ? 'border-accent-success/30 bg-accent-success/10 text-accent-success' : 'border-border-default bg-bg-surface text-text-secondary'"
          >
            {{ budgetStatus.enabled ? '已启用' : '已禁用' }}
          </span>
        </div>

        <div class="budget-overview-grid">
          <div class="budget-overview-card">
            <p class="budget-overview-card__label text-text-secondary">
              今日成本
            </p>
            <p class="budget-overview-card__value text-text-primary">
              ${{ budgetStatus.current_costs.today.toFixed(4) }}
            </p>
          </div>
          <div class="budget-overview-card">
            <p class="budget-overview-card__label text-text-secondary">
              本周成本
            </p>
            <p class="budget-overview-card__value text-text-primary">
              ${{ budgetStatus.current_costs.this_week.toFixed(4) }}
            </p>
          </div>
          <div class="budget-overview-card">
            <p class="budget-overview-card__label text-text-secondary">
              本月成本
            </p>
            <p class="budget-overview-card__value text-text-primary">
              ${{ budgetStatus.current_costs.this_month.toFixed(4) }}
            </p>
          </div>
        </div>

        <div class="budget-limits">
          <h3 class="budget-subsection-title text-text-primary">
            预算限制
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
                {{ (limit !== null && limit !== undefined) ? `$${limit.toFixed(2)}` : '无限制' }}
              </span>
            </div>
          </div>
        </div>

        <div
          v-if="budgetStatus.warnings.length > 0"
          class="budget-warning-group"
        >
          <h3 class="budget-subsection-title text-accent-danger">
            预算警告
          </h3>
          <div
            v-for="(warning, index) in budgetStatus.warnings"
            :key="index"
            class="budget-warning-card"
          >
            <p class="budget-warning-card__text">
              <strong class="budget-warning-card__period">{{ warning.period }}</strong>:
              当前成本 ${{ warning.current_cost.toFixed(2) }}
              / 限制 ${{ warning.limit.toFixed(2) }}
              ({{ warning.usage_percent.toFixed(1) }}%)
            </p>
          </div>
        </div>
      </section>

      <section class="budget-shell glass-effect">
        <div class="budget-section-header budget-section-header--compact">
          <div>
            <h2 class="budget-section-title text-text-primary">
              配置预算
            </h2>
            <p class="budget-section-copy text-text-secondary">
              调整预算开关、上限以及告警阈值
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
                  启用预算控制
                </p>
                <p class="budget-toggle__copy text-text-secondary">
                  开启后将根据下方限制进行预算提醒
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
                每日限制 ($)
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
                每周限制 ($)
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
                每月限制 ($)
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
              警告阈值 (%)
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
              {{ saving ? '保存中...' : '保存配置' }}
            </button>
            <button
              type="button"
              :disabled="saving"
              class="budget-secondary-button"
              @click="handleReset"
            >
              重置所有限制
            </button>
          </div>
        </form>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { getBudgetStatus, setBudget, resetBudget } from '@/api'
import { useUIStore } from '@/stores/ui'
import type { BudgetStatus, SetBudgetRequest } from '@/types'
import { getErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'

const uiStore = useUIStore()
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
    '每日': budgetStatus.value.daily_limit,
    '每周': budgetStatus.value.weekly_limit,
    '每月': budgetStatus.value.monthly_limit,
  }
})

const loadData = async () => {
  loading.value = true
  error.value = null

  try {
    const status = await getBudgetStatus<BudgetStatus>()
    budgetStatus.value = status

    form.value.enabled = status.enabled
    form.value.daily_limit = status.daily_limit
    form.value.weekly_limit = status.weekly_limit
    form.value.monthly_limit = status.monthly_limit
    form.value.warn_threshold = status.warn_threshold
  } catch (e: unknown) {
    error.value = getErrorMessage(e) || '加载失败'
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

    uiStore.showSuccess('配置已保存')
  } catch (e: unknown) {
    uiStore.showError(`保存失败: ${getErrorMessage(e) || '未知错误'}`)
    logger.error('Failed to save budget:', e)
  } finally {
    saving.value = false
  }
}

const handleReset = async () => {
  const confirmed = await uiStore.requestConfirm({
    title: '重置预算限制',
    message: '确定要重置所有预算限制吗？',
    confirmText: '重置',
    cancelText: '取消',
    type: 'danger'
  })
  if (!confirmed) return

  saving.value = true

  try {
    await resetBudget()
    await loadData()

    uiStore.showSuccess('预算限制已重置')
  } catch (e: unknown) {
    uiStore.showError(`重置失败: ${getErrorMessage(e) || '未知错误'}`)
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
  min-height: 100%;
  gap: 1.5rem;
  padding: 1rem;
}

.budget-content,
.budget-form,
.budget-warning-group {
  gap: 1.5rem;
}

.budget-shell {
  border: 1px solid rgb(255 255 255 / 20%);
  border-radius: 1.5rem;
  padding: 1.25rem;
  box-shadow: var(--shadow-small);
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
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.75rem;
  height: 2.75rem;
  border: 1px solid rgb(var(--color-accent-primary-rgb), 0.2);
  border-radius: 1rem;
  background: linear-gradient(135deg, rgb(139 92 246 / 20%), rgb(217 70 239 / 20%));
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
  background: linear-gradient(90deg, rgb(139 92 246), rgb(147 51 234));
  color: white;
  font-size: 0.875rem;
  font-weight: 600;
  box-shadow: 0 16px 30px rgb(139 92 246 / 25%);
}

.budget-primary-button:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 18px 34px rgb(139 92 246 / 35%);
}

.budget-primary-button--wide {
  padding-inline: 1.5rem;
}

.budget-secondary-button {
  border: 1px solid var(--border-border-default, var(--border-color));
  background: var(--bg-surface);
  color: var(--text-secondary);
  padding-inline: 1.5rem;
  font-weight: 500;
}

.budget-secondary-button:hover:not(:disabled) {
  background: var(--bg-elevated);
  color: var(--text-primary);
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
  border: 4px solid rgb(var(--color-accent-primary-rgb), 0.15);
  border-top-color: var(--accent-primary);
  border-radius: 9999px;
}

.budget-error {
  border: 1px solid rgb(239 68 68 / 30%);
  border-radius: 1rem;
  background: rgb(239 68 68 / 10%);
  padding: 1rem;
  color: rgb(254 226 226);
  backdrop-filter: blur(12px);
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
  color: rgb(252 165 165);
}

.budget-error__title {
  font-size: 0.875rem;
  font-weight: 600;
  color: rgb(254 202 202);
}

.budget-error__message {
  margin-top: 0.25rem;
  font-size: 0.875rem;
  color: rgb(254 226 226 / 90%);
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
  border-width: 1px;
  border-radius: 9999px;
  padding: 0.25rem 0.75rem;
  font-size: 0.875rem;
  font-weight: 500;
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
  border: 1px solid rgb(255 255 255 / 10%);
  border-radius: 1rem;
  background: rgb(var(--color-bg-surface-rgb, 255 255 255), 0.5);
}

.budget-overview-card {
  padding: 1rem;
  backdrop-filter: blur(8px);
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
  border: 1px solid rgb(239 68 68 / 25%);
  border-radius: 1rem;
  background: rgb(239 68 68 / 10%);
  padding: 1rem;
}

.budget-warning-card__text {
  font-size: 0.875rem;
  line-height: 1.75;
  color: rgb(254 226 226 / 90%);
}

.budget-warning-card__period {
  color: rgb(254 226 226);
}

.budget-toggle-card {
  padding: 1rem;
  background: rgb(var(--color-bg-surface-rgb, 255 255 255), 0.4);
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
  border: 1px solid var(--border-border-default, var(--border-color));
  border-radius: 0.75rem;
  background: var(--bg-surface);
  color: var(--text-primary);
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}

.budget-input::placeholder {
  color: var(--text-muted);
}

.budget-input:focus {
  outline: none;
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 2px rgb(var(--color-accent-primary-rgb), 0.2);
}

.budget-form-actions {
  flex-direction: column;
  gap: 0.75rem;
}

@media (width >= 640px) {
  .budget-view {
    padding: 1.5rem;
  }

  .budget-shell {
    padding: 1.5rem;
  }

  .budget-title {
    font-size: 1.875rem;
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
