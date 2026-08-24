import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { Link } from 'react-router'
import { PageHeader, PageShell, SIcon } from '@/ui'
import { copyText } from '@/utils/clipboard'
import { GeminiSubnav } from './GeminiSubnav'
import { HeroTag, ModuleCard, QuickCard, TerminalRow } from './home/GeminiHomeGrids'
import { GeminiUsageStrip } from './home/GeminiUsageStrip'
import {
  geminiConfigPreview,
  geminiHeroTags,
  geminiModuleCards,
  geminiQuickCards,
  geminiTerminalSnippets,
} from './home/geminiHomeModel'
import { t } from './locale'

export function GeminiCliView() {
  const tags = useMemo(() => geminiHeroTags(), [])
  const snippets = useMemo(() => geminiTerminalSnippets(), [])
  const preview = useMemo(() => geminiConfigPreview(), [])
  const modules = useMemo(() => geminiModuleCards(), [])
  const quick = useMemo(() => geminiQuickCards(), [])
  const [copiedCommand, setCopiedCommand] = useState<string | null>(null)
  const copyTimerRef = useRef<number | null>(null)

  useEffect(() => {
    return () => {
      if (copyTimerRef.current === null) return
      window.clearTimeout(copyTimerRef.current)
      copyTimerRef.current = null
    }
  }, [])

  const copyCommand = useCallback(async (command: string) => {
    if (!(await copyText(command))) return
    setCopiedCommand(command)
    if (copyTimerRef.current !== null) window.clearTimeout(copyTimerRef.current)
    copyTimerRef.current = window.setTimeout(() => {
      copyTimerRef.current = null
      setCopiedCommand((current) => (current === command ? null : current))
    }, 1600)
  }, [])

  return (
    <PageShell
      className="gemini-view bg-bg-elevated"
      header={
        <PageHeader
          title={t('gemini.overview.breadcrumb')}
          eyebrow={t('gemini.overview.hero.eyebrow')}
          description={t('gemini.overview.hero.description')}
          actions={
            <div className="flex flex-wrap gap-2">
              <Link
                to="/antigravity/mcp"
                className="inline-flex items-center rounded-lg bg-accent-primary px-3 py-2 text-sm text-[color:var(--color-accent-primary-contrast)]"
              >
                <SIcon name="Server" size="w-4 h-4" className="mr-2" />
                {t('gemini.overview.hero.primaryAction')}
              </Link>
              <Link
                to="/antigravity/slash-commands"
                className="inline-flex items-center rounded-lg border border-border-default px-3 py-2 text-sm"
              >
                <SIcon name="Command" size="w-4 h-4" className="mr-2" />
                {t('gemini.overview.hero.secondaryAction')}
              </Link>
              <Link to="/" className="inline-flex items-center rounded-lg px-3 py-2 text-sm text-text-secondary">
                <SIcon name="Home" size="w-4 h-4" className="mr-2" />
                {t('common.backToHome')}
              </Link>
            </div>
          }
        />
      }
      subnav={<GeminiSubnav />}
    >
      <div className="gemini-tag-row relative z-10 mt-2 flex flex-wrap gap-2.5">
        {tags.map((tag) => (
          <HeroTag key={tag.key} tag={tag} />
        ))}
      </div>

      <section className="mt-6">
        <article className="gemini-terminal-card relative overflow-hidden rounded-[2rem] border border-border-subtle bg-bg-surface p-5">
          <div className="flex items-start justify-between gap-4">
            <div>
              <p className="text-xs font-semibold uppercase tracking-wide text-[color:var(--stage-text-muted)]">
                {t('gemini.overview.terminal.eyebrow')}
              </p>
              <h2 className="mt-1 text-xl font-semibold tracking-tight text-[color:var(--stage-text-primary)]">
                {t('gemini.overview.terminal.title')}
              </h2>
            </div>
            <div className="flex gap-1.5 rounded-full border border-[color:var(--stage-border-soft)] bg-[var(--stage-surface-soft)] px-2 py-1.5">
              <span className="h-2 w-2 rounded-full bg-accent-danger" />
              <span className="h-2 w-2 rounded-full bg-accent-warning" />
              <span className="h-2 w-2 rounded-full bg-accent-success" />
            </div>
          </div>
          <div className="mt-5 space-y-2">
            {snippets.map((snippet) => (
              <TerminalRow
                key={snippet.command}
                snippet={snippet}
                copied={copiedCommand === snippet.command}
                onCopy={copyCommand}
              />
            ))}
          </div>
          <div className="mt-4 grid gap-2 sm:grid-cols-2">
            {preview.map((item) => (
              <div key={item.label} className="rounded-2xl border border-[color:var(--stage-border-soft)] bg-[var(--stage-surface-soft)] p-3">
                <span className="block text-xs font-semibold uppercase tracking-wide text-[color:var(--stage-text-muted)]">
                  {item.label}
                </span>
                <code className="mt-1 block truncate font-mono text-xs text-[color:var(--stage-text-primary)]">{item.value}</code>
              </div>
            ))}
          </div>
        </article>
      </section>

      <div className="mt-6">
        <GeminiUsageStrip platform="antigravity" />
      </div>

      <section className="mt-6 space-y-5" aria-label="Gemini modules">
        <div className="flex flex-col gap-3 md:flex-row md:items-end md:justify-between">
          <div>
            <p className="text-xs font-semibold uppercase tracking-wide text-[color:var(--stage-text-muted)]">
              {t('gemini.overview.modules.eyebrow')}
            </p>
            <h2 className="mt-1 text-2xl font-semibold tracking-tight text-[color:var(--stage-text-primary)]">
              {t('gemini.overview.modules.title')}
            </h2>
          </div>
          <p className="max-w-xl text-sm leading-6 text-[color:var(--stage-text-secondary)]">
            {t('gemini.overview.modules.subtitle')}
          </p>
        </div>
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
          {modules.map((item) => (
            <ModuleCard key={item.key} item={item} />
          ))}
        </div>
      </section>

      <section className="mt-6 grid grid-cols-1 gap-4 lg:grid-cols-3">
        {quick.map((card) => (
          <QuickCard key={card.key} card={card} />
        ))}
      </section>
    </PageShell>
  )
}
