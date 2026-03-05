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
