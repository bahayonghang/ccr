import { beforeEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.hoisted(() => vi.fn())

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

import { sshConfirmHostFingerprint } from '@/api/domains/environment'

describe('SSH host trust boundary', () => {
  beforeEach(() => {
    invokeMock.mockReset()
    invokeMock.mockResolvedValue(undefined)
  })

  it('confirms only a backend-issued challenge id', async () => {
    const challengeId = '11111111-1111-4111-8111-111111111111'

    await sshConfirmHostFingerprint(challengeId)

    expect(invokeMock).toHaveBeenCalledWith('ssh_confirm_host_fingerprint', {
      request: { challenge_id: challengeId },
    })
  })
})
