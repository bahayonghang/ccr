<template>
  <div class="flex flex-col gap-5 px-4 py-4">
    <PageHeaderCard
      v-if="!hidePageHeader"
      :title="pageTitleResolved"
      :description="pageDescriptionResolved"
      :badge="pageBadgeResolved"
      icon="Package"
      tone="secondary"
    >
      <template #actions>
        <button
          class="rounded-xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
          :disabled="inventoryLoading || sourcesLoading || marketplaceLoading"
          @click="handleRefresh"
        >
          <SIcon
            name="RefreshCw"
            size="w-4 h-4"
            :class="{ 'animate-spin': inventoryLoading || sourcesLoading || marketplaceLoading }"
          />
        </button>
      </template>
      <div class="grid gap-3 pt-4 md:grid-cols-3">
        <div class="rounded-2xl border border-border-default/45 bg-bg-base/40 p-4">
          <div class="text-[11px] uppercase tracking-[0.16em] text-text-muted">
            Logical skills
          </div>
          <div class="mt-2 text-2xl font-semibold text-text-primary">
            {{ stats.logicalSkills }}
          </div>
        </div>
        <div class="rounded-2xl border border-border-default/45 bg-bg-base/40 p-4">
          <div class="text-[11px] uppercase tracking-[0.16em] text-text-muted">
            Tracked sources
          </div>
          <div class="mt-2 text-2xl font-semibold text-text-primary">
            {{ stats.sources }}
          </div>
        </div>
        <div class="rounded-2xl border border-border-default/45 bg-bg-base/40 p-4">
          <div class="text-[11px] uppercase tracking-[0.16em] text-text-muted">
            Installations
          </div>
          <div class="mt-2 text-2xl font-semibold text-text-primary">
            {{ stats.installations }}
          </div>
        </div>
      </div>
    </PageHeaderCard>
    <AsyncStatePanel
      v-if="runtimeUnavailable"
      state="runtime-unavailable"
      :title="runtimeCopy.title"
      :description="runtimeCopy.description"
      :action-label="runtimeCopy.actionLabel"
      action-icon="ArrowLeft"
      @action="$router.push('/')"
    />
    <template v-else>
      <div
        v-if="primaryLoadError"
        class="rounded-2xl border border-danger/30 bg-danger/10 px-4 py-3 text-sm text-danger"
      >
        Skills inventory failed to load: {{ primaryLoadError }}
      </div>
      <div
        v-if="npxWarning"
        class="rounded-2xl border border-warning/30 bg-warning/10 px-4 py-3 text-sm text-text-primary"
      >
        npx capability probe failed, but the page remains usable: {{ npxWarning }}
      </div>
      <div class="flex flex-wrap gap-2 rounded-3xl border border-border-default/55 bg-bg-elevated/60 p-2">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="inline-flex items-center gap-2 rounded-2xl px-4 py-2.5 text-sm"
          :class="activeTab===tab.id?'border border-accent-primary/30 bg-accent-primary/10 text-text-primary':'text-text-secondary'"
          @click="setTab(tab.id)"
        >
          <SIcon
            :name="tab.icon"
            size="w-4 h-4"
          /><span>{{ tab.label }}</span><strong class="rounded-full bg-bg-base/70 px-2 py-0.5 text-xs">{{ tab.count }}</strong>
        </button>
      </div>

      <section
        v-if="activeTab==='library'"
        class="grid gap-4 xl:grid-cols-[320px_minmax(0,1fr)]"
      >
        <aside class="flex flex-col gap-4">
          <div class="rounded-3xl border border-border-default/55 bg-bg-elevated/60 p-4">
            <div class="mb-3 flex items-center justify-between">
              <h2 class="text-xs uppercase tracking-[0.16em] text-text-muted">
                Filters
              </h2><button
                class="text-xs text-text-muted"
                @click="resetLibraryFilters"
              >
                Reset
              </button>
            </div>
            <div class="flex flex-col gap-3">
              <input
                v-model="librarySearch"
                aria-label="Search installed skills"
                class="rounded-2xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
                placeholder="search"
              >
              <select
                v-model="filters.platform"
                aria-label="Filter skills by platform"
                class="rounded-2xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
                :disabled="Boolean(props.forcedPlatform)"
              >
                <option value="all">
                  All platforms
                </option><option
                  v-for="platform in platforms"
                  :key="platform.id"
                  :value="platform.id"
                >
                  {{ platform.displayName }}
                </option>
              </select>
              <select
                v-model="filters.origin"
                aria-label="Filter skills by origin"
                class="rounded-2xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
              >
                <option value="all">
                  All origins
                </option><option
                  v-for="origin in originOptions"
                  :key="origin"
                  :value="origin"
                >
                  {{ origin }}
                </option>
              </select>
            </div>
          </div>
          <PlatformSelector
            v-model="selectedPlatforms"
            :platforms="platforms"
            @select-detected="selectDetectedPlatforms"
          />
          <div
            v-if="(onboardingCandidates || []).length"
            class="rounded-3xl border border-border-default/55 bg-bg-elevated/60 p-4"
          >
            <div class="mb-3 flex items-center justify-between">
              <h2 class="text-xs uppercase tracking-[0.16em] text-text-muted">
                Onboarding
              </h2><span class="text-xs text-text-muted">{{ onboardingCandidates.length }}</span>
            </div>
            <div
              v-for="candidate in onboardingCandidates"
              :key="candidate.skillId"
              class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-3"
            >
              <div class="mb-2">
                <strong class="text-sm text-text-primary">{{ candidate.name }}</strong><p class="text-xs text-text-muted">
                  {{ candidate.reason }}
                </p>
              </div>
              <button
                data-testid="onboarding-import"
                class="rounded-xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
                @click="importOnboardingCandidate(candidate)"
              >
                Import
              </button>
            </div>
          </div>
          <div
            v-else-if="sources.length === 0"
            class="rounded-3xl border border-border-default/55 bg-bg-elevated/60 p-4 text-sm text-text-secondary"
          >
            <div class="mb-2 text-xs uppercase tracking-[0.16em] text-text-muted">
              Source Tracking
            </div>
            <p>已跟踪来源仓库为 0；当前 Library 仍会扫描各平台已安装 skills。</p>
          </div>
          <ActivityLog :entries="operationLog" />
        </aside>
        <InventoryPanel
          :selected-platforms="selectedPlatforms"
          @select="handleInventorySelect"
        />
      </section>

      <section
        v-else-if="activeTab==='explore'"
        class="grid gap-4"
      >
        <MarketplacePanel
          :selected-platforms="selectedPlatforms"
          :search-initial="exploreQuery"
          :page="routeState.page"
          @update:page="updateMarketplacePage"
          @update:search="updateMarketplaceSearch"
        />
      </section>

      <section
        v-else-if="activeTab==='platforms'"
        class="grid gap-4 md:grid-cols-2 2xl:grid-cols-3"
      >
        <article
          v-for="platform in platforms"
          :key="platform.id"
          class="rounded-3xl border border-border-default/55 bg-bg-elevated/60 p-4"
        >
          <div class="mb-4 flex items-start justify-between gap-3">
            <div>
              <h2 class="text-base font-semibold text-text-primary">
                {{ platform.displayName }}
              </h2><p class="text-xs text-text-muted">
                {{ platform.id }}
              </p>
            </div><span
              class="rounded-full px-2 py-0.5 text-[11px]"
              :class="platform.detected ? 'bg-success/10 text-success' : 'bg-danger/10 text-danger'"
            >{{ platform.detected ? 'Detected' : 'Missing' }}</span>
          </div><div class="grid gap-3">
            <div class="rounded-2xl bg-bg-base/35 p-3">
              <span class="block text-[11px] uppercase tracking-[0.14em] text-text-muted">Skills dir</span><strong class="mt-1 block break-all text-sm text-text-primary">{{ platform.globalSkillsDir }}</strong>
            </div><div class="rounded-2xl bg-bg-base/35 p-3">
              <span class="block text-[11px] uppercase tracking-[0.14em] text-text-muted">Install strategy</span><strong class="mt-1 block text-sm text-text-primary">{{ platform.installStrategy || 'managedcopy' }}</strong>
            </div><div class="rounded-2xl bg-bg-base/35 p-3">
              <span class="block text-[11px] uppercase tracking-[0.14em] text-text-muted">npx agent key</span><strong class="mt-1 block text-sm text-text-primary">{{ platform.npxAgentKey || 'fallback' }}</strong>
            </div>
          </div>
        </article>
      </section>

      <section
        v-else
        class="flex flex-col gap-4"
      >
        <SourcesPanel :selected-platforms="selectedPlatforms" />
      </section>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import ActivityLog from '@/components/skills/ActivityLog.vue'
import InventoryPanel from '@/components/skills/InventoryPanel.vue'
import MarketplacePanel from '@/components/skills/MarketplacePanel.vue'
import PlatformSelector from '@/components/skills/PlatformSelector.vue'
import SourcesPanel from '@/components/skills/SourcesPanel.vue'
import AsyncStatePanel from '@/components/ui/AsyncStatePanel.vue'
import PageHeaderCard from '@/components/PageHeaderCard.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { useUIStore } from '@/stores/ui'
import { useUnifiedSkills } from '@/composables/useUnifiedSkills'
import type { Platform, SkillOrigin, SkillsTab } from '@/types/skills'
import { getRuntimeUnavailableCopy } from '@/utils/runtimeState'
import { handleSkillsChangedPayload } from './skillsWatcher'
import { isTauriRuntime } from '@/utils/tauriRuntime'

const props = withDefaults(defineProps<{
  forcedPlatform?: Platform | null
  routeBasePath?: string
  pageTitle?: string
  pageDescription?: string
  pageBadge?: string
  hidePageHeader?: boolean
}>(), {
  forcedPlatform: null,
  routeBasePath: '/skills',
  pageTitle: 'Skills Hub',
  pageDescription: '统一管理 Skills 库存、探索、平台和来源。',
  pageBadge: 'Workspace',
  hidePageHeader: false,
})

const route = useRoute()
const router = useRouter()
const uiStore = useUIStore()
const {
  initialize,
  refresh,
  loadMarketplace,
  loadOnboardingCandidates,
  loadNpxCapabilities,
  importFromLocal,
  selectSkill,
  platforms,
  sources,
  marketplace,
  onboardingCandidates,
  operationLog,
  stats,
  filters,
  routeState,
  marketplaceLoading,
  inventoryLoading,
  sourcesLoading,
} = useUnifiedSkills()
const runtimeUnavailable = computed(() => !isTauriRuntime())
const runtimeCopy = computed(() => getRuntimeUnavailableCopy('skills'))
const pageTitleResolved = computed(() => props.pageTitle)
const pageDescriptionResolved = computed(() => {
  if (sources.value.length === 0) {
    return `${props.pageDescription} 已跟踪来源仓库为 0；当前仍会扫描各平台已安装 skills。`
  }
  return props.pageDescription
})
const pageBadgeResolved = computed(() => props.pageBadge)
const activeTab = computed<SkillsTab>(() => routeState.value.tab || 'library')
const tabs = computed(() => [{ id: 'library' as SkillsTab, label: 'Library', icon: 'LibraryBig', count: stats.value.logicalSkills }, { id: 'explore' as SkillsTab, label: 'Explore', icon: 'Store', count: marketplace.value.total }, { id: 'platforms' as SkillsTab, label: 'Platforms', icon: 'Cpu', count: platforms.value.length }, { id: 'sources' as SkillsTab, label: 'Sources', icon: 'FolderGit2', count: sources.value.length }])
const originOptions: SkillOrigin[] = ['marketplace', 'github', 'repo', 'local', 'npx', 'unknown']
const selectedPlatforms = ref<Platform[]>([])
const librarySearch = ref('')
const exploreQuery = ref('')
const primaryLoadError = ref<string | null>(null)
const npxWarning = ref<string | null>(null)
let stopSkillsEvent: null | (() => void) = null
let searchTimer = 0

function normalizeRouteState(query: Record<string, unknown>) {
  const incomingPlatform = typeof query.platform === 'string' ? query.platform : null
  const forcedPlatform = props.forcedPlatform
  const platform = (forcedPlatform ?? incomingPlatform ?? 'all') as Platform | 'all'
  return {
    tab: (query.tab === 'explore' || query.tab === 'platforms' || query.tab === 'sources' ? query.tab : 'library') as SkillsTab,
    selected: typeof query.selected === 'string' ? query.selected : null,
    mode: 'view' as const,
    platform,
    origin: (typeof query.origin === 'string' ? query.origin : 'all') as SkillOrigin | 'all',
    q: typeof query.q === 'string' ? query.q : '',
    page: 1,
    source: null,
  }
}
function syncRoute(extra: Record<string, string | null>) {
  const next: Record<string, string> = {}
  const merged = { ...route.query, ...extra }
  if (props.forcedPlatform) merged.platform = props.forcedPlatform
  for (const [key, value] of Object.entries(merged)) {
    if (typeof value === 'string' && value.trim()) next[key] = value
  }
  void router.replace({ path: props.routeBasePath, query: next })
}
function setTab(tab: SkillsTab) { routeState.value.tab = tab; syncRoute({ tab: tab === 'library' ? null : tab }); if (tab === 'explore') void reloadMarketplace(false) }
function resetLibraryFilters() {
  librarySearch.value = ''
  filters.value.search = ''
  filters.value.platform = props.forcedPlatform ?? 'all'
  filters.value.origin = 'all'
  filters.value.source = 'all'
  filters.value.tags = []
  syncRoute({ q: null, platform: props.forcedPlatform ?? null, origin: null })
}
function selectDetectedPlatforms() {
  if (props.forcedPlatform) {
    selectedPlatforms.value = platforms.value
      .filter((item) => item.detected && item.id === props.forcedPlatform)
      .map((item) => item.id)
    return
  }
  selectedPlatforms.value = platforms.value.filter((item) => item.detected).map((item) => item.id)
}
async function importOnboardingCandidate(candidate: { name: string; platformIds: string[]; installationPaths: string[] }) { try { await importFromLocal({ sourcePath: candidate.installationPaths[0], agents: candidate.platformIds, skillName: candidate.name }); uiStore.showSuccess(`Imported ${candidate.name}`) } catch (error) { uiStore.showError(error instanceof Error ? error.message : String(error)) } }
async function reloadMarketplace(force: boolean) { routeState.value.q = exploreQuery.value.trim(); syncRoute({ q: routeState.value.q || null, tab: 'explore' }); await loadMarketplace(force) }
function handleInventorySelect(skillId: string | null) {
  syncRoute({ selected: skillId })
}

function updateMarketplaceSearch(query: string) {
  exploreQuery.value = query
  routeState.value.q = query
  routeState.value.page = 1
  syncRoute({ q: query || null, tab: 'explore' })
  void loadMarketplace(true)
}

function updateMarketplacePage(page: number) {
  routeState.value.page = page
  syncRoute({ tab: 'explore', q: routeState.value.q || null })
  void loadMarketplace(true)
}

watch(() => route.query, (query) => {
  const normalized = normalizeRouteState(query as Record<string, unknown>)
  routeState.value = normalized
  filters.value.platform = normalized.platform
  filters.value.origin = normalized.origin as SkillOrigin | 'all'
  filters.value.search = normalized.q
  librarySearch.value = normalized.q
  exploreQuery.value = normalized.q
  selectSkill(normalized.selected, null)
}, { immediate: true })
watch(librarySearch, (value) => { window.clearTimeout(searchTimer); searchTimer = window.setTimeout(() => { filters.value.search = value.trim(); syncRoute({ q: value.trim() || null }) }, 250) })
watch(() => filters.value.platform, (value) => {
  syncRoute({ platform: value === 'all' ? props.forcedPlatform ?? null : value })
})
watch(() => filters.value.origin, (value) => {
  syncRoute({ origin: value && value !== 'all' ? value : null })
})

async function handleRefresh() {
  primaryLoadError.value = null
  try {
    await refresh(activeTab.value === 'explore')
    void loadDeferredState()
  } catch (error) {
    primaryLoadError.value = error instanceof Error ? error.message : String(error)
  }
}

async function loadDeferredState() {
  try {
    await loadNpxCapabilities?.(true)
    npxWarning.value = null
  } catch (error) {
    npxWarning.value = error instanceof Error ? error.message : String(error)
  }
}

onMounted(async () => {
  if (runtimeUnavailable.value) return
  try {
    await initialize(activeTab.value === 'explore')
    primaryLoadError.value = null
  } catch (error) {
    primaryLoadError.value = error instanceof Error ? error.message : String(error)
  }
  if (selectedPlatforms.value.length === 0) selectDetectedPlatforms()
  void loadDeferredState()
  if (isTauriRuntime()) {
    const { listen } = await import('@tauri-apps/api/event')
    stopSkillsEvent = await listen('skills-changed', async (event) => {
      await handleSkillsChangedPayload(
        event.payload as { affectsInventory?: boolean; affectsSources?: boolean; affectsMarketplace?: boolean },
        { currentTab: activeTab.value, loadOnboardingCandidates, refresh }
      )
    })
  }
})
onUnmounted(() => { stopSkillsEvent?.(); stopSkillsEvent = null })
</script>
