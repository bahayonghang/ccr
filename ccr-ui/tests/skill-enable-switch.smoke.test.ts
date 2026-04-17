import { createApp, h, nextTick, reactive } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import SkillEnableSwitch from '@/components/skills/SkillEnableSwitch.vue'
import { createI18nStub } from './helpers/i18n-stub'

function mount(component: () => ReturnType<typeof h>): { container: HTMLElement, app: ReturnType<typeof createApp> } {
  const container = document.createElement('div')
  const app = createApp({ render: component })
  app.use(createI18nStub())
  app.mount(container)
  return { container, app }
}

afterEach(() => {
  vi.clearAllMocks()
})

describe('SkillEnableSwitch', () => {
  it('renders Enabled label when skill not in disabled set', async () => {
    const disabledSet = new Set<string>()
    const { container, app } = mount(() =>
      h(SkillEnableSwitch, { disabledSet, skillName: 'alpha' }),
    )
    await nextTick()
    expect(container.textContent).toContain('Enabled')
    app.unmount()
  })

  it('renders Disabled label when skill is in disabled set', async () => {
    const disabledSet = new Set<string>(['alpha'])
    const { container, app } = mount(() =>
      h(SkillEnableSwitch, { disabledSet, skillName: 'alpha' }),
    )
    await nextTick()
    expect(container.textContent).toContain('Disabled')
    app.unmount()
  })

  it('emits toggle event on checkbox change', async () => {
    const disabledSet = new Set<string>()
    const handler = vi.fn()
    const { container, app } = mount(() =>
      h(SkillEnableSwitch, { disabledSet, skillName: 'beta', onToggle: handler }),
    )
    await nextTick()

    const input = container.querySelector('input[type="checkbox"]') as HTMLInputElement
    expect(input).toBeTruthy()
    input.checked = false
    input.dispatchEvent(new Event('change'))
    await nextTick()

    expect(handler).toHaveBeenCalledWith('beta', false)
    app.unmount()
  })

  it('is reactive to external disabled set mutations', async () => {
    const state = reactive({ disabled: new Set<string>() })
    const { container, app } = mount(() =>
      h(SkillEnableSwitch, { disabledSet: state.disabled, skillName: 'gamma' }),
    )
    await nextTick()
    expect(container.textContent).toContain('Enabled')

    state.disabled = new Set(['gamma'])
    await nextTick()
    expect(container.textContent).toContain('Disabled')
    app.unmount()
  })
})
