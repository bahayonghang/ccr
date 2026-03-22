import type { HeatmapData } from '@/api'

export interface ActivityHeatmapDayData {
  date: string
  dateKey: string
  count: number
  level: number
  isToday: boolean
}

export interface ActivityHeatmapMonthLabel {
  name: string
  weekOffset: number
}

export interface ActivityHeatmapTooltipState {
  visible: boolean
  date: string
  count: number
  x: number
  y: number
}

export interface ActivityHeatmapStatItem {
  id: 'activeDays' | 'totalTokens'
  label: string
  value: string
}

export type ActivityHeatmapData = HeatmapData
