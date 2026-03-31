import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

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

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => key,
  }),
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

const sampleProfiles = [
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
    name: 'missing-provider',
    description: 'Temporary local sandbox',
    base_url: 'https://sandbox.internal',
    tags: ['local'],
    enabled: true,
    is_current: false,
  },
]

const mountView = async () => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () => h(ClaudeCodeProfilesView)
    },
  }))

  app.config.globalProperties.$t = (key: string) => key

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

beforeEach(() => {
  apiMocks.listClaudeProfiles.mockReset()
  apiMocks.addClaudeProfile.mockReset()
  apiMocks.updateClaudeProfile.mockReset()
  apiMocks.deleteClaudeProfile.mockReset()
  apiMocks.applyClaudeProfile.mockReset()

  apiMocks.listClaudeProfiles.mockResolvedValue({
    profiles: sampleProfiles,
    current_profile: 'zeta-current',
  })
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('ClaudeCodeProfilesView smoke', () => {
  it('limits quick-switch actions to the filtered profile set', async () => {
    const { el, unmount } = await mountView()

    try {
      expect(findQuickSwitchButtons(el).map(button => button.textContent?.trim())).toEqual([
        'zeta-current',
        'anthropic-a',
        'missing-provider',
      ])

      const searchInput = el.querySelector<HTMLInputElement>('input[placeholder="claudeProfiles.searchPlaceholder"]')
      expect(searchInput).not.toBeNull()

      searchInput!.value = 'local'
      searchInput!.dispatchEvent(new Event('input', { bubbles: true }))
      await flushPromises()

      expect(findQuickSwitchButtons(el).map(button => button.textContent?.trim())).toEqual([
        'missing-provider',
      ])
    } finally {
      unmount()
    }
  })

  it('shows the search empty state without leaving stale quick-switch actions behind', async () => {
    const { el, unmount } = await mountView()

    try {
      const searchInput = el.querySelector<HTMLInputElement>('input[placeholder="claudeProfiles.searchPlaceholder"]')
      expect(searchInput).not.toBeNull()

      searchInput!.value = 'no-such-profile'
      searchInput!.dispatchEvent(new Event('input', { bubbles: true }))
      await flushPromises()

      expect(el.textContent).toContain('claudeProfiles.searchEmptyTitle')
      expect(findQuickSwitchButtons(el)).toHaveLength(0)
    } finally {
      unmount()
    }
  })
})
