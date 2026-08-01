import type {
  GrokSettingsCommandResponse,
  GrokSettingsPatchDto,
} from '@/types/grok'

export const GROK_REASONING_EFFORTS = [
  'none',
  'minimal',
  'low',
  'medium',
  'high',
  'xhigh',
  'max',
] as const

export const GROK_THEMES = ['system', 'light', 'dark'] as const
export const GROK_CHANNELS = ['stable', 'alpha'] as const
export const GROK_WORKTREE_MODES = ['ask', 'always', 'never'] as const

export const GROK_SETTINGS_KEYS = [
  'models.default',
  'models.default_reasoning_effort',
  'ui.theme',
  'session.auto_compact_threshold_percent',
  'session.load_envrc',
  'cli.auto_update',
  'cli.channel',
  'cli.show_tips',
  'hints.new_session_worktree_mode',
  'hints.fork_worktree_mode',
] as const

export type GrokSettingsKey = typeof GROK_SETTINGS_KEYS[number]
export type GrokSettingsFormValue = string | boolean | null
export type GrokSettingsForm = Record<GrokSettingsKey, GrokSettingsFormValue>
export type GrokSettingsOkResponse = Extract<GrokSettingsCommandResponse, { status: 'ok' }>

export const createEmptyGrokSettingsForm = (): GrokSettingsForm => ({
  'models.default': '',
  'models.default_reasoning_effort': '',
  'ui.theme': '',
  'session.auto_compact_threshold_percent': '',
  'session.load_envrc': null,
  'cli.auto_update': null,
  'cli.channel': '',
  'cli.show_tips': null,
  'hints.new_session_worktree_mode': '',
  'hints.fork_worktree_mode': '',
})

export const grokSettingsResponseToForm = (
  response: GrokSettingsOkResponse,
): GrokSettingsForm => ({
  'models.default': response.models.default ?? '',
  'models.default_reasoning_effort': response.models.default_reasoning_effort ?? '',
  'ui.theme': response.ui.theme ?? '',
  'session.auto_compact_threshold_percent': response.session.auto_compact_threshold_percent === null
    ? ''
    : String(response.session.auto_compact_threshold_percent),
  'session.load_envrc': response.session.load_envrc,
  'cli.auto_update': response.cli.auto_update,
  'cli.channel': response.cli.channel ?? '',
  'cli.show_tips': response.cli.show_tips,
  'hints.new_session_worktree_mode': response.hints.new_session_worktree_mode ?? '',
  'hints.fork_worktree_mode': response.hints.fork_worktree_mode ?? '',
})

export const validateGrokSettingsForm = (
  form: GrokSettingsForm,
  dirtyKeys: ReadonlySet<GrokSettingsKey>,
): GrokSettingsKey | null => {
  const key: GrokSettingsKey = 'session.auto_compact_threshold_percent'
  if (!dirtyKeys.has(key)) return null

  const value = form[key]
  if (value === null || value === '') return null
  const parsed = Number(value)
  return Number.isInteger(parsed) && parsed >= 0 && parsed <= 100 ? null : key
}

export const buildGrokSettingsPatch = (
  form: GrokSettingsForm,
  dirtyKeys: ReadonlySet<GrokSettingsKey>,
): GrokSettingsPatchDto => {
  const set: GrokSettingsPatchDto['set'] = {}
  const unset: string[] = []

  for (const key of GROK_SETTINGS_KEYS) {
    if (!dirtyKeys.has(key)) continue
    const value = form[key]
    if (value === null || (typeof value === 'string' && value.trim() === '')) {
      unset.push(key)
      continue
    }

    if (key === 'session.auto_compact_threshold_percent') {
      set[key] = Number(value)
    } else {
      set[key] = typeof value === 'string' ? value.trim() : value
    }
  }

  return { set, unset }
}
