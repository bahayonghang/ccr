<template>
  <div
    v-if="presentation.state !== 'ready'"
    class="usage-stale-banner"
    :class="`usage-stale-banner--${presentation.tone}`"
    role="status"
    data-usage-stale-banner
  >
    <SIcon
      :name="toneIcon"
      size="w-4 h-4"
      class="usage-stale-banner__icon"
    />

    <div class="usage-stale-banner__copy">
      <p class="usage-stale-banner__title">
        <span>{{ presentation.title }}</span>
        <span
          v-if="presentation.freshnessAgeLabel"
          class="usage-stale-banner__age"
        >{{ presentation.freshnessAgeLabel }}</span>
      </p>
      <p class="usage-stale-banner__detail">
        {{ presentation.detail }}
      </p>
    </div>

    <div class="usage-stale-banner__actions">
      <button
        v-if="presentation.primaryActionLabel"
        type="button"
        class="usage-stale-banner__action usage-stale-banner__action--primary"
        @click="emit('primary-action', presentation.primaryAction)"
      >
        {{ presentation.primaryActionLabel }}
      </button>
      <button
        v-if="presentation.secondaryActionLabel"
        type="button"
        class="usage-stale-banner__action"
        @click="emit('secondary-action')"
      >
        {{ presentation.secondaryActionLabel }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { UsageOpsAction, UsageOpsCockpitPresentation } from '@/views/usage/usageOpsCockpit'

const props = defineProps<{
  presentation: UsageOpsCockpitPresentation
}>()

const emit = defineEmits<{
  'primary-action': [action: UsageOpsAction]
  'secondary-action': []
}>()

// syncing/empty 语气偏中性用 Refresh/Circle，stale/degraded/needs_import 偏警示用 Triangle。
const toneIcon = computed(() => {
  switch (props.presentation.tone) {
    case 'info':
      return 'RefreshCw'
    case 'muted':
      return 'AlertCircle'
    default:
      return 'AlertTriangle'
  }
})
</script>

<style scoped>
.usage-stale-banner {
  --usage-stale-rgb: var(--color-warning-rgb);

  position: relative;
  isolation: isolate;
  overflow: hidden;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  border-radius: 1.1rem;
  border: 1px solid rgb(var(--usage-stale-rgb) / 20%);
  background: rgb(var(--usage-stale-rgb) / 8%);
  padding: 0.62rem 0.85rem;
  box-shadow: var(--elevation-1);
}

.usage-stale-banner--danger {
  --usage-stale-rgb: var(--color-danger-rgb);
}

.usage-stale-banner--info {
  --usage-stale-rgb: var(--color-info-rgb);
}

.usage-stale-banner--muted {
  --usage-stale-rgb: var(--color-text-muted-rgb);
}

.usage-stale-banner::before {
  content: '';
  position: absolute;
  inset: 0 auto 0 0;
  z-index: var(--z-behind);
  width: 3px;
  background: linear-gradient(180deg, rgb(var(--usage-stale-rgb) / 82%), transparent);
}

.usage-stale-banner__icon {
  flex: none;
  color: rgb(var(--usage-stale-rgb));
}

.usage-stale-banner__copy {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: 0.1rem;
}

.usage-stale-banner__title {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 0.4rem;
  color: var(--color-text-primary);
  font-size: 0.86rem;
  font-weight: 730;
}

.usage-stale-banner__age {
  border-radius: 999px;
  background: rgb(var(--usage-stale-rgb) / 12%);
  padding: 0.1rem 0.48rem;
  color: rgb(var(--usage-stale-rgb));
  font-size: 0.68rem;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.usage-stale-banner__detail {
  overflow: hidden;
  color: var(--color-text-secondary);
  font-size: 0.76rem;
  line-height: 1.4;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.usage-stale-banner__actions {
  display: flex;
  flex: none;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 0.4rem;
}

.usage-stale-banner__action {
  border-radius: 999px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 16%);
  background: rgb(var(--color-bg-elevated-rgb) / 72%);
  padding: 0.4rem 0.75rem;
  color: var(--color-text-primary);
  font-size: 0.76rem;
  font-weight: 720;
  white-space: nowrap;
  transition:
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    transform var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.usage-stale-banner__action:hover {
  transform: translateY(-1px);
  border-color: rgb(var(--usage-stale-rgb) / 26%);
  background: rgb(var(--usage-stale-rgb) / 10%);
}

.usage-stale-banner__action--primary {
  border-color: rgb(var(--usage-stale-rgb) / 26%);
  background: rgb(var(--usage-stale-rgb) / 15%);
}

@media (width < 760px) {
  .usage-stale-banner {
    flex-wrap: wrap;
  }

  .usage-stale-banner__detail {
    white-space: normal;
  }

  .usage-stale-banner__actions {
    justify-content: flex-start;
    width: 100%;
    padding-left: 1.65rem;
  }
}
</style>
