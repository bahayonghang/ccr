import { afterEach, describe, expect, it } from 'vitest'
import { isTauriRuntime } from '@/utils/tauriRuntime'

const clearRuntimeMarkers = () => {
  Reflect.deleteProperty(window, '__TAURI__')
  Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
}

describe('tauri runtime smoke', () => {
  afterEach(() => {
    clearRuntimeMarkers()
  })

  it('returns false when no tauri runtime markers exist', () => {
    clearRuntimeMarkers()

    expect(isTauriRuntime()).toBe(false)
  })

  it('accepts the legacy __TAURI__ marker', () => {
    Object.defineProperty(window, '__TAURI__', {
      configurable: true,
      value: {},
    })

    expect(isTauriRuntime()).toBe(true)
  })

  it('accepts the v2 __TAURI_INTERNALS__ marker', () => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', {
      configurable: true,
      value: {},
    })

    expect(isTauriRuntime()).toBe(true)
  })
})

