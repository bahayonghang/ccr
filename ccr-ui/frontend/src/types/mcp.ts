// MCP Server, Agent, Plugin & Slash Command type definitions (Claude Code)

// ============ MCP Server Management Types ============

export interface McpServer {
  name: string;
  command: string;
  args: string[];
  env?: Record<string, string>;
  disabled?: boolean;
}

export interface McpServerRequest {
  name: string;
  command: string;
  args: string[];
  env?: Record<string, string>;
  disabled?: boolean;
}

export interface McpServersResponse {
  servers: McpServer[];
}

// ============ Slash Command Management Types ============

export interface SlashCommand {
  name: string;
  description: string;
  command: string;
  args?: string[];
  disabled?: boolean;
  folder?: string; // Folder path: '' for root, 'subfolder' for subfolder
}

export interface SlashCommandRequest {
  name: string;
  description: string;
  command: string;
  args?: string[];
  disabled?: boolean;
}

export interface SlashCommandsResponse {
  commands: SlashCommand[];
  folders?: string[]; // List of subdirectories
}

// ============ Agent Management Types ============

export interface Agent {
  name: string;
  model: string;
  tools: string[];
  system_prompt?: string;
  disabled?: boolean;
  folder?: string; // Folder path: '' for root, 'kfc' for subfolder
}

export interface AgentRequest {
  name: string;
  model: string;
  tools?: string[];
  system_prompt?: string;
  disabled?: boolean;
}

export interface AgentsResponse {
  agents: Agent[];
  folders?: string[]; // List of subdirectories
}

// ============ Plugin Management Types ============

export interface Plugin {
  id: string;
  name: string;
  version: string;
  enabled: boolean;
  config?: Record<string, any>;
}

export interface PluginRequest {
  id: string;
  name: string;
  version: string;
  enabled?: boolean;
  config?: Record<string, any>;
}

export interface PluginsResponse {
  plugins: Plugin[];
}
