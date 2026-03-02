<template>
  <div class="budget-view p-6 space-y-6">
    <!-- 页面标题 -->
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">
          💰 预算管理
        </h1>
        <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
          管理成本预算限制和警告阈值
        </p>
      </div>
      <button
        :disabled="loading"
        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg flex items-center space-x-2 disabled:opacity-50"
        @click="loadData"
      >
        <svg
          class="w-5 h-5"
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

    <!-- 加载状态 -->
    <div
      v-if="loading"
      class="flex items-center justify-center py-12"
    >
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600" />
    </div>

    <!-- 错误提示 -->
    <div
      v-if="error"
      class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4"
    >
      <div class="flex">
        <svg
          class="h-5 w-5 text-red-400"
          fill="currentColor"
          viewBox="0 0 20 20"
        >
          <path
            fill-rule="evenodd"
            d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
            clip-rule="evenodd"
          />
        </svg>
        <div class="ml-3">
          <h3 class="text-sm font-medium text-red-800 dark:text-red-200">
            加载失败
          </h3>
          <p class="mt-2 text-sm text-red-700 dark:text-red-300">
            {{ error }}
          </p>
        </div>
      </div>
    </div>

    <!-- 预算内容 -->
    <div
      v-if="!loading && !error && budgetStatus"
      class="space-y-6"
    >
      <!-- 状态卡片 -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
            预算状态
          </h2>
          <div class="flex items-center space-x-2">
            <span
              class="px-3 py-1 rounded-full text-sm font-medium"
              :class="budgetStatus.enabled ? 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400' : 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-400'"
            >
              {{ budgetStatus.enabled ? '✅ 已启用' : '❌ 已禁用' }}
            </span>
          </div>
        </div>

        <!-- 当前成本 -->
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
          <div class="bg-gray-50 dark:bg-gray-700/50 rounded-lg p-4">
            <p class="text-sm text-gray-600 dark:text-gray-400">
              今日成本
            </p>
            <p class="mt-2 text-2xl font-bold text-gray-900 dark:text-white">
              ${{ budgetStatus.current_costs.today.toFixed(4) }}
            </p>
          </div>
          <div class="bg-gray-50 dark:bg-gray-700/50 rounded-lg p-4">
            <p class="text-sm text-gray-600 dark:text-gray-400">
              本周成本
            </p>
            <p class="mt-2 text-2xl font-bold text-gray-900 dark:text-white">
              ${{ budgetStatus.current_costs.this_week.toFixed(4) }}
            </p>
          </div>
          <div class="bg-gray-50 dark:bg-gray-700/50 rounded-lg p-4">
            <p class="text-sm text-gray-600 dark:text-gray-400">
              本月成本
            </p>
            <p class="mt-2 text-2xl font-bold text-gray-900 dark:text-white">
              ${{ budgetStatus.current_costs.this_month.toFixed(4) }}
            </p>
          </div>
        </div>

        <!-- 预算限制 -->
        <div class="space-y-4">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
            预算限制
          </h3>
          <div class="space-y-3">
            <div
              v-for="(limit, period) in budgetLimits"
              :key="period"
              class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg"
            >
              <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                {{ period }}
              </span>
              <span class="text-sm text-gray-900 dark:text-white font-semibold">
                {{ (limit !== null && limit !== undefined) ? `$${limit.toFixed(2)}` : '无限制' }}
              </span>
            </div>
          </div>
        </div>

        <!-- 警告 -->
        <div
          v-if="budgetStatus.warnings.length > 0"
          class="mt-6 space-y-3"
        >
          <h3 class="text-lg font-semibold text-red-600 dark:text-red-400">
            ⚠️ 预算警告
          </h3>
          <div
            v-for="(warning, index) in budgetStatus.warnings"
            :key="index"
            class="p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg"
          >
            <p class="text-sm text-red-800 dark:text-red-300">
              <strong>{{ warning.period }}</strong>:
              当前成本 ${{ warning.current_cost.toFixed(2) }}
              / 限制 ${{ warning.limit.toFixed(2) }}
              ({{ warning.usage_percent.toFixed(1) }}%)
            </p>
          </div>
        </div>
      </div>

      <!-- 配置面板 -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
          配置预算
        </h2>
        <form
          class="space-y-4"
          @submit.prevent="saveBudget"
        >
          <!-- 启用/禁用 -->
          <div class="flex items-center">
            <input
              id="enabled"
              v-model="form.enabled"
              type="checkbox"
              class="w-4 h-4 text-blue-600 border-gray-300 rounded"
            >
            <label
              for="enabled"
              class="ml-2 text-sm font-medium text-gray-700 dark:text-gray-300"
            >
              启用预算控制
            </label>
          </div>

          <!-- 预算限制输入 -->
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                每日限制 ($)
              </label>
              <input
                v-model.number="form.daily_limit"
                type="number"
                step="0.01"
                min="0"
                class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                placeholder="留空表示无限制"
              >
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                每周限制 ($)
              </label>
              <input
                v-model.number="form.weekly_limit"
                type="number"
                step="0.01"
                min="0"
                class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                placeholder="留空表示无限制"
              >
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                每月限制 ($)
              </label>
              <input
                v-model.number="form.monthly_limit"
                type="number"
                step="0.01"
                min="0"
                class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                placeholder="留空表示无限制"
              >
            </div>
          </div>

          <!-- 警告阈值 -->
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              警告阈值 (%)
            </label>
            <input
              v-model.number="form.warn_threshold"
              type="number"
              min="0"
              max="100"
              class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            >
          </div>

          <!-- 按钮 -->
          <div class="flex items-center space-x-4">
            <button
              type="submit"
              :disabled="saving"
              class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg disabled:opacity-50"
            >
              {{ saving ? '保存中...' : '保存配置' }}
            </button>
            <button
              type="button"
              :disabled="saving"
              class="px-6 py-2 bg-gray-500 hover:bg-gray-600 text-white rounded-lg disabled:opacity-50"
              @click="handleReset"
            >
              重置所有限制
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/* eslint-disable no-console -- Development debugging, console output acceptable */
import { ref, computed, onMounted } from 'vue'
import { getBudgetStatus, setBudget, resetBudget } from '@/api/modules/stats'
import type { BudgetStatus, SetBudgetRequest } from '@/types'

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
    budgetStatus.value = await getBudgetStatus()

    // 更新表单
    form.value.enabled = budgetStatus.value.enabled
    form.value.daily_limit = budgetStatus.value.daily_limit
    form.value.weekly_limit = budgetStatus.value.weekly_limit
    form.value.monthly_limit = budgetStatus.value.monthly_limit
    form.value.warn_threshold = budgetStatus.value.warn_threshold
  } catch (e) {
    error.value = (e instanceof Error ? e.message : "Error") || '加载失败'
    console.error('Failed to load budget:', e)
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
  } catch (e) {
    alert('保存失败: ' + ((e instanceof Error ? e.message : "Error") || '未知错误'))
    console.error('Failed to save budget:', e)
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
  } catch (e) {
    alert('重置失败: ' + ((e instanceof Error ? e.message : "Error") || '未知错误'))
    console.error('Failed to reset budget:', e)
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  loadData()
})
</script>
