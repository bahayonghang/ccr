<template>
  <section
    v-if="output"
    class="sync-output-card"
    :class="statusClass"
  >
    <header class="sync-output-card__header">
      <div class="sync-output-card__heading">
        <div class="sync-output-card__icon">
          <SIcon
            :name="statusIcon"
            size="w-5 h-5"
          />
        </div>
        <div>
          <p class="sync-output-card__eyebrow">
            {{ $t('sync.output.title') }}
          </p>
          <h2>{{ output.title }}</h2>
          <p class="sync-output-card__summary">
            {{ output.summary }}
          </p>
        </div>
      </div>
      <button
        type="button"
        class="sync-output-card__close"
        :aria-label="$t('common.close')"
        @click="clearOutput"
      >
        <SIcon
          name="XCircle"
          size="w-4 h-4"
        />
      </button>
    </header>

    <div class="sync-output-card__metrics">
      <div class="sync-output-card__metric">
        <span>{{ $t('sync.output.successMetric') }}</span>
        <strong>{{ successRatioText }}</strong>
      </div>
      <div class="sync-output-card__metric">
        <span>{{ $t('sync.output.failedMetric') }}</span>
        <strong>{{ output.failedCount }}</strong>
      </div>
      <div class="sync-output-card__metric">
        <span>{{ $t('sync.output.durationMetric') }}</span>
        <strong>{{ durationText }}</strong>
      </div>
    </div>

    <section
      v-if="output.suggestions.length > 0"
      class="sync-output-card__advice"
    >
      <p class="sync-output-card__section-title">
        {{ $t('sync.output.suggestionsTitle') }}
      </p>
      <ul>
        <li
          v-for="suggestion in output.suggestions"
          :key="suggestion"
        >
          {{ suggestion }}
        </li>
      </ul>
    </section>

    <section
      v-if="output.failures.length > 0"
      class="sync-output-card__failures"
    >
      <p class="sync-output-card__section-title">
        {{ $t('sync.output.failuresTitle', { count: output.failures.length }) }}
      </p>
      <article
        v-for="failure in output.failures"
        :key="`${failure.assetId ?? failure.assetName}-${failure.message}`"
        class="sync-output-card__failure"
      >
        <header>
          <strong>{{ failure.assetName }}</strong>
          <span>{{ failure.reason }}</span>
        </header>
        <dl>
          <div v-if="failure.remotePath">
            <dt>{{ $t('sync.assets.remotePath') }}</dt>
            <dd :title="failure.remotePath">
              {{ failure.remotePath }}
            </dd>
          </div>
          <div v-if="failure.localPath">
            <dt>{{ $t('sync.assets.localPath') }}</dt>
            <dd :title="failure.localPath">
              {{ failure.localPath }}
            </dd>
          </div>
        </dl>
        <p class="sync-output-card__failure-message">
          {{ failure.message }}
        </p>
      </article>
    </section>

    <details class="sync-output-card__raw">
      <summary>{{ $t('sync.output.rawDetails') }}</summary>
      <div class="sync-output-card__raw-body">
        <button
          type="button"
          class="sync-output-card__copy"
          @click="copyRawDetails"
        >
          <SIcon
            name="Copy"
            size="w-4 h-4"
          />
          {{ copied ? $t('sync.output.copied') : $t('sync.output.copyRaw') }}
        </button>
        <pre>{{ output.rawLog }}</pre>
      </div>
    </details>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import { copyText } from '@/utils/clipboard'
import type { SyncOperationOutput } from '@/types/syncSelection'

interface Props {
  output: SyncOperationOutput | null
  clearOutput: () => void
}

const props = defineProps<Props>()
const { t } = useI18n()
const copied = ref(false)

const statusClass = computed(() => `sync-output-card--${props.output?.status ?? 'success'}`)

const statusIcon = computed(() => {
  if (props.output?.status === 'success') return 'CheckCircle'
  if (props.output?.status === 'partial') return 'AlertTriangle'
  return 'XCircle'
})

const successRatioText = computed(() => {
  const successCount = props.output?.successCount ?? 0
  const total = props.output?.total
  if (typeof total === 'number') {
    return t('sync.output.successRatio', { success: successCount, total })
  }
  return t('sync.output.successCountText', { success: successCount })
})

const durationText = computed(() => {
  const durationMs = props.output?.durationMs
  if (typeof durationMs !== 'number') return '—'
  if (durationMs < 1000) return `${durationMs} ms`
  return `${(durationMs / 1000).toFixed(1)} s`
})

const copyRawDetails = async () => {
  if (!props.output) return
  const ok = await copyText(props.output.rawLog)
  if (!ok) return
  copied.value = true
  window.setTimeout(() => {
    copied.value = false
  }, 1600)
}
</script>

<style scoped>
.sync-output-card {
  @apply rounded-3xl p-5;

  border: 1px solid rgb(var(--color-border-default-rgb) / 38%);
  background: var(--color-bg-surface);
  box-shadow: var(--surface-card-shadow);
}

.sync-output-card--success {
  border-color: rgb(var(--color-success-rgb) / 32%);
}

.sync-output-card--partial {
  border-color: rgb(var(--color-warning-rgb) / 34%);
}

.sync-output-card--failed {
  border-color: rgb(var(--color-danger-rgb) / 32%);
}

.sync-output-card__header {
  @apply flex items-start justify-between gap-3;
}

.sync-output-card__heading {
  @apply flex min-w-0 gap-3;
}

.sync-output-card__icon {
  @apply flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl;

  background: rgb(var(--color-bg-base-rgb) / 62%);
  color: var(--color-text-secondary);
}

.sync-output-card--success .sync-output-card__icon {
  background: rgb(var(--color-success-rgb) / 12%);
  color: var(--accent-success);
}

.sync-output-card--partial .sync-output-card__icon {
  background: rgb(var(--color-warning-rgb) / 13%);
  color: var(--accent-warning);
}

.sync-output-card--failed .sync-output-card__icon {
  background: rgb(var(--color-danger-rgb) / 11%);
  color: var(--accent-danger);
}

.sync-output-card__eyebrow,
.sync-output-card__section-title {
  @apply text-xs font-bold uppercase tracking-[0.15em];

  color: var(--color-text-muted);
}

.sync-output-card h2 {
  @apply mt-1 text-lg font-semibold tracking-[-0.02em];

  color: var(--color-text-primary);
}

.sync-output-card__summary {
  @apply mt-2 break-words text-sm leading-6;

  color: var(--color-text-secondary);
}

.sync-output-card__close {
  @apply inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-xl border transition-all duration-200;

  border-color: rgb(var(--color-border-default-rgb) / 34%);
  background: rgb(var(--color-bg-elevated-rgb) / 64%);
  color: var(--color-text-muted);
}

.sync-output-card__close:hover {
  color: var(--color-text-primary);
  transform: translateY(-1px);
}

.sync-output-card__metrics {
  @apply mt-5 grid grid-cols-3 gap-2;
}

.sync-output-card__metric {
  @apply min-w-0 rounded-2xl px-3 py-2;

  border: 1px solid rgb(var(--color-border-default-rgb) / 30%);
  background: rgb(var(--color-bg-base-rgb) / 48%);
}

.sync-output-card__metric span {
  @apply block text-[0.68rem] font-bold uppercase tracking-[0.13em];

  color: var(--color-text-muted);
}

.sync-output-card__metric strong {
  @apply mt-1 block truncate text-sm;

  color: var(--color-text-primary);
}

.sync-output-card__advice,
.sync-output-card__failures,
.sync-output-card__raw {
  @apply mt-4 rounded-2xl p-4;

  border: 1px solid rgb(var(--color-border-default-rgb) / 30%);
  background: rgb(var(--color-bg-base-rgb) / 42%);
}

.sync-output-card__advice {
  border-color: rgb(var(--color-warning-rgb) / 28%);
  background: rgb(var(--color-warning-rgb) / 9%);
}

.sync-output-card__advice ul {
  @apply mt-2 space-y-1 text-sm leading-6;

  color: var(--color-text-secondary);
}

.sync-output-card__advice li {
  @apply break-words;
}

.sync-output-card__failure {
  @apply mt-3 rounded-xl p-3;

  border: 1px solid rgb(var(--color-border-default-rgb) / 26%);
  background: rgb(var(--color-bg-elevated-rgb) / 54%);
}

.sync-output-card__failure header {
  @apply flex flex-col gap-1;
}

.sync-output-card__failure strong {
  @apply text-sm;

  color: var(--color-text-primary);
}

.sync-output-card__failure header span {
  @apply text-sm leading-6;

  color: var(--color-text-secondary);
}

.sync-output-card__failure dl {
  @apply mt-3 grid grid-cols-1 gap-2;
}

.sync-output-card__failure dt {
  @apply text-[0.68rem] font-bold uppercase tracking-[0.13em];

  color: var(--color-text-muted);
}

.sync-output-card__failure dd {
  @apply mt-1 break-all font-mono text-xs;

  color: var(--color-text-secondary);
}

.sync-output-card__failure-message {
  @apply mt-3 break-words font-mono text-xs leading-5;

  color: var(--color-text-muted);
}

.sync-output-card__raw {
  @apply p-0;
}

.sync-output-card__raw summary {
  @apply cursor-pointer px-4 py-3 text-sm font-semibold;

  color: var(--color-text-secondary);
}

.sync-output-card__raw-body {
  @apply border-t p-4;

  border-color: rgb(var(--color-border-default-rgb) / 28%);
}

.sync-output-card__copy {
  @apply mb-3 inline-flex items-center gap-2 rounded-xl border px-3 py-2 text-xs font-semibold transition-all duration-200;

  border-color: rgb(var(--color-border-default-rgb) / 34%);
  background: rgb(var(--color-bg-elevated-rgb) / 64%);
  color: var(--color-text-secondary);
}

.sync-output-card__copy:hover {
  color: var(--color-text-primary);
  transform: translateY(-1px);
}

.sync-output-card__raw pre {
  @apply max-h-80 overflow-auto whitespace-pre-wrap break-words rounded-xl p-3 font-mono text-xs leading-5;

  border: 1px solid rgb(var(--color-border-default-rgb) / 26%);
  background: rgb(var(--color-bg-elevated-rgb) / 62%);
  color: var(--color-text-secondary);
}

.sync-output-card__close:focus-visible,
.sync-output-card__copy:focus-visible,
.sync-output-card__raw summary:focus-visible {
  outline: 2px solid rgb(var(--color-accent-primary-rgb) / 54%);
  outline-offset: 2px;
}
</style>
