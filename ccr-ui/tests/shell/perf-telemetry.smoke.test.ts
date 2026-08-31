import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const loggerMocks = vi.hoisted(() => ({
  info: vi.fn(),
}))

const runtimeMocks = vi.hoisted(() => ({
  isTauriRuntime: vi.fn(() => false),
}))

vi.mock('@/utils/logger', () => ({
  logger: loggerMocks,
}))

vi.mock('@/utils/tauriRuntime', () => runtimeMocks)

interface ObserverHarness {
  type: string
  emit: (entries: PerformanceEntry[]) => void
}

const observerHarnesses: ObserverHarness[] = []

class PerformanceObserverStub {
  static supportedEntryTypes = [
    'paint',
    'largest-contentful-paint',
    'layout-shift',
    'event',
    'longtask',
  ]

  private type = ''

  constructor(private readonly callback: PerformanceObserverCallback) {}

  observe(options: PerformanceObserverInit) {
    this.type = options.type ?? ''
    observerHarnesses.push({
      type: this.type,
      emit: (entries) => {
        this.callback(
          {
            getEntries: () => entries,
          } as PerformanceObserverEntryList,
          this as unknown as PerformanceObserver
        )
      },
    })
  }

  disconnect() {}
  takeRecords(): PerformanceEntryList {
    return []
  }
}

const createPerformance = () => {
  let nextMeasure: PerformanceEntry[] = [{ name: 'measure', duration: 12.6 } as PerformanceEntry]
  const navigation = {
    type: 'reload',
    redirectCount: 1,
    startTime: 0.4,
    domContentLoadedEventEnd: 20.6,
    loadEventEnd: 31.2,
    transferSize: 1200,
    encodedBodySize: 900,
    decodedBodySize: 1800,
  } as PerformanceNavigationTiming

  return {
    api: {
      now: vi.fn(() => 42.4),
      mark: vi.fn(),
      measure: vi.fn(),
      getEntriesByName: vi.fn(() => nextMeasure),
      getEntriesByType: vi.fn((type: string) => (type === 'navigation' ? [navigation] : [])),
    } as unknown as Performance,
    setMeasureEntries: (entries: PerformanceEntry[]) => {
      nextMeasure = entries
    },
  }
}

const loadTelemetry = async () => {
  vi.resetModules()
  return import('@/utils/perfTelemetry')
}

beforeEach(() => {
  vi.clearAllMocks()
  observerHarnesses.length = 0
  window.history.replaceState({}, '', '/')
  window.localStorage.clear()
  runtimeMocks.isTauriRuntime.mockReturnValue(false)
})

afterEach(() => {
  vi.unstubAllGlobals()
  vi.restoreAllMocks()
})

describe('performance telemetry smoke', () => {
  it('honors every query override before falling back to persisted settings', async () => {
    const telemetry = await loadTelemetry()

    for (const value of ['1', 'true', 'on', 'yes', ' TRUE ']) {
      window.history.replaceState({}, '', `/?perf=${encodeURIComponent(value)}`)
      expect(telemetry.isPerfTelemetryEnabled()).toBe(true)
    }
    for (const value of ['0', 'false', 'off', 'no', ' FALSE ']) {
      window.history.replaceState({}, '', `/?perf=${encodeURIComponent(value)}`)
      expect(telemetry.isPerfTelemetryEnabled()).toBe(false)
    }

    window.history.replaceState({}, '', '/?perf=maybe')
    expect(telemetry.isPerfTelemetryEnabled()).toBe(false)
    window.localStorage.setItem('ccr-ui:perf', '1')
    expect(telemetry.isPerfTelemetryEnabled()).toBe(true)

    telemetry.setPerfTelemetryEnabled(false)
    expect(window.localStorage.getItem('ccr-ui:perf')).toBeNull()
    telemetry.setPerfTelemetryEnabled(true)
    expect(window.localStorage.getItem('ccr-ui:perf')).toBe('1')
  })

  it('collects observer, mark, measure, route, navigation, and sampling data', async () => {
    window.localStorage.setItem('ccr-ui:perf', '1')
    runtimeMocks.isTauriRuntime.mockReturnValue(true)
    const performanceHarness = createPerformance()
    vi.stubGlobal('performance', performanceHarness.api)
    vi.stubGlobal('PerformanceObserver', PerformanceObserverStub)
    const telemetry = await loadTelemetry()

    expect(telemetry.initPerfTelemetry()).toBe(true)
    expect(telemetry.initPerfTelemetry()).toBe(true)
    expect(observerHarnesses.map((observer) => observer.type)).toEqual([
      'paint',
      'largest-contentful-paint',
      'layout-shift',
      'event',
      'longtask',
    ])

    observerHarnesses.find((observer) => observer.type === 'paint')?.emit([
      { name: 'first-paint', startTime: 7.2 } as PerformanceEntry,
      { name: 'first-contentful-paint', startTime: 9.8 } as PerformanceEntry,
      { name: 'other-paint', startTime: 11 } as PerformanceEntry,
    ])
    observerHarnesses.find((observer) => observer.type === 'largest-contentful-paint')?.emit([])
    observerHarnesses.find((observer) => observer.type === 'largest-contentful-paint')?.emit([
      { name: 'largest-contentful-paint', startTime: 15.5 } as PerformanceEntry,
    ])
    observerHarnesses.find((observer) => observer.type === 'layout-shift')?.emit([
      { hadRecentInput: true, value: 0.7 } as unknown as PerformanceEntry,
      { hadRecentInput: false, value: 0.12 } as unknown as PerformanceEntry,
      { hadRecentInput: false } as unknown as PerformanceEntry,
    ])
    observerHarnesses.find((observer) => observer.type === 'event')?.emit([
      { interactionId: 0, duration: 99 } as unknown as PerformanceEntry,
      { interactionId: 1 } as unknown as PerformanceEntry,
      { interactionId: 1, duration: 44.6 } as unknown as PerformanceEntry,
      { interactionId: 2, duration: 20 } as unknown as PerformanceEntry,
    ])
    observerHarnesses.find((observer) => observer.type === 'longtask')?.emit([
      { duration: 40 } as PerformanceEntry,
      { duration: 80 } as PerformanceEntry,
    ])

    for (let index = 0; index < 125; index += 1) {
      telemetry.perfMark(`mark-${index}`)
    }
    expect(performanceHarness.api.mark).toHaveBeenCalledTimes(125)

    expect(telemetry.perfMeasure('measure-a', 'start', 'end')).toBe(13)
    expect(telemetry.perfMeasure('measure-b', 'start')).toBe(13)
    performanceHarness.setMeasureEntries([])
    expect(telemetry.perfMeasure('missing', 'start')).toBeNull()

    for (let index = 0; index < 31; index += 1) {
      telemetry.recordRouteTiming(`/from-${index}`, `/to-${index}`, index === 0 ? -4 : index * 10)
    }
    telemetry.recordRouteTiming('/slow', '/route', 300.4)
    expect(loggerMocks.info).toHaveBeenCalledWith(
      '[Perf]',
      expect.objectContaining({ scope: 'route', durationMs: 300 })
    )

    expect(telemetry.shouldLogPerfTelemetry()).toBe(true)
    vi.spyOn(Math, 'random').mockReturnValueOnce(0).mockReturnValueOnce(1)
    expect(telemetry.samplePerfTelemetryRate(-1)).toBe(true)
    expect(telemetry.samplePerfTelemetryRate(2)).toBe(true)

    telemetry.flushPerfTelemetry('manual')
    const snapshot: unknown =
      loggerMocks.info.mock.calls[loggerMocks.info.mock.calls.length - 1]?.[1]
    expect(snapshot).toMatchObject({
      env: 'tauri',
      reason: 'manual',
      paint: { fpMs: 7.2, fcpMs: 9.8, lcpMs: 15.5 },
      cls: 0.12,
      inpMs: 45,
      longTasks: { count: 2, totalDurationMs: 120, maxDurationMs: 80 },
      navigation: {
        type: 'reload',
        redirectCount: 1,
        startTimeMs: 0,
        domContentLoadedMs: 21,
        loadEventMs: 31,
      },
    })
    expect((snapshot as { marks: unknown[] }).marks).toHaveLength(120)
    expect((snapshot as { routes: unknown[] }).routes).toHaveLength(30)

    const beforeOnce = loggerMocks.info.mock.calls.length
    telemetry.flushPerfTelemetryOnce('route-ready')
    telemetry.flushPerfTelemetryOnce('route-ready')
    expect(loggerMocks.info.mock.calls.length).toBe(beforeOnce + 1)
  })

  it('handles unavailable and throwing browser performance APIs without surfacing errors', async () => {
    window.localStorage.setItem('ccr-ui:perf', '1')
    const performanceHarness = createPerformance()
    performanceHarness.api.mark = vi.fn(() => {
      throw new Error('mark denied')
    })
    performanceHarness.api.measure = vi.fn(() => {
      throw new Error('measure denied')
    })
    performanceHarness.api.getEntriesByType = vi.fn(() => [])
    vi.stubGlobal('performance', performanceHarness.api)

    class ThrowingObserver {
      static supportedEntryTypes = ['paint']
      constructor() {
        throw new Error('observer denied')
      }
    }
    vi.stubGlobal('PerformanceObserver', ThrowingObserver)
    const telemetry = await loadTelemetry()

    expect(telemetry.initPerfTelemetry()).toBe(true)
    telemetry.perfMark('safe-mark')
    expect(telemetry.perfMeasure('safe-measure', 'start')).toBeNull()
    telemetry.flushPerfTelemetry('observer-error')
    expect(loggerMocks.info).toHaveBeenLastCalledWith(
      '[Perf]',
      expect.objectContaining({
        reason: 'observer-error',
        observerError: 'observer denied',
        navigation: undefined,
      })
    )

    vi.stubGlobal('performance', undefined)
    expect(telemetry.perfMeasure('no-performance', 'start')).toBeNull()
  })

  it('keeps all collection APIs inert while telemetry is disabled', async () => {
    const performanceHarness = createPerformance()
    vi.stubGlobal('performance', performanceHarness.api)
    vi.stubGlobal('PerformanceObserver', undefined)
    const telemetry = await loadTelemetry()

    expect(telemetry.initPerfTelemetry()).toBe(false)
    telemetry.perfMark('ignored')
    expect(telemetry.perfMeasure('ignored', 'start')).toBeNull()
    telemetry.recordRouteTiming('/ignored', '/ignored', 100)
    telemetry.flushPerfTelemetry('ignored')
    telemetry.flushPerfTelemetryOnce('ignored')
    expect(telemetry.shouldLogPerfTelemetry()).toBe(false)
    expect(telemetry.samplePerfTelemetryRate()).toBe(false)
    expect(performanceHarness.api.mark).not.toHaveBeenCalled()
    expect(loggerMocks.info).not.toHaveBeenCalled()
  })

  it('ignores storage failures and safely no-ops without a window', async () => {
    const getItem = vi.spyOn(window.localStorage, 'getItem').mockImplementation(() => {
      throw new Error('storage denied')
    })
    const setItem = vi.spyOn(window.localStorage, 'setItem').mockImplementation(() => {
      throw new Error('storage denied')
    })
    const removeItem = vi.spyOn(window.localStorage, 'removeItem').mockImplementation(() => {
      throw new Error('storage denied')
    })
    const telemetry = await loadTelemetry()

    expect(telemetry.isPerfTelemetryEnabled()).toBe(false)
    expect(() => telemetry.setPerfTelemetryEnabled(true)).not.toThrow()
    expect(() => telemetry.setPerfTelemetryEnabled(false)).not.toThrow()
    expect(getItem).toHaveBeenCalled()
    expect(setItem).toHaveBeenCalled()
    expect(removeItem).toHaveBeenCalled()

    vi.stubGlobal('window', undefined)
    expect(telemetry.isPerfTelemetryEnabled()).toBe(false)
    expect(() => telemetry.setPerfTelemetryEnabled(true)).not.toThrow()
  })
})
