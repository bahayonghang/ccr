import { createApp, defineComponent, h, nextTick, ref } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const listEnvironmentsMock = vi.fn()
const switchEnvironmentMock = vi.fn()
const refreshEnvironmentsMock = vi.fn()

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string) => key,
    locale: ref('en-US'),
  }),
}))

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

vi.mock('@/api/runtime/environment', () => ({
  listEnvironments: (...args: unknown[]) => listEnvironmentsMock(...args),
  switchEnvironment: (...args: unknown[]) => switchEnvironmentMock(...args),
  refreshEnvironments: (...args: unknown[]) => refreshEnvironmentsMock(...args),
}))

import EnvironmentSwitcher from '@/components/EnvironmentSwitcher.vue'

const baseEnvironments = [
  {
    id: 'local-env',
    name: 'Local',
    env_type: 'local',
    is_active: true,
    description: 'Run commands on this machine',
  },
  {
    id: 'wsl-env',
    name: 'Ubuntu WSL',
    env_type: 'wsl',
    is_active: false,
    description: 'Run commands inside WSL',
  },
]

const refreshedEnvironments = [
  {
    id: 'local-env',
    name: 'Local',
    env_type: 'local',
    is_active: false,
    description: 'Run commands on this machine',
  },
  {
    id: 'wsl-env',
    name: 'Ubuntu WSL',
    env_type: 'wsl',
    is_active: true,
    description: 'Run commands inside WSL',
  },
]

const flush = async () => {
  await Promise.resolve()
  await nextTick()
  await nextTick()
}

const mountSwitcher = async () => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () => h(EnvironmentSwitcher)
    },
  }))

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
  listEnvironmentsMock.mockReset()
  switchEnvironmentMock.mockReset()
  refreshEnvironmentsMock.mockReset()
  listEnvironmentsMock.mockResolvedValue(baseEnvironments)
  switchEnvironmentMock.mockResolvedValue(undefined)
  refreshEnvironmentsMock.mockResolvedValue(refreshedEnvironments)
})

afterEach(() => {
  document.body.innerHTML = ''
})

describe('EnvironmentSwitcher smoke', () => {
  it('loads environments on mount and switches to a newly selected environment', async () => {
    listEnvironmentsMock
      .mockResolvedValueOnce(baseEnvironments)
      .mockResolvedValueOnce(refreshedEnvironments)

    const { el, unmount } = await mountSwitcher()

    try {
      expect(listEnvironmentsMock).toHaveBeenCalledTimes(1)
      expect(el.textContent).toContain('Local')

      const trigger = el.querySelector('button')
      trigger?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await flush()

      const options = Array.from(el.querySelectorAll<HTMLButtonElement>('[role="option"]'))
      const wslOption = options.find((option) => option.textContent?.includes('Ubuntu WSL'))

      expect(wslOption).toBeDefined()
      wslOption?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await flush()

      expect(switchEnvironmentMock).toHaveBeenCalledWith('wsl-env')
      expect(listEnvironmentsMock).toHaveBeenCalledTimes(2)
      expect(el.textContent).toContain('Ubuntu WSL')
    } finally {
      unmount()
    }
  })

  it('refreshes the environment inventory from the toolbar button', async () => {
    const { el, unmount } = await mountSwitcher()

    try {
      const trigger = el.querySelector('button')
      trigger?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await flush()

      const refreshButton = el.querySelector('.env-switcher__menu button[title="common.environment.refresh"]')
      expect(refreshButton).not.toBeNull()

      refreshButton?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
      await flush()

      expect(refreshEnvironmentsMock).toHaveBeenCalledTimes(1)
      expect(el.textContent).toContain('Ubuntu WSL')
    } finally {
      unmount()
    }
  })
})
