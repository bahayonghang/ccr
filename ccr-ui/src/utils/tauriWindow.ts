import type { Window as TauriWindow } from '@tauri-apps/api/window'
import { logger } from '@/utils/logger'
import { isTauriRuntime } from '@/utils/tauriRuntime'

export const CODEX_TRAY_PANEL_WINDOW_LABEL = 'codex-tray-panel'

let windowPromise: Promise<TauriWindow | null> | null = null

const waitForTauriRuntime = async (timeoutMs = 3000): Promise<boolean> => {
  if (isTauriRuntime()) {
    return true
  }

  if (typeof window === 'undefined') {
    return false
  }

  const start = Date.now()

  while (Date.now() - start < timeoutMs) {
    await new Promise((resolve) => window.setTimeout(resolve, 50))

    if (isTauriRuntime()) {
      return true
    }
  }

  return false
}

export const getCurrentWindowSafe = async (): Promise<TauriWindow | null> => {
  const tauriReady = await waitForTauriRuntime()
  if (!tauriReady) {
    return null
  }

  if (!windowPromise) {
    windowPromise = import('@tauri-apps/api/window')
      .then(({ getCurrentWindow }) => getCurrentWindow())
      .catch((error) => {
        windowPromise = null
        logger.warn('[tauriWindow] failed to get current window', error)
        return null
      })
  }

  return windowPromise
}

export const showCurrentWindowIfTauri = async (): Promise<void> => {
  const win = await getCurrentWindowSafe()
  if (!win) {
    return
  }

  if (win.label === CODEX_TRAY_PANEL_WINDOW_LABEL) {
    return
  }

  try {
    await win.show()
  } catch (error) {
    logger.warn('[tauriWindow] failed to show current window', error)
  }
}
