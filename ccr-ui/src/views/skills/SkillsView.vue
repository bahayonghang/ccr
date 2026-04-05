<template>
  <div class="skills-console">
    <PageHeaderCard
      title="Skills 管理中心"
      description="跨平台 AI CLI 工具的统一技能库存、来源和安装工作台。"
      badge="Workspace"
      icon="Package"
      tone="secondary"
    >
      <template #actions>
        <button
          class="console-button"
          :disabled="inventoryLoading || sourcesLoading || marketplaceLoading"
          @click="handleRefresh"
        >
          <SIcon
            name="RefreshCw"
            size="w-4 h-4"
            :class="{ 'animate-spin': inventoryLoading || sourcesLoading || marketplaceLoading }"
          />
          <span>Refresh</span>
        </button>
      </template>

      <div class="skills-header-stats">
        <div class="skills-header-stat">
          <span class="skills-header-stat__label">Logical skills</span>
          <strong class="skills-header-stat__value">{{ stats.logicalSkills }}</strong>
        </div>
        <div class="skills-header-stat">
          <span class="skills-header-stat__label">Sources</span>
          <strong class="skills-header-stat__value">{{ stats.sources }}</strong>
        </div>
        <div class="skills-header-stat">
          <span class="skills-header-stat__label">Installed packages</span>
          <strong class="skills-header-stat__value">{{ stats.installations }}</strong>
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
      <!-- Tab 切换栏 -->
      <div class="console-tabs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="console-tab"
          :class="{ 'console-tab--active': routeState.tab === tab.id }"
          @click="setTab(tab.id)"
        >
          <SIcon
            :name="tab.icon"
            size="w-4 h-4"
          />
          <span>{{ tab.label }}</span>
          <span class="console-tab__count">{{ tab.count }}</span>
        </button>
      </div>

      <!-- 主内容区 -->
      <div class="console-layout">
        <!-- 左侧边栏：平台选择 + 上下文面板 + 活动日志 -->
        <aside class="console-sidebar">
          <PlatformSelector
            v-model="selectedPlatforms"
            :platforms="platforms"
            @select-detected="selectDetectedPlatforms"
          />

          <!-- Inventory 筛选器 -->
          <section
            v-if="routeState.tab === 'inventory'"
            class="panel"
          >
            <div class="panel__header">
              <h2 class="panel__title">
                Filters
              </h2>
              <button
                class="panel__link"
                @click="resetFilters"
              >
                Reset
              </button>
            </div>

            <label class="field">
              <span class="field__label">Search</span>
              <input
                v-model="searchInput"
                class="field__input"
                type="text"
                placeholder="name / tag / category"
              >
            </label>

            <label class="field">
              <span class="field__label">Platform</span>
              <select
                class="field__input"
                :value="routeState.platform"
                @change="updatePlatformFilter(($event.target as HTMLSelectElement).value)"
              >
                <option value="all">All platforms</option>
                <option
                  v-for="p in platforms"
                  :key="p.id"
                  :value="p.id"
                >
                  {{ p.displayName }}
                </option>
              </select>
            </label>

            <label class="field">
              <span class="field__label">Origin</span>
              <select
                class="field__input"
                :value="routeState.origin"
                @change="updateOriginFilter(($event.target as HTMLSelectElement).value)"
              >
                <option value="all">All origins</option>
                <option
                  v-for="origin in originOptions"
                  :key="origin"
                  :value="origin"
                >
                  {{ origin }}
                </option>
              </select>
            </label>

            <label class="field">
              <span class="field__label">Category</span>
              <select
                v-model="filters.category"
                class="field__input"
              >
                <option :value="null">All categories</option>
                <option
                  v-for="cat in availableCategories"
                  :key="cat"
                  :value="cat"
                >
                  {{ cat }}
                </option>
              </select>
            </label>

            <label class="field">
              <span class="field__label">Source</span>
              <select
                class="field__input"
                :value="routeState.source ?? 'all'"
                @change="updateSourceFilter(($event.target as HTMLSelectElement).value)"
              >
                <option value="all">All sources</option>
                <option
                  v-for="source in sourceOptions"
                  :key="source.id"
                  :value="source.id"
                >
                  {{ source.name }}
                </option>
              </select>
            </label>

            <div class="tag-cloud">
              <button
                v-for="tag in availableTags"
                :key="tag"
                class="tag-chip"
                :class="{ 'tag-chip--active': filters.tags.includes(tag) }"
                @click="toggleTag(tag)"
              >
                {{ tag }}
              </button>
            </div>
          </section>

          <!-- Sources 添加表单 -->
          <section
            v-else-if="routeState.tab === 'sources'"
            class="panel"
          >
            <div class="panel__header">
              <h2 class="panel__title">
                Add Source
              </h2>
            </div>

            <label class="field">
              <span class="field__label">Git repo</span>
              <input
                v-model="gitSourceUrl"
                class="field__input"
                type="text"
                placeholder="https://github.com/owner/repo"
              >
            </label>
            <button
              class="console-button console-button--primary"
              :disabled="mutationLoading || !gitSourceUrl.trim()"
              @click="handleAddGitSource"
            >
              <SIcon
                name="Github"
                size="w-4 h-4"
              />
              <span>Add Git Source</span>
            </button>

            <label class="field">
              <span class="field__label">Local directory</span>
              <div class="field__row">
                <input
                  v-model="localSourcePath"
                  class="field__input"
                  type="text"
                  placeholder="D:/skills/repo"
                >
                <button
                  class="console-button"
                  @click="handlePickFolder"
                >
                  <SIcon
                    name="FolderOpen"
                    size="w-4 h-4"
                  />
                </button>
              </div>
            </label>
            <button
              class="console-button console-button--primary"
              :disabled="mutationLoading || !localSourcePath.trim()"
              @click="handleAddLocalSource"
            >
              <SIcon
                name="FolderGit2"
                size="w-4 h-4"
              />
              <span>Add Local Source</span>
            </button>
          </section>

          <!-- Marketplace 手动安装 -->
          <section
            v-else
            class="panel"
          >
            <div class="panel__header">
              <h2 class="panel__title">
                Marketplace Context
              </h2>
            </div>

            <div class="skills-marketplace-context">
              <div class="skills-marketplace-context__item">
                <span class="field__label">Target platforms</span>
                <strong>{{ selectedPlatforms.length > 0 ? selectedPlatforms.length : 0 }}</strong>
              </div>
              <div class="skills-marketplace-context__item">
                <span class="field__label">Search</span>
                <strong>{{ routeState.q || 'Trending' }}</strong>
              </div>
              <div class="skills-marketplace-context__item">
                <span class="field__label">Page</span>
                <strong>{{ routeState.page }}</strong>
              </div>
            </div>
          </section>

          <ActivityLog :entries="operationLog" />
        </aside>

        <!-- 主内容：按 Tab 切换 -->
        <div class="console-main">
          <InventoryPanel
            v-if="routeState.tab === 'inventory'"
            :selected-platforms="selectedPlatforms"
            @select="handleSkillSelect"
            @update:mode="handleModeChange"
          />

          <SourcesPanel
            v-else-if="routeState.tab === 'sources'"
            :selected-platforms="selectedPlatforms"
            @view-source="handleViewSource"
          />

          <MarketplacePanel
            v-else
            :selected-platforms="selectedPlatforms"
            :search-initial="routeState.q"
            :page="routeState.page"
            @update:page="updatePage"
            @update:search="updateSearchQuery"
          />
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import PageHeaderCard from '@/components/PageHeaderCard.vue'
import AsyncStatePanel from '@/components/ui/AsyncStatePanel.vue'
import SIcon from '@/components/ui/SIcon.vue'
import PlatformSelector from '@/components/skills/PlatformSelector.vue'
import ActivityLog from '@/components/skills/ActivityLog.vue'
import InventoryPanel from '@/components/skills/InventoryPanel.vue'
import SourcesPanel from '@/components/skills/SourcesPanel.vue'
import MarketplacePanel from '@/components/skills/MarketplacePanel.vue'
import { useUnifiedSkills } from '@/composables/useUnifiedSkills'
import { useUIStore } from '@/stores/ui'
import { PLATFORM_CONFIG } from '@/types/skills'
import { getRuntimeUnavailableCopy } from '@/utils/runtimeState'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import type { Platform, SkillOrigin, SkillsRouteState, SkillsTab } from '@/types/skills'

const router = useRouter()
const route = useRoute()
const uiStore = useUIStore()
const {
  initialize,
  refresh,
  loadMarketplace,
  loadNpxStatus,
  applyRouteState,
  addGitSource,
  addLocalSourceRecord,
  browseFolder,
  platforms,
  sources,
  marketplace,
  filters,
  routeState,
  operationLog,
  mutationLoading,
  availableCategories,
  availableTags,
  stats,
  inventoryLoading,
  sourcesLoading,
  marketplaceLoading,
  selectSkill,
} = useUnifiedSkills()

// --- 本地 UI 状态 ---
const originOptions: SkillOrigin[] = ['marketplace', 'github', 'repo', 'local', 'npx', 'unknown']
const selectedPlatforms = ref<Platform[]>([])
const searchInput = ref('')
const gitSourceUrl = ref('')
const localSourcePath = ref('')
let stopSkillsEvent: null | (() => void) = null
let searchTimer = 0
const sourceOptions = computed(() => sources.value.map((source) => ({
  id: source.id,
  name: source.name,
})))

const tabs = computed(() => [
  { id: 'inventory' as SkillsTab, label: 'Inventory', icon: 'Package', count: stats.value.logicalSkills },
  { id: 'sources' as SkillsTab, label: 'Sources', icon: 'FolderGit2', count: stats.value.sources },
  { id: 'marketplace' as SkillsTab, label: 'Marketplace', icon: 'Store', count: marketplace.value.total },
])
const runtimeUnavailable = computed(() => !isTauriRuntime())
const runtimeCopy = computed(() => getRuntimeUnavailableCopy('skills'))

// --- 路由同步 ---
function normalizeRouteState(query: Record<string, unknown>): SkillsRouteState {
  return {
    tab: query.tab === 'sources' || query.tab === 'marketplace' ? query.tab : 'inventory',
    selected: typeof query.selected === 'string' ? query.selected : null,
    mode: query.mode === 'edit' ? 'edit' : 'view',
    platform: typeof query.platform === 'string' && query.platform in PLATFORM_CONFIG ? query.platform as Platform : 'all',
    origin: typeof query.origin === 'string' && originOptions.includes(query.origin as SkillOrigin) ? query.origin as SkillOrigin : 'all',
    q: typeof query.q === 'string' ? query.q : '',
    page: typeof query.page === 'string' ? Math.max(Number(query.page) || 1, 1) : 1,
    source: typeof query.source === 'string' ? query.source : null,
  }
}

function replaceRoute(patch: Partial<SkillsRouteState>) {
  const next = { ...routeState.value, ...patch }
  applyRouteState(next)
  const query: Record<string, string> = {}
  if (next.tab !== 'inventory') query.tab = next.tab
  if (next.selected) query.selected = next.selected
  if (next.mode !== 'view') query.mode = next.mode
  if (next.platform !== 'all') query.platform = next.platform
  if (next.origin !== 'all') query.origin = next.origin
  if (next.q) query.q = next.q
  if (next.page > 1) query.page = String(next.page)
  if (next.source) query.source = next.source
  void router.replace({ path: '/skills', query })
}

function setTab(tab: SkillsTab) { replaceRoute({ tab, page: 1 }) }
function updatePlatformFilter(value: string) { replaceRoute({ platform: value as SkillsRouteState['platform'] }) }
function updateOriginFilter(value: string) { replaceRoute({ origin: value as SkillsRouteState['origin'] }) }
function updateSourceFilter(value: string) { replaceRoute({ source: value === 'all' ? null : value, selected: null }) }
function updatePage(page: number) { replaceRoute({ page }) }
function updateSearchQuery(q: string) { replaceRoute({ q, page: 1 }) }

function handleSkillSelect(skillId: string) { replaceRoute({ selected: skillId }) }
function handleModeChange(mode: 'view' | 'edit') { replaceRoute({ mode }) }

function resetFilters() {
  filters.value = { search: '', platform: 'all', origin: 'all', category: null, tags: [], source: 'all' }
  searchInput.value = ''
  replaceRoute({ platform: 'all', origin: 'all', q: '', source: null, selected: null })
}

function toggleTag(tag: string) {
  filters.value = {
    ...filters.value,
    tags: filters.value.tags.includes(tag)
      ? filters.value.tags.filter(t => t !== tag)
      : [...filters.value.tags, tag],
  }
}

function selectDetectedPlatforms() {
  selectedPlatforms.value = platforms.value.filter(p => p.detected).map(p => p.id)
}

// --- 侧边栏操作 ---
async function handleAddGitSource() {
  try {
    await addGitSource(gitSourceUrl.value.trim())
    gitSourceUrl.value = ''
    uiStore.showSuccess('Git source added')
  }
  catch (error) { uiStore.showError(error instanceof Error ? error.message : String(error)) }
}

async function handleAddLocalSource() {
  try {
    await addLocalSourceRecord(localSourcePath.value.trim())
    localSourcePath.value = ''
    uiStore.showSuccess('Local source added')
  }
  catch (error) { uiStore.showError(error instanceof Error ? error.message : String(error)) }
}

async function handlePickFolder() {
  try {
    const path = await browseFolder()
    if (!path) return
    localSourcePath.value = path
  }
  catch (error) { uiStore.showError(error instanceof Error ? error.message : String(error)) }
}

async function handleRefresh() {
  if (runtimeUnavailable.value) return
  await refresh(routeState.value.tab === 'marketplace')
}

function handleViewSource(sourceId: string) {
  replaceRoute({ tab: 'inventory', source: sourceId, selected: null, page: 1 })
}

// --- Watchers ---
watch(
  () => route.query,
  (query) => {
    const normalized = normalizeRouteState(query as Record<string, unknown>)
    applyRouteState(normalized)
    searchInput.value = normalized.q

    if (runtimeUnavailable.value) { selectSkill(null, null); return }

    if (normalized.selected) {
      selectSkill(normalized.selected, null)
    }
    else {
      selectSkill(null, null)
    }
    if (normalized.tab === 'marketplace') void loadMarketplace(true)
  },
  { immediate: true },
)

watch(searchInput, (value) => {
  if (routeState.value.tab !== 'inventory') return
  window.clearTimeout(searchTimer)
  searchTimer = window.setTimeout(() => {
    filters.value = { ...filters.value, search: value.trim() }
    replaceRoute({ q: value.trim() })
  }, 300)
})

watch(
  platforms,
  (current) => {
    if (selectedPlatforms.value.length === 0 && current.length > 0) selectDetectedPlatforms()
  },
  { immediate: true },
)

// --- 生命周期 ---
onMounted(async () => {
  if (runtimeUnavailable.value) return
  await initialize(false)
  await loadNpxStatus(true)
  if (routeState.value.tab === 'marketplace') await loadMarketplace()

  if (isTauriRuntime()) {
    const { listen } = await import('@tauri-apps/api/event')
    stopSkillsEvent = await listen('skills-changed', async () => {
      await refresh(routeState.value.tab === 'marketplace')
    })
  }
})

onUnmounted(() => {
  stopSkillsEvent?.()
  stopSkillsEvent = null
})
</script>

<style scoped>
.skills-console {
  @apply flex flex-col gap-5 px-4 py-4;
}

.panel,
.console-tabs {
  @apply rounded-3xl p-4;

  background: var(--surface-workspace-bg);
  border: 1px solid var(--surface-workspace-border);
  box-shadow: var(--elevation-2);
  backdrop-filter: var(--surface-workspace-blur);
}

.skills-header-stats {
  @apply grid gap-3 md:grid-cols-3;
}

.skills-header-stat {
  @apply rounded-2xl border border-border-default/55 px-4 py-3;

  background-color: rgb(var(--color-bg-elevated-rgb) / 72%);
  backdrop-filter: blur(14px);
}

.skills-header-stat__label {
  @apply block text-[11px] uppercase tracking-[0.12em] text-text-muted;
}

.skills-header-stat__value {
  @apply mt-2 block text-lg font-semibold tracking-tight text-text-primary;
}

.panel__title,
.field__label {
  @apply text-xs font-semibold uppercase tracking-[0.16em] text-text-muted;
}

.console-tabs {
  @apply flex flex-wrap gap-2 p-2;
}

.console-tab,
.console-button,
.tag-chip {
  @apply inline-flex items-center gap-2 rounded-xl border border-border-default/55 px-3 py-2 text-sm text-text-secondary;

  background: var(--surface-status-bg);
  border-color: var(--surface-status-border);
  box-shadow: var(--elevation-1);
  transition:
    color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.console-tab--active,
.tag-chip--active,
.console-button--primary {
  @apply text-text-primary;

  background: linear-gradient(180deg, rgb(var(--color-accent-primary-rgb) / 18%), rgb(var(--color-accent-secondary-rgb) / 10%));
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
  box-shadow: var(--elevation-2);
}

.console-tab__count {
  @apply ml-auto rounded-full px-2 py-0.5 text-xs text-text-secondary;

  background-color: rgb(var(--color-bg-base-rgb) / 68%);
}

.console-layout {
  @apply grid gap-4 xl:grid-cols-[320px_minmax(0,1fr)];
}

.console-sidebar {
  @apply flex flex-col gap-4;
}

.skills-marketplace-context {
  @apply flex flex-col gap-3;
}

.skills-marketplace-context__item {
  @apply rounded-2xl border border-border-default/45 p-3;

  background-color: rgb(var(--color-bg-base-rgb) / 55%);
}

.skills-marketplace-context__item strong {
  @apply mt-1 block text-sm text-text-primary;
}

.panel__header {
  @apply mb-3 flex items-center justify-between gap-2;
}

.panel__link {
  @apply text-xs text-text-muted;
}

.tag-cloud {
  @apply flex flex-col gap-2;
}

.field {
  @apply flex flex-col gap-2;
}

.field__input {
  @apply w-full rounded-2xl border border-border-default/55 px-3 py-2 text-sm text-text-primary outline-none placeholder:text-text-muted/70;

  background: var(--surface-status-bg);
  border-color: var(--surface-status-border);
  box-shadow: var(--elevation-1);
  backdrop-filter: var(--surface-status-blur);
  transition:
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.field__input:hover {
  border-color: rgb(var(--color-accent-primary-rgb) / 28%);
}

.field__input:focus {
  border-color: rgb(var(--color-accent-primary-rgb) / 40%);
  box-shadow: var(--elevation-2);
}

.field__row {
  @apply flex gap-2;
}

.console-main {
  @apply min-w-0;
}

@media (width <= 1279px) {
  .console-layout {
    grid-template-columns: 1fr;
  }
}
</style>
