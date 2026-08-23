import { render, screen, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { EditConfigModal } from '@/features/configs/components/EditConfigModal'
import { emptyConfigForm } from '@/features/configs/lib/configForm'
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
  beforeEach(() => {
    useConfigsViewStore.setState(useConfigsViewStore.getInitialState())
    vi.mocked(api.getConfig).mockResolvedValue({
      name: 'work',
      description: 'from-api',
      base_url: 'https://api.example.com',
      auth_token: 'sk-from-api',
      is_current: false,
      is_default: false,
      usage_count: 0,
      enabled: true,
    })
  })

  it('restores the in-memory draft instead of the loaded config', async () => {
    useConfigsViewStore.getState().setFormDraft('work', {
      ...emptyConfigForm(),
      description: 'from-draft',
      base_url: 'https://draft.example.com',
      auth_token: 'sk-draft',
    })
    render(<EditConfigModal isOpen configName="work" onClose={vi.fn()} onSaved={vi.fn()} />)
    await waitFor(() => {
      expect(screen.getByDisplayValue('from-draft')).toBeTruthy()
    })
    expect(screen.queryByDisplayValue('from-api')).toBeNull()
  })
})
