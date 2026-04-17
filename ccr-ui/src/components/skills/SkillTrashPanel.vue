<template>
  <section class="trash-panel">
    <header class="trash-panel__header">
      <div>
        <h3 class="trash-panel__title">
          {{ t('skillsExt.trash.title') }}
        </h3>
        <p class="trash-panel__subtitle">
          {{ t('skillsExt.trash.subtitle', { count }) }}
        </p>
      </div>
      <button
        class="console-button"
        :disabled="loading"
        @click="refresh"
      >
        {{ t('skillsExt.versionHistory.refresh') }}
      </button>
    </header>

    <p
      v-if="error"
      class="trash-panel__error"
    >
      {{ error }}
    </p>

    <div
      v-if="entries.length === 0 && !loading"
      class="trash-panel__empty"
    >
      {{ t('skillsExt.trash.empty') }}
    </div>

    <ul
      v-else
      class="trash-panel__list"
    >
      <li
        v-for="entry in entries"
        :key="entry.id"
        class="trash-panel__row"
      >
        <div class="trash-panel__row-main">
          <strong class="trash-panel__name">{{ entry.skillName }}</strong>
          <code class="trash-panel__path">{{ entry.originalPath }}</code>
          <span class="trash-panel__meta">
            {{ t('skillsExt.trash.deletedAt', { time: formatTime(entry.deletedAt) }) }}
            ·
            {{ t('skillsExt.trash.expiresAt', { time: formatTime(entry.expiresAt) }) }}
          </span>
        </div>
        <div class="trash-panel__row-actions">
          <button
            class="console-button console-button--primary"
            :disabled="loading"
            @click="handleRestore(entry.id)"
          >
            {{ t('skillsExt.trash.restore') }}
          </button>
          <button
            class="console-button console-button--danger"
            :disabled="loading"
            @click="handlePurge(entry.id)"
          >
            {{ t('skillsExt.trash.deleteForever') }}
          </button>
        </div>
      </li>
    </ul>
  </section>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSkillTrash } from '@/composables/useSkillTrash'

const { t } = useI18n()

const emit = defineEmits<{
  (e: 'restored', path: string): void
  (e: 'purged', trashId: string): void
}>()

const { entries, count, loading, error, refresh, restore, purge } = useSkillTrash()

onMounted(() => {
  void refresh()
})

async function handleRestore(id: string) {
  const path = await restore(id)
  if (path) emit('restored', path)
}

async function handlePurge(id: string) {
  const ok = window.confirm(t('skillsExt.trash.purgeConfirm'))
  if (!ok) return
  const purged = await purge(id)
  if (purged) emit('purged', id)
}

function formatTime(iso: string): string {
  try {
    return new Date(iso).toLocaleString()
  } catch {
    return iso
  }
}
</script>

<style scoped>
.trash-panel {
  @apply flex flex-col gap-3 rounded-3xl p-4;

  background: var(--surface-workspace-bg);
  border: 1px solid var(--surface-workspace-border);
  box-shadow: var(--elevation-2);
}

.trash-panel__header {
  @apply flex items-center justify-between gap-3;
}

.trash-panel__title {
  @apply text-xs font-semibold uppercase tracking-[0.16em] text-text-muted;
}

.trash-panel__subtitle {
  @apply text-xs text-text-muted;
}

.trash-panel__error {
  @apply rounded-xl border border-rose-500/50 bg-rose-500/10 px-3 py-2 text-sm text-rose-400;
}

.trash-panel__empty {
  @apply rounded-2xl border border-border-default/40 p-6 text-center text-sm text-text-muted;
}

.trash-panel__list {
  @apply flex flex-col gap-2;
}

.trash-panel__row {
  @apply flex items-center justify-between gap-3 rounded-2xl border border-border-default/50 p-3;

  background-color: rgb(var(--color-bg-base-rgb) / 55%);
}

.trash-panel__row-main {
  @apply flex min-w-0 flex-col gap-1;
}

.trash-panel__name {
  @apply truncate text-sm font-semibold text-text-primary;
}

.trash-panel__path {
  @apply truncate font-mono text-xs text-text-muted;
}

.trash-panel__meta {
  @apply text-[11px] text-text-muted;
}

.trash-panel__row-actions {
  @apply flex flex-shrink-0 items-center gap-2;
}

.console-button {
  @apply inline-flex items-center gap-2 rounded-xl border border-border-default/55 px-2.5 py-1 text-xs text-text-secondary;

  background: var(--surface-status-bg);
  border-color: var(--surface-status-border);
}

.console-button--primary {
  @apply text-text-primary;

  background: linear-gradient(
    180deg,
    rgb(var(--color-accent-primary-rgb) / 18%),
    rgb(var(--color-accent-secondary-rgb) / 10%)
  );
}

.console-button--danger {
  @apply text-rose-300;

  border-color: rgb(244 63 94 / 40%);
}
</style>
