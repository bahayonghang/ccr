import { readFile } from 'node:fs/promises'
import { describe, expect, it } from 'vitest'

const invokeCommands = (source: string): string[] =>
  Array.from(source.matchAll(/\binvoke\(\s*['"]([^'"]+)['"]/g), (match) => match[1])

describe('generated auth clients', () => {
  it('owns every Claude auth invoke without generic escape hatches', async () => {
    const source = await readFile('src/api/generated/claudeAuth.ts', 'utf8')

    expect(invokeCommands(source)).toEqual([
      'claude_list_auth_accounts',
      'claude_get_auth_current',
      'claude_save_auth',
      'claude_switch_auth',
      'claude_delete_auth',
    ])
    expect(source).not.toMatch(/<T\b|\bunknown\b|\bany\b/)
  })

  it('owns every Codex auth and provider invoke without generic escape hatches', async () => {
    const source = await readFile('src/api/generated/codexAuth.ts', 'utf8')

    expect(invokeCommands(source)).toEqual([
      'codex_list_auth_accounts',
      'codex_get_auth_current',
      'codex_save_auth',
      'codex_switch_auth',
      'codex_delete_auth',
      'codex_rename_auth',
      'codex_detect_process',
      'codex_oauth_login_start',
      'codex_oauth_login_completed',
      'codex_oauth_login_cancel',
      'codex_oauth_submit_callback_url',
      'codex_is_oauth_port_in_use',
      'codex_release_oauth_port',
      'codex_open_external_url',
      'codex_import_auth_payload',
      'codex_import_auth_from_local',
      'codex_add_auth_with_api_key',
      'codex_list_model_providers',
      'codex_save_model_provider',
      'codex_delete_model_provider',
    ])
    expect(source).not.toMatch(/<T\b|\bunknown\b|\bany\b/)
  })

  it('keeps migrated auth invokes out of handwritten domain wrappers', async () => {
    const [claude, codex] = await Promise.all([
      readFile('src/api/domains/claude.ts', 'utf8'),
      readFile('src/api/domains/codex.ts', 'utf8'),
    ])

    expect(claude).not.toMatch(/invoke\(['"]claude_(?:list_auth|get_auth|save_auth|switch_auth|delete_auth)/)
    expect(codex).not.toMatch(/invoke\(['"]codex_(?:list_auth|get_auth|save_auth|switch_auth|delete_auth|rename_auth|detect_process|oauth_|is_oauth|release_oauth|open_external|import_auth|add_auth|list_model_provider|save_model_provider|delete_model_provider)/)
  })
})
