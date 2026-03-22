<template>
  <div class="add-skill-view">
    <!-- Header -->
    <header class="add-skill-header">
      <div class="add-skill-header__left">
        <RouterLink
          to="/skills"
          class="add-skill-header__back"
        >
          <SIcon
            name="ArrowLeft"
            size="w-4 h-4"
          />
          <span>{{ $t('skills.backToSkills') }}</span>
        </RouterLink>
        <h1 class="add-skill-header__title">
          <SIcon
            name="Plus"
            size="w-5 h-5"
          />
          {{ $t('skills.addSkillPageTitle') }}
        </h1>
        <p class="add-skill-header__subtitle">
          {{ $t('skills.addSkillPageSubtitle') }}
        </p>
      </div>
    </header>

    <SkillMarketplaceBrowsePanel
      :batch-mode="batchMode"
      :batch-selected-count="batchSelected.size"
      :current-page="currentPage"
      :is-batch-selected="isBatchSelected"
      :is-installing="isInstallingPackage"
      :is-marketplace-loading="isMarketplaceLoading"
      :is-refreshing="isRefreshing"
      :is-skill-installed="isSkillInstalled"
      :marketplace-cached="marketplaceCached"
      :marketplace-error="marketplaceError"
      :marketplace-items="marketplaceItems"
      :page-size="pageSize"
      :paged-items="pagedItems"
      :search-query="searchQuery"
      :sort-by="sortBy"
      :sorted-items="sortedItems"
      @batch-install="handleBatchInstall"
      @batch-select="handleBatchSelect"
      @clear-batch-selected="clearBatchSelected"
      @marketplace-install="handleMarketplaceInstall"
      @refresh-cache="handleRefreshCache"
      @search="handleSearch"
      @update:batch-mode="updateBatchMode"
      @update:current-page="currentPage = $event"
      @update:search-query="searchQuery = $event"
      @update:sort-by="sortBy = $event"
    />

    <SkillManualInstallPanel
      :active-source="activeSource"
      :can-manual-install="canManualInstall"
      :clear-selected-platforms="clearSelectedPlatforms"
      :github-url="githubUrl"
      :handle-browse="handleBrowse"
      :handle-manual-install="handleManualInstall"
      :local-path="localPath"
      :manual-installing="manualInstalling"
      :manual-tabs="manualTabs"
      :npx-available="npxAvailable"
      :npx-global="npxGlobal"
      :npx-package="npxPackage"
      :npx-version="npxVersion"
      :platforms="platforms"
      :selected-platforms="selectedPlatforms"
      :select-detected="selectDetected"
      :set-active-source="setActiveSource"
      :update-github-url="updateGithubUrl"
      :update-local-path="updateLocalPath"
      :update-npx-global="updateNpxGlobal"
      :update-npx-package="updateNpxPackage"
      :update-selected-platforms="updateSelectedPlatforms"
    />

    <SkillPlatformSelectModal
      :close-modal="closePlatformModal"
      :confirm-install="confirmMarketplaceInstall"
      :pending-package="pendingInstallPackage"
      :platforms="platforms"
      :selected-platforms="modalSelectedPlatforms"
      :select-detected="selectDetectedForModal"
      :show="showPlatformModal"
      :update-selected-platforms="updateModalSelectedPlatforms"
    />

    <!-- Install Progress Toast -->
    <SkillInstallToast
      :progress="installProgress"
      @dismiss="setInstallProgress(null)"
    />
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed, onMounted, reactive } from 'vue'
import { useI18n } from 'vue-i18n'
import SkillManualInstallPanel from '@/components/skills/SkillManualInstallPanel.vue'
import SkillMarketplaceBrowsePanel from '@/components/skills/SkillMarketplaceBrowsePanel.vue'
import SkillPlatformSelectModal from '@/components/skills/SkillPlatformSelectModal.vue'
import SkillInstallToast from '@/components/skills/SkillInstallToast.vue'
import { useUnifiedSkills } from '@/composables/useUnifiedSkills'
import type { MarketplaceItem, ImportSource } from '@/types/skills'
import { logger } from '@/utils/logger'

useI18n()

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

const installedSkillNameSet = computed(() => {
  return new Set(skills.value.map(s => s.name.toLowerCase()))
})

// Pagination
const pagedItems = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  return sortedItems.value.slice(start, start + pageSize)
})

// Check installed
function isSkillInstalled(item: MarketplaceItem): boolean {
  const skillName = (item.skill || item.repo || '').toLowerCase()
  return installedSkillNameSet.value.has(skillName)
}

function handleSearch() {
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

function clearBatchSelected() {
  batchSelected.clear()
}

function updateBatchMode(value: boolean) {
  batchMode.value = value
}

function isInstallingPackage(pkg: string) {
  return installingPackages.has(pkg)
}

function isBatchSelected(pkg: string) {
  return batchSelected.has(pkg)
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

function updateModalSelectedPlatforms(value: string[]) {
  modalSelectedPlatforms.value = value
}

function closePlatformModal() {
  showPlatformModal.value = false
}

const pendingInstallPackage = computed(() => pendingInstallItem.value?.package || '')

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
    } catch (err) {
      setInstallProgress({
        phase: 'error',
        package: `${pendingBatchPackages.value.length} skills`,
        message: (err instanceof Error ? err.message : "Error") || 'Batch install failed',
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
  } catch (err) {
    setInstallProgress({
      phase: 'error',
      package: item.package,
      message: (err instanceof Error ? err.message : "Error") || 'Install failed',
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
  { id: 'github' as ManualSource, label: 'skills.github', icon: 'Github' },
  { id: 'local' as ManualSource, label: 'skills.local', icon: 'FolderOpen' },
  { id: 'npx' as ManualSource, label: 'skills.npx', icon: 'Zap' },
]

const setActiveSource = (source: ManualSource) => {
  activeSource.value = source
}

const updateGithubUrl = (value: string) => {
  githubUrl.value = value
}

const updateLocalPath = (value: string) => {
  localPath.value = value
}

const updateNpxPackage = (value: string) => {
  npxPackage.value = value
}

const updateNpxGlobal = (value: boolean) => {
  npxGlobal.value = value
}

const updateSelectedPlatforms = (value: string[]) => {
  selectedPlatforms.value = value
}

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

function clearSelectedPlatforms() {
  selectedPlatforms.value = []
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
  } catch (err) {
    const pkg = activeSource.value === 'github' ? githubUrl.value
              : activeSource.value === 'local' ? localPath.value
              : npxPackage.value
    setInstallProgress({
      phase: 'error',
      package: pkg,
      message: (err instanceof Error ? err.message : "Error") || 'Installation failed',
      startedAt: Date.now()
    })
  } finally {
    manualInstalling.value = false
    setInstalling(false)
  }
}

// === Init ===
onMounted(async () => {
  try {
    await Promise.all([
      fetchPlatforms(),
      fetchMarketplaceTrending(),
      checkNpxStatus(),
    ])
    selectDetected()
  } catch (err) {
    logger.error('[AddSkillView] onMounted error:', err)
  }
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
  @apply flex items-center gap-1.5 text-sm text-white/50
         hover:text-white transition-colors w-fit;
}

.add-skill-header__title {
  @apply flex items-center gap-2 text-2xl font-bold text-white mt-1;
}

.add-skill-header__subtitle {
  @apply text-sm text-white/80;
}

.btn-install {
  @apply flex items-center gap-2 px-5 py-2.5 rounded-xl
         text-sm font-semibold text-white
         bg-accent-primary hover:bg-accent-primary/90
         disabled:opacity-50 disabled:cursor-not-allowed transition-colors;
}
</style>
