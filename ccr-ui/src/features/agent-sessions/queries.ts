import type { AgentSessionListRequestDto } from '@/types/generated/agent_sessions/AgentSessionListRequestDto'

export const agentSessionKeys = {
  all: ['agent-sessions'] as const,
  providers: (environmentId: string | null) => [...agentSessionKeys.all, 'providers', environmentId] as const,
  list: (environmentId: string | null, request: AgentSessionListRequestDto) =>
    [...agentSessionKeys.all, 'list', environmentId, request] as const,
  detail: (environmentId: string | null, archiveId: string) =>
    [...agentSessionKeys.all, 'detail', environmentId, archiveId] as const,
  refresh: (environmentId: string | null, jobId: string) =>
    [...agentSessionKeys.all, 'refresh', environmentId, jobId] as const,
}
