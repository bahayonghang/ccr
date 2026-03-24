import type { Window as TauriWindow } from '@tauri-apps/api/window'
import { logger } from '@/utils/logger'
import { isTauriRuntime } from '@/utils/tauriRuntime'

let windowPromise: Promise<TauriWindow | null> | null = null

export const getCurrentWindowSafe = async (): Promise<TauriWindow | null> => {
  if (!isTauriRuntime()) {
    return null
  }

  if (!windowPromise) {
    windowPromise = import('@tauri-apps/api/window')
      .then(({ getCurrentWindow }) => getCurrentWindow())
      .catch((error) => {
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

  try {
    await win.show()
  } catch (error) {
    logger.warn('[tauriWindow] failed to show current window', error)
  }
}
