import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { fireEvent, render, screen, within } from '@testing-library/react'
import type { ComponentProps, ReactElement } from 'react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  ProfileCardGrid,
  ProfileEditorModal,
  ProfilesHeader,
  ProfilesOffBanner,
  ProfilesPageHeader,
} from '@/components/profiles'
import {
  claudeProfilePresentation,
  codexProfilePresentation,
} from '@/configs/profilePresentation'
import { ProfilesSurface } from '@/features/platform/profiles/ProfilesSurface'
import { useProfilesViewStore } from '@/features/profiles/stores'
import { claudeDisplayRecords, codexDisplayRecords } from '../fixtures/profiles'
import '@/ui/primitives.css'
import '@/components/profiles/profiles-shared.css'

const profilesSharedCss = readFileSync(
  join(process.cwd(), 'src/components/profiles/profiles-shared.css'),
  'utf8',
)

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

describe('profiles visual types', () => {
  beforeEach(() => {
    localStorage.clear()
    useProfilesViewStore.setState({ viewByPlatform: {} })
  })

  it('maps Claude fieldSlots to url / text / chip kinds', () => {
    const slots = claudeProfilePresentation.fieldSlots
    expect(slots[0].kind).toBe('url')
    expect(slots[1].kind).toBe('text')
    expect(slots[2].kind).toBe('chip')
    expect(slots[3].kind).toBe('chip')
    expect(slots[2].chip).toBe(true)
    expect(slots[3].chip).toBe(true)
  })

  it('renders Base URL with UrlText title on the current Claude card', () => {
    const emptyModel = {
      ...claudeDisplayRecords[0],
      name: 'claude-empty-model',
      current: false,
      slots: [
        claudeDisplayRecords[0].slots[0],
        '',
        claudeDisplayRecords[0].slots[2],
        claudeDisplayRecords[0].slots[3],
      ] as const,
    }
    render(
      <ProfileCardGrid
        records={[claudeDisplayRecords[0], emptyModel]}
        presentation={claudeProfilePresentation}
        inspectorOpen={false}
        onSelect={vi.fn()}
        onEdit={vi.fn()}
        onApply={vi.fn()}
      />,
    )
    const card = screen.getByTestId('profiles-card-grid').querySelector('[data-name="claude-current"]')
    expect(card).toBeTruthy()
    const urlNode = card!.querySelector('.ui-url-text')
    expect(urlNode?.getAttribute('title')).toBe('https://api.anthropic.com/v1')
    const fieldBadges = card!.querySelectorAll('.cp-card__field .ui-badge--static')
    expect(fieldBadges).toHaveLength(2)
    expect(card!.textContent).toContain('api_key')
    expect(card!.textContent).toContain('Anthropic')
    expect(card!.textContent).toContain('claude-sonnet-4-6')

    const emptyCard = screen
      .getByTestId('profiles-card-grid')
      .querySelector('[data-name="claude-empty-model"]')
    const modelValue = emptyCard?.querySelectorAll('.cp-card__field dd')[1]
    expect(modelValue?.querySelector('.ui-badge')).toBeNull()
    expect(modelValue?.textContent?.trim()).toMatch(/^(—|-|profilesSurface\.placeholder)$/)
  })

  it('styles Off banner container and warning action separately', () => {
    render(<ProfilesOffBanner canOff currentName="claude-current" onOff={vi.fn(async () => undefined)} />)
    const banner = screen.getByTestId('profiles-off-banner')
    expect(profilesSharedCss).toMatch(/\.cp-off-banner[\s\S]*background:\s*var\(--color-warning-tint\)/)
    expect(profilesSharedCss).toMatch(/\.cp-off-banner[\s\S]*border:\s*1px solid var\(--color-warning\)/)
    const action = within(banner).getByRole('button')
    expect(action.className).toContain('ui-btn--warning')
    expect(action.className).not.toContain('cp-btn')
  })

  it('uses primary and ghost header buttons in document order', () => {
    render(
      <ProfilesPageHeader
        presentation={claudeProfilePresentation}
        environmentLabel="local"
        environmentOk
        loading={false}
        onAdd={vi.fn()}
        onReload={vi.fn()}
        onExport={vi.fn()}
        onEditSource={vi.fn()}
      />,
    )
    const actions = screen.getByTestId('profiles-page-header').querySelector('.cp-page-header__actions')
    const buttons = Array.from(actions!.querySelectorAll('.ui-btn')) as HTMLElement[]
    expect(buttons).toHaveLength(4)
    expect(buttons[0].className).toContain('ui-btn--ghost')
    expect(buttons[1].className).toContain('ui-btn--ghost')
    expect(buttons[2].className).toContain('ui-btn--ghost')
    expect(buttons[3].className).toContain('ui-btn--primary')
  })

  it('renders running status badge and record badges without pointer cursor', () => {
    render(
      <ProfileCardGrid
        records={codexDisplayRecords.slice(0, 1)}
        presentation={codexProfilePresentation}
        inspectorOpen={false}
        onSelect={vi.fn()}
        onEdit={vi.fn()}
        onApply={vi.fn()}
      />,
    )
    const card = screen.getByTestId('profiles-card-grid').querySelector('[data-name="codex-current"]')
    const statusBadge = card!.querySelector('[data-testid="profile-row-status-badge"]') as HTMLElement
    expect(statusBadge.className).toContain('ui-badge--accent')
    expect(window.getComputedStyle(statusBadge).cursor).not.toBe('pointer')
    const recordBadges = card!.querySelectorAll('[data-testid="profile-record-badge"]')
    expect(recordBadges.length).toBeGreaterThan(0)
    recordBadges.forEach((badge) => {
      expect(window.getComputedStyle(badge as HTMLElement).cursor).not.toBe('pointer')
    })
    const tagBadges = card!.querySelectorAll('.cp-card__tags .ui-badge--static')
    expect(tagBadges.length).toBeGreaterThan(0)
    tagBadges.forEach((badge) => {
      expect(window.getComputedStyle(badge as HTMLElement).cursor).not.toBe('pointer')
    })
  })

  it('keeps QuickRail switch chips as cp-chip--switch', () => {
    const source = readFileSync(join(process.cwd(), 'src/components/profiles/ProfilesQuickRail.tsx'), 'utf8')
    expect(source).toContain('cp-chip--switch')
  })

  it('uses accent-soft Apply on running card and ghost when idle', () => {
    render(
      <ProfileCardGrid
        records={claudeDisplayRecords.slice(0, 2)}
        presentation={claudeProfilePresentation}
        inspectorOpen={false}
        onSelect={vi.fn()}
        onEdit={vi.fn()}
        onApply={vi.fn()}
      />,
    )
    const running = screen
      .getByTestId('profiles-card-grid')
      .querySelector('[data-name="claude-current"]')!
    const idle = screen.getByTestId('profiles-card-grid').querySelector('[data-name="claude-disabled"]')!
    const runningApply = within(running as HTMLElement).getAllByRole('button').find((btn) =>
      btn.className.includes('ui-btn--accent-soft'),
    )
    const idleApply = within(idle as HTMLElement).getAllByRole('button').find((btn) =>
      btn.className.includes('ui-btn--ghost'),
    )
    expect(runningApply).toBeTruthy()
    expect(idleApply).toBeTruthy()
  })

  it('keeps table at six columns without a fourth field data column', () => {
    renderClaudeSurface()
    fireEvent.click(screen.getByTitle('profilesSurface.toolbar.viewTable'))
    const table = screen.getByTestId('profiles-table')
    expect(table.querySelector('.cp-table__head')!.children).toHaveLength(6)
    const row = table.querySelector('[data-name="claude-current"]')!
    expect(row.children).toHaveLength(6)
    expect(row.children[1].querySelector('.ui-url-text')).toBeTruthy()
    expect(row.children[3].querySelector('.ui-badge--static')).toBeTruthy()
    expect(row.textContent).toContain('api_key')
    expect(row.textContent).not.toContain('Anthropic')
  })

  it('migrates ProfilesHeader actions to ui-btn without cp-btn', () => {
    const source = readFileSync(join(process.cwd(), 'src/components/profiles/ProfilesHeader.tsx'), 'utf8')
    expect(source).not.toMatch(/cp-btn/)
    renderRouted(
      <ProfilesHeader
        icon="Cpu"
        backTo="/codex"
        labels={{
          title: 'Profiles',
          subtitle: 'Manage',
          back: 'Back',
          reload: 'Reload',
          export: 'Export',
          add: 'Add',
        }}
        onAdd={vi.fn()}
        onExport={vi.fn()}
        onReload={vi.fn()}
        onOpenPalette={vi.fn()}
        onEditSource={vi.fn()}
      />,
    )
    const buttons = document.querySelectorAll('.cp-header .ui-btn')
    expect(buttons.length).toBeGreaterThan(0)
  })

  it('uses primary save-and-apply and ghost cancel in editor footer', () => {
    const adapter = {
      createEmpty: () => ({ name: '' }),
      fromRecord: (record: { name: string }) => record,
      sections: [],
      validate: () => [],
      submit: async () => ({ status: 'ok' as const }),
    }
    render(
      <ProfileEditorModal
        open
        adapter={adapter}
        presentation={claudeProfilePresentation}
        target={null}
        originalName={null}
        existingNames={[]}
        onClose={vi.fn()}
      />,
    )
    expect(screen.getByTestId('profile-editor-cancel').className).toContain('ui-btn--ghost')
    expect(screen.getByTestId('profile-editor-save-apply').className).toContain('ui-btn--primary')
  })

  it('clips long URL fields inside card grid columns', () => {
    const longUrlRecord = {
      ...claudeDisplayRecords[0],
      name: 'claude-long-url',
      slots: [
        `https://relay.example.com/${'segment/'.repeat(24)}end`,
        claudeDisplayRecords[0].slots[1],
        claudeDisplayRecords[0].slots[2],
        claudeDisplayRecords[0].slots[3],
      ] as const,
    }
    render(
      <ProfileCardGrid
        records={[longUrlRecord]}
        presentation={claudeProfilePresentation}
        inspectorOpen={false}
        onSelect={vi.fn()}
        onEdit={vi.fn()}
        onApply={vi.fn()}
      />,
    )
    const dd = screen
      .getByTestId('profiles-card-grid')
      .querySelector('[data-name="claude-long-url"] .cp-card__field dd') as HTMLElement
    expect(profilesSharedCss).toMatch(/\.cp-card__field dd[\s\S]*overflow:\s*hidden/)
    expect(profilesSharedCss).toMatch(/\.cp-card__field[\s\S]*min-width:\s*0/)
    expect(dd.querySelector('.ui-url-text')).toBeTruthy()
  })
})
