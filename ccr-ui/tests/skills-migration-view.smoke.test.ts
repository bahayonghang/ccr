import { createApp, defineComponent, h, nextTick } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createI18nStub } from './helpers/i18n-stub'

const systemMocks = vi.hoisted(() => ({
  detectSkillportApp: vi.fn(),
  isTauriEnvironment: vi.fn(),
  openSkillportApp: vi.fn(),
}))

vi.mock('@/api/domains/system', () => ({
  detectSkillportApp: (...args: unknown[]) => systemMocks.detectSkillportApp(...args),
  isTauriEnvironment: (...args: unknown[]) => systemMocks.isTauriEnvironment(...args),
  openSkillportApp: (...args: unknown[]) => systemMocks.openSkillportApp(...args),
}))

vi.mock('@/utils/logger', () => ({
  logger: {
    info: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
  },
}))

const flush = async () => {
  await Promise.resolve()
  await nextTick()
  await nextTick()
}

const mountView = async (component: unknown) => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: defineComponent({ template: '<div />' }) },
      { path: '/skills', component: defineComponent({ template: '<div />' }) },
      { path: '/configs', component: defineComponent({ template: '<div />' }) },
    ],
  })

  const app = createApp(defineComponent({
    setup() {
      return () => h(component as never)
    },
  }))

  app.use(router)
  app.use(createI18nStub('zh-CN'))
  await router.push('/skills')
  await router.isReady()
  app.mount(el)
  await flush()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

beforeEach(() => {
  document.body.innerHTML = ''
  vi.resetModules()

  systemMocks.detectSkillportApp.mockReset()
  systemMocks.isTauriEnvironment.mockReset()
  systemMocks.openSkillportApp.mockReset()

  systemMocks.isTauriEnvironment.mockReturnValue(true)
  systemMocks.detectSkillportApp.mockResolvedValue({
    supported: true,
    installed: false,
    platform: 'windows',
    source: 'not_found',
  })
  systemMocks.openSkillportApp.mockResolvedValue(undefined)
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('SkillsMigrationView smoke', () => {
  it('renders the installed state with a launch button', async () => {
    systemMocks.detectSkillportApp.mockResolvedValue({
      supported: true,
      installed: true,
      platform: 'windows',
      source: 'registry',
    })

    const { default: SkillsMigrationView } = await import('@/views/SkillsMigrationView.vue')
    const { el, unmount } = await mountView(SkillsMigrationView)

    try {
      expect(el.textContent).toContain('已检测到本机安装')
      expect(el.querySelector('[data-testid="skills-migration-primary"]')?.textContent).toContain('打开 skillport')
    } finally {
      unmount()
    }
  })

  it('falls back to the repository CTA when the app is not installed', async () => {
    const { default: SkillsMigrationView } = await import('@/views/SkillsMigrationView.vue')
    const { el, unmount } = await mountView(SkillsMigrationView)

    try {
      expect(el.textContent).toContain('当前没有检测到本机安装')

      const primaryLink = el.querySelector<HTMLAnchorElement>('[data-testid="skills-migration-primary"]')
      expect(primaryLink?.textContent).toContain('前往 skillport 仓库')
      expect(primaryLink?.getAttribute('href')).toBe('https://github.com/bahayonghang/skills-manage-windows')
    } finally {
      unmount()
    }
  })

  it('shows the unsupported state without invoking native detection in web runtime', async () => {
    systemMocks.isTauriEnvironment.mockReturnValue(false)

    const { default: SkillsMigrationView } = await import('@/views/SkillsMigrationView.vue')
    const { el, unmount } = await mountView(SkillsMigrationView)

    try {
      expect(systemMocks.detectSkillportApp).not.toHaveBeenCalled()
      expect(el.textContent).toContain('当前运行环境暂不支持自动检测')
    } finally {
      unmount()
    }
  })

  it('keeps the repository fallback visible when launching fails', async () => {
    systemMocks.detectSkillportApp.mockResolvedValue({
      supported: true,
      installed: true,
      platform: 'macos',
      source: 'bundle_id',
    })
    systemMocks.openSkillportApp.mockRejectedValue(new Error('launch failed'))

    const { default: SkillsMigrationView } = await import('@/views/SkillsMigrationView.vue')
    const { el, unmount } = await mountView(SkillsMigrationView)

    try {
      const primaryButton = el.querySelector<HTMLButtonElement>('[data-testid="skills-migration-primary"]')
      primaryButton?.click()
      await flush()

      expect(el.querySelector('[data-testid="skills-migration-error"]')?.textContent).toContain('已检测到 skillport，但拉起失败')

      const helperLink = el.querySelector<HTMLAnchorElement>('.skills-migration-view__helper-link')
      expect(helperLink?.getAttribute('href')).toBe('https://github.com/bahayonghang/skills-manage-windows')
    } finally {
      unmount()
    }
  })
})
