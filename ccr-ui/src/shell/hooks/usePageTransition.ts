import { useEffect, useRef, useState } from 'react'

// 路由感知过渡动画 hook（08-22-state-logic-port 批次 5c；自
// `composables/usePageTransition.ts` 迁入 shell/hooks，原文件删除）。
//
// 原 vue-router beforeEach 守卫无 React 等价物，改为「调用方喂当前路由描述、
// hook 内部对比前后路由」的形态。MainLayout 接线归 08-22-shell-port：
//
//   const route = useRoute()  // 或 React Router 的 useLocation + 自有路由表
//   const { transitionName } = usePageTransition({
//     depth: route.meta.depth,
//     group: route.meta.group,
//   })
//   // <transition :name="transitionName"> 对应物：以 pathname 为 key 的过渡容器
//
// 语义保留：
// - depth/group 缺省回退（?? 1 / ?? ''）逐行对应原 meta 读取；
// - popstate → isBack 标记：window 监听保留在 hook 内部，消费后复位；
// - 首次导航 / 深层链接（prev 为 null，对应原 `!from.name`）→ 'page-fade'。
//
// 签名变化：usePageTransition() 无参（内部取 vue-router）→
// usePageTransition(current: PageRouteInfo)；transitionName 由 Ref 改为普通值。

export type PageTransitionName =
  | 'page-fade'
  | 'page-slide-up'
  | 'page-slide-down'
  | 'page-cross-fade'
  | 'page-slide-lateral'

/** 当前路由的过渡判定输入（来自路由 meta：depth 层级、group 分组）。 */
export interface PageRouteInfo {
  depth?: number
  group?: string
}

const resolveDepth = (info: PageRouteInfo): number => info.depth ?? 1
const resolveGroup = (info: PageRouteInfo): string => info.group ?? ''

/**
 * Route-aware transition name picker.
 * Compares the previous and current route descriptors on every change;
 * the transition starts as `'page-fade'` (first load / deep link).
 */
export function usePageTransition(current: PageRouteInfo): {
  transitionName: PageTransitionName
} {
  const [transitionName, setTransitionName] = useState<PageTransitionName>('page-fade')
  const prevRef = useRef<PageRouteInfo | null>(null)
  // 后退/前进标记：跨渲染存活（popstate 监听与导航判定在不同渲染闭包中执行）
  const isBackRef = useRef(false)

  // 监听 popstate 检测浏览器后退/前进（原 window 级监听语义保留）
  useEffect(() => {
    const onPopState = () => {
      isBackRef.current = true
    }
    window.addEventListener('popstate', onPopState)
    return () => window.removeEventListener('popstate', onPopState)
  }, [])

  const depth = current.depth
  const group = current.group

  useEffect(() => {
    const prev = prevRef.current
    const nextInfo = { depth, group }

    if (!prev) {
      setTransitionName('page-fade')
      isBackRef.current = false
      prevRef.current = nextInfo
      return
    }

    const toDepth = resolveDepth(nextInfo)
    const fromDepth = resolveDepth(prev)
    const toGroup = resolveGroup(nextInfo)
    const fromGroup = resolveGroup(prev)

    // 顶层大页面统一使用轻量淡入淡出，避免复杂滑动动画拖慢切换
    if (toDepth === 1 && fromDepth === 1) {
      setTransitionName('page-fade')
      isBackRef.current = false
      prevRef.current = nextInfo
      return
    }

    if (isBackRef.current) {
      // 浏览器后退 → 反向动画
      if (fromDepth > toDepth) {
        setTransitionName('page-slide-down')
      } else if (fromDepth < toDepth) {
        setTransitionName('page-slide-up')
      } else {
        setTransitionName('page-cross-fade')
      }
    } else if (toDepth > fromDepth) {
      // 向下导航（进入子页面）
      setTransitionName('page-slide-up')
    } else if (toDepth < fromDepth) {
      // 向上导航（返回上级）
      setTransitionName('page-slide-down')
    } else if (toGroup && fromGroup && toGroup === fromGroup) {
      // 同组内同级切换（如 codex/mcp → codex/profiles）
      setTransitionName('page-slide-lateral')
    } else {
      // 不同组同级切换（如 claude-code → codex）
      setTransitionName('page-cross-fade')
    }

    isBackRef.current = false
    prevRef.current = nextInfo
  }, [depth, group])

  return { transitionName }
}
