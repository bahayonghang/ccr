import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { QueryClientProvider } from '@tanstack/react-query'
import { I18nextProvider } from 'react-i18next'
import { RouterProvider } from 'react-router'
import i18n from './i18n'
import { queryClient } from './shell/queryClient'
import { router } from './shell/router'
import '@/shell/stores/shellPreferences'
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
loadDeferredStyles()
initPerfTelemetry()
installStartupErrorHandlers()
logger.info('[startup] react shell mounting')

createRoot(container).render(
  <StrictMode>
    <I18nextProvider i18n={i18n}>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </I18nextProvider>
  </StrictMode>,
)
