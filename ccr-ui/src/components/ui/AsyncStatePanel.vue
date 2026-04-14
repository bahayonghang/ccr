<template>
  <div
    class="async-state-panel"
    :class="panelClass"
    role="status"
    aria-live="polite"
  >
    <template v-if="state === 'loading'">
      <Spinner
        size="xl"
        class="text-accent-primary"
      />
      <p class="mt-3 text-sm text-text-secondary">
        {{ title }}
      </p>
      <p
        v-if="description"
        class="mt-1 text-center text-xs text-text-muted"
      >
        {{ description }}
      </p>
    </template>

    <template v-else>
      <div
        class="async-state-panel__icon"
        :class="iconContainerClass"
      >
        <SIcon
          :name="iconName"
          size="w-8 h-8"
          :class="iconClass"
        />
      </div>
      <h3 class="mt-4 text-lg font-semibold text-text-primary">
        {{ title }}
      </h3>
      <p
        v-if="description"
        class="mt-2 max-w-[520px] text-center text-sm text-text-secondary"
      >
        {{ description }}
      </p>
      <Button
        v-if="actionLabel"
        variant="primary"
        size="md"
        class="mt-5"
        @click="$emit('action')"
      >
        <template
          v-if="actionIcon"
          #leading
        >
          <SIcon
            :name="actionIcon"
            size="w-4 h-4"
          />
        </template>
        {{ actionLabel }}
      </Button>
    </template>
  </div>
</template>

<script setup lang="ts">
import Button from '@/components/ui/Button.vue'
import SIcon from '@/components/ui/SIcon.vue'
import Spinner from '@/components/ui/Spinner.vue'
import { computed } from 'vue'

type AsyncState = 'loading' | 'error' | 'empty' | 'runtime-unavailable'

const props = withDefaults(defineProps<{
  state: AsyncState
  title: string
  description?: string
  icon?: string
  actionLabel?: string
  actionIcon?: string
  compact?: boolean
}>(), {
  compact: false,
})

defineEmits<{
  (e: 'action'): void
}>()

const iconName = computed(() => {
  if (props.icon) return props.icon
  if (props.state === 'error') return 'AlertCircle'
  if (props.state === 'runtime-unavailable') return 'MonitorOff'
  if (props.state === 'empty') return 'FileX'
  return 'Loader2'
})

const iconClass = computed(() => {
  if (props.state === 'error') return 'text-accent-danger'
  if (props.state === 'runtime-unavailable') return 'text-accent-secondary'
  return 'text-text-muted'
})

const iconContainerClass = computed(() => {
  if (props.state === 'error') return 'bg-danger/10 border-danger/18'
  if (props.state === 'runtime-unavailable') return 'bg-accent-primary/10 border-accent-primary/18'
  return 'bg-bg-surface/90 border-border-default/55'
})

const panelClass = computed(() => (props.compact ? 'async-state-panel--compact' : ''))
</script>

<style scoped>
.async-state-panel {
  @apply flex flex-col items-center justify-center rounded-2xl px-6 py-16 text-center;

  background: var(--surface-card-bg);
  border: 1px solid var(--surface-card-border);
  box-shadow: var(--surface-card-shadow), var(--glass-inner-glow);
  backdrop-filter: var(--surface-card-blur);
}

.async-state-panel--compact {
  @apply py-12;
}

.async-state-panel__icon {
  @apply flex h-16 w-16 items-center justify-center rounded-full border;
}
</style>
