import { createI18n } from 'vue-i18n'
import { createApp, defineComponent, nextTick, ref } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'
import enUS from '@/i18n/locales/en-US'

const unifiedSkillsMock = {
  initialize: vi.fn(),
  refresh: vi.fn(),
  loadMarketplace: vi.fn(),
  loadNpxStatus: vi.fn(),
  loadOnboardingCandidates: vi.fn(async () => [
    {
      skillId: 'sg_skill_alpha',
      name: 'Skill Alpha',
      platformIds: ['codex'],
      installationIds: ['ins_skill_alpha'],
      installationPaths: ['/tmp/skill-alpha'],
      reason: 'missing_source',
    },
  ]),
  applyRouteState: vi.fn(),
  addGitSource: vi.fn(),
  addLocalSourceRecord: vi.fn(),
  importFromLocal: vi.fn(async () => undefined),
  browseFolder: vi.fn(),
  platforms: ref([{ id: 'codex', displayName: 'Codex', detected: true, installedCount: 1 }]),
  sources: ref([]),
  marketplace: ref({ total: 0 }),
  onboardingCandidates: ref([
    {
      skillId: 'sg_skill_alpha',
      name: 'Skill Alpha',
      platformIds: ['codex'],
      installationIds: ['ins_skill_alpha'],
      installationPaths: ['/tmp/skill-alpha'],
      reason: 'missing_source',
    },
  ]),
  filters: ref({ search: '', platform: 'all', origin: 'all', category: null, tags: [], source: 'all' }),
  routeState: ref({ tab: 'inventory', selected: null, mode: 'view', platform: 'all', origin: 'all', q: '', page: 1, source: null }),
  operationLog: ref([]),
  workflowState: ref({ action: 'idle', target: '', status: 'idle' }),
  mutationLoading: ref(false),
  availableCategories: ref([]),
  availableTags: ref([]),
  stats: ref({ logicalSkills: 1, sources: 0, installations: 1 }),
  inventoryLoading: ref(false),
  sourcesLoading: ref(false),
  marketplaceLoading: ref(false),
  selectSkill: vi.fn(),
}

vi.mock('@/composables/useUnifiedSkills', () => ({
  useUnifiedSkills: () => unifiedSkillsMock,
}))

vi.mock('@/components/PageHeaderCard.vue', () => ({ default: defineComponent({ template: '<div><slot /><slot name="actions" /></div>' }) }))
vi.mock('@/components/ui/AsyncStatePanel.vue', () => ({ default: defineComponent({ template: '<div />' }) }))
vi.mock('@/components/ui/SIcon.vue', () => ({ default: defineComponent({ template: '<span />' }) }))
vi.mock('@/components/skills/PlatformSelector.vue', () => ({ default: defineComponent({ template: '<div />' }) }))
vi.mock('@/components/skills/ActivityLog.vue', () => ({ default: defineComponent({ template: '<div />' }) }))
vi.mock('@/components/skills/InventoryPanel.vue', () => ({ default: defineComponent({ template: '<div />' }) }))
vi.mock('@/components/skills/SourcesPanel.vue', () => ({ default: defineComponent({ template: '<div />' }) }))
vi.mock('@/components/skills/MarketplacePanel.vue', () => ({ default: defineComponent({ template: '<div />' }) }))
vi.mock('@/stores/ui', () => ({ useUIStore: () => ({ showSuccess: vi.fn(), showError: vi.fn() }) }))
vi.mock('@/utils/runtimeState', () => ({ getRuntimeUnavailableCopy: () => ({ title: '', description: '', actionLabel: '' }) }))
vi.mock('@/utils/tauriRuntime', () => ({ isTauriRuntime: () => true }))
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => () => {}),
}))
vi.mock('vue-router', () => ({
  useRoute: () => ({ query: {} }),
  useRouter: () => ({ replace: vi.fn() }),
}))

const i18n = createI18n({
  legacy: false,
  locale: 'en-US',
  fallbackLocale: 'en-US',
  missingWarn: false,
  fallbackWarn: false,
  messages: { 'en-US': enUS },
})

const mountView = async () => {
  const { default: SkillsView } = await import('@/views/skills/SkillsView.vue')
  const el = document.createElement('div')
  document.body.appendChild(el)

  const app = createApp(SkillsView)
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

describe('SkillsView onboarding smoke', () => {
  it('triggers import flow from onboarding candidate action', async () => {
    const { el, unmount } = await mountView()

    try {
      const button = el.querySelector('[data-testid="onboarding-import"]') as HTMLButtonElement | null
      expect(button).toBeTruthy()
      button?.click()
      await nextTick()
      expect(unifiedSkillsMock.importFromLocal).toHaveBeenCalledTimes(1)
    } finally {
      unmount()
    }
  })
})
