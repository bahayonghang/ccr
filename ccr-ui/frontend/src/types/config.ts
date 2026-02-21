// Config management, system, version & command type definitions

// ============ Command Execution Types ============

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

// ============ Config Management Types ============

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

// ============ History Types ============

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

// ============ System Info Types ============

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

// ============ Clean Backup Types ============

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

// ============ Export/Import Types ============

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

// ============ Version Management Types ============

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

export interface CliVersionEntry {
  platform: string;
  installed: boolean;
  version?: string;
  status?: 'ok' | 'not_installed' | 'timeout' | 'error';
  elapsed_ms?: number;
}

export interface CliVersionsResponse {
  versions: CliVersionEntry[];
}

export type CliVersionsMode = 'fast' | 'full';
