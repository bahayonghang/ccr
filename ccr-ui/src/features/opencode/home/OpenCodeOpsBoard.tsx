import { ActionLink } from './OpenCodeHomeCards'

interface OpenCodeOpsBoardProps {
  configPath: string
  tuiPath: string
  defaultAgent: string
  serverLabel: string
  webLabel: string
  shareLabel: string
  configWarn: boolean
  nextLabel: string
  statusLabel: string
  providerCount: number
  mcpCount: number
  themeLabel: string
}

export function OpenCodeOpsBoard({
  configPath,
  tuiPath,
  defaultAgent,
  serverLabel,
  webLabel,
  shareLabel,
  configWarn,
  nextLabel,
  statusLabel,
  providerCount,
  mcpCount,
  themeLabel,
}: OpenCodeOpsBoardProps) {
  return (
    <div className="mb-4 grid gap-4 xl:grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)_340px]">
      <section className="rounded-[1.75rem] border border-border-default/55 bg-bg-base p-4">
        <div className="mt-2 flex flex-wrap gap-2">
          <PathChip label="config" value={configPath} />
          <PathChip label="tui" value={tuiPath} />
          <PathChip label="default agent" value={defaultAgent} />
        </div>
      </section>
      <section className="rounded-[1.75rem] border border-border-default/55 bg-bg-base p-4" aria-label="OpenCode live metrics">
        <div className="mt-2 flex flex-wrap gap-2">
          <RuntimeChip label="serve" value={serverLabel} warn={configWarn} />
          <RuntimeChip label="web" value={webLabel} />
          <RuntimeChip label="share" value={shareLabel} />
        </div>
      </section>
      <aside className="rounded-[1.75rem] border border-border-default/55 bg-bg-base p-4">
        <p className="text-[11px] font-semibold uppercase tracking-wide text-text-muted">{nextLabel}</p>
        <p className="mb-3 text-sm text-text-secondary">{statusLabel}</p>
        <div className="flex flex-col gap-2">
          <ActionLink href="/opencode/providers" label="Provider matrix" detail={`${providerCount} configured`} />
          <ActionLink href="/opencode/mcp" label="MCP wiring" detail={`${mcpCount} servers`} />
          <ActionLink href="/opencode/settings" label="Runtime settings" detail={themeLabel} />
        </div>
      </aside>
    </div>
  )
}

function PathChip({ label, value }: { label: string; value: string }) {
  return (
    <span className="inline-flex max-w-full items-center gap-2 rounded-full border border-border-default/55 bg-bg-base px-3 py-1.5 text-xs text-text-secondary">
      {label}
      <strong className="truncate font-mono text-text-primary">{value}</strong>
    </span>
  )
}

function RuntimeChip({ label, value, warn = false }: { label: string; value: string; warn?: boolean }) {
  return (
    <span className="inline-flex items-center gap-2 rounded-full border border-border-default/55 px-3 py-1.5 text-xs">
      <span className={warn ? 'h-1.5 w-1.5 rounded-full bg-accent-warning' : 'h-1.5 w-1.5 rounded-full bg-accent-success'} />
      <span>{label}</span>
      <strong>{value}</strong>
    </span>
  )
}
