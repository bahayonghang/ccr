import { createPinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick, reactive } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { CodexProfileEditorForm } from '@/utils/codexProfileEditor'
import type { ProviderTemplateDraftContext, ProviderTemplateSelection } from '@/types/providerTemplates'

const helperMocks = vi.hoisted(() => ({
  copyToClipboard: vi.fn(),
}))

vi.mock('@/utils/codexHelpers', () => ({
  copyToClipboard: helperMocks.copyToClipboard,
}))

vi.mock('@/components/common/BaseModal.vue', () => ({
  default: defineComponent({
    props: {
      modelValue: { type: Boolean, required: true },
      contentClass: { type: String, default: '' },
    },
    setup(props, { slots }) {
      return () => props.modelValue
        ? h('div', { class: props.contentClass }, [
            slots.header?.({ titleId: 'modal-title' }),
            slots.default?.(),
          ])
        : null
    },
  }),
}))

vi.mock('@/components/ui/SIcon.vue', () => ({
  default: defineComponent({
    props: {
      name: { type: String, required: true },
      size: { type: String, default: '' },
    },
    setup(props) {
      return () => h('span', { 'data-icon': props.name, class: props.size })
    },
  }),
}))

vi.mock('@/components/provider-templates/ProviderTemplateSelector.vue', () => ({
  default: defineComponent({
    props: {
      platform: { type: String, required: true },
      selectedTemplateId: { type: String, default: null },
      selectedEndpoint: { type: String, default: '' },
      draftContext: { type: Object, default: null },
      label: { type: String, default: '' },
      helper: { type: String, default: '' },
      placeholder: { type: String, default: '' },
    },
    emits: ['select', 'manual'],
    setup(props, { emit }) {
      return () => h('section', {
        'data-testid': 'provider-template-selector',
        'data-platform': props.platform,
        'data-selected-template': props.selectedTemplateId || '',
        'data-selected-endpoint': props.selectedEndpoint || '',
      }, [
        h('span', { 'data-testid': 'provider-template-label' }, props.label),
        h('span', { 'data-testid': 'provider-template-helper' }, props.helper),
        h('span', { 'data-testid': 'provider-template-placeholder' }, props.placeholder),
        h('button', {
          type: 'button',
          'data-testid': 'provider-template-select',
          onClick: () => emit('select', {
            template: { id: 'openrouter', name: 'OpenRouter', platforms: { codex: {} } },
            endpoint: 'https://openrouter.ai/api/v1',
          }),
        }, 'select'),
        h('button', {
          type: 'button',
          'data-testid': 'provider-template-manual',
          onClick: () => emit('manual'),
        }, 'manual'),
      ])
    },
  }),
}))

import CodexProfileEditorModal from '@/components/codex/CodexProfileEditorModal.vue'
import {
  buildCodexProfileRequest,
  createCodexProfileEditorForm,
  REASONING_EFFORT_OPTIONS,
} from '@/utils/codexProfileEditor'

const i18n = createI18n({
  legacy: false,
  locale: 'en',
  missingWarn: false,
  fallbackWarn: false,
  messages: {
    en: {
      common: {
        close: 'Close',
        cancel: 'Cancel',
      },
      codex: {
        actions: {
          save: 'Save',
        },
        states: {
          enabled: 'Enabled',
          disabled: 'Disabled',
        },
        profiles: {
          addProfile: 'Add Profile',
          editProfile: 'Edit Profile',
          subtitle: 'Manage profiles',
          notAvailable: 'N/A',
          openAiAuthOn: 'Uses OpenAI auth',
          openAiAuthOff: 'Direct runtime auth',
          modalFooterHint: 'Footer hint',
          nameCreateHint: 'Create name hint',
          nameRenameHint: 'Rename name hint',
          baseUrlRequiredHint: 'Base URL required',
          baseUrlOptionalHint: 'Base URL optional',
          envKeyHint: 'Env key hint',
          customModelOption: 'Custom model',
          customModelHint: 'Custom model hint',
          modelPresetHint: 'Preset hint',
          templateSelector: {
            label: 'Provider template',
            helper: 'Fill non-secret provider fields.',
            placeholder: 'Choose a Codex provider template',
          },
          reasoningEffortHint: 'Reasoning hint',
          enabledHint: 'Enabled hint',
          deprecatedAuthModeHint: 'Deprecated {mode}',
          sections: {
            identity: 'Identity',
            authentication: 'Authentication',
            runtime: 'Model & Runtime',
            metadata: 'Metadata',
          },
          sectionHints: {
            identity: 'Identity hint',
            authentication: 'Authentication hint',
            runtime: 'Runtime hint',
            metadata: 'Metadata hint',
          },
          tokenActions: {
            show: 'Show auth token',
            hide: 'Hide auth token',
            copy: 'Copy auth token',
          },
          fields: {
            name: 'Name',
            description: 'Description',
            authMode: 'Auth Mode',
            openAiLoginMethod: 'OpenAI Login Method',
            baseUrl: 'Base URL',
            authToken: 'Auth Token',
            envKey: 'Env Key',
            model: 'Model',
            reasoningEffort: 'Reasoning Effort',
            wireApi: 'Wire API',
            provider: 'Provider',
            providerType: 'Provider Type',
            tags: 'Tags',
            enabled: 'Enabled',
          },
          placeholders: {
            name: 'profile-name',
            description: 'description',
            baseUrl: 'https://example.com',
            authToken: 'sk-...',
            envKey: 'EXAMPLE_KEY',
            customModel: 'custom-model',
            wireApi: 'responses',
            provider: 'provider',
            providerType: 'provider-type',
            tags: 'free, stable',
          },
          messages: {
            tokenCopied: 'Token copied',
            tokenCopyFailed: 'Token copy failed',
          },
        },
      },
    },
  },
})

const createForm = (): CodexProfileEditorForm => ({
  ...createCodexProfileEditorForm(),
  name: 'ice',
  description: 'relay profile',
  auth_token: 'sk-test-123',
  base_url: 'https://ice.vu/v1',
  auth_mode: 'openai_api_key',
  wire_api: 'responses',
  model_reasoning_effort: 'high',
  provider: 'ice',
  provider_type: 'official_relay',
  tags_input: 'free, stable',
})

const mountModal = async (
  form: CodexProfileEditorForm,
  options: {
    selectedProviderTemplate?: string | null
    selectedProviderEndpoint?: string
    providerTemplateDraft?: ProviderTemplateDraftContext | null
    onSelectTemplate?: (selection: ProviderTemplateSelection) => void
    onManualTemplate?: () => void
  } = {},
) => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      const state = reactive(form)

      return () => h(CodexProfileEditorModal, {
        modelValue: true,
        editingName: 'ice',
        saving: false,
        form: state,
        updateField: (field: keyof CodexProfileEditorForm, value: string | boolean) => {
          state[field] = value as never
        },
        availableAuthModeOptions: ['openai_api_key', 'no_auth'],
        modelCatalog: ['gpt-5.4'],
        selectedModelOption: 'gpt-5.4',
        customModelInput: '',
        requiresBaseUrl: true,
        requiresSecret: true,
        requiresEnvKey: false,
        authTokenHint: 'token hint',
        isDeprecatedAuthMode: false,
        displayOpenAiLoginMethod: 'api',
        authModeLabel: (mode: string) => mode,
        selectedProviderTemplate: options.selectedProviderTemplate,
        selectedProviderEndpoint: options.selectedProviderEndpoint,
        providerTemplateDraft: options.providerTemplateDraft,
        'onUpdate:modelValue': () => undefined,
        'onUpdate:selectedModelOption': () => undefined,
        'onUpdate:customModelInput': () => undefined,
        onSelectTemplate: options.onSelectTemplate,
        onManualTemplate: options.onManualTemplate,
      })
    },
  }))

  app.use(createPinia())
  app.use(i18n)
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

beforeEach(() => {
  helperMocks.copyToClipboard.mockReset()
  helperMocks.copyToClipboard.mockResolvedValue(true)
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('codex profile editor helpers', () => {
  it('builds payloads with explicit clears for removed fields', () => {
    const request = buildCodexProfileRequest(createForm(), 'gpt-5.4')

    expect(request.model).toBe('gpt-5.4')
    expect(request.model_reasoning_effort).toBe('high')
    expect(request.tags).toEqual(['free', 'stable'])
    expect(request.small_fast_model).toBeNull()
    expect(request.extra).toBeNull()
  })
})

describe('CodexProfileEditorModal smoke', () => {
  const templateDraft: ProviderTemplateDraftContext = {
    platform: 'codex',
    defaultName: 'OpenRouter',
    name: 'OpenRouter',
    category: 'aggregator',
    baseUrls: ['https://openrouter.ai/api/v1'],
    modelCatalog: ['anthropic/claude-sonnet-4.6'],
    platformOverride: {
      baseUrl: 'https://openrouter.ai/api/v1',
      modelCatalog: ['anthropic/claude-sonnet-4.6'],
    },
  }

  it('keeps removed fields out of the UI', async () => {
    const { el, unmount } = await mountModal(createForm())

    try {
      expect(el.textContent).not.toContain('Small Fast Model')
      expect(el.textContent).not.toContain('Advanced Fields')
    } finally {
      unmount()
    }
  })

  it('masks the auth token by default and toggles visibility', async () => {
    const { el, unmount } = await mountModal(createForm())

    try {
      const input = el.querySelector<HTMLInputElement>('[data-testid="codex-auth-token-input"]')
      const toggle = el.querySelector<HTMLButtonElement>('[data-testid="codex-auth-token-visibility"]')

      expect(input?.type).toBe('password')
      toggle?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await nextTick()

      expect(input?.type).toBe('text')
    } finally {
      unmount()
    }
  })

  it('copies the real auth token value', async () => {
    const { el, unmount } = await mountModal(createForm())

    try {
      const copyButton = el.querySelector<HTMLButtonElement>('[data-testid="codex-auth-token-copy"]')
      copyButton?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await Promise.resolve()

      expect(helperMocks.copyToClipboard).toHaveBeenCalledWith('sk-test-123')
    } finally {
      unmount()
    }
  })

  it('keeps the name input editable while editing an existing profile', async () => {
    const { el, unmount } = await mountModal(createForm())

    try {
      const input = el.querySelector<HTMLInputElement>('[data-testid="codex-profile-name-input"]')

      expect(input?.disabled).toBe(false)
      expect(el.textContent).toContain('Rename name hint')
    } finally {
      unmount()
    }
  })

  it('renders the full reasoning effort option set', async () => {
    const { el, unmount } = await mountModal(createForm())

    try {
      const select = el.querySelector<HTMLSelectElement>('[data-testid="codex-reasoning-effort-select"]')
      const values = Array.from(select?.querySelectorAll('option') ?? []).map(option => option.value)

      expect(values).toEqual(['', ...REASONING_EFFORT_OPTIONS])
    } finally {
      unmount()
    }
  })

  it('renders the Codex provider template selector and forwards template events', async () => {
    const onSelectTemplate = vi.fn()
    const onManualTemplate = vi.fn()
    const { el, unmount } = await mountModal(createForm(), {
      selectedProviderTemplate: 'openrouter',
      selectedProviderEndpoint: 'https://openrouter.ai/api/v1',
      providerTemplateDraft: templateDraft,
      onSelectTemplate,
      onManualTemplate,
    })

    try {
      const selector = el.querySelector<HTMLElement>('[data-testid="provider-template-selector"]')
      expect(selector).toBeTruthy()
      expect(selector?.dataset.platform).toBe('codex')
      expect(selector?.dataset.selectedTemplate).toBe('openrouter')
      expect(selector?.dataset.selectedEndpoint).toBe('https://openrouter.ai/api/v1')
      expect(el.textContent).toContain('Provider template')
      expect(el.textContent).toContain('Choose a Codex provider template')

      el.querySelector<HTMLButtonElement>('[data-testid="provider-template-select"]')
        ?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      el.querySelector<HTMLButtonElement>('[data-testid="provider-template-manual"]')
        ?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await nextTick()

      expect(onSelectTemplate).toHaveBeenCalledWith(expect.objectContaining({
        endpoint: 'https://openrouter.ai/api/v1',
      }))
      expect(onManualTemplate).toHaveBeenCalled()
    } finally {
      unmount()
    }
  })
})
