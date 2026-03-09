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

/** Claude Profile 快照内容 */
export interface ClaudeProfileSnapshot {
  settings: Record<string, unknown>;
  mcp_servers: Record<string, unknown>;
  output_styles: { name: string; content: string }[];
  budget?: Record<string, unknown>;
}

/** Claude Profile 列表项 */
export interface ClaudeProfile {
  id: string;
  name: string;
  description?: string;
  /** 列表接口不再返回 snapshot_json，改为统计摘要 */
  snapshot_json?: string;
  snapshot_stats?: {
    mcp_count: number;
    style_count: number;
  };
  tags?: string;
  is_current: boolean;
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

/** 创建/更新 Profile 请求 */
export interface ClaudeProfileRequest {
  name: string;
  description?: string;
  snapshot_from_current?: boolean;
  snapshot_json?: string;
  tags?: string[];
  enabled?: boolean;
}

/** Profile 列表响应 */
export interface ClaudeProfilesResponse {
  profiles: ClaudeProfile[];
  current_profile: string | null;
}
