// Droid CLI configuration type definitions

// ============ Droid MCP Server Types ============

export interface DroidMcpServer {
  name: string;
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  url?: string;
  httpUrl?: string;
  headers?: Record<string, string>;
  timeout?: number;
  transportType?: string;
}

export interface DroidMcpServerRequest {
  name: string;
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  url?: string;
  httpUrl?: string;
  headers?: Record<string, string>;
  timeout?: number;
}

// ============ Droid Plugin Types ============

export interface DroidPlugin {
  id: string;
  data: Record<string, unknown>;
}
