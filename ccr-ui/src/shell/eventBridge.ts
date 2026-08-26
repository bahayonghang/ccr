import { useQueryClient } from '@tanstack/react-query'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useEffect } from 'react'
import { claudeObserverKeys } from '@/features/claude/queries'
import {
  useCommandsStreamStore,
  type CommandStreamChannel,
  type CommandStreamLine,
} from '@/features/commands/stores'
import { homeUsageKeys, usageKeys } from '@/features/usage/queries'
import type { CommandJobDelta } from '@/types/config'
import { logger } from '@/utils/logger'
import { isTauriRuntime } from '@/utils/tauriRuntime'

// Tauri Event → Query 桥接层（08-22-state-logic-port 批次 3，design.md §3）。
//
// 后端 emit 的全局事件在这里集中转成 queryClient 失效/写入，store 不再直接
// 持有服务端数据。逐事件判定（setQueryData vs invalidateQueries）见
// `event-adjudication.md`；事件名清单（全局部分，协同点 M）亦在该文件。
//
// 取消协议：`listen()` 返回 Promise<UnlistenFn>，cleanup 可能先于 resolve 执行
// （StrictMode 挂载→卸载→再挂载、快速路由切换）。cleanup 已跑过时迟到的
// unlisten 立即调用，不入数组——否则监听器永久泄漏。泄漏断言见批次 6 的
// `tests/event-bridge-leak`（三用例，含延迟 resolve）。
//
// 接线：挂在应用外壳（08-22-shell-port）；本文件只提供桥接组件与原语。

/** 高频事件的批量提交间隔。保守值 250ms；待 arch-quality-perf 场景 3 的
 * React 侧基线数据复核（该数据由 08-22-regression-release 步骤 7 补测），
 * 复核记录见 event-adjudication.md §3。 */
export const HIGH_FREQUENCY_FLUSH_INTERVAL_MS = 250

/** 桥接层管理的全局事件名（集中可见；局部事件见 event-adjudication.md §4）。 */
export const TAURI_GLOBAL_EVENTS = [
  'usage:snapshot-updated',
  'usage:job-progress',
  'usage:job-finished',
  'usage:job-failed',
  'usage:job-recent-ready',
  'usage:session-index-progress',
  'usage:session-index-finished',
  'usage:session-index-failed',
  'usage:import-completed',
  'claude_observer:updated',
  'env:refresh-requested',
  'env:changed',
  'commands:job-progress',
  'commands:job-finished',
  'commands:job-cancelled',
] as const

export type TauriGlobalEvent = (typeof TAURI_GLOBAL_EVENTS)[number]

type EventListener = (payload: unknown) => void

const toStreamLines = (delta: CommandJobDelta): CommandStreamLine[] =>
  delta.lines.map((text) => ({
    channel: delta.channel as CommandStreamChannel,
    text,
    seq: delta.seq,
    jobId: delta.job_id,
  }))

const appendCommandDelta = (payload: unknown): void => {
  const delta = payload as CommandJobDelta
  if (!Array.isArray(delta.lines)) return
  useCommandsStreamStore.getState().appendStreamLines({ lines: toStreamLines(delta) })
}

const appendCommandFinished = (payload: unknown): void => {
  const snapshot = payload as { job_id?: string }
  if (typeof snapshot.job_id !== 'string') return
  useCommandsStreamStore.getState().appendStreamLines({
    lines: [
      {
        channel: 'system',
        text: `job ${snapshot.job_id} finished`,
        seq: Number.MAX_SAFE_INTEGER,
        jobId: snapshot.job_id,
      },
    ],
  })
}

/** createEventBatcher 的返回契约（消费方按名引用，避免 ReturnType 耦合）。 */
export interface EventBatcher<T> {
  push: (item: T) => void
  dispose: () => void
  commit: () => void
}

/**
 * 高频事件缓冲：ref 累积 + 定时批量提交，避免逐条 setQueryData 逐条重渲染。
 * flush 的提交动作由调用方给出（setQueryData 拼接 / 追加语义归消费方）。
 */
export function createEventBatcher<T>(
  flush: (batch: T[]) => void,
  intervalMs = HIGH_FREQUENCY_FLUSH_INTERVAL_MS,
): EventBatcher<T> {
  let buffer: T[] = []
  let timer: ReturnType<typeof setInterval> | null = null

  const commit = () => {
    if (buffer.length === 0) return
    const batch = buffer
    buffer = []
    flush(batch)
  }

  const push = (item: T) => {
    buffer.push(item)
    if (timer === null) {
      timer = setInterval(() => {
        commit()
        if (timer !== null && buffer.length === 0) {
          clearInterval(timer)
          timer = null
        }
      }, intervalMs)
    }
  }

  const dispose = () => {
    commit()
    if (timer !== null) {
      clearInterval(timer)
      timer = null
    }
  }

  return { push, dispose, commit }
}

/** 非浏览器/Tauri 环境下 listen 建立的桩（保持 track 协议形状一致）。 */
const listenSafe = (event: string, handler: EventListener) => {
  if (!isTauriRuntime()) {
    const noop: UnlistenFn = () => {}
    return Promise.resolve(noop)
  }
  return listen(event, (e) => handler(e.payload))
}

/**
 * 全局事件桥。挂在应用外壳一次；重复挂载安全（各自独立订阅与解绑）。
 */
export function useTauriEventBridge() {
  const queryClient = useQueryClient()

  useEffect(() => {
    let disposed = false
    const unlistens: UnlistenFn[] = []

    // 取消协议：cleanup 已跑过时，迟到的 unlisten 立即调用，不入数组。
    const track = (pending: Promise<UnlistenFn>) => {
      pending.then((unlisten) => {
        if (disposed) unlisten()
        else unlistens.push(unlisten)
      }).catch((error) => {
        logger.warn('[eventBridge] listen failed', { event: String(error) })
      })
    }

    const invalidate = (key: readonly unknown[]) => () => {
      void queryClient.invalidateQueries({ queryKey: key })
    }

    // —— 用量：数据切片整体失效（payload 为通知，非完整数据）——
    track(listenSafe('usage:snapshot-updated', invalidate(usageKeys.all)))
    track(listenSafe('usage:snapshot-updated', invalidate(homeUsageKeys.all)))
    track(listenSafe('usage:job-progress', invalidate(usageKeys.all)))
    track(listenSafe('usage:job-finished', invalidate(usageKeys.all)))
    track(listenSafe('usage:job-failed', invalidate(usageKeys.all)))
    track(listenSafe('usage:job-recent-ready', invalidate(usageKeys.all)))
    track(listenSafe('usage:import-completed', invalidate(usageKeys.all)))
    track(listenSafe('usage:session-index-progress', invalidate(homeUsageKeys.all)))
    track(listenSafe('usage:session-index-finished', invalidate(homeUsageKeys.all)))
    track(listenSafe('usage:session-index-failed', invalidate(homeUsageKeys.all)))

    // —— Claude 观测：单事件驱动全切片 refetch（原 store 语义）——
    track(listenSafe('claude_observer:updated', invalidate(claudeObserverKeys.all)))

    // —— 环境：全量失效（环境变更影响多数数据域）——
    track(listenSafe('env:refresh-requested', invalidate([])))
    track(listenSafe('env:changed', invalidate([])))

    // —— 命令流：按 client 累积缓冲，视图卸载不清空（外壳门 AC4）——
    track(listenSafe('commands:job-progress', appendCommandDelta))
    track(listenSafe('commands:job-finished', appendCommandFinished))
    track(listenSafe('commands:job-cancelled', appendCommandFinished))

    return () => {
      disposed = true
      unlistens.forEach((unlisten) => unlisten())
      unlistens.length = 0
    }
  }, [queryClient])
}
