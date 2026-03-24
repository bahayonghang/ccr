<template>
  <div class="provider-health-view p-6 space-y-6">
    <!-- 页面标题 -->
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold text-text-primary">
          🏥 Provider 健康检查
        </h1>
        <p class="mt-2 text-sm text-text-secondary">
          检测 API 端点连通性和 Key 有效性
        </p>
      </div>
      <div class="flex items-center space-x-4">
        <!-- 测试所有按钮 -->
        <button
          :disabled="testing"
          class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg flex items-center space-x-2 disabled:opacity-50"
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
      class="grid grid-cols-1 md:grid-cols-4 gap-6"
    >
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-text-secondary">
              总计
            </p>
            <p class="mt-2 text-3xl font-bold text-text-primary">
              {{ summary.total }}
            </p>
          </div>
          <div class="p-3 bg-gray-100 dark:bg-gray-700 rounded-full">
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
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-text-secondary">
              健康
            </p>
            <p class="mt-2 text-3xl font-bold text-green-600 dark:text-green-400">
              {{ summary.healthy }}
            </p>
          </div>
          <div class="p-3 bg-green-100 dark:bg-green-900/20 rounded-full">
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
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-text-secondary">
              降级
            </p>
            <p class="mt-2 text-3xl font-bold text-yellow-600 dark:text-yellow-400">
              {{ summary.degraded }}
            </p>
          </div>
          <div class="p-3 bg-yellow-100 dark:bg-yellow-900/20 rounded-full">
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
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-text-secondary">
              不可用
            </p>
            <p class="mt-2 text-3xl font-bold text-red-600 dark:text-red-400">
              {{ summary.unhealthy }}
            </p>
          </div>
          <div class="p-3 bg-red-100 dark:bg-red-900/20 rounded-full">
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
      class="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden"
    >
      <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
        <thead class="bg-gray-50 dark:bg-gray-900/50">
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
        <tbody class="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
          <tr
            v-for="result in results"
            :key="result.provider_name"
            class="hover:bg-gray-50 dark:hover:bg-gray-700/50"
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
                  class="px-2 py-0.5 text-xs bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-300 rounded"
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
      class="bg-red-50 dark:bg-red-900/20 rounded-lg p-6"
    >
      <h3 class="text-lg font-bold text-red-800 dark:text-red-200 mb-4">
        错误详情
      </h3>
      <div class="space-y-3">
        <div
          v-for="result in errorResults"
          :key="result.provider_name"
          class="p-3 bg-white dark:bg-gray-800 rounded border border-red-200 dark:border-red-800"
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
      class="bg-white dark:bg-gray-800 rounded-lg shadow p-12 text-center"
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
}
</style>
