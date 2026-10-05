import { useCallback, useMemo, useState } from 'react'
import type { CheckinDashboardTrend, CheckinDashboardTrendPoint } from '@/types/checkin'
import { useCheckinLocale } from '../hooks/useCheckinT'
import '../styles/trend.css'

interface AccountDashboardTrendProps {
  trend: CheckinDashboardTrend | null
}

const WIDTH = 800
const HEIGHT = 220
const PADDING = 20
const CHART_HEIGHT = HEIGHT - PADDING * 2
const CHART_WIDTH = WIDTH - PADDING * 2
const BASELINE_Y = PADDING + CHART_HEIGHT
const MIN_BAR_RATIO = 0.18

export function AccountDashboardTrend({ trend }: AccountDashboardTrendProps) {
  const locale = useCheckinLocale()
  const isZh = locale.startsWith('zh')
  const tt = (zh: string, en: string) => (isZh ? zh : en)
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null)
  const clearHover = useCallback(() => setHoveredIndex(null), [])
  const chartData = useMemo(() => trend?.data_points ?? [], [trend])

  const maxReward = useMemo(() => {
    const max = chartData.reduce((acc, point) => Math.max(acc, point.reward_amount ?? 0), 0)
    return max > 0 ? max : 1
  }, [chartData])
  const allZeroReward = chartData.length === 0 || chartData.every((point) => !point.reward_amount)
  const barGap = chartData.length > 40 ? 2 : 4
  const barWidth =
    chartData.length === 0
      ? 0
      : Math.max((CHART_WIDTH - barGap * (chartData.length - 1)) / chartData.length, 2)

  const barX = (index: number) => PADDING + index * (barWidth + barGap)
  const barH = (point: CheckinDashboardTrendPoint) => {
    if (!point.is_checked_in) return CHART_HEIGHT * MIN_BAR_RATIO
    if (allZeroReward) return CHART_HEIGHT * 0.45
    return CHART_HEIGHT * Math.max(point.reward_amount / maxReward, MIN_BAR_RATIO)
  }

  const midTicks = useMemo(() => {
    if (chartData.length < 30) return []
    const n = chartData.length
    return [Math.floor(n * 0.25), Math.floor(n * 0.5), Math.floor(n * 0.75)]
      .map((i) => chartData[i]?.date ?? '')
      .filter(Boolean)
  }, [chartData])

  const hovered = hoveredIndex !== null ? chartData[hoveredIndex] : null

  if (chartData.length === 0) {
    return <div className="trend-empty">{tt('暂无趋势数据', 'No trend data yet')}</div>
  }

  return (
    <div className="trend-chart-container">
      <svg viewBox={`0 0 ${WIDTH} ${HEIGHT}`} className="trend-svg" preserveAspectRatio="xMidYMid meet">
        <line x1={PADDING} y1={BASELINE_Y} x2={WIDTH - PADDING} y2={BASELINE_Y} className="baseline" />
        {chartData.map((point, index) => (
          <TrendBar
            key={point.date}
            point={point}
            x={barX(index)}
            y={BASELINE_Y - barH(point)}
            width={barWidth}
            height={barH(point)}
            index={index}
            onEnter={setHoveredIndex}
            onLeave={clearHover}
          />
        ))}
      </svg>
      {hovered ? (
        <div className="chart-tooltip">
          <div className="tooltip-date">{hovered.date}</div>
          <div className="tooltip-row">
            <span>{tt('状态', 'Status')}</span>
            <span>{hovered.is_checked_in ? tt('已签到', 'Checked in') : tt('未签到', 'Missed')}</span>
          </div>
          <div className="tooltip-row">
            <span>{tt('本日奖励', 'Reward')}</span>
            <span>{hovered.reward_amount > 0 ? `+${hovered.reward_amount.toFixed(2)}` : '—'}</span>
          </div>
        </div>
      ) : null}
      <div className="chart-axis">
        <span>{trend?.start_date}</span>
        {midTicks.map((tick) => (
          <span key={tick} className="chart-axis-mid">
            {tick}
          </span>
        ))}
        <span>{trend?.end_date}</span>
      </div>
    </div>
  )
}

function TrendBar({
  point,
  x,
  y,
  width,
  height,
  index,
  onEnter,
  onLeave,
}: {
  point: CheckinDashboardTrendPoint
  x: number
  y: number
  width: number
  height: number
  index: number
  onEnter: (index: number) => void
  onLeave: () => void
}) {
  const handleEnter = useCallback(() => {
    onEnter(index)
  }, [index, onEnter])
  const barClass = !point.is_checked_in
    ? 'bar bar-missed'
    : !point.reward_amount
      ? 'bar bar-checked-flat'
      : 'bar bar-checked'
  return (
    <rect
      x={x}
      y={y}
      width={width}
      height={height}
      rx={Math.min(width / 3, 3)}
      className={barClass}
      onMouseEnter={handleEnter}
      onMouseLeave={onLeave}
    />
  )
}
