// Qoder CLI configuration type definitions

// ============ Qoder MCP Server Types ============

export interface QoderMcpServer {
  name?: string;
  command?: string;
  url?: string;
  args?: string[];
  env?: Record<string, string>;
}

export interface QoderMcpServerRequest {
  name: string;
  command?: string;
  url?: string;
  args?: string[];
  env?: Record<string, string>;
}

export interface QoderMcpServersResponse {
  servers: QoderMcpServer[];
}
