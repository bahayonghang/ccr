import { beforeEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.hoisted(() => vi.fn())
const runtimeMock = vi.hoisted(() => vi.fn(() => true))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

vi.mock('@/utils/tauriRuntime', () => ({
  isTauriRuntime: runtimeMock,
}))

import * as tauriApi from '@/api/tauri'
import * as checkinApi from '@/api/domains/checkin'
import * as claudeApi from '@/api/domains/claude'
import * as claudeObserverApi from '@/api/domains/claudeObserver'
import * as codexApi from '@/api/domains/codex'
import * as configApi from '@/api/domains/config'
import * as converterApi from '@/api/domains/converter'
import * as environmentApi from '@/api/domains/environment'
import * as eventsApi from '@/api/domains/events'
import * as geminiApi from '@/api/domains/gemini'
import * as installApi from '@/api/domains/install'
import * as monitoringApi from '@/api/domains/monitoring'
import * as opencodeApi from '@/api/domains/opencode'
import * as statsApi from '@/api/domains/stats'
import * as syncApi from '@/api/domains/sync'
import * as systemApi from '@/api/domains/system'
import * as systemPromptsApi from '@/api/domains/systemPrompts'
import * as uiStateApi from '@/api/domains/uiState'
import * as unifiedMcpApi from '@/api/domains/unifiedMcp'
import * as usageApi from '@/api/domains/usage'
import * as wafApi from '@/api/domains/waf'
import * as runtimeEnvironmentApi from '@/api/runtime/environment'
import * as runtimeSystemApi from '@/api/runtime/system'
import * as runtimeWslApi from '@/api/runtime/wsl'

const richResponse = {
  accounts: [],
  agents: [],
  capabilities: [],
  commands: {},
  diagnostics: [],
  hooks: {},
  items: [],
  mcp: {},
  mcp_servers: {},
  mcpServers: {},
  plugins: [],
  profiles: [],
  provider: {},
  providers: {},
  records: [],
  results: [],
  servers: [],
  settings: {},
  sources: [],
  styles: [],
  themes: [],
  total: 0,
}

const responseForCommand = (command: string): unknown => {
  if (command === 'list_configs') return []
  if (command === 'claude_get_profiles_raw' || command === 'codex_get_profiles_raw') {
    return { status: 'ok', content: '', token: 'token', path: 'profiles.toml', exists: true }
  }
  if (command === 'claude_save_profiles_raw' || command === 'codex_save_profiles_raw') {
    return { status: 'saved', token: 'token', profiles_count: 0 }
  }
  if (command === 'codex_export_profiles') return { content: '', filename: 'profiles.toml' }
  if (command === 'codex_list_agents') return { context: {}, agents: [], diagnostics: [] }
  if (
    command === 'codex_add_agent'
    || command === 'codex_update_agent'
    || command === 'codex_rename_agent'
    || command === 'codex_copy_agent'
    || command === 'codex_validate_agent_toml'
  ) {
    return { context: {}, agent: {} }
  }
  if (command === 'codex_add_agent_source') {
    return { id: 'source', repoUrl: 'https://example.invalid/repo', owner: 'owner', repo: 'repo' }
  }
  if (command === 'codex_get_agent_source_catalog') {
    return {
      source: {
        id: 'source',
        repoUrl: 'https://example.invalid/repo',
        owner: 'owner',
        repo: 'repo',
      },
      agents: [],
      diagnostics: [],
      installs: [],
    }
  }
  if (command === 'codex_list_models') {
    return { models: [], builtin_models: [], custom_models: [] }
  }
  if (command === 'codex_list_sessions') return { sessions: [] }
  if (command === 'codex_get_session_detail') {
    return { session: {}, messages: [], clipped: false, message_limit: 0 }
  }
  if (command === 'codex_export_session') {
    return {
      session_id: 'session',
      file_name: 'session.json',
      content: '',
      truncated: false,
      max_messages: 0,
    }
  }
  if (command === 'codex_clone_session') return { message: 'ok', session: {} }
  if (command === 'codex_get_tray_snapshot') {
    return { fetched_at: '2026-01-01T00:00:00Z', runtime_mode: 'local', accounts: [] }
  }
  if (command === 'codex_get_dashboard_overview') {
    return { auth: {}, profiles: {}, config: {}, inventory: {} }
  }
  if (command === 'codex_get_dashboard_usage_summary') {
    return { freshness: 'empty', five_hour: {}, seven_day: {}, all_time: {} }
  }
  if (command === 'codex_get_usage') {
    return { five_hour: {}, seven_day: {}, all_time: {}, by_model: {} }
  }
  if (command === 'codex_get_all_quotas') return []
  if (command === 'codex_get_quota') {
    return { account_name: 'sample', fetched_at: '2026-01-01T00:00:00Z' }
  }
  if (
    command === 'opencode_list_themes'
    || command === 'opencode_list_local_plugins'
  ) return []
  if (command === 'opencode_add_agent' || command === 'opencode_update_agent') {
    return { name: 'sample', path: 'agent.md', scope: 'global', body: '' }
  }
  if (command === 'opencode_add_command' || command === 'opencode_update_command') {
    return { name: 'sample', path: 'command.md', scope: 'global', template: '' }
  }
  if (command === 'system_prompts_list') return { status: 'ok', files: [], rules: [] }
  if (command === 'system_prompts_get') {
    return { status: 'ok', content: '', token: 'token', path: 'AGENTS.md', exists: true }
  }
  if (command === 'system_prompts_save' || command === 'system_prompts_create') {
    return { status: 'saved', token: 'token' }
  }
  return richResponse
}

const genericRequest = {
  id: 'sample',
  name: 'sample',
  platform: 'claude',
  scope: 'user',
  content: '{}',
  token: 'token',
  config: {},
  settings: {},
  accountIds: [],
  paths: [],
  enabled: true,
}

const namespaces: Array<[string, Record<string, unknown>]> = [
  ['tauri', tauriApi],
  ['checkin', checkinApi],
  ['claude', claudeApi],
  ['claudeObserver', claudeObserverApi],
  ['codex', codexApi],
  ['config', configApi],
  ['converter', converterApi],
  ['environment', environmentApi],
  ['events', eventsApi],
  ['gemini', geminiApi],
  ['install', installApi],
  ['monitoring', monitoringApi],
  ['opencode', opencodeApi],
  ['stats', statsApi],
  ['sync', syncApi],
  ['system', systemApi],
  ['systemPrompts', systemPromptsApi],
  ['uiState', uiStateApi],
  ['unifiedMcp', unifiedMcpApi],
  ['usage', usageApi],
  ['waf', wafApi],
  ['runtimeEnvironment', runtimeEnvironmentApi],
  ['runtimeSystem', runtimeSystemApi],
  ['runtimeWsl', runtimeWslApi],
]

const argsFor = (name: string): unknown[] => {
  if (name === 'toggleUnifiedMcp') return ['claude', 'sample', true, 'user']
  if (name.toLowerCase().includes('import') || name.toLowerCase().includes('batch')) {
    return [[], genericRequest, 'user']
  }
  if (name.toLowerCase().includes('ids')) return [[], genericRequest]
  return [genericRequest, genericRequest, 'user', true]
}

beforeEach(() => {
  vi.clearAllMocks()
  runtimeMock.mockReturnValue(true)
  invokeMock.mockImplementation(async (command: string) => structuredClone(responseForCommand(command)))
})

describe('API facade execution coverage', () => {
  it('executes every exported command wrapper against a deterministic invoke boundary', async () => {
    const exercised: string[] = []
    const rejected: string[] = []

    for (const [namespaceName, namespace] of namespaces) {
      for (const [exportName, value] of Object.entries(namespace)) {
        if (typeof value !== 'function') continue
        exercised.push(`${namespaceName}.${exportName}`)
        const command = value as (...args: unknown[]) => unknown
        try {
          await command(...argsFor(exportName))
        } catch {
          rejected.push(`${namespaceName}.${exportName}`)
        }
      }
    }

    expect(exercised.length).toBeGreaterThan(300)
    expect(invokeMock.mock.calls.length).toBeGreaterThan(250)
    expect(invokeMock.mock.calls.every(([command]) => typeof command === 'string')).toBe(true)
    expect(rejected).toEqual([
      'tauri.toggleCodexAgent',
      'codex.toggleCodexAgent',
    ])
  })

  it('keeps runtime wrappers inert in web mode instead of invoking desktop commands', async () => {
    runtimeMock.mockReturnValue(false)

    await expect(runtimeEnvironmentApi.listEnvironments()).rejects.toThrow(
      'Tauri runtime is unavailable for list_environments'
    )
    expect(tauriApi.getEnvironmentName()).toBe('web')
    await expect(tauriApi.getTauriVersion()).resolves.toBeNull()
    expect(invokeMock).not.toHaveBeenCalled()
  })
})
