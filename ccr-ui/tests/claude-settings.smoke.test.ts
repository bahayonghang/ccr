import { createPinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const apiMocks = vi.hoisted(() => ({
  getClaudeSettings: vi.fn(),
  updateClaudeSettings: vi.fn(),
}))

vi.mock('@/api', () => ({
  getClaudeSettings: apiMocks.getClaudeSettings,
  updateClaudeSettings: apiMocks.updateClaudeSettings,
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

import ClaudeCodeSettingsView from '@/views/ClaudeCodeSettingsView.vue'

const i18n = createI18n({
  legacy: false,
  locale: 'en',
  missingWarn: false,
  fallbackWarn: false,
  messages: {
    en: {
      claudeSettings: {
        title: 'Claude Code Settings',
        back: 'Back',
        save: 'Save',
        saving: 'Saving',
        saveSuccess: 'Saved',
        tabs: {
          model: 'Model',
          permissions: 'Permissions',
          env: 'Environment Variables',
          ui: 'UI',
          sandbox: 'Sandbox',
          git: 'Git',
        },
        env: {
          add: 'Add',
          empty: 'No environment variables yet. Click above to add one.',
        },
      },
    },
  },
})

const mountView = async () => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () => h(ClaudeCodeSettingsView)
    },
  }))

  app.use(createPinia())
  app.use(i18n)
  app.component(
    'RouterLink',
    defineComponent({
      props: {
        to: { type: [String, Object], required: true },
      },
      setup(_props, { slots }) {
        return () => h('a', {}, slots.default?.())
      },
    }),
  )

  app.mount(el)
  await Promise.resolve()
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
  apiMocks.getClaudeSettings.mockReset()
  apiMocks.updateClaudeSettings.mockReset()
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('ClaudeCodeSettingsView smoke', () => {
  it('renders env entries returned from Claude settings', async () => {
    apiMocks.getClaudeSettings.mockResolvedValue({
      model: 'opus',
      env: {
        ANTHROPIC_BASE_URL: 'https://example.com',
        MCP_TIMEOUT: '30000',
      },
    })

    const { el, unmount } = await mountView()

    try {
      const envTab = Array.from(el.querySelectorAll('button')).find(
        button => button.textContent?.includes('Environment Variables'),
      )
      envTab?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await nextTick()

      const keyInputs = Array.from(
        el.querySelectorAll<HTMLInputElement>('input[placeholder="KEY"]'),
      )
      const valueInputs = Array.from(
        el.querySelectorAll<HTMLInputElement>('input[placeholder="value"]'),
      )

      expect(keyInputs.map(input => input.value)).toEqual([
        'ANTHROPIC_BASE_URL',
        'MCP_TIMEOUT',
      ])
      expect(valueInputs.map(input => input.value)).toEqual([
        'https://example.com',
        '30000',
      ])
      expect(el.textContent).not.toContain('No environment variables yet')
    } finally {
      unmount()
    }
  })

  it('shows the empty state only when env is absent', async () => {
    apiMocks.getClaudeSettings.mockResolvedValue({
      model: 'sonnet',
      env: {},
    })

    const { el, unmount } = await mountView()

    try {
      const envTab = Array.from(el.querySelectorAll('button')).find(
        button => button.textContent?.includes('Environment Variables'),
      )
      envTab?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await nextTick()

      expect(el.textContent).toContain('No environment variables yet. Click above to add one.')
      expect(el.querySelectorAll('input[placeholder="KEY"]')).toHaveLength(0)
    } finally {
      unmount()
    }
  })

  it('submits env updates even when settings contain official object hooks', async () => {
    apiMocks.getClaudeSettings.mockResolvedValue({
      model: 'opus',
      env: {
        ANTHROPIC_BASE_URL: 'https://example.com',
      },
      hooks: {
        PreToolUse: [
          {
            matcher: 'Bash',
            hooks: [
              {
                type: 'command',
                command: './security-check.sh',
              },
            ],
          },
        ],
      },
    })
    apiMocks.updateClaudeSettings.mockResolvedValue({})

    const { el, unmount } = await mountView()

    try {
      const envTab = Array.from(el.querySelectorAll('button')).find(
        button => button.textContent?.includes('Environment Variables'),
      )
      envTab?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await nextTick()

      const valueInput = el.querySelector<HTMLInputElement>('input[placeholder="value"]')
      expect(valueInput).not.toBeNull()

      valueInput!.value = 'https://updated.example.com'
      valueInput!.dispatchEvent(new Event('input', { bubbles: true }))
      await nextTick()

      const saveButton = Array.from(el.querySelectorAll('button')).find(
        button => button.textContent?.includes('Save'),
      )
      saveButton?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await Promise.resolve()
      await nextTick()

      expect(apiMocks.updateClaudeSettings).toHaveBeenCalledTimes(1)
      expect(apiMocks.updateClaudeSettings).toHaveBeenCalledWith(
        expect.objectContaining({
          model: 'opus',
          env: {
            ANTHROPIC_BASE_URL: 'https://updated.example.com',
          },
        }),
      )
    } finally {
      unmount()
    }
  })
})
