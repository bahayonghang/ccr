import { onMounted, onUnmounted, ref, type Ref } from 'vue'

/**
 * useAccessibility composable
 * 提供焦点管理和 ARIA 属性生成工具
 */

/**
 * 焦点陷阱 Hook
 * 用于模态框等需要限制焦点范围的组件
 * 
 * @param containerRef 容器元素引用
 * @param enabled 是否启用焦点陷阱
 */
export function useFocusTrap(
  containerRef: Ref<HTMLElement | null>,
  enabled: Ref<boolean> = ref(true)
) {
  const FOCUSABLE_SELECTOR = [
    'a[href]',
    'button:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    'textarea:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
  ].join(',')

  const handleTabKey = (event: KeyboardEvent) => {
    if (!enabled.value || !containerRef.value) return
    if (event.key !== 'Tab') return

    const focusableElements = Array.from(
      containerRef.value.querySelectorAll(FOCUSABLE_SELECTOR)
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

  onMounted(() => {
    document.addEventListener('keydown', handleTabKey)
  })

  onUnmounted(() => {
    document.removeEventListener('keydown', handleTabKey)
  })

  /**
   * 聚焦到容器内第一个可聚焦元素
   */
  const focusFirstElement = () => {
    if (!containerRef.value) return

    const focusableElements = containerRef.value.querySelectorAll(FOCUSABLE_SELECTOR)
    const firstElement = focusableElements[0] as HTMLElement | undefined
    
    if (firstElement) {
      firstElement.focus()
    }
  }

  return {
    focusFirstElement,
  }
}

/**
 * Escape 键关闭 Hook
 * 用于模态框、下拉菜单等组件
 * 
 * @param callback 按下 Escape 时的回调函数
 * @param enabled 是否启用
 */
export function useEscapeKey(
  callback: () => void,
  enabled: Ref<boolean> = ref(true)
) {
  const handleKeyDown = (event: KeyboardEvent) => {
    if (enabled.value && event.key === 'Escape') {
      callback()
    }
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeyDown)
  })

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeyDown)
  })
}

/**
 * ARIA 属性生成器
 */
export const ariaUtils = {
  /**
   * 生成展开/折叠控件的 ARIA 属性
   */
  expandable(expanded: boolean, controlsId?: string) {
    return {
      'aria-expanded': expanded.toString(),
      ...(controlsId && { 'aria-controls': controlsId }),
    }
  },

  /**
   * 生成描述关联的 ARIA 属性
   */
  describedBy(ids: string | string[]) {
    const idString = Array.isArray(ids) ? ids.join(' ') : ids
    return {
      'aria-describedby': idString,
    }
  },

  /**
   * 生成标签关联的 ARIA 属性
   */
  labelledBy(ids: string | string[]) {
    const idString = Array.isArray(ids) ? ids.join(' ') : ids
    return {
      'aria-labelledby': idString,
    }
  },

  /**
   * 生成 live region 的 ARIA 属性
   */
  liveRegion(politeness: 'polite' | 'assertive' | 'off' = 'polite', atomic: boolean = false) {
    return {
      'aria-live': politeness,
      ...(atomic && { 'aria-atomic': 'true' }),
    }
  },

  /**
   * 生成禁用状态的 ARIA 属性
   */
  disabled(disabled: boolean) {
    return disabled ? { 'aria-disabled': 'true' } : {}
  },

  /**
   * 生成选中状态的 ARIA 属性
   */
  checked(checked: boolean) {
    return {
      'aria-checked': checked.toString(),
    }
  },

  /**
   * 生成当前状态的 ARIA 属性
   * 用于导航项等
   */
  current(type: 'page' | 'step' | 'location' | 'date' | 'time' | 'true' | 'false' = 'page') {
    return {
      'aria-current': type,
    }
  },
}

/**
 * 生成唯一 ID
 * 用于 ARIA 关联
 */
let idCounter = 0
export function useUniqueId(prefix: string = 'a11y'): string {
  return `${prefix}-${++idCounter}`
}

/**
 * 焦点管理工具
 */
export const focusUtils = {
  /**
   * 保存并恢复焦点
   * 用于模态框打开/关闭时恢复焦点
   */
  createFocusStore() {
    let previousElement: HTMLElement | null = null

    return {
      save() {
        previousElement = document.activeElement as HTMLElement
      },
      restore() {
        if (previousElement && previousElement.focus) {
          previousElement.focus()
        }
      },
    }
  },

  /**
   * 移动焦点到指定元素
   */
  moveTo(element: HTMLElement | null) {
    if (element && element.focus) {
      element.focus()
    }
  },

  /**
   * 移动焦点到匹配选择器的第一个元素
   */
  moveToSelector(selector: string, container: HTMLElement | Document = document) {
    const element = container.querySelector(selector) as HTMLElement | null
    this.moveTo(element)
  },
}
