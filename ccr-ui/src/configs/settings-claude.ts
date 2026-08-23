import { getClaudeSettings, updateClaudeSettings } from '@/api'
import { asBool, asList, asString, boolField, selectField, splitList, textField } from '@/configs/settings-helpers'
import { surfaceNotify } from '@/configs/surfaceNotify'
import type { SettingsConfig, SettingsFieldOption } from '@/configs/settings-types'
import type { ClaudeSettingsData } from '@/types/claude'

const MODEL_OPTIONS: SettingsFieldOption[] = [
  { value: '', labelKey: 'claudeSettings.model.noOverride' },
  { value: 'opus', labelKey: 'opus' },
  { value: 'sonnet', labelKey: 'sonnet' },
  { value: 'haiku', labelKey: 'haiku' },
]

const PERMISSION_MODE_OPTIONS: SettingsFieldOption[] = [
  { value: 'default', labelKey: 'default' },
  { value: 'plan', labelKey: 'plan' },
  { value: 'bypassPermissions', labelKey: 'bypassPermissions' },
]

const CHANNEL_OPTIONS: SettingsFieldOption[] = [
  { value: 'stable', labelKey: 'stable' },
  { value: 'latest', labelKey: 'latest' },
]

const ATTRIBUTION_OPTIONS: SettingsFieldOption[] = [
  { value: 'none', labelKey: 'none' },
  { value: 'co-authored-by', labelKey: 'co-authored-by' },
  { value: 'authored-by', labelKey: 'authored-by' },
]

export const claudeSettingsConfig: SettingsConfig = {
  cacheKey: 'settings-claude',
  homePath: '/claude-code',
  module: 'claude-code',
  i18nPrefix: 'claudeSettings',
  titleKey: 'claudeSettings.title',
  subtitleKey: 'claudeSettings.subtitle',
  features: { rawSource: true },
  notify: surfaceNotify,
  tabs: [
    { id: 'model', labelKey: 'claudeSettings.tabs.model' },
    { id: 'permissions', labelKey: 'claudeSettings.tabs.permissions' },
    { id: 'env', labelKey: 'claudeSettings.tabs.env' },
    { id: 'ui', labelKey: 'claudeSettings.tabs.ui' },
    { id: 'sandbox', labelKey: 'claudeSettings.tabs.sandbox' },
    { id: 'git', labelKey: 'claudeSettings.tabs.git' },
  ],
  fields: [
    selectField({ id: 'model', tab: 'model', labelKey: 'claudeSettings.model.defaultModel', options: MODEL_OPTIONS }),
    textField('effortLevel', 'model', 'claudeSettings.model.effortLevel'),
    boolField('alwaysThinkingEnabled', 'model', 'claudeSettings.model.alwaysThinking'),
    textField('maxThinkingTokens', 'model', 'claudeSettings.model.maxThinkingTokens'),
    textField('maxOutputTokens', 'model', 'claudeSettings.model.maxOutputTokens'),
    { id: 'availableModels', tab: 'model', kind: 'textarea', labelKey: 'claudeSettings.model.availableModels', listValue: true },
    selectField({ id: 'permDefaultMode', tab: 'permissions', labelKey: 'claudeSettings.permissions.defaultMode', options: PERMISSION_MODE_OPTIONS }),
    boolField('skipDangerousModePermissionPrompt', 'permissions', 'claudeSettings.permissions.skipDangerous'),
    { id: 'permAllow', tab: 'permissions', kind: 'textarea', labelKey: 'claudeSettings.permissions.allow', listValue: true },
    { id: 'permDeny', tab: 'permissions', kind: 'textarea', labelKey: 'claudeSettings.permissions.deny', listValue: true },
    { id: 'permAdditionalDirs', tab: 'permissions', kind: 'textarea', labelKey: 'claudeSettings.permissions.additionalDirs', listValue: true },
    { id: 'envText', tab: 'env', kind: 'textarea', labelKey: 'claudeSettings.tabs.env' },
    textField('theme', 'ui', 'claudeSettings.ui.theme'),
    textField('language', 'ui', 'claudeSettings.ui.language'),
    boolField('showTurnDuration', 'ui', 'claudeSettings.ui.showTurnDuration'),
    boolField('spinnerTipsEnabled', 'ui', 'claudeSettings.ui.spinnerTips'),
    boolField('terminalProgressBarEnabled', 'ui', 'claudeSettings.ui.progressBar'),
    boolField('showSpinnerTree', 'ui', 'claudeSettings.ui.spinnerTree'),
    boolField('prefersReducedMotion', 'ui', 'claudeSettings.ui.reducedMotion'),
    selectField({ id: 'autoUpdatesChannel', tab: 'ui', labelKey: 'claudeSettings.ui.updateChannel', options: CHANNEL_OPTIONS }),
    { id: 'cleanupPeriodDays', tab: 'ui', kind: 'number', labelKey: 'claudeSettings.ui.cleanupDays' },
    boolField('autoUpdates', 'ui', 'claudeSettings.ui.autoUpdates'),
    boolField('respectGitignore', 'ui', 'claudeSettings.ui.respectGitignore'),
    boolField('sandboxEnabled', 'sandbox', 'claudeSettings.sandbox.enabled'),
    boolField('sandboxAutoAllow', 'sandbox', 'claudeSettings.sandbox.autoAllowBash'),
    boolField('sandboxAllowLocal', 'sandbox', 'claudeSettings.sandbox.allowLocalBinding'),
    { id: 'sandboxAllowedDomains', tab: 'sandbox', kind: 'textarea', labelKey: 'claudeSettings.sandbox.allowedDomains', listValue: true },
    { id: 'sandboxExcludedCmds', tab: 'sandbox', kind: 'textarea', labelKey: 'claudeSettings.sandbox.excludedCommands', listValue: true },
    selectField({ id: 'attrCommit', tab: 'git', labelKey: 'claudeSettings.git.commitAttribution', options: ATTRIBUTION_OPTIONS }),
    selectField({ id: 'attrPr', tab: 'git', labelKey: 'claudeSettings.git.prAttribution', options: ATTRIBUTION_OPTIONS }),
    boolField('includeCoAuthoredBy', 'git', 'claudeSettings.git.includeCoAuthored'),
  ],
  load: async () => {
    const data = await getClaudeSettings()
    const envLines = Object.entries(data.env ?? {}).map(([key, value]) => `${key}=${value}`)
    return {
      model: asString(data.model),
      effortLevel: asString(data.effortLevel),
      alwaysThinkingEnabled: asBool(data.alwaysThinkingEnabled),
      maxThinkingTokens: asString(data.maxThinkingTokens),
      maxOutputTokens: asString(data.maxOutputTokens),
      availableModels: asList(data.availableModels),
      permDefaultMode: asString(data.permissions?.defaultMode),
      skipDangerousModePermissionPrompt: asBool(data.skipDangerousModePermissionPrompt),
      permAllow: asList(data.permissions?.allow),
      permDeny: asList(data.permissions?.deny),
      permAdditionalDirs: asList(data.permissions?.additionalDirectories),
      envText: envLines.join('\n'),
      theme: asString(data.theme),
      language: asString(data.language),
      showTurnDuration: asBool(data.showTurnDuration),
      spinnerTipsEnabled: asBool(data.spinnerTipsEnabled),
      terminalProgressBarEnabled: asBool(data.terminalProgressBarEnabled),
      showSpinnerTree: asBool(data.showSpinnerTree),
      prefersReducedMotion: asBool(data.prefersReducedMotion),
      autoUpdatesChannel: asString(data.autoUpdatesChannel),
      cleanupPeriodDays: data.cleanupPeriodDays ?? '',
      autoUpdates: asBool(data.autoUpdates),
      respectGitignore: asBool(data.respectGitignore),
      sandboxEnabled: asBool(data.sandbox?.enabled),
      sandboxAutoAllow: asBool(data.sandbox?.autoAllowBashIfSandboxed),
      sandboxAllowLocal: asBool(data.sandbox?.network?.allowLocalBinding),
      sandboxAllowedDomains: asList(data.sandbox?.network?.allowedDomains),
      sandboxExcludedCmds: asList(data.sandbox?.excludedCommands),
      attrCommit: asString(data.attribution?.commit),
      attrPr: asString(data.attribution?.pr),
      includeCoAuthoredBy: asBool(data.includeCoAuthoredBy),
    }
  },
  save: async ({ values }) => {
    const env: Record<string, string> = {}
    for (const line of splitList(values.envText)) {
      const sep = line.indexOf('=')
      if (sep <= 0) continue
      env[line.slice(0, sep)] = line.slice(sep + 1)
    }
    const payload: ClaudeSettingsData = {
      model: asString(values.model) || undefined,
      effortLevel: asString(values.effortLevel) || undefined,
      alwaysThinkingEnabled: asBool(values.alwaysThinkingEnabled),
      maxThinkingTokens: asString(values.maxThinkingTokens) ? Number(values.maxThinkingTokens) : undefined,
      maxOutputTokens: asString(values.maxOutputTokens) ? Number(values.maxOutputTokens) : undefined,
      availableModels: splitList(values.availableModels),
      skipDangerousModePermissionPrompt: asBool(values.skipDangerousModePermissionPrompt),
      theme: asString(values.theme) || undefined,
      language: asString(values.language) || undefined,
      showTurnDuration: asBool(values.showTurnDuration),
      spinnerTipsEnabled: asBool(values.spinnerTipsEnabled),
      terminalProgressBarEnabled: asBool(values.terminalProgressBarEnabled),
      showSpinnerTree: asBool(values.showSpinnerTree),
      prefersReducedMotion: asBool(values.prefersReducedMotion),
      autoUpdatesChannel: asString(values.autoUpdatesChannel) || undefined,
      cleanupPeriodDays: values.cleanupPeriodDays === '' ? undefined : Number(values.cleanupPeriodDays),
      autoUpdates: asBool(values.autoUpdates),
      respectGitignore: asBool(values.respectGitignore),
      includeCoAuthoredBy: asBool(values.includeCoAuthoredBy),
      env,
      permissions: {
        defaultMode: asString(values.permDefaultMode) || undefined,
        allow: splitList(values.permAllow),
        deny: splitList(values.permDeny),
        additionalDirectories: splitList(values.permAdditionalDirs),
      },
      sandbox: {
        enabled: asBool(values.sandboxEnabled),
        autoAllowBashIfSandboxed: asBool(values.sandboxAutoAllow),
        network: {
          allowLocalBinding: asBool(values.sandboxAllowLocal),
          allowedDomains: splitList(values.sandboxAllowedDomains),
        },
        excludedCommands: splitList(values.sandboxExcludedCmds),
      },
      attribution: {
        commit: asString(values.attrCommit) || undefined,
        pr: asString(values.attrPr) || undefined,
      },
    }
    await updateClaudeSettings(payload)
  },
}
