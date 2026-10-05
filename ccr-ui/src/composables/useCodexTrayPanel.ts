import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { codexKeys } from '@/features/codex/queries'
import { getErrorMessage } from '@/utils/errorHandler'
import { getCodexTraySnapshot, switchCodexAuth } from '@/api'
import {
  shellBeginTrayPanelDrag,
  shellCompleteTrayPanelDrag,
  shellRequestQuit,
  shellShowMainWindow,
} from '@/api/runtime/environment'
import type { CodexTraySnapshot } from '@/types'
import { logger } from '@/utils/logger'
import { getCurrentWindowSafe } from '@/utils/tauriWindow'

// Codex 托盘面板的 React 迁移（08-22-state-logic-port 批次 5b-ii）。
// - snapshot → Query（codexKeys.tray.snapshot，staleTime Infinity：仅挂载拉取 +
//   显式 loadSnapshot + 事件推送写入，无自动 refetch）；force 参数经 forceRef
//   在重跑 queryFn 时消费一次（与 useCodexDashboard 同款透传）；
// - `codex-tray:refresh` 为组件级窗口事件（独立托盘窗口，不进全局桥接层），payload
//   含完整快照 → setQueryData 直写缓存（event-adjudication.md §5 判定）。订阅走
//   取消协议：cleanup 已跑过时迟到的 unlisten 立即调用；
// - screen / isDragging / busyAccount → useState；computed → useMemo。
//
// 签名变化（消费方 CodexTrayPanelView.vue 待迁移）：Ref<T> → 普通值。

const TRAY_PANEL_MANUAL_MOVE_THRESHOLD_PX = 12

export interface TrayPanelWindowPosition {
  x: number
  y: number
}

export const shouldPersistTrayPanelManualPosition = (
  beforePosition: TrayPanelWindowPosition | null,
  afterPosition: TrayPanelWindowPosition
): boolean => {
  if (!beforePosition) {
    return true
  }

  return (
    Math.abs(afterPosition.x - beforePosition.x) >= TRAY_PANEL_MANUAL_MOVE_THRESHOLD_PX ||
    Math.abs(afterPosition.y - beforePosition.y) >= TRAY_PANEL_MANUAL_MOVE_THRESHOLD_PX
  )
}

export function useCodexTrayPanel() {
  const queryClient = useQueryClient()

  // force 透传：refetch 重跑 queryFn 时消费一次（原 loadSnapshot(force) 语义）。
  const forceRef = useRef(false)

  const snapshotQuery = useQuery({
    queryKey: codexKeys.tray.snapshot(),
    staleTime: Infinity,
    queryFn: () => {
      const force = forceRef.current
      forceRef.current = false
      return getCodexTraySnapshot(force)
    },
  })

  const [screen, setScreen] = useState<'overview' | 'switch'>('overview')
  const [busyAccount, setBusyAccount] = useState<string | null>(null)
  const [switchError, setSwitchError] = useState<string | null>(null)
  const [isDragging, setIsDragging] = useState(false)

  const snapshot = snapshotQuery.data ?? null

  const currentAccount = useMemo(() => snapshot?.current_account ?? null, [snapshot])
  const accounts = useMemo(() => snapshot?.accounts ?? [], [snapshot])
  const canManageAccounts = useMemo(() => snapshot?.can_manage_accounts ?? false, [snapshot])
  const canOpenSwitchScreen = canManageAccounts

  const loading = snapshotQuery.isFetching
  const error =
    switchError ?? (snapshotQuery.error ? getErrorMessage(snapshotQuery.error) : null)

  const { refetch: refetchSnapshot } = snapshotQuery

  const loadSnapshot = useCallback(
    async (force = false) => {
      forceRef.current = force
      await refetchSnapshot()
    },
    [refetchSnapshot]
  )

  const switchAccount = useCallback(
    async (name: string) => {
      setBusyAccount(name)
      setSwitchError(null)

      try {
        await switchCodexAuth(name)
        forceRef.current = true
        await refetchSnapshot()
        setScreen('overview')
      } catch (switchError) {
        logger.error('Failed to switch Codex tray account:', switchError)
        setSwitchError(getErrorMessage(switchError))
      } finally {
        setBusyAccount(null)
      }
    },
    [refetchSnapshot]
  )

  const openMain = async (targetRoute?: string) => {
    await shellShowMainWindow(targetRoute)
  }

  const openUsage = async () => {
    await shellShowMainWindow('/usage')
  }

  const openAuth = async () => {
    await shellShowMainWindow('/codex/auth')
  }

  const quit = async () => {
    await shellRequestQuit()
  }

  const startPanelDrag = async () => {
    const win = await getCurrentWindowSafe()
    if (!win || win.label !== 'codex-tray-panel') {
      return
    }

    let beforePosition: { x: number; y: number } | null = null
    try {
      beforePosition = await win.outerPosition()
    } catch (positionError) {
      logger.warn('Failed to read tray panel position before dragging:', positionError)
    }

    setIsDragging(true)

    try {
      await shellBeginTrayPanelDrag()
      await win.startDragging()

      const afterPosition = await win.outerPosition()
      const moved = shouldPersistTrayPanelManualPosition(beforePosition, afterPosition)

      await shellCompleteTrayPanelDrag(moved ? { x: afterPosition.x, y: afterPosition.y } : null)
    } catch (dragError) {
      logger.error('Failed to drag Codex tray panel:', dragError)
      try {
        await shellCompleteTrayPanelDrag(null)
      } catch (cleanupError) {
        logger.warn('Failed to clear tray panel drag state:', cleanupError)
      }
    } finally {
      setIsDragging(false)
    }
  }

  const goToSwitchScreen = () => {
    if (!canOpenSwitchScreen) {
      return
    }
    setScreen('switch')
  }

  const goToOverview = () => {
    setScreen('overview')
  }

  // 组件级 `codex-tray:refresh` 订阅（原 onMounted 内 win.listen + onUnmounted 解绑），
  // 带取消协议：cleanup 先于 listen resolve 时迟到的 unlisten 立即调用。
  useEffect(() => {
    let disposed = false
    const unlistens: Array<() => void> = []
    const track = (pending: Promise<() => void>) =>
      pending.then((unlisten) => {
        if (disposed) unlisten()
        else unlistens.push(unlisten)
      })

    void (async () => {
      const win = await getCurrentWindowSafe()
      if (!win || disposed) {
        return
      }

      track(
        win.listen<CodexTraySnapshot>('codex-tray:refresh', (event) => {
          queryClient.setQueryData<CodexTraySnapshot>(codexKeys.tray.snapshot(), event.payload)
          setSwitchError(null)
        })
      )
    })()

    return () => {
      disposed = true
      unlistens.forEach((unlisten) => unlisten())
      unlistens.length = 0
    }
  }, [queryClient])

  return {
    accounts,
    busyAccount,
    canManageAccounts,
    canOpenSwitchScreen,
    currentAccount,
    error,
    goToOverview,
    goToSwitchScreen,
    isDragging,
    loading,
    openAuth,
    openMain,
    openUsage,
    quit,
    screen,
    snapshot,
    loadSnapshot,
    startPanelDrag,
    switchAccount,
  }
}
