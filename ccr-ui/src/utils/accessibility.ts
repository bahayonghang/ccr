// 无状态可访问性纯工具（08-22-state-logic-port 批次 5c）。
// 自 `composables/useAccessibility.ts` 拆出（SPLIT：工具部分 → utils，判定见
// arch-quality-perf state-disposition.md §4.3）。无框架依赖，Vue/React 两侧均可直接使用。

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
