import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import i18n from './i18n'
import { useUIStore } from '@/stores/ui'
import { logger } from '@/utils/logger'
import { showCurrentWindowIfTauri } from '@/utils/tauriWindow'
import './styles/index.css'

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

// Tauri 模式：Vue 挂载完成后再显示窗口，避免启动黑屏
void showCurrentWindowIfTauri()
