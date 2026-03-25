import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import enUS from '@/i18n/locales/en-US'
import type { MarketplaceItem } from '@/types/skills'

vi.mock('@/components/ui/SIcon.vue', () => ({
  default: defineComponent({
    name: 'SIconStub',
    props: {
      name: { type: String, required: true },
      size: { type: String, default: '' },
    },
    setup(props) {
      return () => h('span', { 'data-icon': props.name, class: props.size })
    },
  }),
}))

const i18n = createI18n({
  legacy: false,
  locale: 'en-US',
  fallbackLocale: 'en-US',
  missingWarn: false,
  fallbackWarn: false,
  messages: {
    'en-US': enUS,
  },
})

const marketplaceItem: MarketplaceItem = {
  package: 'owner/repo',
  owner: 'owner',
  repo: 'repo',
  skill: 'Repo Skill',
  skillsShUrl: 'https://skills.sh/owner/repo',
  description: 'A useful skill',
  stars: 1200,
}

const mountModal = async (props: {
  show: boolean
  item: MarketplaceItem | null
  isInstalled: boolean
  installDisabled: boolean
}) => {
  const { default: SkillMarketplaceDetailModal } = await import('@/components/skills/SkillMarketplaceDetailModal.vue')
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      return () => h(SkillMarketplaceDetailModal, props)
    },
  }))

  app.use(i18n)
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
  document.body.innerHTML = ''
  vi.clearAllMocks()
})

describe('SkillMarketplaceDetailModal smoke', () => {
  it('does not render the detail card body when no item is selected', async () => {
    const { unmount } = await mountModal({
      show: true,
      item: null,
      isInstalled: false,
      installDisabled: true,
    })

    try {
      expect(document.body.querySelector('.marketplace-detail-overlay')).not.toBeNull()
      expect(document.body.querySelector('.marketplace-detail-modal')).toBeNull()
    } finally {
      unmount()
    }
  })

  it('renders selected item details and disables install when requested', async () => {
    const { unmount } = await mountModal({
      show: true,
      item: marketplaceItem,
      isInstalled: false,
      installDisabled: true,
    })

    try {
      const modal = document.body.querySelector('.marketplace-detail-modal')
      expect(modal?.textContent).toContain('Marketplace Detail')
      expect(modal?.textContent).toContain('Repo Skill')
      expect(modal?.textContent).toContain('owner/repo')
      expect(modal?.textContent).toContain('Not installed yet')
      expect(modal?.textContent).toContain('Description')
      const installButton = Array.from(document.body.querySelectorAll('button')).find(
        button => button.textContent?.includes('Install'),
      )
      expect(installButton?.hasAttribute('disabled')).toBe(true)
    } finally {
      unmount()
    }
  })
})
