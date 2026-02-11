// Config converter type definitions

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
