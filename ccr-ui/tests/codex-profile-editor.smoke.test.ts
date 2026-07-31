import { createPinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick, reactive } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { CodexProfileEditorForm } from '@/utils/codexProfileEditor'
import type {
  ProviderTemplateDraftContext,
  ProviderTemplateSelection,
} from '@/types/providerTemplates'

const helperMocks = vi.hoisted(() => ({
  copyText: vi.fn(),
}))

vi.mock('@/utils/clipboard', () => ({
  copyText: helperMocks.copyText,
}))

vi.mock('@/components/common/BaseModal.vue', () => ({
  default: defineComponent({
    props: {
      modelValue: { type: Boolean, required: true },
      contentClass: { type: String, default: '' },
    },
    setup(props, { slots }) {
      return () =>
        props.modelValue
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
      return () =>
        h(
          'section',
          {
            'data-testid': 'provider-template-selector',
            'data-platform': props.platform,
            'data-selected-template': props.selectedTemplateId || '',
            'data-selected-endpoint': props.selectedEndpoint || '',
          },
          [
            h('span', { 'data-testid': 'provider-template-label' }, props.label),
            h('span', { 'data-testid': 'provider-template-helper' }, props.helper),
            h('span', { 'data-testid': 'provider-template-placeholder' }, props.placeholder),
            h(
              'button',
              {
                type: 'button',
                'data-testid': 'provider-template-select',
                onClick: () =>
                  emit('select', {
                    template: { id: 'openrouter', name: 'OpenRouter', platforms: { codex: {} } },
                    endpoint: 'https://openrouter.ai/api/v1',
                  }),
              },
              'select'
            ),
            h(
              'button',
              {
                type: 'button',
                'data-testid': 'provider-template-manual',
                onClick: () => emit('manual'),
              },
              'manual'
            ),
          ]
        )
    },
  }),
}))

import CodexProfileEditorModal from '@/components/codex/CodexProfileEditorModal.vue'
import {
  buildCodexProfileModelCatalog,
  buildCodexProfileRequest,
  createCodexProfileEditorForm,
  CUSTOM_MODEL_OPTION,
  REASONING_EFFORT_OPTIONS,
  resolveModelSelection,
} from '@/utils/codexProfileEditor'

const modelPresets = ['gpt-5.6-luna', 'gpt-5.6-terra', 'gpt-5.6-sol']

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
          currentModelOption: '{model} (current profile value)',
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
          validationJump: 'Jump',
          validation: {
            nameRequired: 'Profile name is required',
            baseUrlRequired: 'Base URL is required',
            authTokenRequired: 'Auth token is required',
            envKeyRequired: 'Env key is required',
            modelRequired: 'Model is required',
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
    editingName?: string | null
    modelCatalog?: string[]
    currentModelOption?: string
    selectedModelOption?: string
    customModelInput?: string
    resolvedModel?: string
    onSave?: () => void
  } = {}
) => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(
    defineComponent({
      setup() {
        const state = reactive(form)

        return () =>
          h(CodexProfileEditorModal, {
            modelValue: true,
            editingName: options.editingName === undefined ? 'ice' : options.editingName,
            saving: false,
            form: state,
            updateField: (field: keyof CodexProfileEditorForm, value: string | boolean) => {
              state[field] = value as never
            },
            availableAuthModeOptions: ['openai_api_key', 'no_auth'],
            modelCatalog: options.modelCatalog ?? modelPresets,
            currentModelOption: options.currentModelOption,
            selectedModelOption: options.selectedModelOption ?? modelPresets[0],
            customModelInput: options.customModelInput ?? '',
            resolvedModel: options.resolvedModel ?? modelPresets[0],
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
            onSave: options.onSave,
          })
      },
    })
  )

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
  helperMocks.copyText.mockReset()
  helperMocks.copyText.mockResolvedValue(true)
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('codex profile editor helpers', () => {
  it('builds a stable preset catalog with an optional current profile value', () => {
    expect(buildCodexProfileModelCatalog(modelPresets)).toEqual(modelPresets)
    expect(
      buildCodexProfileModelCatalog([...modelPresets, 'gpt-5.6-luna', ''], ' gpt-5.4 ')
    ).toEqual([...modelPresets, 'gpt-5.4'])
  })

  it('routes non-preset provider models through the per-profile custom input', () => {
    expect(resolveModelSelection('openai/gpt-5.1', modelPresets)).toEqual({
      selectedModelOption: CUSTOM_MODEL_OPTION,
      customModelInput: 'openai/gpt-5.1',
    })
  })

  it('builds payloads with explicit clears for removed fields', () => {
    const request = buildCodexProfileRequest(createForm(), 'gpt-5.4')

    expect(request.model).toBe('gpt-5.4')
    expect(request.model_reasoning_effort).toBe('high')
    expect(request.tags).toEqual(['free', 'stable'])
    expect(request.small_fast_model).toBeNull()
    expect(request.extra).toBeNull()
  })

  it('derives the OpenAI auth flags from auth_mode instead of stored form state', () => {
    const apiKeyRequest = buildCodexProfileRequest(
      { ...createForm(), auth_mode: 'openai_api_key' },
      'gpt-5.4'
    )
    expect(apiKeyRequest.requires_openai_auth).toBe(true)
    expect(apiKeyRequest.openai_login_method).toBe('api')

    const noAuthRequest = buildCodexProfileRequest(
      { ...createForm(), auth_mode: 'no_auth' },
      'gpt-5.4'
    )
    expect(noAuthRequest.requires_openai_auth).toBe(false)
    expect(noAuthRequest.openai_login_method).toBeNull()
  })

  it('serializes env_key only in provider_env_key mode', () => {
    const envKeyForm: CodexProfileEditorForm = {
      ...createForm(),
      auth_mode: 'provider_env_key',
      env_key: 'MISTRAL_API_KEY',
    }

    expect(buildCodexProfileRequest(envKeyForm, 'gpt-5.4').env_key).toBe('MISTRAL_API_KEY')

    // 模式切走后旧 env_key 仍留在表单上，但绝不能再写回 profiles.toml
    for (const authMode of ['openai_api_key', 'no_auth', 'openai_chatgpt'] as const) {
      const request = buildCodexProfileRequest({ ...envKeyForm, auth_mode: authMode }, 'gpt-5.4')
      expect(request.env_key).toBeNull()
    }
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

  it('renders exactly three presets plus the custom action for new profiles', async () => {
    const { el, unmount } = await mountModal(createForm(), {
      editingName: null,
    })

    try {
      const select = el.querySelector<HTMLSelectElement>(
        '[data-testid="codex-profile-model-select"]'
      )
      const values = Array.from(select?.querySelectorAll('option') ?? []).map(
        (option) => option.value
      )

      expect(values).toEqual([...modelPresets, CUSTOM_MODEL_OPTION])
      expect(el.textContent).not.toContain('current profile value')
    } finally {
      unmount()
    }
  })

  it('labels a non-preset model as the current profile value while editing', async () => {
    const currentModel = 'gpt-5.4'
    const { el, unmount } = await mountModal(createForm(), {
      modelCatalog: [...modelPresets, currentModel],
      currentModelOption: currentModel,
      selectedModelOption: currentModel,
    })

    try {
      const select = el.querySelector<HTMLSelectElement>(
        '[data-testid="codex-profile-model-select"]'
      )
      const options = Array.from(select?.querySelectorAll('option') ?? [])

      expect(options.map((option) => option.value)).toEqual([
        ...modelPresets,
        currentModel,
        CUSTOM_MODEL_OPTION,
      ])
      expect(options.find((option) => option.value === currentModel)?.textContent).toContain(
        'current profile value'
      )
    } finally {
      unmount()
    }
  })

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
      const toggle = el.querySelector<HTMLButtonElement>(
        '[data-testid="codex-auth-token-visibility"]'
      )

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
      const copyButton = el.querySelector<HTMLButtonElement>(
        '[data-testid="codex-auth-token-copy"]'
      )
      copyButton?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await Promise.resolve()

      expect(helperMocks.copyText).toHaveBeenCalledWith('sk-test-123')
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
      const select = el.querySelector<HTMLSelectElement>(
        '[data-testid="codex-reasoning-effort-select"]'
      )
      const values = Array.from(select?.querySelectorAll('option') ?? []).map(
        (option) => option.value
      )

      expect(values).toEqual(['', ...REASONING_EFFORT_OPTIONS])
    } finally {
      unmount()
    }
  })

  it('blocks save behind a validation summary until the model is resolved', async () => {
    const onSave = vi.fn()
    const { el, unmount } = await mountModal(createForm(), { resolvedModel: '', onSave })

    try {
      const saveButton = Array.from(
        el.querySelectorAll<HTMLButtonElement>('.pe-footer button')
      ).find((button) => button.textContent?.includes('Save'))
      expect(saveButton).not.toBeUndefined()

      saveButton!.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await nextTick()

      expect(onSave).not.toHaveBeenCalled()
      expect(el.querySelector('.pe-summary')?.textContent).toContain('Model is required')
    } finally {
      unmount()
    }
  })

  it('emits save once every required field resolves', async () => {
    const onSave = vi.fn()
    const { el, unmount } = await mountModal(createForm(), { onSave })

    try {
      const saveButton = Array.from(
        el.querySelectorAll<HTMLButtonElement>('.pe-footer button')
      ).find((button) => button.textContent?.includes('Save'))

      saveButton!.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await nextTick()

      expect(onSave).toHaveBeenCalledTimes(1)
      expect(el.querySelector('.pe-summary')).toBeNull()
    } finally {
      unmount()
    }
  })

  it('drops the legacy --editor-* token shell in favour of the shared pe-* base', async () => {
    const { el, unmount } = await mountModal(createForm())

    try {
      expect(el.querySelector('.pe-modal')).not.toBeNull()
      expect(el.querySelector('.codex-profile-editor-modal')).toBeNull()
      expect(el.querySelector('.editor-input')).toBeNull()
      expect(el.querySelector('.editor-panel')).toBeNull()
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

      el.querySelector<HTMLButtonElement>(
        '[data-testid="provider-template-select"]'
      )?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      el.querySelector<HTMLButtonElement>(
        '[data-testid="provider-template-manual"]'
      )?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await nextTick()

      expect(onSelectTemplate).toHaveBeenCalledWith(
        expect.objectContaining({
          endpoint: 'https://openrouter.ai/api/v1',
        })
      )
      expect(onManualTemplate).toHaveBeenCalled()
    } finally {
      unmount()
    }
  })
})
