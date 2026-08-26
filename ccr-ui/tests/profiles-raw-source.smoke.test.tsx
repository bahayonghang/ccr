import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import type { ReactElement } from 'react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { claudeProfilePresentation } from '@/configs/profilePresentation'
import { ProfilesSurface } from '@/features/platform/profiles/ProfilesSurface'
import { useUIStore } from '@/shell/stores/ui'
import { claudeDisplayRecords } from './fixtures/profiles'

const renderRouted = (ui: ReactElement) => {
  const router = createMemoryRouter([{ path: '*', element: ui }], { initialEntries: ['/'] })
  return render(<RouterProvider router={router} />)
}

describe('profiles raw source mode', () => {
  beforeEach(() => {
    useUIStore.setState({ confirmDialog: null })
  })

  it('does not render the source entry without a rawSource capability', () => {
    renderRouted(
      <ProfilesSurface
        platformKey="raw-none"
        presentation={claudeProfilePresentation}
        records={claudeDisplayRecords}
        current="claude-current"
        environmentLabel="local"
        environmentOk
        canOff={false}
        onAdd={vi.fn()}
        onEdit={vi.fn()}
        onApply={vi.fn()}
        onOff={vi.fn(async () => undefined)}
        onReload={vi.fn()}
      />,
    )
    expect(screen.queryByTestId('profiles-edit-source')).toBeNull()
  })

  it('asks for confirmation before entering source mode', async () => {
    const confirm = vi.fn().mockResolvedValue(false)
    useUIStore.setState({ requestConfirm: confirm })
    const getRaw = vi.fn()
    renderRouted(
      <ProfilesSurface
        platformKey="raw-warn"
        presentation={claudeProfilePresentation}
        records={claudeDisplayRecords}
        current="claude-current"
        environmentLabel="local"
        environmentOk
        canOff={false}
        onAdd={vi.fn()}
        onEdit={vi.fn()}
        onApply={vi.fn()}
        onOff={vi.fn(async () => undefined)}
        onReload={vi.fn()}
        rawSource={{
          getRaw,
          saveRaw: vi.fn(),
          refreshAll: vi.fn(async () => undefined),
        }}
      />,
    )
    fireEvent.click(screen.getByTestId('profiles-edit-source'))
    await waitFor(() => expect(confirm).toHaveBeenCalled())
    expect(getRaw).not.toHaveBeenCalled()
  })

  it('shows only reload and cancel on conflict', async () => {
    const confirm = vi.fn().mockResolvedValue(true)
    useUIStore.setState({ requestConfirm: confirm })
    renderRouted(
      <ProfilesSurface
        platformKey="raw-conflict"
        presentation={claudeProfilePresentation}
        records={claudeDisplayRecords}
        current="claude-current"
        environmentLabel="local"
        environmentOk
        canOff={false}
        onAdd={vi.fn()}
        onEdit={vi.fn()}
        onApply={vi.fn()}
        onOff={vi.fn(async () => undefined)}
        onReload={vi.fn()}
        rawSource={{
          getRaw: async () => ({
            status: 'ok',
            content: 'name = "a"',
            token: 'tok-1',
            path: '/tmp/profiles.toml',
            exists: true,
          }),
          saveRaw: async () => ({ status: 'conflict' }),
          refreshAll: vi.fn(async () => undefined),
        }}
      />,
    )
    fireEvent.click(screen.getByTestId('profiles-edit-source'))
    const editor = await screen.findByTestId('profiles-raw-editor')
    fireEvent.input(editor, { target: { value: 'name = "b"' } })
    fireEvent.click(screen.getByRole('button', { name: /profilesRaw\.save|保存源文件|保存/ }))
    await waitFor(() => expect(screen.getByText(/profilesRaw\.conflictTitle|文件已被外部修改/)).toBeTruthy())
    expect(screen.queryByRole('button', { name: /overwrite|覆盖/i })).toBeNull()
    expect(screen.getAllByRole('button', { name: /profilesRaw\.reload|重新加载|重载/ }).length).toBeGreaterThan(0)
  })

  it('retries activation_conflict with the same content, token, and force true', async () => {
    const confirm = vi.fn().mockResolvedValue(true)
    useUIStore.setState({ requestConfirm: confirm })
    const saveRaw = vi
      .fn()
      .mockResolvedValueOnce({ status: 'activation_conflict', current: 'claude-current' })
      .mockResolvedValueOnce({ status: 'saved', token: 'tok-2', profiles_count: 1 })
    const refreshAll = vi.fn(async () => undefined)
    renderRouted(
      <ProfilesSurface
        platformKey="raw-activation"
        presentation={claudeProfilePresentation}
        records={claudeDisplayRecords}
        current="claude-current"
        environmentLabel="local"
        environmentOk
        canOff={false}
        onAdd={vi.fn()}
        onEdit={vi.fn()}
        onApply={vi.fn()}
        onOff={vi.fn(async () => undefined)}
        onReload={vi.fn()}
        rawSource={{
          getRaw: async () => ({
            status: 'ok',
            content: 'name = "a"',
            token: 'tok-1',
            path: '/tmp/profiles.toml',
            exists: true,
          }),
          saveRaw,
          refreshAll,
        }}
      />,
    )
    fireEvent.click(screen.getByTestId('profiles-edit-source'))
    const editor = await screen.findByTestId('profiles-raw-editor')
    fireEvent.input(editor, { target: { value: 'name = "forced"' } })
    fireEvent.click(screen.getByRole('button', { name: /profilesRaw\.save|保存源文件|保存/ }))
    await waitFor(() => expect(saveRaw).toHaveBeenCalledTimes(2))
    expect(saveRaw.mock.calls[0]).toEqual(['name = "forced"', 'tok-1', false])
    expect(saveRaw.mock.calls[1]).toEqual(['name = "forced"', 'tok-1', true])
    await waitFor(() => expect(refreshAll).toHaveBeenCalled())
    expect(screen.queryByTestId('profiles-raw-editor')).toBeNull()
  })
})
