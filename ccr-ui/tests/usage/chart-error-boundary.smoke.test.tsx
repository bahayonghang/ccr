import { render, screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { ChartErrorBoundary } from '@/features/usage/charts/ChartErrorBoundary'

vi.spyOn(console, 'error').mockImplementation(() => undefined)

function Boom(): ReactNode {
  throw new Error('chart boom')
}

describe('chart error boundary', () => {
  it('keeps sibling content available when a chart subtree throws', () => {
    render(
      <div>
        <ChartErrorBoundary>
          <Boom />
        </ChartErrorBoundary>
        <p>rest of page</p>
      </div>,
    )
    expect(screen.getByRole('alert')).toBeTruthy()
    expect(screen.getByText('rest of page')).toBeTruthy()
  })
})
