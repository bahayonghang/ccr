// iFlow CLI configuration type definitions

// ============ iFlow MCP Server Types (no name field, uses command/url as identifier) ============

export interface IflowMcpServer {
  command?: string;
  url?: string;
  args?: string[];
  env?: Record<string, string>;
}

export interface IflowMcpServerRequest {
  command?: string;
  url?: string;
  args?: string[];
  env?: Record<string, string>;
}

export interface IflowMcpServersResponse {
  servers: IflowMcpServer[];
}
