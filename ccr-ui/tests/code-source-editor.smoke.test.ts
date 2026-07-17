import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick, ref } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'

vi.mock('@/components/ui/SIcon.vue', () => ({
  default: defineComponent({
    setup() {
      return () => h('span')
    },
  }),
}))

import CodeSourceEditor from '@/components/editor/CodeSourceEditor.vue'

Object.defineProperty(Range.prototype, 'getClientRects', {
  configurable: true,
  value: () => [],
})
Object.defineProperty(Range.prototype, 'getBoundingClientRect', {
  configurable: true,
  value: () => new DOMRect(0, 0, 0, 0),
})

const i18n = createI18n({
  legacy: false,
  locale: 'en',
  missingWarn: false,
  fallbackWarn: false,
  messages: {
    en: {
      settingsRaw: {
        editorLoading: 'Loading editor...',
        editorLabel: 'Configuration source editor',
      },
    },
  },
})

const waitForEditorText = async (el: HTMLElement, text: string) => {
  await vi.waitFor(() => {
    expect(el.querySelector('.cm-editor')).not.toBeNull()
    expect(el.querySelector('.cm-content')?.textContent).toContain(text)
  }, { timeout: 2_000 })
  await nextTick()
}

afterEach(() => {
  document.body.innerHTML = ''
})

describe('CodeSourceEditor', () => {
  it('mounts CodeMirror, follows model updates, and renders an error marker', async () => {
    const value = ref('{"model":"first"}')
    const marker = ref<{ line: number; column: number; message: string } | null>(null)
    const el = document.createElement('div')
    document.body.appendChild(el)
    const app = createApp(defineComponent({
      setup() {
        return () => h(CodeSourceEditor, {
          modelValue: value.value,
          language: 'json',
          errorMarker: marker.value,
          'onUpdate:modelValue': (next: string) => { value.value = next },
        })
      },
    }))
    app.use(i18n)
    app.mount(el)

    try {
      await waitForEditorText(el, 'first')

      value.value = '{"model":"second"}'
      marker.value = { line: 1, column: 10, message: 'Invalid model value' }
      await nextTick()
      await waitForEditorText(el, 'second')

      expect(el.textContent).toContain('Invalid model value')
    } finally {
      app.unmount()
    }
  })
})
