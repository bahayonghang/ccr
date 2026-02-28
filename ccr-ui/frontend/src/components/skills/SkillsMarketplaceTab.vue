<template>
  <div class="marketplace-tab">
    <!-- Search Bar + Controls -->
    <div class="marketplace-controls">
      <div class="marketplace-search">
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
          :disabled="isLoading"
          @click="handleSearch"
        >
          <Loader2
            v-if="isLoading"
            class="w-4 h-4 animate-spin"
          />
          <Search
            v-else
            class="w-4 h-4"
          />
          <span>{{ $t('common.search') }}</span>
        </button>
      </div>

      <!-- Sort + Batch Controls -->
      <div class="marketplace-toolbar">
        <div class="toolbar-left">
          <!-- Result Count Badge -->
          <span
            v-if="items.length > 0"
            class="result-badge"
          >
            {{ items.length }} {{ $t('skills.resultCount') }}
          </span>
        </div>

        <div class="toolbar-right">
          <!-- Sort Dropdown -->
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

          <!-- Batch Mode Toggle -->
          <button
            class="btn-batch"
            :class="{ 'btn-batch--active': batchMode }"
            @click="toggleBatchMode"
          >
            <CheckSquare class="w-4 h-4" />
            <span>{{ $t('skills.batchMode') }}</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Error State -->
    <div
      v-if="error"
      class="error-state"
    >
      <AlertCircle class="w-8 h-8 text-danger" />
      <p class="text-danger mt-2">
        {{ error }}
      </p>
    </div>

    <!-- Empty State -->
    <div
      v-else-if="!isLoading && sortedItems.length === 0"
      class="empty-state"
    >
      <Store class="w-12 h-12 text-text-muted" />
      <h3 class="text-lg font-semibold text-text-primary mt-4">
        {{ $t('skills.noMarketplaceResults') }}
      </h3>
      <p class="text-sm text-text-secondary mt-1">
        {{ $t('skills.tryDifferentSearch') }}
      </p>
    </div>

    <!-- Marketplace Grid (Card Layout) -->
    <div
      v-else-if="!isLoading"
      class="marketplace-grid"
    >
      <MarketplaceSkillCard
        v-for="item in pagedItems"
        :key="item.package"
        :item="item"
        :is-installed="isSkillInstalled(item)"
        :is-installing="installingPackages.has(item.package)"
        :batch-mode="batchMode"
        :is-selected="batchSelection.has(item.package)"
        @install="handleInstall"
        @select="handleBatchSelect"
        @view-source="handleViewSource"
      />
    </div>

    <!-- Loading Skeleton -->
    <div
      v-if="isLoading"
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

    <!-- Pagination -->
    <MarketplacePagination
      v-if="!isLoading && sortedItems.length > 0"
      :current-page="currentPage"
      :total-items="sortedItems.length"
      :page-size="pageSize"
      @page-change="onPageChange"
    />

    <!-- Batch Action Bar -->
    <Transition name="batch-bar">
      <div
        v-if="batchMode && batchSelection.size > 0"
        class="batch-bar"
      >
        <span class="batch-bar__count">
          {{ $t('skills.selectedCount', { count: batchSelection.size }) }}
        </span>
        <div class="batch-bar__actions">
          <button
            class="batch-bar__clear"
            @click="clearBatchSelection"
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  Search,
  Loader2,
  AlertCircle,
  Store,
  Download,
  ArrowUpDown,
  CheckSquare
} from 'lucide-vue-next'
import MarketplaceSkillCard from './MarketplaceSkillCard.vue'
import MarketplacePagination from './MarketplacePagination.vue'
import type { MarketplaceItem } from '@/types/skills'

const props = defineProps<{
  items: MarketplaceItem[]
  isLoading: boolean
  error: string | null
  installedPackages?: Set<string>
}>()

const emit = defineEmits<{
  (e: 'install', item: MarketplaceItem): void
  (e: 'search', query: string): void
  (e: 'batch-install', packages: string[]): void
}>()

// Search state
const searchQuery = ref('')
const sortBy = ref<'stars' | 'name'>('stars')
const currentPage = ref(1)
const pageSize = 20

// Batch mode
const batchMode = ref(false)
const batchSelection = ref<Set<string>>(new Set())

// Installing packages
const installingPackages = ref<Set<string>>(new Set())

// Debounced search
let searchTimer: ReturnType<typeof setTimeout> | null = null
function onSearchInput() {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    emit('search', searchQuery.value)
  }, 300)
}

function handleSearch() {
  if (searchTimer) clearTimeout(searchTimer)
  emit('search', searchQuery.value)
}

// Sorted items
const sortedItems = computed(() => {
  const list = [...props.items]
  if (sortBy.value === 'stars') {
    list.sort((a, b) => (b.stars ?? 0) - (a.stars ?? 0))
  } else {
    list.sort((a, b) => (a.skill || a.package).localeCompare(b.skill || b.package))
  }
  return list
})

// Paged items
const pagedItems = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  return sortedItems.value.slice(start, start + pageSize)
})

// Check if skill is already installed
function isSkillInstalled(item: MarketplaceItem): boolean {
  return props.installedPackages?.has(item.package) ?? false
}

// Page change
function onPageChange(page: number) {
  currentPage.value = page
}

// Reset page when items change
watch(() => props.items, () => {
  currentPage.value = 1
})

// Batch operations
function toggleBatchMode() {
  batchMode.value = !batchMode.value
  if (!batchMode.value) {
    batchSelection.value = new Set()
  }
}

function handleBatchSelect(item: MarketplaceItem) {
  const newSet = new Set(batchSelection.value)
  if (newSet.has(item.package)) {
    newSet.delete(item.package)
  } else {
    newSet.add(item.package)
  }
  batchSelection.value = newSet
}

function clearBatchSelection() {
  batchSelection.value = new Set()
}

function handleBatchInstall() {
  emit('batch-install', [...batchSelection.value])
}

function handleInstall(item: MarketplaceItem) {
  emit('install', item)
}

function handleViewSource(item: MarketplaceItem) {
  window.open(`https://github.com/${item.owner}/${item.repo}`, '_blank')
}
</script>

<style scoped>
.marketplace-tab {
  @apply mt-4 flex flex-col gap-4;
}

.marketplace-controls {
  @apply flex flex-col gap-3;
}

.marketplace-search {
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

/* Toolbar */
.marketplace-toolbar {
  @apply flex items-center justify-between;
}

.toolbar-left {
  @apply flex items-center gap-2;
}

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

/* States */
.error-state,
.empty-state {
  @apply flex flex-col items-center justify-center py-16
         rounded-2xl border border-border-subtle;

  background: rgb(var(--color-bg-surface-rgb) / 30%);
}

/* Grid Layout */
.marketplace-grid {
  @apply grid gap-4;

  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
}

/* Skeleton Styles */
.skeleton-card {
  @apply flex flex-col gap-3 p-4 rounded-2xl
         border border-border-subtle;

  background: rgb(var(--color-bg-elevated-rgb) / 30%);
}

.skeleton-header {
  @apply flex items-center justify-between;
}

.skeleton-avatar {
  @apply w-6 h-6 rounded-full bg-bg-surface animate-pulse;
}

.skeleton-owner {
  @apply w-16 h-4 rounded bg-bg-surface animate-pulse;
}

.skeleton-stars {
  @apply w-12 h-4 rounded bg-bg-surface animate-pulse;
}

.skeleton-name {
  @apply w-32 h-5 rounded bg-bg-surface animate-pulse;
}

.skeleton-desc {
  @apply flex flex-col gap-1.5;
}

.skeleton-line {
  @apply h-3.5 rounded bg-bg-surface animate-pulse;
}

.skeleton-footer {
  @apply flex items-center justify-between mt-auto pt-3
         border-t border-border-subtle;
}

.skeleton-link {
  @apply w-20 h-4 rounded bg-bg-surface animate-pulse;
}

.skeleton-btn {
  @apply w-16 h-7 rounded-lg bg-bg-surface animate-pulse;
}

/* Batch Bar */
.batch-bar {
  @apply fixed bottom-6 left-1/2 -translate-x-1/2 z-40
         flex items-center gap-4 px-6 py-3 rounded-2xl
         border border-border-subtle shadow-2xl;

  background: rgb(var(--color-bg-elevated-rgb) / 95%);
  backdrop-filter: blur(16px);
}

.batch-bar__count {
  @apply text-sm font-semibold text-text-primary;
}

.batch-bar__actions {
  @apply flex items-center gap-2;
}

.batch-bar__clear {
  @apply px-3 py-1.5 rounded-lg text-sm text-text-secondary
         hover:text-text-primary hover:bg-bg-surface
         transition-colors;
}

.batch-bar__install {
  @apply flex items-center gap-1.5 px-4 py-1.5 rounded-lg
         text-sm font-semibold text-white
         bg-accent-primary hover:bg-accent-primary/90
         transition-colors;
}

/* Batch Bar Animation */
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
</style>
