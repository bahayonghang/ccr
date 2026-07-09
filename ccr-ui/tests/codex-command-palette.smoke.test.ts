import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick, reactive } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { CodexProfile } from '@/types'

vi.mock('@/components/ui/SIcon.vue', () => ({
  default: defineComponent({
    props: {
      name: { type: String, required: true },
      size: { type: String, default: '' },
    },
    setup(props) {
      return () =>
        h('span', {
          'data-icon': props.name,
          class: props.size,
        })
    },
  }),
}))

import ProfilesCommandPalette, {
  type ProfilesCommandPaletteAction,
  type ProfilesCommandPaletteDescriptor,
} from '@/components/profiles/ProfilesCommandPalette.vue'

const messages = {
  en: {
    codex: {
      profiles: {
        commandPalette: {
          title: 'Command palette',
          eyebrow: 'Codex Profiles',
          placeholder: 'Type a command or profile name…',
          resultSummary: '{commands} commands · {profiles} profiles',
          groupCommands: 'Common commands',
          groupProfiles: 'Switch Profile',
          execute: 'Run',
          select: 'Select',
          close: 'Close',
          kindCommand: 'Command',
          kindSwitch: 'Switch',
          empty: 'No matches',
          actionAdd: 'Add profile',
          actionExport: 'Export profiles.toml',
          actionReload: 'Reload configuration',
          actionApply: 'Switch to {name}',
        },
      },
    },
  },
}

const flush = async () => {
  await Promise.resolve()
  await nextTick()
  await Promise.resolve()
  await nextTick()
}

const dispatchKey = (element: Element, key: string) => {
  element.dispatchEvent(
    new KeyboardEvent('keydown', {
      key,
      bubbles: true,
      cancelable: true,
    })
  )
}

const profiles: CodexProfile[] = [
  {
    name: 'alpha',
    description: 'fast relay with a deliberately long descriptive hint',
    enabled: true,
  },
  {
    name: 'beta',
    base_url: 'https://beta.example.com/v1',
    enabled: true,
  },
  {
    name: 'archived',
    description: 'disabled profile',
    enabled: false,
  },
]

const descriptor: ProfilesCommandPaletteDescriptor<CodexProfile> = {
  isEnabled: (profile) => profile.enabled !== false,
  hint: (profile) => profile.description || profile.base_url || undefined,
}

const mountPalette = async () => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const state = reactive({ open: false })
  const events = {
    apply: vi.fn(),
    add: vi.fn(),
    export: vi.fn(),
    reload: vi.fn(),
  }

  const actions: ProfilesCommandPaletteAction[] = [
    {
      id: '__add',
      icon: 'Plus',
      labelKey: 'codex.profiles.commandPalette.actionAdd',
      handler: events.add,
    },
    {
      id: '__reload',
      icon: 'RefreshCw',
      labelKey: 'codex.profiles.commandPalette.actionReload',
      handler: events.reload,
    },
    {
      id: '__export',
      icon: 'Download',
      labelKey: 'codex.profiles.commandPalette.actionExport',
      handler: events.export,
    },
  ]

  const i18n = createI18n({
    legacy: false,
    locale: 'en',
    missingWarn: false,
    fallbackWarn: false,
    messages,
  })

  const app = createApp(
    defineComponent({
      setup() {
        return () =>
          h(ProfilesCommandPalette, {
            open: state.open,
            profiles,
            descriptor,
            actions,
            i18nPrefix: 'codex.profiles.commandPalette',
            'onUpdate:open': (value: boolean) => {
              state.open = value
            },
            onApply: events.apply,
          })
      },
    })
  )

  app.use(i18n)
  app.mount(el)
  await flush()

  return {
    state,
    events,
    open: async () => {
      state.open = true
      await flush()
    },
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

beforeEach(() => {
  Element.prototype.scrollIntoView = vi.fn()
})

afterEach(() => {
  document.body.innerHTML = ''
  document.body.style.overflow = ''
  vi.clearAllMocks()
})

describe('Codex CommandPalette smoke', () => {
  it('renders through the shared BaseModal shell with command and profile groups', async () => {
    const palette = await mountPalette()

    try {
      await palette.open()

      expect(document.body.querySelector('.base-modal-root')).toBeTruthy()
      expect(document.body.querySelector('.cp-palette-modal')).toBeTruthy()
      expect(document.body.textContent).toContain('Common commands')
      expect(document.body.textContent).toContain('Switch Profile')
      expect(document.body.textContent).toContain('3 commands · 2 profiles')
      expect(document.body.textContent).toContain('Switch to alpha')
      expect(document.body.textContent).not.toContain('Switch to archived')
    } finally {
      palette.unmount()
    }
  })

  it('keeps arrow navigation and Enter execution working for profile switches', async () => {
    const palette = await mountPalette()

    try {
      await palette.open()
      const input = document.body.querySelector<HTMLInputElement>('#cp-command-palette-search')
      expect(input).toBeTruthy()

      dispatchKey(input!, 'ArrowDown')
      dispatchKey(input!, 'ArrowDown')
      dispatchKey(input!, 'ArrowDown')
      dispatchKey(input!, 'Enter')
      await flush()

      expect(palette.events.apply).toHaveBeenCalledWith('alpha')
      expect(palette.state.open).toBe(false)
    } finally {
      palette.unmount()
    }
  })

  it('closes on Escape without leaving the body scroll lock behind', async () => {
    const palette = await mountPalette()

    try {
      await palette.open()
      expect(document.body.style.overflow).toBe('hidden')

      const input = document.body.querySelector<HTMLInputElement>('#cp-command-palette-search')
      dispatchKey(input!, 'Escape')
      await flush()

      expect(palette.state.open).toBe(false)
      expect(document.body.style.overflow).toBe('')
    } finally {
      palette.unmount()
    }
  })
})
