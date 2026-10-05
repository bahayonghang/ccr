import { useCallback, useEffect, type RefObject } from 'react'

// 可访问性 DOM 生命周期 hooks（08-22-state-logic-port 批次 5c；原 Vue composable
// `useAccessibility.ts` 的 SPLIT 迁移）。ariaUtils / focusUtils / useUniqueId 为无状态
// 纯工具，已迁至 `utils/accessibility.ts`；本文件只保留依赖 document keydown 监听的
// 两个 DOM 生命周期 hook（onMounted/onUnmounted → useEffect + cleanup）。
//
// 签名变化（消费方均为待迁移 .vue 组件）：
// - containerRef：Vue Ref → React RefObject<HTMLElement | null>；
// - enabled：Vue Ref<boolean>（默认 ref(true)）→ 普通 boolean（默认 true），
//   原实现事件时读 enabled.value，现经 effect 重挂达到同一语义。

const FOCUSABLE_SELECTOR = [
  'a[href]',
  'button:not([disabled])',
  'input:not([disabled])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  '[tabindex]:not([tabindex="-1"])',
].join(',')

/**
 * Focus trap hook for modals and other focus-constrained containers.
 *
 * @param containerRef container element reference
 * @param enabled whether the trap is active (default `true`)
 */
export function useFocusTrap(
  containerRef: RefObject<HTMLElement | null>,
  enabled: boolean = true
) {
  useEffect(() => {
    if (!enabled) return

    const handleTabKey = (event: KeyboardEvent) => {
      const container = containerRef.current
      if (!container) return
      if (event.key !== 'Tab') return

      const focusableElements = Array.from(
        container.querySelectorAll(FOCUSABLE_SELECTOR)
      ) as HTMLElement[]

      if (focusableElements.length === 0) return

      const firstElement = focusableElements[0]
      const lastElement = focusableElements[focusableElements.length - 1]

      // Shift + Tab: 焦点在第一个元素时跳到最后一个
      if (event.shiftKey && document.activeElement === firstElement) {
        event.preventDefault()
        lastElement.focus()
      }
      // Tab: 焦点在最后一个元素时跳到第一个
      else if (!event.shiftKey && document.activeElement === lastElement) {
        event.preventDefault()
        firstElement.focus()
      }
    }

    document.addEventListener('keydown', handleTabKey)
    return () => {
      document.removeEventListener('keydown', handleTabKey)
    }
  }, [containerRef, enabled])

  /** Focus the first focusable element inside the container. */
  const focusFirstElement = useCallback(() => {
    const container = containerRef.current
    if (!container) return

    const focusableElements = container.querySelectorAll(FOCUSABLE_SELECTOR)
    const firstElement = focusableElements[0] as HTMLElement | undefined

    if (firstElement) {
      firstElement.focus()
    }
  }, [containerRef])

  return {
    focusFirstElement,
  }
}

/**
 * Escape-to-close hook for modals, dropdowns and similar surfaces.
 *
 * @param callback invoked when Escape is pressed while enabled
 * @param enabled whether the listener is active (default `true`)
 */
export function useEscapeKey(
  callback: () => void,
  enabled: boolean = true
) {
  useEffect(() => {
    if (!enabled) return

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        callback()
      }
    }

    document.addEventListener('keydown', handleKeyDown)
    return () => {
      document.removeEventListener('keydown', handleKeyDown)
    }
  }, [callback, enabled])
}
