import { memo, useCallback } from 'react'
import { Link } from 'react-router'
import { SIcon } from '@/ui'
import { t } from '../locale'
import {
  geminiModuleClass,
  geminiTagClass,
  type GeminiHeroTag,
  type GeminiModuleCard,
  type GeminiQuickCard,
  type GeminiTerminalSnippet,
} from './geminiHomeModel'

export const HeroTag = memo(function HeroTag({ tag }: { tag: GeminiHeroTag }) {
  return (
    <span className={`inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-xs font-semibold ${geminiTagClass[tag.tone]}`}>
      <SIcon name={tag.icon} size="w-3.5 h-3.5" />
      {tag.label}
    </span>
  )
})

export const TerminalRow = memo(function TerminalRow({
  snippet,
  copied,
  onCopy,
}: {
  snippet: GeminiTerminalSnippet
  copied: boolean
  onCopy: (command: string) => void
}) {
  const handleCopy = useCallback(() => {
    onCopy(snippet.command)
  }, [onCopy, snippet.command])
  return (
    <button
      type="button"
      className="group flex w-full items-center gap-3 rounded-2xl border border-[color:var(--stage-border-soft)] bg-bg-base/30 p-3 text-left hover:border-[color:color-mix(in_srgb,var(--platform-gemini)_30%,transparent)]"
      aria-label={t('gemini.overview.terminal.copyCommand', { command: snippet.command })}
      onClick={handleCopy}
    >
      <span className="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg bg-[color:color-mix(in_srgb,var(--platform-gemini)_12%,transparent)] font-mono text-sm font-bold text-[color:var(--platform-gemini)]">
        $
      </span>
      <span className="min-w-0 flex-1">
        <span className="block text-xs text-[color:var(--stage-text-muted)]">{snippet.label}</span>
        <code className="mt-0.5 block truncate font-mono text-sm text-[color:var(--stage-text-primary)]">{snippet.command}</code>
      </span>
      <span className="inline-flex items-center gap-1.5 rounded-full border border-[color:var(--stage-chip-neutral-border)] bg-[var(--stage-chip-neutral-bg)] px-2 py-1 text-[0.68rem] font-semibold text-[color:var(--stage-text-secondary)]">
        <SIcon name="Copy" size="w-3.5 h-3.5" />
        {copied ? t('gemini.overview.terminal.copied') : t('gemini.overview.terminal.copy')}
      </span>
    </button>
  )
})

export const ModuleCard = memo(function ModuleCard({ item }: { item: GeminiModuleCard }) {
  return (
    <Link to={item.to} className={item.spotlight ? 'block h-full lg:col-span-2' : 'block h-full'}>
      <article
        className={`relative flex h-full min-h-60 flex-col rounded-[1.6rem] border bg-bg-surface p-5 ${geminiModuleClass[item.tone]}`}
      >
        <div className="flex items-center justify-between gap-3">
          <div className="flex h-11 w-11 items-center justify-center rounded-2xl border border-border-subtle bg-bg-elevated">
            <SIcon name={item.icon} size="w-5 h-5" />
          </div>
          <span className="rounded-full border border-border-subtle px-2.5 py-1 text-[0.68rem] font-semibold uppercase tracking-wide text-text-secondary">
            {item.badge}
          </span>
        </div>
        <div className="mt-5 flex-1">
          <h3 className="text-xl font-semibold tracking-tight text-[color:var(--stage-text-primary)]">{item.title}</h3>
          <p className="mt-2 text-sm leading-6 text-[color:var(--stage-text-secondary)]">{item.description}</p>
        </div>
        <div className="mt-5 flex items-center justify-between gap-3 rounded-2xl border border-[color:var(--stage-border-soft)] p-3 text-xs text-[color:var(--stage-text-muted)]">
          <span>{item.hint}</span>
          <strong>{item.status}</strong>
        </div>
      </article>
    </Link>
  )
})

export const QuickCard = memo(function QuickCard({ card }: { card: GeminiQuickCard }) {
  return (
    <article className="rounded-[1.6rem] border border-border-subtle bg-bg-surface p-5">
      <div className="flex items-start gap-3">
        <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl border border-border-subtle text-[color:var(--platform-gemini)]">
          <SIcon name={card.icon} size="w-4 h-4" />
        </div>
        <div>
          <p className="text-xs font-semibold uppercase tracking-wide text-[color:var(--stage-text-muted)]">{card.kicker}</p>
          <h3 className="mt-1 text-base font-semibold text-[color:var(--stage-text-primary)]">{card.title}</h3>
        </div>
      </div>
      <ul className="mt-4 space-y-3">
        {card.items.map((item) => (
          <li key={item} className="flex gap-3 text-sm leading-6 text-[color:var(--stage-text-secondary)]">
            <span className="mt-2.5 h-1.5 w-1.5 shrink-0 rounded-full bg-[color:var(--platform-gemini)]" />
            <span>{item}</span>
          </li>
        ))}
      </ul>
    </article>
  )
})
