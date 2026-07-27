// Config management, system, version & command type definitions

// ============ Command Execution Types ============

import type { CommandArgSchema as GeneratedCommandArgSchema } from './generated/command_exec/CommandArgSchema'
import type { CommandInfo as GeneratedCommandInfo } from './generated/command_exec/CommandInfo'

export interface CommandRequest {
  command: string;
  args?: string[];
  confirmationToken?: string | null;
}

export type CommandArgSchema = Pick<
  GeneratedCommandArgSchema,
  'name' | 'label' | 'type' | 'required'
> &
  Partial<Omit<GeneratedCommandArgSchema, 'name' | 'label' | 'type' | 'required'>>
export type { CommandCatalog } from './generated/command_exec/CommandCatalog'
export type {
  CommandExecutionResult,
  CommandExecutionResult as CommandResponse,
} from './generated/command_exec/CommandExecutionResult'
export type { CommandFlagSchema } from './generated/command_exec/CommandFlagSchema'
export type { CommandHelpResponse } from './generated/command_exec/CommandHelpResponse'
export type CommandInfo = Pick<
  GeneratedCommandInfo,
  'name' | 'description' | 'usage' | 'examples'
> &
  Partial<Omit<GeneratedCommandInfo, 'name' | 'description' | 'usage' | 'examples' | 'args'>> & {
    args?: CommandArgSchema[]
  }
export type { GeneratedCommandInfo }
export type { CommandJobDelta } from './generated/command_exec/CommandJobDelta'
export type { CommandJobSnapshot } from './generated/command_exec/CommandJobSnapshot'
export type { CommandJobStatus } from './generated/command_exec/CommandJobStatus'
export type { OutputChannel } from './generated/command_exec/OutputChannel'
export type { StartCommandJobResponse } from './generated/command_exec/StartCommandJobResponse'

// ============ Config Management Types ============

export interface ConfigItem {
  name: import('./generated/config/ConfigInfo').ConfigInfo['name'];
  description: import('./generated/config/ConfigInfo').ConfigInfo['description'];
  base_url: import('./generated/config/ConfigInfo').ConfigInfo['base_url'];
  auth_token: import('./generated/config/ConfigInfo').ConfigInfo['auth_token'];
  model?: string;
  small_fast_model?: string;
  is_current: boolean;
  is_default: boolean;
  provider?: string;
  provider_type?: string;
  account?: string;
  tags?: string[];
  usage_count: number;
  enabled: boolean;
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

export type HistoryEntry = import('./generated/config/HistoryEntry').HistoryEntry & {
  from_config?: string;
  to_config?: string;
  changes?: EnvChange[];
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

export type SystemInfo = import('./generated/system/SystemInfo').SystemInfo

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

export type CliVersionEntry = import('./generated/system/CliVersionEntry').CliVersionEntry
export type CliVersionsResponse = import('./generated/system/CliVersionsResponse').CliVersionsResponse

export type CliVersionsMode = 'fast' | 'full';

export interface CliVersionsOptions {
  mode?: CliVersionsMode;
  timeoutMs?: number;
  parallelism?: number;
}

export interface CliVersionCommandOptions {
  tool: string;
  timeoutMs?: number;
  force?: boolean;
}
