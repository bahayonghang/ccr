import { getCheckinJobStatus, startCheckinJob } from '@/api'
import { logger } from '@/utils/logger'
import type {
  AccountInfo,
  CheckinDisplayResponse,
  CheckinFlowPhase,
  CheckinJobDelta,
  CheckinJobSnapshot,
  CheckinLogEntry,
  CheckinResponse,
  StartCheckinJobResponse,
} from '@/types/checkin'
import type { CheckinRefreshOptions } from './checkinData'
import { createListenBag, disposeListens, trackListen, type ListenBag } from './listenCancel'
import { isTerminalJobSnapshot, mapCheckinJobLogEntry } from './wafFormat'

type RefreshCheckinData = (options?: CheckinRefreshOptions) => Promise<void>
type RunWafRecovery = (result: CheckinDisplayResponse) => Promise<CheckinDisplayResponse>

export interface CheckinJobBox {
  accounts: AccountInfo[]
  checkinLoading: boolean
  checkinResult: CheckinDisplayResponse | null
  checkinResultRef: HTMLElement | null
  showProgressModal: boolean
  checkinFlowPhase: CheckinFlowPhase
  checkinProgress: { total: number; completed: number; currentAccountName: string }
  checkinLogs: CheckinLogEntry[]
  wafRecoveryRunning: boolean
  wafRecoveryProviderName: string | null
  wafRecoveryMessage: string | null
  activeCheckinJobId: string | null
}

interface JobRuntimeOptions {
  box: CheckinJobBox
  refreshCheckinData: RefreshCheckinData
  runWafRecovery: RunWafRecovery
  notifyJobStartFailed: (error: unknown) => void
  notify: () => void
}

const toDisplayResponse = (response: CheckinResponse): CheckinDisplayResponse => ({
  results: response.results.map((item) => ({ ...item })),
  summary: { ...response.summary },
})

export const createCheckinJobRuntime = (options: JobRuntimeOptions) => {
  const { box, refreshCheckinData, runWafRecovery, notifyJobStartFailed, notify } = options
  let bag: ListenBag = createListenBag()

  const cleanupCheckinJobListeners = async () => {
    disposeListens(bag)
    bag = createListenBag()
  }

  const applyCheckinJobSnapshot = (snapshot: CheckinJobSnapshot) => {
    box.checkinProgress = {
      total: snapshot.total,
      completed: snapshot.completed,
      currentAccountName: snapshot.current_account_name,
    }
    box.checkinLogs = snapshot.logs.map(mapCheckinJobLogEntry)
    if (isTerminalJobSnapshot(snapshot)) {
      box.checkinResult = toDisplayResponse({
        results: snapshot.results,
        summary: snapshot.summary,
      })
    }
    notify()
  }

  const applyCheckinJobDelta = (delta: CheckinJobDelta) => {
    if (box.activeCheckinJobId !== delta.jobId) return
    box.checkinProgress = {
      total: delta.total,
      completed: delta.completed,
      currentAccountName: delta.currentAccountName,
    }
    if (delta.changedLogs.length === 0) {
      notify()
      return
    }
    const nextLogs = [...box.checkinLogs]
    for (const rawLog of delta.changedLogs) {
      const mapped = mapCheckinJobLogEntry(rawLog)
      const idx = nextLogs.findIndex((entry) => entry.accountId === rawLog.account_id)
      if (idx >= 0) nextLogs.splice(idx, 1, mapped)
      else nextLogs.push(mapped)
    }
    box.checkinLogs = nextLogs
    notify()
  }

  const finalizeCheckinJob = async (snapshot: CheckinJobSnapshot) => {
    if (box.activeCheckinJobId !== snapshot.job_id) return
    try {
      applyCheckinJobSnapshot(snapshot)
      box.activeCheckinJobId = null
      await cleanupCheckinJobListeners()
      await refreshCheckinData({
        reloadAccounts: true,
        reloadRecords: true,
        reloadStats: true,
      })
      if (snapshot.summary.failed > 0) {
        await Promise.resolve()
        box.checkinResultRef?.scrollIntoView({ behavior: 'smooth', block: 'start' })
      }
      const currentResult =
        box.checkinResult ??
        toDisplayResponse({
          results: snapshot.results,
          summary: snapshot.summary,
        })
      box.checkinResult = await runWafRecovery(currentResult)
      notify()
    } finally {
      box.checkinFlowPhase = 'finished'
      box.checkinLoading = false
      notify()
    }
  }

  const startAndTrackCheckinJob = async (accountIds: string[]) => {
    if (accountIds.length === 0) return
    box.checkinLoading = true
    box.checkinResult = null
    box.wafRecoveryRunning = false
    box.wafRecoveryProviderName = null
    box.wafRecoveryMessage = null
    box.checkinFlowPhase = 'running'
    box.showProgressModal = true
    box.checkinProgress = { total: accountIds.length, completed: 0, currentAccountName: '' }
    box.checkinLogs = []
    notify()
    await cleanupCheckinJobListeners()

    try {
      const response = await startCheckinJob<StartCheckinJobResponse>(accountIds)
      box.activeCheckinJobId = response.job_id
      applyCheckinJobSnapshot(response.snapshot)

      trackListen<CheckinJobDelta>('checkin:job-delta', applyCheckinJobDelta, bag)
      trackListen<CheckinJobSnapshot>('checkin:job-finished', (snapshot) => {
        void finalizeCheckinJob(snapshot)
      }, bag)
      trackListen<CheckinJobSnapshot>('checkin:job-timeout', (snapshot) => {
        void finalizeCheckinJob(snapshot)
      }, bag)

      const latestSnapshot = await getCheckinJobStatus<CheckinJobSnapshot>(response.job_id)
      if (latestSnapshot.job_id !== response.job_id) return
      if (isTerminalJobSnapshot(latestSnapshot)) {
        await finalizeCheckinJob(latestSnapshot)
        return
      }
      applyCheckinJobSnapshot(latestSnapshot)
    } catch (error: unknown) {
      box.checkinLoading = false
      box.showProgressModal = false
      box.activeCheckinJobId = null
      box.checkinFlowPhase = 'finished'
      await cleanupCheckinJobListeners()
      notify()
      notifyJobStartFailed(error)
      logger.error('Checkin job failed', error)
    }
  }

  const executeCheckinAll = async () => {
    const enabledAccountIds = box.accounts.filter((account) => account.enabled).map((account) => account.id)
    await startAndTrackCheckinJob(enabledAccountIds)
  }

  const executeCheckinSingle = async (accountId: string) => {
    await startAndTrackCheckinJob([accountId])
  }

  return {
    cleanupCheckinJobListeners,
    executeCheckinAll,
    executeCheckinSingle,
  }
}
