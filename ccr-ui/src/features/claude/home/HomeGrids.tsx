import { memo, useCallback } from 'react'
import { Link } from 'react-router'
import { SIcon } from '@/ui'
import { copyText } from '@/utils/clipboard'

export interface HomeModule {
  to: string
  icon: string
  title: string
  desc: string
  badge: string
  cardClass?: string
  iconClass?: string
}

export interface HomeCommand {
  label: string
  cmd: string
}

export interface HomeResource {
  label: string
  url: string
  icon: string
}

export const CoreModuleCard = memo(function CoreModuleCard({ item }: { item: HomeModule }) {
  return (
    <Link to={item.to} className="group h-full">
      <article
        className={`relative flex h-full min-h-48 flex-col justify-between gap-5 overflow-hidden rounded-2xl border bg-bg-surface p-5 transition-transform duration-300 group-hover:-translate-y-1 ${item.cardClass ?? ''}`}
      >
        <SIcon
          name={item.icon}
          size="w-20 h-20 lg:w-24 lg:h-24"
          className="absolute right-3 bottom-3 rotate-12 text-text-disabled transition-colors group-hover:text-text-ghost"
        />
        <div className="flex h-12 w-12 items-center justify-center rounded-xl border border-[color:var(--stage-border-soft)] bg-[var(--stage-surface-soft)] text-accent-warning">
          <SIcon name={item.icon} size="w-6 h-6" />
        </div>
        <div className="relative z-10">
          <div className="mb-2 flex flex-wrap items-center gap-2">
            <h3 className="text-xl font-bold text-[color:var(--stage-text-primary)]">{item.title}</h3>
            <span className="rounded border border-accent-warning/25 bg-accent-warning/10 px-2 py-0.5 text-[0.625rem] font-bold tracking-wide text-accent-warning uppercase">
              {item.badge}
            </span>
          </div>
          <p className="text-sm leading-6 text-[color:var(--stage-text-secondary)]">{item.desc}</p>
        </div>
      </article>
    </Link>
  )
})

export const ExtensionCard = memo(function ExtensionCard({ item }: { item: HomeModule }) {
  return (
    <Link to={item.to} className="group">
      <article className="flex h-full min-h-40 flex-col justify-between gap-4 rounded-2xl border border-border-subtle bg-bg-surface p-4">
        <div className="flex items-center justify-between">
          <div className={`flex h-10 w-10 items-center justify-center rounded-lg transition-transform group-hover:scale-105 ${item.iconClass ?? ''}`}>
            <SIcon name={item.icon} size="w-5 h-5" />
          </div>
          <span className="rounded border border-[color:var(--stage-chip-neutral-border)] bg-[var(--stage-chip-neutral-bg)] px-2 py-1 text-[0.625rem] font-semibold tracking-wide text-[color:var(--stage-text-muted)] uppercase">
            {item.badge}
          </span>
        </div>
        <div>
          <h3 className="mb-1 font-bold text-[color:var(--stage-text-primary)] transition-colors group-hover:text-accent-warning">
            {item.title}
          </h3>
          <p className="text-xs leading-5 text-[color:var(--stage-text-muted)]">{item.desc}</p>
        </div>
      </article>
    </Link>
  )
})

export const CommandRow = memo(function CommandRow({
  item,
  onCopy,
}: {
  item: HomeCommand
  onCopy: (cmd: string) => void
}) {
  const handleClick = useCallback(() => {
    onCopy(item.cmd)
  }, [item.cmd, onCopy])
  return (
    <button
      type="button"
      className="group flex w-full cursor-copy items-center justify-between gap-3 rounded-xl border border-[color:var(--stage-border-soft)] bg-[var(--stage-surface-soft)] p-3 text-left transition-colors hover:border-accent-warning/25"
      onClick={handleClick}
    >
      <span className="text-sm font-medium text-[color:var(--stage-text-secondary)]">{item.label}</span>
      <span className="flex shrink-0 items-center gap-2">
        <code className="rounded border border-[color:var(--stage-chip-neutral-border)] bg-[var(--stage-chip-neutral-bg)] px-2 py-1 font-mono text-xs text-[color:var(--stage-text-muted)] transition-colors group-hover:text-accent-warning">
          {item.cmd}
        </code>
        <SIcon name="Copy" size="w-3 h-3" className="text-[color:var(--stage-text-muted)] opacity-0 transition-opacity group-hover:opacity-100" />
      </span>
    </button>
  )
})

export const ResourceLink = memo(function ResourceLink({ item }: { item: HomeResource }) {
  return (
    <a
      href={item.url}
      target="_blank"
      rel="noreferrer"
      className="group flex items-center gap-3 rounded-xl border border-[color:var(--stage-border-soft)] bg-[var(--stage-surface-soft)] p-3 transition-colors hover:border-accent-secondary/25"
    >
      <SIcon name={item.icon} size="w-4 h-4" className="text-[color:var(--stage-text-muted)] transition-colors group-hover:text-accent-secondary" />
      <span className="text-sm font-medium text-[color:var(--stage-text-secondary)]">{item.label}</span>
      <SIcon name="ExternalLink" size="w-3 h-3" className="ml-auto text-[color:var(--stage-text-muted)] opacity-0 group-hover:opacity-100" />
    </a>
  )
})

export async function copyHomeCommand(cmd: string): Promise<void> {
  await copyText(cmd)
}
