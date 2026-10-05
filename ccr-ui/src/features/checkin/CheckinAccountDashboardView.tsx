import { useCallback, useEffect, useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { checkinAccount, getCheckinAccountDashboard, queryCheckinBalance } from '@/api'
import { getErrorMessage } from '@/types/api'
import type { BalanceSnapshot, CheckinAccountDashboardResponse } from '@/types/checkin'
import { extractStringParam } from '@/types/router'
import { PageHeader, PageShell, PillToggleGroup, SIcon, StatTile, buttonClass } from '@/ui'
import { AccountDashboardCalendar } from './components/AccountDashboardCalendar'
import { AccountDashboardTrend } from './components/AccountDashboardTrend'
import { checkinNotify } from './lib/checkinNotify'
import { useCheckinLocale, useTt } from './hooks/useCheckinT'
import './styles/dashboard.css'

const TREND_OPTIONS = [7, 30, 90]

const formatCurrency = (value?: number, currency?: string) => {
  if (value === undefined || value === null) return '-'
  const symbol = currency === 'CNY' ? '¥' : currency === 'USD' ? '$' : currency ? `${currency} ` : '$'
  return `${symbol}${value.toFixed(2)}`
}

export function CheckinAccountDashboardView() {
  const params = useParams()
  const navigate = useNavigate()
  const locale = useCheckinLocale()
  const isZh = locale.startsWith('zh')
  const tt = useTt()
  const accountId = extractStringParam(params.accountId) || ''
  const [dashboard, setDashboard] = useState<CheckinAccountDashboardResponse | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [checkinLoading, setCheckinLoading] = useState(false)
  const [balanceLoading, setBalanceLoading] = useState(false)
  const now = useMemo(() => new Date(), [])
  const [calendarYear, setCalendarYear] = useState(now.getFullYear())
  const [calendarMonth, setCalendarMonth] = useState(now.getMonth() + 1)
  const [trendDays, setTrendDays] = useState(30)

  const loadDashboard = useCallback(async () => {
    if (!accountId) return
    setLoading(true)
    setError(null)
    try {
      setDashboard(
        await getCheckinAccountDashboard(accountId, {
          year: calendarYear,
          month: calendarMonth,
          days: trendDays,
        }),
      )
    } catch (currentError: unknown) {
      setError(getErrorMessage(currentError, tt('加载失败', 'Load failed')))
    } finally {
      setLoading(false)
    }
  }, [accountId, calendarMonth, calendarYear, trendDays, tt])

  useEffect(() => {
    void loadDashboard()
  }, [loadDashboard])

  const goBack = useCallback(() => {
    if (window.history.length > 1) navigate(-1)
    else navigate('/checkin')
  }, [navigate])

  const handleCheckin = useCallback(async () => {
    if (!accountId) return
    setCheckinLoading(true)
    try {
      const result = await checkinAccount<{ status?: string; message?: string }>(accountId)
      const label =
        result.status === 'success'
          ? tt('签到成功', 'Check-in successful')
          : result.status === 'already_checked_in'
            ? tt('今日已签到', 'Already checked in today')
            : tt('签到失败', 'Check-in failed')
      const message = result.message ? `${label}: ${result.message}` : label
      if (result.status === 'failed') checkinNotify.error(message)
      else checkinNotify.success(message)
      await loadDashboard()
    } catch (currentError: unknown) {
      checkinNotify.error(tt('签到失败：', 'Check-in failed: ') + getErrorMessage(currentError, tt('未知错误', 'Unknown error')))
    } finally {
      setCheckinLoading(false)
    }
  }, [accountId, loadDashboard, tt])

  const handleBalanceRefresh = useCallback(async () => {
    if (!accountId) return
    setBalanceLoading(true)
    try {
      const result = await queryCheckinBalance<BalanceSnapshot>(accountId)
      checkinNotify.success(
        tt(
          `余额：${result.currency}${result.remaining_quota.toFixed(2)}（已用 ${result.usage_percentage.toFixed(1)}%）`,
          `Balance: ${result.currency}${result.remaining_quota.toFixed(2)} (${result.usage_percentage.toFixed(1)}% used)`,
        ),
      )
      await loadDashboard()
    } catch (currentError: unknown) {
      checkinNotify.error(
        tt('刷新余额失败：', 'Balance refresh failed: ') + getErrorMessage(currentError, tt('未知错误', 'Unknown error')),
      )
    } finally {
      setBalanceLoading(false)
    }
  }, [accountId, loadDashboard, tt])

  const prevMonth = useCallback(() => {
    if (calendarMonth === 1) {
      setCalendarMonth(12)
      setCalendarYear((year) => year - 1)
      return
    }
    setCalendarMonth((month) => month - 1)
  }, [calendarMonth])

  const nextMonth = useCallback(() => {
    if (calendarMonth === 12) {
      setCalendarMonth(1)
      setCalendarYear((year) => year + 1)
      return
    }
    setCalendarMonth((month) => month + 1)
  }, [calendarMonth])

  const trendToggleOptions = TREND_OPTIONS.map((option) => ({
    value: option,
    label: String(option),
  }))

  return (
    <PageShell
      className="checkin-account-dashboard"
      header={
        <DashboardHeader
          title={dashboard?.account?.name || tt('账号 Dashboard', 'Account dashboard')}
          description={tt('签到账号 · Dashboard', 'Check-in account dashboard')}
          providerName={dashboard?.account?.provider_name || tt('未知提供商', 'Unknown provider')}
          enabled={dashboard?.account?.enabled}
          loading={loading}
          hasDashboard={Boolean(dashboard?.account)}
          checkinLoading={checkinLoading}
          balanceLoading={balanceLoading}
          tt={tt}
          onBack={goBack}
          onCheckin={handleCheckin}
          onBalance={handleBalanceRefresh}
          onRefresh={loadDashboard}
        />
      }
    >
      {error ? (
        <div className="state-card checkin-surface-card">
          <p>{error}</p>
          <button type="button" className="ghost-link" onClick={loadDashboard}>
            {tt('重试', 'Retry')}
          </button>
        </div>
      ) : null}
      {loading ? <div className="state-card checkin-surface-card">{tt('加载中...', 'Loading...')}</div> : null}
      {dashboard && dashboard.account && !loading ? (
        <DashboardBody
          dashboard={dashboard}
          tt={tt}
          isZh={isZh}
          calendarYear={calendarYear}
          calendarMonth={calendarMonth}
          trendDays={trendDays}
          trendToggleOptions={trendToggleOptions}
          onPrevMonth={prevMonth}
          onNextMonth={nextMonth}
          onTrendDays={setTrendDays}
        />
      ) : null}
    </PageShell>
  )
}

function DashboardHeader({
  title,
  description,
  providerName,
  enabled,
  loading,
  hasDashboard,
  checkinLoading,
  balanceLoading,
  tt,
  onBack,
  onCheckin,
  onBalance,
  onRefresh,
}: {
  title: string
  description: string
  providerName: string
  enabled?: boolean
  loading: boolean
  hasDashboard: boolean
  checkinLoading: boolean
  balanceLoading: boolean
  tt: (zh: string, en: string) => string
  onBack: () => void
  onCheckin: () => void
  onBalance: () => void
  onRefresh: () => void
}) {
  const enabledLabel = enabled ? tt('启用', 'Enabled') : tt('已禁用', 'Disabled')
  return (
    <PageHeader
      title={title}
      description={description}
      leading={
        <button type="button" className="icon-button" onClick={onBack} aria-label={tt('返回账号列表', 'Back to account list')}>
          <SIcon name="ArrowLeft" size="w-4 h-4" />
        </button>
      }
      status={
        <>
          <span className="provider-pill">{providerName}</span>
          {hasDashboard ? (
            <span className={`status-pill ${enabled ? 'status-on' : 'status-off'}`}>{enabledLabel}</span>
          ) : null}
        </>
      }
      actions={
        <>
          <button type="button" className={buttonClass({ variant: 'ghost', className: 'action-btn' })} disabled={loading || !hasDashboard || checkinLoading} onClick={onCheckin}>
            {tt('签到', 'Check in')}
          </button>
          <button type="button" className={buttonClass({ variant: 'ghost', className: 'action-btn' })} disabled={loading || !hasDashboard || balanceLoading} onClick={onBalance}>
            {tt('刷新余额', 'Refresh balance')}
          </button>
          <button type="button" className={buttonClass({ variant: 'primary', className: 'action-btn' })} disabled={loading} onClick={onRefresh}>
            {tt('刷新', 'Refresh')}
          </button>
        </>
      }
    />
  )
}

function DashboardBody({
  dashboard,
  tt,
  isZh,
  calendarYear,
  calendarMonth,
  trendDays,
  trendToggleOptions,
  onPrevMonth,
  onNextMonth,
  onTrendDays,
}: {
  dashboard: CheckinAccountDashboardResponse
  tt: (zh: string, en: string) => string
  isZh: boolean
  calendarYear: number
  calendarMonth: number
  trendDays: number
  trendToggleOptions: Array<{ value: number; label: string }>
  onPrevMonth: () => void
  onNextMonth: () => void
  onTrendDays: (value: number) => void
}) {
  const monthLabel = isZh
    ? `${calendarYear}年${calendarMonth}月`
    : new Date(calendarYear, calendarMonth - 1, 1).toLocaleDateString('en-US', {
        month: 'long',
        year: 'numeric',
      })
  return (
    <div className="dashboard-stack">
      <div className="dashboard-main-grid">
        <section className="stats-card-vertical checkin-surface-card">
          <h2>{tt('账号统计', 'Account stats')}</h2>
          <StatTile
            label={tt('当前余额', 'Current balance')}
            value={formatCurrency(dashboard.account.latest_balance, dashboard.account.balance_currency)}
          />
          <StatTile
            label={tt('总额度', 'Total quota')}
            value={formatCurrency(dashboard.account.total_quota, dashboard.account.balance_currency)}
          />
          <StatTile
            label={tt('历史消耗', 'Usage to date')}
            value={formatCurrency(dashboard.account.used_quota, dashboard.account.balance_currency)}
          />
        </section>
        <section className="stats-card-vertical checkin-surface-card">
          <h2>{tt('签到统计', 'Check-in stats')}</h2>
          <StatTile label={tt('当前连续', 'Current streak')} value={dashboard.streak.current_streak} hint={tt('天', 'days')} />
          <StatTile label={tt('最长连续', 'Longest streak')} value={dashboard.streak.longest_streak} hint={tt('天', 'days')} />
          <StatTile
            label={tt('总签到天数', 'Total check-in days')}
            value={dashboard.streak.total_check_in_days}
            hint={tt('天', 'days')}
          />
          <div className="checkin-progress">
            <span>{tt('本月签到率', 'Check-in rate this month')}</span>
            <span>{dashboard.calendar.month_stats.check_in_rate.toFixed(1)}%</span>
          </div>
        </section>
        <section className="calendar-card checkin-surface-card">
          <div className="card-header">
            <h2>{tt('签到日历', 'Check-in calendar')}</h2>
            <div className="calendar-nav">
              <button type="button" className="nav-btn" onClick={onPrevMonth}>
                ‹
              </button>
              <span className="calendar-month">{monthLabel}</span>
              <button type="button" className="nav-btn" onClick={onNextMonth}>
                ›
              </button>
            </div>
          </div>
          <AccountDashboardCalendar calendar={dashboard.calendar} />
        </section>
      </div>
      <section className="trend-card checkin-surface-card">
        <div className="trend-header">
          <h2>{tt('签到趋势', 'Check-in trend')}</h2>
          <PillToggleGroup options={trendToggleOptions} value={trendDays} onValueChange={onTrendDays} />
        </div>
        <AccountDashboardTrend trend={dashboard.trend} />
      </section>
    </div>
  )
}
