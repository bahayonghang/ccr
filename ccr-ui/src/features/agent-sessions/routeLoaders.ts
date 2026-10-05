export function loadAgentSessionsView() {
  return import('./AgentSessionsView').then((mod) => ({ Component: mod.AgentSessionsView }))
}

export const agentSessionsRouteLoaders = {
  'agent-sessions': loadAgentSessionsView,
} as const
