import { memo, useCallback } from 'react'
import { SIcon, cn } from '@/ui'
import type { AgentSessionProviderStatusDto } from '@/types/generated/agent_sessions/AgentSessionProviderStatusDto'
import type { AgentSessionAgentDto } from '@/types/generated/agent_sessions/AgentSessionAgentDto'
import type { TranslateFunction } from '@/utils/tf'
import {
  AGENT_SESSION_AGENTS,
  AGENT_SESSION_ICONS,
  formatAgentName,
  formatAvailability,
  formatFidelity,
} from './model'

interface ProviderButtonProps {
  agent: AgentSessionAgentDto
  status?: AgentSessionProviderStatusDto
  selected: boolean
  pending: boolean
  failed: boolean
  t: TranslateFunction
  onToggle: (agent: AgentSessionAgentDto) => void
}

const ProviderButton = memo(function ProviderButton({
  agent,
  status,
  selected,
  pending,
  failed,
  t,
  onToggle,
}: ProviderButtonProps) {
  const handleToggle = useCallback(() => {
    onToggle(agent)
  }, [agent, onToggle])
  const statusText = status
    ? formatAvailability(status.availability, t)
    : failed ? t('agentSessions.error') : pending ? t('common.loading') : t('agentSessions.noData')
  const fidelityText = status?.fidelity ? formatFidelity(status.fidelity, t) : null

  return (
    <button
      type="button"
      aria-pressed={selected}
      aria-label={`${formatAgentName(agent, t)} · ${statusText}`}
      className={cn(
        'min-w-0 rounded-xl border bg-bg-elevated px-3 py-3 text-left transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-primary',
        selected ? 'border-accent-primary/50 bg-accent-primary/10' : 'border-border-default/20 hover:border-border-default/50',
      )}
      onClick={handleToggle}
    >
      <span className="flex items-center gap-2">
        <SIcon name={AGENT_SESSION_ICONS[agent]} size="w-4 h-4" className="shrink-0 text-text-secondary" />
        <strong className="whitespace-nowrap text-xs font-semibold text-text-primary">{formatAgentName(agent, t)}</strong>
      </span>
      <span className="mt-2 flex items-center justify-between gap-2 text-xs text-text-muted">
        <span className="truncate">{statusText}</span>
        <span className="shrink-0 tabular-nums">{status?.source_count ?? 0}</span>
      </span>
      {fidelityText ? <span className="mt-1 block truncate text-xs text-text-ghost">{fidelityText}</span> : null}
    </button>
  )
})

interface AgentSessionProviderStripProps {
  statuses: AgentSessionProviderStatusDto[]
  selectedAgents: AgentSessionAgentDto[]
  pending: boolean
  failed: boolean
  t: TranslateFunction
  onToggle: (agent: AgentSessionAgentDto) => void
}

export const AgentSessionProviderStrip = memo(function AgentSessionProviderStrip({
  statuses,
  selectedAgents,
  pending,
  failed,
  t,
  onToggle,
}: AgentSessionProviderStripProps) {
  const statusByAgent = new Map(statuses.map((status) => [status.agent, status]))
  return (
    <section aria-labelledby="agent-provider-status-title">
      <div className="mb-3 flex items-center justify-between gap-3">
        <h2 id="agent-provider-status-title" className="text-sm font-semibold text-text-primary">
          {t('agentSessions.providers')}
        </h2>
        <span className="text-xs text-text-muted">{t('agentSessions.providerFilterHint')}</span>
      </div>
      <div className="grid grid-cols-2 gap-2 sm:grid-cols-4 xl:grid-cols-8">
        {AGENT_SESSION_AGENTS.map((agent) => (
          <ProviderButton
            key={agent}
            agent={agent}
            status={statusByAgent.get(agent)}
            selected={selectedAgents.includes(agent)}
            pending={pending}
            failed={failed}
            t={t}
            onToggle={onToggle}
          />
        ))}
      </div>
    </section>
  )
})
