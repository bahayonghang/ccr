import { createApp, defineComponent, h, nextTick } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const systemMocks = vi.hoisted(() => ({
  detectSkillsManageApp: vi.fn(),
  isTauriEnvironment: vi.fn(),
  openSkillsManageApp: vi.fn(),
}))

vi.mock('@/api/domains/system', () => ({
  detectSkillsManageApp: (...args: unknown[]) => systemMocks.detectSkillsManageApp(...args),
  isTauriEnvironment: (...args: unknown[]) => systemMocks.isTauriEnvironment(...args),
  openSkillsManageApp: (...args: unknown[]) => systemMocks.openSkillsManageApp(...args),
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

  systemMocks.detectSkillsManageApp.mockReset()
  systemMocks.isTauriEnvironment.mockReset()
  systemMocks.openSkillsManageApp.mockReset()

  systemMocks.isTauriEnvironment.mockReturnValue(true)
  systemMocks.detectSkillsManageApp.mockResolvedValue({
    supported: true,
    installed: false,
    platform: 'windows',
    source: 'not_found',
  })
  systemMocks.openSkillsManageApp.mockResolvedValue(undefined)
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('SkillsMigrationView smoke', () => {
  it('renders the installed state with a launch button', async () => {
    systemMocks.detectSkillsManageApp.mockResolvedValue({
      supported: true,
      installed: true,
      platform: 'windows',
      source: 'registry',
    })

    const { default: SkillsMigrationView } = await import('@/views/SkillsMigrationView.vue')
    const { el, unmount } = await mountView(SkillsMigrationView)

    try {
      expect(el.textContent).toContain('已检测到本机安装')
      expect(el.querySelector('[data-testid="skills-migration-primary"]')?.textContent).toContain('打开 skills-manage')
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
      expect(primaryLink?.textContent).toContain('前往 skills-manage 仓库')
      expect(primaryLink?.getAttribute('href')).toBe('https://github.com/iamzhihuix/skills-manage')
    } finally {
      unmount()
    }
  })

  it('shows the unsupported state without invoking native detection in web runtime', async () => {
    systemMocks.isTauriEnvironment.mockReturnValue(false)

    const { default: SkillsMigrationView } = await import('@/views/SkillsMigrationView.vue')
    const { el, unmount } = await mountView(SkillsMigrationView)

    try {
      expect(systemMocks.detectSkillsManageApp).not.toHaveBeenCalled()
      expect(el.textContent).toContain('当前运行环境暂不支持自动检测')
    } finally {
      unmount()
    }
  })

  it('keeps the repository fallback visible when launching fails', async () => {
    systemMocks.detectSkillsManageApp.mockResolvedValue({
      supported: true,
      installed: true,
      platform: 'macos',
      source: 'bundle_id',
    })
    systemMocks.openSkillsManageApp.mockRejectedValue(new Error('launch failed'))

    const { default: SkillsMigrationView } = await import('@/views/SkillsMigrationView.vue')
    const { el, unmount } = await mountView(SkillsMigrationView)

    try {
      const primaryButton = el.querySelector<HTMLButtonElement>('[data-testid="skills-migration-primary"]')
      primaryButton?.click()
      await flush()

      expect(el.querySelector('[data-testid="skills-migration-error"]')?.textContent).toContain('已检测到 skills-manage，但拉起失败')

      const helperLink = el.querySelector<HTMLAnchorElement>('.skills-migration-view__helper-link')
      expect(helperLink?.getAttribute('href')).toBe('https://github.com/iamzhihuix/skills-manage')
    } finally {
      unmount()
    }
  })
})
