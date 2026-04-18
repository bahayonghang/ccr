import { describe, expect, it } from 'vitest'

import { shouldPersistTrayPanelManualPosition } from '@/composables/useCodexTrayPanel'

describe('useCodexTrayPanel manual drag persistence', () => {
  it('ignores tiny drag jitter so reopening stays tray-anchored', () => {
    expect(
      shouldPersistTrayPanelManualPosition({ x: 1180, y: 84 }, { x: 1186, y: 91 })
    ).toBe(false)
  })

  it('persists intentional tray panel moves', () => {
    expect(
      shouldPersistTrayPanelManualPosition({ x: 1180, y: 84 }, { x: 1212, y: 156 })
    ).toBe(true)
  })
})
