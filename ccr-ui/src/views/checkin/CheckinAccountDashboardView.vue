<template>
  <div class="checkin-account-dashboard">
    <div
      class="dashboard-scene"
      aria-hidden="true"
    >
      <div class="dashboard-scrim" />
      <div class="dashboard-vignette" />
      <div class="dashboard-glow glow-primary" />
      <div class="dashboard-glow glow-secondary" />
    </div>

    <div class="dashboard-shell">
      <section class="dashboard-header dashboard-surface">
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
        class="state-card dashboard-surface state-error"
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
        class="state-card dashboard-surface state-loading"
      >
        <div class="loader" />
        加载中...
      </div>

      <div
        v-else-if="dashboard"
        class="dashboard-stack"
      >
        <div class="dashboard-main-grid">
          <section class="stats-card-vertical dashboard-surface">
            <div class="card-lead">
              <div class="stats-icon purple">
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
                <div class="vertical-icon green">
                  <SIcon
                    name="Wallet"
                    size="w-4 h-4"
                  />
                </div>
                <div class="vertical-copy">
                  <span class="vertical-label">当前余额</span>
                  <span class="vertical-value green">
                    {{ formatCurrency(dashboard.account.latest_balance, dashboard.account.balance_currency) }}
                  </span>
                </div>
              </article>

              <article class="vertical-stat">
                <div class="vertical-icon blue">
                  <SIcon
                    name="TrendingUp"
                    size="w-4 h-4"
                  />
                </div>
                <div class="vertical-copy">
                  <span class="vertical-label">总额度</span>
                  <span class="vertical-value blue">
                    {{ formatCurrency(dashboard.account.total_quota, dashboard.account.balance_currency) }}
                  </span>
                </div>
              </article>

              <article class="vertical-stat">
                <div class="vertical-icon orange">
                  <SIcon
                    name="History"
                    size="w-4 h-4"
                  />
                </div>
                <div class="vertical-copy">
                  <span class="vertical-label">历史消耗</span>
                  <span class="vertical-value orange">
                    {{ formatCurrency(dashboard.account.used_quota, dashboard.account.balance_currency) }}
                  </span>
                </div>
              </article>
            </div>
          </section>

          <section class="stats-card-vertical dashboard-surface">
            <div class="card-lead">
              <div class="stats-icon orange">
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
                <div class="vertical-icon orange">
                  <SIcon
                    name="Flame"
                    size="w-4 h-4"
                  />
                </div>
                <div class="vertical-copy">
                  <span class="vertical-label">当前连续</span>
                  <span class="vertical-value orange">
                    {{ dashboard.streak.current_streak }} <small>天</small>
                  </span>
                </div>
              </article>

              <article class="vertical-stat">
                <div class="vertical-icon yellow">
                  <SIcon
                    name="Trophy"
                    size="w-4 h-4"
                  />
                </div>
                <div class="vertical-copy">
                  <span class="vertical-label">最长连续</span>
                  <span class="vertical-value">
                    {{ dashboard.streak.longest_streak }} <small>天</small>
                  </span>
                </div>
              </article>

              <article class="vertical-stat">
                <div class="vertical-icon purple">
                  <SIcon
                    name="Calendar"
                    size="w-4 h-4"
                  />
                </div>
                <div class="vertical-copy">
                  <span class="vertical-label">总签到天数</span>
                  <span class="vertical-value purple">
                    {{ dashboard.streak.total_check_in_days }} <small>天</small>
                  </span>
                </div>
              </article>
            </div>

            <div class="checkin-progress">
              <div class="progress-info">
                <span>签到率</span>
                <span class="progress-percent">
                  {{ dashboard.calendar.month_stats.check_in_rate.toFixed(1) }}%
                </span>
              </div>
              <div class="progress-bar-track">
                <div
                  class="progress-bar-fill"
                  :style="{ transform: `scaleX(${dashboard.calendar.month_stats.check_in_rate / 100})` }"
                />
              </div>
              <div class="progress-days">
                {{ dashboard.calendar.month_stats.checked_in_days }} / {{ dashboard.calendar.month_stats.total_days }} 天
              </div>
            </div>
          </section>

          <section class="calendar-card dashboard-surface">
            <div class="card-header">
              <div class="card-copy">
                <p class="card-overline">
                  Monthly history
                </p>
                <h2>签到日历</h2>
              </div>

              <div class="calendar-picker">
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
            </div>

            <AccountDashboardCalendar :calendar="dashboard.calendar" />
          </section>
        </div>

        <section class="trend-card dashboard-surface">
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
const getErrorMessage = (currentError: unknown, fallback: string) =>
  currentError instanceof Error ? currentError.message : fallback

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
    alert(`签到${result.status === 'success' ? '成功' : result.status === 'already_checked_in' ? '：今日已签到' : '失败'}: ${result.message || ''}`)
    await loadDashboard()
  } catch (currentError: unknown) {
    alert('签到失败: ' + getErrorMessage(currentError, '未知错误'))
  } finally {
    checkinLoading.value = false
  }
}

const handleBalanceRefresh = async () => {
  if (!accountId.value) return
  balanceLoading.value = true

  try {
    const result = await queryCheckinBalance<BalanceSnapshot>(accountId.value)
    alert(`余额: ${result.currency}${result.remaining_quota.toFixed(2)} (已用: ${result.usage_percentage.toFixed(1)}%)`)
    await loadDashboard()
  } catch (currentError: unknown) {
    alert('刷新余额失败: ' + getErrorMessage(currentError, '未知错误'))
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
  --dashboard-surface-bg: rgb(var(--color-bg-elevated-rgb) / 86%);
  --dashboard-surface-muted: rgb(var(--color-bg-surface-rgb) / 72%);
  --dashboard-surface-strong: rgb(var(--color-bg-base-rgb) / 94%);
  --dashboard-border: rgb(var(--color-border-default-rgb) / 82%);
  --dashboard-border-soft: rgb(var(--color-border-default-rgb) / 56%);
  --dashboard-shadow: 0 24px 64px rgb(74 36 78 / 18%);
  --dashboard-shadow-soft: 0 14px 38px rgb(74 36 78 / 10%);

  position: relative;
  min-height: 100vh;
  overflow: hidden;
  isolation: isolate;
  background:
    radial-gradient(circle at top left, rgb(var(--color-info-rgb) / 18%), transparent 40%),
    radial-gradient(circle at 80% 18%, rgb(var(--color-accent-secondary-rgb) / 16%), transparent 32%),
    linear-gradient(180deg, rgb(var(--color-bg-base-rgb) / 18%), rgb(var(--color-bg-elevated-rgb) / 58%));
}

.dashboard-scene {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.dashboard-scrim {
  position: absolute;
  inset: 0;
  backdrop-filter: blur(16px) saturate(135%);
  background:
    linear-gradient(180deg, rgb(255 252 253 / 18%), rgb(255 245 247 / 44%)),
    radial-gradient(circle at 18% 15%, rgb(var(--color-accent-primary-rgb) / 10%), transparent 28%);
}

.dashboard-vignette {
  position: absolute;
  inset: 0;
  background:
    linear-gradient(180deg, rgb(255 255 255 / 0%), rgb(255 248 251 / 32%) 55%, rgb(255 244 247 / 54%) 100%),
    linear-gradient(90deg, rgb(255 248 251 / 36%), rgb(255 255 255 / 0%) 24%, rgb(255 255 255 / 0%) 76%, rgb(255 248 251 / 34%));
}

.dashboard-glow {
  position: absolute;
  border-radius: 999px;
  filter: blur(18px);
  opacity: 0.72;
}

.dashboard-glow.glow-primary {
  top: 4rem;
  right: -4rem;
  width: 18rem;
  height: 18rem;
  background: radial-gradient(circle, rgb(var(--color-accent-primary-rgb) / 30%), transparent 72%);
}

.dashboard-glow.glow-secondary {
  bottom: 8rem;
  left: -5rem;
  width: 20rem;
  height: 20rem;
  background: radial-gradient(circle, rgb(var(--color-info-rgb) / 18%), transparent 72%);
}

.dashboard-shell {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  width: 100%;
  max-width: 1520px;
  margin: 0 auto;
  padding: 1.5rem;
}

.dashboard-stack {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.dashboard-surface {
  background: var(--dashboard-surface-bg);
  border: 1px solid var(--dashboard-border);
  box-shadow: var(--dashboard-shadow);
  backdrop-filter: blur(22px) saturate(150%);
}

.dashboard-header {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 1rem 1.5rem;
  padding: 1.35rem 1.5rem;
  border-radius: 1.5rem;
}

.header-left {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
  min-width: 0;
}

.header-copy {
  display: grid;
  gap: 0.85rem;
  min-width: 0;
}

.header-title-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.75rem;
}

.header-title-row h1 {
  margin: 0;
  color: var(--text-primary);
  font-size: clamp(1.65rem, 2vw, 2.15rem);
  font-weight: 700;
  letter-spacing: -0.03em;
}

.header-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.65rem;
}

.provider-pill,
.status-pill,
.meta-chip,
.trend-tag {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  border-radius: 999px;
  font-size: 0.76rem;
  font-weight: 600;
  line-height: 1;
  white-space: nowrap;
}

.provider-pill,
.status-pill,
.trend-tag {
  padding: 0.42rem 0.8rem;
}

.provider-pill {
  background: rgb(var(--color-platform-gemini-rgb) / 10%);
  border: 1px solid rgb(var(--color-platform-gemini-rgb) / 22%);
  color: var(--platform-gemini);
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
  padding: 0.5rem 0.8rem;
  background: var(--dashboard-surface-muted);
  border: 1px solid var(--dashboard-border-soft);
  color: var(--text-secondary);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 35%);
}

.icon-button,
.action-btn,
.nav-btn,
.trend-btn {
  border: 1px solid var(--dashboard-border-soft);
  backdrop-filter: blur(18px) saturate(135%);
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
  width: 2.5rem;
  height: 2.5rem;
  border-radius: 999px;
  background: var(--dashboard-surface-muted);
  color: var(--text-secondary);
  box-shadow: var(--dashboard-shadow-soft);
}

.icon-button:hover {
  transform: translateY(-1px);
  background: var(--dashboard-surface-strong);
  border-color: var(--dashboard-border);
  color: var(--text-primary);
}

.header-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.45rem;
  min-height: 2.8rem;
  padding: 0.7rem 1rem;
  border-radius: 1rem;
  background: var(--dashboard-surface-muted);
  color: var(--text-primary);
  font-size: 0.88rem;
  font-weight: 600;
  box-shadow: var(--dashboard-shadow-soft);
}

.action-btn:hover:not(:disabled),
.nav-btn:hover,
.trend-btn:hover {
  transform: translateY(-1px);
  border-color: var(--dashboard-border);
  background: var(--dashboard-surface-strong);
}

.action-btn.primary {
  border-color: transparent;
  background: linear-gradient(135deg, rgb(var(--color-platform-gemini-rgb) / 92%), rgb(var(--color-info-rgb) / 88%));
  color: white;
  box-shadow: 0 16px 32px rgb(var(--color-platform-gemini-rgb) / 28%);
}

.action-btn.primary:hover:not(:disabled) {
  background: linear-gradient(135deg, rgb(var(--color-platform-gemini-rgb) / 100%), rgb(var(--color-info-rgb) / 96%));
}

.action-btn:disabled,
.nav-btn:disabled,
.trend-btn:disabled {
  opacity: 0.55;
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
  padding: 1.1rem 1.35rem;
  border-radius: 1.25rem;
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
  font-size: 0.88rem;
  font-weight: 600;
  text-decoration: underline;
  text-underline-offset: 0.18em;
}

.loader {
  width: 1.45rem;
  height: 1.45rem;
  border: 2px solid rgb(var(--color-border-default-rgb) / 55%);
  border-top-color: var(--platform-gemini);
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
  grid-template-columns: minmax(18rem, 0.95fr) minmax(18rem, 0.95fr) minmax(26rem, 1.4fr);
  gap: 1.25rem;
}

.stats-card-vertical,
.calendar-card,
.trend-card {
  border-radius: 1.4rem;
}

.stats-card-vertical {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1.35rem;
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
  gap: 0.22rem;
}

.card-overline {
  margin: 0;
  color: var(--text-muted);
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.card-copy h2,
.trend-title-row h2 {
  margin: 0;
  color: var(--text-primary);
  font-size: 1.05rem;
  font-weight: 700;
}

.stats-icon,
.vertical-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 0.95rem;
  flex-shrink: 0;
}

.stats-icon {
  width: 2.5rem;
  height: 2.5rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 58%);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 32%);
}

.vertical-icon {
  width: 2.75rem;
  height: 2.75rem;
}

.stats-icon.purple,
.vertical-icon.purple {
  background: rgb(var(--color-accent-secondary-rgb) / 12%);
  color: var(--platform-claude);
}

.stats-icon.green,
.vertical-icon.green {
  background: rgb(var(--color-success-rgb) / 12%);
  color: var(--accent-success);
}

.stats-icon.blue,
.vertical-icon.blue {
  background: rgb(var(--color-platform-gemini-rgb) / 12%);
  color: var(--platform-gemini);
}

.stats-icon.orange,
.vertical-icon.orange {
  background: rgb(var(--color-warning-rgb) / 12%);
  color: var(--accent-warning);
}

.stats-icon.yellow,
.vertical-icon.yellow {
  background: rgb(var(--color-warning-rgb) / 14%);
  color: var(--platform-codex);
}

.vertical-items {
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
}

.vertical-stat {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 0.9rem;
  align-items: center;
  padding: 0.9rem 1rem;
  border-radius: 1.05rem;
  background: rgb(var(--color-bg-elevated-rgb) / 52%);
  border: 1px solid rgb(var(--color-border-default-rgb) / 60%);
  box-shadow:
    inset 0 1px 0 rgb(255 255 255 / 36%),
    0 10px 22px rgb(74 36 78 / 7%);
}

.vertical-copy {
  display: grid;
  gap: 0.25rem;
  min-width: 0;
}

.vertical-label {
  color: var(--text-muted);
  font-size: 0.75rem;
  font-weight: 600;
}

.vertical-value {
  color: var(--text-primary);
  font-size: 1.35rem;
  font-weight: 700;
  line-height: 1.1;
  letter-spacing: -0.02em;
}

.vertical-value.green {
  color: var(--accent-success);
}

.vertical-value.blue {
  color: var(--platform-gemini);
}

.vertical-value.orange {
  color: var(--accent-warning);
}

.vertical-value.purple {
  color: var(--platform-claude);
}

.vertical-value small {
  color: var(--text-secondary);
  font-size: 0.84rem;
  font-weight: 600;
}

.checkin-progress {
  margin-top: auto;
  padding: 1rem 1rem 0;
  border-top: 1px solid rgb(var(--color-border-default-rgb) / 62%);
}

.progress-info,
.progress-days {
  color: var(--text-secondary);
  font-size: 0.76rem;
}

.progress-info {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.45rem;
}

.progress-percent {
  color: var(--accent-success);
  font-weight: 700;
}

.progress-bar-track {
  height: 0.42rem;
  overflow: hidden;
  border-radius: 999px;
  background: rgb(var(--color-border-default-rgb) / 72%);
  box-shadow: inset 0 1px 2px rgb(0 0 0 / 8%);
}

.progress-bar-fill {
  width: 100%;
  height: 100%;
  transform-origin: left center;
  background: linear-gradient(90deg, var(--platform-gemini), var(--accent-info));
  box-shadow: 0 0 12px rgb(var(--color-platform-gemini-rgb) / 24%);
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.progress-days {
  margin-top: 0.35rem;
}

.calendar-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1.35rem;
}

.calendar-picker {
  display: flex;
  align-items: center;
}

.calendar-nav {
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
  padding: 0.3rem;
  border-radius: 999px;
  background: var(--dashboard-surface-muted);
  border: 1px solid var(--dashboard-border-soft);
}

.nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  padding: 0;
  border-radius: 999px;
  background: rgb(var(--color-bg-elevated-rgb) / 72%);
  color: var(--text-primary);
  font-size: 1.05rem;
  box-shadow: var(--dashboard-shadow-soft);
}

.calendar-month {
  min-width: 6.5rem;
  color: var(--text-primary);
  font-size: 0.84rem;
  font-weight: 700;
  text-align: center;
}

.trend-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1.35rem;
}

.trend-header {
  padding-bottom: 0.95rem;
  border-bottom: 1px solid rgb(var(--color-border-default-rgb) / 62%);
}

.trend-title-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.75rem;
}

.trend-tag {
  background: rgb(var(--color-platform-gemini-rgb) / 10%);
  border: 1px solid rgb(var(--color-platform-gemini-rgb) / 22%);
  color: var(--platform-gemini);
}

.trend-actions {
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
  padding: 0.3rem;
  border-radius: 999px;
  background: var(--dashboard-surface-muted);
  border: 1px solid var(--dashboard-border-soft);
}

.trend-btn {
  min-width: 2.75rem;
  padding: 0.45rem 0.8rem;
  border-radius: 999px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 0.76rem;
  font-weight: 700;
}

.trend-btn.active {
  border-color: rgb(var(--color-platform-gemini-rgb) / 24%);
  background: rgb(var(--color-bg-elevated-rgb) / 82%);
  color: var(--platform-gemini);
  box-shadow: 0 10px 24px rgb(var(--color-platform-gemini-rgb) / 14%);
}

.trend-body {
  min-height: 15rem;
}

:global(.dark) .checkin-account-dashboard {
  --dashboard-surface-bg: rgb(var(--color-bg-elevated-rgb) / 84%);
  --dashboard-surface-muted: rgb(var(--color-bg-surface-rgb) / 76%);
  --dashboard-surface-strong: rgb(var(--color-bg-surface-rgb) / 92%);
  --dashboard-border: rgb(var(--color-border-default-rgb) / 88%);
  --dashboard-border-soft: rgb(var(--color-border-default-rgb) / 68%);
  --dashboard-shadow: 0 28px 72px rgb(0 0 0 / 42%);
  --dashboard-shadow-soft: 0 16px 34px rgb(0 0 0 / 24%);

  background:
    radial-gradient(circle at top left, rgb(var(--color-info-rgb) / 20%), transparent 40%),
    radial-gradient(circle at 82% 16%, rgb(var(--color-accent-secondary-rgb) / 16%), transparent 34%),
    linear-gradient(180deg, rgb(var(--color-bg-base-rgb) / 24%), rgb(var(--color-bg-base-rgb) / 66%));
}

:global(.dark) .dashboard-scrim {
  background:
    linear-gradient(180deg, rgb(12 6 18 / 18%), rgb(15 8 20 / 42%)),
    radial-gradient(circle at 18% 15%, rgb(var(--color-accent-primary-rgb) / 10%), transparent 28%);
}

:global(.dark) .dashboard-vignette {
  background:
    linear-gradient(180deg, rgb(12 6 18 / 0%), rgb(15 8 20 / 26%) 55%, rgb(15 8 20 / 48%) 100%),
    linear-gradient(90deg, rgb(12 6 18 / 24%), rgb(12 6 18 / 0%) 24%, rgb(12 6 18 / 0%) 76%, rgb(12 6 18 / 22%));
}

:global(.dark) .meta-chip,
:global(.dark) .vertical-stat,
:global(.dark) .nav-btn,
:global(.dark) .trend-btn.active {
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 6%);
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
    padding: 1.2rem;
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

  .calendar-picker,
  .calendar-nav,
  .trend-actions {
    width: 100%;
  }

  .calendar-nav,
  .trend-actions {
    justify-content: space-between;
  }

  .trend-btn {
    flex: 1 1 0;
    text-align: center;
  }
}
</style>
