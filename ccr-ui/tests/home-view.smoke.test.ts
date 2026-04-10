import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest'

vi.mock('vue-router', () => ({
  useRouter: () => ({
    push: vi.fn(),
  }),
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => key,
  }),
}))

vi.mock('@/api/runtime/system', () => ({
  getSystemInfo: vi.fn(async () => ({
    cpu_usage: 11.4,
    memory_usage_percent: 27.3,
  })),
  getCliVersions: vi.fn(async () => ({
    versions: [
      { platform: 'claude', installed: true, version: '1.0.0', status: 'ok' },
      { platform: 'codex', installed: true, version: '2.1.0', status: 'ok' },
      { platform: 'gemini', installed: true, version: '3.2.0', status: 'ok' },
      { platform: 'qoder', installed: true, version: '0.1.0', status: 'ok' },
    ],
  })),
}))

vi.mock('@/utils/scheduling', () => ({
  scheduleWhenIdle: (callback: () => void) => {
    callback()
    return () => {}
  },
}))

vi.mock('@/utils/perfTelemetry', () => ({
  perfMark: vi.fn(),
  shouldLogPerfTelemetry: () => false,
}))

vi.mock('@/utils/logger', () => ({
  logger: {
    error: vi.fn(),
    info: vi.fn(),
  },
}))

beforeAll(() => {
  class MockIntersectionObserver {
    observe() {}
    disconnect() {}
  }

  vi.stubGlobal('IntersectionObserver', MockIntersectionObserver)
})

const mountHomeView = async () => {
  const HomeView = (await import('@/views/HomeView.vue')).default
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () => h(HomeView)
    },
  }))

  app.component(
    'RouterLink',
    defineComponent({
      props: {
        to: { type: [String, Object], required: true },
      },
      setup(_props, { slots }) {
        return () => h('a', {}, slots.default?.())
      },
    }),
  )

  app.config.globalProperties.$t = (key: string) => key
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

describe('HomeView smoke', () => {
  it('renders the redesigned hero and platform sections', async () => {
    const { el, unmount } = await mountHomeView()

    try {
      expect(el.querySelector('[data-home-hero]')).not.toBeNull()
      expect(el.querySelector('[data-home-actions]')).not.toBeNull()
      expect(el.querySelector('[data-home-platforms]')).not.toBeNull()
      expect(el.querySelector('[data-home-usage-preview]')).not.toBeNull()
      expect(el.querySelector('.page-header-card')).toBeNull()
      expect(el.textContent).toContain('11.4%')
      expect(el.textContent).toContain('27.3%')
      expect(el.textContent).toContain('3/4')
    } finally {
      unmount()
    }
  })

  it('uses professional home copy in both locales', async () => {
    const [{ default: zhCN }, { default: enUS }] = await Promise.all([
      import('@/i18n/locales/zh-CN'),
      import('@/i18n/locales/en-US'),
    ])

    expect(zhCN.home.posterEyebrow).toBe('控制台概览')
    expect(zhCN.home.visualEyebrow).toBe('CLI 状态')
    expect(zhCN.home.visualTitle).toBe('平台概览')
    expect(zhCN.home.actionsEyebrow).toBe('快捷入口')
    expect(zhCN.home.actionsTitle).toBe('常用操作')
    expect(zhCN.home.usageSectionTitle).toBe('使用概览')
    expect(zhCN.home.posterDescription).not.toContain('脉冲')

    expect(enUS.home.posterEyebrow).toBe('Console Overview')
    expect(enUS.home.visualEyebrow).toBe('CLI Status')
    expect(enUS.home.visualTitle).toBe('Platform Overview')
    expect(enUS.home.actionsEyebrow).toBe('Quick Access')
    expect(enUS.home.actionsTitle).toBe('Common Actions')
    expect(enUS.home.usageSectionTitle).toBe('Usage Overview')
    expect(enUS.home.posterDescription).not.toContain('pulse')
    expect(enUS.home.visualTitle).not.toBe('Command fabric')
  })
})
