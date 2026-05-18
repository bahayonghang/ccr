import { createI18n } from 'vue-i18n'
import { computed, createApp, defineComponent, h, nextTick, ref } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import enUS from '@/i18n/locales/en-US'
import type { CcrCommand, CcrModule, CommandHistory, FavoriteCommand } from '@/api/ccr-control'

const waitForTransition = () => new Promise((resolve) => window.setTimeout(resolve, 250))

const command: CcrCommand = {
  name: 'Init',
  command: 'init',
  description: 'Create config template',
  dangerous: true,
  executable: false,
}

const executableDangerousCommand: CcrCommand = {
  name: 'Import',
  command: 'import',
  description: 'Import config file',
  dangerous: true,
  executable: true,
}

const modules = ref<CcrModule[]>([{
  id: 'config',
  name: 'Config',
  icon: 'Settings',
  description: 'Config commands',
  commands: [command, executableDangerousCommand],
}])
const selectedModuleId = ref('config')
const selectedCommand = ref<CcrCommand | null>(null)
const favorites = ref<FavoriteCommand[]>([{
  id: 'fav-init',
  command: 'init',
  args: [],
  display_name: 'Init',
  module: 'config',
  created_at: '2026-04-10T00:00:00.000Z',
}])
const history = ref<CommandHistory[]>([{
  id: 'hist-init',
  full_command: 'ccr init',
  command: 'init',
  args: [],
  success: true,
  executed_at: '2026-04-10T00:00:00.000Z',
  duration_ms: 12,
}])

const ccrControlMock = {
  versionInfo: ref(null),
  loadVersionInfo: vi.fn(),
  modules,
  selectedModuleId,
  selectedModule: computed(() => modules.value[0]),
  selectedCommand,
  selectModule: vi.fn((id: string) => { selectedModuleId.value = id }),
  selectCommand: vi.fn((cmd: CcrCommand) => { selectedCommand.value = cmd }),
  commandArgs: ref({}),
  commandFlags: ref({}),
  favorites,
  addToFavorites: vi.fn(),
  removeFromFavorites: vi.fn(),
  isFavorite: vi.fn((cmd: string) => favorites.value.some((fav) => fav.command === cmd)),
  history,
  clearHistory: vi.fn(),
  isExecuting: ref(false),
  outputLines: ref([]),
  lastExitCode: ref(null),
  executeCommand: vi.fn(),
  executeFromFavorite: vi.fn(),
  executeFromHistory: vi.fn(),
  clearOutput: vi.fn(),
}

vi.mock('@/components/ThemeToggle.vue', () => ({ default: defineComponent({ template: '<button type="button">theme</button>' }) }))
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

const mountView = async () => {
  vi.doMock('@/composables/useCcrControl', () => ({
    useCcrControl: () => ccrControlMock,
  }))
  const { default: CcrControlView } = await import('@/views/CcrControlView.vue')
  const el = document.createElement('div')
  document.body.appendChild(el)
  const app = createApp(CcrControlView)
  app.use(i18n)
  app.mount(el)
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

afterEach(() => {
  document.body.innerHTML = ''
  selectedCommand.value = null
  vi.clearAllMocks()
})

describe('CcrControlView accessibility semantics', () => {
  it('exposes command, favorite, and history rows as semantic buttons', async () => {
    const { el, unmount } = await mountView()

    try {
      const commandButton = el.querySelector('button[aria-label="Select command Init: ccr init"]') as HTMLButtonElement | null
      expect(commandButton).toBeTruthy()
      commandButton?.click()
      expect(ccrControlMock.selectCommand).toHaveBeenCalledWith(command)
      ccrControlMock.selectCommand.mockClear()

      const favoriteTab = [...el.querySelectorAll('button')].find((button) => button.textContent?.includes('Favorites')) as HTMLButtonElement | undefined
      expect(favoriteTab).toBeTruthy()
      favoriteTab?.click()
      await nextTick()
      await nextTick()
      await waitForTransition()
      const favoriteButton = el.querySelector('button[aria-label^="Run favorite command"]') as HTMLButtonElement | null
      expect(favoriteButton).toBeTruthy()
      favoriteButton?.click()
      expect(ccrControlMock.selectCommand).toHaveBeenCalledWith(command)
      expect(ccrControlMock.executeFromFavorite).not.toHaveBeenCalled()
      ccrControlMock.selectCommand.mockClear()

      const historyTab = [...el.querySelectorAll('button')].find((button) => button.textContent?.includes('History')) as HTMLButtonElement | undefined
      expect(historyTab).toBeTruthy()
      historyTab?.click()
      await nextTick()
      await nextTick()
      await waitForTransition()
      const historyButton = el.querySelector('button[aria-label^="Run history command"]') as HTMLButtonElement | null
      expect(historyButton).toBeTruthy()
      historyButton?.click()
      expect(ccrControlMock.selectCommand).toHaveBeenCalledWith(command)
      expect(ccrControlMock.executeFromHistory).not.toHaveBeenCalled()
    } finally {
      unmount()
    }
  })

  it('disables unsupported catalog commands instead of executing them', async () => {
    const { el, unmount } = await mountView()

    try {
      const commandButton = el.querySelector('button[aria-label="Select command Init: ccr init"]') as HTMLButtonElement | null
      commandButton?.click()
      await nextTick()

      expect(el.textContent).toContain('Unsupported')
      expect(el.textContent).toContain('local Rust allowlist')
      const executeButton = el.querySelector('button[aria-label="Execute ccr init"]') as HTMLButtonElement | null
      expect(executeButton?.disabled).toBe(true)
    } finally {
      unmount()
    }
  })

  it('requires explicit confirmation before executing supported dangerous commands', async () => {
    const { el, unmount } = await mountView()

    try {
      const commandButton = el.querySelector('button[aria-label="Select command Import: ccr import"]') as HTMLButtonElement | null
      commandButton?.click()
      await nextTick()

      const executeButton = el.querySelector('button[aria-label="Execute ccr import"]') as HTMLButtonElement | null
      expect(executeButton?.disabled).toBe(true)

      const checkbox = el.querySelector('input[type="checkbox"]') as HTMLInputElement | null
      checkbox?.click()
      await nextTick()
      expect(executeButton?.disabled).toBe(false)

      executeButton?.click()
      expect(ccrControlMock.executeCommand).toHaveBeenCalledWith(executableDangerousCommand, { confirmedDanger: true })
    } finally {
      unmount()
    }
  })
})
