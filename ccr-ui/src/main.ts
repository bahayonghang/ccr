import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import i18n, { hydratePreferredLocale } from './i18n'
import { useUIStore } from '@/stores/ui'
import { logger } from '@/utils/logger'
import { scheduleAfterPaint } from '@/utils/scheduling'
import { showCurrentWindowIfTauri } from '@/utils/tauriWindow'
import { applyInitialTheme } from '@/utils/themeBootstrap'
import './styles/index.css'

applyInitialTheme()
await Promise.all([
  hydratePreferredLocale(),
  import('@/config/iconRegistry').then(({ registerAppIcons }) => {
    registerAppIcons()
  }),
])

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(i18n)

// 等待初始路由完成匹配后再挂载，避免 web 模式首屏停留在空 RouterView。
await router.isReady()

if (import.meta.env.DEV && router.currentRoute.value.matched.length === 0) {
  logger.warn('[router] initial navigation resolved without matched records', {
    path: router.currentRoute.value.fullPath,
  })
}

// 全局错误处理：兜底未捕获的 Vue 组件异常
app.config.errorHandler = (err, _instance, info) => {
  logger.error(`[Vue Error] ${info}`, err)

  // Pinia 已在上方初始化，store 可安全使用
  try {
    const ui = useUIStore()
    const message = err instanceof Error ? err.message : '未知错误'
    ui.showError(`应用错误: ${message}`)
  } catch {
    // Store 异常时静默降级到 console
  }
}

app.mount('#app')

// 非关键初始化推迟到首帧之后，优先让主界面完成首次绘制。
scheduleAfterPaint(() => {
  void showCurrentWindowIfTauri()
})
