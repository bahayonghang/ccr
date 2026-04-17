<template>
  <section class="category-filter">
    <header class="category-filter__header">
      <h3 class="category-filter__title">
        {{ t('skillsExt.category.title') }}
      </h3>
      <button
        class="console-button"
        :disabled="selectedId === null"
        @click="$emit('select', null)"
      >
        {{ t('skillsExt.category.all') }}
      </button>
    </header>
    <div class="category-filter__chips">
      <button
        v-for="cat in categories"
        :key="cat.id"
        class="category-filter__chip"
        :class="{ 'category-filter__chip--active': selectedId === cat.id }"
        @click="$emit('select', cat.id)"
      >
        <span class="category-filter__icon">{{ cat.icon }}</span>
        <span class="category-filter__name">{{ displayName(cat) }}</span>
        <span class="category-filter__count">{{ cat.count }}</span>
      </button>
    </div>
    <p
      v-if="categories.length === 0"
      class="category-filter__empty"
    >
      {{ t('skillsExt.category.empty') }}
    </p>
  </section>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { CategorySummary } from '@/types/skillVersioning'

const { t, locale: i18nLocale } = useI18n()

const props = defineProps<{
  categories: CategorySummary[]
  selectedId: string | null
  /** 强制指定语言；未指定时跟随 vue-i18n 全局 locale */
  locale?: 'en' | 'zh'
}>()

defineEmits<{
  (e: 'select', id: string | null): void
}>()

function displayName(cat: CategorySummary): string {
  const target = props.locale ?? (i18nLocale.value.startsWith('zh') ? 'zh' : 'en')
  return target === 'zh' ? cat.nameZh : cat.nameEn
}
</script>

<style scoped>
.category-filter {
  @apply flex flex-col gap-3 rounded-3xl p-4;

  background: var(--surface-workspace-bg);
  border: 1px solid var(--surface-workspace-border);
  box-shadow: var(--elevation-2);
}

.category-filter__header {
  @apply flex items-center justify-between gap-3;
}

.category-filter__title {
  @apply text-xs font-semibold uppercase tracking-[0.16em] text-text-muted;
}

.category-filter__chips {
  @apply flex flex-wrap gap-2;
}

.category-filter__chip {
  @apply inline-flex items-center gap-2 rounded-full border border-border-default/55 px-3 py-1 text-xs text-text-secondary transition-colors;

  background: var(--surface-status-bg);
}

.category-filter__chip--active {
  @apply text-text-primary;

  background: linear-gradient(
    180deg,
    rgb(var(--color-accent-primary-rgb) / 20%),
    rgb(var(--color-accent-secondary-rgb) / 10%)
  );
  border-color: rgb(var(--color-accent-primary-rgb) / 40%);
}

.category-filter__icon {
  @apply text-sm;
}

.category-filter__count {
  @apply rounded-full bg-bg-base/60 px-1.5 text-[10px];
}

.category-filter__empty {
  @apply text-xs text-text-muted;
}

.console-button {
  @apply inline-flex items-center gap-2 rounded-xl border border-border-default/55 px-2.5 py-1 text-xs text-text-secondary;

  background: var(--surface-status-bg);
}
</style>
