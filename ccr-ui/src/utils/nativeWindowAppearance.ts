import type { Color } from '@tauri-apps/api/window'
import { logger } from '@/utils/logger'
import { getCurrentWindowSafe } from '@/utils/tauriWindow'
import { getClientPlatform } from '@/utils/windowChrome'
import type { ResolvedThemeMode } from '@/utils/themeBootstrap'

const MACOS_WINDOW_BACKGROUNDS: Record<ResolvedThemeMode, Color> = {
  light: '#EEF4FF',
  dark: '#07121F',
}

export const shouldSyncNativeWindowAppearance = (): boolean => {
  return getClientPlatform() === 'macos'
}

export const syncNativeWindowAppearance = async (theme: ResolvedThemeMode): Promise<void> => {
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
