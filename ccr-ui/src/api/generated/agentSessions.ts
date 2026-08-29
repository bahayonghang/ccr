/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@/api/invokeRuntime'
import type { AgentSessionDetailDto } from '@/types/generated/agent_sessions/AgentSessionDetailDto'
import type { AgentSessionDetailRequestDto } from '@/types/generated/agent_sessions/AgentSessionDetailRequestDto'
import type { AgentSessionListRequestDto } from '@/types/generated/agent_sessions/AgentSessionListRequestDto'
import type { AgentSessionPageDto } from '@/types/generated/agent_sessions/AgentSessionPageDto'
import type { AgentSessionProviderStatusDto } from '@/types/generated/agent_sessions/AgentSessionProviderStatusDto'
import type { SessionIndexJobSnapshot } from '@/types/generated/usage/SessionIndexJobSnapshot'
import type { StartSessionIndexJobResponse } from '@/types/generated/usage/StartSessionIndexJobResponse'

export const agentSessionsList = (request: AgentSessionListRequestDto): Promise<AgentSessionPageDto> => invoke('agent_sessions_list', { request })
export const agentSessionsGetDetail = (request: AgentSessionDetailRequestDto): Promise<AgentSessionDetailDto> => invoke('agent_sessions_get_detail', { request })
export const agentSessionsGetProviderStatus = (): Promise<AgentSessionProviderStatusDto[]> => invoke('agent_sessions_get_provider_status')
export const agentSessionsStartRefresh = (): Promise<StartSessionIndexJobResponse> => invoke('agent_sessions_start_refresh')
export const agentSessionsGetRefreshStatus = (jobId: string): Promise<SessionIndexJobSnapshot> => invoke('agent_sessions_get_refresh_status', { jobId })
