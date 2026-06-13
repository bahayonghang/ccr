import { createPinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick, reactive } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { ClaudeProfileEditorForm } from '@/types/claudeProfileEditor'

const helperMocks = vi.hoisted(() => ({
  copyToClipboard: vi.fn(),
}))

vi.mock('@/utils/codexHelpers', () => ({
  copyToClipboard: helperMocks.copyToClipboard,
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

import ClaudeProfileEditorSections from '@/components/claude/ClaudeProfileEditorSections.vue'
import { useUIStore } from '@/stores/ui'

const createForm = (): ClaudeProfileEditorForm => ({
  name: 'mimo',
  description: 'Primary profile',
  auth_mode: 'api_key',
  base_url: 'https://api.anthropic.com',
  auth_token: 'sk-ant-test-123',
  default_opus_model: 'claude-opus-4-5',
  default_sonnet_model: 'claude-sonnet-4-5',
  default_haiku_model: 'claude-haiku-4-5',
  subagent_model: '',
  effort_level: '',
  provider: 'anthropic',
  provider_type: 'official',
  account: 'work',
  tagsInput: 'free',
  enabled: true,
})

const i18nMessages = {
  en: {
    claudeProfiles: {
      operationFailed: 'Operation failed',
      nameLabel: 'Name',
      namePlaceholder: 'name',
      nameHelper: 'Name helper',
      readonlyNameHint: 'Readonly name',
      renameWarningHint: 'Rename warning',
      descLabel: 'Description',
      descPlaceholder: 'description',
      descriptionHelper: 'Description helper',
      baseUrlLabel: 'Base URL',
      baseUrlPlaceholder: 'https://example.com',
      baseUrlHelper: 'Base URL helper',
      modelLabel: 'Model',
      modelPlaceholder: 'model',
      modelHelper: 'Model helper',
      smallFastModelLabel: 'Small Fast Model',
      smallFastModelPlaceholder: 'fast model',
      smallFastModelHelper: 'Fast model helper',
      providerLabel: 'Provider',
      providerPlaceholder: 'provider',
      providerHelper: 'Provider helper',
      advancedModelsTitle: 'Advanced models',
      advancedModelsDescription: 'Advanced model description',
      defaultOpusModelLabel: 'Opus model',
      defaultOpusModelPlaceholder: 'opus model',
      defaultOpusModelHelper: 'Opus model helper',
      defaultSonnetModelLabel: 'Sonnet model',
      defaultSonnetModelPlaceholder: 'sonnet model',
      defaultSonnetModelHelper: 'Sonnet model helper',
      defaultHaikuModelLabel: 'Haiku model',
      defaultHaikuModelPlaceholder: 'haiku model',
      defaultHaikuModelHelper: 'Haiku model helper',
      subagentModelLabel: 'Subagent model',
      subagentModelPlaceholder: 'subagent model',
      subagentModelHelper: 'Subagent model helper',
      effortLevelLabel: 'Effort level',
      effortLevelHelper: 'Effort helper',
      effortLevelOptionDefault: 'Default',
      effortLevelOptionLow: 'Low',
      effortLevelOptionMedium: 'Medium',
      effortLevelOptionHigh: 'High',
      effortLevelOptionXhigh: 'XHigh',
      effortLevelOptionMax: 'Max',
      authModeLabel: 'Auth mode',
      authModeHelper: 'Auth mode helper',
      authModeOptionSubscription: 'Subscription',
      authModeOptionApiKey: 'API key',
      accountLabel: 'Account',
      accountPlaceholder: 'account',
      accountHelper: 'Account helper',
      providerTypeLabel: 'Provider Type',
      providerTypePlaceholder: 'provider type',
      providerTypeHelper: 'Provider type helper',
      authTokenLabel: 'Auth Token',
      authTokenPlaceholder: 'token',
      authTokenHelper: 'Token helper',
      authTokenActions: {
        show: 'Show Auth Token',
        hide: 'Hide Auth Token',
        copy: 'Copy Auth Token',
      },
      authTokenCopied: 'Auth token copied',
      authTokenCopyFailed: 'Failed to copy auth token',
      tagsLabel: 'Tags',
      tagsPlaceholder: 'tags',
      tagsHelper: 'Tags helper',
      enabledProfile: 'Enabled',
      enabledHelper: 'Enabled helper',
      sections: {
        basic: {
          title: 'Basic',
          description: 'Basic section',
        },
        connection: {
          title: 'Connection',
          description: 'Connection section',
        },
        auth: {
          title: 'Auth',
          description: 'Auth section',
        },
        status: {
          title: 'Status',
          description: 'Status section',
        },
      },
    },
  },
}

const mountSections = async (form = createForm()) => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const state = reactive(form)
  const pinia = createPinia()
  const i18n = createI18n({
    legacy: false,
    locale: 'en',
    messages: i18nMessages,
  })
  const registerModalSectionRef = vi.fn()

  const app = createApp(
    defineComponent({
      setup() {
        return () =>
          h(ClaudeProfileEditorSections, {
            editingName: state.name,
            form: state,
            isEditing: true,
            monospaceFieldClass: 'editor-input editor-input--mono w-full',
            parsedFormTags: ['free'],
            registerModalSectionRef,
            saveError: null,
            textareaClass: 'editor-input w-full',
            textFieldClass: 'editor-input w-full',
            updateFormField: (field: keyof ClaudeProfileEditorForm, value: string | boolean) => {
              if (field === 'enabled') {
                state.enabled = Boolean(value)
                return
              }

              const target = state as Record<string, string | boolean>
              target[field] = String(value)
            },
          })
      },
    })
  )

  app.use(pinia)
  app.use(i18n)
  app.mount(el)
  await nextTick()

  return {
    el,
    form: state,
    uiStore: useUIStore(),
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

describe('ClaudeProfileEditorSections token controls', () => {
  it('masks the auth token by default and toggles visibility', async () => {
    const { el, unmount } = await mountSections()

    try {
      const input = el.querySelector<HTMLInputElement>('[data-testid="claude-auth-token-input"]')
      const toggle = el.querySelector<HTMLButtonElement>(
        '[data-testid="claude-auth-token-visibility"]'
      )

      expect(input?.type).toBe('password')
      expect(toggle?.title).toBe('Show Auth Token')

      toggle?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await nextTick()

      expect(input?.type).toBe('text')
      expect(toggle?.title).toBe('Hide Auth Token')
    } finally {
      unmount()
    }
  })

  it('copies the real auth token value', async () => {
    const { el, uiStore, unmount } = await mountSections()

    try {
      const copyButton = el.querySelector<HTMLButtonElement>(
        '[data-testid="claude-auth-token-copy"]'
      )
      copyButton?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await Promise.resolve()

      expect(helperMocks.copyToClipboard).toHaveBeenCalledWith('sk-ant-test-123')
      expect(uiStore.toasts.at(-1)?.message).toBe('Auth token copied')
    } finally {
      unmount()
    }
  })
})
