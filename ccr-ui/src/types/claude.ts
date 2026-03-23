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
  provider?: string | null
  provider_type?: string | null
  account?: string | null
  tags?: string[] | null
  usage_count?: number | null
  enabled?: boolean | null
  platform_data?: Record<string, unknown>
  is_current: boolean
}

export interface ClaudeProfileRequest {
  name: string
  description?: string
  base_url?: string
  auth_token?: string
  model?: string
  small_fast_model?: string
  provider?: string
  provider_type?: string
  account?: string
  tags?: string[]
  usage_count?: number
  enabled?: boolean
  platform_data?: Record<string, unknown>
  extra?: Record<string, unknown>
}

export interface ClaudeProfilesResponse {
  profiles: ClaudeProfile[]
  current_profile: string | null
}
