<template>
  <section class="marketplace-layout">
    <SkillMarketplaceBrowsePanel
      class="marketplace-layout__browse"
      :batch-mode="batchMode"
      :batch-selected-count="batchSelection.size"
      :content-mode="contentMode"
      :content-state="contentState"
      :current-page="page"
      :has-detected-platforms="hasDetectedPlatforms"
      :is-batch-selected="isBatchSelected"
      :is-installing="isInstalling"
      :is-marketplace-loading="marketplaceLoading"
      :is-refreshing="isRefreshing"
      :is-skill-installed="isSkillInstalled"
      :marketplace-cached="marketplace.cached"
      :marketplace-error="marketplaceError"
      :marketplace-items="marketplace.items"
      :no-platform-hint="noPlatformHint"
      :page-size="marketplace.pageSize"
      :paged-items="sortedItems"
      :search-query="searchQuery"
      :sort-by="sortBy"
      :sorted-items="sortedItems"
      :total-items="marketplace.total"
      @batch-install="openBatchInstallModal"
      @clear-batch-selected="clearBatchSelection"
      @marketplace-install="openSingleInstallModal"
      @refresh="handleRefresh"
      @search="handleSearch"
      @toggle-batch="toggleBatchSelection"
      @update:batch-mode="batchMode = $event"
      @update:current-page="emit('update:page', $event)"
      @update:search-query="searchQuery = $event"
      @update:sort-by="sortBy = $event"
      @view-detail="openDetail"
    />

    <section class="panel manual-actions">
      <div class="panel__header">
        <h2 class="panel__title">
          Manual Install
        </h2>
      </div>

      <div class="manual-target-summary">
        <span class="badge">Targets</span>
        <span>{{ targetSummary }}</span>
      </div>

      <label class="field">
        <span class="field__label">GitHub</span>
        <input
          v-model="manualGithub"
          class="field__input"
          type="text"
          placeholder="owner/repo or owner/repo@skill"
        >
      </label>
      <button
        class="console-button console-button--primary"
        :disabled="mutationLoading || !manualGithub.trim()"
        @click="installManual('github')"
      >
        <SIcon
          name="Github"
          size="w-4 h-4"
        />
        <span>Install GitHub</span>
      </button>

      <label class="field">
        <span class="field__label">Local path</span>
        <div class="field__row">
          <input
            v-model="manualLocalPath"
            class="field__input"
            type="text"
            placeholder="D:/skills/local-skill"
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
        :disabled="mutationLoading || !manualLocalPath.trim()"
        @click="installManual('local')"
      >
        <SIcon
          name="HardDrive"
          size="w-4 h-4"
        />
        <span>Install Local</span>
      </button>

      <label class="field">
        <span class="field__label">npx package</span>
        <input
          v-model="manualNpx"
          class="field__input"
          type="text"
          placeholder="owner/repo or owner/repo@skill"
        >
      </label>
      <button
        class="console-button console-button--primary"
        :disabled="mutationLoading || !manualNpx.trim()"
        @click="installManual('npx')"
      >
        <SIcon
          name="Terminal"
          size="w-4 h-4"
        />
        <span>Install npx</span>
      </button>

      <p class="npx-status">
        <span
          class="npx-dot"
          :class="npxStatus?.available ? 'npx-dot--ok' : 'npx-dot--off'"
        />
        {{ npxStatus?.available ? `npx ${npxStatus.version ?? ''}` : 'npx unavailable' }}
      </p>
    </section>

    <SkillMarketplaceDetailModal
      :show="detailModalOpen"
      :item="detailItem"
      :is-installed="detailItem ? isSkillInstalled(detailItem) : false"
      :install-disabled="!hasDetectedPlatforms"
      @close="closeDetail"
      @install="openSingleInstallModal"
    />

    <SkillPlatformSelectModal
      :show="platformModalOpen"
      :mode="platformModalMode"
      :pending-item="pendingItem"
      :batch-packages="pendingBatchPackages"
      :platforms="modalPlatforms"
      :selected-platforms="modalSelectedPlatforms"
      :close-modal="closePlatformModal"
      :select-detected="selectDetectedPlatforms"
      :update-selected-platforms="updateModalSelectedPlatforms"
      :confirm-install="confirmPlatformInstall"
    />
  </section>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import SIcon from '@/components/ui/SIcon.vue'
import SkillMarketplaceBrowsePanel from '@/components/skills/SkillMarketplaceBrowsePanel.vue'
import SkillMarketplaceDetailModal from '@/components/skills/SkillMarketplaceDetailModal.vue'
import SkillPlatformSelectModal from '@/components/skills/SkillPlatformSelectModal.vue'
import { useUnifiedSkills } from '@/composables/useUnifiedSkills'
import { useUIStore } from '@/stores/ui'
import type { MarketplaceItem, Platform, PlatformSummary } from '@/types/skills'
import { isMarketplaceItemInstalled } from '@/utils/skills'

const props = defineProps<{
  selectedPlatforms: Platform[]
  searchInitial?: string
  page: number
}>()

const emit = defineEmits<{
  'update:page': [page: number]
  'update:search': [query: string]
}>()

const uiStore = useUIStore()
const {
  skills,
  platforms,
  marketplace,
  npxStatus,
  mutationLoading,
  marketplaceLoading,
  marketplaceError,
  install,
  batchInstall,
  browseFolder,
  loadMarketplace,
} = useUnifiedSkills()

const searchQuery = ref(props.searchInitial ?? '')
const sortBy = ref<'stars' | 'name'>('stars')
const batchMode = ref(false)
const batchSelection = ref<Set<string>>(new Set())
const installingPackages = ref<Set<string>>(new Set())
const isRefreshing = ref(false)

const manualGithub = ref('')
const manualLocalPath = ref('')
const manualNpx = ref('')

const detailModalOpen = ref(false)
const detailItem = ref<MarketplaceItem | null>(null)

const platformModalOpen = ref(false)
const platformModalMode = ref<'single' | 'batch'>('single')
const pendingItem = ref<MarketplaceItem | null>(null)
const pendingBatchPackages = ref<string[]>([])
const modalSelectedPlatforms = ref<string[]>([])

const detectedPlatforms = computed<Platform[]>(() => {
  return platforms.value
    .filter((platform) => platform.detected)
    .map((platform) => platform.id)
})

const modalPlatforms = computed<PlatformSummary[]>(() => {
  return platforms.value.map((platform) => ({
    id: platform.id,
    displayName: platform.displayName,
    display_name: platform.displayName,
    globalSkillsDir: platform.globalSkillsDir,
    global_skills_dir: platform.globalSkillsDir,
    detected: platform.detected,
    installedCount: platform.installedCount,
    installed_count: platform.installedCount,
  }))
})

const hasDetectedPlatforms = computed(() => detectedPlatforms.value.length > 0)
const targetPlatforms = computed<Platform[]>(() => {
  return props.selectedPlatforms.length > 0 ? props.selectedPlatforms : detectedPlatforms.value
})
const targetSummary = computed(() => {
  if (targetPlatforms.value.length === 0) {
    return 'No detected targets'
  }
  return `${targetPlatforms.value.length} selected`
})
const contentMode = computed(() => searchQuery.value.trim() ? 'search' : 'trending')
const contentState = computed<'loading' | 'error' | 'empty' | 'ready'>(() => {
  if (marketplaceLoading.value) {
    return 'loading'
  }
  if (marketplaceError.value) {
    return 'error'
  }
  if (marketplace.value.items.length === 0) {
    return 'empty'
  }
  return 'ready'
})
const sortedItems = computed(() => {
  const list = [...marketplace.value.items]
  if (sortBy.value === 'stars') {
    list.sort((left, right) => (right.stars ?? 0) - (left.stars ?? 0))
  } else {
    list.sort((left, right) => (left.skill || left.repo).localeCompare(right.skill || right.repo))
  }
  return list
})
const noPlatformHint = 'Detect at least one supported CLI platform before installing or importing skills.'

watch(
  () => props.searchInitial,
  (value) => {
    searchQuery.value = value ?? ''
  },
)

function isSkillInstalled(item: MarketplaceItem) {
  return isMarketplaceItemInstalled(item, skills.value)
}

function isInstalling(pkg: string) {
  return installingPackages.value.has(pkg)
}

function isBatchSelected(pkg: string) {
  return batchSelection.value.has(pkg)
}

function toggleBatchSelection(item: MarketplaceItem) {
  const next = new Set(batchSelection.value)
  if (next.has(item.package)) {
    next.delete(item.package)
  } else {
    next.add(item.package)
  }
  batchSelection.value = next
}

function clearBatchSelection() {
  batchSelection.value = new Set()
}

function openDetail(item: MarketplaceItem) {
  detailItem.value = item
  detailModalOpen.value = true
}

function closeDetail() {
  detailModalOpen.value = false
  detailItem.value = null
}

function seedModalPlatforms() {
  modalSelectedPlatforms.value = [...targetPlatforms.value]
}

function openSingleInstallModal(item: MarketplaceItem) {
  closeDetail()
  pendingItem.value = item
  pendingBatchPackages.value = []
  platformModalMode.value = 'single'
  seedModalPlatforms()
  platformModalOpen.value = true
}

function openBatchInstallModal() {
  if (batchSelection.value.size === 0) {
    return
  }

  pendingItem.value = null
  pendingBatchPackages.value = [...batchSelection.value]
  platformModalMode.value = 'batch'
  seedModalPlatforms()
  platformModalOpen.value = true
}

function closePlatformModal() {
  platformModalOpen.value = false
  pendingItem.value = null
  pendingBatchPackages.value = []
}

function selectDetectedPlatforms() {
  modalSelectedPlatforms.value = [...detectedPlatforms.value]
}

function updateModalSelectedPlatforms(value: string[]) {
  modalSelectedPlatforms.value = value
}

async function confirmPlatformInstall() {
  const selected = modalSelectedPlatforms.value as Platform[]
  if (selected.length === 0) {
    return
  }

  const packages = platformModalMode.value === 'batch'
    ? [...pendingBatchPackages.value]
    : pendingItem.value
      ? [pendingItem.value.package]
      : []

  if (packages.length === 0) {
    closePlatformModal()
    return
  }

  installingPackages.value = new Set([...installingPackages.value, ...packages])
  platformModalOpen.value = false

  try {
    if (platformModalMode.value === 'batch') {
      const response = await batchInstall({
        packages,
        agents: selected,
      })
      if (response.failCount > 0) {
        throw new Error(`${response.failCount} installs failed`)
      }
      batchMode.value = false
      clearBatchSelection()
      uiStore.showSuccess(`Installed ${response.successCount} marketplace skills`)
    } else if (pendingItem.value) {
      await install({
        sourceKind: 'marketplace',
        sourceRef: pendingItem.value.package,
        sourceSkillId: pendingItem.value.skill,
        targetPlatforms: selected,
      })
      uiStore.showSuccess('Marketplace skill installed')
    }
  }
  catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  } finally {
    const remaining = new Set(installingPackages.value)
    packages.forEach((pkg) => remaining.delete(pkg))
    installingPackages.value = remaining
    pendingItem.value = null
    pendingBatchPackages.value = []
  }
}

async function handleRefresh() {
  isRefreshing.value = true
  try {
    await loadMarketplace(true)
  } finally {
    isRefreshing.value = false
  }
}

function handleSearch() {
  batchMode.value = false
  clearBatchSelection()
  emit('update:search', searchQuery.value.trim())
}

async function installManual(kind: 'github' | 'local' | 'npx') {
  if (targetPlatforms.value.length === 0) {
    uiStore.showError(noPlatformHint)
    return
  }

  const sourceRef = kind === 'github'
    ? manualGithub.value.trim()
    : kind === 'local'
      ? manualLocalPath.value.trim()
      : manualNpx.value.trim()

  if (!sourceRef) {
    return
  }

  try {
    await install({
      sourceKind: kind,
      sourceRef,
      targetPlatforms: targetPlatforms.value,
    })
    if (kind === 'github') manualGithub.value = ''
    if (kind === 'local') manualLocalPath.value = ''
    if (kind === 'npx') manualNpx.value = ''
    uiStore.showSuccess(`Installed via ${kind}`)
  }
  catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function handlePickFolder() {
  try {
    const path = await browseFolder()
    if (path) {
      manualLocalPath.value = path
    }
  }
  catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}
</script>

<style scoped>
.marketplace-layout {
  @apply grid gap-4 xl:grid-cols-[minmax(0,1fr)_320px];
}

.marketplace-layout__browse {
  @apply min-w-0;
}

.manual-actions {
  @apply flex h-fit flex-col gap-3;
}

.manual-target-summary {
  @apply flex items-center justify-between rounded-2xl border border-border-default/45 p-3 text-sm text-text-secondary;

  background: rgb(var(--color-bg-base-rgb) / 55%);
}

.field {
  @apply flex flex-col gap-2;
}

.field__label {
  @apply text-xs font-semibold uppercase tracking-[0.16em] text-text-muted;
}

.field__input {
  @apply w-full rounded-xl border border-border-default/55 px-3 py-2 text-sm text-text-primary;

  background: rgb(var(--color-bg-base-rgb) / 55%);
}

.field__row {
  @apply flex gap-2;
}

.npx-status {
  @apply flex items-center gap-2 text-xs text-text-secondary;
}

.npx-dot {
  @apply h-2.5 w-2.5 rounded-full bg-white/25;
}

.npx-dot--ok {
  @apply bg-emerald-400;
}

.npx-dot--off {
  @apply bg-rose-400;
}
</style>
