import { describe, expect, it, vi } from 'vitest'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async () => ({})),
}))

import { invoke } from '@tauri-apps/api/core'
import * as api from '@/api'
import { claudeObserver } from '@/api/domains/claudeObserver'
import { convertConfig } from '@/api/domains/converter'
import { getRecentEvents, getRuntimeMetrics } from '@/api/domains/events'
import { llmusageInstallCheck, llmusageInstallDetect } from '@/api/domains/install'
import { getMonitoringFeed } from '@/api/domains/monitoring'
import { getClaudeSettings, listClaudeProfiles } from '@/api/domains/platforms'
import { getFavorites, getRecentItems } from '@/api/domains/uiState'
import { getUsageDashboardV2, getUsageSummaryV2 } from '@/api/domains/usage'

describe('idle API domain wrappers', () => {
  it('calls shipped exports from each idle domain module', async () => {
    await expect(claudeObserver.cacheStats()).resolves.toEqual({})
    await expect(claudeObserver.getInsight()).resolves.toEqual({})
    await expect(convertConfig({
      source_format: 'claude-code',
      target_format: 'codex',
      config_data: '{}',
    })).resolves.toEqual({})
    await expect(getRecentEvents()).resolves.toEqual({})
    await expect(getRuntimeMetrics()).resolves.toEqual({})
    await expect(llmusageInstallDetect()).resolves.toEqual({})
    await expect(llmusageInstallCheck()).resolves.toEqual({})
    await expect(getMonitoringFeed()).resolves.toEqual({})
    await expect(getClaudeSettings()).resolves.toEqual({})
    await expect(listClaudeProfiles()).resolves.toEqual({
      profiles: [],
      current_profile: null,
      can_off: false,
    })
    await expect(getFavorites()).resolves.toEqual({})
    await expect(getRecentItems()).resolves.toEqual({})
    await expect(getUsageSummaryV2()).resolves.toEqual({})
    await expect(getUsageDashboardV2()).resolves.toEqual({})
    await expect(api.platformApi.getGeminiConfig()).resolves.toEqual({})
    await expect(api.usageApi.getUsageTrendsV2()).resolves.toEqual({})
    expect(api.claudeObserver).toBe(claudeObserver)
  })

  it('touches unused barrel wrappers that still have uncovered bodies', async () => {
    vi.mocked(invoke).mockImplementation(async (command, rawArgs) => {
      const args = (rawArgs ?? {}) as Record<string, unknown>
      if (command === 'gemini_list_mcp_servers') {
        return [
          {
            name: 'ok',
            command: 'npx',
            args: ['a', 1],
            env: { K: 'v', n: 1 },
            cwd: '/tmp',
            timeout: 1,
            trust: true,
            includeTools: ['t', 2],
            url: 'https://example.invalid',
          },
          { name: 1 },
          null,
          ['x'],
        ]
      }
      if (command === 'list_configs') return [{ name: 'sample' }]
      if (command === 'unified_add_mcp_server') {
        const request = (args.request ?? {}) as { fail?: boolean }
        if (request.fail) throw new Error('add failed')
        return { message: 'ok' }
      }
      if (command === 'unified_delete_mcp_server') throw new Error('missing')
      return {}
    })

    await expect(api.getSkipExitConfirm()).resolves.toBe(false)
    await api.setSkipExitConfirm(true)
    await expect(api.getSkipExitConfirm()).resolves.toBe(true)
    await expect(api.getConfig('sample')).resolves.toEqual({ name: 'sample' })
    await expect(api.getConfig('missing')).resolves.toBeNull()
    await expect(api.installMcpPreset({
      preset_id: 'p1',
      env: { TOKEN: 't', skip: 1 },
    }, ['claude'])).resolves.toEqual({})
    await expect(api.installMcpPresetSingle({
      id: 'p2',
      platform: 'claude',
      env: { TOKEN: 't', skip: 1 },
    })).resolves.toEqual({})
    await expect(api.importUnifiedMcpServers([
      { name: 'ok-server' },
      { name: 'bad-server', fail: true },
    ])).resolves.toEqual([
      { name: 'ok-server', ok: true, message: 'ok' },
      { name: 'bad-server', ok: false, error: 'add failed' },
    ])
    await expect(api.updateUnifiedMcp('codex', 'srv', { scope: 'user' })).resolves.toEqual({
      message: 'ok',
    })
    await expect(api.toggleUnifiedMcp('codex', 'srv')).rejects.toThrow('不支持')
    await expect(api.listGeminiMcpServers()).resolves.toEqual([
      {
        name: 'ok',
        command: 'npx',
        args: ['a'],
        env: { K: 'v' },
        cwd: '/tmp',
        timeout: 1,
        trust: true,
        includeTools: ['t'],
        url: 'https://example.invalid',
      },
    ])
    expect(api.grokApi.isGrokDashboardResponse(null)).toBe(false)
    expect(api.grokApi.isGrokDashboardResponse({ status: 'ok' })).toBe(true)
  })
})
