import { describe, expect, it, vi } from 'vitest'

const invokeMock = vi.hoisted(() => vi.fn())

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

import { updateCodexProfile } from '@/api/domains/codex'

describe('codex profile update API', () => {
  it('preserves the original profile key while sending the renamed target name in config', async () => {
    invokeMock.mockResolvedValue({
      name: 'ice-renamed',
      message: "Codex Profile 'ice-renamed' 已更新",
    })

    await updateCodexProfile('ice', {
      name: 'ice-renamed',
      model: 'gpt-5.4',
      auth_mode: 'provider_env_key',
    })

    expect(invokeMock).toHaveBeenCalledWith('codex_update_profile', {
      name: 'ice',
      config: expect.objectContaining({
        name: 'ice-renamed',
        model: 'gpt-5.4',
        auth_mode: 'provider_env_key',
      }),
    })
  })
})
