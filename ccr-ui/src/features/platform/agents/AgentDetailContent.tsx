import { Link } from 'react-router'
import type { Agent } from '@/types'
import { SIcon } from '@/ui'
import type { TranslateFunction } from '@/utils/tf'

interface AgentDetailContentProps {
  loading: boolean
  error: string | null
  agent: Agent | null
  copied: boolean
  t: TranslateFunction
  onCopy: () => void
}

export function AgentDetailContent({
  loading,
  error,
  agent,
  copied,
  t,
  onCopy,
}: AgentDetailContentProps) {
  if (loading) {
    return <p className="py-20 text-center text-text-muted">{t('common.loading')}</p>
  }
  if (error) {
    return (
      <div className="py-20 text-center">
        <p className="text-lg font-medium text-text-primary">{t('agents.loadError')}</p>
        <p className="mt-2 text-sm text-text-muted">{error}</p>
        <Link to="/agents" className="mt-4 inline-flex items-center gap-2 rounded-lg bg-bg-elevated px-4 py-2 text-sm font-medium">
          <SIcon name="ArrowLeft" size="w-4 h-4" />
          {t('common.back')}
        </Link>
      </div>
    )
  }
  if (!agent) return null
  return (
    <div>
      {agent.tools && agent.tools.length > 0 ? (
        <div className="mb-6 rounded-xl border border-border-default/25 bg-bg-surface p-6">
          <h2 className="mb-4 flex items-center gap-2 text-lg font-bold text-text-primary">
            <SIcon name="Wrench" size="w-5 h-5" className="text-accent-secondary" />
            {t('agents.toolsLabel')}
            <span className="text-sm font-normal text-text-muted">({agent.tools.length})</span>
          </h2>
          <div className="flex flex-wrap gap-2">
            {agent.tools.map((tool) => (
              <span key={tool} className="rounded-lg border border-border-default/50 bg-bg-surface px-3 py-1.5 text-sm text-text-primary">
                {tool}
              </span>
            ))}
          </div>
        </div>
      ) : null}
      <div className="rounded-xl border border-border-default/25 bg-bg-surface p-6">
        <div className="mb-4 flex items-center justify-between">
          <h2 className="flex items-center gap-2 text-lg font-bold text-text-primary">
            <SIcon name="FileText" size="w-5 h-5" className="text-accent-secondary" />
            {t('agents.systemPromptLabel')}
          </h2>
          {agent.system_prompt ? (
            <button
              type="button"
              className="inline-flex items-center gap-1.5 rounded-lg bg-bg-surface px-3 py-1.5 text-xs font-medium text-text-secondary"
              onClick={onCopy}
            >
              <SIcon name="Copy" size="w-3.5 h-3.5" />
              {copied ? t('common.copied') : t('common.copy')}
            </button>
          ) : null}
        </div>
        {agent.system_prompt ? (
          <pre className="max-h-96 overflow-auto rounded-xl border border-border-default/30 bg-bg-elevated p-4">
            <code className="whitespace-pre-wrap break-words font-mono text-sm leading-relaxed text-text-primary">
              {agent.system_prompt}
            </code>
          </pre>
        ) : (
          <div className="py-12 text-center text-text-muted">
            <SIcon name="FileText" size="w-12 h-12" className="mx-auto mb-3 opacity-30" />
            <p>{t('agents.noSystemPrompt')}</p>
          </div>
        )}
      </div>
    </div>
  )
}
