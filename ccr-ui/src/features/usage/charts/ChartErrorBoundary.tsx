import { Component, type ErrorInfo, type ReactNode } from 'react'
import { logger } from '@/utils/logger'

interface ChartErrorBoundaryProps {
  children: ReactNode
  fallback?: ReactNode
}

interface ChartErrorBoundaryState {
  error: Error | null
}

/** 图表级错误边界：单个图表失败不影响页面其余部分。 */
export class ChartErrorBoundary extends Component<
  ChartErrorBoundaryProps,
  ChartErrorBoundaryState
> {
  state: ChartErrorBoundaryState = { error: null }

  static getDerivedStateFromError(error: Error): ChartErrorBoundaryState {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    logger.error('[ChartErrorBoundary] chart render failed', {
      error,
      componentStack: info.componentStack,
    })
  }

  render(): ReactNode {
    if (!this.state.error) return this.props.children
    if (this.props.fallback) return this.props.fallback
    return (
      <div
        role="alert"
        className="flex min-h-40 items-center justify-center rounded-xl border border-danger/30 bg-danger/10 p-4 text-sm text-text-secondary"
      >
        图表渲染失败
      </div>
    )
  }
}
