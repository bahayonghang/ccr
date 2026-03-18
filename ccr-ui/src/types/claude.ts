// Claude Code feature type definitions: Hooks, Output Styles, Statusline

// ============ Hooks Management Types ============

export type HookType = 'PreToolUse' | 'PostToolUse' | 'Stop' | 'SessionStart' | 'SessionEnd' | 'Error';

export interface Hook {
  name: string;
  hook_type: HookType;
  command: string;
  args?: string[];
  enabled?: boolean;
}

export interface HookRequest {
  name: string;
  hook_type: HookType;
  command: string;
  args?: string[];
  enabled?: boolean;
}

export interface HooksResponse {
  hooks: Hook[];
}

// ============ Output Styles Management Types ============

export interface OutputStyle {
  name: string;
  content: string;
}

export interface OutputStyleRequest {
  name: string;
  content: string;
}

export interface UpdateOutputStyleRequest {
  content: string;
}

// ============ Statusline Configuration Types ============

export interface StatuslineConfig {
  command?: string;
  enabled: boolean;
}

// ============ Claude Profile Management Types ============

/** Claude Profile（对齐 CCR Core ProfileConfig） */
export interface ClaudeProfile {
  name: string;
  description?: string | null;
  base_url?: string | null;
  auth_token?: string | null;
  model?: string | null;
  small_fast_model?: string | null;
  provider?: string | null;
  provider_type?: string | null;
  account?: string | null;
  tags?: string[] | null;
  usage_count?: number | null;
  enabled?: boolean | null;
  platform_data?: Record<string, unknown>;
  is_current: boolean;
}

/** 创建/更新 Profile 请求 */
export interface ClaudeProfileRequest {
  name: string;
  description?: string;
  base_url?: string;
  auth_token?: string;
  model?: string;
  small_fast_model?: string;
  provider?: string;
  provider_type?: string;
  account?: string;
  tags?: string[];
  usage_count?: number;
  enabled?: boolean;
  platform_data?: Record<string, unknown>;
  extra?: Record<string, unknown>;
}

/** Profile 列表响应 */
export interface ClaudeProfilesResponse {
  profiles: ClaudeProfile[];
  current_profile: string | null;
}
