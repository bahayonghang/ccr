<template>
  <div class="budget-view min-h-screen space-y-6 p-4 sm:p-6">
    <div class="glass-effect rounded-3xl border border-white/20 p-5 shadow-sm sm:p-6">
      <div class="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div class="space-y-2">
          <div class="flex items-center gap-3">
            <div class="flex h-11 w-11 items-center justify-center rounded-2xl bg-gradient-to-br from-violet-500/20 to-fuchsia-500/20 text-accent-primary border border-accent-primary/20">
              <svg
                class="h-5 w-5"
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
              <h1 class="text-2xl font-bold text-text-primary sm:text-3xl">
                预算管理
              </h1>
              <p class="text-sm text-text-secondary">
                管理成本预算限制和警告阈值
              </p>
            </div>
          </div>
        </div>

        <button
          :disabled="loading"
          class="inline-flex min-h-[44px] items-center justify-center gap-2 rounded-xl bg-gradient-to-r from-violet-500 to-purple-600 px-4 py-2.5 text-sm font-semibold text-white shadow-lg shadow-violet-500/25 transition-[color,background-color,border-color,transform] hover:-translate-y-0.5 hover:shadow-violet-500/35 focus:outline-none focus:ring-2 focus:ring-accent-primary/30 disabled:cursor-not-allowed disabled:opacity-50"
          @click="loadData"
        >
          <svg
            class="h-4 w-4"
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
      class="glass-effect flex items-center justify-center rounded-3xl border border-white/20 py-16"
      aria-live="polite"
    >
      <div class="flex flex-col items-center gap-4">
        <div class="h-12 w-12 animate-spin rounded-full border-4 border-accent-primary/15 border-t-accent-primary" />
        <p class="text-sm text-text-secondary">
          正在加载预算数据...
        </p>
      </div>
    </div>

    <div
      v-if="error"
      class="rounded-2xl border border-red-500/30 bg-red-500/10 p-4 text-red-100 backdrop-blur-md"
      role="alert"
    >
      <div class="flex items-start gap-3">
        <svg
          class="mt-0.5 h-5 w-5 flex-none text-red-300"
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
          <h2 class="text-sm font-semibold text-red-200">
            加载失败
          </h2>
          <p class="mt-1 text-sm text-red-100/90">
            {{ error }}
          </p>
        </div>
      </div>
    </div>

    <div
      v-if="!loading && !error && budgetStatus"
      class="space-y-6"
    >
      <section class="glass-effect rounded-3xl border border-white/20 p-5 shadow-sm sm:p-6">
        <div class="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 class="text-xl font-semibold text-text-primary">
              预算状态
            </h2>
            <p class="mt-1 text-sm text-text-secondary">
              当前预算开关与各周期成本总览
            </p>
          </div>

          <span
            class="inline-flex min-h-[36px] items-center rounded-full border px-3 py-1 text-sm font-medium"
            :class="budgetStatus.enabled ? 'border-accent-success/30 bg-accent-success/10 text-accent-success' : 'border-border-default bg-bg-surface text-text-secondary'"
          >
            {{ budgetStatus.enabled ? '已启用' : '已禁用' }}
          </span>
        </div>

        <div class="mb-6 grid grid-cols-1 gap-4 md:grid-cols-3">
          <div class="rounded-2xl border border-white/10 bg-bg-surface/60 p-4 backdrop-blur-sm">
            <p class="text-sm text-text-secondary">
              今日成本
            </p>
            <p class="mt-2 text-2xl font-bold text-text-primary">
              ${{ budgetStatus.current_costs.today.toFixed(4) }}
            </p>
          </div>
          <div class="rounded-2xl border border-white/10 bg-bg-surface/60 p-4 backdrop-blur-sm">
            <p class="text-sm text-text-secondary">
              本周成本
            </p>
            <p class="mt-2 text-2xl font-bold text-text-primary">
              ${{ budgetStatus.current_costs.this_week.toFixed(4) }}
            </p>
          </div>
          <div class="rounded-2xl border border-white/10 bg-bg-surface/60 p-4 backdrop-blur-sm">
            <p class="text-sm text-text-secondary">
              本月成本
            </p>
            <p class="mt-2 text-2xl font-bold text-text-primary">
              ${{ budgetStatus.current_costs.this_month.toFixed(4) }}
            </p>
          </div>
        </div>

        <div class="space-y-4">
          <h3 class="text-lg font-semibold text-text-primary">
            预算限制
          </h3>
          <div class="grid grid-cols-1 gap-3 md:grid-cols-3">
            <div
              v-for="(limit, period) in budgetLimits"
              :key="period"
              class="flex items-center justify-between gap-3 rounded-2xl border border-white/10 bg-bg-surface/50 p-4"
            >
              <span class="text-sm font-medium text-text-secondary">
                {{ period }}
              </span>
              <span class="text-sm font-semibold text-text-primary">
                {{ (limit !== null && limit !== undefined) ? `$${limit.toFixed(2)}` : '无限制' }}
              </span>
            </div>
          </div>
        </div>

        <div
          v-if="budgetStatus.warnings.length > 0"
          class="mt-6 space-y-3"
        >
          <h3 class="text-lg font-semibold text-accent-danger">
            预算警告
          </h3>
          <div
            v-for="(warning, index) in budgetStatus.warnings"
            :key="index"
            class="rounded-2xl border border-red-500/25 bg-red-500/10 p-4"
          >
            <p class="text-sm leading-6 text-red-100/90">
              <strong class="text-red-100">{{ warning.period }}</strong>:
              当前成本 ${{ warning.current_cost.toFixed(2) }}
              / 限制 ${{ warning.limit.toFixed(2) }}
              ({{ warning.usage_percent.toFixed(1) }}%)
            </p>
          </div>
        </div>
      </section>

      <section class="glass-effect rounded-3xl border border-white/20 p-5 shadow-sm sm:p-6">
        <div class="mb-5">
          <h2 class="text-xl font-semibold text-text-primary">
            配置预算
          </h2>
          <p class="mt-1 text-sm text-text-secondary">
            调整预算开关、上限以及告警阈值
          </p>
        </div>

        <form
          class="space-y-5"
          @submit.prevent="saveBudget"
        >
          <div class="rounded-2xl border border-white/10 bg-bg-surface/40 p-4">
            <label
              for="enabled"
              class="flex cursor-pointer items-center gap-3"
            >
              <input
                id="enabled"
                v-model="form.enabled"
                type="checkbox"
                class="h-4 w-4 rounded border-border-default bg-bg-surface text-accent-primary focus:ring-2 focus:ring-accent-primary/30"
              >
              <div>
                <p class="text-sm font-medium text-text-primary">
                  启用预算控制
                </p>
                <p class="text-xs text-text-secondary">
                  开启后将根据下方限制进行预算提醒
                </p>
              </div>
            </label>
          </div>

          <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
            <div>
              <label
                for="daily_limit"
                class="block text-sm font-medium text-text-secondary"
              >
                每日限制 ($)
              </label>
              <input
                id="daily_limit"
                v-model.number="form.daily_limit"
                type="number"
                step="0.01"
                min="0"
                class="mt-2 block w-full rounded-xl border border-border-default bg-bg-surface px-4 py-2.5 text-text-primary transition-[border-color,box-shadow] placeholder:text-text-muted focus:border-accent-primary focus:outline-none focus:ring-2 focus:ring-accent-primary/20"
                placeholder="留空表示无限制"
              >
            </div>

            <div>
              <label
                for="weekly_limit"
                class="block text-sm font-medium text-text-secondary"
              >
                每周限制 ($)
              </label>
              <input
                id="weekly_limit"
                v-model.number="form.weekly_limit"
                type="number"
                step="0.01"
                min="0"
                class="mt-2 block w-full rounded-xl border border-border-default bg-bg-surface px-4 py-2.5 text-text-primary transition-[border-color,box-shadow] placeholder:text-text-muted focus:border-accent-primary focus:outline-none focus:ring-2 focus:ring-accent-primary/20"
                placeholder="留空表示无限制"
              >
            </div>

            <div>
              <label
                for="monthly_limit"
                class="block text-sm font-medium text-text-secondary"
              >
                每月限制 ($)
              </label>
              <input
                id="monthly_limit"
                v-model.number="form.monthly_limit"
                type="number"
                step="0.01"
                min="0"
                class="mt-2 block w-full rounded-xl border border-border-default bg-bg-surface px-4 py-2.5 text-text-primary transition-[border-color,box-shadow] placeholder:text-text-muted focus:border-accent-primary focus:outline-none focus:ring-2 focus:ring-accent-primary/20"
                placeholder="留空表示无限制"
              >
            </div>
          </div>

          <div>
            <label
              for="warn_threshold"
              class="block text-sm font-medium text-text-secondary"
            >
              警告阈值 (%)
            </label>
            <input
              id="warn_threshold"
              v-model.number="form.warn_threshold"
              type="number"
              min="0"
              max="100"
              class="mt-2 block w-full rounded-xl border border-border-default bg-bg-surface px-4 py-2.5 text-text-primary transition-[border-color,box-shadow] focus:border-accent-primary focus:outline-none focus:ring-2 focus:ring-accent-primary/20"
            >
          </div>

          <div class="flex flex-col gap-3 sm:flex-row sm:flex-wrap">
            <button
              type="submit"
              :disabled="saving"
              class="inline-flex min-h-[44px] items-center justify-center rounded-xl bg-gradient-to-r from-violet-500 to-purple-600 px-6 py-2.5 font-semibold text-white shadow-lg shadow-violet-500/25 transition-[color,background-color,border-color,transform] hover:-translate-y-0.5 hover:shadow-violet-500/35 focus:outline-none focus:ring-2 focus:ring-accent-primary/30 disabled:cursor-not-allowed disabled:opacity-50"
            >
              {{ saving ? '保存中...' : '保存配置' }}
            </button>
            <button
              type="button"
              :disabled="saving"
              class="inline-flex min-h-[44px] items-center justify-center rounded-xl border border-border-default bg-bg-surface px-6 py-2.5 font-medium text-text-secondary transition-colors hover:bg-bg-elevated hover:text-text-primary focus:outline-none focus:ring-2 focus:ring-accent-primary/20 disabled:cursor-not-allowed disabled:opacity-50"
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
import type { BudgetStatus, SetBudgetRequest } from '@/types'
import { getErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'

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

    alert('配置已保存')
  } catch (e: unknown) {
    alert(`保存失败: ${getErrorMessage(e) || '未知错误'}`)
    logger.error('Failed to save budget:', e)
  } finally {
    saving.value = false
  }
}

const handleReset = async () => {
  if (!confirm('确定要重置所有预算限制吗？')) return

  saving.value = true

  try {
    await resetBudget()
    await loadData()

    alert('预算限制已重置')
  } catch (e: unknown) {
    alert(`重置失败: ${getErrorMessage(e) || '未知错误'}`)
    logger.error('Failed to reset budget:', e)
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  loadData()
})
</script>
