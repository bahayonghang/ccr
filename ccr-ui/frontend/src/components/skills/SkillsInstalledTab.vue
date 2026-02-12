<template>
  <div class="installed-tab">
    <!-- Empty State -->
    <div
      v-if="!isLoading && skills.length === 0"
      class="empty-state"
    >
      <Package class="w-12 h-12 text-text-muted" />
      <h3 class="text-lg font-semibold text-text-primary mt-4">
        {{ $t('skills.noSkillsInstalled') }}
      </h3>
      <p class="text-sm text-text-secondary mt-1">
        {{ $t('skills.noSkillsInstalledHint') }}
      </p>
    </div>

    <!-- Skills List (single column) -->
    <div
      v-else
      class="skills-list"
    >
      <SkillCard
        v-for="skill in skills"
        :key="`${skill.platform}-${skill.name}`"
        :skill="skill"
        @click="$emit('click', skill)"
        @edit="$emit('edit', skill)"
        @delete="$emit('delete', skill)"
      />
    </div>

    <!-- Loading Skeleton (single column list) -->
    <div
      v-if="isLoading && skills.length === 0"
      class="skills-list"
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
import { Package } from 'lucide-vue-next'
import SkillCard from '@/components/skills/SkillCard.vue'
import type { UnifiedSkill } from '@/types/skills'

defineProps<{
  skills: UnifiedSkill[]
  isLoading: boolean
}>()

defineEmits<{
  (e: 'click', skill: UnifiedSkill): void
  (e: 'edit', skill: UnifiedSkill): void
  (e: 'delete', skill: UnifiedSkill): void
}>()
</script>

<style scoped>
.installed-tab {
  @apply mt-4;
}

.empty-state {
  @apply flex flex-col items-center justify-center py-16
         rounded-2xl border border-border-subtle;

  background: rgb(var(--color-bg-surface-rgb) / 30%);
}

.skills-list {
  @apply flex flex-col gap-3;
}

/* Skeleton Styles - horizontal row layout */
.skeleton-row {
  @apply flex flex-row items-start gap-4 p-4 rounded-2xl border border-border-subtle;

  background: rgb(var(--color-bg-elevated-rgb) / 30%);
}

.skeleton-platform {
  @apply flex flex-col items-center gap-2 shrink-0 w-16;
}

.skeleton-icon {
  @apply w-12 h-12 rounded-xl bg-bg-surface animate-pulse;
}

.skeleton-badge {
  @apply w-14 h-4 rounded-full bg-bg-surface animate-pulse;
}

.skeleton-body {
  @apply flex flex-col gap-2 flex-1;
}

.skeleton-title-row {
  @apply flex items-center gap-2;
}

.skeleton-name {
  @apply w-36 h-5 rounded bg-bg-surface animate-pulse;
}

.skeleton-category {
  @apply w-20 h-4 rounded-md bg-bg-surface animate-pulse;
}

.skeleton-description {
  @apply w-full h-10 rounded bg-bg-surface animate-pulse;
}

.skeleton-tags {
  @apply flex gap-1;
}

.skeleton-tag {
  @apply w-14 h-5 rounded-md bg-bg-surface animate-pulse;
}

.skeleton-path {
  @apply w-48 h-4 rounded bg-bg-surface animate-pulse;
}

.skeleton-actions {
  @apply flex flex-col gap-1 shrink-0;
}

.skeleton-action-btn {
  @apply w-8 h-8 rounded-lg bg-bg-surface animate-pulse;
}
</style>
