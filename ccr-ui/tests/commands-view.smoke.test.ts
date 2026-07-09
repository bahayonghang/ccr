import { createApp, nextTick, ref } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { CommandInfo, CommandJobSnapshot } from '@/types'

const apiMocks = vi.hoisted(() => ({
  cancelCcrCommandJob: vi.fn(),
  listCommands: vi.fn(),
  listConfigs: vi.fn(),
  startCcrCommandJob: vi.fn(),
}))

const uiStateMocks = vi.hoisted(() => ({
  addFavorite: vi.fn(),
  addRecentItem: vi.fn(),
  clearRecentItems: vi.fn(),
  getFavorites: vi.fn(),
  getRecentItems: vi.fn(),
  removeFavorite: vi.fn(),
}))

const eventMocks = vi.hoisted(() => ({
  handlers: new Map<string, (event: { payload: CommandJobSnapshot }) => void>(),
  unlisten: vi.fn(),
  listen: vi.fn((event: string, handler: (event: { payload: CommandJobSnapshot }) => void) => {
    eventMocks.handlers.set(event, handler)
    return Promise.resolve(eventMocks.unlisten)
  }),
}))

const runtimeMocks = vi.hoisted(() => ({
  isTauriRuntime: vi.fn(() => true),
}))

const routerMocks = vi.hoisted(() => ({
  route: { params: { client: 'ccr' } },
  replace: vi.fn(),
}))

vi.mock('@/api', () => apiMocks)

vi.mock('@/api/domains/uiState', () => uiStateMocks)

vi.mock('@tauri-apps/api/event', () => ({
  listen: eventMocks.listen,
}))

vi.mock('@/utils/tauriRuntime', () => ({
  isTauriRuntime: runtimeMocks.isTauriRuntime,
}))

vi.mock('@/utils/runtimeState', () => ({
  getRuntimeUnavailableCopy: () => ({
    title: 'Desktop runtime unavailable',
    description: 'Open the desktop app',
  }),
}))

vi.mock('@/utils/logger', () => ({
  logger: {
    error: vi.fn(),
  },
}))

vi.mock('@iconify/vue', () => ({
  Icon: {
    props: ['icon'],
    template: '<span data-icon="true" />',
  },
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      if (!params) return key
      return `${key} ${JSON.stringify(params)}`
    },
    locale: ref('en-US'),
  }),
}))

vi.mock('vue-router', () => ({
  useRoute: () => routerMocks.route,
  useRouter: () => ({ replace: routerMocks.replace }),
}))

const flush = async () => {
  await Promise.resolve()
  await nextTick()
  await Promise.resolve()
  await nextTick()
}

const baseCommands: CommandInfo[] = [
  {
    name: 'status',
    description: 'Show current status',
    usage: 'ccr status',
    examples: ['ccr status'],
    category: 'read',
    risk: 'safe',
    executable: true,
  },
  {
    name: 'delete',
    description: 'Use the typed desktop configuration API to delete a saved CCR configuration.',
    usage: 'Config domain: delete_config(name)',
    examples: ["delete_config({ name: 'old-config' })"],
    category: 'danger',
    risk: 'typed_only',
    executable: false,
    requiresConfirmation: false,
  },
]

const runningSnapshot: CommandJobSnapshot = {
  job_id: 'ccr-command-test',
  command: 'status',
  args: [],
  status: 'running',
  started_at: '2026-05-18T08:00:00.000Z',
  finished_at: null,
  duration_ms: null,
  exit_code: null,
  stdout_lines: ['\u001B[32mstatus ok\u001B[0m', '| profile | active | source |'],
  stderr_lines: [],
  system_lines: ['Process started'],
  error: null,
}

const mountView = async () => {
  const { default: CommandsView } = await import('@/views/CommandsView.vue')
  const el = document.createElement('div')
  document.body.appendChild(el)
  const app = createApp(CommandsView)
  app.mount(el)
  await flush()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

const resetMocks = () => {
  vi.resetModules()
  vi.clearAllMocks()
  document.body.innerHTML = ''
  routerMocks.route.params.client = 'ccr'
  routerMocks.replace.mockReset()
  runtimeMocks.isTauriRuntime.mockReturnValue(true)
  eventMocks.handlers.clear()
  eventMocks.unlisten.mockReset()
  eventMocks.listen.mockClear()
  apiMocks.listCommands.mockResolvedValue(baseCommands)
  apiMocks.listConfigs.mockResolvedValue({ configs: [{ name: 'default' }] })
  apiMocks.startCcrCommandJob.mockResolvedValue({
    job_id: runningSnapshot.job_id,
    snapshot: runningSnapshot,
  })
  apiMocks.cancelCcrCommandJob.mockResolvedValue({
    ...runningSnapshot,
    status: 'cancelled',
    finished_at: '2026-05-18T08:00:01.000Z',
    duration_ms: 1000,
    error: 'Command cancelled',
  })
  uiStateMocks.getFavorites.mockResolvedValue([])
  uiStateMocks.getRecentItems.mockResolvedValue([])
  uiStateMocks.addFavorite.mockResolvedValue({
    id: 'favorite-status',
    command: 'status',
    args: [],
    display_name: 'status',
    module: 'commands',
    created_at: '2026-05-18T08:00:00.000Z',
  })
  uiStateMocks.removeFavorite.mockResolvedValue(true)
  uiStateMocks.addRecentItem.mockResolvedValue({
    id: 'history-status',
    full_command: 'ccr status',
    command: 'status',
    args: [],
    success: true,
    executed_at: '2026-05-18T08:00:02.000Z',
    duration_ms: 2000,
  })
  uiStateMocks.clearRecentItems.mockResolvedValue('History cleared successfully')
}

describe('CommandsView smoke', () => {
  beforeEach(resetMocks)

  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('uses the CCR job API and updates the ledger from Tauri events', async () => {
    const { el, unmount } = await mountView()

    try {
      expect(el.querySelector('.commands-workbench__main .commands-composer + .commands-ledger')).toBeTruthy()
      expect(el.textContent).toContain('commands.runtimeReady')
      expect(eventMocks.listen).toHaveBeenCalledWith('commands:job-progress', expect.any(Function))
      expect(eventMocks.listen).toHaveBeenCalledWith('commands:job-finished', expect.any(Function))
      expect(eventMocks.listen).toHaveBeenCalledWith('commands:job-cancelled', expect.any(Function))

      const run = Array.from(el.querySelectorAll<HTMLButtonElement>('button'))
        .find((button) => button.textContent?.includes('commands.run'))
      expect(run).toBeTruthy()

      run?.click()
      await flush()

      expect(apiMocks.startCcrCommandJob).toHaveBeenCalledWith({ command: 'status', args: [] })
      expect(el.textContent).toContain('status ok')
      expect(el.querySelector('.commands-ledger__metrics')).toBeTruthy()
      expect(el.querySelector('.commands-terminal')).toBeTruthy()
      expect(el.querySelector('.commands-terminal__text .ansi-green-fg')).toBeTruthy()

      eventMocks.handlers.get('commands:job-finished')?.({
        payload: {
          ...runningSnapshot,
          status: 'success',
          finished_at: '2026-05-18T08:00:02.000Z',
          duration_ms: 2000,
          exit_code: 0,
          stdout_lines: ['\u001B[32mstatus ok\u001B[0m', '| profile | active | source |', 'done'],
        },
      })
      await flush()

      expect(el.textContent).toContain('done')
      expect(el.textContent).toContain('commands.status.success')
      expect(uiStateMocks.addRecentItem).toHaveBeenCalledWith('status', [], true, 2000)
    } finally {
      unmount()
    }

    expect(eventMocks.unlisten).toHaveBeenCalledTimes(3)
  })

  it('keeps web preview honest and blocks execution', async () => {
    runtimeMocks.isTauriRuntime.mockReturnValue(false)

    const { el, unmount } = await mountView()

    try {
      expect(el.textContent).toContain('commands.runtimeWeb')
      expect(el.textContent).toContain('commands.webUnavailableDetail')

      const run = Array.from(el.querySelectorAll<HTMLButtonElement>('button'))
        .find((button) => button.textContent?.includes('commands.run'))
      expect(run?.disabled).toBe(true)
      run?.click()
      await flush()

      expect(apiMocks.startCcrCommandJob).not.toHaveBeenCalled()
      expect(eventMocks.listen).not.toHaveBeenCalled()
    } finally {
      unmount()
    }
  })

  it('blocks typed-only destructive catalog commands from generic execution', async () => {
    const { el, unmount } = await mountView()

    try {
      const deleteCommand = Array.from(el.querySelectorAll<HTMLButtonElement>('.command-row'))
        .find((button) => button.textContent?.includes('delete'))
      expect(deleteCommand?.className).toContain('command-row--disabled')
      deleteCommand?.click()
      await flush()

      expect(el.textContent).toContain('commands.commandBlockedTitle')

      const run = Array.from(el.querySelectorAll<HTMLButtonElement>('button'))
        .find((button) => button.textContent?.includes('commands.run'))
      expect(run?.disabled).toBe(true)
      run?.click()
      await flush()

      expect(apiMocks.startCcrCommandJob).not.toHaveBeenCalled()
    } finally {
      unmount()
    }
  })

  it('requires explicit confirmation before metadata-only dangerous commands can run', async () => {
    apiMocks.listCommands.mockResolvedValue([
      baseCommands[0],
      {
        name: 'purge-cache',
        description: 'Metadata-only destructive command',
        usage: 'ccr purge-cache <target>',
        examples: ['ccr purge-cache temp'],
        category: 'danger',
        risk: 'destructive',
        executable: true,
        requiresConfirmation: true,
        args: [
          {
            name: 'target',
            label: 'Target',
            type: 'text',
            required: true,
            description: 'Target cache',
          },
        ],
      },
    ] satisfies CommandInfo[])

    const { el, unmount } = await mountView()

    try {
      const purgeCommand = Array.from(el.querySelectorAll<HTMLButtonElement>('.command-row'))
        .find((button) => button.textContent?.includes('purge-cache'))
      purgeCommand?.click()
      await flush()

      expect(el.textContent).toContain('commands.dangerConfirmTitle')
      const run = Array.from(el.querySelectorAll<HTMLButtonElement>('button'))
        .find((button) => button.textContent?.includes('commands.run'))
      expect(run?.disabled).toBe(true)

      const confirm = el.querySelector<HTMLInputElement>('.commands-danger-confirm input')
      expect(confirm).toBeTruthy()
      confirm!.checked = true
      confirm!.dispatchEvent(new Event('change'))
      await flush()

      const args = el.querySelector<HTMLInputElement>('.commands-field input')
      expect(args).toBeTruthy()
      args!.value = 'temp'
      args!.dispatchEvent(new Event('input'))
      await flush()

      run?.click()
      await flush()

      expect(apiMocks.startCcrCommandJob).toHaveBeenCalledWith({
        command: 'purge-cache',
        args: ['temp'],
        confirmationToken: 'desktop-confirm:purge-cache',
      })
    } finally {
      unmount()
    }
  })

  it('uses backend metadata instead of a frontend hardcoded command allowlist', async () => {
    apiMocks.listCommands.mockResolvedValue([
      {
        name: 'status',
        description: 'Show current status',
        usage: 'ccr status',
        examples: ['ccr status'],
        category: 'read',
        risk: 'safe',
        executable: true,
      },
      {
        name: 'purge-cache',
        description: 'Metadata-only destructive command',
        usage: 'ccr purge-cache <target>',
        examples: ['ccr purge-cache temp'],
        category: 'danger',
        risk: 'destructive',
        executable: true,
        requiresConfirmation: true,
        args: [
          {
            name: 'target',
            label: 'Target',
            type: 'text',
            required: true,
            description: 'Target cache',
          },
        ],
      },
      {
        name: 'platform',
        description: 'Preview-only command',
        usage: 'ccr platform list',
        examples: ['ccr platform list'],
        category: 'preview',
        risk: 'preview_only',
        executable: false,
      },
    ] satisfies CommandInfo[])

    const { el, unmount } = await mountView()

    try {
      const purgeCommand = Array.from(el.querySelectorAll<HTMLButtonElement>('.command-row'))
        .find((button) => button.textContent?.includes('purge-cache'))
      purgeCommand?.click()
      await flush()

      const run = Array.from(el.querySelectorAll<HTMLButtonElement>('button'))
        .find((button) => button.textContent?.includes('commands.run'))
      expect(run?.disabled).toBe(true)

      const confirm = el.querySelector<HTMLInputElement>('.commands-danger-confirm input')
      expect(confirm).toBeTruthy()
      confirm!.checked = true
      confirm!.dispatchEvent(new Event('change'))
      await flush()

      const args = el.querySelector<HTMLInputElement>('.commands-field input')
      expect(args).toBeTruthy()
      args!.value = 'temp'
      args!.dispatchEvent(new Event('input'))
      await flush()

      run?.click()
      await flush()

      expect(apiMocks.startCcrCommandJob).toHaveBeenCalledWith({
        command: 'purge-cache',
        args: ['temp'],
        confirmationToken: 'desktop-confirm:purge-cache',
      })

      const previewCommand = Array.from(el.querySelectorAll<HTMLButtonElement>('.command-row'))
        .find((button) => button.textContent?.includes('platform'))
      expect(previewCommand?.className).toContain('command-row--disabled')
    } finally {
      unmount()
    }
  })

  it('loads favorites and history into the composer instead of executing directly', async () => {
    apiMocks.listCommands.mockResolvedValue([
      baseCommands[0],
      {
        name: 'purge-cache',
        description: 'Metadata-only destructive command',
        usage: 'ccr purge-cache <target>',
        examples: ['ccr purge-cache temp'],
        category: 'danger',
        risk: 'destructive',
        executable: true,
        requiresConfirmation: true,
        args: [
          {
            name: 'target',
            label: 'Target',
            type: 'text',
            required: true,
            description: 'Target cache',
          },
        ],
      },
    ] satisfies CommandInfo[])
    uiStateMocks.getFavorites.mockResolvedValue([
      {
        id: 'favorite-purge-cache',
        command: 'purge-cache',
        args: ['temp'],
        display_name: 'Purge temp cache',
        module: 'commands',
        created_at: '2026-05-18T08:00:00.000Z',
      },
    ])
    uiStateMocks.getRecentItems.mockResolvedValue([
      {
        id: 'history-purge-cache',
        full_command: 'ccr purge-cache temp',
        command: 'purge-cache',
        args: ['temp'],
        success: false,
        executed_at: '2026-05-18T08:00:00.000Z',
        duration_ms: 42,
      },
    ])

    const { el, unmount } = await mountView()

    try {
      const favoritesTab = Array.from(el.querySelectorAll<HTMLButtonElement>('.commands-source-tabs__item'))
        .find((button) => button.textContent?.includes('commands.favorites'))
      favoritesTab?.click()
      await flush()

      const favorite = Array.from(el.querySelectorAll<HTMLButtonElement>('.command-row'))
        .find((button) => button.textContent?.includes('Purge temp cache'))
      favorite?.click()
      await flush()

      const composerTitle = el.querySelector('.commands-composer .commands-panel__title--large')
      expect(composerTitle?.textContent).toContain('purge-cache')

      expect(apiMocks.startCcrCommandJob).not.toHaveBeenCalled()
      expect(el.textContent).toContain('commands.dangerConfirmTitle')

      const run = Array.from(el.querySelectorAll<HTMLButtonElement>('button'))
        .find((button) => button.textContent?.includes('commands.run'))
      expect(run?.disabled).toBe(true)

      const confirm = el.querySelector<HTMLInputElement>('.commands-danger-confirm input')
      confirm!.checked = true
      confirm!.dispatchEvent(new Event('change'))
      await flush()

      const args = el.querySelector<HTMLInputElement>('.commands-field input')
      expect(args?.value).toBe('temp')
      expect(run?.disabled).toBe(false)

      run?.click()
      await flush()

      expect(apiMocks.startCcrCommandJob).toHaveBeenCalledWith({
        command: 'purge-cache',
        args: ['temp'],
        confirmationToken: 'desktop-confirm:purge-cache',
      })
    } finally {
      unmount()
    }
  })
})
