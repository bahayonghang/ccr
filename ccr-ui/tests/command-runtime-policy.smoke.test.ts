import { beforeEach, describe, expect, it, vi } from 'vitest'

const tauriInvoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: tauriInvoke,
}))

describe('generated command runtime policy', () => {
  beforeEach(() => {
    tauriInvoke.mockResolvedValue(undefined)
  })

  it('injects the registry-owned confirmation token for user-gesture commands', async () => {
    const { invoke } = await import('@/api/invokeRuntime')

    await invoke('sync_push', { force: true })

    expect(tauriInvoke).toHaveBeenCalledWith(
      'sync_push',
      { force: true, confirmationToken: 'desktop-confirm:sync_push' },
    )
  })

  it('does not add confirmation data to read-only commands', async () => {
    const { invoke } = await import('@/api/invokeRuntime')

    await invoke('list_sync_assets')

    expect(tauriInvoke).toHaveBeenCalledWith('list_sync_assets', undefined)
  })

  it('leaves opaque install capability proof to the backend plan handle', async () => {
    const { invoke } = await import('@/api/invokeRuntime')

    await invoke('llmusage_install_execute', { planId: 'plan-123' })

    expect(tauriInvoke).toHaveBeenCalledWith(
      'llmusage_install_execute',
      { planId: 'plan-123' },
    )
  })

  it('rejects raw payloads when confirmation metadata requires JSON', async () => {
    const { invoke } = await import('@/api/invokeRuntime')

    expect(() => invoke('sync_push', new Uint8Array([1]))).toThrow(
      'Command sync_push requires a JSON confirmation payload',
    )
    expect(tauriInvoke).not.toHaveBeenCalled()
  })
})
