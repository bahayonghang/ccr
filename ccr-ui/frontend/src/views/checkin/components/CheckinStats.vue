<template>
  <div class="checkin-stats">
    <div class="stats-grid">
      <div class="stat-card consecutive">
        <div class="stat-icon">
          🔥
        </div>
        <div class="stat-content">
          <div class="stat-value">
            {{ stats.consecutive_days }}
          </div>
          <div class="stat-label">
            {{ t('checkin.stats.consecutive_days') }}
          </div>
        </div>
      </div>

      <div class="stat-card total">
        <div class="stat-icon">
          📅
        </div>
        <div class="stat-content">
          <div class="stat-value">
            {{ stats.total_days }}
          </div>
          <div class="stat-label">
            {{ t('checkin.stats.total_days') }}
          </div>
        </div>
      </div>

      <div class="stat-card longest">
        <div class="stat-icon">
          🏆
        </div>
        <div class="stat-content">
          <div class="stat-value">
            {{ stats.longest_consecutive }}
          </div>
          <div class="stat-label">
            {{ t('checkin.stats.longest_consecutive') }}
          </div>
        </div>
      </div>

      <div class="stat-card rate">
        <div class="stat-icon">
          📈
        </div>
        <div class="stat-content">
          <div class="stat-value">
            {{ stats.monthly_rate.toFixed(0) }}%
          </div>
          <div class="stat-label">
            {{ t('checkin.stats.monthly_rate') }}
          </div>
        </div>
      </div>
    </div>

    <div
      v-if="error"
      class="error"
    >
      {{ error }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import axios from 'axios'
import { useI18n } from 'vue-i18n'

interface DashboardStats {
  consecutive_days: number
  total_days: number
  longest_consecutive: number
  monthly_rate: number
  weekly_rate: number
  total_accounts: number
  checked_in_today: number
  not_checked_in_today: number
}

const { t } = useI18n()

const stats = ref<DashboardStats>({
  consecutive_days: 0,
  total_days: 0,
  longest_consecutive: 0,
  monthly_rate: 0,
  weekly_rate: 0,
  total_accounts: 0,
  checked_in_today: 0,
  not_checked_in_today: 0
})

const error = ref<string | null>(null)

const fetchStats = async () => {
  try {
    const response = await axios.get('/api/checkin/dashboard/stats')
    stats.value = response.data
  } catch (err) {
    error.value = t('checkin.stats.load_error')
  }
}

onMounted(() => {
  fetchStats()
})
</script>

<style scoped>
.checkin-stats {
  width: 100%;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
}

.stat-card {
  background: var(--gradient-secondary);
  border-radius: 1rem;
  padding: 1.5rem;
  display: flex;
  align-items: center;
  gap: 1rem;
  color: white;
  box-shadow: var(--shadow-small);
}

.stat-icon {
  width: 48px;
  height: 48px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 0.75rem;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.25rem;
}

.stat-value {
  font-size: 2rem;
  font-weight: 700;
  line-height: 1;
}

.stat-label {
  font-size: 0.875rem;
  opacity: 0.9;
  margin-top: 0.25rem;
}

.error {
  margin-top: 1rem;
  padding: 0.75rem 1rem;
  border-radius: 0.75rem;
  background: rgba(var(--color-danger-rgb), 0.2);
  color: var(--accent-danger);
  text-align: center;
  font-size: 0.875rem;
}
</style>
