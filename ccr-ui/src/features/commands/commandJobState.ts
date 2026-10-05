import type { CommandJobDelta, CommandJobSnapshot } from '@/types'

export const COMMAND_OUTPUT_LINES_PER_CHANNEL = 500
export const COMMAND_OUTPUT_BYTES_PER_CHANNEL = 512 * 1024
export const COMMAND_RECONCILE_WINDOW_MS = 250
export const COMMAND_EVENT_BUFFER_CAP = 500

export interface CommandJobState {
  snapshot: CommandJobSnapshot | null
  lastSeq: number
  authoritative: boolean
  expired: boolean
}

export const emptyCommandJob = (): CommandJobState => ({
  snapshot: null, lastSeq: -1, authoritative: false, expired: false,
})

export const isCommandActive = (snapshot: CommandJobSnapshot | null) =>
  snapshot?.status === 'queued' || snapshot?.status === 'running'

export const isCommandTerminal = (snapshot: CommandJobSnapshot) => !isCommandActive(snapshot)

const encoder = new TextEncoder()

export const boundCommandLines = (lines: string[]) => {
  let bytes = 0
  let start = lines.length
  for (let index = lines.length - 1; index >= Math.max(0, lines.length - COMMAND_OUTPUT_LINES_PER_CHANNEL); index--) {
    bytes += encoder.encode(lines[index]).length
    if (bytes > COMMAND_OUTPUT_BYTES_PER_CHANNEL) break
    start = index
  }
  return { lines: lines.slice(start), dropped: start }
}

const boundedSnapshot = (snapshot: CommandJobSnapshot): CommandJobSnapshot => {
  const stdout = boundCommandLines(snapshot.stdout_lines)
  const stderr = boundCommandLines(snapshot.stderr_lines)
  const system = boundCommandLines(snapshot.system_lines)
  const dropped = stdout.dropped + stderr.dropped + system.dropped
  return {
    ...snapshot, stdout_lines: stdout.lines, stderr_lines: stderr.lines, system_lines: system.lines,
    truncated: snapshot.truncated || dropped > 0, dropped_lines: snapshot.dropped_lines + dropped,
  }
}

/** A snapshot has no sequence watermark. After reconciliation, deltas invalidate
 * snapshots instead of appending potentially already-covered output. */
export function mergeCommandSnapshot(
  state: CommandJobState,
  snapshot: CommandJobSnapshot,
  authoritative = true,
): CommandJobState {
  const current = state.snapshot
  if (current && current.job_id !== snapshot.job_id) return state
  if (current && isCommandTerminal(current)) return state
  const next = boundedSnapshot(snapshot)
  if (current?.status === 'running' && next.status === 'queued') return state
  return { ...state, snapshot: next, authoritative: state.authoritative || authoritative, expired: false }
}

export function mergeCommandDelta(state: CommandJobState, delta: CommandJobDelta): CommandJobState {
  const snapshot = state.snapshot
  if (!snapshot || snapshot.job_id !== delta.job_id || isCommandTerminal(snapshot)) return state
  if (!Number.isSafeInteger(delta.seq) || delta.seq < 0 || delta.seq <= state.lastSeq) return state
  const authoritative = state.authoritative || delta.seq !== state.lastSeq + 1
  if (authoritative) return { ...state, lastSeq: delta.seq, authoritative: true }
  const field = delta.channel === 'stdout' ? 'stdout_lines' : delta.channel === 'stderr' ? 'stderr_lines' : 'system_lines'
  const bounded = boundCommandLines([...snapshot[field], ...delta.lines])
  const dropped = delta.dropped_count + bounded.dropped
  // A delta is not the terminal snapshot: only the backend terminal snapshot
  // carries final output, duration, exit code and cleanup outcome.
  const status = delta.status === 'running' ? 'running' : snapshot.status
  return {
    ...state, lastSeq: delta.seq,
    snapshot: {
      ...snapshot, status, [field]: bounded.lines,
      truncated: snapshot.truncated || dropped > 0, dropped_lines: snapshot.dropped_lines + dropped,
    },
  }
}

export const commandJobMissing = (error: unknown, jobId: string): boolean => {
  const message = error instanceof Error ? error.message : String(error)
  return message === `Command job '${jobId}' not found`
}
