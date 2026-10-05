import { beforeEach, describe, expect, it, vi } from 'vitest'
const transport = vi.hoisted(() => ({ invoke: vi.fn<(command: string, args?: { settings?: Record<string, unknown> }) => Promise<unknown>>() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: transport.invoke }))

import { claudeSettingsConfig, codexSettingsConfig, opencodeSettingsConfig } from '@/configs/settings'
import type { SettingsConfig } from '@/configs/settings'
import { saveSettingsValues } from '@/features/platform/settings-model'

let codex: Record<string, unknown>
let claude: Record<string, unknown>
let runtime: Record<string, unknown>
let tui: Record<string, unknown>

async function change(config: SettingsConfig, values: Record<string, string | boolean | number>) {
  const snapshot = await config.load()
  await saveSettingsValues(config, { values: { ...snapshot.values, ...values }, dirtyKeys: Object.keys(values), snapshot })
  return config.load()
}

function payload(command: string) {
  return transport.invoke.mock.calls.find(([id]) => id === command)?.[1]?.settings
}

function codexResponse() {
  const currentTui = codex.tui as Record<string, unknown>
  // codex_settings_to_json projects known fields; extension keys stay on disk.
  return structuredClone({
    model: codex.model ?? null, model_reasoning_effort: codex.model_reasoning_effort ?? null,
    model_context_window: codex.model_context_window ?? null,
    tui: {
      alternate_screen: currentTui.alternate_screen ?? null,
      animations: currentTui.animations ?? null,
      notifications: currentTui.notifications ?? null,
      show_tooltips: currentTui.show_tooltips ?? null,
    },
  })
}

function applyCodexPatch(patch: Record<string, unknown> = {}) {
  // apply_codex_settings_update delegates TUI updates to field-level merging.
  const { tui: tuiPatch, ...topLevel } = patch
  for (const [key, value] of Object.entries(topLevel)) {
    if (value === null) delete codex[key]
    else codex[key] = value
  }
  if (tuiPatch !== undefined) {
    const current = { ...codex.tui as Record<string, unknown> }
    const incoming = tuiPatch as Record<string, unknown>
    for (const key of ['alternate_screen', 'animations', 'notifications', 'show_tooltips']) {
      if (!(key in incoming)) continue
      if (incoming[key] === null) delete current[key]
      else current[key] = incoming[key]
    }
    codex.tui = current
  }
}

beforeEach(() => {
  codex = {
    model: 'before', model_reasoning_effort: 'future-effort',
    tui: { notifications: ['agent-turn-complete', 'approval-requested'], animations: true, custom: { keep: 7 } },
    experimental: { keep: [1, 2] },
  }
  claude = { model: 'before', permissions: { allow: ['Read'], future: { keep: true } }, future: ['keep'] }
  runtime = { model: 'before', autoupdate: 'notify', server: { port: 3000, hostname: 'localhost', future: [1, 2] }, future: { keep: true } }
  tui = { theme: 'dark', mouse: true, unknown: [1, 2] }
  transport.invoke.mockReset()
  transport.invoke.mockImplementation(async (command, args) => {
    switch (command) {
      case 'codex_get_settings': return codexResponse()
      case 'codex_update_settings': applyCodexPatch(args?.settings); return { message: 'saved' }
      case 'claude_get_settings': return structuredClone(claude)
      case 'claude_update_settings': claude = { ...claude, ...args?.settings }; return structuredClone(claude)
      case 'opencode_get_settings': return structuredClone(runtime)
      case 'opencode_get_tui_settings': return structuredClone(tui)
      case 'opencode_update_settings': runtime = { ...runtime, ...args?.settings }; return structuredClone(runtime)
      case 'opencode_update_tui_settings': tui = { ...tui, ...args?.settings }; return structuredClone(tui)
      default: throw new Error('Unexpected command: ' + command)
    }
  })
})

describe('Settings read-edit-save contract with real domain transport and mappers', () => {
  it('changes only Codex model and preserves notifications array, unknown enum and unknown fields', async () => {
    const before = structuredClone(codex)
    const after = await change(codexSettingsConfig, { model: 'after' })
    expect(payload('codex_update_settings')).toEqual({ model: 'after' })
    expect(codex).toEqual({ ...before, model: 'after' })
    expect(after.values.tuiNotifications).toBe(JSON.stringify(['agent-turn-complete', 'approval-requested']))
    expect(after.values.model_reasoning_effort).toBe('future-effort')
  })

  it('preserves unedited nested Codex fields and does not mutate the loaded snapshot', async () => {
    const snapshot = await codexSettingsConfig.load()
    const original = structuredClone(snapshot.source)
    await saveSettingsValues(codexSettingsConfig, { snapshot, values: { ...snapshot.values, tuiAnimations: false }, dirtyKeys: ['tuiAnimations'] })
    expect(payload('codex_update_settings')).toEqual({ tui: { animations: false } })
    expect(codex.tui).toEqual({ notifications: ['agent-turn-complete', 'approval-requested'], animations: false, custom: { keep: 7 } })
    expect(snapshot.source).toEqual(original)
  })

  it('does not resend stale Codex sibling values after an external edit', async () => {
    const snapshot = await codexSettingsConfig.load()
    codex.tui = { ...codex.tui as Record<string, unknown>, notifications: ['external-event'], custom: { keep: 8 } }
    await saveSettingsValues(codexSettingsConfig, { snapshot, values: { ...snapshot.values, tuiAnimations: false }, dirtyKeys: ['tuiAnimations'] })
    expect(payload('codex_update_settings')).toEqual({ tui: { animations: false } })
    expect(codex.tui).toEqual({ notifications: ['external-event'], animations: false, custom: { keep: 8 } })
  })

  it('allows explicit replacement of the notification union', async () => {
    await change(codexSettingsConfig, { tuiNotifications: 'false' })
    expect(codex.tui).toEqual({ notifications: false, animations: true, custom: { keep: 7 } })
  })

  it('clears optional Codex text, number, enum and nested leaves through explicit nulls', async () => {
    codex.model_context_window = 32000
    codex.tui = { ...codex.tui as Record<string, unknown>, alternate_screen: 'auto' }
    const after = await change(codexSettingsConfig, {
      model: '', model_context_window: '', model_reasoning_effort: '',
      tuiAlternateScreen: '', tuiNotifications: '',
    })
    expect(payload('codex_update_settings')).toEqual({
      model: null, model_context_window: null, model_reasoning_effort: null,
      tui: { alternate_screen: null, notifications: null },
    })
    expect(after.values).toMatchObject({
      model: '', model_context_window: '', model_reasoning_effort: '',
      tuiAlternateScreen: '', tuiNotifications: '',
    })
    expect(codex.tui).toEqual({ animations: true, custom: { keep: 7 } })
    expect(codex.experimental).toEqual({ keep: [1, 2] })
  })

  it('keeps OpenCode notify and never submits the unchanged runtime field or TUI file', async () => {
    const before = structuredClone(runtime)
    await change(opencodeSettingsConfig, { model: 'after' })
    expect(payload('opencode_update_settings')).toEqual({ model: 'after' })
    expect(payload('opencode_update_tui_settings')).toBeUndefined()
    expect(runtime).toEqual({ ...before, model: 'after' })
  })

  it('preserves unknown OpenCode nested keys under the shallow-merge backend contract', async () => {
    await change(opencodeSettingsConfig, { serverPort: 4444 })
    expect(payload('opencode_update_settings')).toEqual({ server: { port: 4444, hostname: 'localhost', future: [1, 2] } })
    expect(runtime.autoupdate).toBe('notify')
  })

  it('saves only the OpenCode TUI file when only a TUI field changes', async () => {
    await change(opencodeSettingsConfig, { mouse: false })
    expect(payload('opencode_update_settings')).toBeUndefined()
    expect(payload('opencode_update_tui_settings')).toEqual({ mouse: false })
    expect(tui).toEqual({ theme: 'dark', mouse: false, unknown: [1, 2] })
  })

  it('preserves Claude nested extensions when editing one permission field', async () => {
    await change(claudeSettingsConfig, { permAllow: 'Read\nEdit' })
    expect(payload('claude_update_settings')).toEqual({ permissions: { allow: ['Read', 'Edit'], future: { keep: true } } })
    expect(claude.future).toEqual(['keep'])
  })

  it.each(['model', 'maxThinkingTokens', 'maxOutputTokens', 'cleanupPeriodDays'])('rejects unsupported Claude top-level clearing of %s before transport', async (field) => {
    claude[field] = field === 'model' ? 'sonnet' : 100
    const before = structuredClone(claude)
    await expect(change(claudeSettingsConfig, { [field]: '' })).rejects.toThrow('settingsRaw.claudeClearUnsupported')
    expect(payload('claude_update_settings')).toBeUndefined()
    expect(claude).toEqual(before)
  })

  it.each(['model', 'smallModel', 'defaultAgent', 'theme'])('rejects unsupported OpenCode top-level clearing of %s before either file is saved', async (field) => {
    const before = { runtime: structuredClone(runtime), tui: structuredClone(tui) }
    await expect(change(opencodeSettingsConfig, { [field]: '', mouse: false })).rejects.toThrow('settingsRaw.opencodeClearUnsupported')
    expect(payload('opencode_update_settings')).toBeUndefined()
    expect(payload('opencode_update_tui_settings')).toBeUndefined()
    expect({ runtime, tui }).toEqual(before)
  })

  it('retains supported nested, list and object clearing for Claude and OpenCode', async () => {
    claude.permissions = { defaultMode: 'plan', allow: ['Read'], future: { keep: true } }
    claude.env = { EXAMPLE: 'synthetic' }
    await change(claudeSettingsConfig, { permDefaultMode: '', permAllow: '', envText: '' })
    expect(payload('claude_update_settings')).toEqual({ permissions: { allow: [], future: { keep: true } }, env: {} })
    runtime.instructions = ['synthetic.md']
    runtime.tools = { read: true }
    await change(opencodeSettingsConfig, { serverHostname: '', serverPort: '', toolsJson: '', instructionsText: '', keybindsJson: '' })
    expect(payload('opencode_update_settings')).toEqual({ server: { future: [1, 2] }, tools: {}, instructions: [] })
    expect(payload('opencode_update_tui_settings')).toEqual({ keybinds: {} })
  })

  it('does not issue any mutation when no field is dirty', async () => {
    await change(codexSettingsConfig, {})
    await change(opencodeSettingsConfig, {})
    await change(claudeSettingsConfig, {})
    expect(transport.invoke.mock.calls.filter(([id]) => String(id).includes('update'))).toEqual([])
  })
})
