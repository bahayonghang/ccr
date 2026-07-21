import { createPinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick } from 'vue'
import { createMemoryHistory, createRouter, RouterView } from 'vue-router'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import enUS from '@/i18n/locales/en-US'

const apiMocks = vi.hoisted(() => ({
  getCurrentEnvironment: vi.fn(),
  listSystemPrompts: vi.fn(),
  getSystemPrompt: vi.fn(),
  saveSystemPrompt: vi.fn(),
  createSystemPrompt: vi.fn(),
}))

vi.mock('@/api', () => ({
  getCurrentEnvironment: apiMocks.getCurrentEnvironment,
  systemPromptsApi: {
    listSystemPrompts: apiMocks.listSystemPrompts,
    getSystemPrompt: apiMocks.getSystemPrompt,
    saveSystemPrompt: apiMocks.saveSystemPrompt,
    createSystemPrompt: apiMocks.createSystemPrompt,
  },
}))

vi.mock('@/components/ModuleSubnav.vue', async () => {
  const { defineComponent, h } = await import('vue')
  return {
    default: defineComponent({
      setup: () => () => h('nav', { 'data-subnav': 'true' }),
    }),
  }
})

vi.mock('@/components/editor/CodeSourceEditor.vue', async () => {
  const { defineComponent, h } = await import('vue')
  return {
    default: defineComponent({
      props: {
        modelValue: { type: String, required: true },
      },
      emits: ['update:modelValue', 'save'],
      setup(props, { emit }) {
        return () => h('div', [
          h('textarea', {
            'data-editor': 'true',
            value: props.modelValue,
            onInput: (event: Event) => emit(
              'update:modelValue',
              (event.target as HTMLTextAreaElement).value,
            ),
          }),
          h('button', {
            'data-editor-save': 'true',
            onClick: () => emit('save'),
          }, 'Editor save'),
        ])
      },
    }),
  }
})

import SystemPromptsView from '@/views/generic/SystemPromptsView.vue'

const i18n = createI18n({
  legacy: false,
  locale: 'en-US',
  messages: { 'en-US': enUS },
})

async function flushView() {
  await Promise.resolve()
  await nextTick()
  await Promise.resolve()
  await nextTick()
}

async function mountView() {
  const el = document.createElement('div')
  document.body.appendChild(el)
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{
      path: '/',
      component: defineComponent({
        setup: () => () => h(SystemPromptsView, { platform: 'codex' }),
      }),
    }],
  })
  const app = createApp(defineComponent({
    setup: () => () => h(RouterView),
  }))
  app.use(createPinia())
  app.use(i18n)
  app.use(router)
  await router.push('/')
  await router.isReady()
  app.mount(el)
  await flushView()
  return { el, unmount: () => app.unmount() }
}

beforeEach(() => {
  for (const mock of Object.values(apiMocks)) mock.mockReset()
  apiMocks.getCurrentEnvironment.mockResolvedValue({ env_type: 'local' })
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('SystemPromptsView', () => {
  it('saves edited markdown with the token returned by the read command', async () => {
    const file = {
      id: 'codex-agents',
      labelKey: 'systemPrompts.files.codexAgents',
      path: 'C:/Users/test/.codex/AGENTS.md',
      exists: true,
      size: 6,
      mtime: 1,
      editable: true,
      limitHint: 32768,
    }
    apiMocks.listSystemPrompts.mockResolvedValue({ status: 'ok', files: [file], rules: [] })
    apiMocks.getSystemPrompt.mockResolvedValue({
      status: 'ok',
      content: '# Old\n',
      token: 'token-before',
      path: file.path,
      exists: true,
      limitHint: 32768,
    })
    apiMocks.saveSystemPrompt.mockResolvedValue({ status: 'saved', token: 'token-after' })

    const { el, unmount } = await mountView()
    try {
      const editor = el.querySelector<HTMLTextAreaElement>('[data-editor]')
      expect(editor?.value).toBe('# Old\n')
      editor!.value = '# Updated\n'
      editor!.dispatchEvent(new Event('input', { bubbles: true }))
      await nextTick()
      el.querySelector<HTMLButtonElement>('[data-editor-save]')?.click()
      await flushView()

      expect(apiMocks.saveSystemPrompt).toHaveBeenCalledWith(
        'codex',
        'codex-agents',
        '# Updated\n',
        'token-before',
      )
      expect(apiMocks.listSystemPrompts).toHaveBeenCalledTimes(2)
      expect(el.textContent).not.toContain('Unsaved changes')
    } finally {
      unmount()
    }
  })

  it('creates a missing file and immediately loads it into the editor', async () => {
    const missing = {
      id: 'codex-agents',
      labelKey: 'systemPrompts.files.codexAgents',
      path: 'C:/Users/test/.codex/AGENTS.md',
      exists: false,
      size: null,
      mtime: null,
      editable: true,
      limitHint: 32768,
    }
    const created = { ...missing, exists: true, size: 0 }
    apiMocks.listSystemPrompts
      .mockResolvedValueOnce({ status: 'ok', files: [missing], rules: [] })
      .mockResolvedValueOnce({ status: 'ok', files: [created], rules: [] })
    apiMocks.createSystemPrompt.mockResolvedValue({ status: 'saved', token: 'created-token' })
    apiMocks.getSystemPrompt.mockResolvedValue({
      status: 'ok',
      content: '',
      token: 'created-token',
      path: created.path,
      exists: true,
      limitHint: 32768,
    })

    const { el, unmount } = await mountView()
    try {
      const createButton = Array.from(el.querySelectorAll('button')).find(
        button => button.textContent?.includes('Create file'),
      )
      createButton?.click()
      await flushView()

      expect(apiMocks.createSystemPrompt).toHaveBeenCalledWith('codex', 'codex-agents')
      expect(apiMocks.getSystemPrompt).toHaveBeenCalledWith('codex', 'codex-agents')
      expect(el.querySelector('[data-editor]')).not.toBeNull()
    } finally {
      unmount()
    }
  })
})
