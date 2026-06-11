<template>
  <div class="checkin-account-dashboard">
    <div class="dashboard-shell">
      <section class="dashboard-header checkin-surface-card">
        <div class="header-left">
          <button
            type="button"
            class="icon-button"
            aria-label="返回账号列表"
            title="返回账号列表"
            @click="goBack"
          >
            <SIcon
              name="ArrowLeft"
              size="w-4 h-4"
            />
          </button>

          <div class="header-copy">
            <p class="header-eyebrow">
              签到账号 · Dashboard
            </p>
            <div class="header-title-row">
              <h1>{{ dashboard?.account.name || '账号 Dashboard' }}</h1>
              <span class="provider-pill">
                {{ dashboard?.account.provider_name || '未知提供商' }}
              </span>
              <span
                v-if="dashboard"
                class="status-pill"
                :class="accountEnabled ? 'status-on' : 'status-off'"
              >
                {{ accountEnabled ? '启用' : '已禁用' }}
              </span>
            </div>

            <div class="header-meta">
              <span class="meta-chip">
                <SIcon
                  name="CalendarDays"
                  size="w-3.5 h-3.5"
                />
                最后签到：{{ dashboard?.streak.last_check_in_date || '-' }}
              </span>
              <span class="meta-chip">
                <SIcon
                  name="Wallet"
                  size="w-3.5 h-3.5"
                />
                余额更新：{{ formatDateTime(dashboard?.account.last_balance_check_at) }}
              </span>
            </div>
          </div>
        </div>

        <div class="header-actions">
          <button
            type="button"
            class="action-btn"
            :disabled="loading || !dashboard || checkinLoading"
            @click="handleCheckin"
          >
            <SIcon
              name="CheckCircle2"
              size="w-4 h-4"
            />
            签到
          </button>
          <button
            type="button"
            class="action-btn"
            :disabled="loading || !dashboard || balanceLoading"
            @click="handleBalanceRefresh"
          >
            <SIcon
              name="Wallet"
              size="w-4 h-4"
            />
            刷新余额
          </button>
          <button
            type="button"
            class="action-btn primary"
            :disabled="loading"
            @click="loadDashboard"
          >
            <SIcon
              name="RefreshCw"
              size="w-4 h-4"
              :class="{ 'animate-spin': loading }"
            />
            刷新
          </button>
        </div>
      </section>

      <div
        v-if="error"
        class="state-card checkin-surface-card state-error"
      >
        <p>{{ error }}</p>
        <button
          type="button"
          class="ghost-link"
          @click="loadDashboard"
        >
          重试
        </button>
      </div>

      <div
        v-else-if="loading"
        class="state-card checkin-surface-card state-loading"
      >
        <div class="loader" />
        加载中...
      </div>

      <div
        v-else-if="dashboard"
        class="dashboard-stack"
      >
        <div class="dashboard-main-grid">
          <section class="stats-card-vertical checkin-surface-card">
            <div class="card-lead">
              <div class="stats-icon accent">
                <SIcon
                  name="TrendingUp"
                  size="w-4 h-4"
                />
              </div>
              <div class="card-copy">
                <p class="card-overline">
                  Account overview
                </p>
                <h2>账号统计</h2>
              </div>
            </div>

            <div class="vertical-items">
              <article class="vertical-stat">
                <div class="vertical-icon success">
                  <SIcon
                    name="Wallet"
                    size="w-4 h-4"
                  />
                </div>
                <div class="vertical-copy">
                  <span class="vertical-label">当前余额</span>
                  <span class="vertical-value success">
                    {{ formatCurrency(dashboard.account.latest_balance, dashboard.account.balance_currency) }}
                  </span>
                </div>
              </article>

              <article class="vertical-stat">
                <div class="vertical-icon accent">
                  <SIcon
                    name="TrendingUp"
                    size="w-4 h-4"
                  />
                </div>
                <div class="vertical-copy">
                  <span class="vertical-label">总额度</span>
                  <span class="vertical-value accent">
                    {{ formatCurrency(dashboard.account.total_quota, dashboard.account.balance_currency) }}
                  </span>
                </div>
              </article>

              <article class="vertical-stat">
                <div class="vertical-icon warning">
                  <SIcon
                    name="History"
                    size="w-4 h-4"
                  />
                </div>
                <div class="vertical-copy">
                  <span class="vertical-label">历史消耗</span>
                  <span class="vertical-value warning">
                    {{ formatCurrency(dashboard.account.used_quota, dashboard.account.balance_currency) }}
                  </span>
                </div>
              </article>
            </div>
          </section>

          <section class="stats-card-vertical checkin-surface-card">
            <div class="card-lead">
              <div class="stats-icon warning">
                <SIcon
                  name="CalendarDays"
                  size="w-4 h-4"
                />
              </div>
              <div class="card-copy">
                <p class="card-overline">
                  Streak snapshot
                </p>
                <h2>签到统计</h2>
              </div>
            </div>

            <div class="vertical-items">
              <article class="vertical-stat">
                <div class="vertical-icon warning">
                  <SIcon
                    name="Flame"
                    size="w-4 h-4"
                  />
                </div>
                <div class="vertical-copy">
                  <span class="vertical-label">当前连续</span>
                  <span class="vertical-value warning">
                    {{ dashboard.streak.current_streak }} <small>天</small>
                  </span>
                </div>
              </article>

              <article class="vertical-stat">
                <div class="vertical-icon accent">
                  <SIcon
                    name="Trophy"
                    size="w-4 h-4"
                  />
                </div>
                <div class="vertical-copy">
                  <span class="vertical-label">最长连续</span>
                  <span class="vertical-value accent">
                    {{ dashboard.streak.longest_streak }} <small>天</small>
                  </span>
                </div>
              </article>

              <article class="vertical-stat">
                <div class="vertical-icon success">
                  <SIcon
                    name="Calendar"
                    size="w-4 h-4"
                  />
                </div>
                <div class="vertical-copy">
                  <span class="vertical-label">总签到天数</span>
                  <span class="vertical-value success">
                    {{ dashboard.streak.total_check_in_days }} <small>天</small>
                  </span>
                </div>
              </article>
            </div>

            <div class="checkin-progress">
              <div class="progress-info">
                <span>本月签到率</span>
                <span class="progress-percent">
                  {{ dashboard.calendar.month_stats.check_in_rate.toFixed(1) }}%
                </span>
              </div>
              <div class="progress-bar-track">
                <div
                  class="progress-bar-fill"
                  :style="{ transform: `scaleX(${Math.min(dashboard.calendar.month_stats.check_in_rate, 100) / 100})` }"
                />
              </div>
              <div class="progress-days">
                {{ dashboard.calendar.month_stats.checked_in_days }} / {{ dashboard.calendar.month_stats.total_days }} 天
              </div>
            </div>
          </section>

          <section class="calendar-card checkin-surface-card">
            <div class="card-header">
              <div class="card-copy">
                <p class="card-overline">
                  Monthly history
                </p>
                <h2>签到日历</h2>
              </div>

              <div class="calendar-nav">
                <button
                  type="button"
                  class="nav-btn"
                  aria-label="上个月"
                  @click="prevMonth"
                >
                  ‹
                </button>
                <span class="calendar-month">{{ calendarYear }}年{{ calendarMonth }}月</span>
                <button
                  type="button"
                  class="nav-btn"
                  aria-label="下个月"
                  @click="nextMonth"
                >
                  ›
                </button>
              </div>
            </div>

            <AccountDashboardCalendar :calendar="dashboard.calendar" />
          </section>
        </div>

        <section class="trend-card checkin-surface-card">
          <div class="trend-header">
            <div class="card-copy">
              <p class="card-overline">
                Rolling changes
              </p>
              <div class="trend-title-row">
                <h2>签到趋势</h2>
                <span class="trend-tag">近 {{ trendDays }} 天</span>
              </div>
            </div>

            <div class="trend-actions">
              <button
                v-for="option in trendOptions"
                :key="option"
                type="button"
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
        </section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { checkinAccount, getCheckinAccountDashboard, queryCheckinBalance } from '@/api'
import type { BalanceSnapshot, CheckinAccountDashboardResponse } from '@/types/checkin'
import { extractStringParam } from '@/types/router'
import { getErrorMessage } from '@/types/api'
import { useUIStore } from '@/stores/ui'
import AccountDashboardCalendar from './components/AccountDashboardCalendar.vue'
import AccountDashboardTrend from './components/AccountDashboardTrend.vue'

const route = useRoute()
const router = useRouter()
const uiStore = useUIStore()

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
  } catch (currentError: unknown) {
    error.value = getErrorMessage(currentError, '加载失败')
  } finally {
    loading.value = false
  }
}

const handleCheckin = async () => {
  if (!accountId.value) return
  checkinLoading.value = true

  try {
    const result = await checkinAccount(accountId.value)
    const label =
      result.status === 'success'
        ? '签到成功'
        : result.status === 'already_checked_in'
          ? '今日已签到'
          : '签到失败'
    const message = result.message ? `${label}：${result.message}` : label
    if (result.status === 'failed') {
      uiStore.showError(message)
    } else {
      uiStore.showSuccess(message)
    }
    await loadDashboard()
  } catch (currentError: unknown) {
    uiStore.showError('签到失败：' + getErrorMessage(currentError, '未知错误'))
  } finally {
    checkinLoading.value = false
  }
}

const handleBalanceRefresh = async () => {
  if (!accountId.value) return
  balanceLoading.value = true

  try {
    const result = await queryCheckinBalance<BalanceSnapshot>(accountId.value)
    uiStore.showSuccess(
      `余额：${result.currency}${result.remaining_quota.toFixed(2)}（已用 ${result.usage_percentage.toFixed(1)}%）`
    )
    await loadDashboard()
  } catch (currentError: unknown) {
    uiStore.showError('刷新余额失败：' + getErrorMessage(currentError, '未知错误'))
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
  min-height: 100%;
  background: var(--color-bg-base);
}

.dashboard-shell {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  width: 100%;
  max-width: 1520px;
  margin: 0 auto;
  padding: 1.5rem;
}

.dashboard-stack {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.dashboard-header {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 1rem 1.5rem;
  padding: 1.25rem 1.5rem;
}

.header-left {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
  min-width: 0;
}

.header-copy {
  display: grid;
  gap: 0.5rem;
  min-width: 0;
}

.header-eyebrow {
  margin: 0;
  color: var(--text-muted);
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.header-title-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.65rem;
}

.header-title-row h1 {
  margin: 0;
  color: var(--text-primary);
  font-size: clamp(1.45rem, 1.75vw, 1.8rem);
  font-weight: 700;
  letter-spacing: -0.02em;
}

.header-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.provider-pill,
.status-pill,
.meta-chip,
.trend-tag {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 600;
  line-height: 1;
  white-space: nowrap;
}

.provider-pill,
.status-pill,
.trend-tag {
  padding: 0.4rem 0.75rem;
}

.provider-pill {
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 22%);
  color: var(--accent-primary);
}

.status-pill.status-on {
  background: rgb(var(--color-success-rgb) / 12%);
  border: 1px solid rgb(var(--color-success-rgb) / 24%);
  color: var(--accent-success);
}

.status-pill.status-off {
  background: rgb(var(--color-danger-rgb) / 12%);
  border: 1px solid rgb(var(--color-danger-rgb) / 22%);
  color: var(--accent-danger);
}

.meta-chip {
  padding: 0.45rem 0.75rem;
  background: rgb(var(--color-bg-elevated-rgb) / 54%);
  border: 1px solid rgb(var(--color-border-default-rgb) / 56%);
  color: var(--text-secondary);
}

.icon-button,
.action-btn,
.nav-btn,
.trend-btn {
  border: 1px solid rgb(var(--color-border-default-rgb) / 56%);
  transition:
    border-color 0.2s ease,
    background-color 0.2s ease,
    box-shadow 0.2s ease,
    color 0.2s ease,
    transform 0.2s ease;
}

.icon-button {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 2.4rem;
  height: 2.4rem;
  border-radius: 999px;
  background: rgb(var(--color-bg-elevated-rgb) / 48%);
  color: var(--text-secondary);
  cursor: pointer;
}

.icon-button:hover {
  transform: translateY(-1px);
  background: rgb(var(--color-bg-elevated-rgb) / 78%);
  color: var(--text-primary);
}

.header-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.6rem;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.45rem;
  min-height: 2.6rem;
  padding: 0.6rem 1rem;
  border-radius: 0.85rem;
  background: rgb(var(--color-bg-elevated-rgb) / 48%);
  color: var(--text-primary);
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
}

.action-btn:hover:not(:disabled),
.nav-btn:hover,
.trend-btn:hover {
  transform: translateY(-1px);
  background: rgb(var(--color-bg-elevated-rgb) / 78%);
  border-color: rgb(var(--color-border-default-rgb) / 82%);
}

.action-btn.primary {
  border-color: transparent;
  background: var(--accent-primary);
  color: white;
  box-shadow: 0 10px 22px rgb(var(--color-accent-primary-rgb) / 24%);
}

.action-btn.primary:hover:not(:disabled) {
  background: rgb(var(--color-accent-primary-rgb) / 92%);
}

.action-btn:disabled,
.nav-btn:disabled,
.trend-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

.state-card {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  min-height: 6rem;
  padding: 1.05rem 1.35rem;
  color: var(--text-secondary);
}

.state-card p {
  margin: 0;
}

.state-error {
  justify-content: space-between;
  color: var(--accent-danger);
  border-color: rgb(var(--color-danger-rgb) / 34%);
}

.state-loading {
  color: var(--text-primary);
}

.ghost-link {
  padding: 0;
  border: none;
  background: none;
  color: inherit;
  cursor: pointer;
  font-size: 0.85rem;
  font-weight: 600;
  text-decoration: underline;
  text-underline-offset: 0.18em;
}

.loader {
  width: 1.4rem;
  height: 1.4rem;
  border: 2px solid rgb(var(--color-border-default-rgb) / 55%);
  border-top-color: var(--accent-primary);
  border-radius: 999px;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.dashboard-main-grid {
  display: grid;
  grid-template-columns: minmax(17rem, 0.9fr) minmax(17rem, 0.9fr) minmax(25rem, 1.4fr);
  gap: 1.1rem;
}

.stats-card-vertical,
.calendar-card,
.trend-card {
  padding: 1.25rem;
}

.stats-card-vertical {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.card-lead,
.card-header,
.trend-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.card-copy {
  display: grid;
  gap: 0.2rem;
}

.card-overline {
  margin: 0;
  color: var(--text-muted);
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.card-copy h2,
.trend-title-row h2 {
  margin: 0;
  color: var(--text-primary);
  font-size: 1rem;
  font-weight: 700;
}

.stats-icon,
.vertical-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 0.85rem;
  flex-shrink: 0;
}

.stats-icon {
  width: 2.3rem;
  height: 2.3rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 56%);
}

.vertical-icon {
  width: 2.5rem;
  height: 2.5rem;
}

.stats-icon.accent,
.vertical-icon.accent {
  background: rgb(var(--color-accent-primary-rgb) / 12%);
  color: var(--accent-primary);
}

.stats-icon.success,
.vertical-icon.success {
  background: rgb(var(--color-success-rgb) / 12%);
  color: var(--accent-success);
}

.stats-icon.warning,
.vertical-icon.warning {
  background: rgb(var(--color-warning-rgb) / 14%);
  color: var(--accent-warning);
}

.vertical-items {
  display: flex;
  flex-direction: column;
  gap: 0.7rem;
}

.vertical-stat {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 0.85rem;
  align-items: center;
  padding: 0.75rem 0.9rem;
  border-radius: 0.95rem;
  background: rgb(var(--color-bg-elevated-rgb) / 48%);
  border: 1px solid rgb(var(--color-border-default-rgb) / 56%);
}

.vertical-copy {
  display: grid;
  gap: 0.2rem;
  min-width: 0;
}

.vertical-label {
  color: var(--text-muted);
  font-size: 0.72rem;
  font-weight: 600;
}

.vertical-value {
  color: var(--text-primary);
  font-size: 1.25rem;
  font-weight: 700;
  line-height: 1.1;
  letter-spacing: -0.02em;
  font-family: var(--font-mono);
}

.vertical-value.success {
  color: var(--accent-success);
}

.vertical-value.accent {
  color: var(--accent-primary);
}

.vertical-value.warning {
  color: var(--accent-warning);
}

.vertical-value small {
  color: var(--text-secondary);
  font-size: 0.8rem;
  font-weight: 600;
}

.checkin-progress {
  margin-top: auto;
  padding: 0.9rem 0 0;
  border-top: 1px solid rgb(var(--color-border-default-rgb) / 56%);
}

.progress-info,
.progress-days {
  color: var(--text-secondary);
  font-size: 0.74rem;
}

.progress-info {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.4rem;
}

.progress-percent {
  color: var(--accent-primary);
  font-weight: 700;
  font-family: var(--font-mono);
}

.progress-bar-track {
  height: 0.38rem;
  overflow: hidden;
  border-radius: 999px;
  background: rgb(var(--color-border-default-rgb) / 46%);
}

.progress-bar-fill {
  width: 100%;
  height: 100%;
  transform-origin: left center;
  background: var(--accent-primary);
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.progress-days {
  margin-top: 0.35rem;
}

.calendar-card {
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
}

.calendar-nav {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.25rem;
  border-radius: 999px;
  background: rgb(var(--color-bg-elevated-rgb) / 48%);
  border: 1px solid rgb(var(--color-border-default-rgb) / 56%);
}

.nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.85rem;
  height: 1.85rem;
  padding: 0;
  border: none;
  border-radius: 999px;
  background: transparent;
  color: var(--text-primary);
  font-size: 1rem;
  cursor: pointer;
}

.nav-btn:hover {
  background: rgb(var(--color-bg-elevated-rgb) / 88%);
}

.calendar-month {
  min-width: 6rem;
  color: var(--text-primary);
  font-size: 0.82rem;
  font-weight: 700;
  text-align: center;
}

.trend-card {
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
}

.trend-header {
  padding-bottom: 0.85rem;
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 56%);
}

.trend-title-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.7rem;
}

.trend-tag {
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 22%);
  color: var(--accent-primary);
}

.trend-actions {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.25rem;
  border-radius: 999px;
  background: rgb(var(--color-bg-elevated-rgb) / 48%);
  border: 1px solid rgb(var(--color-border-default-rgb) / 56%);
}

.trend-btn {
  min-width: 2.6rem;
  padding: 0.38rem 0.75rem;
  border: none;
  border-radius: 999px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 0.74rem;
  font-weight: 700;
  cursor: pointer;
}

.trend-btn.active {
  background: rgb(var(--color-bg-elevated-rgb) / 92%);
  color: var(--accent-primary);
  box-shadow: 0 2px 6px rgb(15 23 42 / 8%);
}

.trend-body {
  min-height: 15rem;
}

@media (width <= 1280px) {
  .dashboard-main-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .calendar-card {
    grid-column: 1 / -1;
  }
}

@media (width <= 960px) {
  .dashboard-header {
    padding: 1rem;
  }

  .header-actions {
    width: 100%;
  }

  .action-btn {
    flex: 1 1 12rem;
  }

  .trend-header {
    align-items: flex-start;
    flex-direction: column;
  }
}

@media (width <= 768px) {
  .dashboard-shell {
    padding: 1rem;
  }

  .dashboard-main-grid {
    grid-template-columns: 1fr;
    gap: 1rem;
  }

  .calendar-card {
    grid-column: auto;
  }

  .header-left {
    width: 100%;
  }

  .header-meta {
    flex-direction: column;
  }

  .meta-chip {
    width: 100%;
    justify-content: flex-start;
  }

  .header-actions {
    flex-direction: column;
  }

  .action-btn {
    width: 100%;
  }

  .card-header {
    align-items: flex-start;
    flex-direction: column;
  }

  .calendar-nav,
  .trend-actions {
    width: 100%;
    justify-content: space-between;
  }

  .trend-btn {
    flex: 1 1 0;
    text-align: center;
  }
}
</style>
