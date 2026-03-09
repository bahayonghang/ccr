<template>
  <div class="space-y-4">
    <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
      签到记录
    </h2>
    <div
      v-if="records.length === 0"
      class="text-center py-12 text-gray-500 dark:text-gray-400"
    >
      暂无签到记录
    </div>
    <div
      v-else
      class="space-y-4"
    >
      <details class="bg-red-50 dark:bg-red-900/20 border border-red-100 dark:border-red-800/60 rounded-lg overflow-hidden">
        <summary class="cursor-pointer select-none px-4 py-3 text-sm font-medium text-red-700 dark:text-red-200 flex items-center justify-between">
          <div class="flex items-center gap-2">
            <XCircle class="w-4 h-4" />
            失败历史记录 ({{ failedHistoryTotal }})
          </div>
          <span class="text-xs text-red-600/80 dark:text-red-300/80">
            点击展开详情
          </span>
        </summary>
        <div class="px-4 pb-4 pt-2">
          <div class="flex flex-wrap items-center gap-2 pb-3">
            <select
              v-model="failedHistoryProviderFilter"
              class="px-2 py-1 rounded border border-red-200 dark:border-red-800 bg-white/80 dark:bg-red-950/30 text-xs text-red-700 dark:text-red-200"
            >
              <option value="all">
                全部提供商
              </option>
              <option
                v-for="provider in providers"
                :key="provider.id"
                :value="provider.id"
              >
                {{ provider.name }}
              </option>
            </select>
            <input
              v-model="failedHistoryKeyword"
              type="text"
              placeholder="账号 / ID / 消息"
              class="px-2 py-1 rounded border border-red-200 dark:border-red-800 bg-white/80 dark:bg-red-950/30 text-xs text-red-700 dark:text-red-200"
            >
            <button
              class="px-2 py-1 rounded border border-red-200 dark:border-red-800 text-xs text-red-700 dark:text-red-200 hover:bg-red-100 dark:hover:bg-red-900/30"
              :disabled="failedHistoryLoading"
              @click="applyFailedHistoryFilters"
            >
              筛选
            </button>
            <button
              class="px-2 py-1 rounded border border-red-200 dark:border-red-800 text-xs text-red-700 dark:text-red-200 hover:bg-red-100 dark:hover:bg-red-900/30"
              :disabled="failedHistoryLoading"
              @click="resetFailedHistoryFilters"
            >
              重置
            </button>
            <button
              class="px-2 py-1 rounded border border-red-200 dark:border-red-800 text-xs text-red-700 dark:text-red-200 hover:bg-red-100 dark:hover:bg-red-900/30"
              :disabled="failedHistoryLoading"
              @click="exportFailedHistory"
            >
              导出
            </button>
          </div>
          <div
            v-if="failedHistoryLoading"
            class="text-sm text-red-500/80 dark:text-red-300/80"
          >
            加载中...
          </div>
          <div
            v-else-if="failedHistoryTotal === 0"
            class="text-sm text-red-500/80 dark:text-red-300/80"
          >
            暂无失败记录
          </div>
          <div
            v-else
            class="space-y-2"
          >
            <div
              v-for="record in failedHistoryRecords"
              :key="record.id"
              class="p-3 rounded-md border border-red-200 dark:border-red-800 bg-white/70 dark:bg-red-950/30"
            >
              <div class="flex items-start justify-between gap-4 flex-wrap">
                <div class="text-sm font-medium text-red-800 dark:text-red-200">
                  {{ getAccountName(record.account_id) }}
                </div>
                <div class="text-xs text-red-600 dark:text-red-300">
                  {{ formatDate(record.checked_in_at) }}
                </div>
              </div>
              <div class="mt-1 text-xs text-red-600 dark:text-red-300">
                提供商: {{ getRecordProviderName(record) }} · 账号ID: {{ record.account_id }}
              </div>
              <div class="mt-2 text-xs text-red-600 dark:text-red-300 break-all">
                原因: {{ getRecordReason(record) }}
              </div>
            </div>
            <div class="flex items-center justify-between pt-2 text-xs text-red-600 dark:text-red-300">
              <span>
                第 {{ failedHistoryPage }} / {{ failedHistoryTotalPages }} 页
              </span>
              <div class="flex items-center gap-2">
                <button
                  class="px-2 py-1 rounded border border-red-200 dark:border-red-800 hover:bg-red-100 dark:hover:bg-red-900/30 disabled:opacity-50"
                  :disabled="failedHistoryPage === 1"
                  @click="goToFailedHistoryPage(failedHistoryPage - 1)"
                >
                  上一页
                </button>
                <button
                  class="px-2 py-1 rounded border border-red-200 dark:border-red-800 hover:bg-red-100 dark:hover:bg-red-900/30 disabled:opacity-50"
                  :disabled="failedHistoryPage === failedHistoryTotalPages"
                  @click="goToFailedHistoryPage(failedHistoryPage + 1)"
                >
                  下一页
                </button>
              </div>
            </div>
          </div>
        </div>
      </details>

      <div class="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
        <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
          <thead class="bg-gray-50 dark:bg-gray-700/50">
            <tr>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                时间
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                账号
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                状态
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                奖励
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                余额
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                原因
              </th>
              <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                详情
              </th>
            </tr>
          </thead>
          <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
            <template
              v-for="record in records"
              :key="record.id"
            >
              <tr class="hover:bg-gray-50 dark:hover:bg-gray-700/50">
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                  {{ formatDate(record.checked_in_at) }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                  {{ getAccountName(record.account_id) }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  <span
                    class="px-2 py-1 text-xs font-medium rounded-full"
                    :class="getStatusClass(record.status)"
                  >
                    {{ getStatusText(record.status) }}
                  </span>
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-green-600 dark:text-green-400">
                  {{ record.reward || '-' }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                  {{ record.balance_after !== undefined && record.balance_after !== null ? `$${record.balance_after.toFixed(2)}` : '-' }}
                </td>
                <td class="px-6 py-4 text-sm text-gray-500 dark:text-gray-400 max-w-xs truncate">
                  {{ getRecordReason(record) }}
                </td>
                <td class="px-6 py-4 text-right">
                  <button
                    class="inline-flex items-center gap-1 text-xs text-blue-600 hover:text-blue-700 dark:text-blue-300 dark:hover:text-blue-200"
                    :aria-expanded="isRecordExpanded(record.id)"
                    @click="toggleRecordExpanded(record.id)"
                  >
                    <ChevronUp
                      v-if="isRecordExpanded(record.id)"
                      class="w-4 h-4"
                    />
                    <ChevronDown
                      v-else
                      class="w-4 h-4"
                    />
                    详情
                  </button>
                </td>
              </tr>
              <tr
                v-if="isRecordExpanded(record.id)"
                class="bg-gray-50/70 dark:bg-gray-800/60"
              >
                <td
                  colspan="7"
                  class="px-6 py-4 text-sm text-gray-600 dark:text-gray-300"
                >
                  <div class="grid gap-3 md:grid-cols-3">
                    <div class="space-y-1">
                      <div class="text-xs text-gray-500 dark:text-gray-400">
                        提供商
                      </div>
                      <div class="text-sm">
                        {{ getRecordProviderName(record) }}
                      </div>
                    </div>
                    <div class="space-y-1">
                      <div class="text-xs text-gray-500 dark:text-gray-400">
                        账号ID
                      </div>
                      <div class="text-sm break-all">
                        {{ record.account_id }}
                      </div>
                    </div>
                    <div class="space-y-1">
                      <div class="text-xs text-gray-500 dark:text-gray-400">
                        原因
                      </div>
                      <div class="text-sm break-all">
                        {{ getRecordReason(record) }}
                      </div>
                    </div>
                    <div class="space-y-1">
                      <div class="text-xs text-gray-500 dark:text-gray-400">
                        原始消息
                      </div>
                      <div class="text-sm break-all">
                        {{ getRecordRawMessage(record) }}
                      </div>
                    </div>
                    <div class="space-y-1">
                      <div class="text-xs text-gray-500 dark:text-gray-400">
                        奖励 / 余额变化
                      </div>
                      <div class="text-sm">
                        {{ record.reward || '-' }} ·
                        {{ record.balance_change !== undefined && record.balance_change !== null ? `$${record.balance_change.toFixed(2)}` : '-' }}
                      </div>
                    </div>
                    <div class="space-y-1">
                      <div class="text-xs text-gray-500 dark:text-gray-400">
                        余额前 / 后
                      </div>
                      <div class="text-sm">
                        {{ record.balance_before !== undefined && record.balance_before !== null ? `$${record.balance_before.toFixed(2)}` : '-' }}
                        →
                        {{ record.balance_after !== undefined && record.balance_after !== null ? `$${record.balance_after.toFixed(2)}` : '-' }}
                      </div>
                    </div>
                  </div>
                </td>
              </tr>
            </template>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import {
  XCircle,
  ChevronDown,
  ChevronUp,
} from 'lucide-vue-next'
import {
  listCheckinRecords,
  exportCheckinRecords,
} from '@/api'
import type {
  CheckinRecordInfo,
  CheckinProvider,
  AccountInfo,
  TodayCheckinStats,
  CheckinRecordsQuery,
  CheckinRecordsResponse,
} from '@/types/checkin'
import { logger } from '@/utils/logger'

interface CheckinRecordsExportResponse {
  blob: Blob
  filename: string
}

const props = defineProps<{
  records: CheckinRecordInfo[]
  providers: CheckinProvider[]
  accounts: AccountInfo[]
  todayStats: TodayCheckinStats | null
}>()

// 记录展开状态
const expandedRecordIds = ref<string[]>([])

// 失败历史记录相关
const failedHistoryRecords = ref<CheckinRecordInfo[]>([])
const failedHistoryTotal = ref(0)
const failedHistoryLoading = ref(false)
const failedHistoryPage = ref(1)
const failedHistoryPageSize = ref(5)
const failedHistoryProviderFilter = ref<string>('all')
const failedHistoryKeyword = ref('')
const getErrorMessage = (error: unknown, fallback: string) =>
  error instanceof Error ? error.message : fallback

const failedHistoryTotalPages = computed(() => {
  const total = Math.ceil(failedHistoryTotal.value / failedHistoryPageSize.value)
  return total > 0 ? total : 1
})

// 辅助函数
const getProviderName = (providerId: string) => {
  return props.providers.find(p => p.id === providerId)?.name || providerId
}

const getAccountName = (accountId: string) => {
  return props.accounts.find(a => a.id === accountId)?.name || accountId
}

const formatDate = (dateStr: string) => {
  return new Date(dateStr).toLocaleString('zh-CN')
}

const getStatusClass = (status: string) => {
  switch (status) {
    case 'success':
      return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400'
    case 'already_checked_in':
      return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400'
    case 'failed':
      return 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400'
    default:
      return 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-400'
  }
}

const getStatusText = (status: string) => {
  switch (status) {
    case 'success':
      return '成功'
    case 'already_checked_in':
      return '已签到'
    case 'failed':
      return '失败'
    default:
      return status
  }
}

const getRecordProviderName = (record: CheckinRecordInfo) => {
  if (record.provider_name) return record.provider_name
  const account = props.accounts.find(a => a.id === record.account_id)
  return account?.provider_id ? getProviderName(account.provider_id) : '-'
}

const getRecordReason = (record: CheckinRecordInfo) => {
  if (record.message) return record.message
  switch (record.status) {
    case 'success':
      return record.reward ? `签到成功 · 奖励 ${record.reward}` : '签到成功'
    case 'already_checked_in':
      return '今日已签到'
    case 'failed':
      return '未知原因'
    default:
      return '-'
  }
}

const getRecordRawMessage = (record: CheckinRecordInfo) => record.message || '-'

const isRecordExpanded = (recordId: string) => {
  return expandedRecordIds.value.includes(recordId)
}

const toggleRecordExpanded = (recordId: string) => {
  expandedRecordIds.value = expandedRecordIds.value.includes(recordId)
    ? expandedRecordIds.value.filter(id => id !== recordId)
    : [...expandedRecordIds.value, recordId]
}

// 失败历史记录加载
const loadFailedHistory = async () => {
  failedHistoryLoading.value = true
  try {
    const params: CheckinRecordsQuery = {
      status: 'failed',
      page: failedHistoryPage.value,
      page_size: failedHistoryPageSize.value,
    }
    if (failedHistoryProviderFilter.value !== 'all') {
      params.provider_id = failedHistoryProviderFilter.value
    }
    if (failedHistoryKeyword.value.trim()) {
      params.keyword = failedHistoryKeyword.value.trim()
    }
    const response = await listCheckinRecords<CheckinRecordsResponse>(params)
    failedHistoryRecords.value = response.records
    failedHistoryTotal.value = response.total
  } catch (e: unknown) {
    logger.error('Failed to load failed history:', e)
  } finally {
    failedHistoryLoading.value = false
  }
}

const applyFailedHistoryFilters = async () => {
  failedHistoryPage.value = 1
  await loadFailedHistory()
}

const resetFailedHistoryFilters = async () => {
  failedHistoryProviderFilter.value = 'all'
  failedHistoryKeyword.value = ''
  failedHistoryPage.value = 1
  await loadFailedHistory()
}

const goToFailedHistoryPage = async (page: number) => {
  const nextPage = Math.min(Math.max(page, 1), failedHistoryTotalPages.value)
  if (nextPage === failedHistoryPage.value) return
  failedHistoryPage.value = nextPage
  await loadFailedHistory()
}

const exportFailedHistory = async () => {
  try {
    const params: CheckinRecordsQuery = { status: 'failed' }
    if (failedHistoryProviderFilter.value !== 'all') {
      params.provider_id = failedHistoryProviderFilter.value
    }
    if (failedHistoryKeyword.value.trim()) {
      params.keyword = failedHistoryKeyword.value.trim()
    }
    const { blob, filename } = await exportCheckinRecords<CheckinRecordsExportResponse>(params)
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = filename
    document.body.appendChild(link)
    link.click()
    link.remove()
    URL.revokeObjectURL(url)
  } catch (e: unknown) {
    alert('导出失败: ' + getErrorMessage(e, '未知错误'))
  }
}

// 监听失败历史总数，修正页码
watch(
  () => failedHistoryTotal.value,
  () => {
    if (failedHistoryPage.value > failedHistoryTotalPages.value) {
      failedHistoryPage.value = failedHistoryTotalPages.value
    }
  }
)

onMounted(() => {
  loadFailedHistory()
})
</script>
