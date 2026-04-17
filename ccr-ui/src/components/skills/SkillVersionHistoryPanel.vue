<template>
  <section class="version-panel">
    <header class="version-panel__header">
      <div>
        <h3 class="version-panel__title">
          {{ t('skillsExt.versionHistory.title') }}
        </h3>
        <p class="version-panel__subtitle">
          {{ t('skillsExt.versionHistory.subtitle', { count: history.length }) }}
        </p>
      </div>
      <div class="version-panel__actions">
        <button
          class="console-button"
          :disabled="!installPath || loading"
          :title="t('skillsExt.versionHistory.takeSnapshot')"
          @click="handleSnapshot"
        >
          {{ t('skillsExt.versionHistory.takeSnapshot') }}
        </button>
        <button
          class="console-button"
          :disabled="loading"
          :title="t('skillsExt.versionHistory.refresh')"
          @click="refresh"
        >
          {{ t('skillsExt.versionHistory.refresh') }}
        </button>
      </div>
    </header>

    <p
      v-if="error"
      class="version-panel__error"
    >
      {{ error }}
    </p>

    <div
      v-if="history.length === 0 && !loading"
      class="version-panel__empty"
    >
      {{ t('skillsExt.versionHistory.empty') }}
    </div>

    <ol
      v-else
      class="version-panel__list"
    >
      <li
        v-for="entry in history"
        :key="entry.id"
        class="version-panel__row"
        :class="{ 'version-panel__row--selected': selectedId === entry.id }"
      >
        <button
          type="button"
          class="version-panel__row-main"
          @click="select(entry.id)"
        >
          <span class="version-panel__id">{{ entry.id.slice(0, 8) }}</span>
          <span class="version-panel__message">{{ entry.message || '(no message)' }}</span>
          <span class="version-panel__meta">
            <span class="badge">{{ entry.source }}</span>
            <time>{{ formatTime(entry.timestamp) }}</time>
          </span>
        </button>
        <div class="version-panel__row-actions">
          <button
            v-if="compareBase && compareBase !== entry.id"
            class="console-button"
            :disabled="loading"
            @click="diffAgainstBase(entry.id)"
          >
            {{ t('skillsExt.versionHistory.diff') }}
          </button>
          <button
            class="console-button"
            :disabled="loading"
            @click="setCompareBase(entry.id)"
          >
            {{ compareBase === entry.id ? t('skillsExt.versionHistory.baseActive') : t('skillsExt.versionHistory.baseInactive') }}
          </button>
          <button
            class="console-button console-button--danger"
            :disabled="loading"
            @click="confirmRollback(entry.id)"
          >
            {{ t('skillsExt.versionHistory.rollback') }}
          </button>
        </div>
      </li>
    </ol>
  </section>
</template>

<script setup lang="ts">
import { ref, toRef, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSkillVersions } from '@/composables/useSkillVersions'
import type { DiffResult, VersionMeta } from '@/types/skillVersioning'

const { t } = useI18n()

const props = defineProps<{
  installPath: string | null
  skillName: string
}>()

const emit = defineEmits<{
  (e: 'diff', result: DiffResult): void
  (e: 'selected', meta: VersionMeta): void
  (e: 'rolled-back', meta: VersionMeta): void
}>()

const installPathRef = toRef(props, 'installPath')
const {
  history,
  loading,
  error,
  refresh,
  loadDiff,
  rollback,
  takeSnapshot,
} = useSkillVersions(() => installPathRef.value)

const selectedId = ref<string | null>(null)
const compareBase = ref<string | null>(null)

function select(id: string) {
  selectedId.value = id
  const entry = history.value.find(v => v.id === id)
  if (entry) emit('selected', entry)
}

function setCompareBase(id: string) {
  compareBase.value = compareBase.value === id ? null : id
}

async function diffAgainstBase(otherId: string) {
  if (!compareBase.value) return
  const result = await loadDiff(compareBase.value, otherId)
  if (result) emit('diff', result)
}

async function handleSnapshot() {
  const defaultMsg = t('skillsExt.versionHistory.defaultMessage')
  const message = window.prompt(t('skillsExt.versionHistory.snapshotPrompt'), defaultMsg)
  if (message == null) return
  await takeSnapshot(props.skillName, message, 'manual')
}

async function confirmRollback(id: string) {
  const entry = history.value.find(v => v.id === id)
  if (!entry) return
  const ok = window.confirm(
    t('skillsExt.versionHistory.rollbackConfirm', {
      id: entry.id.slice(0, 8),
      message: entry.message || '(no message)',
    }),
  )
  if (!ok) return
  const meta = await rollback(id)
  if (meta) emit('rolled-back', meta)
}

function formatTime(iso: string): string {
  try {
    return new Date(iso).toLocaleString()
  } catch {
    return iso
  }
}

watch(installPathRef, () => {
  selectedId.value = null
  compareBase.value = null
})
</script>

<style scoped>
.version-panel {
  @apply flex flex-col gap-3 rounded-3xl p-4;

  background: var(--surface-workspace-bg);
  border: 1px solid var(--surface-workspace-border);
  box-shadow: var(--elevation-2);
}

.version-panel__header {
  @apply flex items-center justify-between gap-3;
}

.version-panel__title {
  @apply text-xs font-semibold uppercase tracking-[0.16em] text-text-muted;
}

.version-panel__subtitle {
  @apply text-xs text-text-muted;
}

.version-panel__actions {
  @apply flex items-center gap-2;
}

.version-panel__error {
  @apply rounded-xl border border-rose-500/50 bg-rose-500/10 px-3 py-2 text-sm text-rose-400;
}

.version-panel__empty {
  @apply rounded-2xl border border-border-default/40 p-6 text-center text-sm text-text-muted;
}

.version-panel__list {
  @apply flex max-h-[60vh] flex-col gap-2 overflow-auto;
}

.version-panel__row {
  @apply flex items-start justify-between gap-2 rounded-2xl border border-border-default/50 p-3;

  background-color: rgb(var(--color-bg-base-rgb) / 55%);
}

.version-panel__row--selected {
  border-color: rgb(var(--color-accent-primary-rgb) / 60%);
  background: linear-gradient(
    135deg,
    rgb(var(--color-accent-primary-rgb) / 12%),
    rgb(var(--color-accent-secondary-rgb) / 6%)
  );
}

.version-panel__row-main {
  @apply flex min-w-0 flex-1 flex-col items-start gap-1 text-left;
}

.version-panel__id {
  @apply font-mono text-xs text-text-muted;
}

.version-panel__message {
  @apply truncate text-sm font-semibold text-text-primary;

  max-width: 100%;
}

.version-panel__meta {
  @apply flex items-center gap-2 text-xs text-text-muted;
}

.version-panel__row-actions {
  @apply flex flex-shrink-0 flex-col items-end gap-1;
}

.console-button {
  @apply inline-flex items-center gap-2 rounded-xl border border-border-default/55 px-2.5 py-1 text-xs text-text-secondary;

  background: var(--surface-status-bg);
  border-color: var(--surface-status-border);
}

.console-button--danger {
  @apply text-rose-300;

  border-color: rgb(244 63 94 / 40%);
}

.badge {
  @apply inline-flex items-center rounded-full border border-border-default/45 px-2 py-0.5 text-[10px] uppercase text-text-secondary;
}
</style>
