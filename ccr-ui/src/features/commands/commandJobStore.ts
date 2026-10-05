import { create } from 'zustand'
import { cancelCcrCommandJob, getCcrCommandJobStatus, startCcrCommandJob } from '@/api'
import { addRecentItem, clearRecentItems, getRecentItems } from '@/api/domains/uiState'
import type { CommandJobDelta, CommandJobSnapshot } from '@/types'
import type { CommandHistoryDto } from '@/types/generated/ui_state/CommandHistoryDto'
import { logger } from '@/utils/logger'
import {
  COMMAND_EVENT_BUFFER_CAP, COMMAND_OUTPUT_LINES_PER_CHANNEL, COMMAND_RECONCILE_WINDOW_MS, commandJobMissing, emptyCommandJob, isCommandActive,
  isCommandTerminal, mergeCommandDelta, mergeCommandSnapshot, type CommandJobState,
} from './commandJobState'

interface StartCommand {
  command: string
  args: string[]
  confirmationToken?: string
}

interface HistoryWrite {
  summary: Pick<CommandJobSnapshot, 'job_id' | 'command' | 'args' | 'status' | 'duration_ms'>
  status: 'pending' | 'saved' | 'failed'
  error: string | null
}

interface CommandsStreamState {
  job: CommandJobState
  submitting: boolean
  cancelling: boolean
  error: string | null
  historyItems: CommandHistoryDto[]
  historyWrites: Record<string, HistoryWrite>
  listenerFailures: Record<string, string>
  reportListener: (event: string, error: string | null) => void
  start: (request: StartCommand) => Promise<void>
  cancel: () => Promise<void>
  reconcile: () => Promise<void>
  receiveDeltas: (deltas: CommandJobDelta[]) => void
  receiveSnapshot: (snapshot: CommandJobSnapshot) => void
  clearOutput: () => void
  loadHistory: () => Promise<void>
  clearHistory: () => Promise<void>
  resume: () => void
  suspend: () => void
}

interface EarlyEvents {
  deltas: CommandJobDelta[]
  terminal?: CommandJobSnapshot
}

const errorMessage = (error: unknown) => error instanceof Error ? error.message : String(error)

/** One in-memory owner for the shell lifetime. Nothing here is persisted to
 * localStorage; only the existing history API stores its existing summary. */
export const useCommandsStreamStore = create<CommandsStreamState>()((set, get) => {
  let generation = 0
  let snapshotRequest = 0
  let historyRequest = 0
  let refresh: Promise<void> | null = null
  let refreshAgain = false
  let timer: ReturnType<typeof setTimeout> | null = null
  let recoveryActive = false
  const early = new Map<string, EarlyEvents>()

  const stopTimer = () => {
    if (timer !== null) clearTimeout(timer)
    timer = null
  }

  const loadHistory = async () => {
    const version = ++historyRequest
    try {
      const historyItems = await getRecentItems(20)
      if (version === historyRequest) set({ historyItems })
    } catch (error) {
      logger.error('Failed to load command history:', error)
    }
  }

  const recordTerminal = (snapshot: CommandJobSnapshot) => {
    if (!isCommandTerminal(snapshot) || get().historyWrites[snapshot.job_id]) return
    const { job_id, command, args, status, duration_ms } = snapshot
    const summary = { job_id, command, args, status, duration_ms }
    const updateWrite = (status: HistoryWrite['status'], error: string | null = null) => {
      set((state) => ({ historyWrites: { ...state.historyWrites, [job_id]: { summary, status, error } } }))
    }
    // Pending is a submission claim, never proof of persistence. The existing
    // API has no job idempotency key; an ambiguous failure must not be replayed.
    updateWrite('pending')
    void addRecentItem(snapshot.command, snapshot.args, snapshot.status === 'success', snapshot.duration_ms ?? 0)
      .then(() => { updateWrite('saved'); return loadHistory() })
      .catch((error: unknown) => { updateWrite('failed', errorMessage(error)) })
  }

  const acceptSnapshot = (snapshot: CommandJobSnapshot) => {
    const job = mergeCommandSnapshot(get().job, snapshot)
    if (job === get().job) return
    set({ job, error: null })
    if (isCommandTerminal(snapshot)) {
      stopTimer()
      recordTerminal(snapshot)
    }
  }

  const scheduleRefresh = (delay = COMMAND_RECONCILE_WINDOW_MS) => {
    if (!recoveryActive || timer !== null || get().job.expired || !isCommandActive(get().job.snapshot)) return
    // Fixed window from the first event. Continuous output cannot defer recovery.
    timer = setTimeout(() => { timer = null; void get().reconcile() }, delay)
  }

  const rememberEarly = (jobId: string): EarlyEvents => {
    const events = early.get(jobId) ?? { deltas: [] }
    early.set(jobId, events)
    if (early.size > 16) early.delete(early.keys().next().value as string)
    return events
  }

  const receiveDeltas = (deltas: CommandJobDelta[]) => {
    let job = get().job
    for (const delta of deltas) {
      if (get().submitting && delta.job_id !== job.snapshot?.job_id) {
        const events = rememberEarly(delta.job_id)
        const lines = delta.lines.slice(-COMMAND_OUTPUT_LINES_PER_CHANNEL)
        events.deltas = [...events.deltas, { ...delta, lines }].slice(-COMMAND_EVENT_BUFFER_CAP)
        continue
      }
      job = mergeCommandDelta(job, delta)
    }
    if (job === get().job) return
    set({ job })
    if (job.authoritative) scheduleRefresh()
  }

  const receiveSnapshot = (snapshot: CommandJobSnapshot) => {
    if (get().submitting && snapshot.job_id !== get().job.snapshot?.job_id) {
      if (isCommandTerminal(snapshot)) rememberEarly(snapshot.job_id).terminal ??= snapshot
      return
    }
    if (snapshot.job_id !== get().job.snapshot?.job_id) return
    acceptSnapshot(snapshot)
  }

  const reconcile = async () => {
    const current = get().job.snapshot
    if (!recoveryActive || !current || get().job.expired || isCommandTerminal(current)) return
    if (refresh) { refreshAgain = true; return refresh }
    const epoch = generation
    const request = ++snapshotRequest
    let succeeded = false
    set({ job: { ...get().job, authoritative: true } })
    const matches = () => epoch === generation && request === snapshotRequest && current.job_id === get().job.snapshot?.job_id
    const pending = getCcrCommandJobStatus(current.job_id).then((snapshot) => {
      if (matches()) { acceptSnapshot(snapshot); succeeded = true }
    }).catch((error: unknown) => {
      if (!matches() || isCommandTerminal(get().job.snapshot ?? current)) return
      stopTimer()
      const expired = commandJobMissing(error, current.job_id)
      set({ job: { ...get().job, expired }, error: expired ? null : errorMessage(error) })
      // Transient failures preserve the last snapshot and require a new event or
      // explicit refresh. Missing retained jobs are not fabricated completions.
      refreshAgain = false
    }).finally(() => {
      if (refresh !== pending) return
      refresh = null
      if (refreshAgain) scheduleRefresh()
      else if (succeeded && Object.keys(get().listenerFailures).length > 0) scheduleRefresh(1000)
      refreshAgain = false
    })
    refresh = pending
    return pending
  }

  const start = async (request: StartCommand) => {
    if (get().submitting || (isCommandActive(get().job.snapshot) && !get().job.expired)) return
    const epoch = ++generation
    stopTimer()
    early.clear()
    refresh = null
    refreshAgain = false
    set({ submitting: true, cancelling: false, error: null, job: emptyCommandJob() })
    try {
      const response = await startCcrCommandJob(request)
      if (epoch !== generation) return
      const buffered = early.get(response.job_id)
      // Only the start response establishes the new job identity. Unrelated or
      // old job events cannot select the active job while its ID is unknown.
      const initial = buffered?.terminal ?? response.snapshot
      const authoritative = initial.status !== 'queued'
      set({ job: mergeCommandSnapshot(emptyCommandJob(), initial, authoritative) })
      receiveDeltas(buffered?.deltas ?? [])
      recordTerminal(initial)
      if (Object.keys(get().listenerFailures).length > 0) scheduleRefresh()
    } catch (error) {
      if (epoch === generation) set({ error: errorMessage(error) })
    } finally {
      if (epoch === generation) { set({ submitting: false }); early.clear() }
    }
  }

  const cancel = async () => {
    const snapshot = get().job.snapshot
    if (!snapshot || !isCommandActive(snapshot) || get().job.expired || get().cancelling) return
    const epoch = generation
    // Cancel responses can contain output with no seq watermark too.
    const request = ++snapshotRequest
    set({ cancelling: true, job: { ...get().job, authoritative: true } })
    try {
      const response = await cancelCcrCommandJob(snapshot.job_id)
      if (epoch === generation && (request === snapshotRequest || isCommandTerminal(response))) acceptSnapshot(response)
    } catch (error) {
      if (epoch === generation && isCommandActive(get().job.snapshot)) {
        const expired = commandJobMissing(error, snapshot.job_id)
        set({ job: { ...get().job, expired }, error: expired ? null : errorMessage(error) })
      }
    } finally {
      if (epoch === generation) { set({ cancelling: false }); scheduleRefresh() }
    }
  }

  return {
    job: emptyCommandJob(), submitting: false, cancelling: false, error: null, historyItems: [],
    historyWrites: {}, listenerFailures: {},
    reportListener: (event, error) => {
      const listenerFailures = { ...get().listenerFailures }
      if (error === null) delete listenerFailures[event]
      else listenerFailures[event] = error
      set({ listenerFailures })
      if (error !== null) scheduleRefresh()
    },
    start, cancel, reconcile, receiveDeltas, receiveSnapshot, loadHistory,
    clearOutput: () => {
      if (get().submitting || (isCommandActive(get().job.snapshot) && !get().job.expired)) return
      ++generation
      stopTimer()
      set({ job: emptyCommandJob(), error: null })
    },
    clearHistory: async () => {
      ++historyRequest
      try { await clearRecentItems(); await loadHistory() }
      catch (error) { logger.error('Failed to clear recent history:', error) }
    },
    resume: () => { recoveryActive = true },
    suspend: () => { recoveryActive = false; stopTimer(); ++snapshotRequest; refresh = null; refreshAgain = false },
  }
})
