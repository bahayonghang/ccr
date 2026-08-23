import type { TranslateFunction } from '@/utils/tf'

export const TRAY_PANEL_MANUAL_MOVE_THRESHOLD_PX = 12

export interface TrayPanelWindowPosition {
  x: number
  y: number
}

export const shouldPersistTrayPanelManualPosition = (
  beforePosition: TrayPanelWindowPosition | null,
  afterPosition: TrayPanelWindowPosition,
): boolean => {
  if (!beforePosition) return true
  return (
    Math.abs(afterPosition.x - beforePosition.x) >= TRAY_PANEL_MANUAL_MOVE_THRESHOLD_PX
    || Math.abs(afterPosition.y - beforePosition.y) >= TRAY_PANEL_MANUAL_MOVE_THRESHOLD_PX
  )
}

export const quotaScale = (value: number) => Math.min(Math.max(value, 0), 100) / 100

export const quotaToneClass = (value: number) => {
  if (value >= 85) return 'tray-overview__quota-card--critical'
  if (value >= 60) return 'tray-overview__quota-card--warning'
  return 'tray-overview__quota-card--healthy'
}

export function formatReset(t: TranslateFunction, timestamp: number, detailed = false): string {
  const remaining = timestamp - Math.floor(Date.now() / 1000)
  if (remaining <= 0) return t('codex.auth.resetDone')
  const hours = Math.floor(remaining / 3600)
  const minutes = Math.floor((remaining % 3600) / 60)
  if (detailed && hours >= 24) {
    const days = Math.floor(hours / 24)
    return `${days}d ${hours % 24}h ${minutes}m`
  }
  if (hours > 0) return `${hours}h ${minutes}m`
  return `${minutes}m`
}

export const traySnapshotKey = ['codex', 'tray', 'snapshot'] as const
