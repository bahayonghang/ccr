import type { ReactNode } from 'react'
import { Link } from 'react-router'
import { PageHeader, PageShell, SIcon, buttonClass } from '@/ui'

interface OpenCodePageShellProps {
  title: string
  description: string
  icon?: string
  tone?: 'lime' | 'violet' | 'cyan' | 'amber' | 'emerald'
  backTo?: string
  backLabel?: string
  badge?: string
  actions?: ReactNode
  meta?: ReactNode
  children?: ReactNode
}

/** OpenCode 子页壳。tone 仅保留调用方契约，不写字面色。 */
export function OpenCodePageShell({
  title,
  description,
  icon = 'TerminalSquare',
  tone = 'lime',
  backTo = '/opencode',
  backLabel = 'OpenCode',
  badge = '',
  actions,
  meta,
  children,
}: OpenCodePageShellProps) {
  return (
    <PageShell
      header={
        <div className="flex flex-col gap-4" data-tone={tone}>
          <Link to={backTo} className="inline-flex w-fit">
            <span className={buttonClass({ variant: 'ghost' })}>
              <SIcon name="ChevronLeft" size="w-4 h-4" />
              {backLabel}
            </span>
          </Link>
          <PageHeader
            title={title}
            description={description}
            eyebrow="OpenCode operator surface"
            eyebrowLang="en"
            leading={
              <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl border border-border-subtle bg-bg-surface text-text-secondary">
                <SIcon name={icon} size="w-5 h-5" />
              </div>
            }
            status={
              badge || meta ? (
                <div className="flex items-center gap-2">
                  {badge ? (
                    <span className="rounded-md border border-border-default px-2 py-0.5 text-xs">{badge}</span>
                  ) : null}
                  {meta}
                </div>
              ) : undefined
            }
            actions={actions}
          />
        </div>
      }
    >
      {children}
    </PageShell>
  )
}
