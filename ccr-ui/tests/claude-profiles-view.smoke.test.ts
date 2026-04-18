import { createApp, defineComponent, h, nextTick } from 'vue'
import { createI18n } from 'vue-i18n'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import zhCnMessages from '@/i18n/locales/zh-CN'

const apiMocks = vi.hoisted(() => ({
  listClaudeProfiles: vi.fn(),
  addClaudeProfile: vi.fn(),
  updateClaudeProfile: vi.fn(),
  deleteClaudeProfile: vi.fn(),
  applyClaudeProfile: vi.fn(),
}))

vi.mock('@/api', () => ({
  listClaudeProfiles: apiMocks.listClaudeProfiles,
  addClaudeProfile: apiMocks.addClaudeProfile,
  updateClaudeProfile: apiMocks.updateClaudeProfile,
  deleteClaudeProfile: apiMocks.deleteClaudeProfile,
  applyClaudeProfile: apiMocks.applyClaudeProfile,
}))

vi.mock('vue-router', () => ({
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
      return () => h('section', { 'data-stub': 'PageHeaderCard' }, [
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
      return () => h('label', { 'data-stub': 'Input' }, [
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
      return () => props.modelValue
        ? h('div', { 'data-stub': 'BaseModal' }, [
            h('div', { 'data-slot': 'header' }, slots.header?.({ titleId: 'modal-title' })),
            slots.default?.(),
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
  profiles.map(profile => ({
    ...profile,
    tags: profile.tags ? [...profile.tags] : undefined,
  }))

const mountView = async () => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () => h(ClaudeCodeProfilesView)
    },
  }))

  app.use(createI18n({
    legacy: false,
    locale: 'zh-CN',
    fallbackLocale: 'zh-CN',
    missingWarn: false,
    fallbackWarn: false,
    messages: {
      'zh-CN': zhCnMessages,
    },
  }))

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

const findQuickSwitchButtons = (el: HTMLElement) =>
  Array.from(el.querySelectorAll<HTMLButtonElement>('button')).filter((button) =>
    sampleProfiles.some((profile) => button.textContent?.includes(profile.name)),
  )

const findQuickSwitchButton = (el: HTMLElement, name: string) =>
  findQuickSwitchButtons(el).find(button => button.textContent?.includes(name)) ?? null

const findProfileCard = (el: HTMLElement, name: string) =>
  Array.from(el.querySelectorAll<HTMLElement>('article')).find((article) => article.textContent?.includes(name)) ?? null

beforeEach(() => {
  apiMocks.listClaudeProfiles.mockReset()
  apiMocks.addClaudeProfile.mockReset()
  apiMocks.updateClaudeProfile.mockReset()
  apiMocks.deleteClaudeProfile.mockReset()
  apiMocks.applyClaudeProfile.mockReset()

  apiMocks.listClaudeProfiles.mockResolvedValue({
    profiles: cloneProfiles(),
    current_profile: 'zeta-current',
  })

  vi.stubGlobal('confirm', vi.fn(() => true))
  vi.stubGlobal('alert', vi.fn())
})

afterEach(() => {
  document.body.innerHTML = ''
  vi.unstubAllGlobals()
})

describe('ClaudeCodeProfilesView smoke', () => {
  it('renders overview statistics for profile density and config coverage', async () => {
    const { el, unmount } = await mountView()

    try {
      expect(el.textContent).toContain('Profiles')
      expect(el.textContent).toContain('3 已启用 · 1 已停用')
      expect(el.textContent).toContain('3 个分组 · 1 未设置 Provider')
      expect(el.textContent).toContain('2 主模型 · 2 快速模型')
      expect(el.textContent).toContain('4 订阅 · 0 API Key · 2 账号')
      expect(el.textContent).toContain('自定义 Endpoint')
      expect(el.textContent).toContain('带标签')
      expect(el.textContent).toContain('缺少主模型')
      expect(el.textContent).toContain('缺少账号')
    } finally {
      unmount()
    }
  })

  it('limits quick-switch actions to the filtered profile set', async () => {
    const { el, unmount } = await mountView()

    try {
      expect(el.textContent).toContain('未设置 Provider')
      expect(el.textContent).not.toContain('Unspecified Provider')
      expect(el.textContent).not.toContain('Other')
      expect(el.textContent).toContain('4 个候选')

      expect(findQuickSwitchButtons(el).map(button => button.textContent?.trim())).toEqual([
        'zeta-current',
        'anthropic-a',
        'anthropic-disabled',
        'missing-provider',
      ])

      const searchInput = el.querySelector<HTMLInputElement>('input[placeholder="搜索名称 / provider / model / tag"]')
      expect(searchInput).not.toBeNull()

      searchInput!.value = 'local'
      searchInput!.dispatchEvent(new Event('input', { bubbles: true }))
      await flushPromises()

      expect(findQuickSwitchButtons(el).map(button => button.textContent?.trim())).toEqual([
        'missing-provider',
      ])
      expect(el.textContent).toContain('1 个候选')
      expect(el.textContent).not.toContain('{count}')
      expect(el.textContent).not.toContain('{enabled}')
    } finally {
      unmount()
    }
  })

  it('shows the search empty state without leaving stale quick-switch actions behind', async () => {
    const { el, unmount } = await mountView()

    try {
      const searchInput = el.querySelector<HTMLInputElement>('input[placeholder="搜索名称 / provider / model / tag"]')
      expect(searchInput).not.toBeNull()

      searchInput!.value = 'no-such-profile'
      searchInput!.dispatchEvent(new Event('input', { bubbles: true }))
      await flushPromises()

      expect(el.textContent).toContain('没有匹配的 Claude Profile')
      expect(el.textContent).not.toContain('claudeProfiles.')
      expect(findQuickSwitchButtons(el)).toHaveLength(0)
    } finally {
      unmount()
    }
  })

  it('normalizes current profile state across overview, provider nav, quick switch, and row cards', async () => {
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
      const anthropicQuickSwitch = findQuickSwitchButton(el, 'anthropic-a')
      const zetaQuickSwitch = findQuickSwitchButton(el, 'zeta-current')
      const anthropicProviderButtons = Array.from(el.querySelectorAll<HTMLButtonElement>('nav button'))
        .filter(button => button.textContent?.includes('Anthropic'))

      expect(anthropicCard?.textContent).toContain('当前已激活')
      expect(zetaCard?.textContent).not.toContain('当前已激活')
      expect(anthropicQuickSwitch?.disabled).toBe(true)
      expect(zetaQuickSwitch?.disabled).toBe(false)
      expect(anthropicProviderButtons.some(button => button.querySelector('[aria-label="当前 Provider"]'))).toBe(true)
      expect(el.textContent).not.toContain('claudeProfiles.')
    } finally {
      unmount()
    }
  })

  it('prevents disabled profiles from applying through quick switch or row actions', async () => {
    const { el, unmount } = await mountView()

    try {
      const disabledQuickSwitch = findQuickSwitchButton(el, 'anthropic-disabled')
      const disabledCard = findProfileCard(el, 'anthropic-disabled')
      const disabledRowApply = Array.from(disabledCard?.querySelectorAll<HTMLButtonElement>('button') ?? [])
        .find(button => button.textContent?.includes('应用此 Profile'))

      expect(disabledQuickSwitch).not.toBeNull()
      expect(disabledQuickSwitch?.disabled).toBe(true)
      expect(disabledRowApply?.disabled).toBe(true)

      disabledQuickSwitch?.click()
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
      const searchInput = el.querySelector<HTMLInputElement>('input[placeholder="搜索名称 / provider / model / tag"]')
      const refreshButton = Array.from(el.querySelectorAll<HTMLButtonElement>('button'))
        .find(button => button.textContent?.includes('刷新'))

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
      expect(el.textContent).toContain('missing-provider-refreshed')
      expect(el.textContent).not.toContain('Temporary local sandbox')
    } finally {
      unmount()
    }
  })

  it('keeps the existing list visible when a manual refresh fails', async () => {
    const { el, unmount } = await mountView()

    try {
      const refreshButton = Array.from(el.querySelectorAll<HTMLButtonElement>('button'))
        .find(button => button.textContent?.includes('刷新'))

      expect(refreshButton).not.toBeNull()
      expect(el.textContent).toContain('zeta-current')

      apiMocks.listClaudeProfiles.mockRejectedValueOnce(new Error('refresh exploded'))

      refreshButton!.click()
      await flushPromises()

      expect(apiMocks.listClaudeProfiles).toHaveBeenCalledTimes(2)
      expect(el.textContent).toContain('刷新 Claude Profiles 失败')
      expect(el.textContent).toContain('refresh exploded')
      expect(el.textContent).toContain('zeta-current')
      expect(el.textContent).not.toContain('加载 Claude Profiles 失败')
    } finally {
      unmount()
    }
  })

  it('renders interpolated confirmation copy when applying a profile', async () => {
    const { el, unmount } = await mountView()

    try {
      const targetCard = findProfileCard(el, 'anthropic-a')
      const applyButton = Array.from(targetCard?.querySelectorAll<HTMLButtonElement>('button') ?? [])
        .find(button => button.textContent?.includes('应用此 Profile'))

      expect(applyButton).not.toBeNull()

      applyButton?.click()
      await flushPromises()

      expect(globalThis.confirm).toHaveBeenCalledWith(
        '确定要应用 Profile "anthropic-a" 吗？这将同步更新当前 Claude 配置。'
      )
      expect(apiMocks.applyClaudeProfile).toHaveBeenCalledWith('anthropic-a')
    } finally {
      unmount()
    }
  })
})
