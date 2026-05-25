<template>
  <div
    class="ui-card group relative overflow-hidden"
    :class="[
      `ui-card--${normalizedVariant}`,
      `ui-card--surface-${resolvedSurface}`,
      `ui-card--elevation-${resolvedElevation}`,
      `ui-card--motion-${resolvedMotion}`,
      `ui-card--density-${resolvedDensity}`,
      isInteractive ? 'ui-card--interactive' : '',
      props.disabled ? 'ui-card--disabled' : '',
      props.className,
    ]"
    :data-surface="resolvedSurface"
    :data-elevation="resolvedElevation"
    :data-motion="resolvedMotion"
    :data-density="resolvedDensity"
    :style="props.style"
    :aria-disabled="props.disabled || undefined"
    @click="handleClick"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <div
      v-if="props.pattern"
      class="absolute inset-0 ui-card-pattern pointer-events-none"
      :style="{ backgroundImage: backgroundPattern }"
    />

    <div
      v-if="showGlow"
      class="absolute inset-0 opacity-0 pointer-events-none transition-opacity duration-500 group-hover:opacity-100"
      :style="glowStyle"
    />

    <div
      v-if="props.gradientBorder"
      class="absolute inset-0 ui-card-inherit-radius pointer-events-none overflow-hidden"
    >
      <div
        class="absolute inset-0 ui-card-inherit-radius opacity-0 transition-opacity duration-300"
        :class="{ 'opacity-100': isInteractive }"
        :style="gradientBorderStyle"
      />
    </div>

    <div
      v-if="isInteractive"
      class="absolute top-3 right-3 opacity-0 group-hover:opacity-100 transition-interactive duration-300 translate-x-1 group-hover:translate-x-0 pointer-events-none"
    >
      <div class="rounded-full bg-bg-elevated/80 p-1.5 backdrop-blur-md text-text-primary">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M7 17L17 7M17 7H7M17 7V17" />
        </svg>
      </div>
    </div>

    <div
      class="relative z-10 h-full"
      :class="[paddingClasses, props.bodyClass]"
    >
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { CSSProperties } from 'vue'

type CardVariant = 'default' | 'base' | 'elevated' | 'glass' | 'outline'
type PaddingSize = 'none' | 'sm' | 'md' | 'lg'
type GlowColor = 'primary' | 'secondary' | 'success' | 'warning' | 'danger'
type SurfaceKind = 'workspace' | 'card' | 'modal' | 'status'
type ElevationLevel = 0 | 1 | 2 | 3 | 4
type MotionKind = 'none' | 'subtle' | 'standard'
type DensityKind = 'compact' | 'default'

interface Props {
  variant?: CardVariant
  surface?: SurfaceKind
  elevation?: ElevationLevel
  motion?: MotionKind
  density?: DensityKind
  hover?: boolean
  interactive?: boolean
  glow?: boolean
  glowEffect?: boolean
  glowColor?: GlowColor
  gradientBorder?: boolean
  pattern?: boolean
  padding?: PaddingSize
  disabled?: boolean
  className?: string
  bodyClass?: string
  style?: Record<string, string | number>
}

const surfaceByVariant: Record<Exclude<CardVariant, 'default'>, SurfaceKind> = {
  base: 'workspace',
  elevated: 'card',
  glass: 'workspace',
  outline: 'status',
}

const variantBySurface: Record<SurfaceKind, Exclude<CardVariant, 'default'>> = {
  workspace: 'glass',
  card: 'elevated',
  modal: 'elevated',
  status: 'outline',
}

const elevationByVariant: Record<Exclude<CardVariant, 'default'>, ElevationLevel> = {
  base: 1,
  elevated: 3,
  glass: 2,
  outline: 0,
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'elevated',
  hover: false,
  interactive: false,
  glow: false,
  glowEffect: false,
  glowColor: 'primary',
  gradientBorder: false,
  pattern: false,
  disabled: false,
  className: '',
  bodyClass: '',
  style: () => ({}),
})

const emit = defineEmits<{
  click: [event: MouseEvent]
  mouseenter: [event: MouseEvent]
  mouseleave: [event: MouseEvent]
}>()

const normalizedVariant = computed<Exclude<CardVariant, 'default'>>(() =>
  props.variant === 'default'
    ? (props.surface ? variantBySurface[props.surface] : 'elevated')
    : props.variant
)

const isInteractive = computed(() => !props.disabled && (props.interactive || props.hover))
const resolvedSurface = computed<SurfaceKind>(() => props.surface ?? surfaceByVariant[normalizedVariant.value])
const resolvedElevation = computed<ElevationLevel>(() => props.elevation ?? elevationByVariant[normalizedVariant.value])
const resolvedMotion = computed<MotionKind>(() => props.motion ?? (isInteractive.value ? 'standard' : 'subtle'))
const resolvedDensity = computed<DensityKind>(() => props.density ?? 'default')
const showGlow = computed(() => props.glow || props.glowEffect)

const paddingClasses = computed(() => {
  const map: Record<PaddingSize, string> = {
    none: '',
    sm: 'p-4',
    md: 'p-6',
    lg: 'p-8',
  }
  const fallbackPadding: PaddingSize = resolvedDensity.value === 'compact' ? 'sm' : 'md'
  return map[props.padding ?? fallbackPadding]
})

const glowColors: Record<GlowColor, string> = {
  primary: 'rgb(var(--color-accent-primary-rgb) / 0.18)',
  secondary: 'rgb(var(--color-accent-secondary-rgb) / 0.18)',
  success: 'rgb(var(--color-success-rgb) / 0.18)',
  warning: 'rgb(var(--color-warning-rgb) / 0.18)',
  danger: 'rgb(var(--color-danger-rgb) / 0.18)',
}

const glowStyle = computed<CSSProperties>(() => ({
  background: `radial-gradient(circle at center, ${glowColors[props.glowColor]} 0%, transparent 70%)`,
  mixBlendMode: 'overlay' as const,
  filter: 'blur(20px)',
}))

const gradientBorderStyle = computed(() => ({
  background: 'linear-gradient(135deg, var(--color-accent-primary), var(--color-accent-secondary))',
  padding: '1px',
  WebkitMask: 'linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0)',
  WebkitMaskComposite: 'xor',
  maskComposite: 'exclude',
}))

const backgroundPattern = `url("data:image/svg+xml,%3Csvg width='20' height='20' viewBox='0 0 20 20' xmlns='http://www.w3.org/2000/svg'%3E%3Cg fill='%239C92AC' fill-opacity='0.2' fill-rule='evenodd'%3E%3Ccircle cx='3' cy='3' r='1'/%3E%3C/g%3E%3C/svg%3E")`

const handleClick = (event: MouseEvent) => {
  if (props.disabled) return
  emit('click', event)
}

const handleMouseEnter = (event: MouseEvent) => {
  if (!isInteractive.value) return
  emit('mouseenter', event)
}

const handleMouseLeave = (event: MouseEvent) => {
  if (!isInteractive.value) return
  emit('mouseleave', event)
}
</script>

<style scoped>
.ui-card {
  transition-property: transform, box-shadow, border-color, background-color, opacity;
  transition-duration: var(--ui-card-duration, var(--motion-standard-duration));
  transition-timing-function: var(--ui-card-ease, var(--motion-standard-ease));
}

.ui-card--disabled {
  @apply cursor-not-allowed opacity-50;
}

.ui-card--base {
  @apply rounded-xl border shadow-sm;

  background: var(--color-bg-elevated);
  box-shadow:
    var(--ui-card-shadow, var(--shadow-sm)),
    var(--inner-glow);
}

.ui-card--elevated {
  @apply rounded-2xl;

  background: var(--surface-card-bg);
  border: 1px solid var(--surface-card-border);
  backdrop-filter: var(--surface-card-blur);
  box-shadow:
    var(--ui-card-shadow, var(--surface-card-shadow)),
    var(--glass-inner-glow);
}

.ui-card--glass {
  @apply rounded-2xl;

  background: var(--surface-workspace-bg);
  border: 1px solid var(--surface-workspace-border);
  backdrop-filter: var(--surface-workspace-blur);
  box-shadow:
    var(--ui-card-shadow, var(--surface-workspace-shadow)),
    var(--glass-inner-glow);
}

.ui-card--outline {
  @apply rounded-xl bg-transparent;

  border: 1px solid var(--surface-status-border);
  backdrop-filter: var(--surface-status-blur);
  box-shadow: var(--ui-card-shadow, none);
}

.ui-card--surface-modal.ui-card--elevated,
.ui-card--surface-modal.ui-card--glass {
  background: var(--surface-modal-bg);
  border-color: var(--surface-modal-border);
  backdrop-filter: var(--surface-modal-blur);
}

.ui-card--interactive {
  @apply cursor-pointer;
}

.ui-card--interactive:hover {
  transform: translateY(var(--ui-card-hover-translate, -0.12rem));
  box-shadow:
    var(--ui-card-hover-shadow, var(--ui-card-shadow, var(--surface-card-shadow))),
    0 14px 28px rgb(var(--color-accent-primary-rgb) / 8%);
  border-color: rgb(var(--color-accent-primary-rgb) / 14%);
}

.ui-card--elevation-0 {
  --ui-card-shadow: var(--elevation-0);
  --ui-card-hover-shadow: var(--elevation-0);
}

.ui-card--elevation-1 {
  --ui-card-shadow: var(--elevation-1);
  --ui-card-hover-shadow: var(--elevation-2);
}

.ui-card--elevation-2 {
  --ui-card-shadow: var(--elevation-2);
  --ui-card-hover-shadow: var(--elevation-3);
}

.ui-card--elevation-3 {
  --ui-card-shadow: var(--elevation-3);
  --ui-card-hover-shadow: var(--elevation-4);
}

.ui-card--elevation-4 {
  --ui-card-shadow: var(--elevation-4);
  --ui-card-hover-shadow: var(--shadow-2xl);
}

.ui-card--motion-none {
  --ui-card-duration: var(--motion-none-duration);
  --ui-card-hover-translate: 0px;
}

.ui-card--motion-subtle {
  --ui-card-duration: var(--motion-subtle-duration);
  --ui-card-ease: var(--motion-subtle-ease);
  --ui-card-hover-translate: -1px;
}

.ui-card--motion-standard {
  --ui-card-duration: var(--motion-standard-duration);
  --ui-card-ease: var(--motion-standard-ease);
  --ui-card-hover-translate: -0.12rem;
}

.ui-card-pattern {
  opacity: 0.08;
}

.ui-card-inherit-radius {
  border-radius: inherit;
}

.ui-card:focus-visible {
  outline: 2px solid var(--color-accent-primary);
  outline-offset: 2px;
  border-radius: inherit;
}

@media (prefers-reduced-motion: reduce) {
  .ui-card {
    transition: none;
  }

  .ui-card:hover {
    transform: none;
  }
}
</style>

