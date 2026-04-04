import { createApp, defineComponent, h, nextTick, reactive } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'

const routeState = reactive({
  meta: {
    hideGlobalBackground: false,
  },
})

vi.mock('vue', async () => {
  const actual = await vi.importActual<typeof import('vue')>('vue')

  return {
    ...actual,
    defineAsyncComponent: () => actual.defineComponent({
      name: 'AsyncComponentStub',
      setup() {
        return () => actual.h('div', { 'data-async-stub': 'true' })
      },
    }),
  }
})

vi.mock('vue-router', () => ({
  useRoute: () => routeState,
}))

import App from '@/App.vue'

const clearRuntimeMarkers = () => {
  Reflect.deleteProperty(window, '__TAURI__')
  Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
}

const originalPlatformDescriptor = Object.getOwnPropertyDescriptor(window.navigator, 'platform')

const setNavigatorPlatform = (value: string) => {
  Object.defineProperty(window.navigator, 'platform', {
    configurable: true,
    value,
  })
}

const mountApp = async () => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () => h(App)
    },
  }))

  app.component(
    'RouterView',
    defineComponent({
      name: 'RouterViewStub',
      setup() {
        return () => h('main', { 'data-stub': 'router-view' })
      },
    }),
  )

  app.mount(el)
  await nextTick()
  await nextTick()

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

afterEach(() => {
  clearRuntimeMarkers()

  if (originalPlatformDescriptor) {
    Object.defineProperty(window.navigator, 'platform', originalPlatformDescriptor)
  } else {
    Reflect.deleteProperty(window.navigator, 'platform')
  }

  document.body.innerHTML = ''
})

describe('App window chrome smoke', () => {
  it('keeps the custom titlebar for non-mac tauri runtimes', async () => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', {
      configurable: true,
      value: {},
    })
    setNavigatorPlatform('Win32')

    const { el, unmount } = await mountApp()

    try {
      expect(el.querySelector('.pt-9')).not.toBeNull()
    } finally {
      unmount()
    }
  })

  it('drops the custom titlebar on macOS tauri runtimes so native window chrome handles drag/maximize', async () => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', {
      configurable: true,
      value: {},
    })
    setNavigatorPlatform('MacIntel')

    const { el, unmount } = await mountApp()

    try {
      expect(el.querySelector('.pt-9')).toBeNull()
    } finally {
      unmount()
    }
  })
})
