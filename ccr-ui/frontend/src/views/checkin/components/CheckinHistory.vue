<template>
  <div class="checkin-history">
    <div class="section-header">
      <h2>{{ t('checkin.history.title') }}</h2>
      <button
        class="refresh-button"
        @click="fetchRecords"
      >
        {{ t('common.refresh') }}
      </button>
    </div>

    <div
      v-if="loading"
      class="loading"
    >
      {{ t('common.loading') }}
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
        {{ t('checkin.history.no_history') }}
      </div>
      <table
        v-else
        class="history-table"
      >
        <thead>
          <tr>
            <th>{{ t('checkin.history.account') }}</th>
            <th>{{ t('checkin.history.status') }}</th>
            <th>{{ t('checkin.history.time') }}</th>
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
import { useI18n } from 'vue-i18n'
import { listCheckinRecords } from '@/api/client'
import type { CheckinRecordInfo } from '@/types/checkin'

const records = ref<CheckinRecordInfo[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const { t } = useI18n()

const fetchRecords = async () => {
  loading.value = true
  error.value = null

  try {
    const response = await listCheckinRecords({ page: 1, page_size: 20 })
    records.value = response.records
  } catch (err) {
    error.value = t('checkin.history.load_error')
  } finally {
    loading.value = false
  }
}

const statusText = (status: string) => {
  switch (status) {
    case 'Success':
      return t('checkin.status.success')
    case 'AlreadyCheckedIn':
      return t('checkin.status.already_checked_in')
    case 'Failed':
      return t('checkin.status.failed')
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

const formatDate = (dateStr: string) => new Date(dateStr).toLocaleString()

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
  color: var(--text-primary);
}

.refresh-button {
  padding: 0.5rem 1rem;
  border-radius: 0.5rem;
  border: 1px solid rgba(var(--color-gray-rgb), 0.35);
  background: var(--bg-primary);
  color: var(--text-primary);
  cursor: pointer;
  font-size: 0.875rem;
  transition: all 0.2s ease;
}

.refresh-button:hover {
  background: var(--bg-secondary);
}

.history-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.875rem;
}

.history-table th,
.history-table td {
  padding: 0.75rem;
  border-bottom: 1px solid var(--glass-border-medium);
  text-align: left;
  color: var(--text-primary);
}

.history-table th {
  color: var(--text-secondary);
}

.status {
  display: inline-block;
  padding: 0.125rem 0.5rem;
  border-radius: 999px;
  font-size: 0.75rem;
}

.status.success {
  background: rgba(var(--color-success-rgb), 0.15);
  color: var(--accent-success);
}

.status.warning {
  background: rgba(var(--color-warning-rgb), 0.15);
  color: var(--accent-warning);
}

.status.danger {
  background: rgba(var(--color-danger-rgb), 0.15);
  color: var(--accent-danger);
}

.loading,
.error,
.empty {
  text-align: center;
  padding: 2rem;
  color: var(--text-muted);
}

.error {
  color: var(--accent-danger);
}

:global(.dark) .refresh-button {
  background: rgba(var(--color-slate-dark-rgb), 0.8);
  border-color: rgba(var(--color-slate-rgb), 0.6);
}

:global(.dark) .refresh-button:hover {
  background: rgba(var(--color-slate-rgb), 0.6);
}

:global(.dark) .history-table th,
:global(.dark) .history-table td {
  border-color: rgba(var(--color-slate-rgb), 0.4);
}
</style>
