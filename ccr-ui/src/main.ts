import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import i18n, { hydratePreferredLocale } from './i18n'
import { useUIStore } from '@/stores/ui'
import { logger } from '@/utils/logger'
import { scheduleAfterPaint, scheduleWhenIdle } from '@/utils/scheduling'
import { showCurrentWindowIfTauri } from '@/utils/tauriWindow'
import { applyInitialTheme } from '@/utils/themeBootstrap'
import { registerAppIcons } from '@/config/iconRegistry'
import './styles/index.css'

applyInitialTheme()
registerAppIcons()

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(i18n)

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

scheduleWhenIdle(() => {
  void hydratePreferredLocale()
}, { timeout: 1600, fallbackDelay: 320 })
