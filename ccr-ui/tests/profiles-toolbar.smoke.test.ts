import { createApp, defineComponent, h, nextTick } from 'vue'
import { createI18n } from 'vue-i18n'
import { afterEach, describe, expect, it, vi } from 'vitest'

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

import ProfilesToolbar from '@/components/profiles/ProfilesToolbar.vue'

const messages = {
  searchPlaceholder: 'Search profiles',
  statusGroupLabel: 'Status',
  statusAll: 'All',
  statusActive: 'Current',
  statusEnabled: 'Enabled',
  statusDisabled: 'Disabled',
  filtersButton: 'Filters',
  tagGroupLabel: 'Tags',
  providerLabel: 'Provider',
  providerAll: 'All providers',
  sortLabel: 'Sort',
  sortRecent: 'Recent',
  sortName: 'Name',
  sortRequests: 'Requests',
  sortEnabled: 'Enabled first',
  clearAll: 'Clear all',
  viewLabel: 'View',
  viewCard: 'Cards',
  viewList: 'List',
}

describe('ProfilesToolbar filters keyboard contract', () => {
  let unmount: (() => void) | null = null

  afterEach(() => {
    unmount?.()
    unmount = null
  })

  it('moves between button options with arrow keys and restores trigger focus on Escape', async () => {
    const el = document.createElement('div')
    document.body.appendChild(el)
    const app = createApp(
      defineComponent({
        setup() {
          return () => h(ProfilesToolbar, {
            query: '',
            statusFilter: 'all',
            tagFilter: null,
            sortBy: 'recent',
            viewMode: 'card',
            resultCount: 2,
            total: 2,
            allTags: ['alpha', 'beta'],
            i18nPrefix: 'toolbar',
          })
        },
      }),
    )
    app.use(createI18n({
      legacy: false,
      locale: 'en-US',
      messages: { 'en-US': { toolbar: messages } },
    }))
    app.mount(el)
    unmount = () => {
      app.unmount()
      el.remove()
    }

    const trigger = el.querySelector<HTMLButtonElement>('.cp-filters__trigger')!
    trigger.click()
    await nextTick()

    const optionButtons = Array.from(
      el.querySelectorAll<HTMLButtonElement>('.cp-filters__section .cp-pill'),
    )
    expect(document.activeElement).toBe(optionButtons[0])

    optionButtons[0].dispatchEvent(new KeyboardEvent('keydown', {
      key: 'ArrowRight',
      bubbles: true,
    }))
    expect(document.activeElement).toBe(optionButtons[1])

    optionButtons[1].dispatchEvent(new KeyboardEvent('keydown', {
      key: 'Escape',
      bubbles: true,
    }))
    await nextTick()
    expect(el.querySelector('.cp-filters__pop')).toBeNull()
    expect(document.activeElement).toBe(trigger)
  })
})
