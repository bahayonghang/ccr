import { readFile } from 'node:fs/promises'
import { describe, expect, it } from 'vitest'

const TAURI_FACADE_PATH = 'src/api/tauri.ts'

const COMPATIBILITY_MARKERS = [
  'Tauri API compatibility facade for CCR Desktop',
  'Compatibility-only',
  'Do not add new business API wrappers or direct `invoke()` calls',
  'New APIs must live in `src/api/domains/*`',
] as const

const ALLOWED_TAURI_FACADE_COMMANDS = [
  'get_skip_exit_confirm',
  'set_skip_exit_confirm',
  'execute_ccr_command',
  'list_ccr_commands',
  'get_ccr_command_help',
  'start_ccr_command_job',
  'get_ccr_command_job_status',
  'cancel_ccr_command_job',
  'switch_config',
  'update_config',
  'list_configs',
  'list_mcp_presets',
  'get_mcp_preset',
  'install_mcp_preset',
  'install_mcp_preset_single',
  'list_source_mcp_servers',
  'sync_mcp_server',
  'sync_all_mcp_servers',
  'list_builtin_prompts',
  'get_builtin_prompt',
  'get_builtin_prompts_by_category',
  'health_check',
  'claude_observer_get_insight',
  'claude_observer_daily_trend',
  'claude_observer_cost_breakdown',
  'claude_observer_cache_stats',
  'claude_observer_top_sessions',
  'claude_observer_tool_heatmap',
  'claude_observer_top_tools',
  'claude_observer_subscription_get',
  'claude_observer_subscription_set',
] as const

const stripTypeScriptComments = (source: string): string => {
  return source
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/(^|[^:])\/\/.*$/gm, '$1')
}

const extractInvokeCommands = (source: string): string[] => {
  const code = stripTypeScriptComments(source)

  return Array.from(code.matchAll(/\binvoke(?:<[^>]+>)?\(\s*['"]([^'"]+)['"]/g), (match) => match[1])
}

describe('API facade boundary', () => {
  it('marks tauri.ts as a compatibility-only facade', async () => {
    const source = await readFile(TAURI_FACADE_PATH, 'utf8')

    for (const marker of COMPATIBILITY_MARKERS) {
      expect(source).toContain(marker)
    }
  })

  it('freezes legacy direct invoke calls in tauri.ts', async () => {
    const source = await readFile(TAURI_FACADE_PATH, 'utf8')

    expect(extractInvokeCommands(source)).toEqual(ALLOWED_TAURI_FACADE_COMMANDS)
  })
})
