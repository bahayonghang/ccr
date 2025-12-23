<template>
  <div class="checkin-account-dashboard">
    <div class="dashboard-glow" />
    <div class="dashboard-shell">
      <div class="dashboard-header">
        <div class="header-left">
          <button
            class="icon-button"
            @click="goBack"
          >
            <ArrowLeft class="w-4 h-4" />
          </button>
          <div>
            <div class="header-title">
              <h1>{{ dashboard?.account.name || '账号 Dashboard' }}</h1>
              <span class="provider-pill">{{ dashboard?.account.provider_name || '未知提供商' }}</span>
              <span
                v-if="dashboard"
                class="status-pill"
                :class="accountEnabled ? 'status-on' : 'status-off'"
              >
                {{ accountEnabled ? '启用' : '已禁用' }}
              </span>
            </div>
            <div class="header-sub">
              <span>最后签到：{{ dashboard?.streak.last_check_in_date || '-' }}</span>
              <span>余额更新：{{ formatDateTime(dashboard?.account.last_balance_check_at) }}</span>
            </div>
          </div>
        </div>

        <div class="header-actions">
          <button
            class="action-btn"
            :disabled="loading || !dashboard || checkinLoading"
            @click="handleCheckin"
          >
            <CheckCircle2 class="w-4 h-4" />
            签到
          </button>
          <button
            class="action-btn"
            :disabled="loading || !dashboard || balanceLoading"
            @click="handleBalanceRefresh"
          >
            <Wallet class="w-4 h-4" />
            刷新余额
          </button>
          <button
            class="action-btn primary"
            :disabled="loading"
            @click="loadDashboard"
          >
            <RefreshCw
              class="w-4 h-4"
              :class="{ 'animate-spin': loading }"
            />
            刷新
          </button>
        </div>
      </div>

      <div
        v-if="error"
        class="state-card state-error"
      >
        <p>{{ error }}</p>
        <button
          class="ghost-link"
          @click="loadDashboard"
        >
          重试
        </button>
      </div>

      <div
        v-else-if="loading"
        class="state-card state-loading"
      >
        <div class="loader" />
        加载中...
      </div>

      <div
        v-else-if="dashboard"
        class="space-y-6"
      >
        <div class="grid grid-cols-1 xl:grid-cols-3 gap-4">
          <div class="space-y-4">
            <div class="dashboard-card">
              <div class="card-header">
                <h2>账号概览</h2>
                <span class="card-tag">{{ dashboard.account.balance_currency || 'USD' }}</span>
              </div>
              <div class="card-body">
                <div class="metric-large">
                  <span class="metric-label">当前余额</span>
                  <span class="metric-value metric-green">
                    {{ formatCurrency(dashboard.account.latest_balance, dashboard.account.balance_currency) }}
                  </span>
                </div>
                <div class="metric-grid">
                  <div>
                    <p>总额度</p>
                    <strong>{{ formatCurrency(dashboard.account.total_quota, dashboard.account.balance_currency) }}</strong>
                  </div>
                  <div>
                    <p>已使用</p>
                    <strong>{{ formatCurrency(dashboard.account.used_quota, dashboard.account.balance_currency) }}</strong>
                  </div>
                  <div>
                    <p>剩余额度</p>
                    <strong>{{ formatCurrency(dashboard.account.remaining_quota, dashboard.account.balance_currency) }}</strong>
                  </div>
                  <div>
                    <p>余额检查</p>
                    <strong>{{ formatDateTime(dashboard.account.last_balance_check_at) }}</strong>
                  </div>
                </div>
                <div class="progress-block">
                  <div class="progress-track">
                    <div
                      class="progress-bar"
                      :style="{ width: `${usagePercent}%` }"
                    />
                  </div>
                  <div class="progress-meta">
                    已使用 {{ usagePercent.toFixed(1) }}%
                  </div>
                </div>
              </div>
            </div>

            <div class="dashboard-card">
              <div class="card-header">
                <h2>签到统计</h2>
                <span class="card-tag">本月</span>
              </div>
              <div class="card-body">
                <div class="metric-grid highlight">
                  <div>
                    <p>当前连续</p>
                    <strong class="metric-orange">{{ dashboard.streak.current_streak }} 天</strong>
                  </div>
                  <div>
                    <p>最长连续</p>
                    <strong>{{ dashboard.streak.longest_streak }} 天</strong>
                  </div>
                  <div>
                    <p>累计签到</p>
                    <strong>{{ dashboard.streak.total_check_in_days }} 天</strong>
                  </div>
                  <div>
                    <p>本月增量</p>
                    <strong class="metric-blue">
                      {{ formatCurrency(dashboard.calendar.month_stats.total_quota_increment, dashboard.account.balance_currency) }}
                    </strong>
                  </div>
                </div>
                <div class="progress-block">
                  <div class="progress-track soft">
                    <div
                      class="progress-bar accent"
                      :style="{ width: `${dashboard.calendar.month_stats.check_in_rate}%` }"
                    />
                  </div>
                  <div class="progress-meta">
                    本月签到率 {{ dashboard.calendar.month_stats.check_in_rate.toFixed(1) }}%
                    · {{ dashboard.calendar.month_stats.checked_in_days }}/{{ dashboard.calendar.month_stats.total_days }} 天
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="dashboard-card xl:col-span-2">
            <div class="card-header">
              <div class="flex items-center gap-3">
                <h2>签到日历</h2>
                <span class="card-tag">{{ monthLabel }}</span>
              </div>
              <div class="calendar-actions">
                <button
                  class="icon-button small"
                  @click="prevMonth"
                >
                  ‹
                </button>
                <button
                  class="icon-button small"
                  @click="nextMonth"
                >
                  ›
                </button>
              </div>
            </div>
            <div class="calendar-meta">
              <div>
                <p>本月已签</p>
                <strong>{{ dashboard.calendar.month_stats.checked_in_days }} / {{ dashboard.calendar.month_stats.total_days }}</strong>
              </div>
              <div>
                <p>签到率</p>
                <strong>{{ dashboard.calendar.month_stats.check_in_rate.toFixed(1) }}%</strong>
              </div>
              <div>
                <p>累计增量</p>
                <strong>{{ formatCurrency(dashboard.calendar.month_stats.total_quota_increment, dashboard.account.balance_currency) }}</strong>
              </div>
            </div>
            <AccountDashboardCalendar :calendar="dashboard.calendar" />
          </div>
        </div>

        <div class="dashboard-card">
          <div class="card-header">
            <div class="flex items-center gap-3">
              <h2>签到趋势</h2>
              <span class="card-tag">近 {{ trendDays }} 天</span>
            </div>
            <div class="trend-actions">
              <button
                v-for="option in trendOptions"
                :key="option"
                class="trend-btn"
                :class="{ active: trendDays === option }"
                @click="trendDays = option"
              >
                {{ option }}D
              </button>
            </div>
          </div>
          <AccountDashboardTrend :trend="dashboard.trend" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft, CheckCircle2, RefreshCw, Wallet } from 'lucide-vue-next'
import { checkinAccount, getCheckinAccountDashboard, queryCheckinBalance } from '@/api/client'
import type { CheckinAccountDashboardResponse } from '@/types/checkin'
import AccountDashboardCalendar from './components/AccountDashboardCalendar.vue'
import AccountDashboardTrend from './components/AccountDashboardTrend.vue'

const route = useRoute()
const router = useRouter()

const accountId = computed(() => route.params.accountId as string)
const dashboard = ref<CheckinAccountDashboardResponse | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const checkinLoading = ref(false)
const balanceLoading = ref(false)

const now = new Date()
const calendarYear = ref(now.getFullYear())
const calendarMonth = ref(now.getMonth() + 1)
const trendDays = ref(30)
const trendOptions = [7, 30, 90]

const monthLabel = computed(() => `${calendarYear.value}年${calendarMonth.value}月`)
const accountEnabled = computed(() => dashboard.value?.account.enabled ?? false)

const usagePercent = computed(() => {
  const total = dashboard.value?.account.total_quota ?? 0
  const used = dashboard.value?.account.used_quota ?? 0
  if (total <= 0) return 0
  return Math.min(100, Math.max(0, (used / total) * 100))
})

const loadDashboard = async () => {
  if (!accountId.value) return
  loading.value = true
  error.value = null

  try {
    dashboard.value = await getCheckinAccountDashboard(accountId.value, {
      year: calendarYear.value,
      month: calendarMonth.value,
      days: trendDays.value,
    })
  } catch (e: any) {
    error.value = e.message || '加载失败'
  } finally {
    loading.value = false
  }
}

const handleCheckin = async () => {
  if (!accountId.value) return
  checkinLoading.value = true
  try {
    const result = await checkinAccount(accountId.value)
    alert(`签到${result.status === 'Success' ? '成功' : result.status === 'AlreadyCheckedIn' ? '：今日已签到' : '失败'}: ${result.message || ''}`)
    await loadDashboard()
  } catch (e: any) {
    alert('签到失败: ' + (e.message || '未知错误'))
  } finally {
    checkinLoading.value = false
  }
}

const handleBalanceRefresh = async () => {
  if (!accountId.value) return
  balanceLoading.value = true
  try {
    const result = await queryCheckinBalance(accountId.value)
    alert(`余额: ${result.currency}${result.remaining_quota.toFixed(2)} (已用: ${result.usage_percentage.toFixed(1)}%)`)
    await loadDashboard()
  } catch (e: any) {
    alert('刷新余额失败: ' + (e.message || '未知错误'))
  } finally {
    balanceLoading.value = false
  }
}

const goBack = () => {
  if (window.history.length > 1) {
    router.back()
  } else {
    router.push({ name: 'checkin' })
  }
}

const prevMonth = () => {
  if (calendarMonth.value === 1) {
    calendarMonth.value = 12
    calendarYear.value -= 1
  } else {
    calendarMonth.value -= 1
  }
}

const nextMonth = () => {
  if (calendarMonth.value === 12) {
    calendarMonth.value = 1
    calendarYear.value += 1
  } else {
    calendarMonth.value += 1
  }
}

const formatCurrency = (value?: number, currency?: string) => {
  if (value === undefined || value === null) return '-'
  const symbol = currency === 'CNY' ? '¥' : currency === 'USD' ? '$' : currency ? `${currency} ` : '$'
  return `${symbol}${value.toFixed(2)}`
}

const formatDateTime = (value?: string) => {
  if (!value) return '-'
  return new Date(value).toLocaleString('zh-CN')
}

watch([accountId, calendarYear, calendarMonth, trendDays], loadDashboard, { immediate: true })
</script>

<style scoped>
.checkin-account-dashboard {
  position: relative;
  min-height: 100vh;
  background: radial-gradient(circle at top left, rgba(59, 130, 246, 0.15), transparent 45%),
    radial-gradient(circle at 20% 20%, rgba(16, 185, 129, 0.12), transparent 40%),
    radial-gradient(circle at 80% 0%, rgba(217, 70, 239, 0.15), transparent 40%),
    #f7f8fb;
  overflow: hidden;
}

.dashboard-glow {
  position: absolute;
  top: -120px;
  right: -80px;
  width: 280px;
  height: 280px;
  background: radial-gradient(circle, rgba(99, 102, 241, 0.25), transparent 70%);
  filter: blur(10px);
  pointer-events: none;
}

.dashboard-shell {
  position: relative;
  z-index: 1;
  padding: 2rem;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.dashboard-header {
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
  align-items: center;
  justify-content: space-between;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.header-title {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.75rem;
}

.header-title h1 {
  margin: 0;
  font-size: 1.75rem;
  font-weight: 700;
  color: #0f172a;
}

.header-sub {
  display: flex;
  gap: 1rem;
  font-size: 0.8rem;
  color: #64748b;
  margin-top: 0.25rem;
}

.provider-pill,
.status-pill,
.card-tag {
  display: inline-flex;
  align-items: center;
  padding: 0.2rem 0.6rem;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 600;
  background: rgba(99, 102, 241, 0.1);
  color: #4f46e5;
}

.status-pill.status-on {
  background: rgba(16, 185, 129, 0.15);
  color: #059669;
}

.status-pill.status-off {
  background: rgba(248, 113, 113, 0.15);
  color: #ef4444;
}

.icon-button {
  height: 2.25rem;
  width: 2.25rem;
  border-radius: 999px;
  border: 1px solid rgba(148, 163, 184, 0.35);
  background: rgba(255, 255, 255, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #475569;
  transition: all 0.2s ease;
}

.icon-button:hover {
  background: white;
  transform: translateY(-1px);
}

.icon-button.small {
  height: 1.75rem;
  width: 1.75rem;
}

.header-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.5rem 0.9rem;
  border-radius: 0.8rem;
  border: 1px solid rgba(148, 163, 184, 0.35);
  background: rgba(255, 255, 255, 0.8);
  color: #0f172a;
  font-size: 0.85rem;
  font-weight: 600;
  transition: all 0.2s ease;
}

.action-btn:hover {
  transform: translateY(-1px);
  background: white;
}

.action-btn.primary {
  background: linear-gradient(135deg, #2563eb, #3b82f6);
  color: white;
  border: none;
}

.action-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.dashboard-card {
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid rgba(226, 232, 240, 0.9);
  border-radius: 1rem;
  padding: 1.5rem;
  box-shadow: 0 12px 30px rgba(15, 23, 42, 0.08);
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
}

.card-header h2 {
  margin: 0;
  font-size: 1rem;
  color: #475569;
}

.card-body {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.metric-large {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.metric-label {
  font-size: 0.75rem;
  color: #94a3b8;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.metric-value {
  font-size: 1.75rem;
  font-weight: 700;
}

.metric-green {
  color: #10b981;
}

.metric-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 0.75rem;
  font-size: 0.85rem;
  color: #475569;
}

.metric-grid strong {
  display: block;
  margin-top: 0.15rem;
  font-size: 1rem;
  color: #0f172a;
}

.metric-grid.highlight strong {
  font-size: 1.05rem;
}

.metric-orange {
  color: #f97316;
}

.metric-blue {
  color: #2563eb;
}

.progress-block {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.progress-track {
  height: 0.45rem;
  border-radius: 999px;
  background: rgba(226, 232, 240, 0.7);
  overflow: hidden;
}

.progress-track.soft {
  height: 0.35rem;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, #10b981, #22d3ee);
  border-radius: inherit;
  transition: width 0.3s ease;
}

.progress-bar.accent {
  background: linear-gradient(90deg, #6366f1, #8b5cf6);
}

.progress-meta {
  font-size: 0.75rem;
  color: #64748b;
}

.calendar-meta {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 0.75rem;
  font-size: 0.85rem;
  color: #475569;
}

.calendar-meta strong {
  display: block;
  margin-top: 0.2rem;
  font-size: 1.1rem;
  color: #0f172a;
}

.calendar-actions {
  display: flex;
  gap: 0.5rem;
}

.trend-actions {
  display: flex;
  gap: 0.4rem;
}

.trend-btn {
  padding: 0.3rem 0.6rem;
  border-radius: 0.6rem;
  border: 1px solid rgba(148, 163, 184, 0.35);
  background: rgba(255, 255, 255, 0.7);
  font-size: 0.75rem;
  color: #64748b;
}

.trend-btn.active {
  background: rgba(37, 99, 235, 0.12);
  color: #2563eb;
  border-color: rgba(37, 99, 235, 0.4);
}

.state-card {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem 1.25rem;
  border-radius: 1rem;
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid rgba(226, 232, 240, 0.9);
  color: #64748b;
}

.state-error {
  border-color: rgba(248, 113, 113, 0.4);
  color: #ef4444;
  justify-content: space-between;
}

.state-loading {
  justify-content: center;
}

.ghost-link {
  background: none;
  border: none;
  color: inherit;
  font-size: 0.85rem;
  text-decoration: underline;
  cursor: pointer;
}

.loader {
  width: 1.5rem;
  height: 1.5rem;
  border-radius: 999px;
  border: 2px solid rgba(148, 163, 184, 0.4);
  border-top-color: #2563eb;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

:global(.dark) .checkin-account-dashboard {
  background: radial-gradient(circle at top left, rgba(56, 189, 248, 0.12), transparent 45%),
    radial-gradient(circle at 60% 20%, rgba(168, 85, 247, 0.18), transparent 40%),
    #0b1120;
}

:global(.dark) .dashboard-card,
:global(.dark) .state-card,
:global(.dark) .icon-button,
:global(.dark) .action-btn,
:global(.dark) .trend-btn {
  background: rgba(15, 23, 42, 0.85);
  border-color: rgba(51, 65, 85, 0.8);
  color: #e2e8f0;
}

:global(.dark) .header-title h1,
:global(.dark) .metric-grid strong,
:global(.dark) .calendar-meta strong {
  color: #f8fafc;
}

:global(.dark) .card-header h2,
:global(.dark) .metric-grid,
:global(.dark) .calendar-meta,
:global(.dark) .progress-meta,
:global(.dark) .header-sub {
  color: #94a3b8;
}

@media (max-width: 768px) {
  .dashboard-shell {
    padding: 1.5rem;
  }

  .header-sub {
    flex-direction: column;
    gap: 0.2rem;
  }

  .header-actions {
    width: 100%;
    justify-content: flex-start;
  }
}
</style>
