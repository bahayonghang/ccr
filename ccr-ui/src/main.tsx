import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { QueryClientProvider } from '@tanstack/react-query'
import { RouterProvider } from 'react-router'
import { queryClient } from './shell/queryClient'
import { router } from './shell/router'
import { loadDeferredStyles } from './utils/deferredStyles'
import { applyReducedMotionToDocument } from './utils/reducedMotion'
import './styles/index.css'

const container = document.getElementById('app')
if (!container) {
  throw new Error('CCR UI 挂载点缺失：#app 不存在于 index.html')
}

// reduced motion 单点收敛（08-22-design-system 批次 7）：读系统偏好并写入根
// data-reduced-motion 属性，CSS 降级规则统一挂该属性。订阅常驻应用生命周期。
applyReducedMotionToDocument()

// 三层 CSS 加载（code-splitting.md §3.1，08-22-design-system 批次 2）：
// 首屏 CSS 已由上方 `import './styles/index.css'` 同步加载（只含 shell-critical 层）；
// deferred-interactive / deferred-decorations 首帧后与空闲时惰性挂载。
loadDeferredStyles()

// Provider 嵌套顺序见 08-22-react-foundation/design.md §1。
// Zustand 无 Provider（模块级单例）；i18n Provider 由 08-22-i18n-port 补入。
createRoot(container).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </StrictMode>,
)
