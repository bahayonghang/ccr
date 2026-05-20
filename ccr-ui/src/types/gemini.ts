// Antigravity CLI configuration type definitions (legacy key: gemini)

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
  general?: Record<string, unknown>;
  output?: Record<string, unknown>;
  ui?: Record<string, unknown>;
  model?: Record<string, unknown>;
  context?: Record<string, unknown>;
  tools?: Record<string, unknown>;
  mcp?: Record<string, unknown>;
  security?: Record<string, unknown>;
  advanced?: Record<string, unknown>;
  experimental?: Record<string, unknown>;
}

export interface GeminiConfigResponse {
  config: GeminiConfig;
}
