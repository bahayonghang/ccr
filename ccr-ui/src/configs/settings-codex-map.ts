import { asBool, asList, asString, splitList } from '@/configs/settings-helpers'
import type { SettingsValues } from '@/configs/settings-types'
import type { CodexConfig } from '@/types/codex'
import { dirtySettingsPatch, settingsRecord } from '@/configs/settings-patch'

const optionalNumber = (value: SettingsValues[string]): number | undefined =>
  value === '' ? undefined : Number(value)

const flattenModel = (form: CodexConfig): SettingsValues => ({
  model: asString(form.model),
  model_provider: asString(form.model_provider),
  model_reasoning_effort: asString(form.model_reasoning_effort),
  model_reasoning_summary: asString(form.model_reasoning_summary),
  model_verbosity: asString(form.model_verbosity),
  model_context_window: form.model_context_window ?? '',
  model_auto_compact_token_limit: form.model_auto_compact_token_limit ?? '',
  personality: asString(form.personality),
})

const flattenSecurity = (form: CodexConfig): SettingsValues => ({
  approval_policy: asString(form.approval_policy),
  sandbox_mode: asString(form.sandbox_mode),
  disable_response_storage: asBool(form.disable_response_storage),
  writableRoots: asList(form.sandbox_workspace_write?.writable_roots),
  sandboxNetworkAccess: asBool(form.sandbox_workspace_write?.network_access),
  shellIncludeOnly: asList(form.shell_environment_policy?.include_only),
})

const flattenTools = (form: CodexConfig): SettingsValues => ({
  web_search: asString(form.web_search),
  file_opener: asString(form.file_opener),
  toolsViewImage: asBool(form.tools?.view_image),
  toolsWebSearch: asBool(form.tools?.web_search),
  developer_instructions: asString(form.developer_instructions),
  instructions: asString(form.instructions),
})

const flattenUi = (form: CodexConfig): SettingsValues => ({
  tuiAlternateScreen: asString(form.tui?.alternate_screen),
  tuiAnimations: asBool(form.tui?.animations),
  tuiNotifications: form.tui?.notifications == null ? '' : JSON.stringify(form.tui.notifications),
  tuiShowTooltips: asBool(form.tui?.show_tooltips),
  hide_agent_reasoning: asBool(form.hide_agent_reasoning),
  show_raw_agent_reasoning: asBool(form.show_raw_agent_reasoning),
  check_for_update_on_startup: asBool(form.check_for_update_on_startup),
  suppress_unstable_features_warning: asBool(form.suppress_unstable_features_warning),
})

const flattenFeatures = (form: CodexConfig): SettingsValues => ({
  experimental_use_rmcp_client: asBool(form.experimental_use_rmcp_client),
  historyPersistence: asString(form.history?.persistence),
  historyMaxBytes: form.history?.max_bytes ?? '',
  analyticsEnabled: asBool(form.analytics?.enabled),
  feedbackEnabled: asBool(form.feedback?.enabled),
})

export function flattenCodexSettings(form: CodexConfig): SettingsValues {
  return {
    ...flattenModel(form),
    ...flattenSecurity(form),
    ...flattenTools(form),
    ...flattenUi(form),
    ...flattenFeatures(form),
  }
}

const buildModel = (values: SettingsValues): CodexConfig => ({
  model: asString(values.model) || undefined,
  model_provider: asString(values.model_provider) || undefined,
  model_reasoning_effort: asString(values.model_reasoning_effort) || undefined,
  model_reasoning_summary: asString(values.model_reasoning_summary) || undefined,
  model_verbosity: asString(values.model_verbosity) || undefined,
  model_context_window: optionalNumber(values.model_context_window),
  model_auto_compact_token_limit: optionalNumber(values.model_auto_compact_token_limit),
  personality: asString(values.personality) || undefined,
})

const buildSecurity = (values: SettingsValues): CodexConfig => ({
  approval_policy: asString(values.approval_policy) || undefined,
  sandbox_mode: asString(values.sandbox_mode) || undefined,
  disable_response_storage: asBool(values.disable_response_storage),
  sandbox_workspace_write: {
    writable_roots: splitList(values.writableRoots),
    network_access: asBool(values.sandboxNetworkAccess),
  },
  shell_environment_policy: { include_only: splitList(values.shellIncludeOnly) },
})

const buildTools = (values: SettingsValues): CodexConfig => ({
  web_search: asString(values.web_search) || undefined,
  file_opener: asString(values.file_opener) || undefined,
  tools: { view_image: asBool(values.toolsViewImage), web_search: asBool(values.toolsWebSearch) },
  developer_instructions: asString(values.developer_instructions) || undefined,
  instructions: asString(values.instructions) || undefined,
})

const buildUi = (values: SettingsValues): CodexConfig => ({
  tui: {
    alternate_screen: asString(values.tuiAlternateScreen) || undefined,
    animations: asBool(values.tuiAnimations),
    notifications: values.tuiNotifications === '' || values.tuiNotifications == null
      ? undefined
      : JSON.parse(String(values.tuiNotifications)) as boolean | string[],
    show_tooltips: asBool(values.tuiShowTooltips),
  },
  hide_agent_reasoning: asBool(values.hide_agent_reasoning),
  show_raw_agent_reasoning: asBool(values.show_raw_agent_reasoning),
  check_for_update_on_startup: asBool(values.check_for_update_on_startup),
  suppress_unstable_features_warning: asBool(values.suppress_unstable_features_warning),
})

const buildFeatures = (values: SettingsValues): CodexConfig => ({
  experimental_use_rmcp_client: asBool(values.experimental_use_rmcp_client),
  history: {
    persistence: asString(values.historyPersistence) || undefined,
    max_bytes: optionalNumber(values.historyMaxBytes),
  },
  analytics: { enabled: asBool(values.analyticsEnabled) },
  feedback: { enabled: asBool(values.feedbackEnabled) },
})

export function buildCodexSettingsPayload(
  values: SettingsValues,
  input: { dirtyKeys: readonly string[]; source: unknown },
): Record<string, unknown> {
  const payload: CodexConfig = {
    ...buildModel(values),
    ...buildSecurity(values),
    ...buildTools(values),
    ...buildUi(values),
    ...buildFeatures(values),
  }
  const paths: Record<string, string> = Object.fromEntries(Object.keys(payload).map((key) => [key, key]))
  Object.assign(paths, {
    writableRoots: 'sandbox_workspace_write.writable_roots',
    sandboxNetworkAccess: 'sandbox_workspace_write.network_access',
    shellIncludeOnly: 'shell_environment_policy.include_only',
    toolsViewImage: 'tools.view_image',
    toolsWebSearch: 'tools.web_search',
    tuiAlternateScreen: 'tui.alternate_screen',
    tuiAnimations: 'tui.animations',
    tuiNotifications: 'tui.notifications',
    tuiShowTooltips: 'tui.show_tooltips',
    historyPersistence: 'history.persistence',
    historyMaxBytes: 'history.max_bytes',
    analyticsEnabled: 'analytics.enabled',
    feedbackEnabled: 'feedback.enabled',
  })
  // Codex merges each submitted leaf and retains disk-only extension fields.
  return codexUnsetPatch(dirtySettingsPatch(paths, payload, { dirtyKeys: input.dirtyKeys, source: {} }))
}

function codexUnsetPatch(patch: Record<string, unknown>): Record<string, unknown> {
  return Object.fromEntries(Object.entries(patch).map(([key, value]) => [
    key,
    value === undefined ? null
      : value !== null && typeof value === 'object' && !Array.isArray(value)
        ? codexUnsetPatch(settingsRecord(value))
        : value,
  ]))
}
