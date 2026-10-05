import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { EditConfigModal } from '@/features/configs/components/EditConfigModal'
import { valuesFromConfig } from '@/features/configs/lib/configForm'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { useConfigsViewStore } from '@/features/configs/stores'
import * as api from '@/api'

vi.mock('@/api', () => ({
  getConfig: vi.fn(),
  updateConfig: vi.fn(),
}))

vi.mock('@/configs/surfaceNotify', () => ({
  surfaceNotify: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    confirm: vi.fn().mockResolvedValue(true),
  },
}))

describe('EditConfigModal form draft', () => {
  const original = {
    version: 'fixture-version', name: 'work', description: 'from-api',
    base_url: 'https://api.example.com', auth_token: 'masked-token',
    is_current: false, is_default: false, usage_count: 0, enabled: true,
  }

  beforeEach(() => {
    vi.clearAllMocks()
    useConfigsViewStore.setState(useConfigsViewStore.getInitialState())
    vi.mocked(api.getConfig).mockResolvedValue(original)
  })

  it('restores the in-memory draft instead of the loaded config', async () => {
    const baseline = valuesFromConfig(original)
    useConfigsViewStore.getState().setFormDraft('work', {
      baseline, version: original.version,
      values: { ...baseline, description: 'from-draft', auth_token: 'synthetic-draft-secret' },
    })
    render(<EditConfigModal isOpen configName="work" onClose={vi.fn()} onSaved={vi.fn()} />)
    await waitFor(() => {
      expect(screen.getByDisplayValue('from-draft')).toBeTruthy()
    })
    expect(screen.queryByDisplayValue('from-api')).toBeNull()
    expect(screen.queryByDisplayValue('synthetic-draft-secret')).toBeNull()
  })

  it('keeps the original version and changed fields when a draft reopens after an external edit', async () => {
    const onClose = vi.fn()
    const onSaved = vi.fn()
    const first = render(<EditConfigModal isOpen configName="work" onClose={onClose} onSaved={onSaved} />)
    const description = await screen.findByDisplayValue('from-api')
    fireEvent.change(description, { target: { value: 'my-draft' } })
    await waitFor(() => expect(useConfigsViewStore.getState().formDrafts.work).toMatchObject({
      version: 'fixture-version', values: { description: 'my-draft' },
      baseline: { description: 'from-api', base_url: 'https://api.example.com' },
    }))
    first.unmount()

    const external = { ...original, version: 'external-version', base_url: 'https://external.example.com' }
    vi.mocked(api.getConfig).mockResolvedValue(external)
    vi.mocked(api.updateConfig).mockImplementation(async (request) => {
      if (request.expectedVersion !== external.version) throw new Error('stale_config_version')
      throw new Error('Unexpected authorization of the old draft')
    })
    render(<EditConfigModal isOpen configName="work" onClose={onClose} onSaved={onSaved} />)
    await screen.findByDisplayValue('my-draft')
    fireEvent.click(screen.getByRole('button', { name: /保存更改|Save Changes/ }))
    await waitFor(() => expect(api.updateConfig).toHaveBeenCalledWith({
      platform: 'claude', name: 'work', expectedVersion: 'fixture-version',
      data: { description: 'my-draft' },
    }))
    await waitFor(() => expect(surfaceNotify.error).toHaveBeenCalledWith('stale_config_version'))
    expect(external.base_url).toBe('https://external.example.com')
    expect(onClose).not.toHaveBeenCalled()
    expect(onSaved).not.toHaveBeenCalled()
    expect(useConfigsViewStore.getState().formDrafts.work).toMatchObject({ version: 'fixture-version' })

    fireEvent.click(screen.getByRole('button', { name: /^重新载入$|^Reload$/ }))
    await screen.findByDisplayValue('https://external.example.com')
    fireEvent.change(screen.getByDisplayValue('from-api'), { target: { value: 'reviewed-draft' } })
    vi.mocked(api.updateConfig).mockResolvedValue({ platform: 'claude', name: 'work' })
    fireEvent.click(screen.getByRole('button', { name: /保存更改|Save Changes/ }))
    await waitFor(() => expect(api.updateConfig).toHaveBeenLastCalledWith({
      platform: 'claude', name: 'work', expectedVersion: 'external-version',
      data: { description: 'reviewed-draft' },
    }))
  })

  it('clears the old save capability when another configuration fails to load', async () => {
    const view = render(<EditConfigModal isOpen configName="work" onClose={vi.fn()} onSaved={vi.fn()} />)
    await screen.findByDisplayValue('from-api')
    vi.mocked(api.getConfig).mockRejectedValueOnce(new Error('cannot_read_other'))
    view.rerender(<EditConfigModal isOpen configName="other" onClose={vi.fn()} onSaved={vi.fn()} />)
    await waitFor(() => expect(surfaceNotify.error).toHaveBeenCalledWith('cannot_read_other'))
    expect((screen.getByRole('button', { name: /保存更改|Save Changes/ }) as HTMLButtonElement).disabled).toBe(true)
    const form = view.baseElement.querySelector('form')
    expect(form).not.toBeNull()
    await act(async () => { fireEvent.submit(form!) })
    expect(api.updateConfig).not.toHaveBeenCalled()
  })

  it('stores no plaintext credential in a versioned edit draft', async () => {
    const view = render(<EditConfigModal isOpen configName="work" onClose={vi.fn()} onSaved={vi.fn()} />)
    await screen.findByDisplayValue('from-api')
    const token = view.baseElement.querySelector('input[name="auth_token"]')
    expect(token).not.toBeNull()
    fireEvent.change(token!, { target: { value: 'synthetic-new-secret' } })
    await waitFor(() => expect(useConfigsViewStore.getState().formDrafts.work).toMatchObject({ values: { auth_token: '' } }))
    expect(JSON.stringify(useConfigsViewStore.getState().formDrafts)).not.toContain('synthetic-new-secret')
  })
})
