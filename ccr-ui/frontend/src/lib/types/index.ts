// TypeScript type definitions for CCR UI

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  message?: string;
}

// Command execution types
export interface CommandRequest {
  command: string;
  args: string[];
}

export interface CommandResponse {
  success: boolean;
  output: string;
  error: string;
  exit_code: number;
  duration_ms: number;
}

export interface CommandInfo {
  name: string;
  description: string;
  usage: string;
  examples: string[];
}

// Config management types
export interface ConfigItem {
  name: string;
  description: string;
  base_url: string;
  auth_token: string;
  model?: string;
  small_fast_model?: string;
  is_current: boolean;
  is_default: boolean;
  provider?: string;
  provider_type?: string;
  account?: string;
  tags?: string[];
}

export interface ConfigListResponse {
  current_config: string;
  default_config: string;
  configs: ConfigItem[];
}

export interface SwitchRequest {
  config_name: string;
}

export interface UpdateConfigRequest {
  name: string;
  description?: string;
  base_url: string;
  auth_token: string;
  model?: string;
  small_fast_model?: string;
  provider?: string;
  provider_type?: string;
  account?: string;
  tags?: string[];
}

// History types
export interface HistoryEntry {
  id: string;
  timestamp: string;
  operation: string;
  actor: string;
  from_config?: string;
  to_config?: string;
  changes: EnvChange[];
}

export interface EnvChange {
  key: string;
  old_value?: string;
  new_value?: string;
}

export interface HistoryResponse {
  entries: HistoryEntry[];
  total: number;
}

// System info types
export interface SystemInfo {
  hostname: string;
  os: string;
  os_version: string;
  kernel_version: string;
  cpu_brand: string;
  cpu_cores: number;
  cpu_usage: number;
  total_memory_gb: number;
  used_memory_gb: number;
  memory_usage_percent: number;
  total_swap_gb: number;
  used_swap_gb: number;
  uptime_seconds: number;
}

// Clean backup types
export interface CleanRequest {
  days: number;
  dry_run: boolean;
}

export interface CleanResponse {
  deleted_count: number;
  skipped_count: number;
  total_size_mb: number;
  dry_run: boolean;
}

// Export/Import types
export interface ExportRequest {
  include_secrets: boolean;
}

export interface ExportResponse {
  content: string;
  filename: string;
}

export interface ImportRequest {
  content: string;
  mode: 'merge' | 'replace';
  backup: boolean;
}

export interface ImportResponse {
  added: number;
  updated: number;
  skipped: number;
}

// Version management types
export interface VersionInfo {
  current_version: string;
  build_time: string;
  git_commit: string;
}

export interface UpdateCheckResponse {
  current_version: string;
  latest_version: string;
  has_update: boolean;
  release_url: string;
  release_notes?: string;
  published_at?: string;
}

export interface UpdateExecutionResponse {
  success: boolean;
  output: string;
  error: string;
  exit_code: number;
}

// MCP Server Management Types
export interface McpServer {
  name: string;
  command: string;
  args: string[];
  env?: Record<string, string>;
  disabled?: boolean;
}

export interface McpServerRequest {
  name: string;
  command: string;
  args: string[];
  env?: Record<string, string>;
  disabled?: boolean;
}

export interface McpServersResponse {
  servers: McpServer[];
}

// Slash Command Management Types
export interface SlashCommand {
  name: string;
  description: string;
  command: string;
  args?: string[];
  disabled?: boolean;
  folder?: string; // Folder path: '' for root, 'subfolder' for subfolder
}

export interface SlashCommandRequest {
  name: string;
  description: string;
  command: string;
  args?: string[];
  disabled?: boolean;
}

export interface SlashCommandsResponse {
  commands: SlashCommand[];
  folders?: string[]; // List of subdirectories
}

// Agent Management Types
export interface Agent {
  name: string;
  model: string;
  tools: string[];
  system_prompt?: string;
  disabled?: boolean;
  folder?: string; // Folder path: '' for root, 'kfc' for subfolder
}

export interface AgentRequest {
  name: string;
  model: string;
  tools?: string[];
  system_prompt?: string;
  disabled?: boolean;
}

export interface AgentsResponse {
  agents: Agent[];
  folders?: string[]; // List of subdirectories
}

// Plugin Management Types
export interface Plugin {
  id: string;
  name: string;
  version: string;
  enabled: boolean;
  config?: Record<string, any>;
}

export interface PluginRequest {
  id: string;
  name: string;
  version: string;
  enabled?: boolean;
  config?: Record<string, any>;
}

export interface PluginsResponse {
  plugins: Plugin[];
}

// ============ Codex CLI Configuration Types ============

// Codex MCP Server Types
export interface CodexMcpServer {
  name: string;
  // STDIO server fields
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  cwd?: string;
  startup_timeout_ms?: number;
  // HTTP server fields
  url?: string;
  bearer_token?: string;
}

export interface CodexMcpServerRequest {
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  cwd?: string;
  startup_timeout_ms?: number;
  url?: string;
  bearer_token?: string;
}

export interface CodexMcpServersResponse {
  servers: CodexMcpServer[];
}

// Codex Profile Types
export interface CodexProfile {
  name: string;
  model?: string;
  approval_policy?: string;
  sandbox_mode?: string;
  model_reasoning_effort?: string;
}

export interface CodexProfileRequest {
  model?: string;
  approval_policy?: string;
  sandbox_mode?: string;
  model_reasoning_effort?: string;
}

export interface CodexProfilesResponse {
  profiles: CodexProfile[];
}

// Codex Base Config Types
export interface CodexConfig {
  model?: string;
  model_provider?: string;
  model_reasoning_effort?: string;
  approval_policy?: string;
  sandbox_mode?: string;
  shell_environment_policy?: {
    include_only?: string[];
  };
  mcp_servers?: Record<string, Omit<CodexMcpServer, 'name'>>;
  profiles?: Record<string, Omit<CodexProfile, 'name'>>;
  experimental_use_rmcp_client?: boolean;
}

export interface CodexConfigResponse {
  config: CodexConfig;
}

// ============ Config Converter Types ============

export type CliType = 'claude-code' | 'codex' | 'gemini' | 'qwen' | 'iflow';

export interface ConverterRequest {
  source_format: CliType;
  target_format: CliType;
  config_data: string; // JSON or TOML string
  convert_mcp?: boolean;
  convert_commands?: boolean;
  convert_agents?: boolean;
}

export interface ConverterResponse {
  success: boolean;
  converted_data: string; // JSON or TOML string
  warnings?: string[];
  format: 'json' | 'toml';
  stats?: ConversionResult;
}

export interface ConversionResult {
  mcp_servers?: number;
  slash_commands?: number;
  agents?: number;
  profiles?: number;
  base_config?: boolean;
}

// ============ Gemini CLI Configuration Types ============

// Gemini MCP Server Types
export interface GeminiMcpServer {
  name: string;
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  cwd?: string;
  timeout?: number;
  trust?: boolean;
  includeTools?: string[];
}

export interface GeminiMcpServerRequest {
  name: string;
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  cwd?: string;
  timeout?: number;
  trust?: boolean;
  includeTools?: string[];
}

export interface GeminiMcpServersResponse {
  servers: GeminiMcpServer[];
}

// Gemini Config Types
export interface GeminiConfig {
  mcpServers?: Record<string, Omit<GeminiMcpServer, 'name'>>;
  general?: Record<string, any>;
  output?: Record<string, any>;
  ui?: Record<string, any>;
  model?: Record<string, any>;
  context?: Record<string, any>;
  tools?: Record<string, any>;
  mcp?: Record<string, any>;
  security?: Record<string, any>;
  advanced?: Record<string, any>;
  experimental?: Record<string, any>;
}

export interface GeminiConfigResponse {
  config: GeminiConfig;
}

// ============ Qwen CLI Configuration Types ============

// Qwen MCP Server Types
export interface QwenMcpServer {
  name: string;
  // STDIO fields
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  // HTTP fields
  url?: string;
  httpUrl?: string;
  headers?: Record<string, string>;
  // Common fields
  timeout?: number;
  transportType?: 'stdio' | 'sse' | 'http';
}

export interface QwenMcpServerRequest {
  name: string;
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  url?: string;
  httpUrl?: string;
  headers?: Record<string, string>;
  timeout?: number;
}

export interface QwenMcpServersResponse {
  servers: QwenMcpServer[];
}

// Qwen Config Types
export interface QwenConfig {
  mcpServers?: Record<string, Omit<QwenMcpServer, 'name'>>;
}

export interface QwenConfigResponse {
  config: QwenConfig;
}

// ============ Sync (WebDAV) Types ============
export interface SyncConfigDetails {
  enabled: boolean;
  webdav_url: string;
  username: string;
  remote_path: string;
  auto_sync: boolean;
  remote_file_exists?: boolean | null;
}

export interface SyncStatusResponse {
  success: boolean;
  output: string;
  configured: boolean;
  config?: SyncConfigDetails | null;
}

export interface SyncOperationRequest {
  force?: boolean;
}

export interface SyncOperationResponse {
  success: boolean;
  output: string;
  error: string;
  duration_ms: number;
}

export interface SyncInfoResponse {
  feature_name: string;
  description: string;
  supported_services: string[];
  setup_steps: string[];
  security_notes: string[];
}

