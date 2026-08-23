// 余额刷新并发治理：同 key（origin）串行、异 key 并行，全局并发上限对齐后端签到 Semaphore
export const BALANCE_REFRESH_CONCURRENCY = 5

// minInterval 节流窗口：距上次余额刷新 < 30s 的账号跳过（单账号手动刷新走 force 路径不受限）
export const BALANCE_REFRESH_MIN_INTERVAL_MS = 30_000

export interface PerKeyTask<T> {
  key: string
  run: () => Promise<T>
}

/**
 * 按 key 分组执行任务：同 key 内严格串行（保持入参顺序），不同 key 之间并行，
 * 同时在运行的 key 组数不超过 concurrency。结果按入参顺序返回（allSettled 语义）。
 */
export const runPerKeySequential = async <T>(
  tasks: PerKeyTask<T>[],
  concurrency = BALANCE_REFRESH_CONCURRENCY,
): Promise<PromiseSettledResult<T>[]> => {
  const results: PromiseSettledResult<T>[] = new Array<PromiseSettledResult<T>>(tasks.length)

  const queues = new Map<string, Array<{ index: number; run: () => Promise<T> }>>()
  for (const [index, task] of tasks.entries()) {
    const queue = queues.get(task.key) ?? []
    queue.push({ index, run: task.run })
    queues.set(task.key, queue)
  }

  const pendingQueues = Array.from(queues.values())
  let cursor = 0

  const runItem = async (item: { index: number; run: () => Promise<T> }) => {
    try {
      results[item.index] = { status: 'fulfilled', value: await item.run() }
    } catch (reason) {
      results[item.index] = { status: 'rejected', reason }
    }
  }

  const worker = async () => {
    while (cursor < pendingQueues.length) {
      const queue = pendingQueues[cursor]
      cursor += 1
      for (const item of queue) {
        await runItem(item)
      }
    }
  }

  const workerCount = Math.max(1, Math.min(concurrency, pendingQueues.length))
  await Promise.all(Array.from({ length: workerCount }, () => worker()))

  return results
}

/** 距上次余额检查不足 minInterval 时跳过本次批量刷新（时间无效或缺失则不跳过） */
export const shouldSkipBalanceRefresh = (
  lastBalanceCheckAt: string | undefined,
  now: number = Date.now(),
  minIntervalMs: number = BALANCE_REFRESH_MIN_INTERVAL_MS,
): boolean => {
  if (!lastBalanceCheckAt) return false
  const checkedAt = new Date(lastBalanceCheckAt).getTime()
  if (Number.isNaN(checkedAt)) return false
  return now - checkedAt < minIntervalMs
}
