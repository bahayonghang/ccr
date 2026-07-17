import { createPinia, setActivePinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import enUS from '@/i18n/locales/en-US'
import type { RawFileGetResult, RawProfilesSaveResult } from '@/api/domains/configRawTypes'

vi.mock('vue-router', () => ({
  onBeforeRouteLeave: vi.fn(),
}))

vi.mock('@/components/common/BaseModal.vue', () => ({
  default: defineComponent({
    setup(_props, { slots }) {
      return () => h('section', { 'data-testid': 'profiles-raw-modal' }, [
        slots.header?.({ titleId: 'profiles-raw-title' }),
        slots.default?.(),
      ])
    },
  }),
}))

vi.mock('@/components/editor/CodeSourceEditor.vue', () => ({
  default: defineComponent({
    props: {
      modelValue: { type: String, required: true },
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

import ProfilesRawEditorPanel from '@/components/profiles/ProfilesRawEditorPanel.vue'
import { useUIStore } from '@/stores/ui'

const i18n = createI18n({
  legacy: false,
  locale: 'en-US',
  fallbackLocale: 'en-US',
  missingWarn: false,
  fallbackWarn: false,
  messages: { 'en-US': enUS },
})

const settle = async () => {
  await Promise.resolve()
  await nextTick()
  await Promise.resolve()
  await nextTick()
}

const findButton = (el: HTMLElement, text: string) =>
  Array.from(el.querySelectorAll<HTMLButtonElement>('button'))
    .find(button => button.textContent?.includes(text))

let pinia: ReturnType<typeof createPinia>

beforeEach(() => {
  pinia = createPinia()
  setActivePinia(pinia)
})

afterEach(() => {
  document.body.innerHTML = ''
  vi.restoreAllMocks()
})

async function mountPanel(
  getRaw: () => Promise<RawFileGetResult>,
  saveRaw: (content: string, token: string, force?: boolean) => Promise<RawProfilesSaveResult>,
  onSaved = vi.fn(),
) {
  const el = document.createElement('div')
  document.body.appendChild(el)
  const app = createApp(defineComponent({
    setup() {
      return () => h(ProfilesRawEditorPanel, { getRaw, saveRaw, onSaved })
    },
  }))
  app.use(i18n)
  app.use(pinia)
  app.mount(el)
  await settle()
  return { el, onSaved, unmount: () => app.unmount() }
}

describe('ProfilesRawEditorPanel', () => {
  it('confirms activation loss and retries the same content with force', async () => {
    const getRaw = vi.fn().mockResolvedValue({
      status: 'ok',
      content: '[active]\nmodel = "first"\n',
      token: 'token-v1',
      path: 'C:/Users/test/.ccr/platforms/claude/profiles.toml',
      exists: true,
    })
    const saveRaw = vi.fn()
      .mockResolvedValueOnce({ status: 'activation_conflict', current: 'active' })
      .mockResolvedValueOnce({ status: 'saved', token: 'token-v2', profiles_count: 1 })
    const uiStore = useUIStore()
    const confirm = vi.spyOn(uiStore, 'requestConfirm').mockResolvedValue(true)
    vi.spyOn(uiStore, 'showSuccess').mockImplementation(() => 1)
    const mounted = await mountPanel(getRaw, saveRaw)

    try {
      const editor = mounted.el.querySelector<HTMLTextAreaElement>('textarea')!
      editor.value = '[replacement]\nmodel = "second"\n'
      editor.dispatchEvent(new Event('input', { bubbles: true }))
      await nextTick()
      findButton(mounted.el, 'Save TOML')?.click()
      await settle()

      expect(saveRaw).toHaveBeenNthCalledWith(
        1,
        '[replacement]\nmodel = "second"\n',
        'token-v1',
        false,
      )
      expect(saveRaw).toHaveBeenNthCalledWith(
        2,
        '[replacement]\nmodel = "second"\n',
        'token-v1',
        true,
      )
      expect(confirm).toHaveBeenCalledWith(expect.objectContaining({ type: 'danger' }))
      expect(mounted.onSaved).toHaveBeenCalledOnce()
    } finally {
      mounted.unmount()
    }
  })

  it('shows a stale-token conflict and reloads the disk version', async () => {
    const getRaw = vi.fn()
      .mockResolvedValueOnce({
        status: 'ok',
        content: '[active]\nmodel = "first"\n',
        token: 'token-v1',
        path: 'C:/profiles.toml',
        exists: true,
      })
      .mockResolvedValueOnce({
        status: 'ok',
        content: '[active]\nmodel = "external"\n',
        token: 'token-v2',
        path: 'C:/profiles.toml',
        exists: true,
      })
    const saveRaw = vi.fn().mockResolvedValue({ status: 'conflict' })
    vi.spyOn(useUIStore(), 'requestConfirm').mockResolvedValue(true)
    const mounted = await mountPanel(getRaw, saveRaw)

    try {
      const editor = mounted.el.querySelector<HTMLTextAreaElement>('textarea')!
      editor.value = '[active]\nmodel = "editor"\n'
      editor.dispatchEvent(new Event('input', { bubbles: true }))
      await nextTick()
      findButton(mounted.el, 'Save TOML')?.click()
      await settle()

      expect(mounted.el.textContent).toContain('The file changed outside CCR')
      findButton(mounted.el, 'Reload')?.click()
      await settle()

      expect(getRaw).toHaveBeenCalledTimes(2)
      expect(mounted.el.querySelector<HTMLTextAreaElement>('textarea')?.value)
        .toBe('[active]\nmodel = "external"\n')
    } finally {
      mounted.unmount()
    }
  })

  it('passes validation positions to the shared editor marker', async () => {
    const getRaw = vi.fn().mockResolvedValue({
      status: 'ok',
      content: '[active]\nmodel = "first"\n',
      token: 'token-v1',
      path: 'C:/profiles.toml',
      exists: true,
    })
    const saveRaw = vi.fn().mockResolvedValue({
      status: 'invalid',
      kind: 'syntax',
      message: 'Invalid profiles TOML syntax',
      line: 3,
      column: 4,
    })
    const mounted = await mountPanel(getRaw, saveRaw)

    try {
      const editor = mounted.el.querySelector<HTMLTextAreaElement>('textarea')!
      editor.value = '[broken'
      editor.dispatchEvent(new Event('input', { bubbles: true }))
      await nextTick()
      findButton(mounted.el, 'Save TOML')?.click()
      await settle()

      expect(editor.dataset.error).toBe('Invalid profiles TOML syntax')
    } finally {
      mounted.unmount()
    }
  })
})
