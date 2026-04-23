<template>
  <div
    v-if="totalPages > 1"
    class="mp-pagination"
  >
    <span class="mp-pagination__info">
      {{ infoLabel }}
    </span>

    <div class="mp-pagination__controls">
      <button
        class="mp-pagination__btn"
        :disabled="currentPage <= 1"
        @click="goTo(1)"
      >
        <SIcon
          name="ChevronsLeft"
          size="w-4 h-4"
        />
      </button>
      <button
        class="mp-pagination__btn"
        :disabled="currentPage <= 1"
        @click="goTo(currentPage - 1)"
      >
        <SIcon
          name="ChevronLeft"
          size="w-4 h-4"
        />
      </button>

      <template
        v-for="page in visiblePages"
        :key="page"
      >
        <button
          v-if="page === -1"
          class="mp-pagination__ellipsis"
          disabled
        >
          …
        </button>
        <button
          v-else
          class="mp-pagination__page"
          :class="{ 'mp-pagination__page--active': page === currentPage }"
          @click="goTo(page)"
        >
          {{ page }}
        </button>
      </template>

      <button
        class="mp-pagination__btn"
        :disabled="currentPage >= totalPages"
        @click="goTo(currentPage + 1)"
      >
        <SIcon
          name="ChevronRight"
          size="w-4 h-4"
        />
      </button>
      <button
        class="mp-pagination__btn"
        :disabled="currentPage >= totalPages"
        @click="goTo(totalPages)"
      >
        <SIcon
          name="ChevronsRight"
          size="w-4 h-4"
        />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import SIcon from '@/components/ui/SIcon.vue'

const props = defineProps<{
  currentPage: number
  totalItems: number
  pageSize: number
}>()

const emit = defineEmits<{
  (e: 'page-change', page: number): void
}>()

const totalPages = computed(() =>
  Math.max(1, Math.ceil(props.totalItems / props.pageSize))
)

const startItem = computed(() =>
  Math.min((props.currentPage - 1) * props.pageSize + 1, props.totalItems)
)

const endItem = computed(() =>
  Math.min(props.currentPage * props.pageSize, props.totalItems)
)

const infoLabel = computed(() => `${startItem.value}-${endItem.value} / ${props.totalItems}`)

const visiblePages = computed(() => {
  const total = totalPages.value
  const current = props.currentPage
  const pages: number[] = []

  if (total <= 7) {
    for (let i = 1; i <= total; i++) pages.push(i)
    return pages
  }

  pages.push(1)

  if (current > 3) {
    pages.push(-1)
  }

  const start = Math.max(2, current - 1)
  const end = Math.min(total - 1, current + 1)
  for (let i = start; i <= end; i++) {
    pages.push(i)
  }

  if (current < total - 2) {
    pages.push(-1)
  }

  pages.push(total)

  return pages
})

function goTo(page: number) {
  const clamped = Math.max(1, Math.min(page, totalPages.value))
  if (clamped !== props.currentPage) {
    emit('page-change', clamped)
  }
}
</script>

<style scoped>
.mp-pagination {
  @apply flex items-center justify-between gap-4 py-3;
}

.mp-pagination__info {
  @apply text-xs text-text-muted;
}

.mp-pagination__controls {
  @apply flex items-center gap-1;
}

.mp-pagination__btn {
  @apply rounded-lg p-1.5 text-text-muted
         transition-colors
         hover:bg-bg-surface/70 hover:text-white
         disabled:cursor-not-allowed disabled:opacity-30;
}

.mp-pagination__page {
  @apply h-8 min-w-[32px] rounded-lg text-sm font-medium text-text-primary
         transition-colors
         hover:bg-bg-surface/70 hover:text-white;
}

.mp-pagination__page--active {
  @apply bg-accent-primary text-white hover:bg-accent-primary;
}

.mp-pagination__ellipsis {
  @apply h-8 min-w-[32px] cursor-default text-sm text-text-muted;
}
</style>
