import { createApp, defineComponent, h } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { useStream } from '@/composables/useStream'

const encoder = new TextEncoder()

const createBodyStream = (chunks: string[]) => new ReadableStream({
  start(controller) {
    for (const chunk of chunks) {
      controller.enqueue(encoder.encode(chunk))
    }
    controller.close()
  }
})

const mountUseStream = async (chunks: string[], maxLines = 2000) => {
  vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
    ok: true,
    status: 200,
    statusText: 'OK',
    body: createBodyStream(chunks),
  }))

  const el = document.createElement('div')
  document.body.appendChild(el)

  const api = {} as ReturnType<typeof useStream>
  const app = createApp(defineComponent({
    setup() {
      Object.assign(api, useStream('/stream', maxLines))
      return () => h('div')
    },
  }))

  app.mount(el)

  return {
    api,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('useStream smoke', () => {
  it('preserves blank lines and merges split chunks into complete lines', async () => {
    const { api, unmount } = await mountUseStream(['alpha\n\nbe', 'ta\ngamma'])

    try {
      await api.start()
      expect(api.lines.value).toEqual(['alpha', '', 'beta', 'gamma'])
      expect(api.isComplete.value).toBe(true)
    } finally {
      unmount()
    }
  })

  it('keeps only the newest lines when maxLines is exceeded', async () => {
    const { api, unmount } = await mountUseStream(['one\ntwo\nthree\nfour\nfive'], 3)

    try {
      await api.start()
      expect(api.lines.value).toEqual(['three', 'four', 'five'])
    } finally {
      unmount()
    }
  })
})
