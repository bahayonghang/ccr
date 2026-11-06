import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { memo, useCallback, useEffect, useRef, useState } from 'react'
import { cancelBenchmarkS2, getBenchmarkS2Status, startBenchmarkS2 } from '@/api/domains/benchmark'
import { surfaceNotify } from '@/configs/surfaceNotify'
import type { BenchmarkS2SnapshotDto } from '@/types/generated/benchmark/BenchmarkS2SnapshotDto'
import { Button } from '@/ui'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import type { TranslateFunction } from '@/utils/tf'
import { useMonitoringLocale, useMonitoringT } from './locale'

const FAILURE_MESSAGES: Record<string, string> = {
  benchmark_s2_cleanup_over_budget: 'overBudget',
  benchmark_s2_delivery_failed: 'deliveryFailed',
  benchmark_s2_worker_failed: 'workerFailed',
  benchmark_s2_producer_failed: 'producerFailed',
  benchmark_s2_not_started: 'notStarted',
}

function isS2Enabled() {
  return isTauriRuntime() && (window as Window & { __CCR_BENCHMARK_S2__?: unknown }).__CCR_BENCHMARK_S2__ === true
}

function canStartS2(snapshot: BenchmarkS2SnapshotDto | null) {
  return isS2Enabled() && snapshot?.phase === 'not_started'
}

export function BenchmarkS2Panel() {
  return isS2Enabled() ? <ActiveBenchmarkS2Panel /> : null
}

function useBenchmarkS2(t: TranslateFunction) {
  const [snapshot, setSnapshot] = useState<BenchmarkS2SnapshotDto | null>(null)
  const [connected, setConnected] = useState(false)
  const [connection, setConnection] = useState(0)
  const [pending, setPending] = useState(false)
  const [stopRequested, setStopRequested] = useState(false)
  const [requestFailed, setRequestFailed] = useState(false)
  const latest = useRef<BenchmarkS2SnapshotDto | null>(null)
  const mounted = useRef(false)
  const operation = useRef(false)
  const actionGeneration = useRef(0)

  // Stable listener identity keeps counter updates from resubscribing the feed.
  const accept = useCallback((incoming: BenchmarkS2SnapshotDto) => {
    if (!mounted.current) return false
    const previous = latest.current
    if (previous && (incoming.revision <= previous.revision
      || (previous.run_id !== null && incoming.run_id !== previous.run_id))) return false
    latest.current = incoming
    setSnapshot(incoming)
    return true
  }, [])

  useEffect(() => {
    let disposed = false
    let detach: UnlistenFn | undefined
    mounted.current = true
    setConnected(false)
    const recoveryGeneration = actionGeneration.current
    void listen<BenchmarkS2SnapshotDto>('benchmark:s2', ({ payload }) => {
      if (!disposed && accept(payload)) setRequestFailed(false)
    }).then(async (unlisten) => {
      if (disposed) return unlisten()
      detach = unlisten
      setConnected(true)
      // Subscribe first: a newer event must win over this recovery response.
      const current = await getBenchmarkS2Status()
      if (!disposed) {
        accept(current)
        // A successful same-revision read restores connectivity without
        // replacing counters. An older action cannot clear a newer refusal.
        if (actionGeneration.current === recoveryGeneration
          && (!latest.current?.run_id || latest.current.run_id === current.run_id)) setRequestFailed(false)
      }
    }).catch(() => {
      if (!disposed && actionGeneration.current === recoveryGeneration) {
        setConnected(false)
        setRequestFailed(true)
      }
    })
    return () => {
      disposed = true
      mounted.current = false
      actionGeneration.current += 1
      detach?.()
    }
  }, [accept, connection])

  const start = useCallback(async () => {
    if (operation.current || !connected || !canStartS2(latest.current)) return
    operation.current = true
    const opening = ++actionGeneration.current
    const trigger = document.getElementById('benchmark-s2-start')
    try {
      const confirmed = await surfaceNotify.confirm({
        title: t('monitoring.s2.confirmTitle'),
        message: t('monitoring.s2.confirmMessage'),
        confirmText: t('monitoring.s2.confirmStart'),
        cancelText: t('monitoring.s2.keepSeed'),
        type: 'warning',
        surface: 'solid',
      })
      if (!confirmed) {
        // The global imperative dialog has no Radix DialogTrigger to restore.
        requestAnimationFrame(() => {
          if (mounted.current && actionGeneration.current === opening && trigger?.isConnected
            && document.activeElement === document.body && !document.querySelector('[role="dialog"]')) trigger.focus()
        })
        return
      }
      if (!mounted.current || actionGeneration.current !== opening || !canStartS2(latest.current)) return
      setPending(true)
      const result = await startBenchmarkS2()
      if (actionGeneration.current === opening) accept(result)
    } catch {
      if (mounted.current && actionGeneration.current === opening) setRequestFailed(true)
    } finally {
      operation.current = false
      if (mounted.current) setPending(false)
    }
  }, [accept, connected, t])

  const cancel = useCallback(async () => {
    const current = latest.current
    if (operation.current || !current?.run_id || !isS2Enabled()
      || !['queued', 'running'].includes(current.phase)) return
    operation.current = true
    const opening = ++actionGeneration.current
    setPending(true)
    try {
      const result = await cancelBenchmarkS2({ run_id: current.run_id })
      if (mounted.current && actionGeneration.current === opening) {
        accept(result)
        setRequestFailed(false)
        setStopRequested(true)
      }
    } catch {
      if (mounted.current && actionGeneration.current === opening) setRequestFailed(true)
    } finally {
      operation.current = false
      if (mounted.current) setPending(false)
    }
  }, [accept])

  const reconnect = useCallback(() => {
    setConnection((value) => value + 1)
  }, [])

  return { snapshot, connected, pending, stopRequested, requestFailed, start, cancel, reconnect }
}

const RATES = [
  ['attemptedRate', 'actual_attempted_per_second', 'sourceRateBasis'],
  ['deliveredRate', 'actual_delivered_per_second', 'deliveryRateBasis'],
  ['committedRate', 'actual_committed_per_second', 'deliveryRateBasis'],
] as const
const COUNTERS = [
  ['scheduled', 'scheduled_slots'], ['attempted', 'attempted'],
  ['skipped', 'scheduler_skipped'], ['queueAccepted', 'queue_accepted'],
  ['queuePending', 'queue_pending'], ['queueDropped', 'queue_dropped'],
  ['queueAbandoned', 'queue_abandoned'], ['ringAccepted', 'ring_accepted'],
  ['ringRejected', 'ring_rejected'], ['ringEvicted', 'ring_evicted'],
  ['dbAttempted', 'db_attempted'], ['dbCommitted', 'db_committed'],
  ['dbFailed', 'db_failed'], ['dbPending', 'db_pending'],
  ['emitAttempted', 'emit_attempted'], ['emitFailed', 'emit_failed'],
  ['unacknowledged', 'unacknowledged'],
] as const

function ActiveBenchmarkS2Panel() {
  const t = useMonitoringT()
  const state = useBenchmarkS2(t)
  return (
    <section aria-labelledby="benchmark-s2-title" aria-busy={state.pending} className="mb-4 rounded-xl border border-border-default/55 bg-bg-elevated p-4">
      <TrialHeader phase={state.snapshot?.phase} connected={state.connected} pending={state.pending} requestFailed={state.requestFailed} stopRequested={state.stopRequested} start={state.start} cancel={state.cancel} reconnect={state.reconnect} t={t} />
      <TrialStatus phase={state.snapshot?.phase} overBudget={state.snapshot?.cleanup_over_budget} failureCode={state.snapshot?.failure_code} requestFailed={state.requestFailed} stopRequested={state.stopRequested} t={t} />
      <TrialMetrics snapshot={state.snapshot} />
    </section>
  )
}

type Phase = BenchmarkS2SnapshotDto['phase'] | undefined
const TrialHeader = memo(function TrialHeader({ phase, connected, pending, requestFailed, stopRequested, start, cancel, reconnect, t }: {
  phase: Phase; connected: boolean; pending: boolean; requestFailed: boolean; stopRequested: boolean
  start: () => Promise<void>; cancel: () => Promise<void>; reconnect: () => void; t: TranslateFunction
}) {
  return (
    <div className="flex flex-wrap items-start justify-between gap-3">
      <div>
        <p className="text-xs font-medium text-text-muted">{t('monitoring.s2.synthetic')}</p>
        <h2 id="benchmark-s2-title" className="mt-1 text-lg font-semibold text-text-primary">{t('monitoring.s2.title')}</h2>
        <p className="mt-1 text-xs text-text-secondary">{t('monitoring.s2.profile')}</p>
      </div>
      <div className="flex flex-wrap gap-2">
        {phase === 'not_started' ? <Button id="benchmark-s2-start" size="sm" variant="primary" onClick={start} disabled={pending || !connected || requestFailed}>{t('monitoring.s2.start')}</Button> : null}
        {(phase === 'queued' || phase === 'running') ? <Button size="sm" onClick={cancel} disabled={pending || stopRequested || !connected || requestFailed}>{t('monitoring.s2.cancel')}</Button> : null}
        {(!connected || requestFailed) ? <Button size="sm" onClick={reconnect}>{t('monitoring.s2.reconnect')}</Button> : null}
      </div>
    </div>
  )
})

const TrialStatus = memo(function TrialStatus({ phase, overBudget, failureCode, requestFailed, stopRequested, t }: {
  phase: Phase; overBudget: boolean | undefined; failureCode: string | null | undefined
  requestFailed: boolean; stopRequested: boolean; t: TranslateFunction
}) {
  const terminal = phase === 'completed' || phase === 'cancelled' || phase === 'failed'
  const failure = FAILURE_MESSAGES[failureCode ?? ''] ?? 'runFailed'
  return (
    <>
      <p role="status" className="mt-3 text-sm font-medium text-text-primary">
        {phase ? t(`monitoring.s2.phases.${phase}`) : t('monitoring.s2.loading')}
      </p>
      {stopRequested && !terminal ? <p className="mt-1 text-xs text-text-secondary">{t('monitoring.s2.stopRequested')}</p> : null}
      {requestFailed ? <p role="alert" className="mt-2 text-sm text-accent-warning">{t('monitoring.s2.requestFailed')}</p> : null}
      {overBudget ? <p className="mt-2 text-sm text-accent-warning">{t('monitoring.s2.overBudget')}</p> : null}
      {phase === 'failed' ? <p className="mt-2 text-sm text-accent-warning">{t(`monitoring.s2.${failure}`)}</p> : null}
      {terminal ? <p className="mt-2 text-xs text-text-secondary">{t('monitoring.s2.finished')}</p> : null}
    </>
  )
})

function TrialMetrics({ snapshot }: { snapshot: BenchmarkS2SnapshotDto | null }) {
  const t = useMonitoringT()
  const locale = useMonitoringLocale()
  const number = (value: number | null | undefined, fraction = 0) => value == null
    ? t('monitoring.s2.notAvailable')
    : new Intl.NumberFormat(locale, { maximumFractionDigits: fraction }).format(value)
  const duration = (value: number | null | undefined) => value == null
    ? t('monitoring.s2.notAvailable') : `${number(value)} ms`

  return (
    <>
      <dl className="mt-4 grid gap-4 sm:grid-cols-3">
        {RATES.map(([label, key, basis]) => (
          <div key={label}>
            <dt title={t(`monitoring.s2.${basis}`)} className="text-xs text-text-muted">{t(`monitoring.s2.${label}`)}</dt>
            <dd className="mt-1 font-mono text-lg tabular-nums text-text-primary">{number(snapshot?.[key], 2)}</dd>
          </div>
        ))}
      </dl>
      <p className="mt-2 text-xs text-text-muted">{t('monitoring.s2.sourceRateBasis')} {t('monitoring.s2.deliveryRateBasis')}</p>
      <div className="mt-4 flex flex-wrap gap-x-5 gap-y-1 text-xs text-text-muted">
        <span>{t('monitoring.s2.elapsed')}: {duration(snapshot?.elapsed_ms)}</span>
        <span>{t('monitoring.s2.sourceElapsed')}: {duration(snapshot?.source_elapsed_ms)}</span>
        <span>{t('monitoring.s2.queueCapacity')}: {number(snapshot?.queue_capacity)}</span>
      </div>
      <details className="mt-4 border-t border-border-default/45 pt-3">
        <summary className="cursor-pointer text-sm font-medium text-text-secondary">{t('monitoring.s2.counters')}</summary>
        <dl className="mt-3 grid grid-cols-2 gap-x-6 gap-y-3 sm:grid-cols-3 xl:grid-cols-6">
          {COUNTERS.map(([label, key]) => (
            <div key={label}>
              <dt className="text-xs text-text-muted">{t(`monitoring.s2.${label}`)}</dt>
              <dd className="mt-1 font-mono text-sm tabular-nums text-text-primary">{number(snapshot?.[key])}</dd>
            </div>
          ))}
        </dl>
      </details>
    </>
  )
}
