import type { AgentSessionAgentDto } from '@/types/generated/agent_sessions/AgentSessionAgentDto'
import type { AgentSessionAvailabilityDto } from '@/types/generated/agent_sessions/AgentSessionAvailabilityDto'
import type { AgentSessionFidelityDto } from '@/types/generated/agent_sessions/AgentSessionFidelityDto'
import type { TranslateFunction } from '@/utils/tf'

export const AGENT_SESSION_AGENTS: AgentSessionAgentDto[] = [
  'grok',
  'claude',
  'codex',
  'opencode',
  'pi',
  'omp',
  'antigravity',
  'kimi',
]

export const AGENT_SESSION_ICONS: Record<AgentSessionAgentDto, string> = {
  grok: 'Zap',
  claude: 'Code2',
  codex: 'Settings',
  opencode: 'TerminalSquare',
  pi: 'Bot',
  omp: 'Workflow',
  antigravity: 'Sparkles',
  kimi: 'Moon',
}

export interface AgentSessionFilterValues {
  q: string
  cwd: string
  startedAt: string
  endedAt: string
  sourceState: string
  fidelity: string
}

export const DEFAULT_AGENT_SESSION_FILTERS: AgentSessionFilterValues = {
  q: '',
  cwd: '',
  startedAt: '',
  endedAt: '',
  sourceState: 'all',
  fidelity: 'all',
}

export const formatAgentName = (agent: AgentSessionAgentDto, t: TranslateFunction): string =>
  t(`agentSessions.agents.${agent}`)

export const formatAvailability = (
  availability: AgentSessionAvailabilityDto,
  t: TranslateFunction,
): string => {
  const keys: Record<AgentSessionAvailabilityDto, string> = {
    not_installed: 'agentSessions.notInstalled',
    no_data: 'agentSessions.noData',
    available: 'agentSessions.available',
    error: 'agentSessions.error',
  }
  return t(keys[availability])
}

export const formatFidelity = (
  fidelity: AgentSessionFidelityDto,
  t: TranslateFunction,
): string => t(`agentSessions.${fidelity}`)

export const formatSessionTime = (value: string, locale: string, fallback: string): string => {
  const parsed = new Date(value)
  if (Number.isNaN(parsed.getTime())) return fallback
  return new Intl.DateTimeFormat(locale, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(parsed)
}

export const dateBoundary = (value: string, endOfDay: boolean): string | undefined => {
  if (!value) return undefined
  const suffix = endOfDay ? 'T23:59:59.999' : 'T00:00:00.000'
  const parsed = new Date(`${value}${suffix}`)
  return Number.isNaN(parsed.getTime()) ? undefined : parsed.toISOString()
}

const AGENT_SESSION_ERROR_CODE = /(?:^|:)(agent_session_[a-z0-9_]+)$/

export const extractAgentSessionErrorCode = (error: string): string | undefined => {
  const trimmed = error.trim()
  if (/^agent_session_[a-z0-9_]+$/.test(trimmed)) {
    return trimmed
  }
  return trimmed.match(AGENT_SESSION_ERROR_CODE)?.[1]
}

export const isUnreadableAgentSessionError = (error: string): boolean => {
  const code = extractAgentSessionErrorCode(error)
  return code === 'agent_session_source_unavailable' || code === 'agent_session_source_validation_failed'
}

export const resolveAgentSessionDetailError = (
  error: string,
  t: TranslateFunction,
): { title: string, description: string } => {
  const code = extractAgentSessionErrorCode(error)
  if (code === 'agent_session_source_unavailable') {
    return {
      title: t('agentSessions.missing'),
      description: t('agentSessions.sourceUnavailableDescription'),
    }
  }
  if (code) {
    const key = `agentSessions.errors.${code}`
    const translated = t(key)
    return {
      title: t('agentSessions.error'),
      description: translated === key ? t('agentSessions.errors.generic') : translated,
    }
  }
  return {
    title: t('agentSessions.error'),
    description: t('agentSessions.errors.generic'),
  }
}

