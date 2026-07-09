import { createPinia } from 'pinia'
import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createI18nStub } from './helpers/i18n-stub'

const apiMocks = vi.hoisted(() => ({
  listHooks: vi.fn(),
  updateHooks: vi.fn(),
}))

vi.mock('@/api', () => ({
  listHooks: apiMocks.listHooks,
  updateHooks: apiMocks.updateHooks,
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

import HooksView from '@/views/HooksView.vue'

const mountView = async () => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () => h(HooksView)
    },
  }))

  app.use(createPinia())
  app.use(createI18nStub('en-US'))
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
  apiMocks.listHooks.mockReset()
  apiMocks.updateHooks.mockReset()
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('HooksView smoke', () => {
  it('renders official grouped hooks by event', async () => {
    apiMocks.listHooks.mockResolvedValue({
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
      UserPromptSubmit: [
        {
          hooks: [
            {
              type: 'prompt',
              prompt: 'Review the prompt before sending',
            },
          ],
        },
      ],
    })
    apiMocks.updateHooks.mockImplementation(async (payload) => payload)

    const { el, unmount } = await mountView()

    try {
      expect(el.textContent).toContain('PreToolUse')
      expect(el.textContent).toContain('UserPromptSubmit')
      expect(el.textContent).toContain('./security-check.sh')
      expect(el.textContent).toContain('Review the prompt before sending')
    } finally {
      unmount()
    }
  })

  it('submits a new hook group using the canonical grouped payload', async () => {
    apiMocks.listHooks.mockResolvedValue({
      PreToolUse: [],
    })
    apiMocks.updateHooks.mockImplementation(async (payload) => payload)

    const { el, unmount } = await mountView()

    try {
      const addButton = Array.from(el.querySelectorAll('button')).find(
        button => button.textContent?.includes('Add Hook Group'),
      )
      addButton?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await nextTick()

      const eventInput = document.body.querySelector<HTMLInputElement>('input[list="known-hook-events"]')
      const commandInput = document.body.querySelector<HTMLInputElement>('input[placeholder="./scripts/check-style.sh"]')

      expect(eventInput).not.toBeNull()
      expect(commandInput).not.toBeNull()

      eventInput!.value = 'Notification'
      eventInput!.dispatchEvent(new Event('input', { bubbles: true }))
      commandInput!.value = './notify.sh'
      commandInput!.dispatchEvent(new Event('input', { bubbles: true }))
      await nextTick()

      const saveButtons = Array.from(document.body.querySelectorAll('button')).filter(
        button => button.textContent?.includes('Add Group'),
      )
      saveButtons.at(-1)?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await Promise.resolve()
      await nextTick()

      expect(apiMocks.updateHooks).toHaveBeenCalledTimes(1)
      expect(apiMocks.updateHooks).toHaveBeenCalledWith({
        PreToolUse: [],
        Notification: [
          {
            hooks: [
              {
                type: 'command',
                command: './notify.sh',
              },
            ],
          },
        ],
      })
    } finally {
      unmount()
    }
  })
})
