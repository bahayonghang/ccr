// Qwen CLI configuration type definitions

// ============ Qwen MCP Server Types ============

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

// ============ Qwen Config Types ============

export interface QwenConfig {
  mcpServers?: Record<string, Omit<QwenMcpServer, 'name'>>;
}

export interface QwenConfigResponse {
  config: QwenConfig;
}
