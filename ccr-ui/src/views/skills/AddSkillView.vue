<template>
  <div class="add-skill-view">
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
      :content-mode="marketplaceContentMode"
      :content-state="marketplaceContentState"
      :current-page="currentPage"
      :has-detected-platforms="hasDetectedPlatforms"
      :is-batch-selected="isBatchSelected"
      :is-installing="isInstallingPackage"
      :is-marketplace-loading="isMarketplaceLoading"
      :is-refreshing="isRefreshing"
      :is-skill-installed="isSkillInstalled"
      :marketplace-cached="marketplaceCached"
      :marketplace-error="marketplaceError"
      :marketplace-items="marketplaceItems"
      :no-platform-hint="$t('skills.noDetectedPlatformsHint')"
      :page-size="pageSize"
      :paged-items="pagedItems"
      :search-query="searchQuery"
      :sort-by="sortBy"
      :sorted-items="sortedItems"
      @batch-install="handleBatchInstall"
      @toggle-batch="handleBatchSelect"
      @clear-batch-selected="clearBatchSelected"
      @marketplace-install="handleMarketplaceInstall"
      @refresh="handleRefreshMarketplace"
      @search="handleSearch"
      @update:batch-mode="updateBatchMode"
      @update:current-page="currentPage = $event"
      @update:search-query="searchQuery = $event"
      @update:sort-by="sortBy = $event"
      @view-detail="openDetailModal"
    />

    <SkillManualInstallPanel
      :active-source="activeSource"
      :can-manual-install="canManualInstall"
      :clear-selected-platforms="clearSelectedPlatforms"
      :github-url="githubUrl"
      :handle-browse="handleBrowse"
      :handle-manual-install="handleManualInstall"
      :has-detected-platforms="hasDetectedPlatforms"
      :local-path="localPath"
      :manual-installing="manualInstalling"
      :manual-tabs="manualTabs"
      :no-platform-hint="$t('skills.noDetectedPlatformsHint')"
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
      :batch-packages="pendingBatchPackages"
      :close-modal="closePlatformModal"
      :confirm-install="confirmMarketplaceInstall"
      :mode="platformModalMode"
      :pending-item="pendingInstallItem"
      :platforms="platforms"
      :selected-platforms="modalSelectedPlatforms"
      :select-detected="selectDetectedForModal"
      :show="showPlatformModal"
      :update-selected-platforms="updateModalSelectedPlatforms"
    />

    <SkillMarketplaceDetailModal
      :item="detailItem"
      :install-disabled="!hasDetectedPlatforms"
      :is-installed="detailItem ? isSkillInstalled(detailItem) : false"
      :show="showDetailModal"
      @close="closeDetailModal"
      @install="handleMarketplaceInstall"
    />

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
import SkillMarketplaceDetailModal from '@/components/skills/SkillMarketplaceDetailModal.vue'
import SkillPlatformSelectModal from '@/components/skills/SkillPlatformSelectModal.vue'
import SkillInstallToast from '@/components/skills/SkillInstallToast.vue'
import { useUnifiedSkills } from '@/composables/useUnifiedSkills'
import type { BatchInstallResponse, MarketplaceItem, ImportSource } from '@/types/skills'
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

const { t } = useI18n()

const searchQuery = ref('')
const sortBy = ref<'stars' | 'name'>('stars')
const currentPage = ref(1)
const pageSize = 20
const isRefreshing = ref(false)
const batchMode = ref(false)
const batchSelected = reactive(new Set<string>())
const installingPackages = reactive(new Set<string>())

const normalizedSearchQuery = computed(() => searchQuery.value.trim())
const marketplaceContentMode = computed<'trending' | 'search'>(() => {
  return normalizedSearchQuery.value ? 'search' : 'trending'
})
const marketplaceContentState = computed<'loading' | 'error' | 'empty' | 'ready'>(() => {
  if (isMarketplaceLoading.value) {
    return 'loading'
  }
  if (marketplaceError.value) {
    return 'error'
  }
  return sortedItems.value.length > 0 ? 'ready' : 'empty'
})
const hasDetectedPlatforms = computed(() => {
  return platforms.value.some((platform) => platform.detected)
})

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

const pagedItems = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  return sortedItems.value.slice(start, start + pageSize)
})

function isSkillInstalled(item: MarketplaceItem): boolean {
  const skillName = (item.skill || item.repo || '').toLowerCase()
  return installedSkillNameSet.value.has(skillName)
}

function showNoPlatformFeedback(target: string) {
  setInstallProgress({
    phase: 'error',
    package: target,
    message: t('skills.noDetectedPlatformsHint'),
    startedAt: Date.now(),
  })
}

async function loadMarketplaceContent(forceTrending = false) {
  currentPage.value = 1

  if (!forceTrending && marketplaceContentMode.value === 'search') {
    await searchMarketplace(normalizedSearchQuery.value)
    return
  }

  await fetchMarketplaceTrending(30, 1, forceTrending)
}

async function handleSearch() {
  await loadMarketplaceContent()
}

async function handleRefreshMarketplace() {
  isRefreshing.value = true
  try {
    if (marketplaceContentMode.value === 'search') {
      await refreshMarketplaceCache()
      await searchMarketplace(normalizedSearchQuery.value)
      return
    }

    await refreshMarketplaceCache()
  } finally {
    isRefreshing.value = false
  }
}

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
  if (!value) {
    clearBatchSelected()
  }
}

function isInstallingPackage(pkg: string) {
  return installingPackages.has(pkg)
}

function isBatchSelected(pkg: string) {
  return batchSelected.has(pkg)
}

const showPlatformModal = ref(false)
const modalSelectedPlatforms = ref<string[]>([])
const pendingInstallItem = ref<MarketplaceItem | null>(null)
const pendingBatchPackages = ref<string[]>([])
const platformModalMode = computed<'single' | 'batch'>(() => {
  return pendingBatchPackages.value.length > 0 ? 'batch' : 'single'
})

function handleBatchInstall() {
  if (!hasDetectedPlatforms.value) {
    showNoPlatformFeedback(`${batchSelected.size} skills`)
    return
  }

  if (batchSelected.size === 0) {
    return
  }

  pendingInstallItem.value = null
  pendingBatchPackages.value = [...batchSelected]
  selectDetectedForModal()
  showPlatformModal.value = true
}

function handleMarketplaceInstall(item: MarketplaceItem) {
  if (!hasDetectedPlatforms.value) {
    showNoPlatformFeedback(item.package)
    return
  }

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
  pendingInstallItem.value = null
  pendingBatchPackages.value = []
}

function formatBatchInstallSummary(response: BatchInstallResponse) {
  const summary = t('skills.batchInstallSummary', {
    success: response.successCount,
    total: response.total,
    platforms: modalSelectedPlatforms.value.length,
  })

  if (response.failCount === 0) {
    return summary
  }

  const failedPackages = response.results
    .filter(result => !result.ok)
    .map(result => result.package)
    .filter(Boolean)

  if (failedPackages.length === 0) {
    return `${summary} · ${t('skills.batchInstallFailedCount', { count: response.failCount })}`
  }

  return `${summary} · ${t('skills.batchInstallFailedPackages', {
    count: response.failCount,
    packages: failedPackages.join(', '),
  })}`
}

async function confirmMarketplaceInstall() {
  showPlatformModal.value = false

  if (pendingBatchPackages.value.length > 0) {
    const pendingPackages = [...pendingBatchPackages.value]
    setInstallProgress({
      phase: 'downloading',
      package: `${pendingPackages.length} skills`,
      startedAt: Date.now(),
    })

    try {
      const response = await batchInstall({
        packages: pendingPackages,
        agents: modalSelectedPlatforms.value,
        force: false,
      })
      setInstallProgress({
        phase: response.failCount === 0 ? 'done' : 'error',
        package: `${pendingPackages.length} skills`,
        message: formatBatchInstallSummary(response),
        startedAt: Date.now(),
      })
      if (response.failCount === 0) {
        batchSelected.clear()
        batchMode.value = false
      }
    } catch (err) {
      setInstallProgress({
        phase: 'error',
        package: `${pendingPackages.length} skills`,
        message: (err instanceof Error ? err.message : 'Error') || 'Batch install failed',
        startedAt: Date.now(),
      })
    } finally {
      pendingBatchPackages.value = []
    }
    return
  }

  if (!pendingInstallItem.value) return
  const item = pendingInstallItem.value
  installingPackages.add(item.package)

  setInstallProgress({
    phase: 'downloading',
    package: item.package,
    startedAt: Date.now(),
  })

  try {
    await installSkill({
      package: item.package,
      agents: modalSelectedPlatforms.value,
      force: false,
    })
    setInstallProgress({
      phase: 'done',
      package: item.package,
      startedAt: Date.now(),
    })
  } catch (err) {
    setInstallProgress({
      phase: 'error',
      package: item.package,
      message: (err instanceof Error ? err.message : 'Error') || 'Install failed',
      startedAt: Date.now(),
    })
  } finally {
    installingPackages.delete(item.package)
    pendingInstallItem.value = null
  }
}

type ManualSource = Exclude<ImportSource, 'marketplace'>
const activeSource = ref<ManualSource>('github')
const selectedPlatforms = ref<string[]>([])
const manualInstalling = ref(false)

const githubUrl = ref('')
const localPath = ref('')
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
  if (!hasDetectedPlatforms.value || selectedPlatforms.value.length === 0) return false
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
  if (!hasDetectedPlatforms.value) {
    const target = activeSource.value === 'github'
      ? githubUrl.value || 'github'
      : activeSource.value === 'local'
        ? localPath.value || 'local'
        : npxPackage.value || 'npx'
    showNoPlatformFeedback(target)
    return
  }

  manualInstalling.value = true
  setInstalling(true)

  try {
    switch (activeSource.value) {
      case 'github': {
        setInstallProgress({ phase: 'downloading', package: githubUrl.value, startedAt: Date.now() })
        await importFromGithub({ url: githubUrl.value.trim(), agents: selectedPlatforms.value, force: false })
        setInstallProgress({ phase: 'done', package: githubUrl.value, startedAt: Date.now() })
        githubUrl.value = ''
        break
      }
      case 'local': {
        setInstallProgress({ phase: 'installing', package: localPath.value, startedAt: Date.now() })
        await importFromLocal({ sourcePath: localPath.value.trim(), agents: selectedPlatforms.value })
        setInstallProgress({ phase: 'done', package: localPath.value, startedAt: Date.now() })
        localPath.value = ''
        break
      }
      case 'npx': {
        setInstallProgress({ phase: 'downloading', package: npxPackage.value, startedAt: Date.now() })
        await importViaNpx({ package: npxPackage.value.trim(), agents: selectedPlatforms.value, global: npxGlobal.value })
        setInstallProgress({ phase: 'done', package: npxPackage.value, startedAt: Date.now() })
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
      message: (err instanceof Error ? err.message : 'Error') || 'Installation failed',
      startedAt: Date.now(),
    })
  } finally {
    manualInstalling.value = false
    setInstalling(false)
  }
}

const showDetailModal = ref(false)
const detailItem = ref<MarketplaceItem | null>(null)

function openDetailModal(item: MarketplaceItem) {
  detailItem.value = item
  showDetailModal.value = true
}

function closeDetailModal() {
  showDetailModal.value = false
}

onMounted(async () => {
  try {
    await Promise.all([
      fetchPlatforms(),
      loadMarketplaceContent(),
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
  @apply mx-auto flex max-w-7xl flex-col gap-6 p-6;
}

.add-skill-header {
  @apply flex flex-col gap-2;
}

.add-skill-header__back {
  @apply flex w-fit items-center gap-1.5 text-sm text-white/50 transition-colors hover:text-white;
}

.add-skill-header__title {
  @apply mt-1 flex items-center gap-2 text-2xl font-bold text-white;
}

.add-skill-header__subtitle {
  @apply text-sm text-white/80;
}
</style>
