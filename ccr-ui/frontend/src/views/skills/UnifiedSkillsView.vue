<template>
  <div class="unified-skills-view">
    <!-- Header -->
    <div class="skills-header">
      <div class="skills-header__info">
        <h1 class="skills-header__title">
          {{ $t('skills.title') }}
          <span
            v-if="stats.installed > 0"
            class="skills-header__badge"
          >
            {{ stats.installed }}
          </span>
        </h1>
        <p class="skills-header__subtitle">
          {{ $t('skills.subtitle') }}
        </p>
      </div>
      <div class="skills-header__actions">
        <!-- Mobile Filter Toggle -->
        <button
          class="btn-filter lg:hidden"
          @click="showMobileFilter = true"
        >
          <Filter class="w-4 h-4" />
          <span
            v-if="hasActiveFilters"
            class="filter-badge"
          />
        </button>
        <!-- Operation Log -->
        <button
          class="btn-log"
          :title="$t('skills.operationLog')"
          @click="showLogModal = true"
        >
          <ScrollText class="w-4 h-4" />
        </button>
        <button
          class="btn-refresh"
          :disabled="isLoading"
          @click="handleRefresh"
        >
          <RefreshCw
            class="w-4 h-4"
            :class="{ 'animate-spin': isLoading }"
          />
        </button>
      </div>
    </div>

    <!-- Two-Column Layout -->
    <div class="skills-layout">
      <!-- Left: Filter Panel (Desktop) -->
      <SkillsFilterPanel
        v-model="filterPanelValue"
        :platforms="platforms"
        :categories="availableCategories"
        :tags="availableTags"
        :collapsed="filterPanelCollapsed"
        class="hidden lg:block"
        @toggle-collapse="filterPanelCollapsed = !filterPanelCollapsed"
      />

      <!-- Right: Main Content -->
      <div class="skills-main">
        <!-- Stats Cards -->
        <SkillsStatsCards
          :stats="stats"
          :platforms="platforms"
          :cached="marketplaceCached"
          :active-platform="filters.platform"
        />

        <!-- Content Tabs -->
        <div class="content-tabs">
          <button
            v-for="tab in contentTabs"
            :key="tab.id"
            class="content-tab"
            :class="{ 'content-tab--active': activeTab === tab.id }"
            @click="setActiveTab(tab.id)"
          >
            <component
              :is="tab.icon"
              class="w-4 h-4"
            />
            <span>{{ $t(tab.label) }}</span>
            <span
              v-if="tab.count > 0"
              class="content-tab__count"
            >{{ tab.count }}</span>
          </button>
        </div>

        <!-- Main Content Area -->
        <div class="skills-content">
          <!-- Loading State -->
          <div
            v-if="isLoading && filteredSkills.length === 0"
            class="loading-state"
          >
            <Loader2 class="w-8 h-8 animate-spin text-accent-primary" />
            <p class="text-text-secondary mt-2">
              {{ $t('common.loading') }}
            </p>
          </div>

          <!-- Error State -->
          <div
            v-else-if="error"
            class="error-state"
          >
            <AlertCircle class="w-8 h-8 text-danger" />
            <p class="text-danger mt-2">
              {{ error }}
            </p>
            <button
              class="btn-retry mt-4"
              @click="handleRefresh"
            >
              {{ $t('common.retry') }}
            </button>
          </div>

          <!-- Installed Tab -->
          <SkillsInstalledTab
            v-else-if="activeTab === 'installed'"
            :skills="filteredSkills"
            :is-loading="isLoading"
            @edit="handleEditSkill"
            @delete="handleDeleteSkill"
            @click="handleSkillClick"
          />

          <!-- Marketplace Tab -->
          <SkillsMarketplaceTab
            v-else-if="activeTab === 'marketplace'"
            :items="marketplaceItems"
            :is-loading="isMarketplaceLoading"
            :error="marketplaceError"
            @install="handleInstallFromMarketplace"
            @search="handleMarketplaceSearch"
          />

          <!-- Repositories Tab (Placeholder) -->
          <div
            v-else-if="activeTab === 'repositories'"
            class="repositories-placeholder"
          >
            <FolderGit2 class="w-12 h-12 text-text-muted" />
            <p class="text-text-secondary mt-2">
              {{ $t('skills.repositoriesComingSoon') }}
            </p>
          </div>
        </div>
      </div>
    </div>

    <!-- Mobile Filter Drawer -->
    <Teleport to="body">
      <Transition name="drawer-fade">
        <div
          v-if="showMobileFilter"
          class="mobile-filter-overlay"
          @click.self="showMobileFilter = false"
        >
          <Transition name="drawer-slide">
            <div
              v-if="showMobileFilter"
              class="mobile-filter-drawer"
            >
              <div class="mobile-filter-header">
                <h3 class="text-lg font-bold text-text-primary">
                  {{ $t('skills.filters') }}
                </h3>
                <button
                  class="p-2 rounded-lg text-text-muted hover:text-text-primary
                         hover:bg-bg-surface transition-colors"
                  @click="showMobileFilter = false"
                >
                  <X class="w-5 h-5" />
                </button>
              </div>
              <SkillsFilterPanel
                v-model="filterPanelValue"
                :platforms="platforms"
                :categories="availableCategories"
                :tags="availableTags"
                :collapsed="false"
                class="mobile-filter-content"
              />
            </div>
          </Transition>
        </div>
      </Transition>
    </Teleport>

    <!-- Install Modal -->
    <SkillInstallModal
      v-model="showInstallModal"
      :skill="selectedSkillForInstall"
      :marketplace-item="selectedMarketplaceItem"
      :platforms="platforms"
      @install="handleInstall"
    />

    <!-- Detail / Edit Modal -->
    <SkillDetailModal
      v-model="showDetailModal"
      :skill="selectedSkillForDetail"
      :initial-mode="detailModalMode"
      @saved="refresh"
    />

    <!-- Delete Confirm Modal -->
    <SkillDeleteConfirmModal
      v-model="showDeleteModal"
      :skill="selectedSkillForDelete"
      @confirm="confirmDeleteSkill"
    />

    <!-- Operation Log Modal -->
    <SkillOperationLogModal v-model="showLogModal" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  RefreshCw,
  Loader2,
  AlertCircle,
  Package,
  Store,
  FolderGit2,
  Filter,
  ScrollText,
  X
} from 'lucide-vue-next'
import { useUnifiedSkills } from '@/composables/useUnifiedSkills'
import { useUIStore } from '@/stores/ui'
import SkillsFilterPanel from '@/components/skills/SkillsFilterPanel.vue'
import SkillsStatsCards from '@/components/skills/SkillsStatsCards.vue'
import SkillsInstalledTab from '@/components/skills/SkillsInstalledTab.vue'
import SkillsMarketplaceTab from '@/components/skills/SkillsMarketplaceTab.vue'
import SkillInstallModal from '@/components/skills/SkillInstallModal.vue'
import SkillDetailModal from '@/components/skills/SkillDetailModal.vue'
import SkillDeleteConfirmModal from '@/components/skills/SkillDeleteConfirmModal.vue'
import SkillOperationLogModal from '@/components/skills/SkillOperationLogModal.vue'
import type { Platform, UnifiedSkill, MarketplaceItem, ContentTab, SkillFilters } from '@/types/skills'

const { t } = useI18n()
const uiStore = useUIStore()

const {
  platforms,
  marketplaceItems,
  isLoading,
  isMarketplaceLoading,
  error,
  marketplaceError,
  marketplaceCached,
  filters,
  activeTab,
  filteredSkills,
  availableCategories,
  availableTags,
  stats,
  setFilter,
  setActiveTab,
  initialize,
  refresh,
  fetchMarketplaceTrending,
  searchMarketplace,
  installSkill,
  removeSkill
} = useUnifiedSkills()

// Local state
const showInstallModal = ref(false)
const selectedSkillForInstall = ref<UnifiedSkill | undefined>()
const selectedMarketplaceItem = ref<MarketplaceItem | undefined>()

// Detail modal state
const showDetailModal = ref(false)
const selectedSkillForDetail = ref<UnifiedSkill | undefined>()
const detailModalMode = ref<'view' | 'edit'>('view')

// Delete confirm modal state
const showDeleteModal = ref(false)
const selectedSkillForDelete = ref<UnifiedSkill | undefined>()

// Operation log modal state
const showLogModal = ref(false)

// Filter panel state
const filterPanelCollapsed = ref(false)
const showMobileFilter = ref(false)

// Filter panel v-model binding (includes platform)
const filterPanelValue = computed({
  get: (): SkillFilters => ({
    search: filters.value.search,
    source: filters.value.source,
    category: filters.value.category,
    tags: filters.value.tags,
    platform: filters.value.platform
  }),
  set: (value: SkillFilters) => {
    setFilter('search', value.search)
    setFilter('source', value.source)
    setFilter('category', value.category)
    setFilter('tags', value.tags)
    setFilter('platform', value.platform)
  }
})

// Check if any filters are active
const hasActiveFilters = computed(() => {
  return filters.value.search !== '' ||
         filters.value.source !== 'all' ||
         filters.value.category !== null ||
         filters.value.tags.length > 0 ||
         filters.value.platform !== 'all'
})

// Content tabs configuration
const contentTabs = computed(() => [
  {
    id: 'installed' as ContentTab,
    label: 'skills.tabInstalled',
    icon: Package,
    count: filteredSkills.value.length
  },
  {
    id: 'marketplace' as ContentTab,
    label: 'skills.tabMarketplace',
    icon: Store,
    count: marketplaceItems.value.length
  },
  {
    id: 'repositories' as ContentTab,
    label: 'skills.tabRepositories',
    icon: FolderGit2,
    count: 0
  }
])

// Event handlers
async function handleRefresh() {
  await refresh()
}

function handleSkillClick(skill: UnifiedSkill) {
  selectedSkillForDetail.value = skill
  detailModalMode.value = 'view'
  showDetailModal.value = true
}

function handleEditSkill(skill: UnifiedSkill) {
  selectedSkillForDetail.value = skill
  detailModalMode.value = 'edit'
  showDetailModal.value = true
}

function handleDeleteSkill(skill: UnifiedSkill) {
  selectedSkillForDelete.value = skill
  showDeleteModal.value = true
}

async function confirmDeleteSkill(skill: UnifiedSkill) {
  try {
    await removeSkill({
      skill: skill.name,
      agents: [skill.platform]
    })
    showDeleteModal.value = false
    uiStore.showSuccess(t('skills.deleteSuccess'))
  } catch (err: any) {
    uiStore.showError(t('skills.deleteFailed') + ': ' + (err.message || ''))
  }
}

function handleInstallFromMarketplace(item: MarketplaceItem) {
  selectedMarketplaceItem.value = item
  selectedSkillForInstall.value = undefined
  showInstallModal.value = true
}

async function handleInstall(targetPlatforms: Platform[]) {
  if (!selectedMarketplaceItem.value) return

  try {
    await installSkill({
      package: selectedMarketplaceItem.value.skillsShUrl,
      agents: targetPlatforms,
      force: false
    })
    showInstallModal.value = false
  } catch (err) {
    console.error('Failed to install skill:', err)
  }
}

async function handleMarketplaceSearch(query: string) {
  if (query.trim()) {
    await searchMarketplace(query)
  } else {
    await fetchMarketplaceTrending()
  }
}

// Initialize on mount
onMounted(() => {
  initialize()
})

// Watch for tab changes to fetch marketplace data
watch(activeTab, (newTab) => {
  if (newTab === 'marketplace' && marketplaceItems.value.length === 0) {
    fetchMarketplaceTrending()
  }
})
</script>

<style scoped>
.unified-skills-view {
  @apply flex flex-col gap-4 px-4 py-4;
}

.skills-header {
  @apply flex items-center justify-between;
}

.skills-header__info {
  @apply flex flex-col gap-1;
}

.skills-header__title {
  @apply flex items-center gap-2 text-2xl font-bold text-text-primary;
}

.skills-header__badge {
  @apply px-2 py-0.5 rounded-full text-sm font-bold
         bg-accent-primary/10 text-accent-primary;
}

.skills-header__subtitle {
  @apply text-sm text-text-secondary;
}

.skills-header__actions {
  @apply flex items-center gap-2;
}

.btn-refresh {
  @apply p-2 rounded-xl bg-bg-surface hover:bg-bg-elevated
         text-text-secondary hover:text-text-primary
         border border-border-subtle hover:border-border-default
         transition-all duration-200 disabled:opacity-50;
}

.btn-log {
  @apply p-2 rounded-xl bg-bg-surface hover:bg-bg-elevated
         text-text-secondary hover:text-text-primary
         border border-border-subtle hover:border-border-default
         transition-all duration-200;
}

.btn-filter {
  @apply relative p-2 rounded-xl bg-bg-surface hover:bg-bg-elevated
         text-text-secondary hover:text-text-primary
         border border-border-subtle hover:border-border-default
         transition-all duration-200;
}

.filter-badge {
  @apply absolute -top-1 -right-1 w-2.5 h-2.5 rounded-full
         bg-accent-primary animate-pulse;
}

/* Two-Column Layout */
.skills-layout {
  @apply grid gap-6 mt-2;

  grid-template-columns: auto 1fr;
}

@media (width <= 1023px) {
  .skills-layout {
    grid-template-columns: 1fr;
  }
}

.skills-main {
  @apply flex flex-col gap-4 min-w-0;
}

.content-tabs {
  @apply flex gap-2 p-1 rounded-xl border border-border-subtle;

  background: rgb(var(--color-bg-surface-rgb) / 50%);
}

.content-tab {
  @apply flex items-center gap-2 px-4 py-2 rounded-lg
         text-sm font-medium text-text-secondary
         hover:text-text-primary
         transition-all duration-200;
}

.content-tab:hover {
  background: rgb(var(--color-bg-elevated-rgb) / 50%);
}

.content-tab--active {
  @apply text-text-primary bg-bg-elevated shadow-sm;
}

.content-tab__count {
  @apply px-1.5 py-0.5 rounded-md text-xs font-bold
         bg-accent-primary/10 text-accent-primary;
}

.skills-content {
  @apply min-h-[400px];
}

.loading-state,
.error-state,
.repositories-placeholder {
  @apply flex flex-col items-center justify-center py-16;
}

.btn-retry {
  @apply px-4 py-2 rounded-xl text-sm font-semibold
         bg-accent-primary text-white hover:bg-accent-primary/90
         transition-colors;
}

/* Mobile Filter Drawer */
.mobile-filter-overlay {
  @apply fixed inset-0 z-50 bg-black/50 backdrop-blur-sm;
}

.mobile-filter-drawer {
  @apply fixed left-0 top-0 bottom-0 w-[300px] max-w-[85vw]
         bg-bg-base border-r border-border-default
         shadow-2xl flex flex-col;
}

.mobile-filter-header {
  @apply flex items-center justify-between p-4
         border-b border-border-subtle;
}

.mobile-filter-content {
  @apply flex-1 overflow-y-auto !w-full !rounded-none !border-0;
}

/* Drawer Transitions */
.drawer-fade-enter-active,
.drawer-fade-leave-active {
  transition: opacity 0.2s ease;
}

.drawer-fade-enter-from,
.drawer-fade-leave-to {
  opacity: 0;
}

.drawer-slide-enter-active,
.drawer-slide-leave-active {
  transition: transform 0.3s ease;
}

.drawer-slide-enter-from,
.drawer-slide-leave-to {
  transform: translateX(-100%);
}
</style>
