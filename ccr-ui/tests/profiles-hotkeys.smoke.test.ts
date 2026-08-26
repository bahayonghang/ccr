import { renderHook } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { useProfilesHotkeys } from '@/composables/useProfilesHotkeys'

const pressDigit = (digit: string) => {
  window.dispatchEvent(new KeyboardEvent('keydown', { key: digit, ctrlKey: true }))
}

describe('useProfilesHotkeys stable targets smoke', () => {
  it('switches to stable targets for digit keys', () => {
    const onApply = vi.fn()
    const hook = renderHook(() =>
      useProfilesHotkeys({
        paletteOpen: false,
        setPaletteOpen: vi.fn(),
        focusSearch: vi.fn(),
        getStableTargets: () => ['pinned-a', 'pinned-b'],
        onApply,
      }),
    )
    pressDigit('1')
    expect(onApply).toHaveBeenCalledWith('pinned-a')
    hook.unmount()
  })

  it('ignores digits beyond the pinned array instead of falling back', () => {
    const onApply = vi.fn()
    const hook = renderHook(() =>
      useProfilesHotkeys({
        paletteOpen: false,
        setPaletteOpen: vi.fn(),
        focusSearch: vi.fn(),
        getStableTargets: () => ['pinned-a'],
        onApply,
      }),
    )
    pressDigit('2')
    expect(onApply).not.toHaveBeenCalled()
    hook.unmount()
  })
})
