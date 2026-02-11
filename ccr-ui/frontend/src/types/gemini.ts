// Gemini CLI configuration type definitions

// ============ Gemini MCP Server Types ============

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

// ============ Gemini Config Types ============

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
