<template>
  <div class="checkin-account-dashboard">
    <div class="dashboard-glow" />
    <div class="dashboard-shell">
      <div class="dashboard-header">
        <div class="header-left">
          <button
            class="icon-button"
            aria-label="返回账号列表"
            title="返回账号列表"
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
        <!-- 新布局：三列 - 账号统计 | 签到统计 | 日历 -->
        <div class="dashboard-main-grid">
          <!-- 账号统计卡片 - 垂直布局 -->
          <div class="stats-card-vertical">
            <div class="vertical-header">
              <div class="stats-icon purple">
                <TrendingUp class="w-4 h-4" />
              </div>
              <h2>账号统计</h2>
            </div>
            <div class="vertical-items">
              <div class="vertical-stat">
                <div class="vertical-icon green">
                  <Wallet class="w-4 h-4" />
                </div>
                <span class="vertical-label">当前余额</span>
                <span class="vertical-value green">
                  {{ formatCurrency(dashboard.account.latest_balance, dashboard.account.balance_currency) }}
                </span>
              </div>
              <div class="vertical-stat">
                <div class="vertical-icon blue">
                  <TrendingUp class="w-4 h-4" />
                </div>
                <span class="vertical-label">总额度</span>
                <span class="vertical-value blue">
                  {{ formatCurrency(dashboard.account.total_quota, dashboard.account.balance_currency) }}
                </span>
              </div>
              <div class="vertical-stat">
                <div class="vertical-icon orange">
                  <History class="w-4 h-4" />
                </div>
                <span class="vertical-label">历史消耗</span>
                <span class="vertical-value orange">
                  {{ formatCurrency(dashboard.account.used_quota, dashboard.account.balance_currency) }}
                </span>
              </div>
            </div>
          </div>

          <!-- 签到统计卡片 - 垂直布局 -->
          <div class="stats-card-vertical">
            <div class="vertical-header">
              <div class="stats-icon orange">
                <CalendarDays class="w-4 h-4" />
              </div>
              <h2>签到统计</h2>
            </div>
            <div class="vertical-items">
              <div class="vertical-stat">
                <div class="vertical-icon orange">
                  <Flame class="w-4 h-4" />
                </div>
                <span class="vertical-label">当前连续</span>
                <span class="vertical-value orange">{{ dashboard.streak.current_streak }} <small>天</small></span>
              </div>
              <div class="vertical-stat">
                <div class="vertical-icon yellow">
                  <Trophy class="w-4 h-4" />
                </div>
                <span class="vertical-label">最长连续</span>
                <span class="vertical-value">{{ dashboard.streak.longest_streak }} <small>天</small></span>
              </div>
              <div class="vertical-stat">
                <div class="vertical-icon purple">
                  <Calendar class="w-4 h-4" />
                </div>
                <span class="vertical-label">总签到天数</span>
                <span class="vertical-value purple">{{ dashboard.streak.total_check_in_days }} <small>天</small></span>
              </div>
            </div>
            <!-- 签到率进度条 -->
            <div class="checkin-progress">
              <div class="progress-info">
                <span>签到率</span>
                <span class="progress-percent">{{ dashboard.calendar.month_stats.check_in_rate.toFixed(1) }}%</span>
              </div>
              <div class="progress-bar-track">
                <div 
                  class="progress-bar-fill"
                  :style="{ width: `${dashboard.calendar.month_stats.check_in_rate}%` }"
                />
              </div>
              <div class="progress-days">
                {{ dashboard.calendar.month_stats.checked_in_days }} / {{ dashboard.calendar.month_stats.total_days }} 天
              </div>
            </div>
          </div>

          <!-- 右侧：签到日历卡片 -->
          <div class="calendar-card">
            <div class="card-header">
              <h2>签到日历</h2>
              <div class="calendar-picker">
                <div class="calendar-nav">
                  <button
                    class="nav-btn"
                    @click="prevMonth"
                  >
                    ‹
                  </button>
                  <span class="calendar-month">{{ calendarYear }}年{{ calendarMonth }}月</span>
                  <button
                    class="nav-btn"
                    @click="nextMonth"
                  >
                    ›
                  </button>
                </div>
              </div>
            </div>
            <AccountDashboardCalendar :calendar="dashboard.calendar" />
          </div>
        </div>

        <!-- 签到趋势卡片 -->
        <div class="trend-card">
          <div class="trend-header">
            <div class="trend-title">
              <h2>签到趋势</h2>
              <span class="trend-tag">近 {{ trendDays }} 天</span>
            </div>
            <div class="trend-actions">
              <button
                v-for="option in trendOptions"
                :key="option"
                class="trend-btn"
                :class="{ active: trendDays === option }"
                @click="trendDays = option"
              >
                {{ option }}
              </button>
            </div>
          </div>
          <div class="trend-body">
            <AccountDashboardTrend :trend="dashboard.trend" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft, CheckCircle2, RefreshCw, Wallet, TrendingUp, History, CalendarDays, Flame, Trophy, Calendar } from 'lucide-vue-next'
import { checkinAccount, getCheckinAccountDashboard, queryCheckinBalance } from '@/api/client'
import type { CheckinAccountDashboardResponse } from '@/types/checkin'
import { extractStringParam } from '@/types/router'
import AccountDashboardCalendar from './components/AccountDashboardCalendar.vue'
import AccountDashboardTrend from './components/AccountDashboardTrend.vue'

const route = useRoute()
const router = useRouter()

const accountId = computed(() => extractStringParam(route.params.accountId) || '')
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

const accountEnabled = computed(() => dashboard.value?.account.enabled ?? false)

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
  background: radial-gradient(circle at top left, rgb(var(--color-info-rgb), 0.15), transparent 45%),
    radial-gradient(circle at 20% 20%, rgb(var(--color-success-rgb), 0.12), transparent 40%),
    radial-gradient(circle at 80% 0%, rgb(var(--color-accent-secondary-rgb), 0.15), transparent 40%),
    var(--bg-secondary);
  overflow: hidden;
}

.dashboard-glow {
  position: absolute;
  top: -120px;
  right: -80px;
  width: 280px;
  height: 280px;
  background: radial-gradient(circle, rgb(var(--color-accent-primary-rgb), 0.25), transparent 70%);
  filter: blur(10px);
  pointer-events: none;
}

.dashboard-shell {
  position: relative;
  z-index: 1;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  max-width: 1600px;
  margin: 0 auto;
  width: 100%;
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
  color: var(--text-primary);
}

.header-sub {
  display: flex;
  gap: 1rem;
  font-size: 0.8rem;
  color: var(--text-muted);
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
  background: rgb(var(--color-accent-primary-rgb), 0.1);
  color: var(--platform-gemini);
}

.status-pill.status-on {
  background: rgb(var(--color-success-rgb), 0.15);
  color: var(--accent-success);
}

.status-pill.status-off {
  background: rgb(var(--color-danger-rgb), 0.15);
  color: var(--accent-danger);
}

.icon-button {
  height: 2.25rem;
  width: 2.25rem;
  border-radius: 999px;
  border: 1px solid rgb(var(--color-gray-rgb), 0.35);
  background: var(--glass-bg-heavy);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  transition: all 0.2s ease;
}

.icon-button:hover {
  background: var(--bg-primary);
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
  border: 1px solid rgb(var(--color-gray-rgb), 0.35);
  background: var(--glass-bg-heavy);
  color: var(--text-primary);
  font-size: 0.85rem;
  font-weight: 600;
  transition: all 0.2s ease;
}

.action-btn:hover {
  transform: translateY(-1px);
  background: var(--bg-primary);
}

.action-btn.primary {
  background: var(--gradient-secondary);
  color: white;
  border: none;
}

.action-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.dashboard-card {
  background: var(--glass-bg-heavy);
  border: 1px solid var(--glass-border-medium);
  border-radius: 1rem;
  padding: 1.5rem;
  box-shadow: var(--shadow-medium);
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
  color: var(--text-secondary);
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
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.metric-value {
  font-size: 1.75rem;
  font-weight: 700;
}

.metric-green {
  color: var(--accent-success);
}

.metric-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 0.75rem;
  font-size: 0.85rem;
  color: var(--text-secondary);
}

.metric-grid strong {
  display: block;
  margin-top: 0.15rem;
  font-size: 1rem;
  color: var(--text-primary);
}

.metric-grid.highlight strong {
  font-size: 1.05rem;
}

.metric-orange {
  color: var(--accent-warning);
}

.metric-blue {
  color: var(--platform-gemini);
}

.progress-block {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.progress-track {
  height: 0.45rem;
  border-radius: 999px;
  background: rgb(var(--color-gray-rgb), 0.2);
  overflow: hidden;
}

.progress-track.soft {
  height: 0.35rem;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, var(--accent-success), var(--platform-iflow));
  border-radius: inherit;
  transition: width 0.3s ease;
}

.progress-bar.accent {
  background: var(--gradient-secondary);
}

.progress-meta {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.calendar-meta {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 0.75rem;
  font-size: 0.85rem;
  color: var(--text-secondary);
}

.calendar-meta strong {
  display: block;
  margin-top: 0.2rem;
  font-size: 1.1rem;
  color: var(--text-primary);
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
  border: 1px solid rgb(var(--color-gray-rgb), 0.35);
  background: var(--glass-bg-medium);
  font-size: 0.75rem;
  color: var(--text-muted);
}

.trend-btn.active {
  background: rgb(var(--color-info-rgb), 0.12);
  color: var(--platform-gemini);
  border-color: rgb(var(--color-info-rgb), 0.4);
}

.state-card {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem 1.25rem;
  border-radius: 1rem;
  background: var(--glass-bg-heavy);
  border: 1px solid var(--glass-border-medium);
  color: var(--text-muted);
}

.state-error {
  border-color: rgb(var(--color-danger-rgb), 0.4);
  color: var(--accent-danger);
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
  border: 2px solid rgb(var(--color-gray-rgb), 0.4);
  border-top-color: var(--platform-gemini);
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

:global(.dark) .checkin-account-dashboard {
  background: radial-gradient(circle at top left, rgb(var(--color-cyan-rgb), 0.12), transparent 45%),
    radial-gradient(circle at 60% 20%, rgb(var(--color-accent-secondary-rgb), 0.18), transparent 40%),
    var(--bg-primary);
}

:global(.dark) .dashboard-card,
:global(.dark) .state-card,
:global(.dark) .icon-button,
:global(.dark) .action-btn,
:global(.dark) .trend-btn {
  background: rgb(var(--color-slate-dark-rgb), 0.85);
  border-color: rgb(var(--color-slate-rgb), 0.8);
  color: var(--text-secondary);
}

:global(.dark) .header-title h1,
:global(.dark) .metric-grid strong,
:global(.dark) .calendar-meta strong {
  color: var(--text-primary);
}

:global(.dark) .card-header h2,
:global(.dark) .metric-grid,
:global(.dark) .calendar-meta,
:global(.dark) .progress-meta,
:global(.dark) .header-sub {
  color: var(--text-muted);
}

/* 横排统计卡片样式 */
.stats-card-row {
  background: var(--glass-bg-heavy);
  border: 1px solid var(--glass-border-medium);
  border-radius: 1rem;
  padding: 1rem 1.25rem;
  box-shadow: var(--shadow-small);
}

.row-header {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  margin-bottom: 0.75rem;
  padding-bottom: 0.6rem;
  border-bottom: 1px solid rgb(var(--color-gray-rgb), 0.2);
}

.row-header h2 {
  margin: 0;
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--text-secondary);
  flex: 1;
}

.row-rate {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--accent-success);
  background: rgb(var(--color-success-rgb), 0.1);
  padding: 0.2rem 0.5rem;
  border-radius: 0.4rem;
}

.row-items {
  display: flex;
  gap: 1rem;
}

.row-stat {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex: 1;
}

.row-icon {
  width: 2rem;
  height: 2rem;
  border-radius: 0.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.row-icon.purple {
  background: rgb(var(--color-accent-secondary-rgb), 0.12);
  color: var(--platform-claude);
}

.row-icon.green {
  background: rgb(var(--color-success-rgb), 0.12);
  color: var(--accent-success);
}

.row-icon.blue {
  background: rgb(var(--color-info-rgb), 0.12);
  color: var(--platform-gemini);
}

.row-icon.orange {
  background: rgb(var(--color-warning-rgb), 0.12);
  color: var(--accent-warning);
}

.row-icon.yellow {
  background: rgb(var(--color-warning-rgb), 0.12);
  color: var(--platform-codex);
}

.row-content {
  display: flex;
  flex-direction: column;
}

.row-label {
  font-size: 0.7rem;
  color: var(--text-muted);
}

.row-value {
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--text-primary);
}

.row-value.green { color: var(--accent-success); }
.row-value.blue { color: var(--platform-gemini); }
.row-value.orange { color: var(--accent-warning); }
.row-value.purple { color: var(--platform-claude); }

.row-value small {
  font-size: 0.65rem;
  font-weight: 500;
  color: var(--text-muted);
}

/* 月份选择器 */
.calendar-picker {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.month-input {
  font-size: 0.8rem;
  padding: 0.25rem 0.5rem;
  border: 1px solid rgb(var(--color-gray-rgb), 0.35);
  border-radius: 0.5rem;
  background: var(--glass-bg-heavy);
  color: var(--text-secondary);
  cursor: pointer;
}

.month-input:focus {
  outline: none;
  border-color: var(--platform-gemini);
}

.calendar-nav {
  display: flex;
  gap: 0.25rem;
}

:global(.dark) .stats-card-row {
  background: rgb(var(--color-slate-dark-rgb), 0.9);
  border-color: rgb(var(--color-slate-rgb), 0.8);
}

:global(.dark) .row-header {
  border-color: rgb(var(--color-slate-rgb), 0.6);
}

:global(.dark) .row-header h2 {
  color: var(--text-secondary);
}

:global(.dark) .row-value {
  color: var(--text-primary);
}

:global(.dark) .row-label {
  color: var(--text-muted);
}

:global(.dark) .month-input {
  background: rgb(var(--color-slate-dark-rgb), 0.8);
  border-color: rgb(var(--color-slate-rgb), 0.8);
  color: var(--text-secondary);
}

/* 新版统计卡片样式 */
.stats-card {
  background: var(--glass-bg-heavy);
  border: 1px solid var(--glass-border-medium);
  border-radius: 1rem;
  padding: 1.25rem;
  box-shadow: var(--shadow-small);
}

.stats-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 1rem;
  padding-bottom: 0.75rem;
  border-bottom: 1px solid rgb(var(--color-gray-rgb), 0.2);
}

.stats-header h2 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-secondary);
}

.stats-icon {
  width: 2rem;
  height: 2rem;
  border-radius: 0.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
}

.stats-icon.purple {
  background: rgb(var(--color-accent-secondary-rgb), 0.12);
  color: var(--platform-claude);
}

.stats-icon.green {
  background: rgb(var(--color-success-rgb), 0.12);
  color: var(--accent-success);
}

.stats-icon.blue {
  background: rgb(var(--color-info-rgb), 0.12);
  color: var(--platform-gemini);
}

.stats-icon.orange {
  background: rgb(var(--color-warning-rgb), 0.12);
  color: var(--accent-warning);
}

.stats-icon.yellow {
  background: rgb(var(--color-warning-rgb), 0.12);
  color: var(--platform-codex);
}

.stats-body {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.stat-icon {
  width: 1.75rem;
  height: 1.75rem;
  border-radius: 0.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.stat-content {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.stat-label {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.stat-value {
  font-size: 1.35rem;
  font-weight: 700;
  color: var(--text-primary);
}

.stat-value.green { color: var(--accent-success); }
.stat-value.blue { color: var(--platform-gemini); }
.stat-value.orange { color: var(--accent-warning); }
.stat-value.purple { color: var(--platform-claude); }

.stat-value small {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--text-muted);
}

.stat-progress {
  margin-top: 0.5rem;
  padding-top: 0.75rem;
  border-top: 1px solid rgb(var(--color-gray-rgb), 0.2);
}

.stat-progress-header {
  display: flex;
  justify-content: space-between;
  font-size: 0.8rem;
  color: var(--text-secondary);
  margin-bottom: 0.4rem;
}

.stat-progress-footer {
  font-size: 0.75rem;
  color: var(--text-muted);
  margin-top: 0.35rem;
}

:global(.dark) .stats-card {
  background: rgb(var(--color-slate-dark-rgb), 0.9);
  border-color: rgb(var(--color-slate-rgb), 0.8);
}

:global(.dark) .stats-header {
  border-color: rgb(var(--color-slate-rgb), 0.6);
}

:global(.dark) .stats-header h2 {
  color: var(--text-secondary);
}

:global(.dark) .stat-value {
  color: var(--text-primary);
}

:global(.dark) .stat-label,
:global(.dark) .stat-progress-header,
:global(.dark) .stat-progress-footer {
  color: var(--text-muted);
}

/* 新布局样式 - 三列布局 */
.dashboard-main-grid {
  display: grid;
  grid-template-columns: 1fr 1fr 2fr;
  gap: 1.25rem;
  width: 100%;
}

/* 垂直统计卡片样式 */
.stats-card-vertical {
  background: var(--glass-bg-heavy);
  border: 1px solid var(--glass-border-medium);
  border-radius: 1rem;
  padding: 1.5rem;
  box-shadow: var(--shadow-small);
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  height: 100%;
  justify-content: space-between;
}

.vertical-header {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  margin-bottom: 1.25rem;
  width: 100%;
}

.vertical-header h2 {
  margin: 0;
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--text-secondary);
}

.vertical-items {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  flex: 1;
  width: 100%;
  justify-content: center;
}

.vertical-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.15rem;
}

.vertical-icon {
  width: 1.5rem;
  height: 1.5rem;
  border-radius: 0.375rem;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 0.15rem;
}

.vertical-icon.purple {
  background: rgb(var(--color-accent-secondary-rgb), 0.12);
  color: var(--platform-claude);
}

.vertical-icon.green {
  background: rgb(var(--color-success-rgb), 0.12);
  color: var(--accent-success);
}

.vertical-icon.blue {
  background: rgb(var(--color-info-rgb), 0.12);
  color: var(--platform-gemini);
}

.vertical-icon.orange {
  background: rgb(var(--color-warning-rgb), 0.12);
  color: var(--accent-warning);
}

.vertical-icon.yellow {
  background: rgb(var(--color-warning-rgb), 0.12);
  color: var(--platform-codex);
}

.vertical-label {
  font-size: 0.7rem;
  color: var(--text-muted);
}

.vertical-value {
  font-size: 1.35rem;
  font-weight: 700;
  color: var(--text-primary);
}

.vertical-value.green { color: var(--accent-success); }
.vertical-value.blue { color: var(--platform-gemini); }
.vertical-value.orange { color: var(--accent-warning); }
.vertical-value.purple { color: var(--platform-claude); }

.vertical-value small {
  font-size: 0.8rem;
  font-weight: 500;
  color: var(--text-muted);
}

/* 签到进度条 */
.checkin-progress {
  margin-top: auto;
  padding-top: 0.75rem;
  border-top: 1px solid rgb(var(--color-gray-rgb), 0.2);
  width: 100%;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  font-size: 0.7rem;
  color: var(--text-muted);
  margin-bottom: 0.4rem;
}

.progress-percent {
  font-weight: 600;
  color: var(--accent-success);
}

.progress-bar-track {
  height: 5px;
  background: rgb(var(--color-gray-rgb), 0.3);
  border-radius: 3px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--platform-gemini), var(--accent-info));
  border-radius: 3px;
  transition: width 0.3s ease;
}

.progress-days {
  font-size: 0.65rem;
  color: var(--text-muted);
  margin-top: 0.25rem;
  text-align: center;
}

/* 日历卡片样式 */
.calendar-card {
  background: var(--glass-bg-heavy);
  border: 1px solid var(--glass-border-medium);
  border-radius: 1rem;
  padding: 1.25rem;
  box-shadow: var(--shadow-small);
  display: flex;
  flex-direction: column;
}

.calendar-card .card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.calendar-card .card-header h2 {
  margin: 0;
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--text-secondary);
}

.calendar-card .calendar-picker {
  display: flex;
  align-items: center;
}

.calendar-card .calendar-nav {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.nav-btn {
  width: 1.75rem;
  height: 1.75rem;
  border-radius: 0.375rem;
  border: 1px solid rgb(var(--color-gray-rgb), 0.35);
  background: var(--glass-bg-heavy);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  font-size: 1rem;
  cursor: pointer;
  transition: all 0.15s ease;
}

.nav-btn:hover {
  background: var(--bg-primary);
  border-color: var(--platform-gemini);
  color: var(--platform-gemini);
}

.calendar-month {
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--text-secondary);
  min-width: 5.5rem;
  text-align: center;
}

/* 趋势卡片优化样式 */
.trend-card {
  background: var(--glass-bg-heavy);
  border: 1px solid var(--glass-border-medium);
  border-radius: 1rem;
  padding: 1.25rem;
  box-shadow: var(--shadow-small);
}

.trend-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
  padding-bottom: 0.75rem;
  border-bottom: 1px solid rgb(var(--color-gray-rgb), 0.2);
}

.trend-title {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.trend-title h2 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-secondary);
}

.trend-tag {
  display: inline-flex;
  align-items: center;
  padding: 0.2rem 0.6rem;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 600;
  background: rgb(var(--color-accent-primary-rgb), 0.1);
  color: var(--platform-gemini);
}

.trend-body {
  min-height: 200px;
}

:global(.dark) .stats-card-vertical,
:global(.dark) .calendar-card,
:global(.dark) .trend-card {
  background: rgb(var(--color-slate-dark-rgb), 0.9);
  border-color: rgb(var(--color-slate-rgb), 0.8);
}

:global(.dark) .vertical-header h2,
:global(.dark) .calendar-card .card-header h2,
:global(.dark) .trend-title h2 {
  color: var(--text-secondary);
}

:global(.dark) .vertical-value {
  color: var(--text-primary);
}

:global(.dark) .vertical-label,
:global(.dark) .progress-info,
:global(.dark) .progress-days {
  color: var(--text-muted);
}

:global(.dark) .calendar-month {
  color: var(--text-secondary);
}

:global(.dark) .nav-btn {
  background: rgb(var(--color-slate-dark-rgb), 0.8);
  border-color: rgb(var(--color-slate-rgb), 0.8);
  color: var(--text-secondary);
}

:global(.dark) .nav-btn:hover {
  background: rgb(var(--color-slate-rgb), 0.9);
  border-color: var(--platform-gemini);
  color: var(--platform-gemini);
}

:global(.dark) .checkin-progress {
  border-color: rgb(var(--color-slate-rgb), 0.6);
}

:global(.dark) .progress-bar-track {
  background: rgb(var(--color-slate-rgb), 0.6);
}

:global(.dark) .trend-header {
  border-color: rgb(var(--color-slate-rgb), 0.6);
}

@media (width <= 1280px) {
  .dashboard-main-grid {
    grid-template-columns: 1fr 1fr;
  }
  
  .calendar-card {
    grid-column: span 2;
  }
}

@media (width <= 768px) {
  .dashboard-main-grid {
    grid-template-columns: 1fr;
    gap: 1rem;
  }
  
  .calendar-card {
    grid-column: span 1;
  }

  .dashboard-shell {
    padding: 1rem;
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
