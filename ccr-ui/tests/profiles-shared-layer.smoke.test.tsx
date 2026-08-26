import { readdirSync } from 'node:fs'
import { join } from 'node:path'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import type { ReactElement } from 'react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import {
  ProfileCardGrid,
  ProfileDiffRows,
  ProfilesCommandPalette,
  ProfilesHeader,
  ProfilesInspector,
  ProfilesQuickRail,
  ProfilesRawEditorPanel,
  ProfilesSection,
  ProfilesStatStrip,
  ProfilesToolbar,
} from '@/components/profiles'
import type { ProfilesQuickSwitch } from '@/composables/useProfilesQuickSwitch'
import type { ProfilesInspectorDescriptor } from '@/utils/profileDescriptors'
import type { ProfileDiffRow } from '@/utils/profileDiff'
import type { ProfilesInsightsResult } from '@/utils/profilesInsights'
import { claudeProfilePresentation } from '@/configs/profilePresentation'
import { claudeDisplayRecords } from './fixtures/profiles'

beforeAll(() => {
  class ResizeObserverStub {
    observe(): void {}
    unobserve(): void {}
    disconnect(): void {}
  }
  if (typeof globalThis.ResizeObserver === 'undefined') {
    globalThis.ResizeObserver = ResizeObserverStub as unknown as typeof ResizeObserver
  }
})

const renderRouted = (ui: ReactElement) => {
  const router = createMemoryRouter([{ path: '*', element: ui }], { initialEntries: ['/'] })
  return render(<RouterProvider router={router} />)
}

interface SampleProfile {
  name: string
  description?: string | null
  enabled?: boolean | null
  tags?: string[] | null
  model?: string | null
}

const sample: SampleProfile = {
  name: 'alpha',
  description: 'relay',
  enabled: true,
  tags: ['fast', 'prod'],
  model: 'sonnet',
}

const emptyInsights = (): ProfilesInsightsResult<SampleProfile, string, string> => ({
  providerBreakdown: [],
  authModeBreakdown: [],
  topTags: [],
  deprecatedAuthIssues: [],
  missingFieldIssues: [],
  duplicateRuntimeIssues: [],
  totalIssueCount: 0,
})

const inspectorDescriptor: ProfilesInspectorDescriptor<SampleProfile> = {
  editIcon: 'Pencil',
  useInsights: emptyInsights,
  activeFields: (profile) => [{ label: 'MODEL', value: profile.model ?? '—', variant: 'accent' }],
  diffFields: [{ key: 'model', label: 'MODEL', value: (profile) => profile.model ?? '' }],
  authModeLabel: (mode) => mode,
  isDeprecatedMode: () => false,
  missingMessage: (missing) => missing.join(','),
  runtimeSummary: (profile) => profile.name,
}

const quickSwitch = (overrides: Partial<ProfilesQuickSwitch> = {}): ProfilesQuickSwitch => ({
  pinned: ['alpha'],
  recent: ['alpha', 'beta'],
  recentNotPinned: ['beta'],
  stableTargets: ['alpha'],
  modifier: 'Ctrl',
  isPinned: (name) => name === 'alpha',
  canPin: true,
  pin: () => true,
  unpin: () => undefined,
  togglePin: () => undefined,
  recordUse: () => undefined,
  renamePinned: () => undefined,
  ...overrides,
})

describe('profiles shared layer (React)', () => {
  it('ships TSX modules in the shared profiles folder', () => {
    const dir = join(process.cwd(), 'src/components/profiles')
    const files = readdirSync(dir)
    expect(files).toEqual(expect.arrayContaining([
      'ProfileDiffRows.tsx',
      'ProfileCardGrid.tsx',
      'ProfilesCommandPalette.tsx',
      'ProfilesHeader.tsx',
      'ProfilesInspector.tsx',
      'ProfilesQuickRail.tsx',
      'ProfilesRawEditorPanel.tsx',
      'ProfilesSection.tsx',
      'ProfilesStatStrip.tsx',
      'ProfilesToolbar.tsx',
      'profile-editor-shell.css',
      'profiles-shared.css',
    ]))
  })

  it('maps ProfileDiffRows rows and placeholder', () => {
    const rows: ProfileDiffRow[] = [
      { key: 'model', label: 'MODEL', from: 'a', to: 'b', changed: true },
      { key: 'url', label: 'URL', from: null, to: null, changed: false },
    ]
    const { container } = render(<ProfileDiffRows rows={rows} placeholder="未设置" />)
    expect(container.querySelector('.cp-diff-row--changed')).toBeTruthy()
    expect(screen.getAllByText('未设置')).toHaveLength(2)
    expect(screen.getByText('a')).toBeTruthy()
    expect(screen.getByText('b')).toBeTruthy()
  })

  it('maps ProfileCardGrid records and action callbacks', () => {
    const onApply = vi.fn()
    const onEdit = vi.fn()
    const onSelect = vi.fn()
    render(
      <ProfileCardGrid
        records={claudeDisplayRecords.slice(0, 1)}
        presentation={claudeProfilePresentation}
        inspectorOpen={false}
        onSelect={onSelect}
        onEdit={onEdit}
        onApply={onApply}
      />,
    )
    expect(screen.getByTestId('profiles-card-grid')).toBeTruthy()
    expect(screen.getAllByText('claude-current').length).toBeGreaterThan(0)
    fireEvent.click(screen.getByRole('button', { name: /profilesSurface.edit|编辑|Edit/ }))
    expect(onEdit).toHaveBeenCalledWith('claude-current')
  })

  it('maps ProfilesSection children', () => {
    render(
      <ProfilesSection title="Anthropic" count={2}>
        <div>card</div>
      </ProfilesSection>,
    )
    expect(screen.getByText('Anthropic')).toBeTruthy()
    expect(screen.getByText('2')).toBeTruthy()
    expect(screen.getByText('card')).toBeTruthy()
  })

  it('maps ProfilesStatStrip four cards', () => {
    render(
      <ProfilesStatStrip
        current="alpha"
        stats={{
          total: 3,
          vendorCount: 2,
          tagCounts: { prod: 2 },
          authCounts: { api_key: 3 },
        }}
        labels={{
          total: 'Total',
          vendors: '2 vendors',
          running: 'Running',
          runningHint: 'hint-running',
          notApplied: 'None',
          tags: 'Tags',
          auth: 'Auth',
        }}
      />,
    )
    expect(screen.getByTestId('profiles-stat-total').textContent).toBe('3')
    expect(screen.getByTestId('profiles-stat-vendors').textContent).toBe('2 vendors')
    expect(screen.getByText('alpha')).toBeTruthy()
    expect(screen.getByText('#prod')).toBeTruthy()
  })

  it('maps ProfilesHeader overflow actions and optional palette', () => {
    const onReload = vi.fn()
    const onExport = vi.fn()
    const onAdd = vi.fn()
    const onOpenPalette = vi.fn()
    const onEditSource = vi.fn()
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
          source: 'Edit TOML',
          overflow: 'More',
        }}
        palette={{ label: 'Command', shortcut: 'Ctrl K', title: 'Open palette' }}
        onAdd={onAdd}
        onExport={onExport}
        onReload={onReload}
        onOpenPalette={onOpenPalette}
        onEditSource={onEditSource}
      />,
    )
    fireEvent.click(screen.getByRole('button', { name: 'Command Ctrl K' }))
    expect(onOpenPalette).toHaveBeenCalledOnce()
    fireEvent.click(screen.getByRole('button', { name: 'More' }))
    fireEvent.click(screen.getByRole('menuitem', { name: 'Reload' }))
    expect(onReload).toHaveBeenCalledOnce()
    fireEvent.click(screen.getByRole('button', { name: 'Add' }))
    expect(onAdd).toHaveBeenCalledOnce()
  })

  it('maps ProfilesToolbar v-model pairs and exposes focusSearch', () => {
    const onUpdateQuery = vi.fn()
    const onUpdateStatusFilter = vi.fn()
    const onUpdateViewMode = vi.fn()
    const handle = { current: null as { focusSearch: () => void } | null }
    render(
      <ProfilesToolbar
        ref={(value) => {
          handle.current = value
        }}
        query="al"
        statusFilter="all"
        tagFilter={null}
        sortBy="recent"
        viewMode="card"
        resultCount={1}
        total={3}
        allTags={['fast']}
        i18nPrefix="profilesTest.toolbar"
        onUpdateQuery={onUpdateQuery}
        onUpdateStatusFilter={onUpdateStatusFilter}
        onUpdateTagFilter={vi.fn()}
        onUpdateProviderFilter={vi.fn()}
        onUpdateSortBy={vi.fn()}
        onUpdateViewMode={onUpdateViewMode}
      />,
    )
    fireEvent.input(screen.getByRole('textbox'), { target: { value: 'beta' } })
    expect(onUpdateQuery).toHaveBeenCalledWith('beta')
    fireEvent.click(screen.getByRole('button', { name: 'profilesTest.toolbar.statusEnabled' }))
    expect(onUpdateStatusFilter).toHaveBeenCalledWith('enabled')
    fireEvent.click(screen.getByTitle('profilesTest.toolbar.viewList'))
    expect(onUpdateViewMode).toHaveBeenCalledWith('list')
    expect(handle.current?.focusSearch).toBeTypeOf('function')
  })

  it('maps ProfilesQuickRail apply/more and pin numbers', () => {
    const onApply = vi.fn()
    const onMore = vi.fn()
    const togglePin = vi.fn()
    render(
      <ProfilesQuickRail
        profiles={[sample, { name: 'beta', enabled: true }]}
        currentName="alpha"
        i18nPrefix="profilesTest"
        quickSwitch={quickSwitch({ togglePin })}
        moreCount={2}
        onApply={onApply}
        onMore={onMore}
      />,
    )
    fireEvent.click(screen.getByText('alpha'))
    expect(onApply).toHaveBeenCalledWith('alpha')
    expect(screen.getByText('1')).toBeTruthy()
    expect(screen.getByText('CtrlK')).toBeTruthy()
    fireEvent.click(screen.getByText(/quickRailMore/))
    expect(onMore).toHaveBeenCalledOnce()
  })

  it('maps ProfilesInspector preview fields, diff, and edit', () => {
    const onEdit = vi.fn()
    render(
      <div className="profiles-view">
        <ProfilesInspector
          profiles={[sample, { name: 'beta', model: 'opus' }]}
          previewProfile={{ name: 'beta', model: 'opus' }}
          currentProfile={sample}
          i18nPrefix="profilesTest.inspector"
          descriptor={inspectorDescriptor}
          onEdit={onEdit}
          onLocate={vi.fn()}
          onTagSelect={vi.fn()}
        />
      </div>,
    )
    expect(screen.getAllByText('beta').length).toBeGreaterThan(0)
    expect(screen.getAllByText('opus').length).toBeGreaterThan(0)
    expect(screen.getByText('sonnet')).toBeTruthy()
    fireEvent.click(screen.getByRole('button', { name: /editAction/ }))
    expect(onEdit).toHaveBeenCalledWith('beta')
  })

  it('maps ProfilesCommandPalette apply and close', async () => {
    const onApply = vi.fn()
    const onUpdateOpen = vi.fn()
    const add = vi.fn()
    render(
      <ProfilesCommandPalette
        open
        profiles={[sample]}
        descriptor={{
          isEnabled: (profile) => profile.enabled !== false,
          hint: (profile) => profile.description ?? undefined,
        }}
        actions={[{ id: 'add', icon: 'Plus', labelKey: 'profilesTest.add', handler: add }]}
        i18nPrefix="profilesTest.commandPalette"
        onUpdateOpen={onUpdateOpen}
        onApply={onApply}
      />,
    )
    fireEvent.click(await screen.findByText(/actionApply/))
    expect(onApply).toHaveBeenCalledWith('alpha')
    expect(onUpdateOpen).toHaveBeenCalledWith(false)
  })

  it('maps ProfilesRawEditorPanel getRaw content and dirty callback', async () => {
    const onDirtyChange = vi.fn()
    const onClose = vi.fn()
    const onSaved = vi.fn()
    const router = createMemoryRouter(
      [
        {
          path: '/',
          element: (
            <ProfilesRawEditorPanel
              getRaw={async () => ({
                status: 'ok',
                content: 'name = "alpha"',
                token: 'tok',
                path: '/tmp/profiles.toml',
                exists: true,
              })}
              saveRaw={async () => ({ status: 'saved', token: 'tok2', profiles_count: 1 })}
              onSaved={onSaved}
              onClose={onClose}
              onDirtyChange={onDirtyChange}
            />
          ),
        },
      ],
      { initialEntries: ['/'] },
    )
    render(<RouterProvider router={router} />)
    const editor = await screen.findByTestId('profiles-raw-editor')
    expect((editor as HTMLTextAreaElement).value).toBe('name = "alpha"')
    fireEvent.input(editor, { target: { value: 'name = "beta"' } })
    await waitFor(() => expect(onDirtyChange).toHaveBeenCalledWith(true))
    expect(screen.getByText(/profilesRaw\.unsaved|有未保存修改/)).toBeTruthy()
  })
})
