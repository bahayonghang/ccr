import { createApp, defineComponent, ref } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { useProfilesHotkeys } from '@/composables/useProfilesHotkeys'

interface MountedHotkeys {
  unmount: () => void
}

const mountHotkeys = (options: {
  getStableTargets: () => string[]
  onApply: (name: string) => void
}): MountedHotkeys => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(
    defineComponent({
      setup() {
        useProfilesHotkeys({
          paletteOpen: ref(false),
          focusSearch: () => undefined,
          getStableTargets: options.getStableTargets,
          onApply: options.onApply,
        })
        return () => null
      },
    }),
  )
  app.mount(el)

  return {
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

const pressDigit = (digit: string) => {
  window.dispatchEvent(new KeyboardEvent('keydown', { key: digit, ctrlKey: true }))
}

describe('useProfilesHotkeys stable targets smoke', () => {
  let mounted: MountedHotkeys | null = null

  afterEach(() => {
    mounted?.unmount()
    mounted = null
  })

  it('switches to stable targets for digit keys', () => {
    const onApply = vi.fn()
    mounted = mountHotkeys({
      getStableTargets: () => ['pinned-a', 'pinned-b'],
      onApply,
    })

    pressDigit('1')
    expect(onApply).toHaveBeenCalledWith('pinned-a')
  })

  it('ignores digits beyond the pinned array instead of falling back', () => {
    const onApply = vi.fn()
    mounted = mountHotkeys({
      getStableTargets: () => ['pinned-a'],
      onApply,
    })

    pressDigit('2')
    expect(onApply).not.toHaveBeenCalled()
  })
})
