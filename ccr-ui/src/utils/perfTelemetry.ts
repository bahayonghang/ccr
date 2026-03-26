import { logger } from '@/utils/logger'
import { isTauriRuntime } from '@/utils/tauriRuntime'

export type PerfEnvironment = 'tauri' | 'web'

export interface PerfRouteTiming {
  id: number
  from: string
  to: string
  durationMs: number
  ts: string
}

interface PerfMarkRecord {
  name: string
  atMs: number
}

interface PerfMeasureRecord {
  name: string
  durationMs: number
}

interface PerfLongTaskSummary {
  count: number
  totalDurationMs: number
  maxDurationMs: number
}

interface PerfTelemetrySnapshot {
  env: PerfEnvironment
  build: 'dev' | 'prod'
  reason: string
  ts: string
  paint?: {
    fpMs?: number
    fcpMs?: number
    lcpMs?: number
  }
  cls?: number
  inpMs?: number
  longTasks?: PerfLongTaskSummary
  marks?: PerfMarkRecord[]
  measures?: PerfMeasureRecord[]
  routes?: PerfRouteTiming[]
  navigation?: {
    type?: string
    redirectCount?: number
    startTimeMs?: number
    domContentLoadedMs?: number
    loadEventMs?: number
    transferSize?: number
    encodedBodySize?: number
    decodedBodySize?: number
  }
}

const PERF_STORAGE_KEY = 'ccr-ui:perf'

const nowMs = (): number => {
  if (typeof performance !== 'undefined' && typeof performance.now === 'function') {
    return performance.now()
  }
  return Date.now()
}

const clamp = (value: number, min: number, max: number): number => {
  return Math.max(min, Math.min(max, value))
}

const readPerfQueryOverride = (): boolean | null => {
  if (typeof window === 'undefined') return null

  try {
    const params = new URLSearchParams(window.location.search)
    const raw = params.get('perf')
    if (!raw) return null

    const value = raw.trim().toLowerCase()
    if (value === '1' || value === 'true' || value === 'on' || value === 'yes') return true
    if (value === '0' || value === 'false' || value === 'off' || value === 'no') return false
    return null
  } catch {
    return null
  }
}

const readPerfLocalStorage = (): boolean => {
  if (typeof window === 'undefined') return false

  try {
    return window.localStorage.getItem(PERF_STORAGE_KEY) === '1'
  } catch {
    return false
  }
}

export const isPerfTelemetryEnabled = (): boolean => {
  const queryOverride = readPerfQueryOverride()
  if (queryOverride !== null) return queryOverride
  return readPerfLocalStorage()
}

export const setPerfTelemetryEnabled = (enabled: boolean): void => {
  if (typeof window === 'undefined') return

  try {
    if (enabled) {
      window.localStorage.setItem(PERF_STORAGE_KEY, '1')
    } else {
      window.localStorage.removeItem(PERF_STORAGE_KEY)
    }
  } catch {
    // ignore
  }
}

interface PerfTelemetryState {
  enabled: boolean
  env: PerfEnvironment
  build: 'dev' | 'prod'
  marks: PerfMarkRecord[]
  measures: PerfMeasureRecord[]
  routes: PerfRouteTiming[]
  routeSequence: number
  paint: {
    fpMs?: number
    fcpMs?: number
    lcpMs?: number
  }
  cls: number
  inpMs?: number
  longTasks: PerfLongTaskSummary
  observers: PerformanceObserver[]
  observerError?: string
}

let perfState: PerfTelemetryState | null = null

const buildBaseState = (): PerfTelemetryState => ({
  enabled: isPerfTelemetryEnabled(),
  env: isTauriRuntime() ? 'tauri' : 'web',
  build: import.meta.env.DEV ? 'dev' : 'prod',
  marks: [],
  measures: [],
  routes: [],
  routeSequence: 0,
  paint: {},
  cls: 0,
  longTasks: { count: 0, totalDurationMs: 0, maxDurationMs: 0 },
  observers: [],
})

const supportedEntryTypes = (): string[] => {
  if (typeof PerformanceObserver === 'undefined') return []

  const types = (PerformanceObserver as typeof PerformanceObserver & {
    supportedEntryTypes?: string[]
  }).supportedEntryTypes

  return Array.isArray(types) ? types : []
}

const observePerformance = (
  state: PerfTelemetryState,
  type: string,
  callback: (entries: PerformanceEntry[]) => void,
): void => {
  if (typeof PerformanceObserver === 'undefined') return

  const supported = supportedEntryTypes()
  if (supported.length > 0 && !supported.includes(type)) return

  try {
    const observer = new PerformanceObserver((list) => {
      callback(list.getEntries())
    })
    observer.observe({ type, buffered: true })
    state.observers.push(observer)
  } catch (error) {
    state.observerError = error instanceof Error ? error.message : String(error)
  }
}

const setupPerfObservers = (state: PerfTelemetryState): void => {
  if (typeof performance === 'undefined') return

  observePerformance(state, 'paint', (entries) => {
    for (const entry of entries) {
      if (entry.name === 'first-paint') {
        state.paint.fpMs = entry.startTime
      }
      if (entry.name === 'first-contentful-paint') {
        state.paint.fcpMs = entry.startTime
      }
    }
  })

  observePerformance(state, 'largest-contentful-paint', (entries) => {
    const last = entries.length > 0 ? entries[entries.length - 1] : null
    if (last) {
      state.paint.lcpMs = last.startTime
    }
  })

  observePerformance(state, 'layout-shift', (entries) => {
    for (const rawEntry of entries) {
      const entry = rawEntry as PerformanceEntry & {
        value?: number
        hadRecentInput?: boolean
      }
      if (entry.hadRecentInput) continue
      if (typeof entry.value === 'number') {
        state.cls += entry.value
      }
    }
  })

  observePerformance(state, 'event', (entries) => {
    for (const rawEntry of entries) {
      const entry = rawEntry as PerformanceEntry & {
        interactionId?: number
        duration?: number
      }

      if (typeof entry.interactionId !== 'number' || entry.interactionId <= 0) continue
      if (typeof entry.duration !== 'number') continue

      state.inpMs = Math.max(state.inpMs ?? 0, entry.duration)
    }
  })

  observePerformance(state, 'longtask', (entries) => {
    for (const entry of entries) {
      state.longTasks.count += 1
      state.longTasks.totalDurationMs += entry.duration
      state.longTasks.maxDurationMs = Math.max(state.longTasks.maxDurationMs, entry.duration)
    }
  })
}

const ensurePerfState = (): PerfTelemetryState => {
  if (perfState) return perfState

  perfState = buildBaseState()

  if (perfState.enabled) {
    setupPerfObservers(perfState)
  }

  return perfState
}

export const initPerfTelemetry = (): boolean => {
  return ensurePerfState().enabled
}

export const perfMark = (name: string): void => {
  const state = ensurePerfState()
  if (!state.enabled) return

  const atMs = nowMs()
  state.marks.push({ name, atMs })

  if (state.marks.length > 120) {
    state.marks.splice(0, state.marks.length - 120)
  }

  if (typeof performance !== 'undefined' && typeof performance.mark === 'function') {
    try {
      performance.mark(name)
    } catch {
      // ignore
    }
  }
}

export const perfMeasure = (name: string, startMark: string, endMark?: string): number | null => {
  const state = ensurePerfState()
  if (!state.enabled) return null

  if (typeof performance === 'undefined' || typeof performance.measure !== 'function') {
    return null
  }

  try {
    if (endMark) {
      performance.measure(name, startMark, endMark)
    } else {
      performance.measure(name, startMark)
    }

    const entries = performance.getEntriesByName(name, 'measure')
    const last = entries.length > 0 ? entries[entries.length - 1] : null
    if (!last) return null

    const durationMs = Math.round(last.duration)
    state.measures.push({ name, durationMs })
    if (state.measures.length > 80) {
      state.measures.splice(0, state.measures.length - 80)
    }

    return durationMs
  } catch {
    return null
  }
}

export const recordRouteTiming = (from: string, to: string, durationMs: number): void => {
  const state = ensurePerfState()
  if (!state.enabled) return

  state.routeSequence += 1

  const record: PerfRouteTiming = {
    id: state.routeSequence,
    from,
    to,
    durationMs: Math.max(0, Math.round(durationMs)),
    ts: new Date().toISOString(),
  }

  state.routes.push(record)

  if (state.routes.length > 30) {
    state.routes.splice(0, state.routes.length - 30)
  }

  if (record.durationMs >= 250) {
    logger.info('[Perf]', { scope: 'route', ...record })
  }
}

const readNavigationTiming = (): PerfTelemetrySnapshot['navigation'] | undefined => {
  if (typeof performance === 'undefined') return undefined

  const entry = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming | undefined
  if (!entry) return undefined

  return {
    type: entry.type,
    redirectCount: entry.redirectCount,
    startTimeMs: Math.round(entry.startTime),
    domContentLoadedMs: Math.round(entry.domContentLoadedEventEnd),
    loadEventMs: Math.round(entry.loadEventEnd),
    transferSize: entry.transferSize,
    encodedBodySize: entry.encodedBodySize,
    decodedBodySize: entry.decodedBodySize,
  }
}

export const flushPerfTelemetry = (reason: string): void => {
  const state = ensurePerfState()
  if (!state.enabled) return

  const snapshot: PerfTelemetrySnapshot = {
    env: state.env,
    build: state.build,
    reason,
    ts: new Date().toISOString(),
    paint: state.paint,
    cls: Number(state.cls.toFixed(4)),
    inpMs: typeof state.inpMs === 'number' ? Math.round(state.inpMs) : undefined,
    longTasks: {
      count: state.longTasks.count,
      totalDurationMs: Math.round(state.longTasks.totalDurationMs),
      maxDurationMs: Math.round(state.longTasks.maxDurationMs),
    },
    marks: state.marks
      .slice()
      .sort((a, b) => a.atMs - b.atMs)
      .map((mark) => ({ name: mark.name, atMs: Math.round(mark.atMs) })),
    measures: state.measures.map((m) => ({ name: m.name, durationMs: Math.round(m.durationMs) })),
    routes: state.routes,
    navigation: readNavigationTiming(),
  }

  if (state.observerError) {
    logger.info('[Perf]', { ...snapshot, observerError: state.observerError })
    return
  }

  logger.info('[Perf]', snapshot)
}

export const shouldLogPerfTelemetry = (): boolean => {
  const state = ensurePerfState()
  if (!state.enabled) return false

  const override = readPerfQueryOverride()
  if (override !== null) return override

  // In prod builds, only log when explicitly persisted/enabled.
  return state.build === 'dev' || readPerfLocalStorage()
}

const flushedReasons = new Set<string>()

export const flushPerfTelemetryOnce = (reason: string): void => {
  if (!shouldLogPerfTelemetry()) return
  if (flushedReasons.has(reason)) return
  flushedReasons.add(reason)
  flushPerfTelemetry(reason)
}

export const samplePerfTelemetryRate = (rate = 1): boolean => {
  const state = ensurePerfState()
  if (!state.enabled) return false

  const probability = clamp(rate, 0, 1)
  return Math.random() <= probability
}
