import { createApp, defineComponent, h, nextTick } from 'vue'
import { createI18n } from 'vue-i18n'
import { createPinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import zhCnMessages from '@/i18n/locales/zh-CN'

const invokeMock = vi.hoisted(() => vi.fn())

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

const apiMocks = vi.hoisted(() => ({
  listCodexProfiles: vi.fn(),
  listCodexModels: vi.fn(),
  getCodexProfile: vi.fn(),
  addCodexProfile: vi.fn(),
  updateCodexProfile: vi.fn(),
  deleteCodexProfile: vi.fn(),
  applyCodexProfile: vi.fn(),
  exportCodexProfiles: vi.fn(),
  getCurrentEnvironment: vi.fn(),
  codexProfileOff: vi.fn(),
}))

vi.mock('@/api', () => ({ ...apiMocks }))

vi.mock('@/api/domains/codex', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api/domains/codex')>()
  return {
    ...actual,
    codexProfileOff: apiMocks.codexProfileOff,
  }
})

vi.mock('vue-router', () => ({
  useRoute: () => ({ path: '/codex/profiles' }),
  RouterLink: defineComponent({
    props: { to: { type: [String, Object], required: true } },
    setup(_props, { slots }) {
      return () => h('a', {}, slots.default?.())
    },
  }),
}))

vi.mock('@/components/ui/SIcon.vue', () => ({
  default: defineComponent({
    props: {
      name: { type: String, required: true },
      size: { type: String, default: '' },
      class: { type: String, default: '' },
    },
    setup(props) {
      return () => h('span', { 'data-icon': props.name, class: [props.size, props.class] })
    },
  }),
}))

vi.mock('@/components/common/BaseModal.vue', () => ({
  default: defineComponent({
    props: { modelValue: { type: Boolean, default: false } },
    setup(props, { slots }) {
      return () =>
        props.modelValue
          ? h('div', { 'data-stub': 'BaseModal' }, [
              h('div', { 'data-slot': 'header' }, slots.header?.({ titleId: 'modal-title' })),
              slots.default?.(),
              h('div', { 'data-slot': 'footer' }, slots.footer?.()),
            ])
          : null
    },
  }),
}))

vi.mock('@/components/codex/CodexProfileEditorModal.vue', () => ({
  default: defineComponent({
    props: { modelValue: { type: Boolean, default: false } },
    setup(props) {
      return () =>
        props.modelValue ? h('div', { 'data-testid': 'codex-profile-editor-modal' }) : null
    },
  }),
}))

import { updateCodexProfile } from '@/api/domains/codex'
import CodexProfilesView from '@/views/CodexProfilesView.vue'
import type { CodexProfile } from '@/types'

const RouterLinkStub = defineComponent({
  props: { to: { type: [String, Object], required: true } },
  setup(_props, { slots }) {
    return () => h('a', {}, slots.default?.())
  },
})

const flushPromises = async () => {
  await Promise.resolve()
  await nextTick()
  await nextTick()
}

const sampleProfiles: CodexProfile[] = [
  {
    name: 'relay-current',
    description: 'Primary relay for production traffic',
    base_url: 'https://relay.codex.ai/v1',
    model: 'gpt-5.6-luna',
    auth_mode: 'openai_api_key',
    provider: 'Relay',
    tags: ['prod'],
    enabled: true,
  },
  {
    name: 'official-direct',
    description: 'Official OpenAI runtime',
    model: 'gpt-5.6-terra',
    auth_mode: 'openai_chatgpt',
    tags: ['official'],
    enabled: true,
  },
  {
    name: 'legacy-env-key',
    description: 'Legacy env-key relay',
    base_url: 'https://legacy.codex.ai/v1',
    auth_mode: 'provider_env_key',
    env_key: 'LEGACY_API_KEY',
    tags: ['legacy'],
    enabled: false,
  },
]

const cloneProfiles = (profiles: CodexProfile[] = sampleProfiles): CodexProfile[] =>
  profiles.map((profile) => ({ ...profile, tags: profile.tags ? [...profile.tags] : undefined }))

const mountView = async () => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(
    defineComponent({
      setup() {
        return () => h(CodexProfilesView)
      },
    })
  )

  app.use(
    createI18n({
      legacy: false,
      locale: 'zh-CN',
      fallbackLocale: 'zh-CN',
      missingWarn: false,
      fallbackWarn: false,
      messages: { 'zh-CN': zhCnMessages },
    })
  )
  app.use(createPinia())
  app.component('RouterLink', RouterLinkStub)

  app.mount(el)
  await flushPromises()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

const findSearchInput = (el: HTMLElement) => el.querySelector<HTMLInputElement>('.cp-search__input')

const findProfileCards = (el: HTMLElement) =>
  Array.from(el.querySelectorAll<HTMLElement>('.cp-grid > article'))

const findProfileCard = (el: HTMLElement, name: string) =>
  findProfileCards(el).find((article) => article.getAttribute('data-profile-name') === name) ?? null

const findApplyButton = (card: HTMLElement | null) =>
  card?.querySelector<HTMLButtonElement>('.cp-card__apply') ?? null

const findTriggerByIcon = (el: HTMLElement, icon: string) =>
  Array.from(el.querySelectorAll(`[data-icon="${icon}"]`))
    .map((node) => node.closest('button') as HTMLButtonElement | null)
    .find((button): button is HTMLButtonElement => Boolean(button)) ?? null

/** Reload / Export / Edit TOML 已收进页头 ··· 溢出菜单，需要先展开 */
const openOverflowMenu = async (el: HTMLElement) => {
  const trigger = findTriggerByIcon(el, 'MenuDots')
  expect(trigger).not.toBeNull()
  trigger!.click()
  await flushPromises()
}

/** 标签 / 排序已收进工具条 Filters 弹层，需要先展开 */
const openFiltersPopover = async (el: HTMLElement) => {
  const trigger = findTriggerByIcon(el, 'SlidersHorizontal')
  expect(trigger).not.toBeNull()
  trigger!.click()
  await flushPromises()
}

describe('codex profile update API', () => {
  it('preserves the original profile key while sending the renamed target name in config', async () => {
    invokeMock.mockResolvedValue({
      name: 'ice-renamed',
      message: "Codex Profile 'ice-renamed' 已更新",
    })

    await updateCodexProfile('ice', {
      name: 'ice-renamed',
      model: 'gpt-5.4',
      auth_mode: 'provider_env_key',
    })

    expect(invokeMock).toHaveBeenCalledWith('codex_update_profile', {
      name: 'ice',
      confirmationToken: 'desktop-confirm:codex_update_profile',
      config: expect.objectContaining({
        name: 'ice-renamed',
        model: 'gpt-5.4',
        auth_mode: 'provider_env_key',
      }),
    })
  })
})

describe('CodexProfilesView smoke', () => {
  beforeEach(() => {
    localStorage.clear()
    for (const mock of Object.values(apiMocks)) mock.mockReset()

    apiMocks.listCodexProfiles.mockResolvedValue({
      profiles: cloneProfiles(),
      current_profile: 'relay-current',
      can_off: true,
    })
    apiMocks.codexProfileOff.mockResolvedValue({
      ok: true,
      changed: true,
      previous_profile: 'relay-current',
      runtime_mode: 'official_auth',
    })
    apiMocks.listCodexModels.mockResolvedValue({ builtin_models: ['gpt-5.6-luna'] })
    apiMocks.exportCodexProfiles.mockResolvedValue({
      content: '[profiles.relay-current]\nauth_token = "secret"\n',
      filename: 'ccr-codex-profiles-test.toml',
    })
    apiMocks.getCurrentEnvironment.mockResolvedValue({ env_type: 'local' })

    Object.defineProperty(URL, 'createObjectURL', {
      configurable: true,
      value: vi.fn(() => 'blob:ccr-codex-profiles-test'),
    })
    Object.defineProperty(URL, 'revokeObjectURL', { configurable: true, value: vi.fn() })
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => undefined)
  })

  afterEach(() => {
    document.body.innerHTML = ''
    vi.restoreAllMocks()
  })

  it('shows the login-prep banner when the backend reports can_off', async () => {
    const { el, unmount } = await mountView()

    try {
      const banner = el.querySelector('[data-testid="codex-profile-off-banner"]')
      expect(banner).not.toBeNull()
      expect(banner?.textContent).toContain('退出 Profile')
    } finally {
      unmount()
    }
  })

  it('hides the login-prep banner when the backend reports can_off is false', async () => {
    apiMocks.listCodexProfiles.mockResolvedValue({
      profiles: cloneProfiles(),
      current_profile: 'relay-current',
      can_off: false,
    })
    const { el, unmount } = await mountView()

    try {
      expect(el.querySelector('[data-testid="codex-profile-off-banner"]')).toBeNull()
    } finally {
      unmount()
    }
  })

  it('does not write when the exit-profile confirmation is cancelled', async () => {
    const { el, unmount } = await mountView()

    try {
      const offButton = Array.from(el.querySelectorAll<HTMLButtonElement>('button')).find(button =>
        button.textContent?.includes('退出 Profile'),
      )
      offButton?.click()
      await flushPromises()
      expect(el.querySelector('[data-stub="BaseModal"]')).not.toBeNull()

      const cancel = el.querySelector<HTMLButtonElement>('.confirm-modal__button--cancel')
      expect(cancel).not.toBeNull()
      cancel?.click()
      await flushPromises()

      expect(apiMocks.codexProfileOff).not.toHaveBeenCalled()
    } finally {
      unmount()
    }
  })

  it('renders the four-slot stat strip with the config-mode specialty slot and health count', async () => {
    const { el, unmount } = await mountView()

    try {
      expect(el.textContent).toContain('当前配置')
      expect(el.textContent).toContain('relay-current')
      expect(el.textContent).toContain('配置总数')
      expect(el.textContent).toContain('2 启用 · 1 禁用')
      expect(el.textContent).toContain('配置模式')
      expect(el.textContent).toContain('自定义中转站')
      expect(el.textContent).toContain('官方 1 · 自定义中转 2')
      // 第四槽是 Health 计数，旧的 Last Write 客户端时钟已移除
      expect(el.textContent).toContain('个配置问题')
      expect(el.textContent).not.toContain('最近写入')
      expect(el.textContent).not.toContain('codex.profiles.')
    } finally {
      unmount()
    }
  })

  it('keeps sorting behind the Filters popover instead of the bare toolbar', async () => {
    const { el, unmount } = await mountView()

    try {
      expect(el.textContent).not.toContain('最近使用')
      await openFiltersPopover(el)
      expect(el.textContent).toContain('最近使用')
    } finally {
      unmount()
    }
  })

  it('renders the inspector preview and health audit instead of the legacy context rail', async () => {
    const { el, unmount } = await mountView()

    try {
      expect(el.textContent).toContain('Profile 预览')
      expect(el.textContent).toContain('分布洞察')
      expect(el.textContent).toContain('健康审计')
      // 弃用 auth 模式与缺失字段都进入审计列表
      expect(el.textContent).toContain('legacy-env-key')
      expect(el.textContent).toContain('缺少 Model')
    } finally {
      unmount()
    }
  })

  it('shows the current → target diff in the apply confirm dialog and only applies after confirming', async () => {
    const { el, unmount } = await mountView()

    try {
      const applyButton = findApplyButton(findProfileCard(el, 'official-direct'))
      expect(applyButton).not.toBeNull()

      applyButton!.click()
      await flushPromises()

      const dialog = el.querySelector<HTMLElement>('[data-stub="BaseModal"]')
      expect(dialog?.textContent).toContain('确定切换到 Profile "official-direct" 吗？')

      const diffRows = Array.from(dialog!.querySelectorAll<HTMLElement>('.cp-diff-row'))
      expect(diffRows).toHaveLength(3)
      expect(diffRows[0].textContent).toContain('https://relay.codex.ai/v1')
      expect(diffRows[0].textContent).toContain('官方 OpenAI 运行时')
      expect(diffRows[1].textContent).toContain('gpt-5.6-terra')
      expect(apiMocks.applyCodexProfile).not.toHaveBeenCalled()

      const confirmButton = Array.from(
        dialog!
          .querySelector<HTMLElement>('[data-slot="footer"]')
          ?.querySelectorAll<HTMLButtonElement>('button') ?? []
      ).find((button) => button.textContent?.includes('应用'))

      expect(confirmButton).not.toBeUndefined()
      confirmButton!.click()
      await flushPromises()

      expect(apiMocks.applyCodexProfile).toHaveBeenCalledWith('official-direct')
    } finally {
      unmount()
    }
  })

  it('states the real backup location in the delete confirm dialog', async () => {
    const { el, unmount } = await mountView()

    try {
      const card = findProfileCard(el, 'official-direct')
      const menuTrigger = Array.from(card?.querySelectorAll<HTMLButtonElement>('button') ?? []).find(
        (button) => button.getAttribute('aria-haspopup') === 'menu'
      )

      expect(menuTrigger).not.toBeUndefined()
      menuTrigger!.click()
      await flushPromises()

      const deleteItem = Array.from(
        card?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]') ?? []
      ).find((button) => button.textContent?.includes('删除'))

      expect(deleteItem).not.toBeUndefined()
      deleteItem!.click()
      await flushPromises()

      const dialog = el.querySelector<HTMLElement>('[data-stub="BaseModal"]')
      expect(dialog?.textContent).toContain('确定删除 Profile "official-direct" 吗？')
      expect(dialog?.textContent).toContain('~/.ccr/backups/codex/')
      // 删除确认框不带 apply 的 diff 行
      expect(dialog!.querySelectorAll('.cp-diff-row')).toHaveLength(0)
      expect(apiMocks.deleteCodexProfile).not.toHaveBeenCalled()
    } finally {
      unmount()
    }
  })

  it('keeps the existing list visible when a manual refresh fails', async () => {
    const { el, unmount } = await mountView()

    try {
      await openOverflowMenu(el)
      const refreshButton = Array.from(el.querySelectorAll<HTMLButtonElement>('button')).find(
        (button) => button.textContent?.includes('重载')
      )
      expect(refreshButton).not.toBeUndefined()

      apiMocks.listCodexProfiles.mockRejectedValueOnce(new Error('refresh exploded'))

      refreshButton!.click()
      await flushPromises()

      expect(el.textContent).toContain('刷新 Codex Profiles 失败')
      expect(el.textContent).toContain('refresh exploded')
      expect(el.textContent).toContain('当前列表已保留')
      expect(el.textContent).not.toContain('加载 Codex Profiles 失败')
    } finally {
      unmount()
    }
  })

  it('shows the search empty state without leaving stale profile cards behind', async () => {
    const { el, unmount } = await mountView()

    try {
      const searchInput = findSearchInput(el)
      expect(searchInput).not.toBeNull()

      searchInput!.value = 'no-such-profile'
      searchInput!.dispatchEvent(new Event('input', { bubbles: true }))
      await flushPromises()

      expect(findProfileCards(el)).toHaveLength(0)
      expect(el.textContent).toContain('没有匹配 "no-such-profile" 的 Profile')
      expect(el.textContent).not.toContain('codex.profiles.')
    } finally {
      unmount()
    }
  })

  it('exports the full Codex profiles TOML from the overflow menu', async () => {
    const { el, unmount } = await mountView()

    try {
      await openOverflowMenu(el)
      const exportButton = findTriggerByIcon(el, 'Download')
      expect(exportButton).not.toBeNull()

      exportButton!.click()
      await flushPromises()

      expect(apiMocks.exportCodexProfiles).toHaveBeenCalledWith(true)
      expect(URL.createObjectURL).toHaveBeenCalled()
      expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:ccr-codex-profiles-test')
    } finally {
      unmount()
    }
  })
})
