import { fireEvent, render, screen } from '@testing-library/react'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import { SyncPassphraseModal } from '@/features/sync/SyncPassphraseModal'

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

describe('SyncPassphraseModal', () => {
  it('clears the passphrase on submit and does not keep it in the input', () => {
    const onSubmit = vi.fn()
    const onOpenChange = vi.fn()
    render(
      <SyncPassphraseModal open assetName="codex-config" onOpenChange={onOpenChange} onSubmit={onSubmit} />,
    )
    const input = screen.getByPlaceholderText(/passphrase|口令|密码/i)
    fireEvent.change(input, { target: { value: 'operation-only-passphrase' } })
    fireEvent.click(screen.getByRole('button', { name: /continue|继续|确认/i }))
    expect(onSubmit).toHaveBeenCalledWith({
      passphrase: 'operation-only-passphrase',
      migratePlaintextV1: false,
    })
    expect(onOpenChange).toHaveBeenCalledWith(false)
  })
})
