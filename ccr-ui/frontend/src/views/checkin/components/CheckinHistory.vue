<template>
  <div class="checkin-history">
    <div class="section-header">
      <h2>签到历史</h2>
      <button
        class="refresh-button"
        @click="fetchRecords"
      >
        刷新
      </button>
    </div>

    <div
      v-if="loading"
      class="loading"
    >
      加载中...
    </div>
    <div
      v-else-if="error"
      class="error"
    >
      {{ error }}
    </div>
    <div v-else>
      <div
        v-if="records.length === 0"
        class="empty"
      >
        暂无签到记录
      </div>
      <table
        v-else
        class="history-table"
      >
        <thead>
          <tr>
            <th>账号</th>
            <th>状态</th>
            <th>时间</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="record in records"
            :key="record.id"
          >
            <td>{{ record.account_name || record.account_id }}</td>
            <td>
              <span :class="statusClass(record.status)">
                {{ statusText(record.status) }}
              </span>
            </td>
            <td>{{ formatDate(record.checked_in_at) }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { listCheckinRecords } from '@/api/client'
import type { CheckinRecordInfo } from '@/types/checkin'

const records = ref<CheckinRecordInfo[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

const fetchRecords = async () => {
  loading.value = true
  error.value = null

  try {
    const response = await listCheckinRecords(20)
    records.value = response.records
  } catch (err) {
    error.value = '加载签到记录失败'
  } finally {
    loading.value = false
  }
}

const statusText = (status: string) => {
  switch (status) {
    case 'Success':
      return '成功'
    case 'AlreadyCheckedIn':
      return '已签到'
    case 'Failed':
      return '失败'
    default:
      return status
  }
}

const statusClass = (status: string) => {
  switch (status) {
    case 'Success':
      return 'status success'
    case 'AlreadyCheckedIn':
      return 'status warning'
    case 'Failed':
      return 'status danger'
    default:
      return 'status'
  }
}

const formatDate = (dateStr: string) => new Date(dateStr).toLocaleString('zh-CN')

onMounted(() => {
  fetchRecords()
})
</script>

<style scoped>
.checkin-history {
  width: 100%;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.section-header h2 {
  margin: 0;
  font-size: 1.25rem;
  color: #1f2937;
}

.refresh-button {
  padding: 0.5rem 1rem;
  border-radius: 0.5rem;
  border: 1px solid #d1d5db;
  background: white;
  cursor: pointer;
  font-size: 0.875rem;
}

.refresh-button:hover {
  background: #f3f4f6;
}

.history-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.875rem;
}

.history-table th,
.history-table td {
  padding: 0.75rem;
  border-bottom: 1px solid #e5e7eb;
  text-align: left;
}

.status {
  display: inline-block;
  padding: 0.125rem 0.5rem;
  border-radius: 999px;
  font-size: 0.75rem;
}

.status.success {
  background: #dcfce7;
  color: #166534;
}

.status.warning {
  background: #fef9c3;
  color: #92400e;
}

.status.danger {
  background: #fee2e2;
  color: #b91c1c;
}

.loading,
.error,
.empty {
  text-align: center;
  padding: 2rem;
}

.error {
  color: #dc2626;
}
</style>
