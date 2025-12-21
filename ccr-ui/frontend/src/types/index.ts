// TypeScript type definitions for CCR UI

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  message?: string;
}

// Statistics types
export interface TokenStats {
  total_input_tokens: number;
  total_output_tokens: number;
  total_cache_tokens: number;
  cache_efficiency: number;
}

export interface DailyCost {
  date: string;
  cost: number;
  count: number;
}

export interface CostStats {
  total_cost: number;
  record_count: number;
  token_stats: TokenStats;
  by_provider: Record<string, number>; // provider -> count
  by_model: Record<string, number>;
  by_project: Record<string, number>;
  trend?: DailyCost[];
}

export interface TopSession {
  session_id: string;
  cost: number;
}

export interface StatsSummary {
  today_cost: number;
  week_cost: number;
  month_cost: number;
  total_sessions: number;
}

// Usage Analytics types
export interface UsageData {
  input_tokens?: number;
  output_tokens?: number;
  cache_read_input_tokens?: number;
}

export interface UsageRecord {
  uuid: string;
  timestamp: string;
  model?: string;
  usage?: UsageData;
}

export interface UsageRecordsResponse {
  records: UsageRecord[];
  total_records: number;
  truncated: boolean;
}

export type TimeRange = '5h' | 'today' | '7d' | 'week' | 'month' | 'all';

// ============ Usage Stats Dashboard Types ============

/** 使用统计视图模式 */
export type StatsViewMode = 'sessions' | 'duration' | 'tokens';

/** 平台每日统计 */
export interface PlatformDailyStats {
  sessions: number;
  messages: number;
  tokens: number;
  duration_seconds: number;
}

/** 每日统计项 */
export interface DailyStatsItem {
  date: string;
  claude: PlatformDailyStats;
  codex: PlatformDailyStats;
  gemini: PlatformDailyStats;
}

/** 使用统计汇总 */
export interface UsageStatsSummary {
  total_sessions: number;
  total_messages: number;
  total_duration_seconds: number;
  by_platform: Record<string, PlatformDailyStats>;
}

/** 每日统计 API 响应 */
export interface DailyStatsResponse {
  daily_stats: DailyStatsItem[];
  summary: UsageStatsSummary;
  last_updated: string;
}


// Budget Management types
export interface BudgetStatus {
  enabled: boolean;
  daily_limit: number | null;
  weekly_limit: number | null;
  monthly_limit: number | null;
  warn_threshold: number;
  current_costs: {
    today: number;
    this_week: number;
    this_month: number;
  };
  warnings: BudgetWarning[];
  last_updated: string;
}

export interface BudgetWarning {
  period: string;
  current_cost: number;
  limit: number;
  usage_percent: number;
}

export interface SetBudgetRequest {
  enabled?: boolean;
  daily_limit?: number | null;
  weekly_limit?: number | null;
  monthly_limit?: number | null;
  warn_threshold?: number;
}

// Pricing Management types
export interface ModelPricing {
  model: string;
  input_price: number;
  output_price: number;
  cache_read_price?: number;
  cache_write_price?: number;
}

export interface PricingListResponse {
  pricings: ModelPricing[];
  default_pricing: ModelPricing;
}

export interface SetPricingRequest {
  model: string;
  input_price: number;
  output_price: number;
  cache_read_price?: number;
  cache_write_price?: number;
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
  category?: string;
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
  // 📊 使用统计和状态字段
  usage_count?: number;
  enabled?: boolean;
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
  name?: string;
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

// Codex Profile Types (CCR 平台：~/.ccr/platforms/codex/profiles.toml)
export interface CodexProfile {
  name: string;
  description?: string;
  base_url: string;
  auth_token: string;
  model: string;
  small_fast_model?: string;
  provider?: string;
  provider_type?: string;
  account?: string;
  tags?: string[];
  usage_count?: number;
  enabled?: boolean;
  extra?: Record<string, any>;
}

export interface CodexProfileRequest {
  name: string;
  description?: string;
  base_url: string;
  auth_token: string;
  model: string;
  small_fast_model?: string;
  provider?: string;
  provider_type?: string;
  account?: string;
  tags?: string[];
  enabled?: boolean;
  extra?: Record<string, any>;
}

export interface CodexProfilesResponse {
  profiles: CodexProfile[];
  current_profile?: string | null;
}

export interface CodexProfileResponse {
  profile: CodexProfile;
}

// Codex Base Config Types
// 说明：这是 Codex CLI 的 ~/.codex/config.toml，不同于 CCR 的 profiles.toml
export interface CodexCliProfile {
  model?: string;
  approval_policy?: string;
  sandbox_mode?: string;
  model_reasoning_effort?: string;
  [key: string]: any;
}

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
  profiles?: Record<string, CodexCliProfile>;
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
  url?: string;  // HTTP server URL
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
  url?: string;  // HTTP server URL
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

// ============ iFlow CLI Configuration Types ============

// iFlow MCP Server Types (no name field, uses command/url as identifier)
export interface IflowMcpServer {
  command?: string;
  url?: string;
  args?: string[];
  env?: Record<string, string>;
}

export interface IflowMcpServerRequest {
  command?: string;
  url?: string;
  args?: string[];
  env?: Record<string, string>;
}

export interface IflowMcpServersResponse {
  servers: IflowMcpServer[];
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
