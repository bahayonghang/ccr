import { readFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'

vi.mock('vue-router', async () => {
  const { defineComponent: define, h: hyperscript } = await vi.importActual<typeof import('vue')>('vue')

  return {
    RouterLink: define({
      props: {
        to: {
          type: String,
          required: true,
        },
      },
      setup(props, { slots }) {
        return () => hyperscript('a', { href: props.to, 'data-router-link': props.to }, slots.default?.())
      },
    }),
  }
})

vi.mock('@/components/ui/SIcon.vue', async () => {
  const { defineComponent: define, h: hyperscript } = await vi.importActual<typeof import('vue')>('vue')

  return {
    default: define({
      props: {
        name: { type: String, required: true },
        size: { type: String, default: '' },
      },
      setup(props) {
        return () => hyperscript('span', { 'data-icon': props.name, class: props.size })
      },
    }),
  }
})

const flush = async () => {
  await Promise.resolve()
  await nextTick()
  await Promise.resolve()
  await nextTick()
}

const rawRgb = /rgba?\(\s*(?!0[\s,]+0[\s,]+0\b)\d{1,3}[\s,]/

const readSource = async (relativePath: string) => {
  const absolutePath = fileURLToPath(new URL(relativePath, import.meta.url))
  return readFile(absolutePath, 'utf8')
}

const mount = async (
  component: Parameters<typeof createApp>[0],
  props: Record<string, unknown> = {},
  slots: Record<string, () => unknown> = {},
) => {
  const el = document.createElement('div')
  document.body.appendChild(el)
  const app = createApp(defineComponent({
    setup() {
      return () => h(component, props, slots)
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

afterEach(() => {
  document.body.innerHTML = ''
  document.documentElement.removeAttribute('lang')
})

describe('Wave 1 UI primitives', () => {
  it('does not export StatCard', async () => {
    const source = await readSource('../src/components/ui/index.ts')
    expect(source).not.toContain('StatCard')
    expect(source).toContain('StatTile')
  })

  it('keeps PageShell and OpenCodePageShell free of literal rgb and decorative glow', async () => {
    const pageShell = await readSource('../src/components/ui/PageShell.vue')
    const openCodeShell = await readSource('../src/components/opencode/OpenCodePageShell.vue')

    expect(pageShell).not.toMatch(rawRgb)
    expect(openCodeShell).not.toMatch(rawRgb)
    expect(pageShell).not.toContain('linear-gradient')
    expect(openCodeShell).not.toContain('linear-gradient')
    expect(openCodeShell).not.toContain('opencode-page-shell__glow')
    expect(openCodeShell).toContain('PageShell')
    expect(openCodeShell).toContain('PageHeader')
  })

  it('renders PageHeader with a Latin eyebrow marked lang=en', async () => {
    document.documentElement.lang = 'zh-CN'
    const { default: PageHeader } = await import('@/components/ui/PageHeader.vue')
    const { el, unmount } = await mount(PageHeader, {
      title: 'Providers',
      eyebrow: 'OpenCode operator surface',
      description: 'Manage providers',
    })

    try {
      const eyebrow = el.querySelector('.page-header__eyebrow') as HTMLElement
      expect(eyebrow).toBeTruthy()
      expect(eyebrow.getAttribute('lang')).toBe('en')
      expect(el.querySelector('h1')?.textContent).toContain('Providers')
    } finally {
      unmount()
    }
  })

  it('does not force uppercase tracking on a CJK eyebrow', async () => {
    document.documentElement.lang = 'zh-CN'
    const { default: PageHeader } = await import('@/components/ui/PageHeader.vue')
    const { el, unmount } = await mount(PageHeader, {
      title: '设置',
      eyebrow: '操作台',
    })

    try {
      const eyebrow = el.querySelector('.page-header__eyebrow') as HTMLElement
      expect(eyebrow.getAttribute('lang')).toBeNull()
      const headerSource = await readSource('../src/components/ui/PageHeader.vue')
      expect(headerSource).toContain(':lang(zh)')
      expect(headerSource).toContain(':lang(zh-CN)')
      expect(headerSource).toContain('text-transform: none')
    } finally {
      unmount()
    }
  })

  it('renders StatTile as a bare tile with tabular-nums', async () => {
    const { default: StatTile } = await import('@/components/ui/StatTile.vue')
    const { el, unmount } = await mount(StatTile, {
      label: 'Providers',
      value: 12,
      hint: 'live',
    })

    try {
      expect(el.querySelector('.ui-card')).toBeNull()
      const value = el.querySelector('.stat-tile__value') as HTMLElement
      expect(value.textContent).toContain('12')
      const tileSource = await readSource('../src/components/ui/StatTile.vue')
      expect(tileSource).toContain('tabular-nums')
      expect(tileSource).not.toContain('ui-card')
    } finally {
      unmount()
    }
  })

  it('keeps PillToggleGroup single-select and tonal when active', async () => {
    const { default: PillToggleGroup } = await import('@/components/ui/PillToggleGroup.vue')
    const updates: Array<string | number> = []
    const { el, unmount } = await mount(PillToggleGroup, {
      options: [
        { value: '7', label: '7d' },
        { value: '30', label: '30d' },
      ],
      modelValue: '7',
      'onUpdate:modelValue': (value: string) => {
        updates.push(value)
      },
    })

    try {
      const buttons = Array.from(el.querySelectorAll<HTMLButtonElement>('.pill-toggle-group__item'))
      expect(buttons).toHaveLength(2)
      expect(buttons[0].getAttribute('aria-checked')).toBe('true')
      expect(buttons[0].classList.contains('pill-toggle-group__item--active')).toBe(true)

      buttons[1].click()
      await flush()
      expect(updates).toEqual(['30'])
    } finally {
      unmount()
    }
  })

  it('does not render Card glow or gradient overlays when deprecated props are set', async () => {
    const { default: Card } = await import('@/components/ui/Card.vue')
    const { el, unmount } = await mount(Card, {
      glow: true,
      glowEffect: true,
      gradientBorder: true,
      pattern: true,
    }, {
      default: () => 'body',
    })

    try {
      expect(el.querySelector('.ui-card')).toBeTruthy()
      expect(el.querySelector('.ui-card-pattern')).toBeNull()
      expect(el.innerHTML).not.toContain('radial-gradient')
      expect(el.innerHTML).not.toContain('linear-gradient')
    } finally {
      unmount()
    }
  })

  it('uses a pill primary button and a squared secondary button', async () => {
    const source = await readSource('../src/components/ui/Button.vue')
    expect(source).toContain('.ui-button--primary')
    expect(source).toContain('border-radius: var(--radius-full)')
    expect(source).toContain('.ui-button--secondary')
    expect(source).toContain('border-radius: var(--radius-lg)')
    expect(source).not.toContain('backdrop-filter')
  })

  it('defaults Badge to a square and keeps pill as an opt-in', async () => {
    const { default: Badge } = await import('@/components/ui/Badge.vue')
    const { el, unmount } = await mount(defineComponent({
      setup() {
        return () => h('div', [
          h(Badge, { label: 'mcp' }),
          h(Badge, { label: 'live', pill: true, dot: true }),
        ])
      },
    }))

    try {
      const badges = el.querySelectorAll('.ui-badge')
      expect(badges[0].classList.contains('ui-badge--square')).toBe(true)
      expect(badges[1].classList.contains('ui-badge--pill')).toBe(true)
    } finally {
      unmount()
    }
  })

  it('lets OpenCodePageShell wrap PageShell and PageHeader', async () => {
    const { default: OpenCodePageShell } = await import('@/components/opencode/OpenCodePageShell.vue')
    const { el, unmount } = await mount(OpenCodePageShell, {
      title: 'Providers',
      description: 'Manage providers',
      badge: 'provider',
    }, {
      default: () => 'content',
    })

    try {
      expect(el.querySelector('.page-shell')).toBeTruthy()
      expect(el.querySelector('.page-header__title')?.textContent).toContain('Providers')
      expect(el.querySelector('.page-header__eyebrow')?.getAttribute('lang')).toBe('en')
      expect(el.textContent).toContain('content')
      expect(el.querySelector('.opencode-page-shell__glow')).toBeNull()
    } finally {
      unmount()
    }
  })
})
