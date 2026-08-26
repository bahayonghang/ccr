import { beforeEach, describe, expect, it, vi } from 'vitest'

const invokeMock = vi.hoisted(() => vi.fn())

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}))

describe('config domain API', () => {
  beforeEach(() => {
    invokeMock.mockReset()
    invokeMock.mockResolvedValue({})
  })

  it('uses typed confirmation tokens for destructive config commands', async () => {
    const { deleteConfig, importConfig, restoreConfig } = await import('@/api/domains/config')

    await deleteConfig('old')
    await importConfig({ content: 'current_config = "default"', mode: 'replace', backup: false })
    await restoreConfig('profiles.toml.pre_restore_20260604_120000.bak')

    expect(invokeMock).toHaveBeenNthCalledWith(1, 'delete_config', {
      name: 'old',
      confirmationToken: 'desktop-confirm:delete_config',
    })
    expect(invokeMock).toHaveBeenNthCalledWith(2, 'import_config', {
      content: 'current_config = "default"',
      mode: 'replace',
      backup: false,
      confirmationToken: 'desktop-confirm:import_config',
    })
    expect(invokeMock).toHaveBeenNthCalledWith(3, 'restore_config', {
      backupPath: 'profiles.toml.pre_restore_20260604_120000.bak',
      confirmationToken: 'desktop-confirm:restore_config',
    })
  })
})
