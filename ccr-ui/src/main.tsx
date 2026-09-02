import { StrictMode, useEffect, type ReactNode } from 'react'
import { createRoot } from 'react-dom/client'
import { QueryClientProvider } from '@tanstack/react-query'
import { I18nextProvider } from 'react-i18next'
import { RouterProvider } from 'react-router'
import i18n from './i18n'
import { queryClient } from './shell/queryClient'
import { router } from './shell/router'
import '@/shell/stores/shellPreferences'
import { registerDeferredIcons, registerShellIcons } from '@/config/iconRegistry'
import { loadDeferredStyles } from './utils/deferredStyles'
import { initPerfTelemetry } from './utils/perfTelemetry'
import { applyReducedMotionToDocument } from './utils/reducedMotion'
import { installStartupErrorHandlers } from './utils/startupRecovery'
import { logger } from './utils/logger'
import './styles/index.css'

const container = document.getElementById('app')
if (!container) {
  throw new Error('CCR UI 挂载点缺失：#app 不存在于 index.html')
}

applyReducedMotionToDocument()
registerShellIcons()
void registerDeferredIcons().catch(() => {
  /* 延迟图标注册失败时保持壳层子集，不让启动 Promise 成为未处理拒绝 */
})
loadDeferredStyles()
initPerfTelemetry()
const uninstallStartupErrorHandlers = installStartupErrorHandlers()
logger.info('[startup] react shell mounting')

function ReactShellRoot({
  onMounted,
  children,
}: {
  onMounted: () => void
  children: ReactNode
}) {
  useEffect(() => {
    onMounted()
  }, [onMounted])
  return children
}

createRoot(container).render(
  <StrictMode>
    <ReactShellRoot onMounted={uninstallStartupErrorHandlers}>
      <I18nextProvider i18n={i18n}>
        <QueryClientProvider client={queryClient}>
          <RouterProvider router={router} />
        </QueryClientProvider>
      </I18nextProvider>
    </ReactShellRoot>
  </StrictMode>,
)
