import { readFile } from 'node:fs/promises'
import { describe, expect, it } from 'vitest'

const invokeCommands = (source: string): string[] =>
  Array.from(source.matchAll(/\binvoke\(\s*['"]([^'"]+)['"]/g), (match) => match[1])

describe('generated open-config clients', () => {
  it.each([
    ['gemini.ts', [
      'gemini_get_settings',
      'gemini_update_settings',
      'gemini_list_mcp_servers',
      'gemini_add_mcp_server',
      'gemini_update_mcp_server',
      'gemini_delete_mcp_server',
      'gemini_list_slash_commands',
      'gemini_add_slash_command',
      'gemini_update_slash_command',
      'gemini_delete_slash_command',
      'gemini_list_extensions',
    ]],
    ['openCode.ts', [
      'opencode_get_settings',
      'opencode_update_settings',
      'opencode_get_tui_settings',
      'opencode_update_tui_settings',
      'opencode_get_keybindings',
      'opencode_update_keybindings',
      'opencode_list_themes',
      'opencode_list_agents',
      'opencode_add_agent',
      'opencode_update_agent',
      'opencode_delete_agent',
      'opencode_list_commands',
      'opencode_add_command',
      'opencode_update_command',
      'opencode_delete_command',
      'opencode_list_local_plugins',
    ]],
  ] as const)('owns every %s invoke with concrete generated types', async (file, commands) => {
    const source = await readFile(`src/api/generated/${file}`, 'utf8')

    expect(invokeCommands(source)).toEqual(commands)
    expect(source).not.toMatch(/<T\b|\bunknown\b|\bany\b/)
  })

  it('keeps migrated invokes out of handwritten domains', async () => {
    const sources = await Promise.all([
      readFile('src/api/domains/gemini.ts', 'utf8'),
      readFile('src/api/domains/opencode.ts', 'utf8'),
    ])

    expect(sources.join('\n')).not.toMatch(/invoke\(['"](?:gemini_|opencode_)/)
  })
})
