// reduced motion 单点收敛（08-22-design-system 批次 7 / design.md §9）。
//
// 全应用对 prefers-reduced-motion 的响应只有这一处读系统偏好：本模块读
// matchMedia 并把结果镜像到根元素的 data-reduced-motion 属性（'true' | 'false'），
// CSS 侧的全部降级规则挂在该属性下（不散写 @media，.vue 组件内的存量
// @media 随阶段 5 视图迁移按此约定收敛）。
//
// 唯一的 @media 兜底保留在 shell-critical.css（首帧无 JS 时 critical 层的
// 加载指示器仍需降级，见 animation-disposition.md §reduced-motion）。
//
// motion（13.1.1）侧：消费方用 MotionConfig 的 reducedMotion="user" 或本模块
// 的 readPrefersReducedMotion()，不再各自读 matchMedia，避免双轨判定。

export const REDUCED_MOTION_QUERY = '(prefers-reduced-motion: reduce)'
export const REDUCED_MOTION_ATTRIBUTE = 'data-reduced-motion'

export const readPrefersReducedMotion = (): boolean => {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return false
  }
  return window.matchMedia(REDUCED_MOTION_QUERY).matches
}

const writeAttribute = (reduced: boolean): void => {
  if (typeof document === 'undefined') return
  document.documentElement.setAttribute(REDUCED_MOTION_ATTRIBUTE, String(reduced))
}

export interface ReducedMotionSubscription {
  /** 当前是否偏好减少动效。 */
  reduced: boolean
  /** 解除监听（系统偏好变化不再同步到根属性）。 */
  dispose: () => void
}

/**
 * 把系统 reduced-motion 偏好同步到根元素的 data-reduced-motion 属性，
 * 并跟随系统设置变化。返回当前值与解除函数；无 window 环境（SSR/测试）时为 no-op。
 */
export const applyReducedMotionToDocument = (): ReducedMotionSubscription => {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return { reduced: false, dispose: () => {} }
  }

  const mediaQuery = window.matchMedia(REDUCED_MOTION_QUERY)
  writeAttribute(mediaQuery.matches)

  const handleChange = (event: MediaQueryListEvent) => writeAttribute(event.matches)
  mediaQuery.addEventListener('change', handleChange)

  return {
    reduced: mediaQuery.matches,
    dispose: () => mediaQuery.removeEventListener('change', handleChange),
  }
}
