import { useParams, useMatches } from 'react-router'
import { useRouteHandle } from '../routeHandle'

/** 阶段 5 之前的页面占位：渲染路由 id 与 handle，保证 75 条路径无白屏。 */
export function RoutePlaceholder() {
  const params = useParams()
  const matches = useMatches()
  const handle = useRouteHandle()
  const leaf = matches[matches.length - 1]
  const routeId = leaf?.id ?? leaf?.pathname ?? 'unknown'

  return (
    <section
      data-testid="route-placeholder"
      data-route-id={routeId}
      className="rounded-2xl border border-border-default/60 bg-bg-surface p-6 text-text-primary"
    >
      <p className="text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">
        Route placeholder
      </p>
      <h1 className="mt-2 text-2xl font-semibold tracking-tight">{routeId}</h1>
      <pre
        data-testid="route-handle"
        className="mt-4 overflow-auto rounded-xl bg-bg-elevated p-3 text-xs text-text-secondary"
      >
        {JSON.stringify({ id: routeId, params, handle }, null, 2)}
      </pre>
    </section>
  )
}
