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

