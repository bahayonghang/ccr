<template>
  <div class="installed-tab">
    <!-- Empty State -->
    <AsyncStatePanel
      v-if="!isLoading && skills.length === 0"
      state="empty"
      icon="Package"
      :title="$t('skills.noSkillsInstalled')"
      :description="$t('skills.noSkillsInstalledHint')"
      compact
    />

    <!-- Skills List (virtualized) -->
    <div
      v-else-if="skills.length > 0"
      ref="scrollRef"
      class="skills-viewport"
      data-testid="skills-installed-viewport"
    >
      <div
        class="skills-list"
        :style="{ height: `${rowVirtualizer.getTotalSize()}px` }"
      >
        <div
          v-for="virtualRow in virtualItems"
          :key="`${skills[virtualRow.index]?.platform}-${skills[virtualRow.index]?.name}`"
          class="skills-list__item"
          data-testid="skills-installed-row"
          :style="{ transform: `translateY(${virtualRow.start}px)` }"
        >
          <div
            :ref="measureElement"
            :data-index="virtualRow.index"
          >
            <SkillCard
              :skill="skills[virtualRow.index]"
              @click="$emit('click', skills[virtualRow.index])"
              @edit="$emit('edit', skills[virtualRow.index])"
              @delete="$emit('delete', skills[virtualRow.index])"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Loading Skeleton (single column list) -->
    <div
      v-if="isLoading && skills.length === 0"
      class="skills-skeleton-list"
    >
      <div
        v-for="i in 4"
        :key="i"
        class="skeleton-row"
      >
        <!-- Left: platform icon skeleton -->
        <div class="skeleton-platform">
          <div class="skeleton-icon" />
          <div class="skeleton-badge" />
        </div>
        <!-- Center: info skeleton -->
        <div class="skeleton-body">
          <div class="skeleton-title-row">
            <div class="skeleton-name" />
            <div class="skeleton-category" />
          </div>
          <div class="skeleton-description" />
          <div class="skeleton-tags">
            <div class="skeleton-tag" />
            <div class="skeleton-tag" />
            <div class="skeleton-tag" />
          </div>
          <div class="skeleton-path" />
        </div>
        <!-- Right: actions skeleton -->
        <div class="skeleton-actions">
          <div class="skeleton-action-btn" />
          <div class="skeleton-action-btn" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { AsyncStatePanel } from '@/components/ui'
import SkillCard from '@/components/skills/SkillCard.vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { computed, ref } from 'vue'
import type { UnifiedSkill } from '@/types/skills'

const props = defineProps<{
  skills: UnifiedSkill[]
  isLoading: boolean
}>()

defineEmits<{
  (e: 'click', skill: UnifiedSkill): void
  (e: 'edit', skill: UnifiedSkill): void
  (e: 'delete', skill: UnifiedSkill): void
}>()

const scrollRef = ref<HTMLElement | null>(null)

const rowVirtualizer = useVirtualizer(computed(() => ({
  count: props.skills.length,
  getScrollElement: () => scrollRef.value,
  estimateSize: () => 184,
  overscan: 8,
})))

const virtualItems = computed(() => rowVirtualizer.value.getVirtualItems())

const measureElement = (element: unknown) => {
  rowVirtualizer.value.measureElement(element instanceof Element ? element : null)
}
</script>

<style scoped>
.installed-tab {
  @apply mt-4;
}

.skills-viewport {
  @apply overflow-y-auto pr-1;

  height: clamp(26rem, calc(100vh - 20rem), 64rem);
}

.skills-list {
  @apply relative w-full;
}

.skills-list__item {
  @apply absolute left-0 top-0 w-full pb-3;
}

.skills-skeleton-list {
  @apply flex flex-col gap-3;
}

/* Skeleton Styles - horizontal row layout */
.skeleton-row {
  @apply flex flex-row items-start gap-4 p-4 rounded-2xl border border-white/5;

  background: rgb(var(--color-bg-elevated-rgb) / 88%);
}

.skeleton-platform {
  @apply flex flex-col items-center gap-2 shrink-0 w-16;
}

.skeleton-icon {
  @apply w-12 h-12 rounded-xl animate-pulse border border-white/10;

  background: rgb(var(--color-bg-overlay-rgb) / 72%);
}

.skeleton-badge {
  @apply w-14 h-4 rounded-full animate-pulse border border-white/10;

  background: rgb(var(--color-bg-overlay-rgb) / 72%);
}

.skeleton-body {
  @apply flex flex-col gap-2 flex-1;
}

.skeleton-title-row {
  @apply flex items-center gap-2;
}

.skeleton-name {
  @apply w-36 h-5 rounded animate-pulse border border-white/10;

  background: rgb(var(--color-bg-overlay-rgb) / 72%);
}

.skeleton-category {
  @apply w-20 h-4 rounded-md animate-pulse border border-white/10;

  background: rgb(var(--color-bg-overlay-rgb) / 72%);
}

.skeleton-description {
  @apply w-full h-10 rounded animate-pulse border border-white/10;

  background: rgb(var(--color-bg-overlay-rgb) / 72%);
}

.skeleton-tags {
  @apply flex gap-1;
}

.skeleton-tag {
  @apply w-14 h-5 rounded-md animate-pulse border border-white/10;

  background: rgb(var(--color-bg-overlay-rgb) / 72%);
}

.skeleton-path {
  @apply w-48 h-4 rounded animate-pulse border border-white/10;

  background: rgb(var(--color-bg-overlay-rgb) / 72%);
}

.skeleton-actions {
  @apply flex flex-col gap-1 shrink-0;
}

.skeleton-action-btn {
  @apply w-8 h-8 rounded-lg animate-pulse border border-white/10;

  background: rgb(var(--color-bg-overlay-rgb) / 72%);
}
</style>
