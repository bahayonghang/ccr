// Codex CLI configuration type definitions

// ============ Codex MCP Server Types ============

export interface CodexMcpServer {
  enabled?: boolean | null
  transport?: 'stdio' | 'http' | null
  name: string
  // STDIO server fields
  command?: string | null
  args?: string[] | null
  env?: Record<string, string> | null
  env_vars?: string[] | null
  cwd?: string | null
  startup_timeout_ms?: number | null
  startup_timeout_sec?: number | null
  tool_timeout_sec?: number | null
  // HTTP server fields
  url?: string | null
  http_headers?: Record<string, string> | null
  env_http_headers?: Record<string, string> | null
  bearer_token?: string | null
  bearer_token_env_var?: string | null
  oauth_resource?: string | null
  scopes?: string[] | null
  // policy fields
  enabled_tools?: string[] | null
  disabled_tools?: string[] | null
  required?: boolean | null
}

export interface CodexMcpServerRequest {
  enabled?: boolean | null
  name?: string
  command?: string | null
  args?: string[] | null
  env?: Record<string, string> | null
  env_vars?: string[] | null
  cwd?: string | null
  startup_timeout_ms?: number | null
  startup_timeout_sec?: number | null
  tool_timeout_sec?: number | null
  url?: string | null
  http_headers?: Record<string, string> | null
  env_http_headers?: Record<string, string> | null
  bearer_token?: string | null
  bearer_token_env_var?: string | null
  oauth_resource?: string | null
  scopes?: string[] | null
  enabled_tools?: string[] | null
  disabled_tools?: string[] | null
  required?: boolean | null
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
  description?: string | null
  base_url?: string | null
  auth_token?: string | null
  model?: string | null
  small_fast_model?: string | null
  provider?: string | null
  provider_type?: string | null
  account?: string | null
  tags?: string[] | null
  enabled?: boolean | null
  wire_api?: string | null
  env_key?: string | null
  requires_openai_auth?: boolean | null
  approval_policy?: string | null
  sandbox_mode?: string | null
  model_reasoning_effort?: string | null
  network_access?: string | boolean | null
  disable_response_storage?: boolean | null
  auth_mode?: CodexProfileAuthMode | null
  openai_login_method?: OpenAiLoginMethod | null
  extra?: Record<string, unknown> | null
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
    notifications?: boolean | string[]
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
  auth_method?: 'chatgpt' | 'api' | string
  api_base_url?: string
  api_provider_name?: string
  saved_at?: string
  last_used?: string
  last_refresh?: string
  plan_type?: string
}

/** Codex Auth 当前信息 */
export interface CodexAuthCurrentInfo {
  account_id: string
  auth_method?: 'chatgpt' | 'api' | string
  email?: string
  plan_type?: string
  last_refresh?: string
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
  force?: boolean
}

/** Codex Auth 进程检测响应 */
export interface CodexAuthProcessResponse {
  has_running_process: boolean
  pids: number[]
  warning?: string
}

export interface CodexOAuthStartResponse {
  loginId: string
  authUrl: string
}

export interface CodexAuthMutationResponse {
  success: boolean
  account_name?: string
  switched?: boolean
  imported?: number
  results?: unknown[]
  message?: string
}

export interface CodexImportAuthPayload {
  content: string
  switchAfterImport?: boolean
  preferredAccountName?: string | null
}

export interface CodexAddApiKeyAuthPayload {
  apiKey: string
  apiBaseUrl?: string | null
  providerName?: string | null
  saveProvider?: boolean
  switchAfterAdd?: boolean
  preferredAccountName?: string | null
}

export interface CodexModelProviderApiKey {
  id: string
  name: string
  api_key: string
  created_at: string
  updated_at: string
}

export interface CodexModelProviderRecord {
  id: string
  name: string
  base_url: string
  website_url?: string | null
  api_key_url?: string | null
  api_keys: CodexModelProviderApiKey[]
  created_at: string
  updated_at: string
}

export interface CodexModelProvidersResponse {
  providers: CodexModelProviderRecord[]
}

// ============ Codex Agent Management Types ============

export type CodexAgentContextMode = 'global' | 'project'

export interface CodexAgentContext {
  mode: CodexAgentContextMode
  label: string
  agentsDir: string
  projectRoot?: string
}

export interface CodexAgentContextRequest {
  mode?: CodexAgentContextMode
  projectRoot?: string | null
}

export interface CodexAgentDiagnostic {
  fileName: string
  path: string
  severity: string
  message: string
}

export interface CodexAgentRecord {
  name: string
  fileName: string
  path: string
  description?: string
  developerInstructions?: string
  nicknameCandidates?: string[]
  model?: string
  modelReasoningEffort?: string
  sandboxMode?: string
  mcpServers?: Record<string, unknown>
  other?: Record<string, unknown>
  rawToml?: string
  parseError?: string
}

export interface CodexAgentsResponse {
  context: CodexAgentContext
  agents: CodexAgentRecord[]
  diagnostics: CodexAgentDiagnostic[]
}

export interface CodexAgentMutationResponse {
  message: string
  context: CodexAgentContext
  agent: CodexAgentRecord
  diagnostics?: CodexAgentDiagnostic[]
  sourceContext?: CodexAgentContext
  targetContext?: CodexAgentContext
  sourceRawToml?: string
}

export interface CodexAgentUpsertRequest {
  name?: string
  description?: string | null
  developerInstructions?: string | null
  nicknameCandidates?: string[] | null
  model?: string | null
  modelReasoningEffort?: string | null
  sandboxMode?: string | null
  mcpServers?: Record<string, unknown> | null
  other?: Record<string, unknown> | null
  rawToml?: string | null
  newName?: string | null
  context?: CodexAgentContextRequest
}

export type CodexAgent = CodexAgentRecord
export type CodexAgentListResponse = CodexAgentsResponse
export type CodexAgentRequest = CodexAgentUpsertRequest

// ============ Codex GitHub Agent Sources ============

export interface CodexAgentSourceRecord {
  id: string
  repoUrl: string
  owner: string
  repo: string
  defaultBranch: string
  status: string
  lastScannedAt?: string
  lastError?: string
  agentCount: number
  diagnosticsCount: number
  scanComplete: boolean
  isStale: boolean
  cacheTtlSeconds: number
}

export interface CodexAgentSourceDiagnostic {
  path: string
  severity: string
  message: string
}

export interface CodexRemoteAgentRecord {
  id: string
  sourceId: string
  sourcePath: string
  fileName: string
  blobSha: string
  contentHash: string
  category: string
  categoryLabel: string
  name: string
  description?: string
  developerInstructions?: string
  nicknameCandidates?: string[]
  model?: string
  modelReasoningEffort?: string
  sandboxMode?: string
  mcpServers?: Record<string, unknown>
  other?: Record<string, unknown>
  rawToml: string
  parseError?: string
}

export interface CodexSourceInstallRecord {
  id: string
  sourceId: string
  repoUrl: string
  sourcePath: string
  installedName: string
  targetPath: string
  status: string
  lastSyncedAt?: string
  lastError?: string
  hasUpstreamUpdate: boolean
  hasLocalChanges: boolean
}

export interface CodexAgentSourcesResponse {
  sources: CodexAgentSourceRecord[]
}

export interface CodexAgentSourceCatalogResponse {
  source: CodexAgentSourceRecord
  agents: CodexRemoteAgentRecord[]
  diagnostics: CodexAgentSourceDiagnostic[]
  installs: CodexSourceInstallRecord[]
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

export interface CodexTrayAccountRow {
  name: string
  description?: string
  email?: string
  is_current: boolean
  is_virtual: boolean
  saved_at?: string
  last_used?: string
  last_refresh?: string
  plan_type?: string
  can_switch: boolean
  quota?: CodexQuota
  quota_error?: string
  quota_fetched_at?: string
}

export interface CodexTraySnapshot {
  fetched_at: string
  runtime_mode: string
  runtime_description: string
  profile_label: string
  auth_label: string
  current_profile_name?: string
  current_profile_provider?: string
  current_profile_auth_mode?: CodexProfileAuthMode | string
  current_auth_name?: string
  login_state: LoginState
  can_manage_accounts: boolean
  current_account?: CodexTrayAccountRow | null
  accounts: CodexTrayAccountRow[]
}

// ============ Codex Session Types ============

export interface CodexSessionSummary {
  session_id: string
  file_path: string
  relative_path: string
  cwd?: string | null
  model?: string | null
  cli_version?: string | null
  originator?: string | null
  source?: string | null
  created_at?: string | null
  updated_at?: string | null
  message_count: number
  preview?: string | null
  total_input_tokens: number
  total_output_tokens: number
  total_requests: number
}

export interface CodexSessionMessage {
  role: string
  text: string
  timestamp?: string | null
}

export interface CodexSessionsResponse {
  sessions: CodexSessionSummary[]
}

export interface CodexSessionDetailResponse {
  session: CodexSessionSummary
  messages: CodexSessionMessage[]
  clipped: boolean
  message_limit: number
}

export interface CodexSessionExportResponse {
  session_id: string
  file_name: string
  content: string
  truncated: boolean
  max_messages: number
}

export interface CodexCloneSessionResponse {
  message: string
  session: CodexSessionSummary
}
