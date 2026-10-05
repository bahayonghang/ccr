import { useEffect, useState, type RefObject } from 'react'
import { readPrefersReducedMotion } from '@/utils/reducedMotion'

// 动画可见性标志（08-22-state-logic-port 批次 5c；原 Vue composable 语义等价迁移）。
// onMounted/onUnmounted → useEffect + cleanup；IntersectionObserver /
// visibilitychange / prefers-reduced-motion 初值读取逐行保留。
//
// 签名变化（当前全仓无消费方，判定见 state-disposition.md §4.3）：
// - targetRef：Vue Ref → React RefObject<HTMLElement | null>；
// - 返回字段由 Ref/computed 改为普通值。

/**
 * Viewport / page-visibility / reduced-motion driven animation gate.
 * The element is observed once on mount（原实现同样仅在 mounted 时 observe 一次，
 * 后续 ref 变化不重新 observe）。
 */
export function useAnimationVisibility(targetRef: RefObject<HTMLElement | null>) {
  const [isInViewport, setIsInViewport] = useState(true)
  const [isPageVisible, setIsPageVisible] = useState(true)
  const [prefersReducedMotion, setPrefersReducedMotion] = useState(false)

  useEffect(() => {
    if (typeof window === 'undefined') return

    // IntersectionObserver: 检测元素是否在视口内
    const intersectionObserver = new IntersectionObserver(
      (entries) => {
        const entry = entries[0]
        if (entry) {
          setIsInViewport(entry.isIntersecting)
        }
      },
      { threshold: 0 },
    )

    if (targetRef.current) {
      intersectionObserver.observe(targetRef.current)
    }

    // document.visibilitychange: 检测标签页是否可见
    const handleVisibilityChange = () => {
      setIsPageVisible(document.visibilityState === 'visible')
    }
    setIsPageVisible(document.visibilityState === 'visible')
    document.addEventListener('visibilitychange', handleVisibilityChange)

    // prefers-reduced-motion 的读取点已收敛到 reducedMotion.ts（批次 7 单点）
    setPrefersReducedMotion(readPrefersReducedMotion())

    return () => {
      intersectionObserver.disconnect()
      document.removeEventListener('visibilitychange', handleVisibilityChange)
    }
  }, [targetRef])

  // 原 computed(:9)：来源 isInViewport / isPageVisible / prefersReducedMotion
  // → 布尔直算（无需 memo）。
  const shouldAnimate = isInViewport && isPageVisible && !prefersReducedMotion

  return { shouldAnimate, isInViewport, isPageVisible, prefersReducedMotion }
}
