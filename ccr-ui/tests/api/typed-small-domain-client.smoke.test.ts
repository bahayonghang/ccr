import { readFile } from 'node:fs/promises'
import { describe, expect, it } from 'vitest'

const invokeCommands = (source: string): string[] =>
  Array.from(source.matchAll(/\binvoke\(\s*['"]([^'"]+)['"]/g), (match) => match[1])

const stripTypeScriptComments = (source: string): string =>
  source.replace(/\/\*[\s\S]*?\*\//g, '').replace(/(^|[^:])\/\/.*$/gm, '$1')

describe('generated small-domain clients', () => {
  it.each([
    ['uiState.ts', [
      'get_favorites',
      'add_favorite',
      'remove_favorite',
      'get_recent_items',
      'add_recent_item',
      'clear_recent_items',
    ]],
    ['systemInfo.ts', ['get_system_info', 'check_version']],
    ['converter.ts', ['convert_config']],
    ['exitConfirm.ts', ['get_skip_exit_confirm', 'set_skip_exit_confirm']],
    ['environment.ts', [
      'list_environments',
      'get_current_environment',
      'switch_environment',
      'refresh_environments',
    ]],
    ['events.ts', [
      'get_recent_events',
      'get_monitoring_feed',
      'append_frontend_logs',
      'get_runtime_metrics',
    ]],
    ['shell.ts', [
      'shell_get_preferences',
      'shell_set_preferences',
      'shell_show_main_window',
      'shell_request_quit',
      'shell_begin_tray_panel_drag',
      'shell_complete_tray_panel_drag',
      'shell_detect_skillport_app',
      'shell_open_skillport_app',
      'shell_detect_skills_manage_app',
      'shell_open_skills_manage_app',
    ]],
    ['systemExtended.ts', ['get_cli_versions', 'get_cli_version']],
    ['builtinPrompts.ts', [
      'list_builtin_prompts',
      'get_builtin_prompt',
      'get_builtin_prompts_by_category',
    ]],
  ] as const)('owns every %s invoke without generic escape hatches', async (file, commands) => {
    const source = await readFile(`src/api/generated/${file}`, 'utf8')

    expect(invokeCommands(source)).toEqual(commands)
    expect(source).not.toMatch(/<T\b|\bunknown\b|\bany\b/)
  })

  it('keeps migrated invokes out of handwritten wrappers', async () => {
    const [uiState, system, converter, environment, events, monitoring, logger, tauri, runtimeSystem, runtimeEnvironment] = await Promise.all([
      readFile('src/api/domains/uiState.ts', 'utf8'),
      readFile('src/api/domains/system.ts', 'utf8'),
      readFile('src/api/domains/converter.ts', 'utf8'),
      readFile('src/api/domains/environment.ts', 'utf8'),
      readFile('src/api/domains/events.ts', 'utf8'),
      readFile('src/api/domains/monitoring.ts', 'utf8'),
      readFile('src/utils/logger.ts', 'utf8'),
      readFile('src/api/tauri.ts', 'utf8'),
      readFile('src/api/runtime/system.ts', 'utf8'),
      readFile('src/api/runtime/environment.ts', 'utf8'),
    ])
    const handwritten = [uiState, system, converter, environment, events, monitoring, logger, tauri, runtimeSystem, runtimeEnvironment]
      .map(stripTypeScriptComments)
      .join('\n')

    expect(handwritten).not.toMatch(/invoke\(['"](?:get_favorites|add_favorite|remove_favorite|get_recent_items|add_recent_item|clear_recent_items|get_system_info|check_version|convert_config|get_skip_exit_confirm|set_skip_exit_confirm|list_environments|get_current_environment|switch_environment|refresh_environments|get_recent_events|get_monitoring_feed|append_frontend_logs|get_runtime_metrics|get_cli_versions|get_cli_version|list_builtin_prompts|get_builtin_prompt|get_builtin_prompts_by_category|shell_)/)
  })
})
