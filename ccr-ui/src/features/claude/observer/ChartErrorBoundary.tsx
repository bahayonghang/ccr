import { Component, type ErrorInfo, type ReactNode } from 'react'
import { ChartPreparingState } from '@/features/claude/observer/ChartPreparingState'
import { logger } from '@/utils/logger'

interface ChartErrorBoundaryProps {
  children: ReactNode
  label?: string
}

interface ChartErrorBoundaryState {
  degraded: boolean
  retries: number
}

const MAX_RETRIES = 2

/** ApexCharts 渲染错误就近接住，有限次重挂后降级为准备态。 */
export class ChartErrorBoundary extends Component<ChartErrorBoundaryProps, ChartErrorBoundaryState> {
  state: ChartErrorBoundaryState = { degraded: false, retries: 0 }
  private retryFrame: number | null = null

  static getDerivedStateFromError(): Pick<ChartErrorBoundaryState, 'degraded'> {
    return { degraded: true }
  }

  override componentDidCatch(error: Error, info: ErrorInfo): void {
    logger.warn('[claudeObserver] chart render error contained', { error, componentStack: info.componentStack })
    if (this.state.retries >= MAX_RETRIES) return
    this.retryFrame = requestAnimationFrame(() => {
      this.retryFrame = null
      this.setState((current) => ({ degraded: false, retries: current.retries + 1 }))
    })
  }

  override componentWillUnmount(): void {
    if (this.retryFrame === null) return
    cancelAnimationFrame(this.retryFrame)
    this.retryFrame = null
  }

  override render(): ReactNode {
    if (this.state.degraded) {
      return <ChartPreparingState label={this.props.label} />
    }
    return this.props.children
  }
}
