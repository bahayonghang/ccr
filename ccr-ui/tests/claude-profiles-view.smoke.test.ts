import { createApp, defineComponent, h, nextTick } from 'vue'
import { createI18n } from 'vue-i18n'
import { createPinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import zhCnMessages from '@/i18n/locales/zh-CN'

const apiMocks = vi.hoisted(() => ({
  listClaudeProfiles: vi.fn(),
  addClaudeProfile: vi.fn(),
  updateClaudeProfile: vi.fn(),
  deleteClaudeProfile: vi.fn(),
  applyClaudeProfile: vi.fn(),
  exportClaudeProfiles: vi.fn(),
}))

vi.mock('@/api', () => ({
  listClaudeProfiles: apiMocks.listClaudeProfiles,
  addClaudeProfile: apiMocks.addClaudeProfile,
  updateClaudeProfile: apiMocks.updateClaudeProfile,
  deleteClaudeProfile: apiMocks.deleteClaudeProfile,
  applyClaudeProfile: apiMocks.applyClaudeProfile,
  exportClaudeProfiles: apiMocks.exportClaudeProfiles,
}))

vi.mock('vue-router', () => ({
  useRoute: () => ({
    path: '/claude-code/profiles',
  }),
  RouterLink: defineComponent({
    props: {
      to: { type: [String, Object], required: true },
    },
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

vi.mock('@/components/PageHeaderCard.vue', () => ({
  default: defineComponent({
    setup(_props, { slots }) {
      return () =>
        h('section', { 'data-stub': 'PageHeaderCard' }, [
          h('div', { 'data-slot': 'meta' }, slots.meta?.()),
          h('div', { 'data-slot': 'actions' }, slots.actions?.()),
          h('div', { 'data-slot': 'default' }, slots.default?.()),
        ])
    },
  }),
}))

vi.mock('@/components/ui/Input.vue', () => ({
  default: defineComponent({
    props: {
      modelValue: { type: String, default: '' },
      placeholder: { type: String, default: '' },
    },
    emits: ['update:modelValue'],
    setup(props, { emit, slots }) {
      return () =>
        h('label', { 'data-stub': 'Input' }, [
          slots.leading?.(),
          h('input', {
            value: props.modelValue,
            placeholder: props.placeholder,
            onInput: (event: Event) => {
              emit('update:modelValue', (event.target as HTMLInputElement).value)
            },
          }),
        ])
    },
  }),
}))

vi.mock('@/components/common/BaseModal.vue', () => ({
  default: defineComponent({
    props: {
      modelValue: { type: Boolean, default: false },
    },
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

vi.mock('@/components/claude/ClaudeProfileEditorSections.vue', () => ({
  default: defineComponent({
    setup() {
      return () => h('div', { 'data-stub': 'ClaudeProfileEditorSections' })
    },
  }),
}))

const RouterLinkStub = defineComponent({
  props: {
    to: { type: [String, Object], required: true },
  },
  setup(_props, { slots }) {
    return () => h('a', {}, slots.default?.())
  },
})

const flushPromises = async () => {
  await Promise.resolve()
  await nextTick()
  await nextTick()
}

import ClaudeCodeProfilesView from '@/views/ClaudeCodeProfilesView.vue'
import type { ClaudeProfile } from '@/types'

const sampleProfiles: ClaudeProfile[] = [
  {
    name: 'zeta-current',
    provider: 'Zeta Relay',
    provider_type: 'official',
    description: 'Primary API relay for production traffic',
    base_url: 'https://relay.zeta.ai',
    model: 'claude-sonnet-4-5',
    small_fast_model: 'claude-3-5-haiku',
    account: 'github_5962',
    tags: ['prod', 'backup'],
    enabled: true,
    is_current: true,
  },
  {
    name: 'anthropic-a',
    provider: 'Anthropic',
    provider_type: 'api',
    description: 'Direct production account',
    base_url: 'https://api.anthropic.com',
    model: 'claude-opus-4-1',
    account: 'work-account',
    tags: ['production'],
    enabled: true,
    is_current: false,
  },
  {
    name: 'anthropic-disabled',
    provider: 'Anthropic',
    provider_type: 'api',
    description: 'Disabled fallback route',
    small_fast_model: 'claude-3-5-haiku',
    tags: ['disabled'],
    enabled: false,
    is_current: false,
  },
  {
    name: 'missing-provider',
    description: 'Temporary local sandbox',
    base_url: 'https://sandbox.internal',
    tags: ['local'],
    enabled: true,
    is_current: false,
  },
]

const cloneProfiles = (profiles: ClaudeProfile[] = sampleProfiles): ClaudeProfile[] =>
  profiles.map((profile) => ({
    ...profile,
    tags: profile.tags ? [...profile.tags] : undefined,
  }))

const mountView = async () => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(
    defineComponent({
      setup() {
        return () => h(ClaudeCodeProfilesView)
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
      messages: {
        'zh-CN': zhCnMessages,
      },
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

const findToolbarMeta = (el: HTMLElement) => el.querySelector<HTMLElement>('.cp-toolbar__meta')

const findButtonByText = (el: HTMLElement, text: string) =>
  Array.from(el.querySelectorAll<HTMLButtonElement>('button')).find((button) =>
    button.textContent?.includes(text)
  ) ?? null

const findProfileCards = (el: HTMLElement) =>
  Array.from(el.querySelectorAll<HTMLElement>('.cp-grid > article'))

const findProfileCardNames = (el: HTMLElement) =>
  findProfileCards(el)
    .map((card) => sampleProfiles.find((profile) => card.textContent?.includes(profile.name))?.name)
    .filter((name): name is string => Boolean(name))

const findProfileCard = (el: HTMLElement, name: string) =>
  findProfileCards(el).find((article) => article.textContent?.includes(name)) ?? null

const findApplyButton = (card: HTMLElement | null) =>
  Array.from(card?.querySelectorAll<HTMLButtonElement>('button') ?? []).find((button) =>
    button.textContent?.includes('应用此 Profile')
  ) ?? null

const findTriggerByIcon = (el: HTMLElement, icon: string) =>
  (el.querySelector(`[data-icon="${icon}"]`)?.closest('button') as HTMLButtonElement | null) ?? null

/** Reload / Export / Edit TOML 已收进页头 ··· 溢出菜单，需要先展开 */
const openOverflowMenu = async (el: HTMLElement) => {
  const trigger = findTriggerByIcon(el, 'MenuDots')
  expect(trigger).not.toBeNull()
  trigger!.click()
  await flushPromises()
}

/** 标签 / provider / 排序已收进工具条 Filters 弹层，需要先展开 */
const openFiltersPopover = async (el: HTMLElement) => {
  const trigger = findTriggerByIcon(el, 'SlidersHorizontal')
  expect(trigger).not.toBeNull()
  trigger!.click()
  await flushPromises()
}

beforeEach(() => {
  apiMocks.listClaudeProfiles.mockReset()
  apiMocks.addClaudeProfile.mockReset()
  apiMocks.updateClaudeProfile.mockReset()
  apiMocks.deleteClaudeProfile.mockReset()
  apiMocks.applyClaudeProfile.mockReset()
  apiMocks.exportClaudeProfiles.mockReset()

  apiMocks.listClaudeProfiles.mockResolvedValue({
    profiles: cloneProfiles(),
    current_profile: 'zeta-current',
  })
  apiMocks.exportClaudeProfiles.mockResolvedValue({
    content: '[profiles.zeta-current]\nauth_token = "secret"\n',
    filename: 'ccr-claude-profiles-test.toml',
  })

  Object.defineProperty(URL, 'createObjectURL', {
    configurable: true,
    value: vi.fn(() => 'blob:ccr-claude-profiles-test'),
  })
  Object.defineProperty(URL, 'revokeObjectURL', {
    configurable: true,
    value: vi.fn(),
  })
  vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => undefined)
})

afterEach(() => {
  document.body.innerHTML = ''
  vi.unstubAllGlobals()
})

describe('ClaudeCodeProfilesView smoke', () => {
  it('renders the redesigned stat strip, filters, and inspector insights', async () => {
    const { el, unmount } = await mountView()

    try {
      expect(el.textContent).toContain('Profiles')
      expect(el.textContent).toContain('当前 Profile')
      expect(el.textContent).toContain('zeta-current')
      expect(el.textContent).toContain('总数')
      expect(el.textContent).toContain('3 启用 · 1 停用')
      expect(el.textContent).toContain('认证分布')
      expect(el.textContent).toContain('订阅 4 · API Key 0')
      // 第四槽从 Last Write 客户端时钟换成 Health 计数
      expect(el.textContent).toContain('2 个配置问题')
      expect(el.textContent).not.toContain('最近写入')

      // provider 下拉已收进 Filters 弹层
      expect(el.textContent).not.toContain('全部 Provider')
      await openFiltersPopover(el)
      expect(el.textContent).toContain('全部 Provider')

      expect(el.textContent).toContain('分布洞察')
      expect(el.textContent).toContain('Anthropic')
      expect(el.textContent).toContain('Zeta Relay')
      expect(el.textContent).toContain('健康审计')
      expect(el.textContent).toContain('anthropic-disabled')
      expect(el.textContent).toContain('missing-provider')
      expect(el.textContent).toContain('缺少模型 · 缺少账号')
      expect(findToolbarMeta(el)?.textContent).toBe('4/4')
    } finally {
      unmount()
    }
  })

  it('filters the profile card set without stale untranslated labels', async () => {
    const { el, unmount } = await mountView()

    try {
      expect(el.textContent).toContain('未设置 Provider')
      expect(el.textContent).not.toContain('Unspecified Provider')
      expect(el.textContent).not.toContain('Other')
      expect(findToolbarMeta(el)?.textContent).toBe('4/4')

      expect(findProfileCardNames(el)).toEqual([
        'zeta-current',
        'anthropic-a',
        'missing-provider',
        'anthropic-disabled',
      ])

      const searchInput = findSearchInput(el)
      expect(searchInput).not.toBeNull()

      searchInput!.value = 'local'
      searchInput!.dispatchEvent(new Event('input', { bubbles: true }))
      await flushPromises()

      expect(findProfileCardNames(el)).toEqual(['missing-provider'])
      expect(findToolbarMeta(el)?.textContent).toBe('1/4')
      expect(el.textContent).not.toContain('{count}')
      expect(el.textContent).not.toContain('{enabled}')
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

      expect(el.textContent).toContain('没有匹配的 Claude Profile')
      expect(el.textContent).not.toContain('claudeProfiles.')
      expect(findProfileCards(el)).toHaveLength(0)
      expect(findToolbarMeta(el)?.textContent).toBe('0/4')
    } finally {
      unmount()
    }
  })

  it('normalizes current profile state across stat strip, context rail, and row cards', async () => {
    apiMocks.listClaudeProfiles.mockResolvedValue({
      profiles: cloneProfiles(),
      current_profile: 'anthropic-a',
    })

    const { el, unmount } = await mountView()

    try {
      expect(el.textContent).toContain('当前 Profile')
      expect(el.textContent).toContain('anthropic-a')

      const anthropicCard = findProfileCard(el, 'anthropic-a')
      const zetaCard = findProfileCard(el, 'zeta-current')
      const anthropicApplyButton = findApplyButton(anthropicCard)
      const zetaApplyButton = findApplyButton(zetaCard)

      expect(anthropicCard?.textContent).toContain('当前已激活')
      expect(zetaCard?.textContent).not.toContain('当前已激活')
      expect(anthropicApplyButton).toBeNull()
      expect(zetaApplyButton).not.toBeNull()
      expect(zetaApplyButton?.disabled).toBe(false)
      expect(el.textContent).not.toContain('claudeProfiles.')
    } finally {
      unmount()
    }
  })

  it('prevents disabled profiles from applying through row actions', async () => {
    const { el, unmount } = await mountView()

    try {
      const disabledCard = findProfileCard(el, 'anthropic-disabled')
      const disabledRowApply = findApplyButton(disabledCard)

      expect(disabledCard).not.toBeNull()
      expect(disabledRowApply?.disabled).toBe(true)

      disabledRowApply?.click()
      await flushPromises()

      expect(apiMocks.applyClaudeProfile).not.toHaveBeenCalled()
    } finally {
      unmount()
    }
  })

  it('refreshes the list without clearing the current search context', async () => {
    const { el, unmount } = await mountView()

    try {
      const searchInput = findSearchInput(el)
      await openOverflowMenu(el)
      const refreshButton = findButtonByText(el, '重载')

      expect(searchInput).not.toBeNull()
      expect(refreshButton).not.toBeNull()

      searchInput!.value = 'local'
      searchInput!.dispatchEvent(new Event('input', { bubbles: true }))
      await flushPromises()

      apiMocks.listClaudeProfiles.mockResolvedValueOnce({
        profiles: cloneProfiles([
          {
            ...sampleProfiles[0],
            name: 'zeta-current-v2',
          },
          {
            name: 'missing-provider-refreshed',
            description: 'Refreshed local sandbox',
            base_url: 'https://sandbox.internal',
            tags: ['local'],
            enabled: true,
            is_current: false,
          },
        ]),
        current_profile: 'zeta-current-v2',
      })

      refreshButton!.click()
      await flushPromises()

      expect(apiMocks.listClaudeProfiles).toHaveBeenCalledTimes(2)
      expect(searchInput!.value).toBe('local')
      expect(findProfileCards(el).map((card) => card.textContent)).toEqual([
        expect.stringContaining('missing-provider-refreshed'),
      ])
      expect(
        findProfileCards(el)
          .map((card) => card.textContent)
          .join('')
      ).not.toContain('Temporary local sandbox')
    } finally {
      unmount()
    }
  })

  it('exports the full Claude profiles TOML', async () => {
    const { el, unmount } = await mountView()

    try {
      await openOverflowMenu(el)
      const exportButton = findTriggerByIcon(el, 'Download')

      expect(exportButton).not.toBeNull()

      exportButton!.click()
      await flushPromises()

      expect(apiMocks.exportClaudeProfiles).toHaveBeenCalledWith(true)
      expect(URL.createObjectURL).toHaveBeenCalled()
      expect(HTMLAnchorElement.prototype.click).toHaveBeenCalled()
      expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:ccr-claude-profiles-test')
    } finally {
      unmount()
    }
  })

  it('keeps the existing list visible when a manual refresh fails', async () => {
    const { el, unmount } = await mountView()

    try {
      await openOverflowMenu(el)
      const refreshButton = findButtonByText(el, '重载')

      expect(refreshButton).not.toBeNull()
      expect(el.textContent).toContain('zeta-current')

      apiMocks.listClaudeProfiles.mockRejectedValueOnce(new Error('refresh exploded'))

      refreshButton!.click()
      await flushPromises()

      expect(apiMocks.listClaudeProfiles).toHaveBeenCalledTimes(2)
      expect(el.textContent).toContain('刷新 Claude Profiles 失败')
      expect(el.textContent).toContain('refresh exploded')
      expect(el.textContent).toContain('当前列表已保留')
      expect(el.textContent).toContain('zeta-current')
      expect(el.textContent).not.toContain('加载 Claude Profiles 失败')
    } finally {
      unmount()
    }
  })

  it('opens a confirm dialog with interpolated copy and only applies after confirmation', async () => {
    const { el, unmount } = await mountView()

    try {
      const targetCard = findProfileCard(el, 'anthropic-a')
      const applyButton = findApplyButton(targetCard)

      expect(applyButton).not.toBeNull()

      applyButton?.click()
      await flushPromises()

      const dialog = el.querySelector<HTMLElement>('[data-stub="BaseModal"]')
      expect(dialog).not.toBeNull()
      expect(dialog?.textContent).toContain(
        '确定要应用 Profile "anthropic-a" 吗？这将同步更新当前 Claude 配置。'
      )
      // 「当前 → 目标」三行 diff：base_url / model / auth_mode
      const diffRows = Array.from(dialog!.querySelectorAll<HTMLElement>('.cp-diff-row'))
      expect(diffRows).toHaveLength(3)
      expect(diffRows[0].textContent).toContain('https://relay.zeta.ai')
      expect(diffRows[0].textContent).toContain('https://api.anthropic.com')
      expect(diffRows[1].textContent).toContain('claude-opus-4-1')
      expect(apiMocks.applyClaudeProfile).not.toHaveBeenCalled()

      const footer = dialog!.querySelector<HTMLElement>('[data-slot="footer"]')
      const confirmButton = Array.from(
        footer?.querySelectorAll<HTMLButtonElement>('button') ?? []
      ).find((button) => button.textContent?.includes('应用此 Profile'))

      expect(confirmButton).not.toBeUndefined()

      confirmButton?.click()
      await flushPromises()

      expect(apiMocks.applyClaudeProfile).toHaveBeenCalledWith('anthropic-a')
    } finally {
      unmount()
    }
  })

  it('states the real backup location in the delete confirm dialog', async () => {
    const { el, unmount } = await mountView()

    try {
      const targetCard = findProfileCard(el, 'anthropic-a')
      const menuTrigger = Array.from(
        targetCard?.querySelectorAll<HTMLButtonElement>('button') ?? []
      ).find((button) => button.getAttribute('aria-haspopup') === 'menu')

      expect(menuTrigger).not.toBeUndefined()
      menuTrigger!.click()
      await flushPromises()

      const deleteItem = Array.from(
        targetCard?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]') ?? []
      ).find((button) => button.textContent?.includes('删除'))

      expect(deleteItem).not.toBeUndefined()
      deleteItem!.click()
      await flushPromises()

      const dialog = el.querySelector<HTMLElement>('[data-stub="BaseModal"]')
      expect(dialog?.textContent).toContain('确定要删除 Profile "anthropic-a" 吗？')
      expect(dialog?.textContent).toContain('~/.ccr/backups/claude/')
      // 删除确认框不带 apply 的 diff 行
      expect(dialog!.querySelectorAll('.cp-diff-row')).toHaveLength(0)
      expect(apiMocks.deleteClaudeProfile).not.toHaveBeenCalled()
    } finally {
      unmount()
    }
  })
})
