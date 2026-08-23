import { logger } from '@/utils/logger'
import { scheduleAfterPaint, scheduleWhenIdle } from '@/utils/scheduling'

/**
 * 三层 CSS 加载（08-22-arch-quality-perf/code-splitting.md §3.1 的 React 落地，
 * 08-22-design-system 批次 2）：
 *   - 首屏 CSS 只含 shell-critical 层（styles/index.css → core.css 链，同步加载）；
 *   - deferred-interactive（非关键交互样式，含自定义工具类/动画/图表配色）在首帧后加载；
 *   - deferred-decorations（装饰与氛围层）在空闲时加载。
 * 挂载形态与旧 Vue main.ts 等价：`<link rel="stylesheet" data-style="deferred-*">`，幂等。
 * 每层只挂载一次（按 data-style 去重）；加载失败仅降级告警，不阻断应用。
 */

const deferredStyleLoaders = {
  interactive: () => import('../styles/deferred-interactive.css?url'),
  decorations: () => import('../styles/deferred-decorations.css?url'),
} as const

type DeferredStyleKey = keyof typeof deferredStyleLoaders

const applyDeferredStyle = (href: string, key: string): void => {
  if (typeof document === 'undefined') return

  const existing = document.head.querySelector<HTMLLinkElement>(`link[data-style="${key}"]`)
  if (existing) return

  const link = document.createElement('link')
  link.dataset.style = key
  link.rel = 'stylesheet'
  link.href = href
  document.head.appendChild(link)
}

const applyDeferredStyleFromImport = async (key: DeferredStyleKey): Promise<void> => {
  try {
    const href = (await deferredStyleLoaders[key]()).default
    applyDeferredStyle(href, `deferred-${key}`)
  } catch (error) {
    logger.warn(`[startup] failed to load deferred ${key} styles`, error)
  }
}

/** 注册三层 CSS 加载时序：首帧后载 interactive，空闲时载 decorations。 */
export const loadDeferredStyles = (): void => {
  scheduleAfterPaint(() => {
    void applyDeferredStyleFromImport('interactive')
  })

  scheduleWhenIdle(() => {
    void applyDeferredStyleFromImport('decorations')
  }, { timeout: 1200, fallbackDelay: 320 })
}
