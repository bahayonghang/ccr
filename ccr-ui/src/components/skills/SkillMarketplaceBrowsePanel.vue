<template>
  <section class="browse-section">
    <div class="section-header">
      <div class="section-header__left">
        <h2
          class="section-title"
          data-testid="marketplace-title"
        >
          <SIcon
            :name="contentMode === 'search' ? 'Search' : 'TrendingUp'"
            size="w-5 h-5"
            class="text-accent-primary"
          />
          {{ headerTitle }}
        </h2>
        <span
          class="section-hint"
          data-testid="marketplace-hint"
        >{{ headerHint }}</span>
      </div>
      <div class="section-header__right">
        <span
          v-if="marketplaceCached"
          class="cache-badge"
        >
          <SIcon
            name="Database"
            size="w-3 h-3"
          />
          {{ $t('skills.cacheStatus') }}
        </span>
        <button
          class="btn-refresh"
          :disabled="isRefreshing"
          @click="emit('refresh')"
        >
          <SIcon
            name="RefreshCw"
            size="w-4 h-4"
            :class="{ 'animate-spin': isRefreshing }"
          />
          <span>{{ refreshLabel }}</span>
        </button>
      </div>
    </div>

    <div class="browse-controls">
      <div class="browse-search">
        <div class="relative flex-1">
          <SIcon
            name="Search"
            size="w-5 h-5"
            class="absolute left-4 top-1/2 -translate-y-1/2 text-white/50"
          />
          <input
            :value="searchQuery"
            type="text"
            class="search-input"
            :placeholder="$t('skills.searchMarketplace')"
            data-testid="marketplace-search-input"
            @input="handleSearchInput"
            @keyup.enter="handleSearchSubmit"
          >
        </div>
        <button
          class="btn-search"
          :disabled="isMarketplaceLoading"
          @click="handleSearchSubmit"
        >
          <SIcon
            v-if="isMarketplaceLoading"
            name="Loader2"
            size="w-4 h-4"
            class="animate-spin"
          />
          <SIcon
            v-else
            name="Search"
            size="w-4 h-4"
          />
          <span>{{ $t('common.search') }}</span>
        </button>
      </div>

      <div class="browse-toolbar">
        <div class="toolbar-left">
          <span
            v-if="sortedItems.length > 0"
            class="result-badge"
            data-testid="marketplace-result-badge"
          >
            {{ resultSummary }}
          </span>
        </div>
        <div class="toolbar-right">
          <div class="sort-select">
            <SIcon
              name="ArrowUpDown"
              size="w-3.5 h-3.5"
              class="text-white/50"
            />
            <select
              :value="sortBy"
              class="sort-dropdown"
              @change="updateSortBy"
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
            @click="toggleBatchMode"
          >
            <SIcon
              name="CheckSquare"
              size="w-4 h-4"
            />
            <span>{{ $t('skills.batchMode') }}</span>
          </button>
        </div>
      </div>
    </div>

    <div
      v-if="!hasDetectedPlatforms"
      class="state-box state-box--warning"
      data-testid="no-platform-blocking"
    >
      <SIcon
        name="Laptop"
        size="w-8 h-8"
        class="text-warning"
      />
      <h3 class="mt-3 text-lg font-semibold text-white">
        {{ $t('skills.noDetectedPlatformsTitle') }}
      </h3>
      <p class="mt-1 text-center text-sm text-white/75">
        {{ noPlatformHint }}
      </p>
    </div>

    <div
      v-if="contentState === 'error'"
      class="state-box state-box--error"
    >
      <SIcon
        name="AlertCircle"
        size="w-8 h-8"
        class="text-danger"
      />
      <p class="mt-2 text-danger">
        {{ marketplaceError }}
      </p>
    </div>

    <div
      v-else-if="contentState === 'empty'"
      class="state-box"
      data-testid="marketplace-empty-state"
    >
      <SIcon
        :name="contentMode === 'search' ? 'SearchX' : 'Store'"
        size="w-12 h-12"
        class="text-white/50"
      />
      <h3 class="mt-4 text-lg font-semibold text-white">
        {{ emptyTitle }}
      </h3>
      <p class="mt-1 text-sm text-white/80">
        {{ emptyHint }}
      </p>
    </div>

    <div
      v-else-if="contentState === 'loading'"
      class="marketplace-grid"
    >
      <div
        v-for="index in 8"
        :key="index"
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

    <div
      v-else
      class="marketplace-grid"
      data-testid="marketplace-grid"
    >
      <MarketplaceSkillCard
        v-for="item in pagedItems"
        :key="item.package"
        :batch-mode="batchMode"
        :install-disabled="!hasDetectedPlatforms"
        :is-installing="isInstalling(item.package)"
        :is-installed="isSkillInstalled(item)"
        :is-selected="isBatchSelected(item.package)"
        :item="item"
        @install="emit('marketplace-install', item)"
        @toggle-batch="emit('toggle-batch', item)"
        @view-detail="emit('view-detail', item)"
      />
    </div>

    <MarketplacePagination
      v-if="contentState === 'ready'"
      :current-page="currentPage"
      :page-size="pageSize"
      :total-items="sortedItems.length"
      @page-change="emit('update:currentPage', $event)"
    />

    <Transition name="batch-bar">
      <div
        v-if="batchSelectedCount > 0"
        class="batch-bar"
        data-testid="marketplace-batch-bar"
      >
        <span class="batch-bar__count">
          {{ $t('skills.selectedCount', { count: batchSelectedCount }) }}
        </span>
        <div class="batch-bar__actions">
          <button
            class="batch-bar__clear"
            @click="emit('clear-batch-selected')"
          >
            {{ $t('skills.clearAll') }}
          </button>
          <button
            class="batch-bar__install"
            :disabled="!hasDetectedPlatforms"
            @click="emit('batch-install')"
          >
            <SIcon
              :name="hasDetectedPlatforms ? 'Download' : 'AlertTriangle'"
              size="w-4 h-4"
            />
            {{ hasDetectedPlatforms ? $t('skills.batchInstall') : $t('skills.noPlatformsDetectedShort') }}
          </button>
        </div>
      </div>
    </Transition>
  </section>
</template>

<script setup lang="ts">
import { computed, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import MarketplacePagination from '@/components/skills/MarketplacePagination.vue'
import MarketplaceSkillCard from '@/components/skills/MarketplaceSkillCard.vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { MarketplaceItem } from '@/types/skills'

type MarketplaceSort = 'stars' | 'name'
type ContentMode = 'trending' | 'search'
type ContentState = 'loading' | 'error' | 'empty' | 'ready'

interface Props {
  batchMode: boolean
  batchSelectedCount: number
  contentMode: ContentMode
  contentState: ContentState
  currentPage: number
  hasDetectedPlatforms: boolean
  isBatchSelected: (pkg: string) => boolean
  isInstalling: (pkg: string) => boolean
  isMarketplaceLoading: boolean
  isRefreshing: boolean
  isSkillInstalled: (item: MarketplaceItem) => boolean
  marketplaceCached: boolean
  marketplaceError: string | null
  marketplaceItems: MarketplaceItem[]
  noPlatformHint: string
  pageSize: number
  pagedItems: MarketplaceItem[]
  searchQuery: string
  sortBy: MarketplaceSort
  sortedItems: MarketplaceItem[]
}

const props = defineProps<Props>()
const { t } = useI18n()

const emit = defineEmits<{
  'batch-install': []
  'clear-batch-selected': []
  'marketplace-install': [item: MarketplaceItem]
  refresh: []
  search: []
  'toggle-batch': [item: MarketplaceItem]
  'update:batchMode': [value: boolean]
  'update:currentPage': [value: number]
  'update:searchQuery': [value: string]
  'update:sortBy': [value: MarketplaceSort]
  'view-detail': [item: MarketplaceItem]
}>()

let searchTimer: ReturnType<typeof setTimeout> | null = null

const trimmedSearchQuery = computed(() => props.searchQuery.trim())
const headerTitle = computed(() => {
  return props.contentMode === 'search'
    ? t('skills.searchResultsTitle', { query: trimmedSearchQuery.value })
    : t('skills.browseTrending')
})

const headerHint = computed(() => {
  return props.contentMode === 'search'
    ? t('skills.searchResultsHint')
    : t('skills.browseTrendingHint')
})

const refreshLabel = computed(() => {
  if (props.isRefreshing) {
    return props.contentMode === 'search' ? t('skills.refreshingSearch') : t('skills.refreshingTrending')
  }
  return props.contentMode === 'search' ? t('skills.refreshSearch') : t('skills.refreshTrending')
})

const resultSummary = computed(() => {
  return props.contentMode === 'search'
    ? t('skills.searchResultCount', { count: props.sortedItems.length })
    : `${props.sortedItems.length} ${t('skills.resultCount')}`
})

const emptyTitle = computed(() => {
  return props.contentMode === 'search'
    ? t('skills.noMarketplaceResults')
    : t('skills.noTrendingResults')
})

const emptyHint = computed(() => {
  return props.contentMode === 'search'
    ? t('skills.tryDifferentSearch')
    : t('skills.tryRefreshTrending')
})

const handleSearchInput = (event: Event) => {
  const value = (event.target as HTMLInputElement).value
  emit('update:searchQuery', value)

  if (searchTimer) {
    clearTimeout(searchTimer)
  }

  searchTimer = setTimeout(() => {
    emit('search')
  }, 300)
}

const handleSearchSubmit = () => {
  if (searchTimer) {
    clearTimeout(searchTimer)
  }

  emit('search')
}

const updateSortBy = (event: Event) => {
  const value = (event.target as HTMLSelectElement).value as MarketplaceSort
  emit('update:sortBy', value)
}

const toggleBatchMode = () => {
  const next = !props.batchMode
  emit('update:batchMode', next)
  if (!next) {
    emit('clear-batch-selected')
  }
}

onUnmounted(() => {
  if (searchTimer) {
    clearTimeout(searchTimer)
  }
})
</script>

<style scoped>
.browse-section {
  @apply flex flex-col gap-4 rounded-2xl border border-white/5 p-5;

  background: rgb(0 0 0 / 30%);
}

.section-header {
  @apply flex flex-wrap items-center justify-between gap-3;
}

.section-header__left {
  @apply flex items-center gap-3;
}

.section-header__right {
  @apply flex items-center gap-2;
}

.section-title {
  @apply flex items-center gap-2 text-lg font-bold text-white;
}

.section-hint {
  @apply text-xs text-white/50;
}

.cache-badge {
  @apply flex items-center gap-1 rounded-lg px-2.5 py-1 text-xs font-medium;

  color: rgb(var(--color-success-rgb));
  background: rgb(var(--color-success-rgb) / 10%);
}

.btn-refresh {
  @apply glass-surface flex items-center gap-1.5 rounded-lg border border-white/5 px-3 py-1.5
         text-xs font-medium text-white/80 transition-colors
         hover:border-white/10 hover:text-white disabled:opacity-50;
}

.browse-controls {
  @apply flex flex-col gap-3;
}

.browse-search {
  @apply flex gap-2;
}

.search-input {
  @apply glass-surface w-full rounded-xl border border-white/5
         py-3 pl-12 pr-4 text-sm font-medium text-white
         transition-[border-color,box-shadow]
         focus:border-accent-primary/50 focus:outline-none focus:ring-2 focus:ring-accent-primary/30;
}

.search-input::placeholder {
  color: rgb(var(--color-text-muted-rgb) / 50%);
}

.btn-search {
  @apply flex items-center gap-2 rounded-xl bg-accent-primary px-4 py-3
         text-sm font-semibold text-white transition-colors
         hover:bg-accent-primary/90 disabled:opacity-50;
}

.browse-toolbar {
  @apply flex items-center justify-between;
}

.toolbar-left,
.toolbar-right {
  @apply flex items-center gap-2;
}

.result-badge {
  @apply rounded-lg bg-accent-primary/10 px-2.5 py-1 text-xs font-semibold text-accent-primary;
}

.sort-select {
  @apply glass-surface flex items-center gap-1.5 rounded-lg border border-white/5 px-3 py-2 text-sm text-white/80;
}

.sort-dropdown {
  @apply cursor-pointer border-none bg-transparent text-sm text-white outline-none;
}

.btn-batch {
  @apply glass-surface flex items-center gap-1.5 rounded-lg border border-white/5 px-3 py-2
         text-sm font-medium text-white/80 transition-colors
         hover:border-white/10 hover:text-white;
}

.btn-batch--active {
  @apply border-accent-primary/30 text-accent-primary;

  background: rgb(var(--color-accent-primary-rgb) / 8%);
}

.state-box {
  @apply flex flex-col items-center justify-center rounded-2xl border border-white/5 py-16;

  background: rgb(0 0 0 / 20%);
}

.state-box--error {
  border-color: rgb(var(--color-danger-rgb) / 20%);
}

.state-box--warning {
  border-color: rgb(var(--color-warning-rgb) / 20%);
  background: rgb(var(--color-warning-rgb) / 8%);
}

.marketplace-grid {
  @apply grid gap-4;

  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
}

.skeleton-card {
  @apply flex flex-col gap-3 rounded-2xl border border-white/5 p-4;

  background: rgb(0 0 0 / 30%);
}

.skeleton-header { @apply flex items-center justify-between; }
.skeleton-avatar { @apply glass-surface h-6 w-6 animate-pulse rounded-full; }
.skeleton-owner { @apply glass-surface h-4 w-16 animate-pulse rounded; }
.skeleton-stars { @apply glass-surface h-4 w-12 animate-pulse rounded; }
.skeleton-name { @apply glass-surface h-5 w-32 animate-pulse rounded; }
.skeleton-desc { @apply flex flex-col gap-1.5; }
.skeleton-line { @apply glass-surface h-3.5 animate-pulse rounded; }
.skeleton-footer { @apply mt-auto flex items-center justify-between border-t border-white/5 pt-3; }
.skeleton-link { @apply glass-surface h-4 w-20 animate-pulse rounded; }
.skeleton-btn { @apply glass-surface h-7 w-16 animate-pulse rounded-lg; }

.batch-bar {
  @apply fixed bottom-6 left-1/2 z-40 flex -translate-x-1/2 items-center gap-4 rounded-2xl
         border border-white/5 px-6 py-3 shadow-2xl;

  background: rgb(0 0 0 / 30%);
  backdrop-filter: blur(16px);
}

.batch-bar__count { @apply text-sm font-semibold text-white; }
.batch-bar__actions { @apply flex items-center gap-2; }

.batch-bar__clear {
  @apply px-3 py-1.5 text-sm text-white/80 transition-colors
         hover:bg-white/5 hover:text-white;
}

.batch-bar__install {
  @apply flex items-center gap-1.5 rounded-lg bg-accent-primary px-4 py-1.5
         text-sm font-semibold text-white transition-colors hover:bg-accent-primary/90 disabled:cursor-not-allowed disabled:opacity-60;
}

.batch-bar-enter-active,
.batch-bar-leave-active {
  transition: all 0.3s ease;
}

.batch-bar-enter-from,
.batch-bar-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(20px);
}
</style>
