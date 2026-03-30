<template>
  <div class="provider-health-view space-y-6">
    <!-- 页面标题 -->
    <div class="provider-health-header flex items-center justify-between gap-4">
      <div>
        <h1 class="text-3xl font-bold text-text-primary">
          🏥 Provider 健康检查
        </h1>
        <p class="mt-2 text-sm text-text-secondary">
          检测 API 端点连通性和 Key 有效性
        </p>
      </div>
      <div class="provider-health-toolbar flex items-center gap-3">
        <!-- 测试所有按钮 -->
        <button
          :disabled="testing"
          class="provider-health-button provider-health-button--primary"
          @click="testAll"
        >
          <svg
            class="w-5 h-5"
            :class="{ 'animate-spin': testing }"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <span>{{ testing ? '检测中...' : '检测所有' }}</span>
        </button>
      </div>
    </div>

    <!-- 健康状态摘要 -->
    <div
      v-if="summary"
      class="grid grid-cols-1 gap-6 md:grid-cols-4"
    >
      <div class="provider-health-summary-card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-text-secondary">
              总计
            </p>
            <p class="mt-2 text-3xl font-bold text-text-primary">
              {{ summary.total }}
            </p>
          </div>
          <div class="provider-health-summary-icon">
            <svg
              class="w-8 h-8 text-text-secondary"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
              />
            </svg>
          </div>
        </div>
      </div>
      <div class="provider-health-summary-card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-text-secondary">
              健康
            </p>
            <p class="mt-2 text-3xl font-bold text-green-600 dark:text-green-400">
              {{ summary.healthy }}
            </p>
          </div>
          <div class="provider-health-summary-icon provider-health-summary-icon--healthy">
            <svg
              class="w-8 h-8 text-green-600 dark:text-green-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          </div>
        </div>
      </div>
      <div class="provider-health-summary-card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-text-secondary">
              降级
            </p>
            <p class="mt-2 text-3xl font-bold text-yellow-600 dark:text-yellow-400">
              {{ summary.degraded }}
            </p>
          </div>
          <div class="provider-health-summary-icon provider-health-summary-icon--degraded">
            <svg
              class="w-8 h-8 text-yellow-600 dark:text-yellow-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
              />
            </svg>
          </div>
        </div>
      </div>
      <div class="provider-health-summary-card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-text-secondary">
              不可用
            </p>
            <p class="mt-2 text-3xl font-bold text-red-600 dark:text-red-400">
              {{ summary.unhealthy }}
            </p>
          </div>
          <div class="provider-health-summary-icon provider-health-summary-icon--unhealthy">
            <svg
              class="w-8 h-8 text-red-600 dark:text-red-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          </div>
        </div>
      </div>
    </div>

    <!-- 加载状态 -->
    <div
      v-if="testing && results.length === 0"
      class="flex items-center justify-center py-12"
    >
      <div class="text-center space-y-4">
        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto" />
        <p class="text-text-secondary">
          正在检测所有 Provider...
        </p>
      </div>
    </div>

    <!-- 检测结果列表 -->
    <div
      v-if="results.length > 0"
      class="provider-health-table-shell"
    >
      <table class="min-w-full">
        <thead class="provider-health-table-head">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider">
              状态
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider">
              名称
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider">
              端点
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider">
              延迟
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider">
              可用模型
            </th>
          </tr>
        </thead>
        <tbody class="provider-health-table-body">
          <tr
            v-for="result in results"
            :key="result.provider_name"
            class="provider-health-table-row"
          >
            <td class="px-6 py-4 whitespace-nowrap">
              <span
                class="px-2 py-1 inline-flex text-xs leading-5 font-semibold rounded-full"
                :class="getStatusClass(result.status)"
              >
                {{ getStatusIcon(result.status) }} {{ getStatusText(result.status) }}
              </span>
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
              <div class="text-sm font-medium text-text-primary">
                {{ result.provider_name }}
              </div>
            </td>
            <td class="px-6 py-4">
              <div
                class="text-sm text-text-muted truncate max-w-xs"
                :title="result.base_url"
              >
                {{ result.base_url || '-' }}
              </div>
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
              <span
                v-if="result.latency_ms"
                class="text-sm text-text-primary"
              >
                {{ result.latency_ms }}ms
              </span>
              <span
                v-else
                class="text-sm text-text-muted"
              >-</span>
            </td>
            <td class="px-6 py-4">
              <div
                v-if="result.available_models && result.available_models.length > 0"
                class="flex flex-wrap gap-1"
              >
                <span
                  v-for="model in result.available_models.slice(0, 3)"
                  :key="model"
                  class="provider-health-model-chip"
                >
                  {{ shortenModelName(model) }}
                </span>
                <span
                  v-if="result.available_models.length > 3"
                  class="text-xs text-text-muted"
                >
                  +{{ result.available_models.length - 3 }}
                </span>
              </div>
              <span
                v-else
                class="text-sm text-text-muted"
              >-</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 错误详情 -->
    <div
      v-if="errorResults.length > 0"
      class="provider-health-errors"
    >
      <h3 class="text-lg font-bold text-red-800 dark:text-red-200 mb-4">
        错误详情
      </h3>
      <div class="space-y-3">
        <div
          v-for="result in errorResults"
          :key="result.provider_name"
          class="provider-health-error-card"
        >
          <p class="font-medium text-text-primary">
            {{ result.provider_name }}
          </p>
          <p class="text-sm text-red-600 dark:text-red-400 mt-1">
            {{ result.error }}
          </p>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div
      v-if="!testing && results.length === 0"
      class="provider-health-empty"
    >
      <svg
        class="mx-auto h-12 w-12 text-text-muted"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
        />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-text-primary">
        尚未检测
      </h3>
      <p class="mt-1 text-sm text-text-muted">
        点击"检测所有"按钮开始健康检查
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { healthCheck } from '@/api/runtime/system'
import { logger } from '@/utils/logger'

interface HealthCheckResult {
  provider_name: string
  base_url: string
  status: string
  status_color: string
  latency_ms: number | null
  error: string | null
  model_available: boolean
  available_models: string[]
}

interface HealthSummary {
  total: number
  healthy: number
  degraded: number
  unhealthy: number
}

const results = ref<HealthCheckResult[]>([])
const summary = ref<HealthSummary | null>(null)
const testing = ref(false)

const testAll = async () => {
  testing.value = true
  results.value = []
  summary.value = null

  try {
    const data = await healthCheck<{
      status?: string
      database?: boolean
      version?: string
    }>()

    const status = data?.status === 'healthy' ? 'healthy' : 'degraded'
    results.value = [{
      provider_name: 'system',
      base_url: 'tauri://health_check',
      status,
      status_color: status === 'healthy' ? 'green' : 'yellow',
      latency_ms: null,
      error: data?.database ? null : 'Database unavailable',
      model_available: false,
      available_models: [],
    }]
    summary.value = {
      total: 1,
      healthy: status === 'healthy' ? 1 : 0,
      degraded: status === 'degraded' ? 1 : 0,
      unhealthy: 0,
    }
  } catch (e) {
    logger.error('Failed to test providers:', e)
    results.value = [{
      provider_name: 'system',
      base_url: 'tauri://health_check',
      status: 'unhealthy',
      status_color: 'red',
      latency_ms: null,
      error: e instanceof Error ? e.message : 'Unknown error',
      model_available: false,
      available_models: [],
    }]
    summary.value = {
      total: 1,
      healthy: 0,
      degraded: 0,
      unhealthy: 1,
    }
  } finally {
    testing.value = false
  }
}

const errorResults = computed(() => {
  return results.value.filter(r => r.error)
})

const getStatusClass = (status: string): string => {
  switch (status) {
    case 'healthy':
      return 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300'
    case 'degraded':
      return 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-300'
    case 'unhealthy':
      return 'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300'
    default:
      return 'bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-300'
  }
}

const getStatusIcon = (status: string): string => {
  switch (status) {
    case 'healthy':
      return '✓'
    case 'degraded':
      return '⚠'
    case 'unhealthy':
      return '✗'
    default:
      return '?'
  }
}

const getStatusText = (status: string): string => {
  switch (status) {
    case 'healthy':
      return '健康'
    case 'degraded':
      return '降级'
    case 'unhealthy':
      return '不可用'
    default:
      return '未知'
  }
}

const shortenModelName = (model: string): string => {
  return model.replace('claude-', '').replace('gpt-', '').replace('-20241022', '')
}
</script>

<style scoped>
.provider-health-view {
  min-height: calc(100vh - 64px);
  padding: 1rem;
}

.provider-health-header {
  align-items: flex-start;
}

.provider-health-toolbar {
  flex-wrap: wrap;
}

.provider-health-button {
  min-height: 44px;
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.625rem 1rem;
  border-radius: 0.875rem;
  font-weight: 600;
  transition:
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease),
    transform var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.provider-health-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.provider-health-button--primary {
  color: var(--color-text-inverted);
  background: linear-gradient(135deg, rgb(var(--color-accent-primary-rgb) / 96%), rgb(var(--color-accent-secondary-rgb) / 84%));
  box-shadow: var(--elevation-2);
}

.provider-health-button--primary:hover:not(:disabled) {
  transform: translateY(-1px);
}

.provider-health-summary-card {
  border-radius: 1rem;
  padding: 1.5rem;
  background: var(--surface-card-bg);
  border: 1px solid var(--surface-card-border);
  backdrop-filter: var(--surface-card-blur);
  box-shadow: var(--elevation-2);
}

.provider-health-summary-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.75rem;
  border-radius: 9999px;
  background: rgb(var(--color-bg-surface-rgb) / 72%);
}

.provider-health-summary-icon--healthy {
  background: rgb(var(--color-success-rgb) / 12%);
}

.provider-health-summary-icon--degraded {
  background: rgb(var(--color-warning-rgb) / 12%);
}

.provider-health-summary-icon--unhealthy {
  background: rgb(var(--color-danger-rgb) / 12%);
}

.provider-health-table-shell {
  overflow: hidden;
  border-radius: 1rem;
  background: var(--surface-workspace-bg);
  border: 1px solid var(--surface-workspace-border);
  backdrop-filter: var(--surface-workspace-blur);
  box-shadow: var(--elevation-2);
}

.provider-health-table-head {
  background: rgb(var(--color-bg-base-rgb) / 58%);
}

.provider-health-table-head th {
  padding: 0.75rem 1.5rem;
  text-align: left;
  font-size: 0.75rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-text-muted);
}

.provider-health-table-body tr + tr {
  border-top: 1px solid rgb(var(--color-border-default-rgb) / 35%);
}

.provider-health-table-row {
  transition: background-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.provider-health-table-row:hover {
  background: rgb(var(--color-bg-surface-rgb) / 48%);
}

.provider-health-model-chip {
  padding: 0.125rem 0.5rem;
  font-size: 0.75rem;
  border-radius: 9999px;
  background: rgb(var(--color-info-rgb) / 12%);
  color: var(--color-info);
}

.provider-health-errors {
  border-radius: 1rem;
  padding: 1.5rem;
  background: rgb(var(--color-danger-rgb) / 10%);
  border: 1px solid rgb(var(--color-danger-rgb) / 24%);
}

.provider-health-error-card {
  border-radius: 0.875rem;
  padding: 0.75rem;
  background: var(--surface-status-bg);
  border: 1px solid rgb(var(--color-danger-rgb) / 20%);
  backdrop-filter: var(--surface-status-blur);
}

.provider-health-empty {
  border-radius: 1rem;
  padding: 3rem;
  text-align: center;
  background: var(--surface-card-bg);
  border: 1px solid var(--surface-card-border);
  backdrop-filter: var(--surface-card-blur);
  box-shadow: var(--elevation-2);
}

@media (width >= 640px) {
  .provider-health-view {
    padding: 1.5rem;
  }
}

@media (width <= 768px) {
  .provider-health-header {
    flex-direction: column;
  }
}
</style>
