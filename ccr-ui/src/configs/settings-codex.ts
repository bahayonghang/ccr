import { getCodexConfig, updateCodexConfig } from '@/api'
import { buildCodexSettingsPayload, flattenCodexSettings } from '@/configs/settings-codex-map'
import { boolField, selectField, textField } from '@/configs/settings-helpers'
import { surfaceNotify } from '@/configs/surfaceNotify'
import type { SettingsConfig, SettingsFieldOption } from '@/configs/settings-types'

const CODEX_EFFORT: SettingsFieldOption[] = [
  { value: 'minimal', labelKey: 'minimal' },
  { value: 'low', labelKey: 'low' },
  { value: 'medium', labelKey: 'medium' },
  { value: 'high', labelKey: 'high' },
]

export const codexSettingsConfig: SettingsConfig = {
  cacheKey: 'settings-codex',
  homePath: '/codex',
  module: 'codex',
  i18nPrefix: 'codex.settings',
  titleKey: 'codex.settings.title',
  subtitleKey: 'codex.settings.subtitle',
  features: { rawSource: true },
  notify: surfaceNotify,
  tabs: [
    { id: 'model', labelKey: 'codex.settings.tabs.model' },
    { id: 'security', labelKey: 'codex.settings.tabs.security' },
    { id: 'tools', labelKey: 'codex.settings.tabs.tools' },
    { id: 'ui', labelKey: 'codex.settings.tabs.ui' },
    { id: 'features', labelKey: 'codex.settings.tabs.features' },
  ],
  fields: [
    textField('model', 'model', 'codex.settings.model.model'),
    textField('model_provider', 'model', 'codex.settings.model.modelProvider'),
    selectField({ id: 'model_reasoning_effort', tab: 'model', labelKey: 'codex.settings.model.reasoningEffort', options: CODEX_EFFORT }),
    textField('model_reasoning_summary', 'model', 'codex.settings.model.reasoningSummary'),
    textField('model_verbosity', 'model', 'codex.settings.model.verbosity'),
    { id: 'model_context_window', tab: 'model', kind: 'number', labelKey: 'codex.settings.model.contextWindow' },
    { id: 'model_auto_compact_token_limit', tab: 'model', kind: 'number', labelKey: 'codex.settings.model.autoCompactLimit' },
    textField('personality', 'model', 'codex.settings.model.personality'),
    textField('approval_policy', 'security', 'codex.settings.security.approvalPolicy'),
    textField('sandbox_mode', 'security', 'codex.settings.security.sandboxMode'),
    boolField('disable_response_storage', 'security', 'codex.settings.security.disableResponseStorage'),
    { id: 'writableRoots', tab: 'security', kind: 'textarea', labelKey: 'codex.settings.security.writableRoots', listValue: true },
    boolField('sandboxNetworkAccess', 'security', 'codex.settings.security.networkAccess'),
    { id: 'shellIncludeOnly', tab: 'security', kind: 'textarea', labelKey: 'codex.settings.security.shellIncludeOnly', listValue: true },
    textField('web_search', 'tools', 'codex.settings.tools.webSearch'),
    textField('file_opener', 'tools', 'codex.settings.tools.fileOpener'),
    boolField('toolsViewImage', 'tools', 'codex.settings.tools.viewImage'),
    boolField('toolsWebSearch', 'tools', 'codex.settings.tools.toolWebSearch'),
    { id: 'developer_instructions', tab: 'tools', kind: 'textarea', labelKey: 'codex.settings.tools.developerInstructions' },
    { id: 'instructions', tab: 'tools', kind: 'textarea', labelKey: 'codex.settings.tools.instructions' },
    textField('tuiAlternateScreen', 'ui', 'codex.settings.ui.alternateScreen'),
    boolField('tuiAnimations', 'ui', 'codex.settings.ui.animations'),
    boolField('tuiNotifications', 'ui', 'codex.settings.ui.notifications'),
    boolField('tuiShowTooltips', 'ui', 'codex.settings.ui.showTooltips'),
    boolField('hide_agent_reasoning', 'ui', 'codex.settings.ui.hideAgentReasoning'),
    boolField('show_raw_agent_reasoning', 'ui', 'codex.settings.ui.showRawAgentReasoning'),
    boolField('check_for_update_on_startup', 'ui', 'codex.settings.ui.checkForUpdate'),
    boolField('suppress_unstable_features_warning', 'ui', 'codex.settings.ui.suppressUnstableWarning'),
    boolField('experimental_use_rmcp_client', 'features', 'codex.settings.features.experimentalRmcp'),
    textField('historyPersistence', 'features', 'codex.settings.features.historyPersistence'),
    { id: 'historyMaxBytes', tab: 'features', kind: 'number', labelKey: 'codex.settings.features.historyMaxBytes' },
    boolField('analyticsEnabled', 'features', 'codex.settings.features.analytics'),
    boolField('feedbackEnabled', 'features', 'codex.settings.features.feedback'),
  ],
  load: async () => flattenCodexSettings(await getCodexConfig()),
  save: async ({ values }) => {
    await updateCodexConfig(buildCodexSettingsPayload(values))
  },
}
