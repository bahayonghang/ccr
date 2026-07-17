import { createPinia, setActivePinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('vue-router', () => ({
  onBeforeRouteLeave: vi.fn(),
}))

vi.mock('@/components/editor/CodeSourceEditor.vue', () => ({
  default: defineComponent({
    props: {
      modelValue: { type: String, required: true },
      language: { type: String, required: true },
      errorMarker: { type: Object, default: null },
    },
    emits: ['update:modelValue', 'save'],
    setup(props, { emit }) {
      return () => h('textarea', {
        value: props.modelValue,
        'data-error': (props.errorMarker as { message?: string } | null)?.message ?? '',
        onInput: (event: Event) => {
          emit('update:modelValue', (event.target as HTMLTextAreaElement).value)
        },
      })
    },
  }),
}))

vi.mock('@/components/ui/SIcon.vue', () => ({
  default: defineComponent({
    setup() {
      return () => h('span')
    },
  }),
}))

import ConfigSourcePanel from '@/components/editor/ConfigSourcePanel.vue'
import { useUIStore } from '@/stores/ui'

const i18n = createI18n({
  legacy: false,
  locale: 'en',
  missingWarn: false,
  fallbackWarn: false,
  messages: { en: {} },
})

const settle = async () => {
  await Promise.resolve()
  await nextTick()
  await Promise.resolve()
  await nextTick()
}

let pinia: ReturnType<typeof createPinia>

beforeEach(() => {
  pinia = createPinia()
  setActivePinia(pinia)
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('ConfigSourcePanel', () => {
  it('loads verbatim content and saves edits with the read token', async () => {
    const getRaw = vi.fn().mockResolvedValue({
      status: 'ok',
      content: 'model = "first"',
      token: 'token-v1',
      path: 'C:/Users/test/.codex/config.toml',
      exists: true,
    })
    const saveRaw = vi.fn().mockResolvedValue({ status: 'saved', token: 'token-v2' })
    const listLayers = vi.fn().mockResolvedValue({
      layers: [{
        id: 'user',
        label: 'User',
        path: 'C:/Users/test/.codex/config.toml',
        exists: true,
        size: 15,
        mtime: 1,
        editable: true,
      }],
    })
    const uiStore = useUIStore()
    vi.spyOn(uiStore, 'requestConfirm').mockResolvedValue(true)
    vi.spyOn(uiStore, 'showSuccess').mockImplementation(() => 1)

    const el = document.createElement('div')
    document.body.appendChild(el)
    const app = createApp(defineComponent({
      setup() {
        return () => h(ConfigSourcePanel, {
          language: 'toml',
          getRaw,
          saveRaw,
          listLayers,
        })
      },
    }))
    app.use(i18n)
    app.use(pinia)
    app.mount(el)

    try {
      await settle()
      const editor = el.querySelector<HTMLTextAreaElement>('textarea')
      expect(editor?.value).toBe('model = "first"')

      editor!.value = 'model = "second"'
      editor!.dispatchEvent(new Event('input', { bubbles: true }))
      await nextTick()
      const saveButton = Array.from(el.querySelectorAll('button')).find(
        button => button.textContent?.includes('settingsRaw.save'),
      )
      saveButton?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await settle()

      expect(saveRaw).toHaveBeenCalledWith('model = "second"', 'token-v1')
      expect(el.textContent).toContain('C:/Users/test/.codex/config.toml')
    } finally {
      app.unmount()
    }
  })
})
