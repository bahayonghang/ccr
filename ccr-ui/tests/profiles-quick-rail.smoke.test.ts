import { createApp, defineComponent, h, nextTick, ref } from 'vue'
import { createI18n } from 'vue-i18n'
import { afterEach, describe, expect, it, vi } from 'vitest'
import zhCnMessages from '@/i18n/locales/zh-CN'

vi.mock('@/components/ui/SIcon.vue', () => ({
  default: defineComponent({
    props: {
      name: { type: String, required: true },
      size: { type: String, default: '' },
      class: { type: String, default: '' },
    },
    setup(props) {
      return () => h('span', { 'data-icon': props.name, class: [props.size, props.class] })
    },
  }),
}))

import ProfilesQuickRail, { type QuickRailProfile } from '@/components/profiles/ProfilesQuickRail.vue'
import { useProfilesQuickSwitch } from '@/composables/useProfilesQuickSwitch'

interface Mounted {
  el: HTMLElement
  unmount: () => void
}

const profiles: QuickRailProfile[] = [
  { name: 'alpha', enabled: true, description: 'Alpha relay' },
  { name: 'beta', enabled: true },
  { name: 'gamma', enabled: false },
  { name: 'delta', enabled: true },
]

const mountRail = (options: {
  quickSwitch?: ReturnType<typeof useProfilesQuickSwitch> | null
  moreCount?: number
  onApply?: (name: string) => void
  onMore?: () => void
}): Mounted => {
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(
    defineComponent({
      setup() {
        return () =>
          h(ProfilesQuickRail, {
            profiles,
            currentName: 'alpha',
            i18nPrefix: 'claudeProfiles',
            quickSwitch: options.quickSwitch ?? null,
            moreCount: options.moreCount ?? 0,
            onApply: options.onApply,
            onMore: options.onMore,
          })
      },
    }),
  )
  app.use(
    createI18n({
      legacy: false,
      locale: 'zh-CN',
      fallbackLocale: 'zh-CN',
      messages: { 'zh-CN': zhCnMessages },
    }),
  )
  app.mount(el)

  return {
    el,
    unmount: () => {
      app.unmount()
      el.remove()
    },
  }
}

const makeQuickSwitch = () => {
  const names = ref(profiles.map(profile => profile.name))
  const quickSwitch = useProfilesQuickSwitch({
    platform: 'rail-test',
    getProfileNames: () => names.value,
  })
  quickSwitch.pin('alpha')
  quickSwitch.pin('gamma')
  quickSwitch.recordUse('delta')
  quickSwitch.recordUse('beta')
  return quickSwitch
}

const switchChips = (el: HTMLElement) =>
  Array.from(el.querySelectorAll<HTMLButtonElement>('.cp-chip--switch'))

const pressKey = (el: HTMLElement, key: string) => {
  el.querySelector('.cp-rail__list')?.dispatchEvent(
    new KeyboardEvent('keydown', { key, bubbles: true }),
  )
}

describe('ProfilesQuickRail quickSwitch mode smoke', () => {
  let mounted: Mounted | null = null

  afterEach(() => {
    mounted?.unmount()
    mounted = null
  })

  it('numbers only pinned chips; recent chips stay unnumbered', () => {
    mounted = mountRail({ quickSwitch: makeQuickSwitch() })
    const chips = switchChips(mounted.el)

    // 钉选 alpha/gamma 在前（带 1/2 序号），recent beta/delta 在后（无序号）
    expect(chips.map(chip => chip.querySelector('.cp-chip__name')?.textContent)).toEqual([
      'alpha',
      'gamma',
      'beta',
      'delta',
    ])
    expect(chips[0].querySelector('.cp-chip__kbd')?.textContent).toBe('1')
    expect(chips[1].querySelector('.cp-chip__kbd')?.textContent).toBe('2')
    expect(chips[2].querySelector('.cp-chip__kbd')).toBeNull()
    expect(chips[3].querySelector('.cp-chip__kbd')).toBeNull()
    // 禁用的钉选 gamma 保留但置灰
    expect(chips[1].disabled).toBe(true)
  })

  it('keeps a single tab stop and roves focus with arrow/Home/End keys', async () => {
    mounted = mountRail({ quickSwitch: makeQuickSwitch() })
    const chips = switchChips(mounted.el)

    // 初始：仅第一个 chip 在 Tab 序中
    expect(chips.map(chip => chip.tabIndex)).toEqual([0, -1, -1, -1])
    // pin 按钮不参与 Tab 序
    for (const pin of mounted.el.querySelectorAll<HTMLButtonElement>('.cp-chip__pin')) {
      expect(pin.tabIndex).toBe(-1)
    }

    pressKey(mounted.el, 'ArrowRight')
    await nextTick()
    expect(chips.map(chip => chip.tabIndex)).toEqual([-1, 0, -1, -1])
    expect(document.activeElement).toBe(chips[1])

    pressKey(mounted.el, 'End')
    await nextTick()
    expect(chips.map(chip => chip.tabIndex)).toEqual([-1, -1, -1, 0])
    expect(document.activeElement).toBe(chips[3])

    pressKey(mounted.el, 'ArrowRight')
    await nextTick()
    // 右移到底后回绕到第一个
    expect(chips[0].tabIndex).toBe(0)

    pressKey(mounted.el, 'ArrowLeft')
    await nextTick()
    // 左移回绕到末尾
    expect(chips[3].tabIndex).toBe(0)

    pressKey(mounted.el, 'Home')
    await nextTick()
    expect(chips[0].tabIndex).toBe(0)
    expect(document.activeElement).toBe(chips[0])
  })

  it('renders the more entry and emits more when clicked', async () => {
    const onMore = vi.fn()
    mounted = mountRail({ quickSwitch: makeQuickSwitch(), moreCount: 12, onMore })

    const more = mounted.el.querySelector<HTMLButtonElement>('.cp-chip--more')
    expect(more).not.toBeNull()
    expect(more?.textContent).toContain('+12')

    more?.click()
    await nextTick()
    expect(onMore).toHaveBeenCalledTimes(1)
  })

  it('falls back to the legacy display-order rendering without quickSwitch', () => {
    mounted = mountRail({})
    const chips = Array.from(mounted.el.querySelectorAll<HTMLButtonElement>('.cp-chip'))

    // 旧行为：启用 profile 按顺序编号（禁用 gamma 不出现）
    expect(chips.map(chip => chip.querySelector('.cp-chip__name')?.textContent)).toEqual([
      'alpha',
      'beta',
      'delta',
    ])
    expect(chips[0].querySelector('.cp-chip__kbd')?.textContent).toBe('1')
    expect(chips[1].querySelector('.cp-chip__kbd')?.textContent).toBe('2')
    expect(chips[2].querySelector('.cp-chip__kbd')?.textContent).toBe('3')
    // 旧模式没有 toolbar 角色与 more 入口
    expect(mounted.el.querySelector('[role="toolbar"]')).toBeNull()
    expect(mounted.el.querySelector('.cp-chip--more')).toBeNull()
  })
})
