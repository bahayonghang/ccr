import { createApp, defineComponent, h, nextTick } from 'vue'
import { createPinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { GrokProfileDto } from '@/types'

const apiMocks = vi.hoisted(() => ({
  listGrokProfiles: vi.fn(),
  addGrokProfile: vi.fn(),
  updateGrokProfile: vi.fn(),
  deleteGrokProfile: vi.fn(),
  applyGrokProfile: vi.fn(),
  grokProfileOff: vi.fn(),
}))
const getCurrentEnvironmentMock = vi.hoisted(() => vi.fn())

vi.mock('@/api', () => ({ grokApi: apiMocks }))
vi.mock('@/api/runtime/environment', () => ({
  getCurrentEnvironment: (...args: unknown[]) => getCurrentEnvironmentMock(...args),
}))
vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, params?: Record<string, unknown>) => (
      params ? `${key}:${JSON.stringify(params)}` : key
    ),
  }),
}))
vi.mock('@/utils/windowChrome', () => ({ getClientPlatform: () => 'windows' }))

vi.mock('@/components/ui/SIcon.vue', () => ({
  default: defineComponent({
    props: { name: { type: String, required: true } },
    setup: props => () => h('span', { 'data-icon': props.name }),
  }),
}))
vi.mock('@/components/ModuleSubnav.vue', () => ({ default: defineComponent(() => () => h('nav')) }))
vi.mock('@/components/profiles/ProfilesHeader.vue', () => ({
  default: defineComponent({
    props: { loading: { type: Boolean, default: false } },
    emits: ['add', 'reload'],
    setup: (props, { emit }) => () => h('header', { 'data-testid': 'profiles-header', 'data-loading': String(props.loading) }, [
      h('button', { 'data-testid': 'add', onClick: () => emit('add') }, 'add'),
      h('button', { 'data-testid': 'reload', onClick: () => emit('reload') }, 'reload'),
    ]),
  }),
}))
vi.mock('@/components/profiles/ProfilesStatStrip.vue', () => ({
  default: defineComponent({
    props: { health: { type: Object, required: true } },
    setup: props => () => h('div', {
      'data-testid': 'stat-health',
      'data-value': (props.health as { value: string }).value,
      'data-warn': String((props.health as { warn: boolean }).warn),
    }),
  }),
}))
vi.mock('@/components/profiles/ProfilesQuickRail.vue', () => ({ default: defineComponent(() => () => h('div')) }))
vi.mock('@/components/profiles/ProfilesToolbar.vue', () => ({
  default: defineComponent({
    setup(_props, { expose }) {
      expose({ focusSearch: vi.fn() })
      return () => h('div')
    },
  }),
}))
vi.mock('@/components/profiles/ProfilesSection.vue', () => ({
  default: defineComponent({ setup: (_props, { slots }) => () => h('section', slots.default?.()) }),
}))
vi.mock('@/components/profiles/ProfileListRow.vue', () => ({ default: defineComponent(() => () => h('div')) }))
vi.mock('@/components/profiles/ProfilesInspector.vue', () => ({ default: defineComponent(() => () => h('aside')) }))
vi.mock('@/components/profiles/ProfilesCommandPalette.vue', () => ({ default: defineComponent(() => () => h('div')) }))
vi.mock('@/components/profiles/ProfileDiffRows.vue', () => ({ default: defineComponent(() => () => h('div')) }))

vi.mock('@/components/grok/GrokProfileCard.vue', () => ({
  default: defineComponent({
    props: { profile: { type: Object, required: true } },
    emits: ['edit', 'delete'],
    setup: (props, { emit }) => () => h('article', { 'data-profile-name': (props.profile as GrokProfileDto).name }, [
      h('button', { 'data-testid': `edit-${(props.profile as GrokProfileDto).name}`, onClick: () => emit('edit', (props.profile as GrokProfileDto).name) }, 'edit'),
      h('button', { 'data-testid': `delete-${(props.profile as GrokProfileDto).name}`, onClick: () => emit('delete', (props.profile as GrokProfileDto).name) }, 'delete'),
    ]),
  }),
}))

vi.mock('@/components/grok/GrokProfileEditorModal.vue', () => ({
  default: defineComponent({
    props: {
      modelValue: { type: Boolean, required: true },
      form: { type: Object, required: true },
      updateField: { type: Function, required: true },
    },
    emits: ['save'],
    setup: (props, { emit }) => () => props.modelValue
      ? h('div', { 'data-testid': 'editor' }, [
          h('input', {
            'data-testid': 'editor-name',
            value: (props.form as GrokProfileDto).name,
            onInput: (event: Event) => props.updateField('name', (event.target as HTMLInputElement).value),
          }),
          h('button', { 'data-testid': 'editor-save', onClick: () => emit('save') }, 'save'),
        ])
      : null,
  }),
}))

vi.mock('@/components/ConfirmModal.vue', () => ({
  default: defineComponent({
    props: {
      isOpen: { type: Boolean, required: true },
      title: { type: String, default: '' },
      message: { type: String, default: '' },
      confirmText: { type: String, default: '' },
    },
    emits: ['confirm', 'update:isOpen'],
    setup: (props, { emit }) => () => props.isOpen
      ? h('div', { 'data-testid': 'confirm' }, [
          h('span', `${props.title} ${props.message}`),
          h('button', {
            'data-testid': 'confirm-action',
            onClick: () => {
              emit('confirm')
              emit('update:isOpen', false)
            },
          }, props.confirmText),
        ])
      : null,
  }),
}))

import GrokProfilesView from '@/views/grok/GrokProfilesView.vue'

const oldProfile: GrokProfileDto = {
  name: 'old-name',
  description: 'Old profile',
  provider: 'Example',
  profile_kind: 'third_party',
  base_url_display: 'https://example.com/v1',
  has_base_url: true,
  model: 'grok-4',
  api_backend: 'responses',
  context_window: 128000,
  supports_backend_search: true,
  reasoning_effort: 'medium',
  auth_mode: 'env_key',
  env_key: 'GROK_API_KEY',
  has_inline_credential: false,
  enabled: true,
  tags: [],
}
const newProfile: GrokProfileDto = { ...oldProfile, name: 'new-name' }

const listResponse = (
  profiles = [oldProfile],
  currentProfile: string | null = 'old-name',
) => ({
  status: 'ok' as const,
  profiles,
  current_profile: currentProfile,
  activation: currentProfile ? 'active' as const : 'inactive' as const,
  activation_name: currentProfile,
  default_profile: null,
})

const flushPromises = async () => {
  await Promise.resolve()
  await nextTick()
  await nextTick()
}

const mountView = async () => {
  const element = document.createElement('div')
  document.body.appendChild(element)
  const app = createApp(defineComponent({ setup: () => () => h(GrokProfilesView) }))
  app.use(createPinia())
  app.mount(element)
  await flushPromises()
  return { element, unmount: () => { app.unmount(); element.remove() } }
}

const click = async (element: Element | null) => {
  expect(element).not.toBeNull()
  ;(element as HTMLElement).click()
  await flushPromises()
}

const renameThroughEditor = async (element: HTMLElement) => {
  await click(element.querySelector('[data-testid="edit-old-name"]'))
  const input = element.querySelector<HTMLInputElement>('[data-testid="editor-name"]')
  expect(input).not.toBeNull()
  input!.value = 'new-name'
  input!.dispatchEvent(new Event('input', { bubbles: true }))
  await click(element.querySelector('[data-testid="editor-save"]'))
}

describe('GrokProfilesView orchestration', () => {
  beforeEach(() => {
    localStorage.clear()
    for (const mock of Object.values(apiMocks)) mock.mockReset()
    getCurrentEnvironmentMock.mockReset()
    getCurrentEnvironmentMock.mockResolvedValue({ env_type: 'local' })
    apiMocks.listGrokProfiles.mockResolvedValue(listResponse())
  })

  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('fails closed before loading profiles in a non-local environment', async () => {
    localStorage.setItem('ccr:profiles:pinned:grok', JSON.stringify(['old-name']))
    getCurrentEnvironmentMock.mockResolvedValue({ env_type: 'wsl' })
    const { element, unmount } = await mountView()
    try {
      expect(apiMocks.listGrokProfiles).not.toHaveBeenCalled()
      expect(element.textContent).toContain('grok.dashboard.localOnly.title')
      expect(element.querySelector('[data-testid="profiles-header"]')?.getAttribute('data-loading')).toBe('true')
      expect(JSON.parse(localStorage.getItem('ccr:profiles:pinned:grok') ?? '[]')).toEqual(['old-name'])
      await click(element.querySelector('[data-testid="add"]'))
      expect(element.querySelector('[data-testid="editor"]')).toBeNull()
    } finally {
      unmount()
    }
  })

  it('reports enabled profiles over total profiles in the health slot', async () => {
    apiMocks.listGrokProfiles.mockResolvedValue(listResponse([
      oldProfile,
      { ...newProfile, enabled: false, reasoning_effort: null },
    ]))
    const { element, unmount } = await mountView()
    try {
      expect(element.querySelector('[data-testid="stat-health"]')?.getAttribute('data-value'))
        .toBe('1/2')
      expect(element.querySelector('[data-testid="stat-health"]')?.getAttribute('data-warn'))
        .toBe('true')
    } finally {
      unmount()
    }
  })

  it('shows durable manual recovery guidance and never offers force for unsafe delete', async () => {
    apiMocks.deleteGrokProfile.mockResolvedValue({
      status: 'blocked',
      reason: 'unsafe_missing_entry_state',
      message: 'entry state is missing',
    })
    const { element, unmount } = await mountView()
    try {
      await click(element.querySelector('[data-testid="delete-old-name"]'))
      await click(element.querySelector('[data-testid="confirm-action"]'))

      const guidance = element.querySelector('[data-testid="unsafe-delete-recovery"]')
      expect(guidance?.textContent).toContain('entry state is missing')
      expect(guidance?.textContent).toContain('~/.grok/config.toml')
      expect(guidance?.querySelector('button')).toBeNull()
      expect(apiMocks.deleteGrokProfile).toHaveBeenCalledTimes(1)
      expect(apiMocks.deleteGrokProfile).toHaveBeenCalledWith('old-name', { force: false })
    } finally {
      unmount()
    }
  })

  it('offers force only after an active or drifted delete envelope', async () => {
    apiMocks.deleteGrokProfile
      .mockResolvedValueOnce({ status: 'blocked', reason: 'active', message: 'active' })
      .mockResolvedValueOnce({ status: 'deleted' })
    apiMocks.listGrokProfiles.mockResolvedValueOnce(listResponse()).mockResolvedValueOnce(listResponse([], null))
    const { element, unmount } = await mountView()
    try {
      await click(element.querySelector('[data-testid="delete-old-name"]'))
      await click(element.querySelector('[data-testid="confirm-action"]'))
      await new Promise(resolve => window.setTimeout(resolve, 0))
      await flushPromises()
      await click(element.querySelector('[data-testid="confirm-action"]'))

      expect(apiMocks.deleteGrokProfile).toHaveBeenNthCalledWith(1, 'old-name', { force: false })
      expect(apiMocks.deleteGrokProfile).toHaveBeenNthCalledWith(2, 'old-name', { force: true })
    } finally {
      unmount()
    }
  })

  it('does not reopen force confirmation when a forced delete remains blocked', async () => {
    apiMocks.deleteGrokProfile.mockResolvedValue({
      status: 'blocked',
      reason: 'active',
      message: 'still active',
    })
    const { element, unmount } = await mountView()
    try {
      await click(element.querySelector('[data-testid="delete-old-name"]'))
      await click(element.querySelector('[data-testid="confirm-action"]'))
      await new Promise(resolve => window.setTimeout(resolve, 0))
      await flushPromises()
      await click(element.querySelector('[data-testid="confirm-action"]'))
      await new Promise(resolve => window.setTimeout(resolve, 0))
      await flushPromises()

      expect(apiMocks.deleteGrokProfile).toHaveBeenCalledTimes(2)
      expect(element.querySelector('[data-testid="confirm"]')).toBeNull()
    } finally {
      unmount()
    }
  })

  it('keeps the old pin after rename apply failure and migrates it only after retry apply succeeds', async () => {
    localStorage.setItem('ccr:profiles:pinned:grok', JSON.stringify(['old-name']))
    localStorage.setItem('ccr:profiles:recent:grok', JSON.stringify(['old-name']))
    apiMocks.listGrokProfiles.mockResolvedValue(listResponse([oldProfile, newProfile]))
    apiMocks.updateGrokProfile.mockResolvedValue({
      status: 'rename_apply_failed',
      old_name: 'old-name',
      new_name: 'new-name',
      message: 'retry apply',
    })
    apiMocks.applyGrokProfile.mockResolvedValue({ status: 'applied', profile: 'new-name' })
    const { element, unmount } = await mountView()
    try {
      await renameThroughEditor(element)

      expect(JSON.parse(localStorage.getItem('ccr:profiles:pinned:grok') ?? '[]')).toEqual(['old-name'])
      expect(element.querySelector('[data-testid="rename-recovery"]')?.textContent).toContain('retry apply')

      await click(element.querySelector('[data-testid="rename-recovery-action"]'))

      expect(apiMocks.applyGrokProfile).toHaveBeenCalledWith('new-name')
      expect(JSON.parse(localStorage.getItem('ccr:profiles:pinned:grok') ?? '[]')).toEqual(['new-name'])
      expect(JSON.parse(localStorage.getItem('ccr:profiles:recent:grok') ?? '[]')).toEqual(['new-name'])
    } finally {
      unmount()
    }
  })

  it('migrates pins immediately after cleanup failure and retries deletion of the old name', async () => {
    localStorage.setItem('ccr:profiles:pinned:grok', JSON.stringify(['old-name']))
    apiMocks.listGrokProfiles.mockResolvedValue(listResponse([oldProfile, newProfile], 'new-name'))
    apiMocks.updateGrokProfile.mockResolvedValue({
      status: 'rename_cleanup_failed',
      old_name: 'old-name',
      new_name: 'new-name',
      message: 'retry cleanup',
    })
    apiMocks.deleteGrokProfile.mockResolvedValue({ status: 'deleted' })
    const { element, unmount } = await mountView()
    try {
      await renameThroughEditor(element)

      expect(JSON.parse(localStorage.getItem('ccr:profiles:pinned:grok') ?? '[]')).toEqual(['new-name'])
      await click(element.querySelector('[data-testid="rename-recovery-action"]'))
      expect(apiMocks.deleteGrokProfile).toHaveBeenCalledWith('old-name')
    } finally {
      unmount()
    }
  })
})
