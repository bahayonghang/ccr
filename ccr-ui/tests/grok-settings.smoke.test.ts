import { createPinia, setActivePinia } from 'pinia'
import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { GrokSettingsCommandResponse } from '@/types'
import {
  buildGrokSettingsPatch,
  grokSettingsResponseToForm,
  validateGrokSettingsForm,
  type GrokSettingsKey,
} from '@/utils/grokSettings'

const apiMocks = vi.hoisted(() => ({
  getGrokSettings: vi.fn(),
  updateGrokSettings: vi.fn(),
  getGrokConfigRaw: vi.fn(),
  saveGrokConfigRaw: vi.fn(),
  listGrokConfigLayers: vi.fn(),
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
vi.mock('@/components/ModuleSubnav.vue', () => ({
  default: defineComponent(() => () => h('nav')),
}))
vi.mock('@/components/ui/SIcon.vue', () => ({
  default: defineComponent({
    props: { name: { type: String, required: true } },
    setup: props => () => h('span', { 'data-icon': props.name }),
  }),
}))
vi.mock('@/components/ui/Button.vue', () => ({
  default: defineComponent({
    props: {
      disabled: { type: Boolean, default: false },
      loading: { type: Boolean, default: false },
    },
    emits: ['click'],
    setup: (props, { emit, slots }) => () => h('button', {
      disabled: props.disabled || props.loading,
      onClick: (event: MouseEvent) => emit('click', event),
    }, slots.default?.()),
  }),
}))
vi.mock('@/components/editor/ConfigSourcePanel.vue', () => ({
  default: defineComponent({
    props: {
      backupNotice: { type: String, default: '' },
      policyNotice: { type: String, default: '' },
    },
    setup: props => () => h('div', {
      'data-testid': 'source-panel',
      'data-backup-notice': props.backupNotice,
      'data-policy-notice': props.policyNotice,
    }),
  }),
}))

import GrokSettingsView from '@/views/grok/GrokSettingsView.vue'

const settingsResponse = (overrides: Partial<Extract<
  GrokSettingsCommandResponse,
  { status: 'ok' }
>> = {}): Extract<GrokSettingsCommandResponse, { status: 'ok' }> => ({
  status: 'ok',
  exists: true,
  activation: 'inactive',
  activation_name: null,
  managed_keys_locked: false,
  models: {
    default: 'grok-code-fast-1',
    default_reasoning_effort: 'medium',
  },
  ui: { theme: 'system' },
  session: {
    auto_compact_threshold_percent: 85,
    load_envrc: true,
  },
  cli: {
    auto_update: true,
    channel: 'stable',
    show_tips: true,
  },
  hints: {
    new_session_worktree_mode: 'ask',
    fork_worktree_mode: 'never',
  },
  custom_models: [],
  ...overrides,
})

const settle = async () => {
  await Promise.resolve()
  await nextTick()
  await Promise.resolve()
  await nextTick()
}

const mountView = async () => {
  const element = document.createElement('div')
  document.body.appendChild(element)
  const app = createApp(defineComponent({ setup: () => () => h(GrokSettingsView) }))
  app.use(createPinia())
  app.component('RouterLink', defineComponent({
    props: { to: { type: String, required: true } },
    setup: (props, { slots }) => () => h('a', { href: props.to }, slots.default?.()),
  }))
  app.mount(element)
  await settle()
  return {
    element,
    unmount: () => {
      app.unmount()
      element.remove()
    },
  }
}

const change = async (element: HTMLInputElement | HTMLSelectElement, value: string) => {
  element.value = value
  element.dispatchEvent(new Event(element instanceof HTMLSelectElement ? 'change' : 'input', {
    bubbles: true,
  }))
  await settle()
}

const clickButton = async (element: HTMLElement, text: string) => {
  const button = Array.from(element.querySelectorAll('button')).find(item => (
    item.textContent?.includes(text)
  ))
  expect(button).toBeDefined()
  button!.click()
  await settle()
}

describe('Grok settings patch serialization', () => {
  it('serializes only dirty keys and maps cleared values to unset', () => {
    const response = settingsResponse()
    const form = grokSettingsResponseToForm(response)
    form['ui.theme'] = 'dark'
    form['cli.show_tips'] = null
    form['session.auto_compact_threshold_percent'] = '90'
    const dirty = new Set<GrokSettingsKey>([
      'ui.theme',
      'cli.show_tips',
      'session.auto_compact_threshold_percent',
    ])

    expect(buildGrokSettingsPatch(form, dirty)).toEqual({
      set: {
        'ui.theme': 'dark',
        'session.auto_compact_threshold_percent': 90,
      },
      unset: ['cli.show_tips'],
    })
    expect(buildGrokSettingsPatch(form, new Set(['ui.theme']))).toEqual({
      set: { 'ui.theme': 'dark' },
      unset: [],
    })
  })

  it('rejects non-integer and out-of-range auto-compact thresholds', () => {
    const form = grokSettingsResponseToForm(settingsResponse())
    const dirty = new Set<GrokSettingsKey>(['session.auto_compact_threshold_percent'])
    form['session.auto_compact_threshold_percent'] = '100.5'
    expect(validateGrokSettingsForm(form, dirty)).toBe('session.auto_compact_threshold_percent')
    form['session.auto_compact_threshold_percent'] = '101'
    expect(validateGrokSettingsForm(form, dirty)).toBe('session.auto_compact_threshold_percent')
    form['session.auto_compact_threshold_percent'] = '0'
    expect(validateGrokSettingsForm(form, dirty)).toBeNull()
  })
})

describe('GrokSettingsView orchestration', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    for (const mock of Object.values(apiMocks)) mock.mockReset()
    getCurrentEnvironmentMock.mockReset()
    getCurrentEnvironmentMock.mockResolvedValue({ env_type: 'local' })
    apiMocks.getGrokSettings.mockResolvedValue(settingsResponse())
    apiMocks.updateGrokSettings.mockResolvedValue({ status: 'saved' })
  })

  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('fails closed before any Grok file API call outside the local environment', async () => {
    getCurrentEnvironmentMock.mockResolvedValue({ env_type: 'wsl' })
    const mounted = await mountView()
    try {
      expect(apiMocks.getGrokSettings).not.toHaveBeenCalled()
      expect(mounted.element.querySelector('[data-testid="grok-settings-local-only"]')).not.toBeNull()
    } finally {
      mounted.unmount()
    }
  })

  it('saves a single edited field as a field-level patch', async () => {
    const mounted = await mountView()
    try {
      const theme = mounted.element.querySelector<HTMLSelectElement>('[data-testid="grok-settings-theme"]')
      expect(theme).not.toBeNull()
      await change(theme!, 'dark')
      await clickButton(mounted.element, 'grok.settings.save')

      expect(apiMocks.updateGrokSettings).toHaveBeenCalledWith({
        set: { 'ui.theme': 'dark' },
        unset: [],
      })
      expect(apiMocks.getGrokSettings).toHaveBeenCalledTimes(2)
    } finally {
      mounted.unmount()
    }
  })

  it('keeps conflict visible until the user reloads the latest values', async () => {
    apiMocks.updateGrokSettings.mockResolvedValue({ status: 'conflict' })
    const mounted = await mountView()
    try {
      const channel = mounted.element.querySelector<HTMLSelectElement>('[data-testid="grok-settings-channel"]')
      await change(channel!, 'alpha')
      await clickButton(mounted.element, 'grok.settings.save')

      const conflict = mounted.element.querySelector<HTMLElement>('[data-testid="grok-settings-conflict"]')
      expect(conflict).not.toBeNull()
      conflict!.querySelector('button')!.click()
      await settle()

      expect(apiMocks.getGrokSettings).toHaveBeenCalledTimes(2)
      expect(mounted.element.querySelector('[data-testid="grok-settings-conflict"]')).toBeNull()
    } finally {
      mounted.unmount()
    }
  })

  it('locks managed model controls and presents backend lock rejection guidance', async () => {
    apiMocks.getGrokSettings.mockResolvedValue(settingsResponse({
      activation: 'active',
      activation_name: 'work',
      managed_keys_locked: true,
    }))
    apiMocks.updateGrokSettings.mockResolvedValue({
      status: 'managed_locked',
      message: 'turn profile mode off first',
    })
    const mounted = await mountView()
    try {
      expect(mounted.element.querySelector<HTMLInputElement>('[data-testid="grok-settings-model"]')?.disabled)
        .toBe(true)
      expect(mounted.element.querySelector('[data-testid="grok-settings-managed-banner"]')).not.toBeNull()

      const theme = mounted.element.querySelector<HTMLSelectElement>('[data-testid="grok-settings-theme"]')
      await change(theme!, 'dark')
      await clickButton(mounted.element, 'grok.settings.save')

      expect(mounted.element.querySelector('[data-testid="grok-settings-managed-error"]')?.textContent)
        .toContain('turn profile mode off first')
    } finally {
      mounted.unmount()
    }
  })
})
