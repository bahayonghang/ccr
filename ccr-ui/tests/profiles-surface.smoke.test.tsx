import { fireEvent, render, screen } from '@testing-library/react'
import type { ComponentProps, ReactElement } from 'react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { claudeProfilePresentation } from '@/configs/profilePresentation'
import { ProfilesSurface } from '@/features/platform/profiles/ProfilesSurface'
import { useProfilesQuickSwitchStore, useProfilesViewStore } from '@/features/profiles/stores'
import { claudeDisplayRecords } from './fixtures/profiles'

const renderRouted = (ui: ReactElement) => {
  const router = createMemoryRouter([{ path: '*', element: ui }], { initialEntries: ['/'] })
  return render(<RouterProvider router={router} />)
}

const renderClaudeSurface = (overrides: Partial<ComponentProps<typeof ProfilesSurface>> = {}) =>
  renderRouted(
    <ProfilesSurface
      platformKey="surface-claude"
      presentation={claudeProfilePresentation}
      records={claudeDisplayRecords}
      current="claude-current"
      environmentLabel="本机"
      environmentOk
      canOff
      commandPalette
      onAdd={vi.fn()}
      onEdit={vi.fn()}
      onApply={vi.fn()}
      onOff={vi.fn(async () => undefined)}
      onReload={vi.fn()}
      {...overrides}
    />,
  )

const orderFollowing = (left: Element, right: Element) =>
  Boolean(left.compareDocumentPosition(right) & Node.DOCUMENT_POSITION_FOLLOWING)

describe('profiles surface', () => {
  beforeEach(() => {
    localStorage.clear()
    useProfilesViewStore.setState({ viewByPlatform: {} })
    useProfilesQuickSwitchStore.setState({
      pinnedByPlatform: {},
      recentByPlatform: {},
    })
  })

  it('renders the injected records without platform-name branches', () => {
    renderClaudeSurface()
    expect(screen.getByTestId('profiles-surface')).toBeTruthy()
    expect(screen.getAllByText('claude-current').length).toBeGreaterThan(0)
    expect(screen.getByTestId('profiles-stat-total').textContent).toBe(String(claudeDisplayRecords.length))
  })

  it('keeps skeleton order and hides Off when canOff is false', () => {
    const { unmount } = renderClaudeSurface({ canOff: true })
    const header = screen.getByTestId('profiles-page-header')
    const off = screen.getByTestId('profiles-off-banner')
    const stats = screen.getByTestId('profiles-stat-strip')
    const rail = screen.getByTestId('profiles-quick-rail')
    const toolbar = screen.getByTestId('profiles-toolbar')
    const list = screen.getByTestId('profiles-list')
    expect(orderFollowing(header, off)).toBe(true)
    expect(orderFollowing(off, stats)).toBe(true)
    expect(orderFollowing(stats, rail)).toBe(true)
    expect(orderFollowing(rail, toolbar)).toBe(true)
    expect(orderFollowing(toolbar, list)).toBe(true)
    expect(screen.queryByRole('menuitem', { name: /off|退出/i })).toBeNull()
    unmount()

    renderClaudeSurface({ canOff: false })
    expect(screen.queryByTestId('profiles-off-banner')).toBeNull()
  })

  it('derives stats from the full record set', () => {
    renderClaudeSurface()
    const vendorCount = new Set(
      claudeDisplayRecords.map((record) => record.vendorKey).filter(Boolean),
    ).size
    expect(screen.getByTestId('profiles-stat-vendors').getAttribute('data-vendor-count')).toBe(
      String(vendorCount),
    )
    expect(screen.getByTestId('profiles-stat-running').textContent).toContain('claude-current')
  })

  it('filters by search text covering name, description, url, and tags', () => {
    renderClaudeSurface()
    const search = screen.getByRole('textbox')
    fireEvent.input(search, { target: { value: 'sandbox' } })
    const list = screen.getByTestId('profiles-list')
    expect(list.textContent).toContain('claude-disabled')
    expect(list.querySelector('[data-name="claude-current"]')).toBeNull()
  })

  it('clears no-result empty state back to the full list', () => {
    renderClaudeSurface()
    fireEvent.input(screen.getByRole('textbox'), { target: { value: 'zzz-no-match' } })
    expect(screen.getByTestId('profiles-empty')).toBeTruthy()
    fireEvent.click(screen.getByTestId('profiles-clear-filters'))
    expect(screen.queryByTestId('profiles-empty')).toBeNull()
    expect(screen.getByTestId('profiles-list').querySelector('[data-name="claude-current"]')).toBeTruthy()
  })

  it('shows a distinct empty state without a clear action when there are no records', () => {
    renderClaudeSurface({ records: [], current: null })
    expect(screen.getByTestId('profiles-empty')).toBeTruthy()
    expect(screen.queryByTestId('profiles-clear-filters')).toBeNull()
  })

  it('switches card and table views for the same records', () => {
    renderClaudeSurface()
    expect(screen.getByTestId('profiles-card-grid')).toBeTruthy()
    fireEvent.click(screen.getByTitle('profilesSurface.toolbar.viewTable'))
    expect(screen.getByTestId('profiles-table')).toBeTruthy()
    expect(screen.queryByTestId('profiles-card-grid')).toBeNull()
    expect(screen.getAllByText('claude-current').length).toBeGreaterThan(0)
  })

  it('keeps table overflow on the table container at 900×800', () => {
    renderClaudeSurface()
    fireEvent.click(screen.getByTitle('profilesSurface.toolbar.viewTable'))
    const scroll = screen.getByTestId('profiles-table-scroll')
    Object.defineProperty(scroll, 'clientWidth', { configurable: true, value: 900 })
    Object.defineProperty(scroll, 'scrollWidth', { configurable: true, value: 1024 })
    Object.defineProperty(document.body, 'clientWidth', { configurable: true, value: 900 })
    Object.defineProperty(document.body, 'scrollWidth', { configurable: true, value: 900 })
    expect(scroll.scrollWidth > scroll.clientWidth).toBe(true)
    expect(document.body.scrollWidth <= document.body.clientWidth).toBe(true)
  })

  it('opens the inspector and drops the card grid to two columns', () => {
    renderClaudeSurface()
    fireEvent.click(screen.getByTestId('profiles-inspector-toggle'))
    expect(screen.getByTestId('profiles-inspector')).toBeTruthy()
    expect(screen.getByTestId('profiles-card-grid').className).toContain('cp-card-grid--inspector')
  })

  it('opens the command palette with an __off action and keeps Off out of the header', () => {
    renderClaudeSurface({ commandPalette: true, canOff: true })
    fireEvent.click(screen.getByTestId('profiles-open-palette'))
    expect(document.querySelector('.cp-palette')?.textContent).toContain('profilesSurface.offAction')
    expect(screen.queryByRole('menuitem', { name: /off|退出/i })).toBeNull()
  })
})
