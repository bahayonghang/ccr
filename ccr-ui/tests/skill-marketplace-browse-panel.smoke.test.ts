import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick, ref } from 'vue'
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

vi.mock('@/components/skills/MarketplacePagination.vue', () => ({
  default: defineComponent({
    name: 'MarketplacePaginationStub',
    template: '<div data-testid="marketplace-pagination" />',
  }),
}))

vi.mock('@/components/skills/MarketplaceSkillCard.vue', () => ({
  default: defineComponent({
    name: 'MarketplaceSkillCardStub',
    props: {
      item: { type: Object, required: true },
      installDisabled: { type: Boolean, default: false },
    },
    emits: ['install', 'toggle-batch', 'view-detail'],
    template: `
      <div data-testid="marketplace-card">
        <span>{{ item.package }}</span>
        <span v-if="installDisabled">disabled</span>
      </div>
    `,
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

const items: MarketplaceItem[] = [
  {
    package: 'owner/alpha',
    owner: 'owner',
    repo: 'alpha',
    skill: 'Alpha',
    skillsShUrl: 'https://skills.sh/owner/alpha',
    description: 'Alpha skill',
    stars: 10,
  },
]

const mountPanel = async (props: Record<string, unknown>) => {
  const { default: SkillMarketplaceBrowsePanel } = await import('@/components/skills/SkillMarketplaceBrowsePanel.vue')
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(defineComponent({
    setup() {
      const searchQuery = ref((props.searchQuery as string) ?? '')
      return () => h(SkillMarketplaceBrowsePanel, {
        batchMode: false,
        batchSelectedCount: 0,
        contentMode: 'trending',
        contentState: 'ready',
        currentPage: 1,
        hasDetectedPlatforms: true,
        isBatchSelected: () => false,
        isInstalling: () => false,
        isMarketplaceLoading: false,
        isRefreshing: false,
        isSkillInstalled: () => false,
        marketplaceCached: false,
        marketplaceError: null,
        marketplaceItems: items,
        noPlatformHint: 'Detect at least one supported CLI platform before installing or importing skills.',
        pageSize: 20,
        pagedItems: items,
        searchQuery: searchQuery.value,
        sortBy: 'stars',
        sortedItems: items,
        'onUpdate:searchQuery': (value: string) => {
          searchQuery.value = value
        },
        ...props,
      })
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

describe('SkillMarketplaceBrowsePanel smoke', () => {
  it('shows trending header when search is empty', async () => {
    const { el, unmount } = await mountPanel({
      contentMode: 'trending',
      searchQuery: '',
    })

    try {
      expect(el.querySelector('[data-testid="marketplace-title"]')?.textContent).toContain('Trending')
      expect(el.querySelector('[data-testid="marketplace-hint"]')?.textContent).toContain('Popular skills from skills.sh')
    } finally {
      unmount()
    }
  })

  it('shows search result header when search query exists', async () => {
    const { el, unmount } = await mountPanel({
      contentMode: 'search',
      searchQuery: 'alpha',
      sortedItems: items,
      pagedItems: items,
    })

    try {
      expect(el.querySelector('[data-testid="marketplace-title"]')?.textContent).toContain('Search results for “alpha”')
      expect(el.querySelector('[data-testid="marketplace-result-badge"]')?.textContent).toContain('1 results')
    } finally {
      unmount()
    }
  })

  it('shows batch bar only when items are selected', async () => {
    const hidden = await mountPanel({
      batchSelectedCount: 0,
    })

    try {
      expect(hidden.el.querySelector('[data-testid="marketplace-batch-bar"]')).toBeNull()
    } finally {
      hidden.unmount()
    }

    const shown = await mountPanel({
      batchSelectedCount: 2,
    })

    try {
      expect(shown.el.querySelector('[data-testid="marketplace-batch-bar"]')?.textContent).toContain('2 selected')
    } finally {
      shown.unmount()
    }
  })

  it('shows no-platform blocking state and disables card installs', async () => {
    const { el, unmount } = await mountPanel({
      hasDetectedPlatforms: false,
      batchSelectedCount: 1,
    })

    try {
      expect(el.querySelector('[data-testid="no-platform-blocking"]')?.textContent).toContain('No compatible platforms detected')
      expect(el.querySelector('[data-testid="marketplace-card"]')?.textContent).toContain('disabled')
      expect(el.querySelector('[data-testid="marketplace-batch-bar"]')?.textContent).toContain('No platforms detected')
    } finally {
      unmount()
    }
  })
})
