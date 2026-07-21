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

const editorThemeRules = () => Array.from(document.styleSheets)
  .flatMap((sheet) => {
    try {
      return Array.from(sheet.cssRules).map(rule => rule.cssText)
    } catch {
      return []
    }
  })

afterEach(() => {
  document.body.innerHTML = ''
})

describe('CodeSourceEditor', () => {
  it('mounts CodeMirror, follows model updates, and renders an error marker', async () => {
    const bootstrapStyle = document.createElement('style')
    bootstrapStyle.nonce = 'tauri-csp-nonce'
    document.head.appendChild(bootstrapStyle)
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

      const codeMirrorStyle = Array.from(document.head.querySelectorAll('style'))
        .find(style => style.textContent?.includes('.cm-content'))
      expect(codeMirrorStyle?.nonce).toBe('tauri-csp-nonce')

      value.value = '{"model":"second"}'
      marker.value = { line: 1, column: 10, message: 'Invalid model value' }
      await nextTick()
      await waitForEditorText(el, 'second')

      expect(el.textContent).toContain('Invalid model value')
    } finally {
      app.unmount()
      bootstrapStyle.remove()
    }
  })

  it('applies an explicit foreground color to markdown content and lines', async () => {
    const el = document.createElement('div')
    document.body.appendChild(el)
    const app = createApp(defineComponent({
      setup: () => () => h(CodeSourceEditor, {
        modelValue: '# Visible heading\n\nVisible body',
        language: 'markdown',
      }),
    }))
    app.use(i18n)
    app.mount(el)

    try {
      await waitForEditorText(el, 'Visible body')

      const rules = editorThemeRules().join('\n')
      expect(rules).toMatch(/\.cm-content\s*\{[^}]*color:\s*var\(--text-primary\)/)
      expect(rules).toMatch(/\.cm-line\s*\{[^}]*color:\s*var\(--text-primary\)/)
    } finally {
      app.unmount()
    }
  })

  it('emits the first editor document change', async () => {
    const value = ref('# Initial')
    const el = document.createElement('div')
    document.body.appendChild(el)
    const app = createApp(defineComponent({
      setup() {
        return () => h(CodeSourceEditor, {
          modelValue: value.value,
          language: 'markdown',
          'onUpdate:modelValue': (next: string) => { value.value = next },
        })
      },
    }))
    app.use(i18n)
    app.mount(el)

    try {
      await waitForEditorText(el, 'Initial')
      const content = el.querySelector<HTMLElement>('.cm-content')!
      content.textContent = '# First edit'
      content.dispatchEvent(new InputEvent('input', {
        bubbles: true,
        data: '# First edit',
        inputType: 'insertText',
      }))

      await vi.waitFor(() => expect(value.value).toBe('# First edit'))
    } finally {
      app.unmount()
    }
  })
})
