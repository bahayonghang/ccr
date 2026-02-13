<template>
  <div class="add-skill-view">
    <!-- Header -->
    <header class="add-skill-header">
      <div class="add-skill-header__left">
        <RouterLink
          to="/skills"
          class="add-skill-header__back"
        >
          <ArrowLeft class="w-4 h-4" />
          <span>{{ $t('skills.backToSkills') }}</span>
        </RouterLink>
        <h1 class="add-skill-header__title">
          <Plus class="w-5 h-5" />
          {{ $t('skills.addSkillPageTitle') }}
        </h1>
        <p class="add-skill-header__subtitle">
          {{ $t('skills.addSkillPageSubtitle') }}
        </p>
      </div>
    </header>

    <!-- Section 1: Browse Trending -->
    <section class="browse-section">
      <div class="section-header">
        <div class="section-header__left">
          <h2 class="section-title">
            <TrendingUp class="w-5 h-5 text-accent-primary" />
            {{ $t('skills.browseTrending') }}
          </h2>
          <span class="section-hint">{{ $t('skills.browseTrendingHint') }}</span>
        </div>
        <div class="section-header__right">
          <span
            v-if="marketplaceCached"
            class="cache-badge"
          >
            <Database class="w-3 h-3" />
            {{ $t('skills.cacheStatus') }}
          </span>
          <button
            class="btn-refresh"
            :disabled="isRefreshing"
            @click="handleRefreshCache"
          >
            <RefreshCw
              class="w-4 h-4"
              :class="{ 'animate-spin': isRefreshing }"
            />
            <span>{{ isRefreshing ? $t('skills.refreshingCache') : $t('skills.refreshCache') }}</span>
          </button>
        </div>
      </div>

      <!-- Search + Sort controls -->
      <div class="browse-controls">
        <div class="browse-search">
          <div class="relative flex-1">
            <Search
              class="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-text-muted"
            />
            <input
              v-model="searchQuery"
              type="text"
              class="search-input"
              :placeholder="$t('skills.searchMarketplace')"
              @input="onSearchInput"
              @keyup.enter="handleSearch"
            >
          </div>
          <button
            class="btn-search"
            :disabled="isMarketplaceLoading"
            @click="handleSearch"
          >
            <Loader2
              v-if="isMarketplaceLoading"
              class="w-4 h-4 animate-spin"
            />
            <Search
              v-else
              class="w-4 h-4"
            />
            <span>{{ $t('common.search') }}</span>
          </button>
        </div>

        <div class="browse-toolbar">
          <div class="toolbar-left">
            <span
              v-if="marketplaceItems.length > 0"
              class="result-badge"
            >
              {{ marketplaceItems.length }} {{ $t('skills.resultCount') }}
            </span>
          </div>
          <div class="toolbar-right">
            <div class="sort-select">
              <ArrowUpDown class="w-3.5 h-3.5 text-text-muted" />
              <select
                v-model="sortBy"
                class="sort-dropdown"
              >
                <option value="stars">
                  {{ $t('skills.sortStars') }}
                </option>
                <option value="name">
                  {{ $t('skills.sortName') }}
                </option>
              </select>
            </div>

            <button
              class="btn-batch"
              :class="{ 'btn-batch--active': batchMode }"
              @click="batchMode = !batchMode; if (!batchMode) batchSelected.clear()"
            >
              <CheckSquare class="w-4 h-4" />
              <span>{{ $t('skills.batchMode') }}</span>
            </button>
          </div>
        </div>
      </div>

      <!-- Error State -->
      <div
        v-if="marketplaceError"
        class="state-box state-box--error"
      >
        <AlertCircle class="w-8 h-8 text-danger" />
        <p class="text-danger mt-2">
          {{ marketplaceError }}
        </p>
      </div>

      <!-- Empty State -->
      <div
        v-else-if="!isMarketplaceLoading && sortedItems.length === 0"
        class="state-box"
      >
        <Store class="w-12 h-12 text-text-muted" />
        <h3 class="text-lg font-semibold text-text-primary mt-4">
          {{ $t('skills.noMarketplaceResults') }}
        </h3>
        <p class="text-sm text-text-secondary mt-1">
          {{ $t('skills.tryDifferentSearch') }}
        </p>
      </div>

      <!-- Loading Skeleton -->
      <div
        v-else-if="isMarketplaceLoading"
        class="marketplace-grid"
      >
        <div
          v-for="i in 8"
          :key="i"
          class="skeleton-card"
        >
          <div class="skeleton-header">
            <div class="flex items-center gap-2">
              <div class="skeleton-avatar" />
              <div class="skeleton-owner" />
            </div>
            <div class="skeleton-stars" />
          </div>
          <div class="skeleton-name" />
          <div class="skeleton-desc">
            <div class="skeleton-line w-full" />
            <div class="skeleton-line w-3/4" />
          </div>
          <div class="skeleton-footer">
            <div class="skeleton-link" />
            <div class="skeleton-btn" />
          </div>
        </div>
      </div>

      <!-- Marketplace Grid -->
      <div
        v-else
        class="marketplace-grid"
      >
        <MarketplaceSkillCard
          v-for="item in pagedItems"
          :key="item.package"
          :item="item"
          :is-installed="isSkillInstalled(item)"
          :is-installing="installingPackages.has(item.package)"
          :batch-mode="batchMode"
          :is-selected="batchSelected.has(item.package)"
          @install="handleMarketplaceInstall"
          @select="handleBatchSelect"
        />
      </div>

      <!-- Pagination -->
      <MarketplacePagination
        v-if="!isMarketplaceLoading && sortedItems.length > 0"
        :current-page="currentPage"
        :total-items="sortedItems.length"
        :page-size="pageSize"
        @page-change="currentPage = $event"
      />

      <!-- Batch Action Bar -->
      <Transition name="batch-bar">
        <div
          v-if="batchMode && batchSelected.size > 0"
          class="batch-bar"
        >
          <span class="batch-bar__count">
            {{ $t('skills.selectedCount', { count: batchSelected.size }) }}
          </span>
          <div class="batch-bar__actions">
            <button
              class="batch-bar__clear"
              @click="batchSelected.clear()"
            >
              {{ $t('skills.clearAll') }}
            </button>
            <button
              class="batch-bar__install"
              @click="handleBatchInstall"
            >
              <Download class="w-4 h-4" />
              {{ $t('skills.batchInstall') }}
            </button>
          </div>
        </div>
      </Transition>
    </section>

    <!-- Section 2: Manual Install -->
    <section class="manual-section">
      <h2 class="section-title">
        <Terminal class="w-5 h-5 text-accent-primary" />
        {{ $t('skills.manualInstall') }}
      </h2>

      <!-- Tab 切换 -->
      <div class="manual-tabs">
        <button
          v-for="tab in manualTabs"
          :key="tab.id"
          class="manual-tab"
          :class="{ 'manual-tab--active': activeSource === tab.id }"
          @click="activeSource = tab.id"
        >
          <component
            :is="tab.icon"
            class="w-4 h-4"
          />
          <span>{{ $t(tab.label) }}</span>
        </button>
      </div>

      <div class="manual-body">
        <!-- GitHub Tab -->
        <div
          v-if="activeSource === 'github'"
          class="tab-content"
        >
          <div class="input-group">
            <Github class="input-icon" />
            <input
              v-model="githubUrl"
              type="text"
              class="text-input"
              :placeholder="$t('skills.githubUrlPlaceholder')"
            >
          </div>
          <p class="tab-hint">
            {{ $t('skills.githubFormats') }}
          </p>
        </div>

        <!-- 本地 Tab -->
        <div
          v-if="activeSource === 'local'"
          class="tab-content"
        >
          <div class="input-group">
            <FolderOpen class="input-icon" />
            <input
              v-model="localPath"
              type="text"
              class="text-input"
              :placeholder="$t('skills.localPathPlaceholder')"
            >
            <button
              class="browse-btn"
              @click="handleBrowse"
            >
              <Folder class="w-4 h-4" />
              {{ $t('skills.browse') }}
            </button>
          </div>
          <p class="tab-hint">
            {{ $t('skills.localHint') }}
          </p>
        </div>

        <!-- npx Tab -->
        <div
          v-if="activeSource === 'npx'"
          class="tab-content"
        >
          <div class="npx-status">
            <div
              class="npx-indicator"
              :class="npxAvailable ? 'npx-indicator--ok' : 'npx-indicator--no'"
            />
            <span class="text-xs">
              {{ npxAvailable ? $t('skills.npxAvailable') : $t('skills.npxNotAvailable') }}
              <span
                v-if="npxVersion"
                class="text-text-muted"
              >(v{{ npxVersion }})</span>
            </span>
          </div>
          <div class="input-group">
            <Zap class="input-icon" />
            <input
              v-model="npxPackage"
              type="text"
              class="text-input"
              :placeholder="$t('skills.npxPackagePlaceholder')"
            >
          </div>
          <label class="checkbox-label">
            <input
              v-model="npxGlobal"
              type="checkbox"
              class="checkbox-input"
            >
            <span>{{ $t('skills.npxGlobal') }}</span>
          </label>
          <p class="tab-hint">
            {{ $t('skills.npxHint') }}
          </p>
        </div>

        <!-- 目标平台选择 -->
        <div class="platform-section">
          <div class="platform-section__header">
            <h3 class="platform-section__title">
              {{ $t('skills.targetPlatforms') }}
            </h3>
            <div class="platform-section__actions">
              <button
                class="platform-action"
                @click="selectDetected"
              >
                {{ $t('skills.selectDetected') }}
              </button>
              <button
                class="platform-action"
                @click="selectedPlatforms = []"
              >
                {{ $t('skills.clearAll') }}
              </button>
            </div>
          </div>
          <div class="platform-grid">
            <label
              v-for="p in platforms"
              :key="p.id"
              class="platform-item"
              :class="{ 'platform-item--disabled': !p.detected }"
            >
              <input
                v-model="selectedPlatforms"
                type="checkbox"
                :value="p.id"
                class="checkbox-input"
              >
              <span class="platform-item__name">{{ p.display_name }}</span>
              <span
                v-if="!p.detected"
                class="platform-item__badge"
              >{{ $t('skills.notDetected') }}</span>
            </label>
          </div>
        </div>

        <!-- 安装按钮 -->
        <div class="manual-footer">
          <button
            class="btn-install"
            :disabled="!canManualInstall || manualInstalling"
            @click="handleManualInstall"
          >
            <Loader2
              v-if="manualInstalling"
              class="w-4 h-4 animate-spin"
            />
            <Download
              v-else
              class="w-4 h-4"
            />
            <span>
              {{ manualInstalling
                ? $t('skills.installing')
                : $t('skills.installTo', { count: selectedPlatforms.length })
              }}
            </span>
          </button>
        </div>
      </div>
    </section>

    <!-- Platform Selection Modal (for marketplace card installs) -->
    <Teleport to="body">
      <Transition name="modal-fade">
        <div
          v-if="showPlatformModal"
          class="platform-modal-overlay"
          @click.self="showPlatformModal = false"
        >
          <Transition name="modal-scale">
            <div
              v-if="showPlatformModal"
              class="platform-modal"
            >
              <div class="platform-modal__header">
                <h3 class="platform-modal__title">
                  {{ $t('skills.installSkill') }}
                </h3>
                <button
                  class="platform-modal__close"
                  @click="showPlatformModal = false"
                >
                  <X class="w-5 h-5" />
                </button>
              </div>
              <p class="platform-modal__pkg">
                {{ pendingInstallItem?.package }}
              </p>
              <div class="platform-section">
                <div class="platform-section__header">
                  <h3 class="platform-section__title">
                    {{ $t('skills.selectPlatforms') }}
                  </h3>
                  <div class="platform-section__actions">
                    <button
                      class="platform-action"
                      @click="selectDetectedForModal"
                    >
                      {{ $t('skills.selectAllDetected') }}
                    </button>
                  </div>
                </div>
                <div class="platform-grid">
                  <label
                    v-for="p in platforms"
                    :key="p.id"
                    class="platform-item"
                    :class="{ 'platform-item--disabled': !p.detected }"
                  >
                    <input
                      v-model="modalSelectedPlatforms"
                      type="checkbox"
                      :value="p.id"
                      class="checkbox-input"
                    >
                    <span class="platform-item__name">{{ p.display_name }}</span>
                    <span
                      v-if="!p.detected"
                      class="platform-item__badge"
                    >{{ $t('skills.notDetected') }}</span>
                  </label>
                </div>
              </div>
              <div class="platform-modal__footer">
                <button
                  class="btn-cancel"
                  @click="showPlatformModal = false"
                >
                  {{ $t('common.cancel') }}
                </button>
                <button
                  class="btn-install"
                  :disabled="modalSelectedPlatforms.length === 0"
                  @click="confirmMarketplaceInstall"
                >
                  <Download class="w-4 h-4" />
                  {{ $t('skills.installTo', { count: modalSelectedPlatforms.length }) }}
                </button>
              </div>
            </div>
          </Transition>
        </div>
      </Transition>
    </Teleport>

    <!-- Install Progress Toast -->
    <SkillInstallToast
      :progress="installProgress"
      @dismiss="setInstallProgress(null)"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, reactive } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  Plus, ArrowLeft, TrendingUp, Database, RefreshCw,
  Search, Loader2, AlertCircle, Store, Download,
  ArrowUpDown, CheckSquare, Terminal,
  Github, FolderOpen, Folder, Zap, X
} from 'lucide-vue-next'
import MarketplaceSkillCard from '@/components/skills/MarketplaceSkillCard.vue'
import MarketplacePagination from '@/components/skills/MarketplacePagination.vue'
import SkillInstallToast from '@/components/skills/SkillInstallToast.vue'
import { useUnifiedSkills } from '@/composables/useUnifiedSkills'
import type { MarketplaceItem, ImportSource } from '@/types/skills'

const { t: _t } = useI18n()

const {
  platforms,
  skills,
  marketplaceItems,
  isMarketplaceLoading,
  marketplaceError,
  marketplaceCached,
  installProgress,
  npxStatus,
  fetchMarketplaceTrending,
  searchMarketplace,
  refreshMarketplaceCache,
  installSkill,
  importFromGithub,
  importFromLocal,
  importViaNpx,
  batchInstall,
  checkNpxStatus,
  browseFolder,
  fetchPlatforms,
  setInstalling,
  setInstallProgress,
} = useUnifiedSkills()

// === Browse Trending ===
const searchQuery = ref('')
const sortBy = ref<'stars' | 'name'>('stars')
const currentPage = ref(1)
const pageSize = 20
const isRefreshing = ref(false)
const batchMode = ref(false)
const batchSelected = reactive(new Set<string>())
const installingPackages = reactive(new Set<string>())

// Sort
const sortedItems = computed(() => {
  const list = [...marketplaceItems.value]
  if (sortBy.value === 'stars') {
    list.sort((a, b) => (b.stars ?? 0) - (a.stars ?? 0))
  } else {
    list.sort((a, b) => (a.skill || a.package).localeCompare(b.skill || b.package))
  }
  return list
})

// Pagination
const pagedItems = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  return sortedItems.value.slice(start, start + pageSize)
})

// Check installed
function isSkillInstalled(item: MarketplaceItem): boolean {
  const installedNames = new Set(skills.value.map(s => s.name.toLowerCase()))
  const skillName = (item.skill || item.repo || '').toLowerCase()
  return installedNames.has(skillName)
}

// Search with debounce
let searchTimer: ReturnType<typeof setTimeout> | null = null
function onSearchInput() {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    handleSearch()
  }, 300)
}

function handleSearch() {
  if (searchTimer) clearTimeout(searchTimer)
  currentPage.value = 1
  if (searchQuery.value.trim()) {
    searchMarketplace(searchQuery.value.trim())
  } else {
    fetchMarketplaceTrending()
  }
}

// Refresh cache
async function handleRefreshCache() {
  isRefreshing.value = true
  try {
    await refreshMarketplaceCache()
  } finally {
    isRefreshing.value = false
  }
}

// Batch
function handleBatchSelect(item: MarketplaceItem) {
  if (batchSelected.has(item.package)) {
    batchSelected.delete(item.package)
  } else {
    batchSelected.add(item.package)
  }
}

async function handleBatchInstall() {
  // Open platform modal for batch install
  pendingBatchPackages.value = [...batchSelected]
  selectDetectedForModal()
  showPlatformModal.value = true
}

// === Platform Selection Modal (for marketplace installs) ===
const showPlatformModal = ref(false)
const modalSelectedPlatforms = ref<string[]>([])
const pendingInstallItem = ref<MarketplaceItem | null>(null)
const pendingBatchPackages = ref<string[]>([])

function handleMarketplaceInstall(item: MarketplaceItem) {
  pendingInstallItem.value = item
  pendingBatchPackages.value = []
  selectDetectedForModal()
  showPlatformModal.value = true
}

function selectDetectedForModal() {
  modalSelectedPlatforms.value = platforms.value
    .filter(p => p.detected)
    .map(p => p.id)
}

async function confirmMarketplaceInstall() {
  showPlatformModal.value = false

  if (pendingBatchPackages.value.length > 0) {
    // Batch install
    setInstallProgress({
      phase: 'downloading',
      package: `${pendingBatchPackages.value.length} skills`,
      startedAt: Date.now()
    })
    try {
      await batchInstall({
        packages: pendingBatchPackages.value,
        agents: modalSelectedPlatforms.value,
        force: false
      })
      setInstallProgress({
        phase: 'done',
        package: `${pendingBatchPackages.value.length} skills`,
        startedAt: Date.now()
      })
      batchSelected.clear()
      batchMode.value = false
    } catch (err: any) {
      setInstallProgress({
        phase: 'error',
        package: `${pendingBatchPackages.value.length} skills`,
        message: err.message || 'Batch install failed',
        startedAt: Date.now()
      })
    }
    return
  }

  if (!pendingInstallItem.value) return
  const item = pendingInstallItem.value
  installingPackages.add(item.package)

  setInstallProgress({
    phase: 'downloading',
    package: item.package,
    startedAt: Date.now()
  })

  try {
    await installSkill({
      package: item.package,
      agents: modalSelectedPlatforms.value,
      force: false
    })
    setInstallProgress({
      phase: 'done',
      package: item.package,
      startedAt: Date.now()
    })
  } catch (err: any) {
    setInstallProgress({
      phase: 'error',
      package: item.package,
      message: err.message || 'Install failed',
      startedAt: Date.now()
    })
  } finally {
    installingPackages.delete(item.package)
    pendingInstallItem.value = null
  }
}

// === Manual Install ===
type ManualSource = Exclude<ImportSource, 'marketplace'>
const activeSource = ref<ManualSource>('github')
const selectedPlatforms = ref<string[]>([])
const manualInstalling = ref(false)

// GitHub
const githubUrl = ref('')
// Local
const localPath = ref('')
// npx
const npxPackage = ref('')
const npxGlobal = ref(false)

const npxAvailable = computed(() => npxStatus.value?.available ?? false)
const npxVersion = computed(() => npxStatus.value?.version)

const manualTabs = [
  { id: 'github' as ManualSource, label: 'skills.github', icon: Github },
  { id: 'local' as ManualSource, label: 'skills.local', icon: FolderOpen },
  { id: 'npx' as ManualSource, label: 'skills.npx', icon: Zap },
]

const canManualInstall = computed(() => {
  if (selectedPlatforms.value.length === 0) return false
  switch (activeSource.value) {
    case 'github': return githubUrl.value.trim().length > 0
    case 'local': return localPath.value.trim().length > 0
    case 'npx': return npxPackage.value.trim().length > 0
    default: return false
  }
})

function selectDetected() {
  selectedPlatforms.value = platforms.value
    .filter(p => p.detected)
    .map(p => p.id)
}

async function handleBrowse() {
  const path = await browseFolder()
  if (path) {
    localPath.value = path
  }
}

async function handleManualInstall() {
  manualInstalling.value = true
  setInstalling(true)

  try {
    switch (activeSource.value) {
      case 'github': {
        setInstallProgress({
          phase: 'downloading',
          package: githubUrl.value,
          startedAt: Date.now()
        })
        await importFromGithub({
          url: githubUrl.value.trim(),
          agents: selectedPlatforms.value,
          force: false
        })
        setInstallProgress({
          phase: 'done',
          package: githubUrl.value,
          startedAt: Date.now()
        })
        githubUrl.value = ''
        break
      }
      case 'local': {
        setInstallProgress({
          phase: 'installing',
          package: localPath.value,
          startedAt: Date.now()
        })
        await importFromLocal({
          sourcePath: localPath.value.trim(),
          agents: selectedPlatforms.value
        })
        setInstallProgress({
          phase: 'done',
          package: localPath.value,
          startedAt: Date.now()
        })
        localPath.value = ''
        break
      }
      case 'npx': {
        setInstallProgress({
          phase: 'downloading',
          package: npxPackage.value,
          startedAt: Date.now()
        })
        await importViaNpx({
          package: npxPackage.value.trim(),
          agents: selectedPlatforms.value,
          global: npxGlobal.value
        })
        setInstallProgress({
          phase: 'done',
          package: npxPackage.value,
          startedAt: Date.now()
        })
        npxPackage.value = ''
        break
      }
    }
  } catch (err: any) {
    const pkg = activeSource.value === 'github' ? githubUrl.value
              : activeSource.value === 'local' ? localPath.value
              : npxPackage.value
    setInstallProgress({
      phase: 'error',
      package: pkg,
      message: err.message || 'Installation failed',
      startedAt: Date.now()
    })
  } finally {
    manualInstalling.value = false
    setInstalling(false)
  }
}

// === Init ===
onMounted(async () => {
  await Promise.all([
    fetchPlatforms(),
    fetchMarketplaceTrending(),
    checkNpxStatus(),
  ])
  selectDetected()
})
</script>

<style scoped>
.add-skill-view {
  @apply flex flex-col gap-6 p-6 max-w-7xl mx-auto;
}

/* Header */
.add-skill-header {
  @apply flex flex-col gap-2;
}

.add-skill-header__back {
  @apply flex items-center gap-1.5 text-sm text-text-muted
         hover:text-text-primary transition-colors w-fit;
}

.add-skill-header__title {
  @apply flex items-center gap-2 text-2xl font-bold text-text-primary mt-1;
}

.add-skill-header__subtitle {
  @apply text-sm text-text-secondary;
}

/* Section */
.browse-section,
.manual-section {
  @apply flex flex-col gap-4 p-5 rounded-2xl
         border border-border-subtle;

  background: rgb(var(--color-bg-elevated-rgb) / 40%);
}

.section-header {
  @apply flex items-center justify-between flex-wrap gap-3;
}

.section-header__left {
  @apply flex items-center gap-3;
}

.section-header__right {
  @apply flex items-center gap-2;
}

.section-title {
  @apply flex items-center gap-2 text-lg font-bold text-text-primary;
}

.section-hint {
  @apply text-xs text-text-muted;
}

/* Cache Badge */
.cache-badge {
  @apply flex items-center gap-1 px-2.5 py-1 rounded-lg
         text-xs font-medium;

  color: rgb(var(--color-success-rgb));
  background: rgb(var(--color-success-rgb) / 10%);
}

.btn-refresh {
  @apply flex items-center gap-1.5 px-3 py-1.5 rounded-lg
         text-xs font-medium text-text-secondary
         bg-bg-surface border border-border-subtle
         hover:text-text-primary hover:border-border-default
         disabled:opacity-50 transition-all;
}

/* Browse Controls */
.browse-controls {
  @apply flex flex-col gap-3;
}

.browse-search {
  @apply flex gap-2;
}

.search-input {
  @apply w-full bg-bg-surface border border-border-subtle rounded-xl
         text-text-primary
         pl-12 pr-4 py-3 text-sm font-medium
         focus:outline-none focus:ring-2 focus:ring-accent-primary/30
         focus:border-accent-primary/50 transition-all;

  &::placeholder {
    color: rgb(var(--color-text-muted-rgb) / 50%);
  }
}

.btn-search {
  @apply flex items-center gap-2 px-4 py-3 rounded-xl
         text-sm font-semibold text-white
         bg-accent-primary hover:bg-accent-primary/90
         disabled:opacity-50 transition-colors;
}

.browse-toolbar {
  @apply flex items-center justify-between;
}

.toolbar-left,
.toolbar-right {
  @apply flex items-center gap-2;
}

.result-badge {
  @apply px-2.5 py-1 rounded-lg text-xs font-semibold
         bg-accent-primary/10 text-accent-primary;
}

.sort-select {
  @apply flex items-center gap-1.5 px-3 py-2 rounded-lg
         bg-bg-surface border border-border-subtle
         text-sm text-text-secondary;
}

.sort-dropdown {
  @apply bg-transparent border-none outline-none text-sm
         text-text-primary cursor-pointer;
}

.btn-batch {
  @apply flex items-center gap-1.5 px-3 py-2 rounded-lg
         text-sm font-medium text-text-secondary
         bg-bg-surface border border-border-subtle
         hover:text-text-primary hover:border-border-default
         transition-all;
}

.btn-batch--active {
  @apply text-accent-primary border-accent-primary/30;

  background: rgb(var(--color-accent-primary-rgb) / 8%);
}

/* State boxes */
.state-box {
  @apply flex flex-col items-center justify-center py-16
         rounded-2xl border border-border-subtle;

  background: rgb(var(--color-bg-surface-rgb) / 30%);
}

/* Grid Layout */
.marketplace-grid {
  @apply grid gap-4;

  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
}

/* Skeleton */
.skeleton-card {
  @apply flex flex-col gap-3 p-4 rounded-2xl border border-border-subtle;

  background: rgb(var(--color-bg-elevated-rgb) / 30%);
}

.skeleton-header { @apply flex items-center justify-between; }
.skeleton-avatar { @apply w-6 h-6 rounded-full bg-bg-surface animate-pulse; }
.skeleton-owner { @apply w-16 h-4 rounded bg-bg-surface animate-pulse; }
.skeleton-stars { @apply w-12 h-4 rounded bg-bg-surface animate-pulse; }
.skeleton-name { @apply w-32 h-5 rounded bg-bg-surface animate-pulse; }
.skeleton-desc { @apply flex flex-col gap-1.5; }
.skeleton-line { @apply h-3.5 rounded bg-bg-surface animate-pulse; }
.skeleton-footer { @apply flex items-center justify-between mt-auto pt-3 border-t border-border-subtle; }
.skeleton-link { @apply w-20 h-4 rounded bg-bg-surface animate-pulse; }
.skeleton-btn { @apply w-16 h-7 rounded-lg bg-bg-surface animate-pulse; }

/* Batch Bar */
.batch-bar {
  @apply fixed bottom-6 left-1/2 -translate-x-1/2 z-40
         flex items-center gap-4 px-6 py-3 rounded-2xl
         border border-border-subtle shadow-2xl;

  background: rgb(var(--color-bg-elevated-rgb) / 95%);
  backdrop-filter: blur(16px);
}

.batch-bar__count { @apply text-sm font-semibold text-text-primary; }
.batch-bar__actions { @apply flex items-center gap-2; }

.batch-bar__clear {
  @apply px-3 py-1.5 rounded-lg text-sm text-text-secondary
         hover:text-text-primary hover:bg-bg-surface transition-colors;
}

.batch-bar__install {
  @apply flex items-center gap-1.5 px-4 py-1.5 rounded-lg
         text-sm font-semibold text-white
         bg-accent-primary hover:bg-accent-primary/90 transition-colors;
}

.batch-bar-enter-active,
.batch-bar-leave-active {
  transition: all 0.3s ease;
}

.batch-bar-enter-from {
  opacity: 0;
  transform: translateX(-50%) translateY(20px);
}

.batch-bar-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(20px);
}

/* Manual Install */
.manual-tabs {
  @apply flex gap-1;
}

.manual-tab {
  @apply flex items-center gap-1.5 px-4 py-2.5 rounded-xl
         text-sm font-medium text-text-secondary
         hover:text-text-primary hover:bg-bg-surface
         transition-all duration-200;
}

.manual-tab--active {
  @apply text-text-primary;

  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: rgb(var(--color-accent-primary-rgb));
}

.manual-body {
  @apply flex flex-col gap-4;
}

.tab-content { @apply flex flex-col gap-3; }

.input-group { @apply relative flex items-center; }

.input-icon {
  @apply absolute left-3 w-4 h-4 text-text-muted pointer-events-none;
}

.text-input {
  @apply w-full pl-10 pr-4 py-2.5 rounded-xl
         text-sm text-text-primary
         bg-bg-surface border border-border-subtle
         focus:border-accent-primary focus:outline-none
         placeholder:text-text-muted transition-colors;
}

.browse-btn {
  @apply ml-2 flex items-center gap-1.5 px-3 py-2.5 rounded-xl shrink-0
         text-sm font-medium text-text-secondary
         bg-bg-surface border border-border-subtle
         hover:border-border-default hover:text-text-primary transition-colors;
}

.tab-hint { @apply text-xs text-text-muted leading-relaxed; }

/* npx */
.npx-status {
  @apply flex items-center gap-2 px-3 py-2 rounded-lg bg-bg-surface;
}

.npx-indicator { @apply w-2 h-2 rounded-full; }

.npx-indicator--ok {
  background: rgb(var(--color-success-rgb));
  box-shadow: 0 0 6px rgb(var(--color-success-rgb) / 50%);
}
.npx-indicator--no { background: rgb(var(--color-danger-rgb)); }

.checkbox-label {
  @apply flex items-center gap-2 text-sm text-text-secondary cursor-pointer;
}

.checkbox-input {
  @apply rounded border-border-default text-accent-primary focus:ring-accent-primary/20;
}

/* Platform Selection */
.platform-section {
  @apply flex flex-col gap-3 pt-3 border-t border-border-subtle;
}
.platform-section__header { @apply flex items-center justify-between; }
.platform-section__title { @apply text-sm font-semibold text-text-primary; }
.platform-section__actions { @apply flex items-center gap-2; }
.platform-action { @apply text-xs text-accent-primary hover:underline cursor-pointer; }
.platform-grid { @apply grid grid-cols-2 sm:grid-cols-3 gap-2; }

.platform-item {
  @apply flex items-center gap-2 px-3 py-2 rounded-lg
         bg-bg-surface text-sm cursor-pointer
         hover:bg-bg-elevated transition-colors;
}
.platform-item--disabled { @apply opacity-50; }
.platform-item__name { @apply text-text-primary font-medium; }
.platform-item__badge { @apply ml-auto text-[10px] text-text-muted; }

/* Manual footer */
.manual-footer {
  @apply flex justify-end pt-3 border-t border-border-subtle;
}

.btn-install {
  @apply flex items-center gap-2 px-5 py-2.5 rounded-xl
         text-sm font-semibold text-white
         bg-accent-primary hover:bg-accent-primary/90
         disabled:opacity-50 disabled:cursor-not-allowed transition-all;
}

.btn-cancel {
  @apply px-4 py-2 rounded-xl text-sm font-medium
         text-text-secondary hover:text-text-primary
         hover:bg-bg-surface transition-colors;
}

/* Platform Modal */
.platform-modal-overlay {
  @apply fixed inset-0 z-50 flex items-center justify-center
         bg-black/50 backdrop-blur-sm;
}

.platform-modal {
  @apply flex flex-col gap-4 w-full max-w-md mx-4 p-6 rounded-2xl
         border border-border-subtle shadow-2xl;

  background: rgb(var(--color-bg-base-rgb));
}

.platform-modal__header { @apply flex items-center justify-between; }
.platform-modal__title { @apply text-lg font-bold text-text-primary; }

.platform-modal__close {
  @apply p-2 rounded-lg text-text-muted
         hover:text-text-primary hover:bg-bg-surface transition-colors;
}

.platform-modal__pkg {
  @apply text-sm text-text-secondary font-mono truncate
         px-3 py-2 rounded-lg bg-bg-surface;
}

.platform-modal__footer {
  @apply flex items-center justify-end gap-3 pt-3 border-t border-border-subtle;
}

/* Modal animations */
.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: opacity 0.2s ease;
}

.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}

.modal-scale-enter-active,
.modal-scale-leave-active {
  transition: all 0.25s ease;
}

.modal-scale-enter-from {
  opacity: 0;
  transform: scale(0.95);
}

.modal-scale-leave-to {
  opacity: 0;
  transform: scale(0.95);
}
</style>
