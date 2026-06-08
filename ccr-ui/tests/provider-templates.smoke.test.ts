import { createApp, defineComponent, h, nextTick, reactive } from 'vue'
import { describe, expect, it, beforeEach, afterEach, vi } from 'vitest'
import { BUILT_IN_PROVIDER_TEMPLATES } from '@/configs/providerTemplates'
import ProviderTemplateSelector from '@/components/provider-templates/ProviderTemplateSelector.vue'
import type { ProviderTemplateDraftContext, ProviderTemplateSelection } from '@/types/providerTemplates'
import {
  buildProviderTemplateOptions,
  createCustomProviderTemplateFromDraft,
  CUSTOM_PROVIDER_TEMPLATES_STORAGE_KEY,
  mapTemplateToClaudeProfilePatch,
  mapTemplateToCodexProviderPatch,
  mapTemplateToOpenCodeProviderPatch,
  readCustomProviderTemplates,
  upsertCustomProviderTemplate,
  writeCustomProviderTemplates,
} from '@/utils/providerTemplates'

vi.mock('@/components/ui/SIcon.vue', () => ({
  default: defineComponent({
    props: {
      name: { type: String, required: true },
      size: { type: String, default: '' },
    },
    setup(props) {
      return () => h('span', {
        'data-icon': props.name,
        class: props.size,
      })
    },
  }),
}))

const flush = async () => {
  await Promise.resolve()
  await nextTick()
  await Promise.resolve()
  await nextTick()
}

const dispatchKey = (element: Element, key: string) => {
  element.dispatchEvent(new KeyboardEvent('keydown', {
    key,
    bubbles: true,
    cancelable: true,
  }))
}

beforeEach(() => {
  Element.prototype.scrollIntoView = vi.fn()
})

afterEach(() => {
  document.body.innerHTML = ''
  document.body.style.overflow = ''
  localStorage.clear()
})

describe('provider template mapping', () => {
  it('filters templates by platform override and indexes name, host, model, and aliases', () => {
    const claudeOptions = buildProviderTemplateOptions(BUILT_IN_PROVIDER_TEMPLATES, 'claude')
    const codexOptions = buildProviderTemplateOptions(BUILT_IN_PROVIDER_TEMPLATES, 'codex')
    const opencodeOptions = buildProviderTemplateOptions(BUILT_IN_PROVIDER_TEMPLATES, 'opencode')

    expect(claudeOptions.some(option => option.template.id === 'deepseek')).toBe(true)
    expect(opencodeOptions.some(option => option.template.id === 'deepseek')).toBe(false)
    expect(codexOptions.some(option => option.template.id === 'deepseek')).toBe(true)
    expect(opencodeOptions.some(option => option.template.id === 'openai')).toBe(true)

    const openrouter = codexOptions.find(option => option.template.id === 'openrouter')
    expect(openrouter?.searchText).toContain('openrouter')
    expect(openrouter?.searchText).toContain('openrouter.ai')
    expect(openrouter?.searchText).toContain('anthropic/claude-sonnet-4.6')
  })

  it('maps templates to non-secret platform fields only', () => {
    const deepseek = BUILT_IN_PROVIDER_TEMPLATES.find(template => template.id === 'deepseek')
    const openrouter = BUILT_IN_PROVIDER_TEMPLATES.find(template => template.id === 'openrouter')
    const local = BUILT_IN_PROVIDER_TEMPLATES.find(template => template.id === 'local-openai-compatible')

    expect(deepseek).toBeTruthy()
    expect(openrouter).toBeTruthy()
    expect(local).toBeTruthy()

    const claudePatch = mapTemplateToClaudeProfilePatch(deepseek!)
    expect(claudePatch.base_url).toBe('https://api.deepseek.com/anthropic')
    expect(claudePatch.provider).toBe('DeepSeek')
    expect(JSON.stringify(claudePatch)).not.toMatch(/auth_token|apiKey|api_key/i)

    const codexPatch = mapTemplateToCodexProviderPatch(openrouter!)
    expect(codexPatch.baseUrl).toBe('https://openrouter.ai/api/v1')
    expect(codexPatch.apiKeyUrl).toBe('https://openrouter.ai/keys')
    expect(JSON.stringify(codexPatch)).not.toMatch(/apiKey":|api_key":|auth_token/i)

    const opencodePatch = mapTemplateToOpenCodeProviderPatch(local!)
    expect(opencodePatch.id).toBe('openai')
    expect(opencodePatch.npm).toBe('@ai-sdk/openai-compatible')
    expect(opencodePatch.baseURL).toBe('http://127.0.0.1:11434/v1')
    expect(JSON.stringify(opencodePatch)).not.toMatch(/apiKey|authToken|secret|password/i)
  })

  it('persists custom templates separately from saved providers and strips secret fields across platform overrides', () => {
    const [next] = upsertCustomProviderTemplate([], {
      id: 'custom-router',
      name: 'Custom Router',
      category: 'third_party',
      source: 'custom',
      platforms: {
        claude: {
          baseUrl: 'https://router.example.com/anthropic',
          authToken: 'should-not-save',
        },
        codex: {
          baseUrl: 'https://router.example.com/v1',
          apiKey: 'should-not-save',
        },
        opencode: {
          id: 'openai',
          name: 'Custom Router',
          npm: '@ai-sdk/openai-compatible',
          baseURL: 'https://router.example.com/v1',
          extraOptions: {
            apiKey: 'should-not-save',
            timeout: 600000,
          },
          rootExtra: {
            headers: {
              secret: 'should-not-save',
              Authorization: 'Bearer should-not-save',
              'x-api-key': 'should-not-save',
              xProvider: 'router',
            },
          },
        },
      },
    } as never)

    writeCustomProviderTemplates([next])
    const raw = localStorage.getItem(CUSTOM_PROVIDER_TEMPLATES_STORAGE_KEY)
    expect(raw).toBeTruthy()
    expect(raw).not.toContain('should-not-save')

    const loaded = readCustomProviderTemplates()
    expect(loaded).toHaveLength(1)
    expect(JSON.stringify(loaded[0].platforms.claude)).not.toMatch(/authToken|should-not-save/i)
    expect(JSON.stringify(loaded[0].platforms.codex)).not.toMatch(/apiKey|should-not-save/i)
    expect(loaded[0].platforms.opencode?.extraOptions).toEqual({ timeout: 600000 })
    expect(loaded[0].platforms.opencode?.rootExtra).toEqual({ headers: { xProvider: 'router' } })
  })

  it('creates global custom templates with explicit platform overrides', () => {
    const template = createCustomProviderTemplateFromDraft({
      platform: 'codex',
      defaultName: 'Gateway',
      category: 'third_party',
      baseUrls: ['https://gateway.example.com/v1'],
      platformOverride: {
        baseUrl: 'https://gateway.example.com/v1',
      },
    }, ['codex', 'opencode'], {
      id: 'gateway',
      name: 'Gateway',
      category: 'third_party',
      baseUrls: ['https://gateway.example.com/v1'],
      platformOverrides: {
        codex: {
          baseUrl: 'https://codex.gateway.example.com/v1',
          websiteUrl: 'https://gateway.example.com',
          apiKey: 'should-not-save',
        } as never,
        opencode: {
          id: 'openai',
          name: 'Gateway OpenCode',
          npm: '@ai-sdk/openai-compatible',
          baseURL: 'https://opencode.gateway.example.com/v1',
          extraOptions: {
            apiKey: 'should-not-save',
            timeout: 600000,
          },
        },
      },
    })

    expect(template.platforms.codex?.baseUrl).toBe('https://codex.gateway.example.com/v1')
    expect(template.platforms.opencode?.baseURL).toBe('https://opencode.gateway.example.com/v1')
    expect(template.platforms.opencode?.extraOptions).toEqual({ timeout: 600000 })
    expect(JSON.stringify(template)).not.toMatch(/apiKey|should-not-save/i)
  })
})

describe('ProviderTemplateSelector smoke', () => {
  const mountSelector = async () => {
    const el = document.createElement('div')
    document.body.appendChild(el)
    const state = reactive({
      selectedTemplateId: null as string | null,
      selectedEndpoint: '',
    })
    const events = {
      select: vi.fn((selection: ProviderTemplateSelection) => {
        state.selectedTemplateId = selection.template.id
        state.selectedEndpoint = selection.endpoint || ''
      }),
      manual: vi.fn(),
    }
    const draft: ProviderTemplateDraftContext = {
      platform: 'codex',
      defaultName: 'Custom Gateway',
      name: 'Custom Gateway',
      category: 'third_party',
      baseUrls: ['https://gateway.example.com/v1'],
      platformOverride: {
        baseUrl: 'https://gateway.example.com/v1',
        websiteUrl: 'https://gateway.example.com',
        apiKeyUrl: 'https://gateway.example.com/keys',
      },
    }

    const app = createApp(defineComponent({
      setup() {
        return () => h(ProviderTemplateSelector, {
          platform: 'codex',
          selectedTemplateId: state.selectedTemplateId,
          selectedEndpoint: state.selectedEndpoint,
          draftContext: draft,
          onSelect: events.select,
          onManual: events.manual,
        })
      },
    }))

    app.mount(el)
    await flush()

    return {
      events,
      open: async () => {
        document.body.querySelector<HTMLElement>('[data-testid="provider-template-trigger"]')?.click()
        await flush()
      },
      unmount: () => {
        app.unmount()
        el.remove()
      },
    }
  }

  it('searches templates and applies the active result from the keyboard', async () => {
    const selector = await mountSelector()

    try {
      await selector.open()
      const input = document.body.querySelector<HTMLInputElement>('[data-testid="provider-template-search"]')
      expect(input).toBeTruthy()

      input!.value = 'openrouter'
      input!.dispatchEvent(new Event('input', { bubbles: true }))
      await flush()

      expect(document.body.textContent).toContain('OpenRouter')
      dispatchKey(input!, 'ArrowDown')
      dispatchKey(input!, 'Enter')
      await flush()

      expect(selector.events.select).toHaveBeenCalled()
      expect(selector.events.select.mock.calls[0][0].template.id).toBe('openrouter')
      expect(document.body.querySelector('[data-testid="provider-template-selected-summary"]')?.textContent)
        .toContain('OpenRouter')
    } finally {
      selector.unmount()
    }
  })

  it('saves a custom template from the current non-secret draft', async () => {
    const selector = await mountSelector()

    try {
      await selector.open()
      document.body.querySelector<HTMLElement>('[data-testid="provider-template-save-current"]')?.click()
      await flush()

      const nameInput = document.body.querySelector<HTMLInputElement>('[data-testid="provider-template-custom-name"]')
      expect(nameInput?.value).toBe('Custom Gateway')

      document.body.querySelector<HTMLElement>('[data-testid="provider-template-save-custom"]')?.click()
      await flush()

      expect(localStorage.getItem(CUSTOM_PROVIDER_TEMPLATES_STORAGE_KEY)).toContain('Custom Gateway')
      expect(localStorage.getItem(CUSTOM_PROVIDER_TEMPLATES_STORAGE_KEY)).not.toMatch(/"apiKey"|"authToken"|"secret"/)
    } finally {
      selector.unmount()
    }
  })

  it('saves selected platform override JSON for custom templates', async () => {
    const selector = await mountSelector()

    try {
      await selector.open()
      document.body.querySelector<HTMLElement>('[data-testid="provider-template-save-current"]')?.click()
      await flush()

      const opencodePlatform = document.body.querySelector<HTMLInputElement>('[data-testid="provider-template-platform-opencode"]')
      expect(opencodePlatform).toBeTruthy()
      opencodePlatform!.click()
      await flush()

      const codexOverride = document.body.querySelector<HTMLTextAreaElement>('[data-testid="provider-template-platform-override-codex"]')
      const opencodeOverride = document.body.querySelector<HTMLTextAreaElement>('[data-testid="provider-template-platform-override-opencode"]')
      expect(codexOverride).toBeTruthy()
      expect(opencodeOverride).toBeTruthy()

      codexOverride!.value = JSON.stringify({
        baseUrl: 'https://codex.gateway.example.com/v1',
        websiteUrl: 'https://gateway.example.com',
        apiKey: 'should-not-save',
      }, null, 2)
      codexOverride!.dispatchEvent(new Event('input', { bubbles: true }))

      opencodeOverride!.value = JSON.stringify({
        id: 'openai',
        name: 'Gateway OpenCode',
        npm: '@ai-sdk/openai-compatible',
        baseURL: 'https://opencode.gateway.example.com/v1',
        extraOptions: {
          apiKey: 'should-not-save',
          timeout: 600000,
        },
      }, null, 2)
      opencodeOverride!.dispatchEvent(new Event('input', { bubbles: true }))
      await flush()

      document.body.querySelector<HTMLElement>('[data-testid="provider-template-save-custom"]')?.click()
      await flush()

      const loaded = readCustomProviderTemplates()
      const saved = loaded.find(template => template.name === 'Custom Gateway')
      expect(saved?.platforms.codex?.baseUrl).toBe('https://codex.gateway.example.com/v1')
      expect(saved?.platforms.opencode?.baseURL).toBe('https://opencode.gateway.example.com/v1')
      expect(saved?.platforms.opencode?.extraOptions).toEqual({ timeout: 600000 })
      expect(localStorage.getItem(CUSTOM_PROVIDER_TEMPLATES_STORAGE_KEY)).not.toContain('should-not-save')
    } finally {
      selector.unmount()
    }
  })

  it('shows an error and does not save when platform override JSON is invalid', async () => {
    const selector = await mountSelector()

    try {
      await selector.open()
      document.body.querySelector<HTMLElement>('[data-testid="provider-template-save-current"]')?.click()
      await flush()

      const codexOverride = document.body.querySelector<HTMLTextAreaElement>('[data-testid="provider-template-platform-override-codex"]')
      expect(codexOverride).toBeTruthy()

      codexOverride!.value = '{ invalid json'
      codexOverride!.dispatchEvent(new Event('input', { bubbles: true }))
      await flush()

      document.body.querySelector<HTMLElement>('[data-testid="provider-template-save-custom"]')?.click()
      await flush()

      expect(document.body.textContent).toContain('Codex override JSON is invalid.')
      expect(localStorage.getItem(CUSTOM_PROVIDER_TEMPLATES_STORAGE_KEY)).toBeNull()
    } finally {
      selector.unmount()
    }
  })
})
