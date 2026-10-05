import { getCheckinJobStatus } from '@/api'
import type { CheckinJobSnapshot } from '@/types/checkin'
import { createListenBag, disposeListens, trackListen } from './listenCancel'
import { isTerminalJobSnapshot } from './wafFormat'

/**
 * 等待补救重试任务结束：复用 checkin:job-finished / checkin:job-timeout 事件，
 * 监听挂载后再用 getCheckinJobStatus 对账一次，覆盖事件先于监听到达的窗口（无轮询）。
 */
export const waitForCheckinJobResult = async (
  jobId: string,
  initialSnapshot: CheckinJobSnapshot,
): Promise<CheckinJobSnapshot> => {
  if (isTerminalJobSnapshot(initialSnapshot)) {
    return initialSnapshot
  }

  return new Promise<CheckinJobSnapshot>((resolve, reject) => {
    const bag = createListenBag()
    let settled = false

    const finish = (snapshot: CheckinJobSnapshot | null, error?: unknown) => {
      if (settled) return
      if (error) {
        settled = true
        disposeListens(bag)
        reject(error instanceof Error ? error : new Error(String(error)))
        return
      }
      if (!snapshot || snapshot.job_id !== jobId || !isTerminalJobSnapshot(snapshot)) return
      settled = true
      disposeListens(bag)
      resolve(snapshot)
    }

    const onPayload = (payload: CheckinJobSnapshot) => finish(payload)

    trackListen<CheckinJobSnapshot>('checkin:job-finished', onPayload, bag)
    trackListen<CheckinJobSnapshot>('checkin:job-timeout', onPayload, bag)

    void getCheckinJobStatus<CheckinJobSnapshot>(jobId)
      .then((latest) => finish(latest))
      .catch((error: unknown) => finish(null, error))
  })
}
