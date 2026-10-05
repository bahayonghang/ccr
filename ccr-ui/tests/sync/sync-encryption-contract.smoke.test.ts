import { beforeEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.hoisted(() => vi.fn())

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

import { pullSyncAsset, pushSyncAsset, syncAllAssets } from '@/api/domains/sync'

describe('sync encryption IPC contract', () => {
  beforeEach(() => {
    invokeMock.mockReset()
    invokeMock.mockResolvedValue(undefined)
  })

  it('keeps the passphrase inside one typed asset operation payload', async () => {
    await pushSyncAsset('codex-config', {
      force: true,
      passphrase: 'operation-only-passphrase',
      migratePlaintextV1: false,
    })

    expect(invokeMock).toHaveBeenCalledWith('sync_push_asset', {
      confirmationToken: 'desktop-confirm:sync_push_asset',
      payload: {
        id: 'codex-config',
        force: true,
        passphrase: 'operation-only-passphrase',
        migratePlaintextV1: false,
      },
    })
  })

  it('does not enable plaintext migration unless explicitly requested', async () => {
    await pullSyncAsset('claude-settings', {
      passphrase: 'migration-passphrase',
      migratePlaintextV1: true,
    })
    await syncAllAssets({ force: false, passphrase: 'batch-passphrase' })

    expect(invokeMock).toHaveBeenNthCalledWith(1, 'sync_pull_asset', {
      confirmationToken: 'desktop-confirm:sync_pull_asset',
      payload: {
        id: 'claude-settings',
        passphrase: 'migration-passphrase',
        migratePlaintextV1: true,
      },
    })
    expect(invokeMock).toHaveBeenNthCalledWith(2, 'sync_all_assets', {
      confirmationToken: 'desktop-confirm:sync_all_assets',
      payload: { force: false, passphrase: 'batch-passphrase' },
    })
  })
})
