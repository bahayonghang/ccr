import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { useTauriEventBridge } from '@/shell/eventBridge'
import { hydrateShellLocale } from '@/shell/i18n'
import { useShellPreferencesStore } from '@/shell/stores/shellPreferences'
import { logger } from '@/utils/logger'
import {
  CODEX_TRAY_PANEL_WINDOW_LABEL,
  getCurrentWindowSafe,
  showCurrentWindowIfTauri,
} from '@/utils/tauriWindow'

/** 外壳启动接线：runtime 偏好、托盘路径、shell:navigate、原生外观。 */
export function useShellRuntime() {
  useTauriEventBridge()
  const navigate = useNavigate()
  const effectiveTheme = useShellPreferencesStore((state) => state.effectiveTheme)
  const hydrateRuntimePreferences = useShellPreferencesStore((state) => state.hydrateRuntimePreferences)

  useEffect(() => {
    void hydrateRuntimePreferences()
    void hydrateShellLocale()
    void showCurrentWindowIfTauri()
  }, [hydrateRuntimePreferences])

  useEffect(() => {
    // 动态导入，避免 nativeWindowAppearance 进入同步图导致 INEFFECTIVE_DYNAMIC_IMPORT。
    void import('@/utils/nativeWindowAppearance')
      .then(({ syncNativeWindowAppearance }) => syncNativeWindowAppearance(effectiveTheme))
      .catch(() => {
        // 浏览器测试环境或非 Tauri 运行时允许静默降级。
      })
  }, [effectiveTheme])

  useEffect(() => {
    let stop: (() => void) | null = null
    void getCurrentWindowSafe().then(async (win) => {
      if (!win) return
      if (win.label === CODEX_TRAY_PANEL_WINDOW_LABEL && window.location.pathname !== '/tray/codex') {
        navigate('/tray/codex', { replace: true })
      }
      stop = await win.listen<string>('shell:navigate', (event) => {
        if (!event.payload || window.location.pathname === event.payload) return
        void navigate(event.payload)
      })
    }).catch((error) => {
      logger.debug('[shell] window bootstrap skipped', error)
    })
    return () => {
      stop?.()
    }
  }, [navigate])
}
