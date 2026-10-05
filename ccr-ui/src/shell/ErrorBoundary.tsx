import { Component, type ErrorInfo, type ReactNode } from 'react'
import { logger } from '@/utils/logger'

interface ErrorBoundaryProps {
  children: ReactNode
  fallback?: ReactNode
}

interface ErrorBoundaryState {
  error: Error | null
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { error: null }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    logger.error('[ErrorBoundary] render failed', { error, componentStack: info.componentStack })
  }

  render(): ReactNode {
    if (!this.state.error) return this.props.children
    if (this.props.fallback) return this.props.fallback
    return (
      <div
        role="alert"
        className="m-6 rounded-2xl border border-danger/30 bg-danger/10 p-6 text-text-primary"
      >
        <h2 className="text-lg font-semibold">页面渲染失败</h2>
        <p className="mt-2 text-sm text-text-secondary">{this.state.error.message}</p>
      </div>
    )
  }
}
