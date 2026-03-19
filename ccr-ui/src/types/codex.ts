// Codex CLI configuration type definitions

// ============ Codex MCP Server Types ============

export interface CodexMcpServer {
  name: string
  // STDIO server fields
  command?: string
  args?: string[]
  env?: Record<string, string>
  cwd?: string
  startup_timeout_ms?: number
  // HTTP server fields
  url?: string
  bearer_token?: string
}

export interface CodexMcpServerRequest {
  name?: string
  command?: string
  args?: string[]
  env?: Record<string, string>
  cwd?: string
  startup_timeout_ms?: number
  url?: string
  bearer_token?: string
}

export interface CodexMcpServersResponse {
  servers: CodexMcpServer[]
}

// ============ Codex Profile Types (CCR 平台：~/.ccr/platforms/codex/profiles.toml) ============

export type CodexProfileAuthMode =
  | 'openai_chatgpt'
  | 'openai_api_key'
  | 'provider_env_key'
  | 'no_auth'

export type OpenAiLoginMethod = 'chatgpt' | 'api'

export interface CodexProfile {
  name: string
  description?: string
  base_url?: string | null
  auth_token?: string | null
  model?: string | null
  small_fast_model?: string
  provider?: string
  provider_type?: string
  account?: string
  tags?: string[]
  usage_count?: number
  enabled?: boolean
  wire_api?: string
  env_key?: string | null
  requires_openai_auth?: boolean | null
  approval_policy?: string
  sandbox_mode?: string
  model_reasoning_effort?: string
  network_access?: string | boolean | null
  disable_response_storage?: boolean | null
  auth_mode?: CodexProfileAuthMode
  openai_login_method?: OpenAiLoginMethod | null
  credential_store?: string | null
  auth_source?: string | null
  env_export?: Record<string, string> | null
  shell_export_script?: string | null
  is_current?: boolean
  extra?: Record<string, unknown>
}

export interface CodexProfileRequest {
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
  enabled?: boolean
  wire_api?: string
  env_key?: string
  requires_openai_auth?: boolean
  approval_policy?: string
  sandbox_mode?: string
  model_reasoning_effort?: string
  network_access?: string | boolean
  disable_response_storage?: boolean
  auth_mode?: CodexProfileAuthMode
  openai_login_method?: OpenAiLoginMethod
  extra?: Record<string, unknown>
}

export interface CodexProfilesResponse {
  profiles: CodexProfile[]
  current_profile?: string | null
}

export interface CodexProfileResponse {
  profile: CodexProfile
}

export interface CodexModelsResponse {
  builtin_models: string[]
  custom_models: string[]
  models: string[]
}

export interface CodexAddCustomModelRequest {
  model: string
}

export interface CodexAddCustomModelResponse {
  model: string
  models: string[]
  message?: string
}

// ============ Codex Base Config Types ============
// 说明：这是 Codex CLI 的 ~/.codex/config.toml，不同于 CCR 的 profiles.toml

export interface CodexCliProfile {
  model?: string
  approval_policy?: string
  sandbox_mode?: string
  model_reasoning_effort?: string
  [key: string]: unknown
}

export interface CodexConfig {
  model?: string
  model_provider?: string
  model_reasoning_effort?: string
  // 模型与推理（扩展）
  model_reasoning_summary?: string
  model_verbosity?: string
  model_context_window?: number
  model_auto_compact_token_limit?: number
  personality?: string
  // 安全与权限
  approval_policy?: string
  sandbox_mode?: string
  disable_response_storage?: boolean
  sandbox_workspace_write?: { writable_roots?: string[]; network_access?: boolean }
  shell_environment_policy?: {
    include_only?: string[]
  }
  // 工具与搜索
  web_search?: string
  file_opener?: string
  developer_instructions?: string
  instructions?: string
  tools?: { view_image?: boolean; web_search?: boolean }
  // TUI 与界面
  tui?: {
    alternate_screen?: string
    animations?: boolean
    notifications?: boolean
    show_tooltips?: boolean
  }
  hide_agent_reasoning?: boolean
  show_raw_agent_reasoning?: boolean
  check_for_update_on_startup?: boolean
  suppress_unstable_features_warning?: boolean
  // MCP / Profiles（独立管理）
  mcp_servers?: Record<string, Omit<CodexMcpServer, 'name'>>
  profiles?: Record<string, CodexCliProfile>
  // 功能开关
  experimental_use_rmcp_client?: boolean
  history?: { persistence?: string; max_bytes?: number }
  analytics?: { enabled?: boolean }
  feedback?: { enabled?: boolean }
  features?: Record<string, boolean>
}

export interface CodexConfigResponse {
  config: CodexConfig
}

// ============ Codex Auth Management Types ============

/** Token 新鲜度 */
export type TokenFreshness = 'Fresh' | 'Stale' | 'Old' | 'Unknown' | (string & {})

/** 登录状态 (tagged union) */
export type LoginState =
  | { type: 'NotLoggedIn' }
  | { type: 'LoggedInUnsaved' }
  | { type: 'LoggedInSaved'; account_name: string }
  | { type: 'ApiKeyActive' }
  | { type: 'ProviderKeyActive'; env_key: string }
  | { type: 'Unknown'; raw_type: string; raw: Record<string, unknown> }

/** Codex Auth 账号列表项 */
export interface CodexAuthAccountItem {
  name: string
  description?: string
  email?: string
  is_current: boolean
  is_virtual: boolean
  last_used?: string
  last_refresh?: string
  freshness: TokenFreshness
  freshness_icon: string
  freshness_description: string
  /** 到期时间 (ISO 8601) */
  expires_at?: string
  /** 是否已过期 */
  is_expired: boolean
}

/** Codex Auth 当前信息 */
export interface CodexAuthCurrentInfo {
  account_id: string
  email?: string
  last_refresh?: string
  freshness: TokenFreshness
  freshness_icon: string
  freshness_description: string
  /** 到期时间 (ISO 8601) */
  expires_at?: string
  /** 是否已过期 */
  is_expired: boolean
}

/** Codex Auth 账号列表响应 */
export interface CodexAuthListResponse {
  accounts: CodexAuthAccountItem[]
  login_state: LoginState
}

/** Codex Auth 当前状态响应 */
export interface CodexAuthCurrentResponse {
  logged_in: boolean
  info?: CodexAuthCurrentInfo
  login_state: LoginState
}

/** Codex Auth 保存请求 */
export interface CodexAuthSaveRequest {
  name: string
  description?: string
  /** 到期时间 (ISO 8601) */
  expires_at?: string
  force?: boolean
}

/** Codex Auth 进程检测响应 */
export interface CodexAuthProcessResponse {
  has_running_process: boolean
  pids: number[]
  warning?: string
}

// ============ Codex Usage Types ============

/** Codex 使用量统计 */
export interface CodexUsageStats {
  total_input_tokens: number
  total_output_tokens: number
  total_requests: number
  window_start?: string
  window_end?: string
}

/** Codex 滚动窗口使用量响应 */
export interface CodexUsageResponse {
  five_hour: CodexUsageStats
  seven_day: CodexUsageStats
  all_time: CodexUsageStats
  by_model: Record<string, CodexUsageStats>
}

// ============ Codex Quota Types ============

/** Codex 配额信息 */
export interface CodexQuota {
  hourly_percentage: number
  hourly_reset_time?: number
  hourly_window_minutes?: number
  hourly_window_present?: boolean
  weekly_percentage: number
  weekly_reset_time?: number
  weekly_window_minutes?: number
  weekly_window_present?: boolean
  plan_type?: string
}

/** 单个账号的配额查询结果 */
export interface CodexAccountQuota {
  account_name: string
  email?: string
  quota?: CodexQuota
  error?: string
  fetched_at: string
}
