export type ProviderTemplatePlatform = 'claude' | 'codex' | 'opencode'

export type ProviderTemplateCategory =
  | 'official'
  | 'cn_official'
  | 'aggregator'
  | 'third_party'
  | 'local'

export type ProviderTemplateSource = 'built_in' | 'custom'

export interface ClaudeProviderTemplateOverride {
  baseUrl?: string
  provider?: string
  providerType?: 'official_relay' | 'third_party_model' | string
  model?: string
  smallFastModel?: string
  defaultOpusModel?: string
  defaultSonnetModel?: string
  defaultHaikuModel?: string
  defaultFableModel?: string
  subagentModel?: string
  claudeCodeAutoCompactWindow?: string
  apiTimeoutMs?: string
  claudeCodeDisableNonessentialTraffic?: string
  description?: string
}

export interface CodexProviderTemplateOverride {
  baseUrl?: string
  websiteUrl?: string
  apiKeyUrl?: string
  modelCatalog?: string[]
  model?: string
  provider?: string
  providerType?: string
  description?: string
  protocol?: string
}

export interface OpenCodeProviderTemplateOverride {
  id?: string
  name?: string
  npm?: string
  baseURL?: string
  models?: Record<string, unknown>
  extraOptions?: Record<string, unknown>
  rootExtra?: Record<string, unknown>
}

export interface ProviderTemplatePlatformOverrides {
  claude?: ClaudeProviderTemplateOverride
  codex?: CodexProviderTemplateOverride
  opencode?: OpenCodeProviderTemplateOverride
}

export interface ProviderTemplate {
  id: string
  name: string
  aliases?: string[]
  category: ProviderTemplateCategory
  websiteUrl?: string
  apiKeyUrl?: string
  tags?: string[]
  baseUrls?: string[]
  modelCatalog?: string[]
  isOfficial?: boolean
  isPartner?: boolean
  source?: ProviderTemplateSource
  platforms: ProviderTemplatePlatformOverrides
  createdAt?: string
  updatedAt?: string
}

export interface ProviderTemplateDraftContext {
  platform: ProviderTemplatePlatform
  defaultName?: string
  name?: string
  websiteUrl?: string
  apiKeyUrl?: string
  baseUrls?: string[]
  modelCatalog?: string[]
  aliases?: string[]
  tags?: string[]
  category?: ProviderTemplateCategory
  platformOverride: NonNullable<ProviderTemplatePlatformOverrides[ProviderTemplatePlatform]>
}

export interface ProviderTemplateSelection {
  template: ProviderTemplate
  endpoint?: string
}

export interface ProviderTemplateOption {
  id: string
  template: ProviderTemplate
  platform: ProviderTemplatePlatform
  endpoint?: string
  label: string
  subtitle: string
  sourceLabel: string
  categoryLabel: string
  searchText: string
}

export interface ClaudeProfileTemplatePatch {
  base_url?: string
  provider?: string
  provider_type?: string
  default_opus_model?: string
  default_sonnet_model?: string
  default_haiku_model?: string
  default_fable_model?: string
  subagent_model?: string
  claude_code_auto_compact_window?: string
  api_timeout_ms?: string
  claude_code_disable_nonessential_traffic?: string
  description?: string
  suggestedName?: string
}

export interface ClaudeLegacyConfigTemplatePatch {
  base_url?: string
  model?: string
  small_fast_model?: string
  provider?: string
  provider_type?: string
  description?: string
  suggestedName?: string
}

export interface CodexProviderTemplatePatch {
  name?: string
  baseUrl?: string
  websiteUrl?: string
  apiKeyUrl?: string
}

export interface CodexApiAccountTemplatePatch {
  providerName?: string
  apiBaseUrl?: string
}

export interface CodexProfileTemplatePatch {
  base_url?: string
  provider?: string
  provider_type?: string
  description?: string
  model?: string
  suggestedName?: string
}

export interface OpenCodeProviderTemplatePatch {
  id?: string
  name?: string
  npm?: string
  baseURL?: string
  modelsJson?: string
  extraOptionsJson?: string
  rootExtraJson?: string
}
