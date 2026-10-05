import { memo, useCallback } from 'react'
import type { McpGroup } from '@/types/mcpManager'
import type { UnifiedMcpDiagnostic, UnifiedMcpServer } from '@/types/unifiedMcp'
import type { TranslateFunction } from '@/utils/tf'
import { AgentIcons, SIcon, cn } from '@/ui'
import { buildRawConfigPreview, pickPrimaryServer, sortPrecedence } from './mcp-detail-model'
import {
  MCP_STATE_TONE_CLASS,
  maskSecret,
  mcpApprovalLabel,
  mcpScopeLabel,
  mcpStateLabel,
  mcpStateTone,
} from './mcp-format'

export interface McpDetailPanelProps {
  group: McpGroup | null
  diagnostics?: UnifiedMcpDiagnostic[]
  t: TranslateFunction
  onEdit: (groupName: string) => void
  onDelete: (group: McpGroup) => void
  onToggle: (server: UnifiedMcpServer) => void
}

interface McpInstanceRowProps {
  item: UnifiedMcpServer
  onToggle: (server: UnifiedMcpServer) => void
  t: TranslateFunction
}

const McpInstanceRow = memo(function McpInstanceRow({ item, onToggle, t }: McpInstanceRowProps) {
  const handleToggle = useCallback(() => {
    onToggle(item)
  }, [item, onToggle])

  return (
    <div className="flex items-center gap-3 rounded-xl border border-border-default/28 bg-bg-base/38 px-2.5 py-2">
      <AgentIcons agents={[item.platform]} compact={false} />
      <span className="text-sm font-semibold capitalize text-text-primary">
        {mcpScopeLabel(item.scope ?? 'global', t)}
      </span>
      <span
        className={cn(
          'ml-auto text-xs font-bold uppercase tracking-wide',
          item.disabled ? 'text-text-muted' : 'text-success',
        )}
      >
        {item.disabled ? t('mcp.manager.state.disabled') : t('mcp.manager.state.enabled')}
      </span>
      <button
        type="button"
        className="inline-flex items-center gap-1.5 rounded-full border border-border-default/55 bg-bg-elevated px-2 py-1 text-text-secondary hover:bg-bg-surface hover:text-text-primary"
        aria-label={`${item.platform}:${item.name}`}
        onClick={handleToggle}
      >
        <SIcon name={item.disabled ? 'ToggleLeft' : 'ToggleRight'} size="w-4 h-4" />
      </button>
    </div>
  )
})

function DetailEmpty({ icon, title, subtitle }: { icon: string; title: string; subtitle?: string }) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-2.5 text-center text-text-muted">
      <SIcon name={icon} size="w-8 h-8" />
      <p className="font-bold text-text-primary">{title}</p>
      {subtitle ? <span className="text-sm">{subtitle}</span> : null}
    </div>
  )
}

function StatePill({ server, t }: { server: UnifiedMcpServer; t: TranslateFunction }) {
  return (
    <span
      className={cn(
        'shrink-0 rounded-full border px-2 py-0.5 text-[0.62rem] font-extrabold uppercase tracking-wide',
        MCP_STATE_TONE_CLASS[mcpStateTone(server)],
      )}
    >
      {mcpStateLabel(server, t)}
    </span>
  )
}

interface McpDetailBodyProps {
  group: McpGroup
  primaryServer: UnifiedMcpServer
  diagnostics: UnifiedMcpDiagnostic[]
  t: TranslateFunction
  onEdit: (groupName: string) => void
  onDelete: (group: McpGroup) => void
  onToggle: (server: UnifiedMcpServer) => void
}

function McpDetailBody({
  group,
  primaryServer,
  diagnostics,
  t,
  onEdit,
  onDelete,
  onToggle,
}: McpDetailBodyProps) {
  const handleEdit = useCallback(() => {
    onEdit(group.name)
  }, [group.name, onEdit])

  const handleDelete = useCallback(() => {
    onDelete(group)
  }, [group, onDelete])

  const envEntries = Object.entries(primaryServer.env ?? {})
  const headerEntries = Object.entries(primaryServer.headers ?? {})
  const precedenceItems = sortPrecedence(group.items)
  const commandOrUrl =
    group.transportType === 'stdio' ? t('mcp.manager.detail.commandLabel') : t('mcp.manager.detail.urlLabel')

  return (
    <div className="flex h-full flex-col gap-4 overflow-y-auto p-6">
      <div className="flex items-start justify-between gap-4 pb-2 max-[53.75rem]:flex-col">
        <div className="flex items-center gap-3.5">
          <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl border border-accent-primary/22 bg-accent-primary/10 text-accent-primary">
            <SIcon name={group.transportType === 'stdio' ? 'Terminal' : 'Globe'} size="w-5 h-5" />
          </div>
          <div>
            <p className="text-[0.64rem] font-bold uppercase tracking-[0.14em] text-accent-primary/88">
              {mcpScopeLabel(primaryServer.scope ?? 'global', t)} {t('mcp.manager.detail.scopeSuffix')} ·{' '}
              {mcpStateLabel(primaryServer, t)}
            </p>
            <h2 className="mt-0.5 font-serif text-[1.55rem] leading-tight tracking-tight text-text-primary">
              {group.name}
            </h2>
            <p className="mt-1 text-xs text-text-muted">
              {group.transportType.toUpperCase()} · {t('mcp.manager.detail.instanceCount', { count: group.items.length })}
            </p>
          </div>
        </div>
        <div className="flex shrink-0 gap-2">
          <button
            type="button"
            className="inline-flex items-center gap-1.5 rounded-full border border-border-default/55 bg-bg-elevated px-3 py-1.5 text-sm font-semibold text-text-secondary hover:bg-bg-surface hover:text-text-primary"
            onClick={handleEdit}
          >
            <SIcon name="Pencil" size="w-4 h-4" />
            <span>{t('common.edit')}</span>
          </button>
          <button
            type="button"
            className="inline-flex items-center gap-1.5 rounded-full border border-border-default/55 bg-bg-elevated px-3 py-1.5 text-sm font-semibold text-danger/85 hover:border-danger/22 hover:bg-danger/8 hover:text-danger"
            onClick={handleDelete}
          >
            <SIcon name="Trash2" size="w-4 h-4" />
            <span>{t('common.delete')}</span>
          </button>
        </div>
      </div>

      <section className="rounded-2xl border border-border-default/40 bg-bg-surface p-4 shadow-sm">
        <div className="mb-3 flex items-center justify-between gap-3">
          <h3 className="text-[0.68rem] font-bold uppercase tracking-[0.13em] text-text-muted">
            {t('mcp.manager.detail.effectiveTitle')}
          </h3>
          <StatePill server={primaryServer} t={t} />
        </div>
        <div className="grid grid-cols-2 gap-3.5 max-[53.75rem]:grid-cols-1">
          <div>
            <span className="mb-1 block text-[0.65rem] font-bold uppercase tracking-widest text-text-muted">
              {t('mcp.manager.detail.typeLabel')}
            </span>
            <span className="break-all text-sm text-text-primary">{group.transportType}</span>
          </div>
          <div className="col-span-full">
            <span className="mb-1 block text-[0.65rem] font-bold uppercase tracking-widest text-text-muted">
              {commandOrUrl}
            </span>
            <span className="break-all font-mono text-xs text-text-primary">{group.transportLabel || '—'}</span>
          </div>
          {primaryServer.args?.length ? (
            <div className="col-span-full">
              <span className="mb-1 block text-[0.65rem] font-bold uppercase tracking-widest text-text-muted">
                {t('mcp.manager.detail.argsLabel')}
              </span>
              <span className="break-all font-mono text-xs text-text-primary">{primaryServer.args.join(' ')}</span>
            </div>
          ) : null}
          {primaryServer.timeout ? (
            <div>
              <span className="mb-1 block text-[0.65rem] font-bold uppercase tracking-widest text-text-muted">
                {t('mcp.manager.detail.timeoutLabel')}
              </span>
              <span className="text-sm text-text-primary">{primaryServer.timeout}ms</span>
            </div>
          ) : null}
          {primaryServer.cwd ? (
            <div className="col-span-full">
              <span className="mb-1 block text-[0.65rem] font-bold uppercase tracking-widest text-text-muted">
                {t('mcp.manager.detail.cwdLabel')}
              </span>
              <span className="break-all font-mono text-xs text-text-primary">{primaryServer.cwd}</span>
            </div>
          ) : null}
        </div>
      </section>

      <section className="rounded-2xl border border-border-default/40 bg-bg-surface p-4 shadow-sm">
        <h3 className="mb-3 text-[0.68rem] font-bold uppercase tracking-[0.13em] text-text-muted">
          {t('mcp.manager.detail.precedenceTitle')}
        </h3>
        <div className="flex flex-col gap-1.5">
          {precedenceItems.map((item) => {
            const active = item.effective !== false && !item.hidden_by
            return (
              <div
                key={`${item.platform}-${item.scope ?? 'global'}-${item.name}`}
                className={cn(
                  'flex items-center gap-3 rounded-xl border px-2.5 py-2',
                  active
                    ? 'border-success/24 bg-success/10'
                    : 'border-border-default/28 bg-bg-base/38',
                )}
              >
                <span className={cn('h-2 w-2 rounded-full', active ? 'bg-success' : 'bg-border-default/80')} />
                <div className="flex min-w-0 flex-1 flex-col gap-0.5">
                  <strong className="text-sm font-semibold capitalize text-text-primary">
                    {mcpScopeLabel(item.scope ?? item.platform, t)}
                  </strong>
                  <span className="overflow-hidden text-ellipsis whitespace-nowrap font-mono text-[0.68rem] text-text-muted">
                    {item.source_path ?? t('mcp.manager.detail.sourceUnavailable')}
                  </span>
                  {item.hidden_by ? (
                    <em className="overflow-hidden text-ellipsis whitespace-nowrap font-mono text-[0.68rem] not-italic text-text-muted">
                      {t('mcp.manager.detail.hiddenBy', { hiddenBy: item.hidden_by })}
                    </em>
                  ) : null}
                  {!item.hidden_by && item.approval_state ? (
                    <em className="overflow-hidden text-ellipsis whitespace-nowrap font-mono text-[0.68rem] not-italic text-text-muted">
                      {t('mcp.manager.detail.approvalState', { state: mcpApprovalLabel(item.approval_state, t) })}
                    </em>
                  ) : null}
                </div>
                <StatePill server={item} t={t} />
              </div>
            )
          })}
        </div>
      </section>

      {envEntries.length > 0 || headerEntries.length > 0 ? (
        <section className="rounded-2xl border border-border-default/40 bg-bg-surface p-4 shadow-sm">
          <h3 className="mb-3 text-[0.68rem] font-bold uppercase tracking-[0.13em] text-text-muted">
            {t('mcp.manager.detail.envHeadersTitle')}
          </h3>
          <div className="flex flex-col gap-1.5">
            {envEntries.map(([key, value]) => (
              <div
                key={`env-${key}`}
                className="flex items-center justify-between gap-3 rounded-xl border border-border-default/28 bg-bg-base/38 px-2.5 py-2"
              >
                <span className="font-mono text-xs font-bold text-text-primary">{key}</span>
                <span className="overflow-hidden text-ellipsis whitespace-nowrap font-mono text-xs text-text-muted">
                  {maskSecret(String(value))}
                </span>
              </div>
            ))}
            {headerEntries.map(([key, value]) => (
              <div
                key={`header-${key}`}
                className="flex items-center justify-between gap-3 rounded-xl border border-border-default/28 bg-bg-base/38 px-2.5 py-2"
              >
                <span className="font-mono text-xs font-bold text-text-primary">{key}</span>
                <span className="overflow-hidden text-ellipsis whitespace-nowrap font-mono text-xs text-text-muted">
                  {maskSecret(String(value))}
                </span>
              </div>
            ))}
          </div>
        </section>
      ) : null}

      <section className="rounded-2xl border border-border-default/40 bg-bg-surface p-4 shadow-sm">
        <h3 className="mb-3 text-[0.68rem] font-bold uppercase tracking-[0.13em] text-text-muted">
          {t('mcp.manager.detail.rawConfigTitle')}
        </h3>
        <pre className="overflow-x-auto whitespace-pre-wrap rounded-xl border border-border-default/28 bg-bg-base/42 p-3 font-mono text-xs leading-relaxed text-text-secondary">
          {buildRawConfigPreview(primaryServer)}
        </pre>
      </section>

      <section className="rounded-2xl border border-border-default/40 bg-bg-surface p-4 shadow-sm">
        <h3 className="mb-3 text-[0.68rem] font-bold uppercase tracking-[0.13em] text-text-muted">
          {t('mcp.manager.detail.instancesTitle')}
        </h3>
        <div className="flex flex-col gap-1.5">
          {group.items.map((item) => (
            <McpInstanceRow
              key={`${item.platform}-${item.scope ?? 'global'}-${item.name}-instance`}
              item={item}
              onToggle={onToggle}
              t={t}
            />
          ))}
        </div>
      </section>

      {diagnostics.length > 0 ? (
        <section className="rounded-2xl border border-border-default/40 bg-bg-surface p-4 shadow-sm">
          <h3 className="mb-3 text-[0.68rem] font-bold uppercase tracking-[0.13em] text-text-muted">
            {t('mcp.manager.detail.diagnosticsTitle')}
          </h3>
          <div className="flex flex-col gap-1.5">
            {diagnostics.map((diagnostic) => (
              <div
                key={`${diagnostic.source_path ?? 'diagnostic'}-${diagnostic.level}-${diagnostic.message}`}
                className="flex items-baseline gap-3 rounded-xl border border-border-default/28 bg-bg-base/38 px-2.5 py-2 text-sm text-text-secondary"
              >
                <span className="text-[0.64rem] font-extrabold uppercase tracking-wide text-accent-primary/90">
                  {diagnostic.level}
                </span>
                <span>{diagnostic.message}</span>
              </div>
            ))}
          </div>
        </section>
      ) : null}
    </div>
  )
}

export function McpDetailPanel({
  group,
  diagnostics = [],
  t,
  onEdit,
  onDelete,
  onToggle,
}: McpDetailPanelProps) {
  const primaryServer = pickPrimaryServer(group)

  if (!group) {
    return (
      <div className="flex h-full flex-col gap-4 overflow-y-auto p-6">
        <DetailEmpty
          icon="Server"
          title={t('mcp.manager.detail.emptyTitle')}
          subtitle={t('mcp.manager.detail.emptySubtitle')}
        />
      </div>
    )
  }

  if (!primaryServer) {
    return (
      <div className="flex h-full flex-col gap-4 overflow-y-auto p-6">
        <DetailEmpty icon="AlertCircle" title={t('mcp.manager.detail.noReadableInstance')} />
      </div>
    )
  }

  return (
    <McpDetailBody
      group={group}
      primaryServer={primaryServer}
      diagnostics={diagnostics}
      t={t}
      onEdit={onEdit}
      onDelete={onDelete}
      onToggle={onToggle}
    />
  )
}
