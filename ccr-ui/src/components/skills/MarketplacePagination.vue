<template>
  <div
    v-if="totalPages > 1"
    class="mp-pagination"
  >
    <span class="mp-pagination__info">
      {{ $t('skills.paginationInfo', { start: startItem, end: endItem, total: totalItems }) }}
    </span>

    <div class="mp-pagination__controls">
      <button
        class="mp-pagination__btn"
        :disabled="currentPage <= 1"
        @click="goTo(1)"
      >
        <ChevronsLeft class="w-4 h-4" />
      </button>
      <button
        class="mp-pagination__btn"
        :disabled="currentPage <= 1"
        @click="goTo(currentPage - 1)"
      >
        <ChevronLeft class="w-4 h-4" />
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
        <ChevronRight class="w-4 h-4" />
      </button>
      <button
        class="mp-pagination__btn"
        :disabled="currentPage >= totalPages"
        @click="goTo(totalPages)"
      >
        <ChevronsRight class="w-4 h-4" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight
} from 'lucide-vue-next'

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

// 生成可见页码列表（含省略号 -1）
const visiblePages = computed(() => {
  const total = totalPages.value
  const current = props.currentPage
  const pages: number[] = []

  if (total <= 7) {
    for (let i = 1; i <= total; i++) pages.push(i)
    return pages
  }

  // 始终显示第1页
  pages.push(1)

  if (current > 3) {
    pages.push(-1) // 省略号
  }

  // 中间页码
  const start = Math.max(2, current - 1)
  const end = Math.min(total - 1, current + 1)
  for (let i = start; i <= end; i++) {
    pages.push(i)
  }

  if (current < total - 2) {
    pages.push(-1) // 省略号
  }

  // 始终显示最后一页
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
  @apply p-1.5 rounded-lg text-text-muted
         hover:text-text-primary hover:bg-bg-surface
         disabled:opacity-30 disabled:cursor-not-allowed
         transition-colors;
}

.mp-pagination__page {
  @apply min-w-[32px] h-8 rounded-lg text-sm font-medium text-text-secondary
         hover:text-text-primary hover:bg-bg-surface
         transition-colors;
}

.mp-pagination__page--active {
  @apply text-white bg-accent-primary hover:bg-accent-primary;
}

.mp-pagination__ellipsis {
  @apply min-w-[32px] h-8 text-sm text-text-muted cursor-default;
}
</style>
