import { readFile } from 'node:fs/promises'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { COMMAND_MANIFEST } from '@/api/generated/commandCapabilities'
import { collectInvokeCommandsFromDir } from '../helpers/apiInvokeScan'

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

interface ManifestFile {
  schema_version: number
  base_command_count: number
  windows_command_count: number
  typed_command_count: number
  commands: Array<{ id: string; platform: 'base' | 'windows' }>
}

const generatedClients = import.meta.glob('../../src/api/generated/*.ts', { eager: true }) as Record<
  string,
  Record<string, unknown>
>

const generatedClientArgs = (exportName: string): unknown[] => {
  if (exportName.toLowerCase().includes('ids') || exportName.toLowerCase().includes('batch')) {
    return [[], 'user']
  }
  return [genericRequest, genericRequest, 'user', true]
}

/**
 * 清单内有注册、但 src/api 无 wrapper 的命令。
 * waf_deliver_cookie：WebView 注入脚本经 __TAURI_INTERNALS__.invoke 调用，不是前端 API。
 * wsl_write_config：Windows 命令，runtime/wsl.ts 尚未提供 wrapper（批次 1 跟踪缺口）。
 */
const UNWRAPPED_MANIFEST_COMMANDS = ['waf_deliver_cookie', 'wsl_write_config'] as const

describe('API facade command-name coverage', () => {
  it('keeps command-manifest.json aligned with the generated COMMAND_MANIFEST client', async () => {
    const file = JSON.parse(
      await readFile('src/api/generated/command-manifest.json', 'utf8'),
    ) as ManifestFile

    expect(file.schema_version).toBe(2)
    expect(file.base_command_count).toBe(339)
    expect(file.windows_command_count).toBe(347)
    expect(COMMAND_MANIFEST.schema_version).toBe(file.schema_version)
    expect(COMMAND_MANIFEST.base_command_count).toBe(file.base_command_count)
    expect(COMMAND_MANIFEST.windows_command_count).toBe(file.windows_command_count)
    expect(COMMAND_MANIFEST.typed_command_count).toBe(file.typed_command_count)
    expect(COMMAND_MANIFEST.commands.map((command) => command.id)).toEqual(
      file.commands.map((command) => command.id),
    )
    expect(file.commands.filter((command) => command.platform === 'base')).toHaveLength(339)
    expect(file.commands.filter((command) => command.platform === 'windows')).toHaveLength(8)
  })

  it('covers every required manifest command with a shipped src/api wrapper', async () => {
    const file = JSON.parse(
      await readFile('src/api/generated/command-manifest.json', 'utf8'),
    ) as ManifestFile
    const wrapped = await collectInvokeCommandsFromDir('src/api')
    const required = file.commands.filter(
      (command) =>
        command.platform === 'base'
        || (process.platform === 'win32' && command.platform === 'windows'),
    )
    const requiredIds = required.map((command) => command.id)
    const missing = requiredIds.filter((id) => !wrapped.has(id))
    const expectedMissing = UNWRAPPED_MANIFEST_COMMANDS.filter((id) => requiredIds.includes(id))
    const extra = [...wrapped].filter(
      (id) => !file.commands.some((command) => command.id === id),
    )

    expect(missing.sort()).toEqual([...expectedMissing].sort())
    expect(extra).toEqual([])
  })

  it('drives generated command clients so typed wrappers actually invoke', async () => {
    const exercised = new Set<string>()
    invokeMock.mockImplementation(async (command: string) => {
      exercised.add(command)
      return structuredClone(responseForCommand(command))
    })

    const callWrapper = async (name: string, value: unknown): Promise<void> => {
      if (typeof value === 'function') {
        try {
          await (value as (...args: unknown[]) => unknown)(...generatedClientArgs(name))
        } catch {
          // 部分 wrapper 在 invoke 前校验参数；仍以 mock 是否被调用为准。
        }
        return
      }
      if (value && typeof value === 'object') {
        for (const [childName, child] of Object.entries(value as Record<string, unknown>)) {
          await callWrapper(childName, child)
        }
      }
    }

    for (const [modulePath, namespace] of Object.entries(generatedClients)) {
      if (modulePath.endsWith('commandCapabilities.ts')) continue
      for (const [exportName, value] of Object.entries(namespace)) {
        await callWrapper(exportName, value)
      }
    }

    const typedIds = COMMAND_MANIFEST.commands
      .filter((command) => command.input_schema === 'generated' && command.output_schema === 'generated')
      .filter(
        (command) =>
          command.platform === 'base'
          || (process.platform === 'win32' && command.platform === 'windows'),
      )
      .map((command) => command.id)
    const missingTyped = typedIds.filter((id) => !exercised.has(id))

    expect(typedIds.length).toBeGreaterThan(200)
    expect(missingTyped).toEqual([])
  })
})
