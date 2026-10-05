import { lazy, Suspense, useCallback, useMemo } from 'react'
import { Link } from 'react-router'
import {
  commonCommands,
  coreModules,
  extensionModules,
  featureTags,
  heroChips,
  resources,
} from '@/features/claude/home/homeModel'
import {
  CommandRow,
  CoreModuleCard,
  ExtensionCard,
  ResourceLink,
  copyHomeCommand,
} from '@/features/claude/home/HomeGrids'
import { t } from '@/features/claude/locale'
import { PageHeader, PageShell, SIcon, Spinner } from '@/ui'

const UsageInsightPanel = lazy(() =>
  import('@/features/claude/observer/UsageInsightPanel').then((mod) => ({ default: mod.UsageInsightPanel })),
)

function SectionHeading({ icon, eyebrow, title }: { icon: string; eyebrow: string; title: string }) {
  return (
    <div className="flex items-center gap-3">
      <SIcon name={icon} size="w-5 h-5" className="text-accent-warning" />
      <div>
        <p className="text-xs font-medium text-[color:var(--stage-text-muted)]">{eyebrow}</p>
        <h2 className="text-xl font-semibold text-[color:var(--stage-text-primary)]">{title}</h2>
      </div>
      <div className="h-px flex-1 bg-border-subtle" />
    </div>
  )
}

/** Claude Code 平台首页。 */
export function ClaudeCodeView() {
  const chips = useMemo(() => heroChips(), [])
  const tags = useMemo(() => featureTags(), [])
  const cores = useMemo(() => coreModules(), [])
  const extensions = useMemo(() => extensionModules(), [])
  const commands = useMemo(() => commonCommands(), [])
  const links = useMemo(() => resources(), [])
  const copyCommand = useCallback((cmd: string) => {
    void copyHomeCommand(cmd)
  }, [])

  const header = (
    <PageHeader
      title={t('claudeCode.title')}
      eyebrow={t('claudeCode.hero.eyebrow')}
      description={t('claudeCode.subtitle')}
      actions={
        <div className="flex flex-wrap gap-2">
          <Link
            to="/claude-code/auth"
            className="inline-flex items-center rounded-lg bg-accent-primary px-3 py-2 text-sm text-[color:var(--color-accent-primary-contrast)]"
          >
            <SIcon name="KeyRound" size="w-4 h-4" className="mr-2" />
            {t('claudeCode.hero.primaryAction')}
          </Link>
          <Link
            to="/claude-code/profiles"
            className="inline-flex items-center rounded-lg border border-border-default px-3 py-2 text-sm text-text-primary"
          >
            <SIcon name="Settings" size="w-4 h-4" className="mr-2" />
            {t('claudeCode.hero.secondaryAction')}
          </Link>
        </div>
      }
    />
  )

  return (
    <PageShell className="claude-view bg-bg-elevated" header={header}>
      <div className="space-y-6">
        <div className="claude-console rounded-xl border border-border-subtle bg-bg-surface p-4 font-mono">
          <p className="claude-console__line m-0 text-sm text-text-primary">
            <span className="mr-2 text-text-muted">$</span> ccr current
          </p>
          <p className="mt-2 text-sm leading-6 text-text-secondary">{t('claudeCode.hero.consoleStatus')}</p>
          <div className="mt-3 flex flex-wrap gap-2">
            {chips.map((chip) => (
              <span
                key={chip}
                className="rounded-lg border border-border-subtle bg-bg-elevated px-2.5 py-1 text-[0.6875rem] font-medium text-text-secondary"
              >
                {chip}
              </span>
            ))}
          </div>
        </div>

        <section className="space-y-5" aria-label="Claude Code usage insight">
          <Suspense fallback={<Spinner size="lg" className="mx-auto text-accent-primary" />}>
            <UsageInsightPanel />
          </Suspense>
        </section>

        <section className="flex flex-wrap gap-3" aria-label="Claude Code capabilities">
          {tags.map((feature) => (
            <span
              key={feature.label}
              className={`flex items-center gap-2 rounded-lg border px-3 py-1.5 text-xs font-medium ${feature.className}`}
            >
              <SIcon name={feature.icon} size="w-3 h-3" />
              {feature.label}
            </span>
          ))}
        </section>

        <section className="space-y-5">
          <SectionHeading icon="Zap" eyebrow={t('claudeCode.modules.title')} title={t('claudeCode.modules.coreTitle')} />
          <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
            {cores.map((item) => (
              <CoreModuleCard key={item.to} item={item} />
            ))}
          </div>
        </section>

        <section className="space-y-5">
          <SectionHeading
            icon="Boxes"
            eyebrow={t('claudeCode.modules.extensionsEyebrow')}
            title={t('claudeCode.modules.extensionsTitle')}
          />
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
            {extensions.map((item) => (
              <ExtensionCard key={item.to} item={item} />
            ))}
          </div>
        </section>

        <section className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          <article className="rounded-2xl border border-border-subtle bg-bg-surface p-6">
            <h3 className="mb-2 flex items-center gap-2 text-lg font-semibold text-[color:var(--stage-text-primary)]">
              <SIcon name="Terminal" size="w-5 h-5" className="text-accent-warning" />
              {t('claudeCode.quickActions.commonCommands')}
            </h3>
            <p className="mb-4 text-sm text-[color:var(--stage-text-muted)]">
              {t('claudeCode.quickActions.commonCommandsDesc')}
            </p>
            <div className="space-y-3">
              {commands.map((item) => (
                <CommandRow key={item.cmd} item={item} onCopy={copyCommand} />
              ))}
            </div>
          </article>
          <div className="space-y-6">
            <article className="rounded-2xl border border-border-subtle bg-bg-surface p-6">
              <h3 className="mb-2 flex items-center gap-2 text-lg font-semibold text-[color:var(--stage-text-primary)]">
                <SIcon name="BookOpen" size="w-5 h-5" className="text-accent-secondary" />
                {t('claudeCode.quickActions.resources')}
              </h3>
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-3 lg:grid-cols-1 xl:grid-cols-3">
                {links.map((item) => (
                  <ResourceLink key={item.url} item={item} />
                ))}
              </div>
            </article>
            <article className="rounded-2xl border border-accent-warning/20 bg-accent-warning/10 p-4">
              <div className="flex gap-3">
                <SIcon name="Info" size="w-5 h-5" className="mt-0.5 shrink-0 text-accent-warning" />
                <div className="space-y-2">
                  <h4 className="text-sm font-semibold text-accent-warning">{t('claudeCode.tips.title')}</h4>
                  <ul className="list-inside list-disc space-y-1 text-xs leading-5 text-[color:var(--stage-text-secondary)]">
                    <li>{t('claudeCode.tips.tip1')}</li>
                    <li>{t('claudeCode.tips.tip2')}</li>
                  </ul>
                </div>
              </div>
            </article>
          </div>
        </section>
      </div>
    </PageShell>
  )
}
