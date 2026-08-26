import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { describe, expect, it } from 'vitest'
import { DashboardNextActions } from '@/features/usage/dashboard/DashboardNextActions'
import type { DashboardAction } from '@/views/dashboard/dashboardPresentation'

const action = (overrides: Partial<DashboardAction> & Pick<DashboardAction, 'id'>): DashboardAction => ({
  titleKey: 'dashboard.actions.commandRunnerTitle',
  descKey: 'dashboard.actions.commandRunnerDesc',
  path: '/commands',
  icon: 'Terminal',
  tone: 'command',
  priority: 1,
  ...overrides,
})

describe('dashboard next actions', () => {
  it('marks only the first action as the emphasized primary row', () => {
    render(
      <MemoryRouter>
        <DashboardNextActions
          actions={[
            action({ id: 'run', path: '/commands' }),
            action({
              id: 'sync',
              path: '/sync',
              titleKey: 'dashboard.actions.cloudSyncTitle',
              descKey: 'dashboard.actions.cloudSyncDesc',
              icon: 'Cloud',
              tone: 'sync',
              priority: 2,
            }),
          ]}
        />
      </MemoryRouter>,
    )

    const rows = [...document.querySelectorAll('.dashboard-action')]
    expect(rows).toHaveLength(2)
    expect(rows[0]?.className).toContain('dashboard-action--primary')
    expect(rows[1]?.className).not.toContain('dashboard-action--primary')
    expect(screen.getByRole('link', { name: /命令中心|Commands/ }).getAttribute('href')).toBe(
      '/commands',
    )
  })

  it('shows a readable empty state when there are no actions', () => {
    const { container } = render(
      <MemoryRouter>
        <DashboardNextActions actions={[]} />
      </MemoryRouter>,
    )

    expect(container.querySelector('.dashboard-actions__empty')).toBeTruthy()
    expect(container.querySelector('.dashboard-action')).toBeNull()
    expect(container.textContent).toMatch(/暂无下一步|No next actions/)
  })
})
