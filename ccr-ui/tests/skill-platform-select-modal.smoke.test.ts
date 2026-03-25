import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, h, nextTick, ref } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import enUS from '@/i18n/locales/en-US'
import type { MarketplaceItem, PlatformSummary } from '@/types/skills'

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

const platforms: PlatformSummary[] = [
  {
    id: 'claude',
    display_name: 'Claude Code',
    global_skills_dir: '/tmp/claude',
    detected: true,
    installed_count: 0,
  },
  {
    id: 'codex',
    display_name: 'Codex',
    global_skills_dir: '/tmp/codex',
    detected: false,
    installed_count: 0,
  },
]

const pendingItem: MarketplaceItem = {
  package: 'owner/repo',
  owner: 'owner',
  repo: 'repo',
  skill: 'Repo Skill',
  skillsShUrl: 'https://skills.sh/owner/repo',
  description: 'A useful skill',
}

const mountModal = async (props: Record<string, unknown>) => {
  const { default: SkillPlatformSelectModal } = await import('@/components/skills/SkillPlatformSelectModal.vue')
  const el = document.createElement('div')
  document.body.appendChild(el)

  const selectedPlatforms = ref<string[]>(['claude'])
  const updateSelectedPlatforms = vi.fn((value: string[]) => {
    selectedPlatforms.value = value
  })

  const app = createApp(defineComponent({
    setup() {
      return () => h(SkillPlatformSelectModal, {
        show: true,
        mode: 'single',
        pendingItem,
        batchPackages: [],
        platforms,
        selectedPlatforms: selectedPlatforms.value,
        closeModal: vi.fn(),
        selectDetected: vi.fn(),
        updateSelectedPlatforms,
        confirmInstall: vi.fn(),
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
    updateSelectedPlatforms,
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

describe('SkillPlatformSelectModal smoke', () => {
  it('renders translated single-install copy for the pending skill', async () => {
    const { unmount } = await mountModal({
      mode: 'single',
      pendingItem,
      batchPackages: [],
    })

    try {
      const overlay = document.body.querySelector('.platform-modal-overlay')
      expect(overlay?.textContent).toContain('Install Skill')
      expect(overlay?.textContent).toContain('Repo Skill')
      expect(overlay?.textContent).toContain('owner/repo')
      expect(overlay?.textContent).toContain('Select target platforms')
      expect(overlay?.textContent).toContain('Install to 1 platforms')
    } finally {
      unmount()
    }
  })

  it('renders translated batch-install summary and action label', async () => {
    const { unmount } = await mountModal({
      mode: 'batch',
      pendingItem: null,
      batchPackages: ['owner/one', 'owner/two'],
      selectedPlatforms: ['claude'],
    })

    try {
      const overlay = document.body.querySelector('.platform-modal-overlay')
      expect(overlay?.textContent).toContain('Batch Install')
      expect(overlay?.textContent).toContain('Install 2 selected skills')
      expect(overlay?.textContent).toContain('owner/one')
      expect(overlay?.textContent).toContain('owner/two')
      expect(overlay?.textContent).toContain('Install to 1 platforms')
    } finally {
      unmount()
    }
  })
})
