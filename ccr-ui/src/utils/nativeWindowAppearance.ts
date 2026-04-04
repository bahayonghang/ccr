import type { Color } from '@tauri-apps/api/window'
import { logger } from '@/utils/logger'
import { getCurrentWindowSafe } from '@/utils/tauriWindow'
import { getClientPlatform } from '@/utils/windowChrome'
import type { ThemeMode } from '@/utils/themeBootstrap'

const MACOS_WINDOW_BACKGROUNDS: Record<ThemeMode, Color> = {
  light: '#F3F1F8',
  dark: '#0F1120',
}

export const shouldSyncNativeWindowAppearance = (): boolean => {
  return getClientPlatform() === 'macos'
}

export const syncNativeWindowAppearance = async (theme: ThemeMode): Promise<void> => {
  if (!shouldSyncNativeWindowAppearance()) {
    return
  }

  const win = await getCurrentWindowSafe()
  if (!win) {
    return
  }

  try {
    await Promise.all([
      win.setTheme(theme),
      win.setBackgroundColor(MACOS_WINDOW_BACKGROUNDS[theme]),
    ])
  } catch (error) {
    logger.warn('[nativeWindowAppearance] failed to sync macOS window appearance', error)
  }
}
