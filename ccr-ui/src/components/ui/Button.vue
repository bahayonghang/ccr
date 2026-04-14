<template>
  <button
    :type="props.type"
    :class="classes"
    :data-variant="props.variant"
    :data-size="resolvedSize"
    :data-surface="resolvedSurface"
    :data-elevation="resolvedElevation"
    :data-motion="resolvedMotion"
    :data-density="resolvedDensity"
    :disabled="props.disabled || props.loading"
    :aria-busy="props.loading || undefined"
    @click="handleClick"
  >
    <!-- Loading Spinner -->
    <svg 
      v-if="props.loading"
      class="animate-spin -ml-1 mr-2 h-4 w-4" 
      xmlns="http://www.w3.org/2000/svg" 
      fill="none" 
      viewBox="0 0 24 24"
    >
      <circle
        class="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        stroke-width="4"
      />
      <path
        class="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
      />
    </svg>

    <!-- Leading Icon -->
    <span
      v-if="$slots.leading && !props.loading"
      class="mr-2 flex items-center"
    >
      <slot name="leading" />
    </span>

    <!-- Content -->
    <slot />

    <!-- Trailing Icon -->
    <span
      v-if="$slots.trailing"
      class="ml-2 flex items-center"
    >
      <slot name="trailing" />
    </span>
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'

type ButtonSurface = 'workspace' | 'card' | 'modal' | 'status'
type ButtonElevation = 0 | 1 | 2 | 3 | 4
type ButtonMotion = 'none' | 'subtle' | 'standard'
type ButtonDensity = 'compact' | 'default'

interface Props {
  variant?: 'primary' | 'secondary' | 'accent' | 'outline' | 'ghost' | 'glass' | 'danger' | 'success'
  size?: 'sm' | 'md' | 'lg' | 'icon'
  surface?: ButtonSurface
  elevation?: ButtonElevation
  motion?: ButtonMotion
  density?: ButtonDensity
  type?: 'button' | 'submit' | 'reset'
  disabled?: boolean
  loading?: boolean
  block?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  type: 'button',
  disabled: false,
  loading: false,
  block: false,
})

const emit = defineEmits(['click'])

const handleClick = (e: MouseEvent) => {
  if (!props.disabled && !props.loading) {
    emit('click', e)
  }
}

const resolvedDensity = computed<ButtonDensity>(() => props.density ?? 'default')
const resolvedSize = computed<NonNullable<Props['size']>>(() => props.size ?? (resolvedDensity.value === 'compact' ? 'sm' : 'md'))
const resolvedSurface = computed<ButtonSurface>(() => props.surface ?? 'status')
const resolvedMotion = computed<ButtonMotion>(() => props.motion ?? 'standard')
const resolvedElevation = computed<ButtonElevation>(() => {
  if (props.elevation !== undefined) return props.elevation
  if (props.variant === 'ghost' || props.variant === 'outline') return 0
  if (props.variant === 'primary' || props.variant === 'danger' || props.variant === 'success') return 2
  return 1
})

const classes = computed(() => {
  return [
    'ui-button',
    `ui-button--${props.variant}`,
    `ui-button--${resolvedSize.value}`,
    `ui-button--surface-${resolvedSurface.value}`,
    `ui-button--elevation-${resolvedElevation.value}`,
    `ui-button--motion-${resolvedMotion.value}`,
    `ui-button--density-${resolvedDensity.value}`,
    props.block ? 'ui-button--block' : '',
  ].join(' ')
})
</script>

<style scoped>
.ui-button {
  @apply inline-flex min-h-[44px] items-center justify-center rounded-full font-medium;
  @apply focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30 focus-visible:ring-offset-2 focus-visible:ring-offset-bg-base;

  transform: translateZ(0);
  letter-spacing: -0.01em;
  backdrop-filter: var(--surface-status-blur);
  transition-property: transform, box-shadow, background-color, border-color, color, opacity;
  transition-duration: var(--ui-button-duration, var(--motion-standard-duration));
  transition-timing-function: var(--ui-button-ease, var(--motion-standard-ease));
}

.ui-button:active {
  transform: scale(var(--ui-button-active-scale, 0.95));
}

.ui-button:disabled {
  @apply cursor-not-allowed opacity-50;

  box-shadow: none;
}

.ui-button--block {
  @apply w-full;
}

.ui-button--sm {
  @apply px-3 py-2 text-xs;
}

.ui-button--md {
  @apply px-4 py-2.5 text-sm;
}

.ui-button--lg {
  @apply px-6 py-3 text-base;
}

.ui-button--icon {
  @apply min-w-[44px] p-2.5;
}

.ui-button--primary {
  @apply border text-text-inverted;

  border-color: rgb(var(--color-accent-primary-rgb) / 14%);
  background: var(--color-accent-primary);
  box-shadow:
    var(--ui-button-shadow, 0 16px 36px rgb(var(--color-accent-primary-rgb) / 22%)),
    inset 0 1px 0 rgb(255 255 255 / 18%);
}

.ui-button--primary:hover:not(:disabled) {
  background: var(--color-accent-primary-hover);
  box-shadow:
    0 18px 40px rgb(var(--color-accent-primary-rgb) / 26%),
    inset 0 1px 0 rgb(255 255 255 / 22%);
  transform: translateY(-1px);
}

.ui-button--secondary {
  @apply border text-text-primary shadow-sm;

  border-color: var(--surface-status-border);
  background: linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 96%), rgb(var(--color-bg-surface-rgb) / 82%));
  box-shadow:
    var(--ui-button-shadow, var(--surface-status-shadow)),
    inset 0 1px 0 rgb(255 255 255 / 12%);
}

.ui-button--secondary:hover:not(:disabled) {
  border-color: rgb(var(--color-accent-primary-rgb) / 18%);
  background: linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 100%), rgb(var(--color-bg-surface-rgb) / 90%));
}

.ui-button--accent {
  @apply border text-text-inverted;

  border-color: rgb(var(--color-accent-secondary-rgb) / 14%);
  background: var(--color-accent-secondary);
  box-shadow: 0 16px 34px rgb(var(--color-accent-secondary-rgb) / 18%);
}

.ui-button--accent:hover:not(:disabled) {
  background: var(--color-accent-secondary-hover);
}

.ui-button--outline {
  @apply border bg-transparent text-text-secondary;

  border-color: rgb(var(--color-border-default-rgb) / 80%);
}

.ui-button--outline:hover:not(:disabled) {
  @apply text-text-primary;

  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
  background-color: rgb(var(--color-bg-surface-rgb) / 70%);
}

.ui-button--ghost {
  @apply text-text-secondary;
}

.ui-button--ghost:hover:not(:disabled) {
  @apply text-text-primary;

  background-color: rgb(var(--color-bg-surface-rgb) / 70%);
}

.ui-button--glass {
  @apply border text-text-primary shadow-sm;

  background: var(--surface-status-bg);
  border-color: var(--surface-status-border);
  backdrop-filter: var(--surface-status-blur);
  box-shadow:
    var(--ui-button-shadow, var(--surface-status-shadow)),
    inset 0 1px 0 rgb(255 255 255 / 14%);
}

.ui-button--glass:hover:not(:disabled) {
  border-color: rgb(var(--color-accent-primary-rgb) / 16%);
  background-color: rgb(var(--color-bg-elevated-rgb) / 84%);
}

.ui-button--danger {
  @apply border text-text-inverted;

  border-color: rgb(var(--color-danger-rgb) / 14%);
  background: var(--color-danger);
  box-shadow: var(--ui-button-shadow, 0 16px 34px rgb(var(--color-danger-rgb) / 18%));
}

.ui-button--danger:hover:not(:disabled) {
  background: var(--color-danger-hover);
}

.ui-button--success {
  @apply border text-text-inverted;

  border-color: rgb(var(--color-success-rgb) / 14%);
  background: var(--color-success);
  box-shadow: var(--ui-button-shadow, 0 16px 34px rgb(var(--color-success-rgb) / 18%));
}

.ui-button--success:hover:not(:disabled) {
  background: var(--color-success-hover);
  box-shadow: var(--ui-button-shadow, 0 16px 34px rgb(var(--color-success-rgb) / 24%));
  transform: translateY(-1px);
}

.ui-button--surface-workspace.ui-button--glass {
  background: var(--surface-workspace-bg);
  backdrop-filter: var(--surface-workspace-blur);
}

.ui-button--surface-card.ui-button--glass {
  background: var(--surface-card-bg);
  backdrop-filter: var(--surface-card-blur);
}

.ui-button--surface-modal.ui-button--glass {
  background: var(--surface-modal-bg);
  backdrop-filter: var(--surface-modal-blur);
}

.ui-button--elevation-0 {
  --ui-button-shadow: none;
}

.ui-button--elevation-1 {
  --ui-button-shadow: var(--elevation-1);
}

.ui-button--elevation-2 {
  --ui-button-shadow: var(--elevation-2);
}

.ui-button--elevation-3 {
  --ui-button-shadow: var(--elevation-3);
}

.ui-button--elevation-4 {
  --ui-button-shadow: var(--elevation-4);
}

.ui-button--motion-none {
  --ui-button-duration: var(--motion-none-duration);
  --ui-button-active-scale: 1;
}

.ui-button--motion-subtle {
  --ui-button-duration: var(--motion-subtle-duration);
  --ui-button-ease: var(--motion-subtle-ease);
  --ui-button-active-scale: 0.98;
}

.ui-button--motion-standard {
  --ui-button-duration: var(--motion-standard-duration);
  --ui-button-ease: var(--motion-standard-ease);
  --ui-button-active-scale: 0.95;
}
</style>
