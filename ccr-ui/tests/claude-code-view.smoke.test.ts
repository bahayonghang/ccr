import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { createI18nStub } from './helpers/i18n-stub'

vi.mock('vue-router', () => ({
  RouterLink: defineComponent({
    props: {
      to: { type: [String, Object], required: true },
    },
    setup(props, { slots }) {
      return () => h('a', { href: String(props.to), 'data-route': String(props.to) }, slots.default?.())
    },
  }),
}))

vi.mock('@/components/common/AnimatedBackground.vue', () => ({
  default: defineComponent({
    setup() {
      return () => h('div', { 'data-testid': 'animated-background' })
    },
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

import ClaudeCodeView from '@/views/ClaudeCodeView.vue'

const mountView = async (locale: 'en-US' | 'zh-CN' = 'en-US') => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () => h(ClaudeCodeView)
    },
  }))

  app.use(createI18nStub(locale))
  app.mount(el)
  await Promise.resolve()
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
  document.body.innerHTML = ''
})

describe('ClaudeCodeView smoke', () => {
  it('focuses the English page on Claude Code modules instead of cloud sync', async () => {
    const { el, unmount } = await mountView('en-US')

    try {
      const text = el.textContent ?? ''
      expect(text).toContain('Profiles')
      expect(text).toContain('Auth')
      expect(text).toContain('Settings')
      expect(text).toContain('MCP Servers')
      expect(text).toContain('Agents')
      expect(text).toContain('Plugins')
      expect(text).toContain('Slash Commands')
      expect(text).not.toContain('Cloud Sync')
      expect(text).not.toContain('WebDAV')
      expect(text).not.toContain('Auto Backup')

      const routeTargets = Array.from(el.querySelectorAll('[data-route]')).map(node => node.getAttribute('data-route'))
      expect(routeTargets).toContain('/claude-code/profiles')
      expect(routeTargets).toContain('/claude-code/auth')
      expect(routeTargets).toContain('/claude-code/settings')
      expect(routeTargets).toContain('/mcp-manager')
      expect(routeTargets).toContain('/agents')
      expect(routeTargets).toContain('/plugins')
      expect(routeTargets).toContain('/slash-commands')
      expect(routeTargets).not.toContain('/sync')
    } finally {
      unmount()
    }
  })

  it('keeps the Chinese page free of cloud backup copy', async () => {
    const { el, unmount } = await mountView('zh-CN')

    try {
      const text = el.textContent ?? ''
      expect(text).toContain('Profiles')
      expect(text).toContain('Auth')
      expect(text).toContain('Settings')
      expect(text).toContain('MCP Servers')
      expect(text).toContain('Agents')
      expect(text).toContain('Plugins')
      expect(text).toContain('Slash Commands')
      expect(text).not.toContain('云同步')
      expect(text).not.toContain('WebDAV')
      expect(text).not.toContain('自动备份')

      const syncLinks = el.querySelectorAll('[data-route="/sync"]')
      expect(syncLinks).toHaveLength(0)
    } finally {
      unmount()
    }
  })
})
