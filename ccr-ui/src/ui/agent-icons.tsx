import type { CSSProperties } from 'react'
import { cn } from './cn'
import { SIcon } from './s-icon'

const AGENT_META: Record<string, { label: string; icon: string; color: string }> = {
  claude: { label: 'Claude', icon: 'Code2', color: 'var(--color-platform-claude)' },
  codex: { label: 'Codex', icon: 'Settings', color: 'var(--color-platform-codex)' },
  gemini: { label: 'Gemini', icon: 'Sparkles', color: 'var(--color-platform-gemini)' },
  opencode: { label: 'OpenCode', icon: 'TerminalSquare', color: 'var(--color-platform-opencode)' },
}

interface AgentIconsProps {
  agents: string[]
  compact?: boolean
  maxVisible?: number
  className?: string
}

export function AgentIcons({
  agents,
  compact = true,
  maxVisible = 4,
  className,
}: AgentIconsProps) {
  const resolved = agents
    .map((id) => {
      const meta = AGENT_META[id]
      return meta ? { id, ...meta } : null
    })
    .filter((item): item is { id: string; label: string; icon: string; color: string } => item !== null)
  const visible = resolved.slice(0, maxVisible)
  const overflowCount = Math.max(0, resolved.length - maxVisible)

  return (
    <div className={cn('agent-icons', compact && 'agent-icons--compact', className)}>
      {visible.map((agent) => (
        <span
          key={agent.id}
          className="agent-icons__chip"
          style={{ '--agent-color': agent.color } as CSSProperties}
          title={agent.label}
        >
          <SIcon name={agent.icon} size="w-3 h-3" />
          {compact ? null : <span className="agent-icons__label">{agent.label}</span>}
        </span>
      ))}
      {overflowCount > 0 ? (
        <span className="agent-icons__overflow" title={`${overflowCount} more agent(s)`}>
          +{overflowCount}
        </span>
      ) : null}
    </div>
  )
}
