// Claude Code feature type definitions: hooks, output styles, statusline

export type HookType = string
export type HookHandlerType = 'command' | 'http' | 'prompt' | 'agent' | string

export interface Hook {
  type: HookHandlerType
  command?: string
  url?: string
  prompt?: string
  model?: string
  timeout?: number
  statusMessage?: string
  allowedEnvVars?: string[]
  headers?: Record<string, string>
  async?: boolean
  [key: string]: unknown
}

export interface HookMatcherGroup {
  matcher?: string
  hooks: Hook[]
  [key: string]: unknown
}

export type HookMap = Record<string, HookMatcherGroup[]>

export interface HooksResponse {
  hooks: HookMap
}

export interface OutputStyle {
  name: string
  content: string
}

export interface OutputStyleRequest {
  name: string
  content: string
}

export interface UpdateOutputStyleRequest {
  content: string
}

export interface StatuslineConfig {
  command?: string
  enabled: boolean
}

export interface ClaudeProfile {
  name: string
  description?: string | null
  base_url?: string | null
  auth_token?: string | null
  model?: string | null
  small_fast_model?: string | null
  default_opus_model?: string | null
  default_sonnet_model?: string | null
  default_haiku_model?: string | null
  default_fable_model?: string | null
  default_opus_model_name?: string | null
  default_sonnet_model_name?: string | null
  default_haiku_model_name?: string | null
  default_fable_model_name?: string | null
  subagent_model?: string | null
  custom_model_option?: string | null
  custom_model_option_name?: string | null
  effort_level?: string | null
  provider?: string | null
  provider_type?: string | null
  account?: string | null
  tags?: string[] | null
  usage_count?: number | null
  enabled?: boolean | null
  auth_mode?: ClaudeProfileAuthMode | null
  auth_source?: string | null
  platform_data?: Record<string, unknown>
  is_current: boolean
  extra?: Record<string, unknown> | null
}

export interface ClaudeProfileRequest {
  name: string
  description?: string
  base_url?: string
  auth_token?: string
  model?: string | null
  small_fast_model?: string | null
  default_opus_model?: string | null
  default_sonnet_model?: string | null
  default_haiku_model?: string | null
  default_fable_model?: string | null
  default_opus_model_name?: string | null
  default_sonnet_model_name?: string | null
  default_haiku_model_name?: string | null
  default_fable_model_name?: string | null
  subagent_model?: string | null
  custom_model_option?: string | null
  custom_model_option_name?: string | null
  effort_level?: string | null
  provider?: string
  provider_type?: string
  account?: string
  tags?: string[]
  usage_count?: number
  enabled?: boolean
  auth_mode?: ClaudeProfileAuthMode | null
  platform_data?: Record<string, unknown>
  extra?: Record<string, unknown>
}

export interface ClaudeProfilesResponse {
  profiles: ClaudeProfile[]
  current_profile: string | null
}

export type ClaudeProfileAuthMode = 'subscription' | 'api_key'

export type ClaudeLoginState =
  | { type: 'NotLoggedIn' }
  | { type: 'LoggedInUnsaved' }
  | { type: 'LoggedInSaved'; account_name: string }
  | { type: 'ApiKeyActive' }

export type ClaudeRuntimeMode =
  | 'profile_only'
  | 'profile_with_auth'
  | 'profile_pending_auth'
  | 'runtime_only'
  | 'unresolved'

export interface ClaudeRuntimeSummary {
  mode: ClaudeRuntimeMode
  current_profile_name?: string | null
  current_profile_provider?: string | null
  current_profile_auth_mode?: ClaudeProfileAuthMode | null
  current_profile_auth_source?: string | null
  current_login_name?: string | null
  official_login_state: ClaudeLoginState
  current_auth_name?: string | null
  login_state: ClaudeLoginState
}

export interface ClaudeAuthAccountItem {
  name: string
  description?: string | null
  email?: string | null
  billing_type?: string | null
  subscription_type?: string | null
  rate_limit_tier?: string | null
  is_current: boolean
  is_logged_in?: boolean
  saved_at: string
  last_used?: string | null
  expires_at?: string | null
}

export interface ClaudeAuthCurrentInfo {
  account_uuid?: string | null
  email?: string | null
  billing_type?: string | null
  subscription_type?: string | null
  rate_limit_tier?: string | null
  expires_at?: string | null
}

export interface ClaudeAuthListResponse {
  accounts: ClaudeAuthAccountItem[]
  login_state: ClaudeLoginState
  runtime_summary: ClaudeRuntimeSummary
  current_profile_auth_mode?: ClaudeProfileAuthMode | null
}

export interface ClaudeAuthCurrentResponse {
  logged_in: boolean
  info?: ClaudeAuthCurrentInfo | null
  runtime_summary: ClaudeRuntimeSummary
  login_state: ClaudeLoginState
}

export interface ClaudeAuthSaveRequest {
  name: string
  description?: string | null
  force?: boolean
}

export interface BuiltinPrompt {
  id: string
  name: string
  description: string
  category: string
  tags: string[]
  content: string
}

export interface ClaudeSettingsData {
  model?: string
  availableModels?: string[]
  alwaysThinkingEnabled?: boolean
  maxThinkingTokens?: number
  maxOutputTokens?: number
  effortLevel?: string
  skipDangerousModePermissionPrompt?: boolean
  theme?: string
  language?: string
  showTurnDuration?: boolean
  prefersReducedMotion?: boolean
  spinnerTipsEnabled?: boolean
  terminalProgressBarEnabled?: boolean
  showSpinnerTree?: boolean
  includeCoAuthoredBy?: boolean
  autoUpdates?: boolean
  autoUpdatesChannel?: string
  cleanupPeriodDays?: number
  respectGitignore?: boolean
  env?: Record<string, string>
  permissions?: {
    allow?: string[]
    deny?: string[]
    defaultMode?: string
    additionalDirectories?: string[]
  }
  sandbox?: {
    enabled?: boolean
    autoAllowBashIfSandboxed?: boolean
    network?: {
      allowLocalBinding?: boolean
      allowedDomains?: string[]
    }
    excludedCommands?: string[]
  }
  attribution?: {
    commit?: string
    pr?: string
  }
  [key: string]: unknown
}
