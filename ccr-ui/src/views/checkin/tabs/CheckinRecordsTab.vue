<template>
  <div class="checkin-records">
    <h2 class="checkin-records__title">
      签到记录
    </h2>
    <div
      v-if="records.length === 0"
      class="checkin-records__empty"
    >
      暂无签到记录
    </div>
    <div
      v-else
      class="checkin-records__content"
    >
      <details class="checkin-records__history">
        <summary class="checkin-records__history-summary">
          <div class="checkin-records__history-summary-label">
            <SIcon
              name="XCircle"
              size="w-4 h-4"
            />
            失败历史记录 ({{ failedHistoryTotal }})
          </div>
          <span class="checkin-records__history-summary-hint">
            点击展开详情
          </span>
        </summary>
        <div class="checkin-records__history-body">
          <div class="checkin-records__history-filters">
            <select
              v-model="failedHistoryProviderFilter"
              class="checkin-records__history-input"
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
              class="checkin-records__history-input"
            >
            <button
              class="checkin-records__history-button"
              :disabled="failedHistoryLoading"
              @click="applyFailedHistoryFilters"
            >
              筛选
            </button>
            <button
              class="checkin-records__history-button"
              :disabled="failedHistoryLoading"
              @click="resetFailedHistoryFilters"
            >
              重置
            </button>
            <button
              class="checkin-records__history-button"
              :disabled="failedHistoryLoading"
              @click="exportFailedHistory"
            >
              导出
            </button>
          </div>
          <div
            v-if="failedHistoryLoading"
            class="checkin-records__history-state"
          >
            加载中...
          </div>
          <div
            v-else-if="failedHistoryTotal === 0"
            class="checkin-records__history-state"
          >
            暂无失败记录
          </div>
          <div
            v-else
            class="checkin-records__history-list"
          >
            <div
              v-for="record in failedHistoryRecords"
              :key="record.id"
              class="checkin-records__history-item"
            >
              <div class="checkin-records__history-item-head">
                <div class="checkin-records__history-item-name">
                  {{ getAccountName(record.account_id) }}
                </div>
                <div class="checkin-records__history-item-time">
                  {{ formatDate(record.checked_in_at) }}
                </div>
              </div>
              <div class="checkin-records__history-item-meta">
                提供商: {{ getRecordProviderName(record) }} · 账号ID: {{ record.account_id }}
              </div>
              <div class="checkin-records__history-item-reason">
                原因: {{ getRecordReason(record) }}
              </div>
              <button
                v-if="record.error_code === 'cookie_expired'"
                type="button"
                class="checkin-records__history-button checkin-records__fix-button"
                @click="emit('update-cookie', record.account_id)"
              >
                更新 Cookie
              </button>
            </div>
            <div class="checkin-records__history-pagination">
              <span>
                第 {{ failedHistoryPage }} / {{ failedHistoryTotalPages }} 页
              </span>
              <div class="checkin-records__history-pagination-actions">
                <button
                  class="checkin-records__history-button"
                  :disabled="failedHistoryPage === 1"
                  @click="goToFailedHistoryPage(failedHistoryPage - 1)"
                >
                  上一页
                </button>
                <button
                  class="checkin-records__history-button"
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

      <div class="checkin-records__table-shell">
        <table class="checkin-records__table">
          <thead class="checkin-records__table-head">
            <tr>
              <th class="checkin-records__table-heading">
                时间
              </th>
              <th class="checkin-records__table-heading">
                账号
              </th>
              <th class="checkin-records__table-heading">
                状态
              </th>
              <th class="checkin-records__table-heading">
                奖励
              </th>
              <th class="checkin-records__table-heading">
                余额
              </th>
              <th class="checkin-records__table-heading">
                原因
              </th>
              <th class="checkin-records__table-heading checkin-records__table-heading--right">
                详情
              </th>
            </tr>
          </thead>
          <tbody class="checkin-records__table-body">
            <template
              v-for="record in records"
              :key="record.id"
            >
              <tr class="checkin-records__table-row">
                <td class="checkin-records__table-cell checkin-records__table-cell--muted checkin-records__table-cell--nowrap">
                  {{ formatDate(record.checked_in_at) }}
                </td>
                <td class="checkin-records__table-cell checkin-records__table-cell--primary checkin-records__table-cell--nowrap">
                  {{ getAccountName(record.account_id) }}
                </td>
                <td class="checkin-records__table-cell checkin-records__table-cell--nowrap">
                  <span
                    class="checkin-records__status-badge checkin-badge-pill"
                    :class="getStatusClass(record.status)"
                  >
                    {{ getStatusText(record.status) }}
                  </span>
                </td>
                <td class="checkin-records__table-cell checkin-records__table-cell--success checkin-records__table-cell--nowrap">
                  {{ record.reward || '-' }}
                </td>
                <td class="checkin-records__table-cell checkin-records__table-cell--muted checkin-records__table-cell--nowrap">
                  {{ record.balance_after !== undefined && record.balance_after !== null ? `$${record.balance_after.toFixed(2)}` : '-' }}
                </td>
                <td class="checkin-records__table-cell checkin-records__table-cell--muted checkin-records__table-cell--truncate">
                  {{ getRecordReason(record) }}
                </td>
                <td class="checkin-records__table-cell checkin-records__table-cell--right">
                  <div class="checkin-records__row-actions">
                    <button
                      v-if="record.error_code === 'cookie_expired'"
                      type="button"
                      class="checkin-records__detail-toggle checkin-records__fix-button"
                      @click="emit('update-cookie', record.account_id)"
                    >
                      更新 Cookie
                    </button>
                    <button
                      class="checkin-records__detail-toggle"
                      :aria-expanded="isRecordExpanded(record.id)"
                      @click="toggleRecordExpanded(record.id)"
                    >
                      <SIcon
                        v-if="isRecordExpanded(record.id)"
                        name="ChevronUp"
                        size="w-4 h-4"
                      />
                      <SIcon
                        v-else
                        name="ChevronDown"
                        size="w-4 h-4"
                      />
                      详情
                    </button>
                  </div>
                </td>
              </tr>
              <tr
                v-if="isRecordExpanded(record.id)"
                class="checkin-records__detail-row"
              >
                <td
                  colspan="7"
                  class="checkin-records__detail-cell"
                >
                  <div class="checkin-records__detail-grid">
                    <div class="checkin-records__detail-item">
                      <div class="checkin-records__detail-label">
                        提供商
                      </div>
                      <div class="checkin-records__detail-value">
                        {{ getRecordProviderName(record) }}
                      </div>
                    </div>
                    <div class="checkin-records__detail-item">
                      <div class="checkin-records__detail-label">
                        账号ID
                      </div>
                      <div class="checkin-records__detail-value checkin-records__detail-value--break">
                        {{ record.account_id }}
                      </div>
                    </div>
                    <div class="checkin-records__detail-item">
                      <div class="checkin-records__detail-label">
                        原因
                      </div>
                      <div class="checkin-records__detail-value checkin-records__detail-value--break">
                        {{ getRecordReason(record) }}
                      </div>
                    </div>
                    <div class="checkin-records__detail-item">
                      <div class="checkin-records__detail-label">
                        原始消息
                      </div>
                      <div class="checkin-records__detail-value checkin-records__detail-value--break">
                        {{ getRecordRawMessage(record) }}
                      </div>
                    </div>
                    <div class="checkin-records__detail-item">
                      <div class="checkin-records__detail-label">
                        奖励 / 余额变化
                      </div>
                      <div class="checkin-records__detail-value">
                        {{ record.reward || '-' }} ·
                        {{ record.balance_change !== undefined && record.balance_change !== null ? `$${record.balance_change.toFixed(2)}` : '-' }}
                      </div>
                    </div>
                    <div class="checkin-records__detail-item">
                      <div class="checkin-records__detail-label">
                        余额前 / 后
                      </div>
                      <div class="checkin-records__detail-value">
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
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed, onMounted, watch } from 'vue'
import { useUIStore } from '@/stores/ui'
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
import { getErrorMessage } from '@/types/api'

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

const emit = defineEmits<{
  /** cookie_expired 快捷修复：请求打开对应账号的编辑弹窗 */
  (e: 'update-cookie', accountId: string): void
}>()

const uiStore = useUIStore()

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
      return 'checkin-records__status-badge--success'
    case 'already_checked_in':
      return 'checkin-records__status-badge--warning'
    case 'failed':
      return 'checkin-records__status-badge--danger'
    case 'skipped':
      return 'checkin-records__status-badge--neutral'
    default:
      return 'checkin-records__status-badge--neutral'
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
    case 'skipped':
      return '跳过'
    default:
      return status
  }
}

const getRecordProviderName = (record: CheckinRecordInfo) => {
  if (record.provider_name) return record.provider_name
  const account = props.accounts.find(a => a.id === record.account_id)
  return account?.provider_id ? getProviderName(account.provider_id) : '-'
}

// skipped 记录的 skip_reason 经由 error_code 列持久化（4 态契约）
const skipReasonText: Record<string, string> = {
  account_disabled: '账号已禁用',
  provider_disabled: '提供商已禁用',
  provider_unsupported: '该提供商不支持签到（仅余额查询）',
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
    case 'skipped':
      return (record.error_code && skipReasonText[record.error_code]) || '已跳过'
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
    uiStore.showError('导出失败: ' + getErrorMessage(e, '未知错误'))
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

<style scoped>
.checkin-records,
.checkin-records__content,
.checkin-records__history-list {
  display: flex;
  flex-direction: column;
}

.checkin-records {
  gap: 1rem;
}

.checkin-records__title {
  font-size: 1.25rem;
  line-height: 1.75rem;
  font-weight: 600;
  color: var(--text-primary);
}

.checkin-records__empty {
  padding: 3rem 0;
  text-align: center;
  color: var(--text-muted);
}

.checkin-records__content {
  gap: 1rem;
}

.checkin-records__history {
  overflow: hidden;
  border: 1px solid rgb(254 226 226 / 100%);
  border-radius: 0.5rem;
  background: rgb(254 242 242 / 100%);
}

.dark .checkin-records__history {
  border-color: rgb(153 27 27 / 60%);
  background: rgb(127 29 29 / 20%);
}

.checkin-records__history-summary,
.checkin-records__history-summary-label,
.checkin-records__history-filters,
.checkin-records__history-item-head,
.checkin-records__history-pagination,
.checkin-records__history-pagination-actions,
.checkin-records__detail-toggle {
  display: flex;
  align-items: center;
}

.checkin-records__history-summary {
  justify-content: space-between;
  cursor: pointer;
  user-select: none;
  padding: 0.75rem 1rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
  color: rgb(185 28 28 / 100%);
}

.dark .checkin-records__history-summary {
  color: rgb(254 202 202 / 100%);
}

.checkin-records__history-summary-label {
  gap: 0.5rem;
}

.checkin-records__history-summary-hint,
.checkin-records__history-item-time,
.checkin-records__history-item-meta,
.checkin-records__history-item-reason,
.checkin-records__history-pagination,
.checkin-records__history-state,
.checkin-records__history-input,
.checkin-records__history-button {
  font-size: 0.75rem;
  line-height: 1rem;
  color: rgb(220 38 38 / 80%);
}

.dark .checkin-records__history-summary-hint,
.dark .checkin-records__history-state,
.dark .checkin-records__history-item-time,
.dark .checkin-records__history-item-meta,
.dark .checkin-records__history-item-reason,
.dark .checkin-records__history-pagination,
.dark .checkin-records__history-input,
.dark .checkin-records__history-button {
  color: rgb(252 165 165 / 80%);
}

.checkin-records__history-body {
  padding: 0.5rem 1rem 1rem;
}

.checkin-records__history-filters {
  flex-wrap: wrap;
  gap: 0.5rem;
  padding-bottom: 0.75rem;
}

.checkin-records__history-input,
.checkin-records__history-button {
  border: 1px solid rgb(254 202 202 / 100%);
  border-radius: 0.375rem;
  background: rgb(255 255 255 / 80%);
  padding: 0.25rem 0.5rem;
}

.dark .checkin-records__history-input,
.dark .checkin-records__history-button {
  border-color: rgb(153 27 27 / 100%);
  background: rgb(69 10 10 / 30%);
}

.checkin-records__history-button {
  transition: background-color 0.2s ease, opacity 0.2s ease;
}

.checkin-records__history-button:hover:not(:disabled) {
  background: rgb(254 226 226 / 100%);
}

.dark .checkin-records__history-button:hover:not(:disabled) {
  background: rgb(127 29 29 / 30%);
}

.checkin-records__history-button:disabled {
  opacity: 0.5;
}

.checkin-records__history-list {
  gap: 0.5rem;
}

.checkin-records__history-item {
  border: 1px solid rgb(254 202 202 / 100%);
  border-radius: 0.375rem;
  background: rgb(255 255 255 / 70%);
  padding: 0.75rem;
}

.dark .checkin-records__history-item {
  border-color: rgb(153 27 27 / 100%);
  background: rgb(69 10 10 / 30%);
}

.checkin-records__history-item-head,
.checkin-records__history-pagination {
  justify-content: space-between;
  gap: 1rem;
}

.checkin-records__history-item-head {
  flex-wrap: wrap;
}

.checkin-records__history-item-name {
  font-size: 0.875rem;
  line-height: 1.25rem;
  font-weight: 500;
  color: rgb(153 27 27 / 100%);
}

.dark .checkin-records__history-item-name {
  color: rgb(254 202 202 / 100%);
}

.checkin-records__history-item-meta {
  margin-top: 0.25rem;
}

.checkin-records__history-item-reason {
  margin-top: 0.5rem;
  word-break: break-all;
}

.checkin-records__history-pagination {
  padding-top: 0.5rem;
}

.checkin-records__history-pagination-actions {
  gap: 0.5rem;
}

.checkin-records__table-shell {
  overflow: hidden;
  border-radius: 0.5rem;
  background: white;
  box-shadow: 0 1px 2px rgb(15 23 42 / 8%);
}

.dark .checkin-records__table-shell {
  background: rgb(31 41 55 / 100%);
}

.checkin-records__table {
  min-width: 100%;
  border-collapse: separate;
  border-spacing: 0;
}

.checkin-records__table-head {
  background: rgb(249 250 251 / 100%);
}

.dark .checkin-records__table-head {
  background: rgb(55 65 81 / 50%);
}

.checkin-records__table-heading,
.checkin-records__table-cell {
  padding: 0.75rem 1.5rem;
}

.checkin-records__table-heading {
  text-align: left;
  font-size: 0.75rem;
  line-height: 1rem;
  font-weight: 500;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.checkin-records__table-heading--right,
.checkin-records__table-cell--right {
  text-align: right;
}

.checkin-records__table-body > tr + tr > td {
  border-top: 1px solid rgb(229 231 235 / 100%);
}

.dark .checkin-records__table-body > tr + tr > td {
  border-top-color: rgb(55 65 81 / 100%);
}

.checkin-records__table-row {
  transition: background-color 0.2s ease;
}

.checkin-records__table-row:hover {
  background: rgb(249 250 251 / 100%);
}

.dark .checkin-records__table-row:hover {
  background: rgb(55 65 81 / 50%);
}

.checkin-records__table-cell {
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.checkin-records__table-cell--muted {
  color: var(--text-muted);
}

.checkin-records__table-cell--primary {
  color: var(--text-primary);
}

.checkin-records__table-cell--success {
  color: rgb(22 163 74 / 100%);
}

.dark .checkin-records__table-cell--success {
  color: rgb(74 222 128 / 100%);
}

.checkin-records__table-cell--nowrap {
  white-space: nowrap;
}

.checkin-records__table-cell--truncate {
  max-width: 20rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 形状配方由全局 .checkin-badge-pill 提供，这里保留尺寸差异 */
.checkin-records__status-badge {
  padding: 0.25rem 0.5rem;
  line-height: 1rem;
  font-weight: 500;
}

.checkin-records__status-badge--success {
  background: rgb(220 252 231 / 100%);
  color: rgb(22 101 52 / 100%);
}

.dark .checkin-records__status-badge--success {
  background: rgb(20 83 45 / 20%);
  color: rgb(74 222 128 / 100%);
}

.checkin-records__status-badge--warning {
  background: rgb(254 249 195 / 100%);
  color: rgb(161 98 7 / 100%);
}

.dark .checkin-records__status-badge--warning {
  background: rgb(113 63 18 / 30%);
  color: rgb(250 204 21 / 100%);
}

.checkin-records__status-badge--danger {
  background: rgb(254 226 226 / 100%);
  color: rgb(153 27 27 / 100%);
}

.dark .checkin-records__status-badge--danger {
  background: rgb(127 29 29 / 20%);
  color: rgb(248 113 113 / 100%);
}

.checkin-records__status-badge--neutral {
  background: rgb(243 244 246 / 100%);
  color: rgb(31 41 55 / 100%);
}

.dark .checkin-records__status-badge--neutral {
  background: rgb(55 65 81 / 100%);
  color: rgb(156 163 175 / 100%);
}

.checkin-records__detail-toggle {
  gap: 0.25rem;
  font-size: 0.75rem;
  line-height: 1rem;
  color: rgb(37 99 235 / 100%);
  transition: color 0.2s ease;
}

.checkin-records__detail-toggle:hover {
  color: rgb(29 78 216 / 100%);
}

.dark .checkin-records__detail-toggle {
  color: rgb(147 197 253 / 100%);
}

.dark .checkin-records__detail-toggle:hover {
  color: rgb(191 219 254 / 100%);
}

.checkin-records__row-actions {
  display: inline-flex;
  align-items: center;
  gap: 0.75rem;
}

/* cookie_expired 快捷修复入口（直达账号编辑弹窗） */
.checkin-records__fix-button {
  color: rgb(220 38 38 / 100%);
}

.checkin-records__fix-button:hover {
  color: rgb(185 28 28 / 100%);
}

.dark .checkin-records__fix-button {
  color: rgb(252 165 165 / 100%);
}

.dark .checkin-records__fix-button:hover {
  color: rgb(254 202 202 / 100%);
}

.checkin-records__history-item .checkin-records__fix-button {
  margin-top: 0.5rem;
}

.checkin-records__detail-row {
  background: rgb(249 250 251 / 70%);
}

.dark .checkin-records__detail-row {
  background: rgb(31 41 55 / 60%);
}

.checkin-records__detail-cell {
  padding: 1rem 1.5rem;
  font-size: 0.875rem;
  line-height: 1.25rem;
  color: var(--text-secondary);
}

.checkin-records__detail-grid {
  display: grid;
  gap: 0.75rem;
}

.checkin-records__detail-item {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.checkin-records__detail-label {
  font-size: 0.75rem;
  line-height: 1rem;
  color: var(--text-muted);
}

.checkin-records__detail-value {
  font-size: 0.875rem;
  line-height: 1.25rem;
}

.checkin-records__detail-value--break {
  word-break: break-all;
}

@media (width >= 768px) {
  .checkin-records__detail-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}

@media (width <= 767px) {
  .checkin-records__table-shell {
    overflow-x: auto;
  }

  .checkin-records__history-summary,
  .checkin-records__history-pagination {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
