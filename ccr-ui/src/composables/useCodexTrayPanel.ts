import { computed, onMounted, onUnmounted, ref } from 'vue'
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
  const snapshot = ref<CodexTraySnapshot | null>(null)
  const screen = ref<'overview' | 'switch'>('overview')
  const loading = ref(false)
  const busyAccount = ref<string | null>(null)
  const error = ref<string | null>(null)
  const isDragging = ref(false)

  let stopRefreshEvent: (() => void) | null = null

  const currentAccount = computed(() => snapshot.value?.current_account ?? null)
  const accounts = computed(() => snapshot.value?.accounts ?? [])
  const canManageAccounts = computed(() => snapshot.value?.can_manage_accounts ?? false)
  const canOpenSwitchScreen = computed(() => canManageAccounts.value)

  const loadSnapshot = async (force = false) => {
    loading.value = true
    error.value = null

    try {
      snapshot.value = await getCodexTraySnapshot(force)
    } catch (loadError) {
      logger.error('Failed to load Codex tray snapshot:', loadError)
      error.value = getErrorMessage(loadError)
    } finally {
      loading.value = false
    }
  }

  const switchAccount = async (name: string) => {
    busyAccount.value = name
    error.value = null

    try {
      await switchCodexAuth(name)
      await loadSnapshot(true)
      screen.value = 'overview'
    } catch (switchError) {
      logger.error('Failed to switch Codex tray account:', switchError)
      error.value = getErrorMessage(switchError)
    } finally {
      busyAccount.value = null
    }
  }

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

    isDragging.value = true

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
      isDragging.value = false
    }
  }

  const goToSwitchScreen = () => {
    if (!canOpenSwitchScreen.value) {
      return
    }
    screen.value = 'switch'
  }

  const goToOverview = () => {
    screen.value = 'overview'
  }

  onMounted(async () => {
    await loadSnapshot(false)

    const win = await getCurrentWindowSafe()
    if (!win) {
      return
    }

    stopRefreshEvent = await win.listen<CodexTraySnapshot>('codex-tray:refresh', (event) => {
      snapshot.value = event.payload
      error.value = null
    })
  })

  onUnmounted(() => {
    stopRefreshEvent?.()
    stopRefreshEvent = null
  })

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
