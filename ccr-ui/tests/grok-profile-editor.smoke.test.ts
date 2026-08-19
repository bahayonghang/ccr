import { createApp, defineComponent, h, nextTick, reactive } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { GrokProfileDto } from '@/types'
import GrokProfileCard from '@/components/grok/GrokProfileCard.vue'
import GrokProfileEditorModal from '@/components/grok/GrokProfileEditorModal.vue'
import {
  buildGrokCreateRequest,
  buildGrokPatch,
  createEmptyGrokForm,
  fillGrokForm,
  type GrokProfileDirtyField,
  type GrokProfileEditorForm,
} from '@/utils/grokProfileEditor'

vi.mock('vue-i18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}))

vi.mock('@/components/common/BaseModal.vue', () => ({
  default: defineComponent({
    props: { modelValue: { type: Boolean, required: true } },
    setup(props, { slots }) {
      return () => props.modelValue
        ? h('div', { 'data-testid': 'modal' }, [slots.default?.(), slots.footer?.()])
        : null
    },
  }),
}))

vi.mock('@/components/ui/SIcon.vue', () => ({
  default: defineComponent({
    props: { name: { type: String, required: true } },
    setup: props => () => h('span', { 'data-icon': props.name }),
  }),
}))

const thirdPartyProfile: GrokProfileDto = {
  name: 'relay',
  description: 'Relay profile',
  provider: 'Example',
  profile_kind: 'third_party',
  base_url_display: 'https://user@example.com/v1?token=redacted',
  has_base_url: true,
  model: 'grok-4',
  api_backend: 'responses',
  context_window: 131072,
  supports_backend_search: true,
  reasoning_effort: 'medium',
  auth_mode: 'env_key',
  env_key: 'GROK_API_KEY',
  has_inline_credential: false,
  enabled: true,
  tags: ['relay'],
}
const officialProfile: GrokProfileDto = {
  ...thirdPartyProfile,
  name: 'official',
  provider: null,
  profile_kind: 'official',
  base_url_display: null,
  has_base_url: false,
  auth_mode: 'session',
  env_key: null,
}

const editorForm = (): GrokProfileEditorForm => ({
  ...fillGrokForm(thirdPartyProfile),
  apiKey: 'secret-api-key',
  envKey: 'REPLACEMENT_KEY',
})

const dirty = (...fields: GrokProfileDirtyField[]) => new Set(fields)

describe('Grok profile create defaults', () => {
  it('starts a new third-party profile with the product defaults', () => {
    const form = createEmptyGrokForm()

    expect(form).toMatchObject({
      profileKind: 'third_party',
      model: 'grok-4.6',
      reasoningEffort: 'high',
      credentialAction: 'replace_api_key',
      apiBackend: 'responses',
      contextWindow: '500000',
      supportsBackendSearch: true,
      tagsInput: 'work',
      enabled: true,
      apiKey: '',
      envKey: '',
    })

    const request = buildGrokCreateRequest({
      ...form,
      name: 'demo',
      baseUrl: 'https://api.example.com/v1',
      apiKey: 'secret-api-key',
    })

    expect(request).toMatchObject({
      name: 'demo',
      profile_kind: 'third_party',
      model: 'grok-4.6',
      reasoning_effort: 'high',
      credential_action: 'replace_api_key',
      api_backend: 'responses',
      context_window: 500000,
      supports_backend_search: true,
      tags: ['work'],
      api_key: 'secret-api-key',
    })
  })

  it('does not send a replace credential action when creating an official profile', () => {
    const request = buildGrokCreateRequest({
      ...createEmptyGrokForm(),
      name: 'official',
      profileKind: 'official',
    })

    expect(request.credential_action).toBe('preserve')
    expect(request).not.toHaveProperty('api_key')
    expect(request).not.toHaveProperty('env_key')
  })
})

describe('Grok profile editor patch contract', () => {
  it('keeps display-safe base URLs out of every editable field and reasoning-only patch', () => {
    const form = editorForm()
    form.reasoningEffort = 'high'

    expect(form.baseUrl).toBe('')
    const patch = buildGrokPatch(form, dirty('reasoningEffort'))

    expect(patch).toEqual({ reasoning_effort: 'high' })
    expect(patch).not.toHaveProperty('base_url')
    expect(patch).not.toHaveProperty('base_url_display')
    expect(patch).not.toHaveProperty('credential_action')
    expect(patch).not.toHaveProperty('api_key')
    expect(patch).not.toHaveProperty('env_key')
  })

  it('serializes an explicitly cleared model as null instead of treating it as untouched', () => {
    const form = editorForm()
    form.model = '   '

    expect(buildGrokPatch(form, dirty('model'))).toEqual({ model: null })
  })

  it.each([
    ['preserve', { credential_action: 'preserve' }],
    ['replace_api_key', { credential_action: 'replace_api_key', api_key: 'secret-api-key' }],
    ['replace_env_key', { credential_action: 'replace_env_key', env_key: 'REPLACEMENT_KEY' }],
    ['clear', { credential_action: 'clear' }],
  ] as const)('sends only fields for the %s credential action', (credentialAction, expected) => {
    const form = editorForm()
    form.credentialAction = credentialAction

    const patch = buildGrokPatch(form, dirty('credentialAction'))

    expect(patch).toEqual(expected)
    if (credentialAction !== 'replace_api_key') expect(patch).not.toHaveProperty('api_key')
    if (credentialAction !== 'replace_env_key') expect(patch).not.toHaveProperty('env_key')
  })
})

describe('GrokProfileEditorModal smoke', () => {
  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('renders the third-party create defaults in the editor', async () => {
    const element = document.createElement('div')
    document.body.appendChild(element)
    const form = reactive(createEmptyGrokForm())
    const app = createApp(defineComponent({
      setup() {
        return () => h(GrokProfileEditorModal, {
          modelValue: true,
          editingName: null,
          saving: false,
          form,
          updateField: vi.fn(),
        })
      },
    }))

    app.mount(element)
    await nextTick()

    try {
      const kinds = Array.from(element.querySelectorAll<HTMLButtonElement>('.grok-kind-control__button'))
      expect(kinds.find(button => button.classList.contains('grok-kind-control__button--active'))?.textContent)
        .toContain('third_party')
      expect(element.querySelector<HTMLInputElement>('#grok-profile-model')?.value).toBe('grok-4.6')
      expect(element.querySelector<HTMLSelectElement>('#grok-profile-reasoning')?.value).toBe('high')
      expect(element.querySelector<HTMLSelectElement>('#grok-credential-action')?.value).toBe('replace_api_key')
      expect(element.querySelector('#grok-profile-api-key')).not.toBeNull()
      expect(element.querySelector<HTMLSelectElement>('#grok-profile-api-backend')?.value).toBe('responses')
      expect(element.querySelector<HTMLInputElement>('#grok-profile-context-window')?.value).toBe('500000')
      expect(element.querySelector<HTMLInputElement>('input[type="checkbox"]')?.checked).toBe(true)
      expect(element.querySelector<HTMLInputElement>('#grok-profile-tags')?.value).toBe('work')
    } finally {
      app.unmount()
      element.remove()
    }
  })

  it('starts write-only fields blank and uses the display URL only as a placeholder', async () => {
    const element = document.createElement('div')
    document.body.appendChild(element)
    const form = reactive(editorForm())
    const app = createApp(defineComponent({
      setup() {
        return () => h(GrokProfileEditorModal, {
          modelValue: true,
          editingName: 'relay',
          saving: false,
          form,
          updateField: (field: keyof GrokProfileEditorForm, value: string | boolean) => {
            form[field] = value as never
          },
          baseUrlDisplay: thirdPartyProfile.base_url_display,
          hasExistingBaseUrl: true,
          currentAuthMode: thirdPartyProfile.auth_mode,
          currentEnvKey: thirdPartyProfile.env_key,
        })
      },
    }))

    app.mount(element)
    await nextTick()

    try {
      const baseUrl = element.querySelector<HTMLInputElement>('#grok-profile-base-url')
      const apiKey = element.querySelector<HTMLInputElement>('#grok-profile-api-key')

      expect(baseUrl?.value).toBe('')
      expect(baseUrl?.placeholder).toBe(thirdPartyProfile.base_url_display)
      expect(apiKey).toBeNull()
      expect(element.textContent).not.toContain('secret-api-key')
      expect(element.textContent).toContain('GROK_API_KEY')
    } finally {
      app.unmount()
      element.remove()
    }
  })

  it('keeps the backend profile kind authoritative while editing', async () => {
    const element = document.createElement('div')
    document.body.appendChild(element)
    const form = reactive({ ...createEmptyGrokForm(), ...fillGrokForm(thirdPartyProfile) })
    const app = createApp(defineComponent({
      setup() {
        return () => h(GrokProfileEditorModal, {
          modelValue: true,
          editingName: 'relay',
          saving: false,
          form,
          updateField: vi.fn(),
        })
      },
    }))

    app.mount(element)
    await nextTick()

    try {
      const kinds = Array.from(element.querySelectorAll<HTMLButtonElement>('.grok-kind-control__button'))
      expect(kinds).toHaveLength(2)
      expect(kinds.every(button => button.disabled)).toBe(true)
      expect(kinds.find(button => button.classList.contains('grok-kind-control__button--active'))?.textContent)
        .toContain('third_party')
    } finally {
      app.unmount()
      element.remove()
    }
  })

  it('hides third-party provider and credential controls for official profiles', async () => {
    const element = document.createElement('div')
    document.body.appendChild(element)
    const form = reactive(fillGrokForm(officialProfile))
    const app = createApp(defineComponent({
      setup() {
        return () => h(GrokProfileEditorModal, {
          modelValue: true,
          editingName: 'official',
          saving: false,
          form,
          updateField: vi.fn(),
          currentAuthMode: officialProfile.auth_mode,
        })
      },
    }))

    app.mount(element)
    await nextTick()

    try {
      expect(element.querySelector('#grok-profile-provider')).toBeNull()
      expect(element.querySelector('#grok-profile-base-url')).toBeNull()
      expect(element.querySelector('#grok-credential-action')).toBeNull()
    } finally {
      app.unmount()
      element.remove()
    }
  })

  it('renders the shared pe-shell navigation, scroll root, and footer for third-party profiles', async () => {
    const element = document.createElement('div')
    document.body.appendChild(element)
    const form = reactive(editorForm())
    const app = createApp(defineComponent({
      setup() {
        return () => h(GrokProfileEditorModal, {
          modelValue: true,
          editingName: 'relay',
          saving: false,
          form,
          updateField: vi.fn(),
        })
      },
    }))

    app.mount(element)
    await nextTick()

    try {
      const shell = element.querySelector('.pe-shell')
      expect(shell).not.toBeNull()
      expect(shell?.className).toMatch(/max-h-\[calc\(90vh-9rem\)\]/)
      expect(shell?.classList.contains('overflow-hidden')).toBe(true)
      expect(element.querySelector('.pe-nav')).not.toBeNull()
      expect(element.querySelector('.pe-scroll')).not.toBeNull()
      expect(element.querySelector('.pe-footer')).not.toBeNull()
      expect(Array.from(element.querySelectorAll('.pe-nav__item')).map(item => item.textContent))
        .toContain('grok.profiles.editor.connection')
    } finally {
      app.unmount()
      element.remove()
    }
  })

  it('omits the connection nav item for official profiles', async () => {
    const element = document.createElement('div')
    document.body.appendChild(element)
    const form = reactive(fillGrokForm(officialProfile))
    const app = createApp(defineComponent({
      setup() {
        return () => h(GrokProfileEditorModal, {
          modelValue: true,
          editingName: 'official',
          saving: false,
          form,
          updateField: vi.fn(),
        })
      },
    }))

    app.mount(element)
    await nextTick()

    try {
      const navItems = Array.from(element.querySelectorAll('.pe-nav__item')).map(item => item.textContent)
      expect(navItems).toContain('grok.profiles.editor.identity')
      expect(navItems).toContain('grok.profiles.editor.runtime')
      expect(navItems).toContain('grok.profiles.editor.status')
      expect(navItems).not.toContain('grok.profiles.editor.connection')
      expect(element.querySelector('#connection')).toBeNull()
      expect(element.querySelector('.pe-scroll')).not.toBeNull()
      expect(element.querySelector('.pe-footer')).not.toBeNull()
    } finally {
      app.unmount()
      element.remove()
    }
  })

  it('shows a jump control when save validation fails', async () => {
    const element = document.createElement('div')
    document.body.appendChild(element)
    const form = reactive(createEmptyGrokForm())
    form.profileKind = 'third_party'
    const save = vi.fn()
    const app = createApp(defineComponent({
      setup() {
        return () => h(GrokProfileEditorModal, {
          modelValue: true,
          editingName: null,
          saving: false,
          form,
          updateField: vi.fn(),
          onSave: save,
        })
      },
    }))

    app.mount(element)
    await nextTick()

    try {
      const saveButton = Array.from(element.querySelectorAll('button'))
        .find(button => button.textContent?.includes('grok.profiles.actions.save'))
      expect(saveButton).toBeDefined()
      saveButton?.click()
      await nextTick()

      expect(save).not.toHaveBeenCalled()
      expect(element.querySelector('.pe-summary')).not.toBeNull()
      expect(element.querySelector('.pe-summary__jump')).not.toBeNull()
      expect(element.querySelector('.pe-summary__jump')?.textContent)
        .toContain('grok.profiles.editor.validationJump')
    } finally {
      app.unmount()
      element.remove()
    }
  })
})

describe('GrokProfileCard smoke', () => {
  it('shows the Grok backend and context-window fields on profile cards', async () => {
    const element = document.createElement('div')
    document.body.appendChild(element)
    const app = createApp(defineComponent({
      setup: () => () => h(GrokProfileCard, {
        profile: thirdPartyProfile,
        isCurrent: false,
      }),
    }))

    app.mount(element)
    await nextTick()

    try {
      expect(element.textContent).toContain('responses')
      expect(element.textContent).toContain((thirdPartyProfile.context_window ?? 0).toLocaleString())
    } finally {
      app.unmount()
      element.remove()
    }
  })
})
