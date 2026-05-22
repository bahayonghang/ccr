<template>
  <header class="usage-dashboard-toolbar">
    <div class="usage-dashboard-toolbar__copy">
      <p class="usage-dashboard-toolbar__eyebrow">
        {{ $t('usage.dashboard.toolbar.eyebrow') }}
      </p>
      <h1>{{ $t('usage.title') }}</h1>
      <p>{{ $t('usage.subtitle') }}</p>
    </div>

    <div
      v-if="!runtimeUnavailable"
      class="usage-dashboard-toolbar__actions"
    >
      <label class="usage-dashboard-toolbar__field">
        <span>{{ $t('usage.dashboard.toolbar.platform') }}</span>
        <select
          :value="selectedPlatform"
          class="usage-dashboard-toolbar__select"
          @change="emitPlatform(($event.target as HTMLSelectElement).value)"
        >
          <option value="">
            {{ $t('usage.dashboard.allPlatforms') }}
          </option>
          <option value="claude">
            Claude
          </option>
          <option value="codex">
            Codex
          </option>
          <option value="gemini">
            Antigravity CLI
          </option>
          <option value="opencode">
            OpenCode
          </option>
        </select>
      </label>

      <div class="usage-dashboard-toolbar__field usage-dashboard-toolbar__field--segmented">
        <span>{{ $t('usage.dashboard.toolbar.window') }}</span>
        <div class="usage-dashboard-toolbar__segments">
          <button
            v-for="option in rangeOptions"
            :key="option.value"
            type="button"
            class="usage-dashboard-toolbar__segment"
            :class="{ 'usage-dashboard-toolbar__segment--active': selectedRange === option.value }"
            @click="emitRange(option.value)"
          >
            {{ $t(option.key) }}
          </button>
        </div>
      </div>

      <a
        class="usage-dashboard-toolbar__pricing-link"
        href="/pricing"
      >
        {{ $t('usage.dashboard.toolbar.pricing') }}
      </a>

      <Button
        variant="glass"
        density="compact"
        surface="card"
        :elevation="1"
        motion="subtle"
        :disabled="importing"
        @click="$emit('import')"
      >
        {{ importButtonLabel }}
      </Button>
    </div>
  </header>
</template>

<script setup lang="ts">
import Button from '@/components/ui/Button.vue'
import type { UsageRangePreset } from '@/views/usage/dateWindow'

interface Props {
  selectedPlatform: string
  selectedRange: UsageRangePreset
  importButtonLabel: string
  importing: boolean
  runtimeUnavailable: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:selectedPlatform': [value: string]
  'update:selectedRange': [value: UsageRangePreset]
  import: []
}>()

const rangeOptions: Array<{ value: UsageRangePreset; key: string }> = [
  { value: 'today', key: 'usage.dashboard.range.today' },
  { value: 'this_week', key: 'usage.dashboard.range.thisWeek' },
  { value: 'this_month', key: 'usage.dashboard.range.thisMonth' },
  { value: 'last_30d', key: 'usage.dashboard.range.last30' },
  { value: 'all_time', key: 'usage.dashboard.range.allTime' },
]

const emitPlatform = (value: string) => {
  if (value !== props.selectedPlatform) {
    emit('update:selectedPlatform', value)
  }
}

const emitRange = (value: UsageRangePreset) => {
  if (value !== props.selectedRange) {
    emit('update:selectedRange', value)
  }
}
</script>

<style scoped>
.usage-dashboard-toolbar {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 1rem;
  border-radius: 1.45rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 13%);
  background:
    linear-gradient(135deg, rgb(var(--color-bg-elevated-rgb) / 88%), rgb(var(--color-bg-surface-rgb) / 70%)),
    radial-gradient(circle at 12% 0%, rgb(var(--color-accent-primary-rgb) / 8%), transparent 38%);
  padding: 1rem 1.08rem;
  box-shadow: var(--elevation-1), inset 0 1px 0 rgb(255 255 255 / 8%);
}

.usage-dashboard-toolbar__copy {
  display: grid;
  gap: 0.22rem;
  max-width: 46rem;
}

.usage-dashboard-toolbar__eyebrow {
  color: var(--color-text-muted);
  font-size: 0.68rem;
  font-weight: 760;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.usage-dashboard-toolbar h1 {
  color: var(--color-text-primary);
  font-size: clamp(1.35rem, 1vw + 1rem, 1.92rem);
  font-weight: 760;
  letter-spacing: -0.035em;
  line-height: 1.05;
}

.usage-dashboard-toolbar p:not(.usage-dashboard-toolbar__eyebrow) {
  color: var(--color-text-secondary);
  font-size: 0.88rem;
  line-height: 1.45;
}

.usage-dashboard-toolbar__actions {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-end;
  justify-content: flex-end;
  gap: 0.65rem;
}

.usage-dashboard-toolbar__field {
  display: grid;
  gap: 0.28rem;
}

.usage-dashboard-toolbar__field > span {
  color: var(--color-text-muted);
  font-size: 0.64rem;
  font-weight: 750;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.usage-dashboard-toolbar__select {
  min-height: 2.5rem;
  min-width: 8.8rem;
  border-radius: 9999px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 15%);
  background: rgb(var(--color-bg-elevated-rgb) / 64%);
  padding: 0 0.82rem;
  color: var(--color-text-primary);
  font-size: 0.84rem;
  outline: none;
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 8%);
}

.usage-dashboard-toolbar__select:focus {
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
}

.usage-dashboard-toolbar__segments {
  display: inline-flex;
  min-height: 2.5rem;
  align-items: center;
  gap: 0.18rem;
  border-radius: 9999px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  background: rgb(var(--color-bg-elevated-rgb) / 46%);
  padding: 0.22rem;
}

.usage-dashboard-toolbar__segment {
  min-height: 2rem;
  border-radius: 9999px;
  padding: 0 0.65rem;
  color: var(--color-text-secondary);
  font-size: 0.78rem;
  font-weight: 650;
  transition:
    color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.usage-dashboard-toolbar__segment:hover {
  color: var(--color-text-primary);
}

.usage-dashboard-toolbar__segment--active {
  background: rgb(var(--color-accent-primary-rgb) / 13%);
  color: var(--color-text-primary);
  box-shadow: inset 0 0 0 1px rgb(var(--color-accent-primary-rgb) / 16%);
}

.usage-dashboard-toolbar__pricing-link {
  display: inline-flex;
  min-height: 2.5rem;
  align-items: center;
  justify-content: center;
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  border-radius: 9999px;
  background: rgb(var(--color-bg-elevated-rgb) / 46%);
  color: var(--color-text-secondary);
  font-size: 0.78rem;
  font-weight: 700;
  padding: 0 0.72rem;
  text-decoration: none;
  transition:
    color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.usage-dashboard-toolbar__pricing-link:hover {
  border-color: rgb(var(--color-accent-primary-rgb) / 18%);
  color: var(--color-text-primary);
}

@media (width < 1180px) {
  .usage-dashboard-toolbar {
    align-items: flex-start;
    flex-direction: column;
  }

  .usage-dashboard-toolbar__actions {
    width: 100%;
    justify-content: flex-start;
  }
}

@media (width < 720px) {
  .usage-dashboard-toolbar__field,
  .usage-dashboard-toolbar__field--segmented,
  .usage-dashboard-toolbar__select,
  .usage-dashboard-toolbar__segments,
  .usage-dashboard-toolbar__pricing-link {
    width: 100%;
  }

  .usage-dashboard-toolbar__segments {
    justify-content: space-between;
  }

  .usage-dashboard-toolbar__segment {
    flex: 1;
  }
}
</style>
