import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import i18n, { hydratePreferredLocale } from './i18n'
import { registerDeferredIcons, registerShellIcons } from '@/config/iconRegistry'
import { useUIStore } from '@/stores/ui'
import { logger } from '@/utils/logger'
import { scheduleAfterPaint, scheduleWhenIdle } from '@/utils/scheduling'
import { installStartupErrorHandlers, reportStartupFailure } from '@/utils/startupRecovery'
import { showCurrentWindowIfTauri } from '@/utils/tauriWindow'
import { applyInitialTheme } from '@/utils/themeBootstrap'
import { flushPerfTelemetryOnce, initPerfTelemetry, perfMark, perfMeasure } from '@/utils/perfTelemetry'
import { getErrorMessage } from '@/types/api'
import deferredInteractiveHref from './styles/deferred-interactive.css?url'
import deferredDecorationsHref from './styles/deferred-decorations.css?url'
import './styles/index.css'

type DeferredStyleRel = 'preload' | 'stylesheet'

const ensureDeferredStylesheet = (href: string, key: string) => {
  if (typeof document === 'undefined') return
  if (document.head.querySelector(`link[data-font="${key}"]`)) return

  const link = document.createElement('link')
  link.rel = 'stylesheet'
  link.href = href
  link.dataset.font = key
  document.head.appendChild(link)
}

const ensureDeferredStyleLink = (href: string, key: string, rel: DeferredStyleRel) => {
  if (typeof document === 'undefined') return null

  const existing = document.head.querySelector<HTMLLinkElement>(`link[data-style="${key}"]`)
  if (existing) return existing

  const link = document.createElement('link')
  link.dataset.style = key
  link.href = href

  if (rel === 'preload') {
    link.rel = 'preload'
    link.as = 'style'
  } else {
    link.rel = 'stylesheet'
  }

  document.head.appendChild(link)
  return link
}

const preloadDeferredStyles = () => {
  ensureDeferredStyleLink(deferredInteractiveHref, 'deferred-interactive', 'preload')
  ensureDeferredStyleLink(deferredDecorationsHref, 'deferred-decorations', 'preload')
}

const applyDeferredStyle = (href: string, key: string) => {
  if (typeof document === 'undefined') return

  const link = ensureDeferredStyleLink(href, key, 'stylesheet')
  if (!link) return

  if (link.rel !== 'stylesheet') {
    link.rel = 'stylesheet'
    link.removeAttribute('as')
  }
}

initPerfTelemetry()
perfMark('app:main-start')

applyInitialTheme()
perfMark('app:theme-applied')
registerShellIcons()
perfMark('app:icons-shell-registered')
preloadDeferredStyles()
perfMark('app:styles-preloaded')

const configureAppErrorHandler = (app: ReturnType<typeof createApp>) => {
  // 全局错误处理：兜底未捕获的 Vue 组件异常
  app.config.errorHandler = (err, _instance, info) => {
    logger.error(`[Vue Error] ${info}`, err)

    // Pinia 已在上方初始化，store 可安全使用
    try {
      const ui = useUIStore()
      const message = getErrorMessage(err, '未知错误')
      ui.showError(`应用错误: ${message}`)
    } catch {
      // Store 异常时静默降级到 console
    }
  }
}

const scheduleDeferredStartupTasks = (disposeStartupErrorHandlers: () => void) => {
  // 非关键初始化推迟到首帧之后，优先让主界面完成首次绘制。
  scheduleAfterPaint(() => {
    perfMark('app:after-paint')

    applyDeferredStyle(deferredInteractiveHref, 'deferred-interactive')
    perfMark('app:styles-deferred-interactive-applied')
    ensureDeferredStylesheet('/fonts/maplebright/MapleBright-Regular/result.css', 'maplebright-regular-full')
    perfMark('app:font-maplebright-link')

    perfMark('app:i18n-hydrate-start')
    void hydratePreferredLocale().catch((error) => {
      logger.warn('[startup] failed to hydrate preferred locale after first paint', error)
    }).finally(() => {
      perfMark('app:i18n-hydrate-end')
    })

    perfMark('app:icons-deferred-register-start')
    void registerDeferredIcons().catch((error) => {
      logger.warn('[startup] failed to register deferred icons', error)
    }).finally(() => {
      perfMark('app:icons-deferred-register-end')
    })

    scheduleWhenIdle(() => {
      applyDeferredStyle(deferredDecorationsHref, 'deferred-decorations')
      perfMark('app:styles-deferred-decorations-applied')
    }, { timeout: 1200, fallbackDelay: 320 })

    scheduleWhenIdle(() => {
      disposeStartupErrorHandlers()
      perfMark('app:startup-handlers-disposed')
    }, { timeout: 4000, fallbackDelay: 3000 })

    scheduleWhenIdle(() => {
      flushPerfTelemetryOnce('startup:idle')
    }, { timeout: 4500, fallbackDelay: 5200 })
  })
}

const bootstrap = async (disposeStartupErrorHandlers: () => void) => {
  perfMark('app:bootstrap-start')

  const app = createApp(App)

  app.use(createPinia())
  app.use(router)
  app.use(i18n)
  configureAppErrorHandler(app)

  try {
    // 等待初始路由完成匹配后再挂载，避免 web 模式首屏停留在空 RouterView。
    await router.isReady()
  } catch (error) {
    reportStartupFailure('Router initialization', error)
    return
  }

  perfMark('app:router-ready')
  perfMeasure('app:router-init', 'app:bootstrap-start', 'app:router-ready')

  if (import.meta.env.DEV && router.currentRoute.value.matched.length === 0) {
    logger.warn('[router] initial navigation resolved without matched records', {
      path: router.currentRoute.value.fullPath,
    })
  }

  try {
    app.mount('#app')
  } catch (error) {
    reportStartupFailure('Vue mount', error)
    return
  }

  perfMark('app:mounted')
  perfMeasure('app:mount', 'app:router-ready', 'app:mounted')

  void showCurrentWindowIfTauri().catch((error) => {
    logger.warn('[startup] failed to show current window', error)
  })
  perfMark('app:tauri-window-show-requested')

  scheduleDeferredStartupTasks(disposeStartupErrorHandlers)
  perfMark('app:deferred-tasks-scheduled')
}

const disposeStartupErrorHandlers = installStartupErrorHandlers()

void bootstrap(disposeStartupErrorHandlers).catch((error) => {
  reportStartupFailure('Application bootstrap', error)
})
