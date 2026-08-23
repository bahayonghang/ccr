import { useMemo } from 'react'
import type { CheckinDashboardCalendar, CheckinDashboardDay } from '@/types/checkin'
import { useCheckinLocale } from '../hooks/useCheckinT'
import '../styles/calendar.css'

interface AccountDashboardCalendarProps {
  calendar: CheckinDashboardCalendar | null
}

const dayReward = (cell: CheckinDashboardDay): number | null => {
  const amount = cell.reward_amount ?? cell.income_increment
  if (amount === undefined || amount === null || amount <= 0) return null
  return amount
}

export function AccountDashboardCalendar({ calendar }: AccountDashboardCalendarProps) {
  const locale = useCheckinLocale()
  const isZh = locale.startsWith('zh')
  const tt = (zh: string, en: string) => (isZh ? zh : en)
  const weekLabels = isZh
    ? ['日', '一', '二', '三', '四', '五', '六']
    : ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']

  const todayString = useMemo(() => {
    const now = new Date()
    const month = String(now.getMonth() + 1).padStart(2, '0')
    const day = String(now.getDate()).padStart(2, '0')
    return `${now.getFullYear()}-${month}-${day}`
  }, [])

  const cells = useMemo<(CheckinDashboardDay | null)[]>(() => {
    if (!calendar) return []
    const firstWeekday = new Date(calendar.year, calendar.month - 1, 1).getDay()
    const blanks: (CheckinDashboardDay | null)[] = Array.from({ length: firstWeekday }, () => null)
    return blanks.concat(calendar.days)
  }, [calendar])

  if (!calendar || cells.length === 0) {
    return (
      <div className="calendar-wrapper">
        <div className="calendar-empty">{tt('暂无日历数据', 'No calendar data yet')}</div>
      </div>
    )
  }

  return (
    <div className="calendar-wrapper">
      <div className="calendar-weekdays">
        {weekLabels.map((label) => (
          <div key={label} className="weekday-label">
            {label}
          </div>
        ))}
      </div>
      <div className="calendar-grid">
        {cells.map((cell, index) => (
          <CalendarCell
            key={cell ? cell.date : `pad-${calendar.year}-${calendar.month}-${index}`}
            cell={cell}
            today={todayString}
            isZh={isZh}
          />
        ))}
      </div>
      <div className="calendar-legend">
        <div className="legend-item">
          <span className="legend-dot checked" />
          {tt('已签到', 'Checked in')}
        </div>
        <div className="legend-item">
          <span className="legend-dot unchecked" />
          {tt('未签到', 'Missed')}
        </div>
        <div className="legend-item">
          <span className="legend-dot today" />
          {tt('今天', 'Today')}
        </div>
      </div>
    </div>
  )
}

function CalendarCell({
  cell,
  today,
  isZh,
}: {
  cell: CheckinDashboardDay | null
  today: string
  isZh: boolean
}) {
  if (!cell) return <div className="calendar-cell cell-empty" />
  const reward = dayReward(cell)
  const status = cell.is_checked_in
    ? isZh
      ? '已签到'
      : 'Checked in'
    : isZh
      ? '未签到'
      : 'Missed'
  const title = isZh
    ? `${cell.date} · ${status} · 奖励 ${reward !== null ? `+${reward.toFixed(2)}` : '-'}`
    : `${cell.date} · ${status} · Reward ${reward !== null ? `+${reward.toFixed(2)}` : '-'}`
  const classes = [
    'calendar-cell',
    cell.is_checked_in ? 'cell-checked' : 'cell-unchecked',
    cell.date === today ? 'cell-today' : '',
  ]
    .filter(Boolean)
    .join(' ')
  return (
    <div className={classes} title={title}>
      <span className="day-number">{Number(cell.date.slice(8, 10))}</span>
      {reward !== null ? <span className="day-increment">+{reward.toFixed(2)}</span> : null}
      {reward === null && cell.is_checked_in ? <span className="day-dot">·</span> : null}
    </div>
  )
}
