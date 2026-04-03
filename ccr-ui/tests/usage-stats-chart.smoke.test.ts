import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, describe, expect, it } from 'vitest'
import UsageStatsChart from '@/components/UsageStatsChart.vue'

const mountChart = async () => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    render() {
      return h(UsageStatsChart, {
        viewMode: 'requests',
        data: [
          {
            date: '2026-04-01',
            claude: { sessions: 0, requests: 3, tokens: 300 },
            codex: { sessions: 0, requests: 5, tokens: 500 },
            gemini: { sessions: 0, requests: 2, tokens: 200 },
            qwen: { sessions: 0, requests: 17, tokens: 1700 },
          },
          {
            date: '2026-04-02',
            claude: { sessions: 0, requests: 1, tokens: 100 },
            codex: { sessions: 0, requests: 2, tokens: 200 },
            gemini: { sessions: 0, requests: 4, tokens: 400 },
            qwen: { sessions: 0, requests: 9, tokens: 900 },
          },
        ],
      })
    },
  }))

  app.mount(el)
  await nextTick()
  await nextTick()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

afterEach(() => {
  document.body.innerHTML = ''
})

describe('usage stats chart smoke', () => {
  it('renders qwen legend, stack segment, and hover tooltip', async () => {
    const { el, unmount } = await mountChart()

    try {
      expect(el.textContent).toContain('Qwen')
      expect(el.querySelectorAll('[data-platform="qwen"]').length).toBeGreaterThan(0)

      const firstQwenSegment = el.querySelector('[data-platform="qwen"]') as HTMLElement | null
      expect(firstQwenSegment).not.toBeNull()

      const row = firstQwenSegment?.closest('.cursor-pointer') as HTMLElement | null
      expect(row).not.toBeNull()

      row?.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true }))
      await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()))
      await nextTick()

      const tooltip = el.querySelector('.absolute.z-20') as HTMLElement | null
      expect(tooltip?.textContent).toContain('Qwen')
      expect(tooltip?.textContent).toContain('17')
    } finally {
      unmount()
    }
  })
})
