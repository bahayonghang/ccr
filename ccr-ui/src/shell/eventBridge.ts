import { useQueryClient } from '@tanstack/react-query'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useEffect } from 'react'
import { claudeObserverKeys } from '@/features/claude/queries'
import { useCommandsStreamStore } from '@/features/commands/stores'
import { COMMAND_EVENT_BUFFER_CAP } from '@/features/commands/commandJobState'
import { homeUsageKeys, usageKeys } from '@/features/usage/queries'
import type { CommandJobDelta, CommandJobSnapshot } from '@/types/config'
import { logger } from '@/utils/logger'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import { currentEnvironmentKey } from '@/configs/environmentSession'

// Tauri Event → Query 桥接层（08-22-state-logic-port 批次 3，design.md §3）。
//
// 后端 emit 的全局事件在这里集中转成 queryClient 失效/写入。命令任务由
// shell-lived command stream store 持有唯一进程内快照。其他逐事件判定见
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
  maxItems = Infinity,
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
    if (buffer.length > maxItems) buffer = buffer.slice(-maxItems)
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
    const track = (pending: Promise<UnlistenFn>, onResult?: (error: string | null) => void) => {
      pending.then((unlisten) => {
        if (disposed) unlisten()
        else { unlistens.push(unlisten); onResult?.(null) }
      }).catch((error) => {
        if (!disposed) onResult?.(String(error))
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
    const refreshEnvironment = () => {
      // Invalidate identity first so editors freeze before environment-bound reads.
      const detected = queryClient.invalidateQueries({ queryKey: currentEnvironmentKey })
      void queryClient.cancelQueries({ predicate: (query) => [
        'platform-settings', 'platform-settings-probe', 'settings-source-environment',
      ].includes(String(query.queryKey[0])) })
      void detected.then(() => {
        if (!disposed) void queryClient.invalidateQueries({ predicate: (query) => query.queryKey[0] !== currentEnvironmentKey[0] })
      })
    }
    track(listenSafe('env:refresh-requested', refreshEnvironment))
    track(listenSafe('env:changed', refreshEnvironment))

    // Shell owns one listener set. Route pages only select the retained job.
    useCommandsStreamStore.getState().resume()
    const commandBatch = createEventBatcher<CommandJobDelta>((batch) => {
      if (!disposed) useCommandsStreamStore.getState().receiveDeltas(batch)
    }, HIGH_FREQUENCY_FLUSH_INTERVAL_MS, COMMAND_EVENT_BUFFER_CAP)
    const trackCommand = (event: string, handler: EventListener) => {
      track(listenSafe(event, handler), (error) => useCommandsStreamStore.getState().reportListener(event, error))
    }
    trackCommand('commands:job-progress', (payload) => {
      if (!disposed) commandBatch.push(payload as CommandJobDelta)
    })
    const finishCommand = (payload: unknown) => {
      if (disposed) return
      commandBatch.commit()
      useCommandsStreamStore.getState().receiveSnapshot(payload as CommandJobSnapshot)
    }
    trackCommand('commands:job-finished', finishCommand)
    trackCommand('commands:job-cancelled', finishCommand)
    if (isTauriRuntime()) void useCommandsStreamStore.getState().reconcile()

    return () => {
      commandBatch.dispose()
      disposed = true
      useCommandsStreamStore.getState().suspend()
      unlistens.forEach((unlisten) => unlisten())
      unlistens.length = 0
    }
  }, [queryClient])
}
