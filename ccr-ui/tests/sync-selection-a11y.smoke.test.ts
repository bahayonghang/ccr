import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import enUS from '@/i18n/locales/en-US'
import type { CustomSyncFolderForm, SyncSelectableItem } from '@/types/syncSelection'

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

const i18n = createI18n({
  legacy: false,
  locale: 'en-US',
  fallbackLocale: 'en-US',
  missingWarn: false,
  fallbackWarn: false,
  messages: { 'en-US': enUS },
})

const presetConfig: SyncSelectableItem = {
  key: 'preset',
  name: 'Preset config',
  description: 'Required preset',
  selected: true,
  required: true,
  localPath: 'D:/preset',
  remotePath: '/preset',
  icon: 'Settings',
}

const optionalItem: SyncSelectableItem = {
  key: 'optional',
  name: 'Optional folder',
  description: 'Optional sync target',
  selected: true,
  localPath: 'D:/optional',
  remotePath: '/optional',
  icon: 'Folder',
}

const customFolder: CustomSyncFolderForm = {
  name: '',
  localPath: '',
  remotePath: '',
  description: '',
}

const mountPanel = async () => {
  const { default: SyncSelectionPanel } = await import('@/components/sync/SyncSelectionPanel.vue')
  const el = document.createElement('div')
  document.body.appendChild(el)
  const app = createApp(SyncSelectionPanel, {
    applying: false,
    addingCustom: false,
    hasChanges: true,
    presetConfig,
    optionalItems: [optionalItem],
    customFolder,
    toggleItem: vi.fn(),
    applySelection: vi.fn(),
    addCustomFolder: vi.fn(),
    updatePresetLocalPath: vi.fn(),
    updateOptionalLocalPath: vi.fn(),
    updateOptionalRemotePath: vi.fn(),
    updateCustomField: vi.fn(),
  })
  app.use(i18n)
  app.mount(el)
  await nextTick()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

afterEach(() => {
  document.body.innerHTML = ''
  vi.clearAllMocks()
})

describe('SyncSelectionPanel accessibility semantics', () => {
  it('names path and custom folder inputs without relying on placeholders only', async () => {
    const { el, unmount } = await mountPanel()

    try {
      expect(el.querySelector('input[aria-label="Local path"]')).toBeTruthy()
      expect(el.querySelector('input[aria-label="Optional folder Local path"]')).toBeTruthy()
      expect(el.querySelector('input[aria-label="Optional folder Remote path (optional)"]')).toBeTruthy()
      expect(el.querySelector('input[aria-label="Folder name"]')).toBeTruthy()
      expect(el.querySelector('input[aria-label="Description (optional)"]')).toBeTruthy()
    } finally {
      unmount()
    }
  })
})
