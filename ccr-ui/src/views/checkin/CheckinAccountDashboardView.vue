<template>
  <PageShell class="checkin-account-dashboard">
    <template #header>
      <PageHeader
        :title="dashboard?.account.name || tt('账号 Dashboard', 'Account dashboard')"
        :description="tt('签到账号 · Dashboard', 'Check-in account dashboard')"
      >
        <template #leading>
          <button
            type="button"
            class="icon-button"
            :aria-label="tt('返回账号列表', 'Back to account list')"
            :title="tt('返回账号列表', 'Back to account list')"
            @click="goBack"
          >
            <SIcon
              name="ArrowLeft"
              size="w-4 h-4"
            />
          </button>
        </template>
        <template #status>
          <span class="provider-pill">
            {{ dashboard?.account.provider_name || tt('未知提供商', 'Unknown provider') }}
          </span>
          <span
            v-if="dashboard"
            class="status-pill"
            :class="accountEnabled ? 'status-on' : 'status-off'"
          >
            {{ accountEnabled ? tt('启用', 'Enabled') : tt('已禁用', 'Disabled') }}
          </span>
        </template>
        <template #actions>
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
            {{ tt('签到', 'Check in') }}
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
            {{ tt('刷新余额', 'Refresh balance') }}
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
            {{ tt('刷新', 'Refresh') }}
          </button>
        </template>
      </PageHeader>
    </template>

    <div class="dashboard-shell">
      <div class="header-meta">
        <span class="meta-chip">
          <SIcon
            name="CalendarDays"
            size="w-3.5 h-3.5"
          />
          {{ tt('最后签到：', 'Last check-in:') }} {{ dashboard?.streak.last_check_in_date || '-' }}
        </span>
        <span class="meta-chip">
          <SIcon
            name="Wallet"
            size="w-3.5 h-3.5"
          />
          {{ tt('余额更新：', 'Balance updated:') }} {{ formatDateTime(dashboard?.account.last_balance_check_at) }}
        </span>
      </div>

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
          {{ tt('重试', 'Retry') }}
        </button>
      </div>

      <div
        v-else-if="loading"
        class="state-card checkin-surface-card state-loading"
      >
        <div class="loader" />
        {{ tt('加载中...', 'Loading...') }}
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
                  {{ tt('账号总览', 'Account overview') }}
                </p>
                <h2>{{ tt('账号统计', 'Account stats') }}</h2>
              </div>
            </div>

            <div class="vertical-items">
              <StatTile
                :label="tt('当前余额', 'Current balance')"
                :value="formatCurrency(dashboard.account.latest_balance, dashboard.account.balance_currency)"
              />
              <StatTile
                :label="tt('总额度', 'Total quota')"
                :value="formatCurrency(dashboard.account.total_quota, dashboard.account.balance_currency)"
              />
              <StatTile
                :label="tt('历史消耗', 'Usage to date')"
                :value="formatCurrency(dashboard.account.used_quota, dashboard.account.balance_currency)"
              />
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
                  {{ tt('连续签到快照', 'Streak snapshot') }}
                </p>
                <h2>{{ tt('签到统计', 'Check-in stats') }}</h2>
              </div>
            </div>

            <div class="vertical-items">
              <StatTile
                :label="tt('当前连续', 'Current streak')"
                :value="dashboard.streak.current_streak"
                :hint="tt('天', 'days')"
              />
              <StatTile
                :label="tt('最长连续', 'Longest streak')"
                :value="dashboard.streak.longest_streak"
                :hint="tt('天', 'days')"
              />
              <StatTile
                :label="tt('总签到天数', 'Total check-in days')"
                :value="dashboard.streak.total_check_in_days"
                :hint="tt('天', 'days')"
              />
            </div>

            <div class="checkin-progress">
              <div class="progress-info">
                <span>{{ tt('本月签到率', 'Check-in rate this month') }}</span>
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
                {{ tt(`${dashboard.calendar.month_stats.checked_in_days} / ${dashboard.calendar.month_stats.total_days} 天`, `${dashboard.calendar.month_stats.checked_in_days} / ${dashboard.calendar.month_stats.total_days} days`) }}
              </div>
            </div>
          </section>

          <section class="calendar-card checkin-surface-card">
            <div class="card-header">
              <div class="card-copy">
                <p class="card-overline">
                  {{ tt('月度记录', 'Monthly history') }}
                </p>
                <h2>{{ tt('签到日历', 'Check-in calendar') }}</h2>
              </div>

              <div class="calendar-nav">
                <button
                  type="button"
                  class="nav-btn"
                  :aria-label="tt('上个月', 'Previous month')"
                  @click="prevMonth"
                >
                  {{ prevGlyph }}
                </button>
                <span class="calendar-month">{{ formatCalendarMonth(calendarYear, calendarMonth) }}</span>
                <button
                  type="button"
                  class="nav-btn"
                  :aria-label="tt('下个月', 'Next month')"
                  @click="nextMonth"
                >
                  {{ nextGlyph }}
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
                {{ tt('滚动变化', 'Rolling changes') }}
              </p>
              <div class="trend-title-row">
                <h2>{{ tt('签到趋势', 'Check-in trend') }}</h2>
                <span class="trend-tag">{{ tt(`近 ${trendDays} 天`, `Last ${trendDays} days`) }}</span>
              </div>
            </div>

            <div class="trend-actions">
              <PillToggleGroup
                :options="trendToggleOptions"
                :model-value="trendDays"
                @update:model-value="trendDays = $event"
              />
            </div>
          </div>

          <div class="trend-body">
            <AccountDashboardTrend :trend="dashboard.trend" />
          </div>
        </section>
      </div>
    </div>
  </PageShell>
</template>

<script setup lang="ts">
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
import PillToggleGroup from '@/components/ui/PillToggleGroup.vue'
import StatTile from '@/components/ui/StatTile.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
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
const { locale } = useI18n()
const uiStore = useUIStore()
const isZh = computed(() => locale.value.startsWith('zh'))
const tt = (zh: string, en: string) => (isZh.value ? zh : en)
const prevGlyph = '\u2039'
const nextGlyph = '\u203A'

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
const trendToggleOptions = computed(() =>
  trendOptions.map((option) => ({
    value: option,
    label: String(option),
  })),
)

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
    error.value = getErrorMessage(currentError, tt('加载失败', 'Load failed'))
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
        ? tt('签到成功', 'Check-in successful')
        : result.status === 'already_checked_in'
          ? tt('今日已签到', 'Already checked in today')
          : tt('签到失败', 'Check-in failed')
    const message = result.message ? tt(`${label}：${result.message}`, `${label}: ${result.message}`) : label
    if (result.status === 'failed') {
      uiStore.showError(message)
    } else {
      uiStore.showSuccess(message)
    }
    await loadDashboard()
  } catch (currentError: unknown) {
    uiStore.showError(tt('签到失败：', 'Check-in failed: ') + getErrorMessage(currentError, tt('未知错误', 'Unknown error')))
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
      tt(
        `余额：${result.currency}${result.remaining_quota.toFixed(2)}（已用 ${result.usage_percentage.toFixed(1)}%）`,
        `Balance: ${result.currency}${result.remaining_quota.toFixed(2)} (${result.usage_percentage.toFixed(1)}% used)`
      )
    )
    await loadDashboard()
  } catch (currentError: unknown) {
    uiStore.showError(tt('刷新余额失败：', 'Balance refresh failed: ') + getErrorMessage(currentError, tt('未知错误', 'Unknown error')))
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
  return new Date(value).toLocaleString(isZh.value ? 'zh-CN' : 'en-US')
}

const formatCalendarMonth = (year: number, month: number) => {
  if (isZh.value) {
    return `${year}年${month}月`
  }

  return new Date(year, month - 1, 1).toLocaleDateString('en-US', {
    month: 'long',
    year: 'numeric',
  })
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
  color: var(--color-text-muted);
  font-size: 0.75rem;
  font-weight: 600;
  letter-spacing: 0;
}

.header-title-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.65rem;
}

.header-title-row h1 {
  margin: 0;
  color: var(--color-text-primary);
  font-size: 1.5rem;
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
  color: var(--color-accent-primary);
}

.status-pill.status-on {
  background: rgb(var(--color-success-rgb) / 12%);
  border: 1px solid rgb(var(--color-success-rgb) / 24%);
  color: var(--color-success);
}

.status-pill.status-off {
  background: rgb(var(--color-danger-rgb) / 12%);
  border: 1px solid rgb(var(--color-danger-rgb) / 22%);
  color: var(--color-danger);
}

.meta-chip {
  padding: 0.45rem 0.75rem;
  background: rgb(var(--color-bg-elevated-rgb) / 54%);
  border: 1px solid rgb(var(--color-border-default-rgb) / 56%);
  color: var(--color-text-secondary);
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
  color: var(--color-text-secondary);
  cursor: pointer;
}

.icon-button:hover {
  transform: translateY(-1px);
  background: rgb(var(--color-bg-elevated-rgb) / 78%);
  color: var(--color-text-primary);
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
  color: var(--color-text-primary);
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
  background: var(--color-accent-primary);
  color: var(--color-accent-primary-contrast);
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
  color: var(--color-text-secondary);
}

.state-card p {
  margin: 0;
}

.state-error {
  justify-content: space-between;
  color: var(--color-danger);
  border-color: rgb(var(--color-danger-rgb) / 34%);
}

.state-loading {
  color: var(--color-text-primary);
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
  border-top-color: var(--color-accent-primary);
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
  color: var(--color-text-muted);
  font-size: 0.75rem;
  font-weight: 600;
  letter-spacing: 0;
}

.card-copy h2,
.trend-title-row h2 {
  margin: 0;
  color: var(--color-text-primary);
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
  color: var(--color-accent-primary);
}

.stats-icon.success,
.vertical-icon.success {
  background: rgb(var(--color-success-rgb) / 12%);
  color: var(--color-success);
}

.stats-icon.warning,
.vertical-icon.warning {
  background: rgb(var(--color-warning-rgb) / 14%);
  color: var(--color-warning);
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
  color: var(--color-text-muted);
  font-size: 0.72rem;
  font-weight: 600;
}

.vertical-value {
  color: var(--color-text-primary);
  font-size: 1.25rem;
  font-weight: 700;
  line-height: 1.1;
  letter-spacing: -0.02em;
  font-family: var(--font-mono);
}

.vertical-value.success {
  color: var(--color-success);
}

.vertical-value.accent {
  color: var(--color-accent-primary);
}

.vertical-value.warning {
  color: var(--color-warning);
}

.vertical-value small {
  color: var(--color-text-secondary);
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
  color: var(--color-text-secondary);
  font-size: 0.74rem;
}

.progress-info {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.4rem;
}

.progress-percent {
  color: var(--color-accent-primary);
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
  background: var(--color-accent-primary);
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
  color: var(--color-text-primary);
  font-size: 1rem;
  cursor: pointer;
}

.nav-btn:hover {
  background: rgb(var(--color-bg-elevated-rgb) / 88%);
}

.calendar-month {
  min-width: 6rem;
  color: var(--color-text-primary);
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
  color: var(--color-accent-primary);
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
  color: var(--color-text-secondary);
  font-size: 0.74rem;
  font-weight: 700;
  cursor: pointer;
}

.trend-btn.active {
  background: rgb(var(--color-bg-elevated-rgb) / 92%);
  color: var(--color-accent-primary);
  box-shadow: var(--shadow-sm);
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
