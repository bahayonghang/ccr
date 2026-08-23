export function loadMcpManagerView() {
  return import('./McpManagerView').then((mod) => ({ Component: mod.McpManagerView }))
}

export const mcpRouteLoaders = {
  'mcp-manager': loadMcpManagerView,
} as const
