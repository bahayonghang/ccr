import type { ReactNode } from 'react'
import { AsyncStatePanel, PageHeader, PageShell, type AsyncPanelState } from '@/ui'

export interface SurfacePageProps {
  title: string
  description?: string
  actions?: ReactNode
  subnav?: ReactNode
  state?: AsyncPanelState | null
  stateTitle?: string
  stateDescription?: string
  onRetry?: () => void
  children?: ReactNode
}

export const surfaceStateOf = (input: {
  unavailable: boolean
  loading: boolean
  error: string | null
}): AsyncPanelState | null => {
  if (input.unavailable) return 'runtime-unavailable'
  if (input.loading) return 'loading'
  if (input.error) return 'error'
  return null
}

/** 功能面页壳：header + 异步态 + 内容。不含平台名分支。 */
export function SurfacePage({
  title,
  description,
  actions,
  subnav,
  state,
  stateTitle,
  stateDescription,
  onRetry,
  children,
}: SurfacePageProps) {
  const showPanel = state === 'loading' || state === 'error' || state === 'empty' || state === 'runtime-unavailable'

  return (
    <PageShell
      header={
        <PageHeader title={title} description={description} actions={actions} />
      }
      subnav={subnav}
    >
      {showPanel && state ? (
        <AsyncStatePanel
          state={state}
          title={stateTitle ?? title}
          description={stateDescription}
          actionLabel={state === 'error' ? 'Retry' : undefined}
          onAction={onRetry}
        />
      ) : (
        children
      )}
    </PageShell>
  )
}
