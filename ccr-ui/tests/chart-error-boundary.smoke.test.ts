import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import ChartErrorBoundary from '@/components/claude-observer/ChartErrorBoundary.vue'
import { createI18nStub } from './helpers/i18n-stub'

/*
 * 回归：claude-observer 图表自愈式错误边界
 * ------------------------------------------------------------------
 * ApexCharts 在异步 init/动画窗口里可能抛 `reading 'node'`，经子组件同步抛出会冒泡到
 * main.ts 全局 errorHandler 弹“应用错误”吐司。ChartErrorBoundary 必须：
 *   1) 接住该异常、不向上冒泡（app.config.errorHandler 不被调用）；
 *   2) 超过重试上限降级为准备态；
 *   3) 瞬时错误可自愈重挂、最终渲染出真实图表内容。
 */

const PREPARING_LABEL = 'Preparing trend chart…'

const mountBoundary = (
  slotRender: (ctx: { reloadKey: number }) => ReturnType<typeof h>,
  maxRetries?: number
) => {
  const errorHandler = vi.fn()
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(
    defineComponent({
      render() {
        return h(
          ChartErrorBoundary,
          { label: PREPARING_LABEL, maxRetries },
          { default: slotRender }
        )
      },
    })
  )

  app.use(createI18nStub())
  app.config.errorHandler = errorHandler
  app.mount(el)

  return {
    el,
    errorHandler,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

// 等待一次 requestAnimationFrame（jsdom 用定时器实现）后续重挂生效
const flushReload = async () => {
  await new Promise((resolve) => setTimeout(resolve, 40))
  await nextTick()
  await nextTick()
}

afterEach(() => {
  document.body.innerHTML = ''
})

describe('ChartErrorBoundary', () => {
  it('contains a chart render error and degrades without propagating', async () => {
    const ThrowingChart = defineComponent({
      name: 'ThrowingChart',
      setup() {
        throw new Error("Cannot read properties of undefined (reading 'node')")
      },
      render() {
        return h('div', { class: 'chart-ok' }, 'chart-ok')
      },
    })

    // maxRetries=0：首挂即抛 -> 直接永久降级，断言纯粹的「接住 + 降级 + 不冒泡」
    const mounted = mountBoundary(({ reloadKey }) => h(ThrowingChart, { key: reloadKey }), 0)
    await nextTick()
    await nextTick()

    try {
      expect(mounted.errorHandler).not.toHaveBeenCalled()
      expect(mounted.el.textContent).toContain(PREPARING_LABEL)
      expect(mounted.el.querySelector('.chart-ok')).toBeNull()
    } finally {
      mounted.unmount()
    }
  })

  it('self-heals by remounting after a transient chart error', async () => {
    let attempts = 0
    const FlakyChart = defineComponent({
      name: 'FlakyChart',
      setup() {
        attempts += 1
        if (attempts === 1) {
          throw new Error("Cannot read properties of undefined (reading 'node')")
        }
      },
      render() {
        return h('div', { class: 'chart-ok' }, 'chart-ok')
      },
    })

    const mounted = mountBoundary(({ reloadKey }) => h(FlakyChart, { key: reloadKey }))
    await flushReload()

    try {
      expect(mounted.errorHandler).not.toHaveBeenCalled()
      expect(mounted.el.textContent).toContain('chart-ok')
      expect(attempts).toBeGreaterThanOrEqual(2)
    } finally {
      mounted.unmount()
    }
  })
})
