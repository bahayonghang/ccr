import { readFile } from 'node:fs/promises'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.hoisted(() => vi.fn())

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

import { listCodexProfiles, updateCodexConfig } from '@/api/domains/codex'
import { listSystemPrompts, saveSystemPrompt } from '@/api/domains/systemPrompts'

const invokeCommands = (source: string): string[] =>
  Array.from(source.matchAll(/\binvoke\(\s*['"]([^'"]+)['"]/g), match => match[1])

describe('typed Codex and system prompts clients', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('owns every migrated invoke in concrete generated clients', async () => {
    const [codex, systemPrompts] = await Promise.all([
      readFile('src/api/generated/codex.ts', 'utf8'),
      readFile('src/api/generated/systemPrompts.ts', 'utf8'),
    ])

    expect(invokeCommands(codex)).toHaveLength(44)
    expect(invokeCommands(codex)).toEqual(expect.arrayContaining([
      'codex_list_profiles',
      'codex_profile_off',
      'codex_update_settings',
      'codex_validate_agent_toml',
      'codex_sync_source_install',
      'codex_get_dashboard_usage_summary',
      'codex_get_quota',
    ]))
    expect(invokeCommands(systemPrompts)).toEqual([
      'system_prompts_list',
      'system_prompts_get',
      'system_prompts_save',
      'system_prompts_create',
    ])
    expect(`${codex}\n${systemPrompts}`).not.toMatch(/<T\b|\bunknown\b|\bany\b/)
  })

  it('keeps migrated invokes and generic result escapes out of handwritten domains', async () => {
    const [codex, systemPrompts] = await Promise.all([
      readFile('src/api/domains/codex.ts', 'utf8'),
      readFile('src/api/domains/systemPrompts.ts', 'utf8'),
    ])

    expect(invokeCommands(codex)).toEqual([
      'codex_get_config_raw_text',
      'codex_save_config_raw_text',
      'codex_list_config_layers',
    ])
    expect(invokeCommands(systemPrompts)).toEqual([])
    expect(codex).not.toMatch(/export const \w+ = async <T/)
  })

  it('projects discriminated results and rejects non-JSON command input', async () => {
    invokeMock.mockImplementation(async (command: string) => {
      if (command === 'codex_list_profiles') {
        return { profiles: [{ name: 'official', enabled: true }], current_profile: 'official' }
      }
      if (command === 'system_prompts_list') {
        return {
          status: 'ok',
          files: [{
            id: 'codex-agents',
            labelKey: 'systemPrompts.files.codexAgents',
            path: 'C:/Users/test/.codex/AGENTS.md',
            exists: true,
            size: 42,
            mtime: null,
            editable: true,
            limitHint: 32768,
          }],
          rules: [],
        }
      }
      if (command === 'system_prompts_save') {
        return { status: 'unsupported_environment', envType: 'ssh' }
      }
      throw new Error(`Unexpected command: ${command}`)
    })

    await expect(listCodexProfiles()).resolves.toMatchObject({
      profiles: [{ name: 'official' }],
      current_profile: 'official',
    })
    await expect(listSystemPrompts('codex')).resolves.toMatchObject({
      status: 'ok',
      files: [{ id: 'codex-agents', limitHint: 32768 }],
    })
    await expect(saveSystemPrompt('codex', 'codex-agents', 'text', 'token')).resolves.toEqual({
      status: 'unsupported_environment',
      envType: 'ssh',
    })
    await expect(updateCodexConfig({ invalid: BigInt(1) })).rejects.toThrow(
      'Codex command payload must be JSON-compatible',
    )
  })
})
