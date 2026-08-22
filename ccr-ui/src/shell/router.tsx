import { createBrowserRouter } from 'react-router'
import { App } from './App'

/**
 * 阶段 1 只接入路由库并验证一条路由可用；
 * 75 条路由的填充归 08-22-shell-port。
 */
export const router = createBrowserRouter([
  {
    path: '/',
    element: <App />,
  },
])
