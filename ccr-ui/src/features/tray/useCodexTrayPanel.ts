import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { getCodexTraySnapshot, switchCodexAuth } from '@/api'
import {
  shellBeginTrayPanelDrag,
  shellCompleteTrayPanelDrag,
  shellRequestQuit,
  shellShowMainWindow,
} from '@/api/runtime/environment'
import type { CodexTraySnapshot } from '@/types'
import { getErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'
import { getCurrentWindowSafe } from '@/utils/tauriWindow'
import { shouldPersistTrayPanelManualPosition, traySnapshotKey } from './tray-format'

export function useCodexTrayPanel() {
  const queryClient = useQueryClient()
  const forceRef = useRef(false)

  const snapshotQuery = useQuery({
    queryKey: traySnapshotKey,
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
  const canManageAccounts = snapshot?.can_manage_accounts ?? false
  const loading = snapshotQuery.isFetching
  const error = switchError ?? (snapshotQuery.error ? getErrorMessage(snapshotQuery.error) : null)
  const { refetch: refetchSnapshot } = snapshotQuery

  const loadSnapshot = useCallback(async (force = false) => {
    forceRef.current = force
    await refetchSnapshot()
  }, [refetchSnapshot])

  const switchAccount = useCallback(async (name: string) => {
    setBusyAccount(name)
    setSwitchError(null)
    try {
      await switchCodexAuth(name)
      forceRef.current = true
      await refetchSnapshot()
      setScreen('overview')
    } catch (caught) {
      logger.error('Failed to switch Codex tray account:', caught)
      setSwitchError(getErrorMessage(caught))
    } finally {
      setBusyAccount(null)
    }
  }, [refetchSnapshot])

  const openMain = useCallback(async () => {
    await shellShowMainWindow()
  }, [])
  const openUsage = useCallback(async () => {
    await shellShowMainWindow('/usage')
  }, [])
  const openAuth = useCallback(async () => {
    await shellShowMainWindow('/codex/auth')
  }, [])
  const quit = useCallback(async () => {
    await shellRequestQuit()
  }, [])

  const startPanelDrag = useCallback(async () => {
    const win = await getCurrentWindowSafe()
    if (!win || win.label !== 'codex-tray-panel') return
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
  }, [])

  const goToSwitchScreen = useCallback(() => {
    if (!canManageAccounts) return
    setScreen('switch')
  }, [canManageAccounts])
  const goToOverview = useCallback(() => {
    setScreen('overview')
  }, [])

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
      if (!win || disposed) return
      track(
        win.listen<CodexTraySnapshot>('codex-tray:refresh', (event) => {
          queryClient.setQueryData<CodexTraySnapshot>(traySnapshotKey, event.payload)
          setSwitchError(null)
        }),
      )
    })()

    return () => {
      disposed = true
      unlistens.forEach((unlisten) => unlisten())
    }
  }, [queryClient])

  return {
    accounts,
    busyAccount,
    canManageAccounts,
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
