import { useCallback, useEffect, useRef, useState } from 'react'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { startUsageImportJobV2 } from '@/api'
import { getErrorMessage } from '@/utils/errorHandler'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import {
  buildImportSummary,
  normalizeUserVisibleImportJob,
} from '@/utils/usageImportNormalization'
import type {
  UsageImportJobSnapshot,
  UsageImportResult,
  UsageImportSummary,
  UsagePlatform,
} from '@/types/usage'

const isTerminalJob = (job: UsageImportJobSnapshot | null) =>
  job?.status === 'finished' || job?.status === 'failed' || job?.status === 'cancelled'

export function useUsageImport(onRefresh: () => Promise<void>) {
  const [importing, setImporting] = useState(false)
  const [isBootstrapping, setIsBootstrapping] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [warning, setWarning] = useState<string | null>(null)
  const [lastImportSummary, setLastImportSummary] = useState<UsageImportSummary | null>(null)
  const [lastImportResults, setLastImportResults] = useState<UsageImportResult[]>([])
  const [currentImportJob, setCurrentImportJob] = useState<UsageImportJobSnapshot | null>(null)
  const unlistenersRef = useRef<UnlistenFn[]>([])
  const activeReasonRef = useRef<'manual' | 'bootstrap' | null>(null)

  const clearListeners = useCallback(async () => {
    const current = unlistenersRef.current
    unlistenersRef.current = []
    await Promise.all(current.map((unlisten) => unlisten()))
  }, [])

  const applyJob = useCallback((job: UsageImportJobSnapshot) => {
    const visible = normalizeUserVisibleImportJob(job)
    const summary = visible.summary ?? (visible.results.length > 0 ? buildImportSummary(visible.results) : null)
    setCurrentImportJob({ ...visible, summary: summary ?? visible.summary })
    setImporting(!isTerminalJob(job))
    setIsBootstrapping(activeReasonRef.current === 'bootstrap' && !isTerminalJob(job))
    if (visible.results.length > 0) setLastImportResults(visible.results)
    if (summary) {
      setLastImportSummary(summary)
      const failedDetails = visible.results
        .filter((result) => result.error)
        .map((result) => `${result.platform}: ${result.error}`)
        .join('\n')
      if (summary.failure_count === visible.results.length && visible.results.length > 0) {
        setError(failedDetails || job.error || '未能导入本地 usage 日志，请检查日志目录或导入错误')
        setWarning(null)
      } else if (summary.has_partial) {
        setWarning(failedDetails || visible.warnings[0] || '仅导入部分 usage 数据，可重试继续导入')
      } else {
        setWarning(null)
      }
    }
    if (job.status === 'failed') setError(visible.error || '后台导入任务失败')
  }, [])

  const startImportJob = useCallback(async (opts: {
    platform?: UsagePlatform
    recentDays?: number
    reason?: 'manual' | 'bootstrap'
    resetSources?: boolean
  }) => {
    const reason = opts.reason ?? 'manual'
    activeReasonRef.current = reason
    setImporting(true)
    setIsBootstrapping(reason === 'bootstrap')
    setError(null)
    setWarning(null)
    setLastImportSummary(null)
    setLastImportResults([])

    try {
      const response = await startUsageImportJobV2(opts.platform, opts.recentDays, opts.resetSources)
      applyJob(response.snapshot)
      if (!isTauriRuntime()) return response.snapshot

      await clearListeners()
      unlistenersRef.current = await Promise.all([
        listen<UsageImportJobSnapshot>('usage:job-progress', (event) => {
          if (event.payload.job_id === response.job_id) applyJob(event.payload)
        }),
        listen<UsageImportJobSnapshot>('usage:job-recent-ready', (event) => {
          if (event.payload.job_id !== response.job_id) return
          applyJob(event.payload)
          void onRefresh()
        }),
        listen<UsageImportJobSnapshot>('usage:job-finished', (event) => {
          if (event.payload.job_id !== response.job_id) return
          applyJob(event.payload)
          activeReasonRef.current = null
          void onRefresh()
          void clearListeners()
        }),
        listen<UsageImportJobSnapshot>('usage:job-failed', (event) => {
          if (event.payload.job_id !== response.job_id) return
          applyJob(event.payload)
          activeReasonRef.current = null
          void clearListeners()
        }),
      ])
      return response.snapshot
    } catch (caught) {
      const message = getErrorMessage(caught)
      setError(message)
      setImporting(false)
      setIsBootstrapping(false)
      activeReasonRef.current = null
      throw caught
    }
  }, [applyJob, clearListeners, onRefresh])

  useEffect(() => () => {
    void clearListeners()
  }, [clearListeners])

  return {
    importing,
    isBootstrapping,
    error,
    warning,
    lastImportSummary,
    lastImportResults,
    currentImportJob,
    startImportJob,
    setError,
  }
}
