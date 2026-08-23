import { getGrokSettings, updateGrokSettings } from '@/api/domains/grok'
import { probeLocalEnvironment } from '@/configs/probeLocal'
import { boolField, selectField, textField } from '@/configs/settings-helpers'
import { surfaceNotify } from '@/configs/surfaceNotify'
import type { SettingsConfig, SettingsValues } from '@/configs/settings-types'
import type { GrokSettingsForm, GrokSettingsKey } from '@/utils/grokSettings'
import {
  GROK_CHANNELS,
  GROK_REASONING_EFFORTS,
  GROK_THEMES,
  GROK_WORKTREE_MODES,
  buildGrokSettingsPatch,
  createEmptyGrokSettingsForm,
  grokSettingsResponseToForm,
  validateGrokSettingsForm,
} from '@/utils/grokSettings'

const GROK_ID_TO_KEY: Record<string, GrokSettingsKey> = {
  models_default: 'models.default',
  models_default_reasoning_effort: 'models.default_reasoning_effort',
  ui_theme: 'ui.theme',
  session_auto_compact_threshold_percent: 'session.auto_compact_threshold_percent',
  session_load_envrc: 'session.load_envrc',
  cli_auto_update: 'cli.auto_update',
  cli_channel: 'cli.channel',
  cli_show_tips: 'cli.show_tips',
  hints_new_session_worktree_mode: 'hints.new_session_worktree_mode',
  hints_fork_worktree_mode: 'hints.fork_worktree_mode',
}

const optionsOf = (values: readonly string[]) => values.map((value) => ({ value, labelKey: value }))

const grokFormFromValues = (values: SettingsValues): GrokSettingsForm => {
  const form = createEmptyGrokSettingsForm()
  for (const [id, key] of Object.entries(GROK_ID_TO_KEY)) {
    const raw = values[id]
    if (raw === undefined) continue
    form[key] = raw as GrokSettingsForm[GrokSettingsKey]
  }
  return form
}

export const grokSettingsConfig: SettingsConfig = {
  cacheKey: 'settings-grok',
  homePath: '/grok',
  module: 'grok',
  i18nPrefix: 'grok.settings',
  titleKey: 'grok.settings.title',
  subtitleKey: 'grok.settings.subtitle',
  features: { rawSource: true, localOnly: true, dirtyPatch: true, managedLocks: true },
  notify: surfaceNotify,
  probe: probeLocalEnvironment,
  tabs: [
    { id: 'model', labelKey: 'grok.settings.tabs.model' },
    { id: 'sessionUi', labelKey: 'grok.settings.tabs.sessionUi' },
    { id: 'cli', labelKey: 'grok.settings.tabs.cli' },
  ],
  fields: [
    textField('models_default', 'model', 'grok.settings.fields.defaultModel'),
    selectField({
      id: 'models_default_reasoning_effort',
      tab: 'model',
      labelKey: 'grok.settings.fields.reasoningEffort',
      options: optionsOf(GROK_REASONING_EFFORTS),
    }),
    selectField({
      id: 'ui_theme',
      tab: 'sessionUi',
      labelKey: 'grok.settings.fields.theme',
      options: optionsOf(GROK_THEMES),
    }),
    {
      id: 'session_auto_compact_threshold_percent',
      tab: 'sessionUi',
      kind: 'number',
      labelKey: 'grok.settings.fields.autoCompact',
      integerRange: { min: 0, max: 100 },
    },
    boolField('session_load_envrc', 'sessionUi', 'grok.settings.fields.loadEnvrc'),
    boolField('cli_auto_update', 'cli', 'grok.settings.fields.autoUpdate'),
    selectField({
      id: 'cli_channel',
      tab: 'cli',
      labelKey: 'grok.settings.fields.channel',
      options: optionsOf(GROK_CHANNELS),
    }),
    boolField('cli_show_tips', 'cli', 'grok.settings.fields.showTips'),
    selectField({
      id: 'hints_new_session_worktree_mode',
      tab: 'cli',
      labelKey: 'grok.settings.fields.newSessionWorktree',
      options: optionsOf(GROK_WORKTREE_MODES),
    }),
    selectField({
      id: 'hints_fork_worktree_mode',
      tab: 'cli',
      labelKey: 'grok.settings.fields.forkWorktree',
      options: optionsOf(GROK_WORKTREE_MODES),
    }),
  ],
  load: async () => {
    const response = await getGrokSettings()
    if (response.status !== 'ok') throw new Error(response.status)
    const form = grokSettingsResponseToForm(response)
    const values: SettingsValues = {}
    for (const [id, key] of Object.entries(GROK_ID_TO_KEY)) {
      values[id] = form[key]
    }
    return values
  },
  save: async ({ values, dirtyKeys }) => {
    const form = grokFormFromValues(values)
    const grokDirty = new Set<GrokSettingsKey>()
    for (const id of dirtyKeys) {
      const key = GROK_ID_TO_KEY[id]
      if (key) grokDirty.add(key)
    }
    const invalid = validateGrokSettingsForm(form, grokDirty)
    if (invalid) throw new Error(invalid)
    const patch = buildGrokSettingsPatch(form, grokDirty)
    const result = await updateGrokSettings(patch)
    if (result.status !== 'saved') throw new Error(result.status)
  },
}
