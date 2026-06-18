import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createI18nStub } from './helpers/i18n-stub'

const clearWslCacheMock = vi.fn()
const detectWslCliMock = vi.fn()
const getWslCacheStatusMock = vi.fn()
const listWslDistrosMock = vi.fn()
const readWslConfigMock = vi.fn()
const refreshWslDistrosMock = vi.fn()
const syncWslConfigMock = vi.fn()

vi.mock('@/components/ui/SIcon.vue', () => ({
  default: defineComponent({
    props: {
      name: { type: String, required: true },
      size: { type: String, default: '' },
    },
    setup(props) {
      return () => h('span', { 'data-icon': props.name, class: props.size })
    },
  }),
}))

vi.mock('@/api/runtime/wsl', () => ({
  clearWslCache: (...args: unknown[]) => clearWslCacheMock(...args),
  detectWslCli: (...args: unknown[]) => detectWslCliMock(...args),
  getWslCacheStatus: (...args: unknown[]) => getWslCacheStatusMock(...args),
  listWslDistros: (...args: unknown[]) => listWslDistrosMock(...args),
  readWslConfig: (...args: unknown[]) => readWslConfigMock(...args),
  refreshWslDistros: (...args: unknown[]) => refreshWslDistrosMock(...args),
  syncWslConfig: (...args: unknown[]) => syncWslConfigMock(...args),
}))

import WslManagementView from '@/views/WslManagementView.vue'

const distros = [
  {
    name: 'Ubuntu-22.04',
    is_default: true,
    version: 'Wsl2',
    state: 'Running',
  },
]

const cacheStatus = {
  has_memory_cache: true,
  has_disk_cache: true,
  cached_at: '2026-04-02T10:00:00Z',
  distro_count: 1,
  is_expired: false,
  age_secs: 42,
}

const cliStatus = {
  claude: true,
  codex: true,
  gemini: false,
}

const flush = async () => {
  await Promise.resolve()
  await nextTick()
  await nextTick()
}

const waitUntil = async <T>(predicate: () => T | undefined, attempts = 10): Promise<T> => {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    const value = predicate()
    if (value !== undefined) {
      return value
    }
    await flush()
  }

  throw new Error('Timed out waiting for expected WSL view state')
}

const mountView = async () => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () => h(WslManagementView)
    },
  }))

  app.use(createI18nStub('zh-CN'))
  app.mount(el)
  await flush()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

beforeEach(() => {
  document.body.innerHTML = ''
  clearWslCacheMock.mockReset()
  detectWslCliMock.mockReset()
  getWslCacheStatusMock.mockReset()
  listWslDistrosMock.mockReset()
  readWslConfigMock.mockReset()
  refreshWslDistrosMock.mockReset()
  syncWslConfigMock.mockReset()

  clearWslCacheMock.mockResolvedValue(undefined)
  detectWslCliMock.mockResolvedValue(cliStatus)
  getWslCacheStatusMock.mockResolvedValue(cacheStatus)
  listWslDistrosMock.mockResolvedValue(distros)
  readWslConfigMock.mockResolvedValue('config = "ok"')
  refreshWslDistrosMock.mockResolvedValue(distros)
  syncWslConfigMock.mockResolvedValue('同步完成')
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('WslManagementView smoke', () => {
  it('loads distro details on mount through the runtime API wrappers', async () => {
    const { el, unmount } = await mountView()

    try {
      await waitUntil(() => {
        if (
          listWslDistrosMock.mock.calls.length === 0 ||
          getWslCacheStatusMock.mock.calls.length === 0 ||
          detectWslCliMock.mock.calls.length === 0 ||
          readWslConfigMock.mock.calls.length === 0
        ) {
          return undefined
        }

        expect(listWslDistrosMock).toHaveBeenCalledWith(false)
        expect(getWslCacheStatusMock).toHaveBeenCalledTimes(1)
        expect(detectWslCliMock).toHaveBeenCalledWith('Ubuntu-22.04')
        expect(readWslConfigMock).toHaveBeenCalledWith({
          distro: 'Ubuntu-22.04',
          platform: 'claude',
          path: 'settings.json',
        })

        return true
      })

      await waitUntil(() => {
        if (
          el.textContent?.includes('Ubuntu-22.04') &&
          el.textContent.includes('config = "ok"')
        ) {
          return true
        }

        return undefined
      })

      expect(el.textContent).toContain('Ubuntu-22.04')
      expect(el.textContent).toContain('config = "ok"')
      const options = Array.from(el.querySelectorAll('option')).map((option) => option.textContent?.trim())
      expect(options).not.toContain('droid')
    } finally {
      unmount()
    }
  })

  it('supports sync and cache-clear actions via the wrapper layer', async () => {
    const { el, unmount } = await mountView()

    try {
      const syncButton = await waitUntil(() => {
        const button = Array.from(el.querySelectorAll<HTMLButtonElement>('button')).find(
          (candidate) => candidate.textContent?.includes('从 WSL 拉取')
        )
        return button
      })
      expect(syncButton).not.toBeUndefined()
      syncButton?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await flush()

      expect(syncWslConfigMock).toHaveBeenCalledWith({
        distro: 'Ubuntu-22.04',
        platform: 'claude',
        direction: 'wslToLocal',
      })
      expect(el.textContent).toContain('同步完成')

      const clearButton = Array.from(el.querySelectorAll<HTMLButtonElement>('button')).find(
        (button) => button.textContent?.includes('清除缓存')
      )
      expect(clearButton).not.toBeUndefined()
      clearButton?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await flush()

      expect(clearWslCacheMock).toHaveBeenCalledTimes(1)
      expect(getWslCacheStatusMock).toHaveBeenCalledTimes(2)
    } finally {
      unmount()
    }
  })
})
