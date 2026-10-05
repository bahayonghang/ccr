import { beforeEach } from 'vitest'

class MemoryStorage implements Storage {
  #store = new Map<string, string>()

  get length(): number {
    return this.#store.size
  }

  clear(): void {
    this.#store.clear()
  }

  getItem(key: string): string | null {
    return this.#store.get(String(key)) ?? null
  }

  key(index: number): string | null {
    const keys = [...this.#store.keys()]
    return keys[index] ?? null
  }

  removeItem(key: string): void {
    this.#store.delete(String(key))
  }

  setItem(key: string, value: string): void {
    this.#store.set(String(key), String(value))
  }
}

const localStorageShim = new MemoryStorage()

Object.defineProperty(globalThis, 'localStorage', {
  configurable: true,
  value: localStorageShim,
  writable: true,
})

if (typeof window !== 'undefined') {
  Object.defineProperty(window, 'localStorage', {
    configurable: true,
    value: localStorageShim,
    writable: true,
  })
}

if (typeof window !== 'undefined' && typeof window.matchMedia !== 'function') {
  Object.defineProperty(window, 'matchMedia', {
    configurable: true,
    writable: true,
    value: (query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addEventListener: () => undefined,
      removeEventListener: () => undefined,
      addListener: () => undefined,
      removeListener: () => undefined,
      dispatchEvent: () => false,
    }),
  })
}

beforeEach(() => {
  localStorageShim.clear()
})
