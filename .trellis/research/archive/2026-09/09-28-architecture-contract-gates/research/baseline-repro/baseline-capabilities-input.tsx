import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { EditorView } from '@codemirror/view'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ReactNode } from 'react'

const transport = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args?: { content?: string; token?: string; patch?: unknown; settings?: unknown }) => Promise<unknown>>(),
  confirm: vi.fn(), error: vi.fn(), success: vi.fn(),
}))
vi.mock('@tauri-apps/api/core', () => ({ invoke: transport.invoke }))
vi.mock('@/utils/tauriRuntime', async (original) => ({ ...await original<object>(), isTauriRuntime: () => true }))
vi.mock('@/configs/surfaceNotify', async (original) => {
  const actual = await original<typeof import('@/configs/surfaceNotify')>()
  return { surfaceNotify: {
    ...actual.surfaceNotify, confirm: transport.confirm, success: transport.success,
    error: (message: string) => { transport.error(message); actual.surfaceNotify.error(message) },
  } }
})
vi.mock('@/i18n', async (original) => {
  const messages = (await import('@/i18n/locales/zh-CN')).default.settingsRaw
  const t = (key: string) => key === 'settingsRaw.claudeClearUnsupported'
    ? messages.claudeClearUnsupported
    : key === 'settingsRaw.opencodeClearUnsupported' ? messages.opencodeClearUnsupported : key
  return { ...await original<object>(), useResolvedT: () => t, useAppT: () => t }
})

import { GrokSettingsView } from '@/features/grok/GrokSettingsView'
import { CodexSettingsView } from '@/features/codex/CodexSettingsView'
import { ClaudeSettingsView } from '@/features/claude/ClaudeSettingsView'
import { OpenCodeSettingsView } from '@/features/opencode/OpenCodeSettingsView'
import { ToastContainer } from '@/shell/ToastContainer'
import { useUIStore } from '@/shell/stores/ui'
import zhCN from '@/i18n/locales/zh-CN'
import type { GrokSettingsCommandResponse } from '@/types/grok'

let environment: string
let settings: Extract<GrokSettingsCommandResponse, { status: 'ok' }>
let typedStatus: 'saved' | 'conflict' | 'managed_locked'
let rawStatus: 'saved' | 'invalid'
let rawContent: string
let rawToken: string
let rawReads: number
let codexSettings: Record<string, unknown>
let claudeSettings: Record<string, unknown>

function mount(node: ReactNode, path = '/grok/settings') {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 }, mutations: { retry: false } } })
  const router = createMemoryRouter([{ path, element: node }], { initialEntries: [path] })
  return render(<QueryClientProvider client={client}><RouterProvider router={router} /><ToastContainer /></QueryClientProvider>)
}

function mutation(id: string) {
  return transport.invoke.mock.calls.filter(([command]) => command === id)
}

async function editSource(content: string) {
  const element = await waitFor(() => {
    const editor = document.querySelector('.cm-editor')
    expect(editor).toBeTruthy()
    return editor as HTMLElement
  })
  const editor = EditorView.findFromDOM(element)!
  act(() => editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: content } }))
}

beforeEach(() => {
  useUIStore.getState().clearToasts()
  // jsdom has no text layout. Keep the real CodeMirror state and DOM, with empty geometry.
  Object.defineProperty(Range.prototype, 'getClientRects', { configurable: true, value: () => [] })
  Object.defineProperty(Range.prototype, 'getBoundingClientRect', { configurable: true, value: () => new DOMRect() })
  transport.invoke.mockReset()
  transport.confirm.mockReset().mockResolvedValue(true)
  transport.success.mockReset()
  transport.error.mockReset()
  environment = 'local'
  settings = {
    status: 'ok', exists: true, activation: 'inactive', activation_name: null, managed_keys_locked: false,
    models: { default: 'grok-before', default_reasoning_effort: 'future-effort' },
    ui: { theme: 'future-theme' }, session: { auto_compact_threshold_percent: 80, load_envrc: null },
    cli: { auto_update: true, channel: 'future-channel', show_tips: null },
    hints: { new_session_worktree_mode: null, fork_worktree_mode: null }, custom_models: [],
  }
  typedStatus = 'saved'
  rawStatus = 'saved'
  rawContent = '[ui]\ntheme = "dark"\n'
  rawToken = 'v1'
  rawReads = 0
  codexSettings = { model: 'codex-before', model_reasoning_effort: 'future-effort', developer_instructions: 'instructions-before', tui: { notifications: ['agent-turn-complete'] } }
  claudeSettings = { model: 'sonnet', maxThinkingTokens: 100, maxOutputTokens: 200, cleanupPeriodDays: 30 }
  transport.invoke.mockImplementation(async (command: string, args?: { content?: string; token?: string; settings?: unknown }) => {
    if (command === 'get_current_environment') return { id: environment, env_type: environment, name: environment, available: true }
    if (command === 'grok_get_settings') return structuredClone(settings)
    if (command === 'grok_update_settings') return { status: typedStatus, message: 'managed rejection' }
    if (command === 'codex_get_settings') return structuredClone(codexSettings)
    if (command === 'codex_update_settings') {
      // These route saves edit optional top-level strings, which Codex trims.
      for (const [key, value] of Object.entries(args?.settings as Record<string, unknown>)) {
        codexSettings[key] = typeof value === 'string' ? value.trim() : value
      }
      return { message: 'saved' }
    }
    if (command === 'claude_get_settings') return structuredClone(claudeSettings)
    if (command === 'claude_update_settings') {
      Object.assign(claudeSettings, args?.settings)
      return structuredClone(claudeSettings)
    }
    if (command === 'opencode_get_settings') return { model: 'before', small_model: 'small', default_agent: 'build' }
    if (command === 'opencode_get_tui_settings') return { theme: 'dark', mouse: true }
    if (command.endsWith('get_config_raw_text') || command === 'claude_get_settings_raw_text') {
      rawReads++
      return { status: 'ok', content: rawContent, token: rawToken, exists: true, path: '/synthetic/config.toml' }
    }
    if (command.includes('list_config_layers') || command === 'claude_list_settings_layers') {
      const layers = [{ id: 'managed_system', label: 'Policy', path: '/synthetic/policy.toml', exists: true, editable: false, size: 0, mtime: 0 }]
      return command.startsWith('grok') ? { status: 'ok', layers } : { layers }
    }
    if (command.endsWith('save_config_raw_text') || command === 'claude_save_settings_raw_text') {
      if (args?.token !== rawToken) return { status: 'conflict' }
      if (rawStatus === 'saved') { rawContent = args?.content ?? ''; rawToken = 'v2' }
      return { status: rawStatus, token: rawToken, kind: 'syntax', message: 'Invalid TOML', line: 1, column: 1 }
    }
    throw new Error('Unexpected command: ' + command)
  })
})

describe('Settings routes expose domain capabilities', () => {
  it('disables managed model inputs, keeps unknown enums visible, and saves only the edited key', async () => {
    settings.managed_keys_locked = true
    settings.activation = 'active'
    mount(<GrokSettingsView />)
    const model = await screen.findByLabelText('grok.settings.fields.defaultModel') as HTMLInputElement
    expect(model.disabled).toBe(true)
    expect(document.getElementById(model.getAttribute('aria-describedby')!)?.textContent).toBe('grok.settings.managed.description')
    const effort = screen.getByLabelText('grok.settings.fields.reasoningEffort') as HTMLSelectElement
    expect(effort.disabled).toBe(true)
    expect(effort.value).toBe('future-effort')
    expect(screen.getByRole('link', { name: 'grok.settings.managed.action' }).getAttribute('href')).toBe('/grok/profiles')
    fireEvent.click(screen.getByText('grok.settings.tabs.sessionUi'))
    expect((screen.getByLabelText('grok.settings.fields.theme') as HTMLSelectElement).value).toBe('future-theme')
    fireEvent.change(screen.getByLabelText('grok.settings.fields.autoCompact'), { target: { value: '75' } })
    fireEvent.click(screen.getByRole('button', { name: 'grok.settings.save' }))
    await waitFor(() => expect(mutation('grok_update_settings')).toHaveLength(1))
    expect(mutation('grok_update_settings')[0]?.[1]?.patch).toEqual({ set: { 'session.auto_compact_threshold_percent': 75 }, unset: [] })
  })

  it('shows Codex unknown enums and notification arrays without resending either on a model edit', async () => {
    mount(<CodexSettingsView />, '/codex/settings')
    expect((await screen.findByLabelText('codex.settings.model.reasoningEffort') as HTMLSelectElement).value).toBe('future-effort')
    fireEvent.click(screen.getByText('codex.settings.tabs.ui'))
    expect((screen.getByLabelText('codex.settings.ui.notifications') as HTMLSelectElement).value).toBe('["agent-turn-complete"]')
    fireEvent.click(screen.getByText('codex.settings.tabs.model'))
    fireEvent.change(screen.getByLabelText('codex.settings.model.model'), { target: { value: 'edited' } })
    fireEvent.click(screen.getByRole('button', { name: 'codex.settings.save' }))
    await waitFor(() => expect(mutation('codex_update_settings')).toHaveLength(1))
    expect(mutation('codex_update_settings')[0]?.[1]?.settings).toEqual({ model: 'edited' })
  })

  it('represents unset separately from false and rejects invalid integer edits before transport', async () => {
    mount(<GrokSettingsView />)
    await screen.findByLabelText('grok.settings.fields.defaultModel')
    fireEvent.click(screen.getByText('grok.settings.tabs.cli'))
    const field = screen.getByLabelText('grok.settings.fields.autoUpdate')
    fireEvent.change(field, { target: { value: '' } })
    fireEvent.click(screen.getByRole('button', { name: 'grok.settings.save' }))
    await waitFor(() => expect(mutation('grok_update_settings')).toHaveLength(1))
    expect(mutation('grok_update_settings')[0]?.[1]?.patch).toEqual({ set: {}, unset: ['cli.auto_update'] })
    fireEvent.click(screen.getByText('grok.settings.tabs.sessionUi'))
    fireEvent.change(screen.getByLabelText('grok.settings.fields.autoCompact'), { target: { value: '101' } })
    expect((screen.getByRole('button', { name: 'grok.settings.save' }) as HTMLButtonElement).disabled).toBe(true)
    fireEvent.change(screen.getByLabelText('grok.settings.fields.autoCompact'), { target: { value: '1.5' } })
    expect((screen.getByRole('button', { name: 'grok.settings.save' }) as HTMLButtonElement).disabled).toBe(true)
    expect(mutation('grok_update_settings')).toHaveLength(1)
  })

  it('populates memoized inputs, keeps edits across tabs and resets DOM values and dirty state after save', async () => {
    mount(<CodexSettingsView />, '/codex/settings')
    const model = await screen.findByLabelText('codex.settings.model.model') as HTMLInputElement
    await waitFor(() => expect(model.value).toBe('codex-before'))
    fireEvent.change(model, { target: { value: ' changed ' } })
    fireEvent.click(screen.getByText('codex.settings.tabs.tools'))
    const instructions = screen.getByLabelText('codex.settings.tools.developerInstructions') as HTMLTextAreaElement
    expect(instructions.value).toBe('instructions-before')
    fireEvent.change(instructions, { target: { value: 'instructions-after' } })
    fireEvent.click(screen.getByText('codex.settings.tabs.model'))
    expect((screen.getByLabelText('codex.settings.model.model') as HTMLInputElement).value).toBe(' changed ')
    fireEvent.click(screen.getByRole('button', { name: 'codex.settings.save' }))
    await waitFor(() => expect((screen.getByLabelText('codex.settings.model.model') as HTMLInputElement).value).toBe('changed'))
    expect(mutation('codex_update_settings')[0]?.[1]?.settings).toEqual({ model: ' changed ', developer_instructions: 'instructions-after' })
    expect((screen.getByRole('button', { name: 'codex.settings.save' }) as HTMLButtonElement).disabled).toBe(true)
    fireEvent.click(screen.getByText('codex.settings.tabs.tools'))
    expect((screen.getByLabelText('codex.settings.tools.developerInstructions') as HTMLTextAreaElement).value).toBe('instructions-after')
    fireEvent.change(screen.getByLabelText('codex.settings.tools.developerInstructions'), { target: { value: 'next-edit' } })
    expect((screen.getByRole('button', { name: 'codex.settings.save' }) as HTMLButtonElement).disabled).toBe(false)
  })

  it.each(['conflict', 'managed_locked'] as const)('keeps edits on typed %s and displays recovery feedback', async (status) => {
    typedStatus = status
    mount(<GrokSettingsView />)
    const input = await screen.findByLabelText('grok.settings.fields.defaultModel')
    await waitFor(() => expect((input as HTMLInputElement).value).toBe('grok-before'))
    fireEvent.change(input, { target: { value: 'edited' } })
    fireEvent.click(screen.getByRole('button', { name: 'grok.settings.save' }))
    await screen.findByRole('alert')
    expect((input as HTMLInputElement).value).toBe('edited')
    expect(transport.success).not.toHaveBeenCalled()
    if (status === 'conflict') expect((screen.getByRole('button', { name: 'grok.settings.save' }) as HTMLButtonElement).disabled).toBe(true)
    else expect(screen.getByRole('link', { name: 'grok.settings.managed.action' })).toBeTruthy()
  })

  it.each(['wsl', 'ssh'])('blocks all Local-only Settings calls in %s', async (env) => {
    environment = env
    mount(<GrokSettingsView />)
    await screen.findByText('settingsRaw.unsupportedEnvironment')
    expect(transport.invoke.mock.calls.map(([id]) => id)).toEqual(['get_current_environment'])
  })

  it.each([
    ['wsl', <CodexSettingsView />], ['ssh', <CodexSettingsView />],
    ['wsl', <OpenCodeSettingsView />], ['ssh', <OpenCodeSettingsView />],
  ])('does not put fixed local settings into a %s environment session', async (env, view) => {
    environment = String(env)
    mount(view as ReactNode)
    await screen.findByText('settingsRaw.unsupportedEnvironment')
    expect(transport.invoke.mock.calls.map(([id]) => id)).toEqual(['get_current_environment'])
  })

  it('binds both Claude typed reads and writes to the acknowledged environment', async () => {
    mount(<ClaudeSettingsView />, '/claude-code/settings')
    const model = await screen.findByLabelText('claudeSettings.model.defaultModel')
    expect(mutation('claude_get_settings')[0]?.[1]).toEqual({ expectedEnvironmentId: 'local' })
    fireEvent.change(model, { target: { value: 'opus' } })
    fireEvent.click(screen.getByRole('button', { name: 'claudeSettings.save' }))
    await waitFor(() => expect(mutation('claude_update_settings')).toHaveLength(1))
    expect(mutation('claude_update_settings')[0]?.[1]).toMatchObject({ settings: { model: 'opus' }, expectedEnvironmentId: 'local' })
  })

  it('shows probe errors and never loads Settings when the environment lookup fails', async () => {
    transport.invoke.mockRejectedValue(new Error('environment lookup failed'))
    mount(<GrokSettingsView />)
    await screen.findByText('environment lookup failed')
    expect(mutation('grok_get_settings')).toHaveLength(0)
  })

  it.each([
    ['codex', <CodexSettingsView />, 'codex_get_config_raw_text'],
    ['claude-code', <ClaudeSettingsView />, 'claude_get_settings_raw_text'],
  ])('opens the existing raw editor from the %s route after plaintext confirmation', async (path, view, command) => {
    mount(view as ReactNode, '/' + path + '/settings')
    fireEvent.click(await screen.findByRole('button', { name: 'settingsRaw.sourceTab' }))
    await waitFor(() => expect(document.querySelector('.cm-editor')).toBeTruthy())
    expect(mutation(String(command))).toHaveLength(1)
    expect(transport.confirm).toHaveBeenCalledWith(expect.objectContaining({ title: 'settingsRaw.warningTitle' }))
  })

  it('preserves the raw draft on conflict, offers reload only and keeps policy/no-backup notices', async () => {
    mount(<GrokSettingsView />)
    expect(await screen.findByText('grok.settings.source.noBackup')).toBeTruthy()
    fireEvent.click(screen.getByRole('button', { name: 'settingsRaw.sourceTab' }))
    await editSource('[ui]\ntheme = "light"\n')
    rawContent = '[ui]\ntheme = "external"\n'
    rawToken = 'external'
    expect(screen.getByTestId('config-source-backup-notice').textContent).toContain('grok.settings.source.noBackup')
    expect(screen.getByTestId('config-source-policy-notice').textContent).toContain('grok.settings.source.policyNotice')
    fireEvent.click(screen.getByRole('button', { name: 'settingsRaw.save' }))
    await screen.findByText('settingsRaw.conflictTitle')
    expect(mutation('grok_save_config_raw_text')[0]?.[1]).toMatchObject({ content: '[ui]\ntheme = "light"\n', token: 'v1' })
    expect((screen.getByRole('button', { name: 'settingsRaw.save' }) as HTMLButtonElement).disabled).toBe(true)
    expect(rawReads).toBe(1)
    expect(document.querySelector('.cm-content')?.textContent).toContain('light')
    expect(rawContent).toBe('[ui]\ntheme = "external"\n')
    fireEvent.click(screen.getAllByRole('button', { name: 'settingsRaw.reload' })[0]!)
    await waitFor(() => expect(rawReads).toBe(2))
    expect(transport.error).not.toHaveBeenCalled()
    await waitFor(() => expect(document.querySelector('.cm-content')?.textContent).toContain('external'))
    expect(rawReads).toBe(2)
    expect(mutation('grok_save_config_raw_text')).toHaveLength(1)
  })

  it('leaves source mode and reloads the typed form after a successful save', async () => {
    mount(<GrokSettingsView />)
    fireEvent.click(await screen.findByRole('button', { name: 'settingsRaw.sourceTab' }))
    await editSource('[ui]\ntheme = "light"\n')
    settings.models.default = 'after-source'
    fireEvent.click(screen.getByRole('button', { name: 'settingsRaw.save' }))
    await waitFor(() => expect((screen.getByLabelText('grok.settings.fields.defaultModel') as HTMLInputElement).value).toBe('after-source'))
    expect(document.querySelector('.cm-editor')).toBeNull()
    expect(transport.confirm).toHaveBeenCalledTimes(1)
    expect(rawReads).toBe(1)
  })

  it('keeps raw invalid edits and displays the backend error marker without a success notice', async () => {
    rawStatus = 'invalid'
    mount(<GrokSettingsView />)
    fireEvent.click(await screen.findByRole('button', { name: 'settingsRaw.sourceTab' }))
    await editSource('[invalid')
    fireEvent.click(screen.getByRole('button', { name: 'settingsRaw.save' }))
    await screen.findByText('Invalid TOML')
    expect(document.querySelector('.cm-content')?.textContent).toContain('[invalid')
    expect(rawToken).toBe('v1')
    expect(rawReads).toBe(1)
    expect(transport.success).not.toHaveBeenCalled()
  })

  it('does not read source files in a remote environment on a platform with remote typed settings', async () => {
    environment = 'ssh'
    mount(<ClaudeSettingsView />, '/claude-code/settings')
    fireEvent.click(await screen.findByRole('button', { name: 'settingsRaw.sourceTab' }))
    await screen.findByText('settingsRaw.unsupportedEnvironment')
    expect(rawReads).toBe(0)
    expect(mutation('claude_list_settings_layers')).toHaveLength(0)
  })

  it('closes source mode without a filesystem request when plaintext access is declined', async () => {
    transport.confirm.mockResolvedValue(false)
    mount(<GrokSettingsView />)
    fireEvent.click(await screen.findByRole('button', { name: 'settingsRaw.sourceTab' }))
    await screen.findByLabelText('grok.settings.fields.defaultModel')
    expect(rawReads).toBe(0)
    expect(mutation('grok_list_config_layers')).toHaveLength(0)
  })

  it('waits for plaintext confirmation before reading raw content or layers', async () => {
    let confirm!: (accepted: boolean) => void
    transport.confirm.mockReturnValueOnce(new Promise<boolean>((resolve) => { confirm = resolve }))
    mount(<GrokSettingsView />)
    fireEvent.click(await screen.findByRole('button', { name: 'settingsRaw.sourceTab' }))
    await waitFor(() => expect(transport.confirm).toHaveBeenCalledTimes(1))
    expect(rawReads).toBe(0)
    expect(mutation('grok_list_config_layers')).toHaveLength(0)
    expect(document.querySelector('.cm-editor')).toBeNull()
    await act(async () => confirm(true))
    await waitFor(() => expect(document.querySelector('.cm-editor')).toBeTruthy())
    expect(rawReads).toBe(1)
  })

  it.each([
    ['model', 'claudeSettings.model.defaultModel', 'sonnet'],
    ['model', 'claudeSettings.model.maxThinkingTokens', '100'],
    ['model', 'claudeSettings.model.maxOutputTokens', '200'],
    ['ui', 'claudeSettings.ui.cleanupDays', '30'],
  ])('keeps the Claude draft and displays the removal limit for %s / %s', async (tab, label, initial) => {
    mount(<ClaudeSettingsView />, '/claude-code/settings')
    await screen.findByLabelText('claudeSettings.model.defaultModel')
    fireEvent.click(screen.getByText('claudeSettings.tabs.' + tab))
    const input = screen.getByLabelText(label) as HTMLInputElement
    await waitFor(() => expect(input.value).toBe(initial))
    fireEvent.change(input, { target: { value: '' } })
    fireEvent.click(screen.getByRole('button', { name: 'claudeSettings.save' }))
    expect(await screen.findByText(zhCN.settingsRaw.claudeClearUnsupported)).toBeTruthy()
    expect(input.value).toBe('')
    expect(mutation('claude_update_settings')).toHaveLength(0)
    expect(transport.success).not.toHaveBeenCalled()
    expect(screen.getByRole('button', { name: 'settingsRaw.sourceTab' })).toBeTruthy()
  })

  it.each([
    ['runtime', 'model', 'before'], ['runtime', 'smallModel', 'small'],
    ['runtime', 'defaultAgent', 'build'], ['tui', 'theme', 'dark'],
  ])('keeps the OpenCode draft and displays the removal limit for %s / %s', async (tab, field, initial) => {
    mount(<OpenCodeSettingsView />, '/opencode/settings')
    await screen.findByLabelText('opencode.settings.fields.model')
    fireEvent.click(screen.getByText('opencode.settings.tabs.' + tab))
    const input = screen.getByLabelText('opencode.settings.fields.' + field) as HTMLInputElement
    await waitFor(() => expect(input.value).toBe(initial))
    fireEvent.change(input, { target: { value: '' } })
    fireEvent.click(screen.getByRole('button', { name: 'opencode.settings.save' }))
    expect(await screen.findByText(zhCN.settingsRaw.opencodeClearUnsupported)).toBeTruthy()
    expect(input.value).toBe('')
    expect(mutation('opencode_update_settings')).toHaveLength(0)
    expect(mutation('opencode_update_tui_settings')).toHaveLength(0)
    expect(transport.success).not.toHaveBeenCalled()
  })
})
