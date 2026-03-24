import { nextTick, type Ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCheckinJobStatus, startCheckinJob } from '@/api'
import { logger } from '@/utils/logger'
import type {
  AccountInfo,
  CheckinDisplayResponse,
  CheckinFlowPhase,
  CheckinJobSnapshot,
  CheckinLogEntry,
  CheckinResponse,
  StartCheckinJobResponse,
} from '@/types/checkin'
import { mapCheckinJobLogEntry } from './checkinWafRecovery'
import type { CheckinRefreshOptions } from './checkinDataState'

type RefreshCheckinData = (options?: CheckinRefreshOptions) => Promise<void>
type RunWafRecovery = (result: CheckinDisplayResponse) => Promise<CheckinDisplayResponse>

interface CheckinJobRefs {
  accounts: Ref<AccountInfo[]>
  checkinLoading: Ref<boolean>
  checkinResult: Ref<CheckinDisplayResponse | null>
  checkinResultRef: Ref<HTMLElement | null>
  showProgressModal: Ref<boolean>
  checkinFlowPhase: Ref<CheckinFlowPhase>
  checkinProgress: Ref<{ total: number; completed: number; currentAccountName: string }>
  checkinLogs: Ref<CheckinLogEntry[]>
  wafRecoveryRunning: Ref<boolean>
  wafRecoveryProviderName: Ref<string | null>
  wafRecoveryMessage: Ref<string | null>
  activeCheckinJobId: Ref<string | null>
}

const toDisplayResponse = (response: CheckinResponse): CheckinDisplayResponse => ({
  results: response.results.map((item) => ({ ...item })),
  summary: { ...response.summary },
})

export const createCheckinJobRuntime = (
  refs: CheckinJobRefs,
  refreshCheckinData: RefreshCheckinData,
  runWafRecovery: RunWafRecovery,
  getErrorMessage: (error: unknown, fallback: string) => string
) => {
  const checkinJobUnlisteners: UnlistenFn[] = []

  const cleanupCheckinJobListeners = async () => {
    const unlisteners = checkinJobUnlisteners.splice(0)
    await Promise.all(unlisteners.map((unlisten) => unlisten()))
  }

  const applyCheckinJobSnapshot = (snapshot: CheckinJobSnapshot) => {
    refs.checkinProgress.value = {
      total: snapshot.total,
      completed: snapshot.completed,
      currentAccountName: snapshot.current_account_name,
    }
    refs.checkinLogs.value = snapshot.logs.map(mapCheckinJobLogEntry)

    if (snapshot.status === 'finished' || snapshot.status === 'timed_out') {
      refs.checkinResult.value = toDisplayResponse({
        results: snapshot.results,
        summary: snapshot.summary,
      })
    }
  }

  const finalizeCheckinJob = async (snapshot: CheckinJobSnapshot) => {
    if (refs.activeCheckinJobId.value !== snapshot.job_id) return

    try {
      applyCheckinJobSnapshot(snapshot)
      refs.activeCheckinJobId.value = null
      await cleanupCheckinJobListeners()

      await refreshCheckinData({
        reloadAccounts: true,
        reloadRecords: true,
        reloadStats: true,
      })

      if (snapshot.summary.failed > 0) {
        await nextTick()
        refs.checkinResultRef.value?.scrollIntoView({ behavior: 'smooth', block: 'start' })
      }

      const currentResult =
        refs.checkinResult.value ??
        toDisplayResponse({
          results: snapshot.results,
          summary: snapshot.summary,
        })
      refs.checkinResult.value = await runWafRecovery(currentResult)
    } finally {
      refs.checkinFlowPhase.value = 'finished'
      refs.checkinLoading.value = false
    }
  }

  const startAndTrackCheckinJob = async (accountIds: string[]) => {
    if (accountIds.length === 0) return

    refs.checkinLoading.value = true
    refs.checkinResult.value = null
    refs.wafRecoveryRunning.value = false
    refs.wafRecoveryProviderName.value = null
    refs.wafRecoveryMessage.value = null
    refs.checkinFlowPhase.value = 'running'
    refs.showProgressModal.value = true
    refs.checkinProgress.value = {
      total: accountIds.length,
      completed: 0,
      currentAccountName: '',
    }
    refs.checkinLogs.value = []

    await cleanupCheckinJobListeners()

    try {
      const response = await startCheckinJob<StartCheckinJobResponse>(accountIds)
      refs.activeCheckinJobId.value = response.job_id
      applyCheckinJobSnapshot(response.snapshot)

      const handleProgressSnapshot = (snapshot: CheckinJobSnapshot) => {
        if (refs.activeCheckinJobId.value !== snapshot.job_id) return
        applyCheckinJobSnapshot(snapshot)
      }

      const handleTerminalSnapshot = (snapshot: CheckinJobSnapshot) => {
        void finalizeCheckinJob(snapshot)
      }

      checkinJobUnlisteners.push(
        await listen<CheckinJobSnapshot>('checkin:job-progress', (event) => {
          handleProgressSnapshot(event.payload)
        })
      )
      checkinJobUnlisteners.push(
        await listen<CheckinJobSnapshot>('checkin:job-finished', (event) => {
          handleTerminalSnapshot(event.payload)
        })
      )
      checkinJobUnlisteners.push(
        await listen<CheckinJobSnapshot>('checkin:job-timeout', (event) => {
          handleTerminalSnapshot(event.payload)
        })
      )

      const latestSnapshot = await getCheckinJobStatus<CheckinJobSnapshot>(response.job_id)
      if (latestSnapshot.job_id === response.job_id) {
        if (latestSnapshot.status === 'finished' || latestSnapshot.status === 'timed_out') {
          await finalizeCheckinJob(latestSnapshot)
        } else {
          applyCheckinJobSnapshot(latestSnapshot)
        }
      }
    } catch (error: unknown) {
      refs.checkinLoading.value = false
      refs.showProgressModal.value = false
      refs.activeCheckinJobId.value = null
      refs.checkinFlowPhase.value = 'finished'
      await cleanupCheckinJobListeners()
      alert('签到失败: ' + getErrorMessage(error, '未知错误'))
      logger.error('Checkin job failed', error)
    }
  }

  const executeCheckinAll = async () => {
    const enabledAccountIds = refs.accounts.value
      .filter((account) => account.enabled)
      .map((account) => account.id)
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
