// Claude Code feature type definitions: hooks, output styles, statusline

import type { ClaudeProfileAuthMode as GeneratedClaudeProfileAuthMode } from './generated/claude_auth/ClaudeProfileAuthMode'

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
  claude_code_auto_compact_window?: string | null
  api_timeout_ms?: string | null
  claude_code_disable_nonessential_traffic?: string | null
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
  claude_code_auto_compact_window?: string | null
  api_timeout_ms?: string | null
  claude_code_disable_nonessential_traffic?: string | null
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

export type ClaudeProfileAuthMode = GeneratedClaudeProfileAuthMode
export type { ClaudeLoginState } from './generated/claude_auth/ClaudeLoginState'
export type { ClaudeRuntimeMode } from './generated/claude_auth/ClaudeRuntimeMode'
export type { ClaudeRuntimeSummary } from './generated/claude_auth/ClaudeRuntimeSummary'
export type { ClaudeAuthConfidence } from './generated/claude_auth/ClaudeAuthConfidence'
export type { ClaudeAuthDiagnosis } from './generated/claude_auth/ClaudeAuthDiagnosis'
export type { ClaudeAuthEvidence } from './generated/claude_auth/ClaudeAuthEvidence'
export type { ClaudeAuthOwnership } from './generated/claude_auth/ClaudeAuthOwnership'
export type { ClaudeAuthSourceKind } from './generated/claude_auth/ClaudeAuthSourceKind'
export type { ClaudeAuthSourceLocation } from './generated/claude_auth/ClaudeAuthSourceLocation'
export type { ClaudeAuthSourceObservation } from './generated/claude_auth/ClaudeAuthSourceObservation'
export type { ClaudeAuthAccountItem } from './generated/claude_auth/ClaudeAuthAccountItem'
export type { ClaudeAuthCurrentInfo } from './generated/claude_auth/ClaudeAuthCurrentInfo'
export type { ClaudeAuthListResponse } from './generated/claude_auth/ClaudeAuthListResponse'
export type { ClaudeAuthCurrentResponse } from './generated/claude_auth/ClaudeAuthCurrentResponse'
export type { ClaudeAuthSaveRequest } from '@/api/generated/claudeAuth'

export type BuiltinPrompt = import('./generated/builtin_prompts/BuiltinPromptDto').BuiltinPromptDto

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
