<template>
  <section
    v-if="result"
    class="diff-viewer"
  >
    <header class="diff-viewer__header">
      <div class="diff-viewer__title">
        <span class="diff-viewer__version">{{ result.oldVersion.id.slice(0, 8) }}</span>
        <span class="diff-viewer__arrow">→</span>
        <span class="diff-viewer__version">{{ result.newVersion.id.slice(0, 8) }}</span>
      </div>
      <div class="diff-viewer__stats">
        <span class="diff-viewer__stat diff-viewer__stat--add">+{{ result.stats.additions }}</span>
        <span class="diff-viewer__stat diff-viewer__stat--remove">−{{ result.stats.deletions }}</span>
        <span class="diff-viewer__stat">{{ t('skillsExt.diff.same', { count: result.stats.unchanged }) }}</span>
      </div>
    </header>

    <!-- P2-3: 超行截断警告 -->
    <p
      v-if="result.truncation?.truncated"
      class="diff-viewer__truncated"
    >
      ⚠ Diff truncated at {{ result.truncation.limit }} lines.
      Old file: {{ result.truncation.totalOldLines }} lines,
      new file: {{ result.truncation.totalNewLines }} lines.
      Content beyond the limit is not shown.
    </p>

    <div class="diff-viewer__body">
      <div
        v-for="(line, idx) in result.lines"
        :key="idx"
        class="diff-viewer__line"
        :class="`diff-viewer__line--${line.kind}`"
      >
        <span class="diff-viewer__gutter">{{ line.oldLine ?? '' }}</span>
        <span class="diff-viewer__gutter">{{ line.newLine ?? '' }}</span>
        <span class="diff-viewer__marker">{{ markerFor(line.kind) }}</span>
        <span class="diff-viewer__content">{{ line.content }}</span>
      </div>
    </div>
  </section>
  <div
    v-else
    class="diff-viewer__empty"
  >
    {{ t('skillsExt.diff.empty') }}
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { DiffLineKind, DiffResult } from '@/types/skillVersioning'

const { t } = useI18n()

defineProps<{
  result: DiffResult | null
}>()

function markerFor(kind: DiffLineKind): string {
  if (kind === 'add') return '+'
  if (kind === 'remove') return '−'
  return ' '
}
</script>

<style scoped>
.diff-viewer {
  @apply flex flex-col gap-2 rounded-2xl border border-border-default/50 p-3;

  background-color: rgb(var(--color-bg-base-rgb) / 55%);
}

.diff-viewer__header {
  @apply flex items-center justify-between gap-3 border-b border-border-default/40 pb-2;
}

.diff-viewer__title {
  @apply flex items-center gap-2 text-sm text-text-primary;
}

.diff-viewer__version {
  @apply rounded-md border border-border-default/60 px-2 py-0.5 font-mono text-xs;
}

.diff-viewer__arrow {
  @apply text-text-muted;
}

.diff-viewer__stats {
  @apply flex items-center gap-2 text-xs;
}

.diff-viewer__stat {
  @apply rounded-md border border-border-default/45 px-2 py-0.5 text-text-muted;
}

.diff-viewer__stat--add {
  @apply border-emerald-500/40 text-emerald-400;
}

.diff-viewer__stat--remove {
  @apply border-rose-500/40 text-rose-400;
}

.diff-viewer__truncated {
  @apply rounded-xl border border-amber-500/40 bg-amber-500/10 px-3 py-2 text-xs text-amber-300;
}

.diff-viewer__body {
  @apply max-h-[50vh] overflow-auto rounded-xl;

  font-family: var(--font-mono);
}

.diff-viewer__line {
  @apply grid gap-2 px-2 py-0.5 text-xs leading-5;

  grid-template-columns: 2.5rem 2.5rem 1rem 1fr;
}

.diff-viewer__line--add {
  background-color: rgb(34 197 94 / 12%);
  color: rgb(134 239 172);
}

.diff-viewer__line--remove {
  background-color: rgb(244 63 94 / 12%);
  color: rgb(252 165 165);
}

.diff-viewer__line--same {
  @apply text-text-secondary;
}

.diff-viewer__gutter {
  @apply text-right text-text-muted;

  user-select: none;
}

.diff-viewer__marker {
  @apply text-center;
}

.diff-viewer__content {
  @apply whitespace-pre-wrap break-words;
}

.diff-viewer__empty {
  @apply rounded-2xl border border-border-default/40 p-6 text-center text-sm text-text-muted;
}
</style>
