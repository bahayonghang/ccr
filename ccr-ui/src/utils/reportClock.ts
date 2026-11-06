/** Explicit benchmark report context; elapsed timers and the global Date stay real. */
export interface CapturedReportClock {
  readonly asOfUtc: string
  readonly timezone: 'UTC'
}

declare global {
  interface Window {
    readonly __CCR_REPORT_CLOCK__?: unknown
  }
}

export const capturedReportClock = (): CapturedReportClock | undefined => {
  if (typeof window === 'undefined' || window.__CCR_REPORT_CLOCK__ === undefined) return undefined
  const value = window.__CCR_REPORT_CLOCK__
  if (typeof value !== 'object' || value === null
    || !('asOfUtc' in value) || typeof value.asOfUtc !== 'string'
    || !('timezone' in value) || value.timezone !== 'UTC'
    || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,9})?(?:Z|\+00:00)$/.test(value.asOfUtc)
    || !Number.isFinite(Date.parse(value.asOfUtc))
    || new Date(value.asOfUtc).toISOString().slice(0, 19) !== value.asOfUtc.slice(0, 19)) {
    throw new Error('Invalid benchmark report clock')
  }
  return { asOfUtc: value.asOfUtc, timezone: 'UTC' }
}

export const reportNow = () => {
  const clock = capturedReportClock()
  return clock ? new Date(clock.asOfUtc) : new Date()
}
