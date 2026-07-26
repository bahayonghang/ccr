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
  invokeMock.mockImplementation(async () => structuredClone(richResponse))
})

describe('API facade execution coverage', () => {
  it('executes every exported command wrapper against a deterministic invoke boundary', async () => {
    const exercised: string[] = []
    const rejected: string[] = []

    for (const [namespaceName, namespace] of namespaces) {
      for (const [exportName, value] of Object.entries(namespace)) {
        if (typeof value !== 'function') continue
        exercised.push(`${namespaceName}.${exportName}`)
        try {
          await value(...argsFor(exportName))
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
