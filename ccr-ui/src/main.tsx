import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { QueryClientProvider } from '@tanstack/react-query'
import { RouterProvider } from 'react-router'
import { queryClient } from './shell/queryClient'
import { router } from './shell/router'
import './styles/index.css'

const container = document.getElementById('app')
if (!container) {
  throw new Error('CCR UI 挂载点缺失：#app 不存在于 index.html')
}

// Provider 嵌套顺序见 08-22-react-foundation/design.md §1。
// Zustand 无 Provider（模块级单例）；i18n Provider 由 08-22-i18n-port 补入。
createRoot(container).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </StrictMode>,
)
