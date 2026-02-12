<template>
  <div class="marketplace-tab">
    <!-- Search Bar -->
    <div class="marketplace-search">
      <div class="relative flex-1">
        <Search
          class="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-text-muted"
        />
        <input
          v-model="searchQuery"
          type="text"
          class="w-full bg-bg-surface border border-border-subtle rounded-xl
                 text-text-primary placeholder:text-text-muted/50
                 pl-12 pr-4 py-3 text-sm font-medium
                 focus:outline-none focus:ring-2 focus:ring-accent-primary/30
                 focus:border-accent-primary/50 transition-all"
          :placeholder="$t('skills.searchMarketplace')"
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
      v-else-if="!isLoading && items.length === 0"
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

    <!-- Marketplace Grid -->
    <div
      v-else
      class="marketplace-grid"
    >
      <div
        v-for="item in items"
        :key="item.package"
        class="marketplace-card group"
        @click="$emit('install', item)"
      >
        <!-- Header -->
        <div class="marketplace-card__header">
          <div class="flex items-center gap-2">
            <Github class="w-4 h-4 text-text-muted" />
            <span class="text-xs text-text-secondary">{{ item.owner }}</span>
          </div>
          <button
            class="marketplace-card__install"
            :title="$t('skills.install')"
            @click.stop="$emit('install', item)"
          >
            <Download class="w-4 h-4" />
          </button>
        </div>

        <!-- Body -->
        <div class="marketplace-card__body">
          <div class="marketplace-card__icon">
            <Package class="w-6 h-6 text-accent-secondary" />
          </div>
          <div class="marketplace-card__info">
            <h3 class="marketplace-card__name">
              {{ item.skill || item.package }}
            </h3>
            <p class="marketplace-card__repo">
              {{ item.repo }}
            </p>
          </div>
        </div>

        <!-- Footer -->
        <div class="marketplace-card__footer">
          <a
            :href="`https://github.com/${item.owner}/${item.repo}`"
            target="_blank"
            class="marketplace-card__link"
            @click.stop
          >
            <ExternalLink class="w-3 h-3" />
            {{ $t('skills.viewOnGithub') }}
          </a>
        </div>

        <!-- Hover Glow -->
        <div class="marketplace-card__glow" />
      </div>
    </div>

    <!-- Loading Skeleton -->
    <div
      v-if="isLoading && items.length === 0"
      class="marketplace-grid"
    >
      <div
        v-for="i in 6"
        :key="i"
        class="skeleton-card"
      >
        <div class="skeleton-header">
          <div class="skeleton-owner" />
          <div class="skeleton-button" />
        </div>
        <div class="skeleton-body">
          <div class="skeleton-icon" />
          <div class="skeleton-info">
            <div class="skeleton-name" />
            <div class="skeleton-repo" />
          </div>
        </div>
        <div class="skeleton-footer">
          <div class="skeleton-link" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import {
  Search,
  Loader2,
  AlertCircle,
  Store,
  Github,
  Download,
  Package,
  ExternalLink
} from 'lucide-vue-next'
import type { MarketplaceItem } from '@/types/skills'

defineProps<{
  items: MarketplaceItem[]
  isLoading: boolean
  error: string | null
}>()

const emit = defineEmits<{
  (e: 'install', item: MarketplaceItem): void
  (e: 'search', query: string): void
}>()

const searchQuery = ref('')

function handleSearch() {
  emit('search', searchQuery.value)
}
</script>

<style scoped>
.marketplace-tab {
  @apply mt-4 space-y-4;
}

.marketplace-search {
  @apply flex gap-2;
}

.btn-search {
  @apply flex items-center gap-2 px-4 py-3 rounded-xl
         text-sm font-semibold text-white
         bg-accent-primary hover:bg-accent-primary/90
         disabled:opacity-50 transition-colors;
}

.error-state,
.empty-state {
  @apply flex flex-col items-center justify-center py-16
         rounded-2xl border border-border-subtle;

  background: rgb(var(--color-bg-surface-rgb) / 30%);
}

.marketplace-grid {
  @apply flex flex-col gap-3;
}

.marketplace-card {
  @apply relative flex flex-col p-4 rounded-2xl cursor-pointer
         backdrop-blur-sm
         border border-border-subtle
         transition-all duration-200 ease-out
         hover:shadow-lg;

  background: rgb(var(--color-bg-elevated-rgb) / 50%);

  &:hover {
    border-color: rgb(var(--color-accent-secondary-rgb) / 30%);
  }
}

.marketplace-card__header {
  @apply flex items-center justify-between mb-3;
}

.marketplace-card__install {
  @apply p-2 rounded-lg text-text-muted
         opacity-0 group-hover:opacity-100
         transition-all duration-200;

  &:hover {
    background: rgb(var(--color-success-rgb) / 10%);
    color: rgb(var(--color-success-rgb));
  }
}

.marketplace-card__body {
  @apply flex items-start gap-3 flex-1;
}

.marketplace-card__icon {
  @apply flex items-center justify-center w-10 h-10 rounded-xl shrink-0;

  background: rgb(var(--color-accent-secondary-rgb) / 10%);
}

.marketplace-card__info {
  @apply flex flex-col gap-1 min-w-0;
}

.marketplace-card__name {
  @apply text-base font-bold text-text-primary truncate;
}

.marketplace-card__repo {
  @apply text-xs text-text-secondary truncate;
}

.marketplace-card__footer {
  @apply flex items-center justify-between mt-auto pt-3
         border-t border-border-subtle;
}

.marketplace-card__link {
  @apply flex items-center gap-1 text-xs text-text-muted
         hover:text-accent-primary transition-colors;
}

.marketplace-card__glow {
  @apply absolute inset-0 rounded-2xl opacity-0
         group-hover:opacity-100 transition-opacity duration-300
         pointer-events-none -z-10;

  background: radial-gradient(
    circle at 50% 50%,
    #a78bfa 0%,
    transparent 70%
  );
  filter: blur(30px);
  transform: scale(0.8);
}

/* Skeleton Styles */
.skeleton-card {
  @apply flex flex-col p-4 rounded-2xl
         border border-border-subtle;

  background: rgb(var(--color-bg-elevated-rgb) / 30%);
}

.skeleton-header {
  @apply flex items-center justify-between mb-3;
}

.skeleton-owner {
  @apply w-20 h-4 rounded bg-bg-surface animate-pulse;
}

.skeleton-button {
  @apply w-8 h-8 rounded-lg bg-bg-surface animate-pulse;
}

.skeleton-body {
  @apply flex items-start gap-3 flex-1;
}

.skeleton-icon {
  @apply w-10 h-10 rounded-xl bg-bg-surface animate-pulse shrink-0;
}

.skeleton-info {
  @apply flex flex-col gap-2 flex-1;
}

.skeleton-name {
  @apply w-32 h-5 rounded bg-bg-surface animate-pulse;
}

.skeleton-repo {
  @apply w-24 h-4 rounded bg-bg-surface animate-pulse;
}

.skeleton-footer {
  @apply flex items-center mt-auto pt-3 border-t border-border-subtle;
}

.skeleton-link {
  @apply w-24 h-4 rounded bg-bg-surface animate-pulse;
}
</style>
