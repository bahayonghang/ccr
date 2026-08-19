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
      class="relative z-10 h-full"
      :class="[paddingClasses, props.bodyClass]"
    >
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

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
  /** @deprecated 发光装饰层已移除；保留 props 以免存量调用方断裂。 */
  glow?: boolean
  /** @deprecated glow 的旧别名。 */
  glowEffect?: boolean
  /** @deprecated 仅配合已废弃的 glow；无效果。 */
  glowColor?: GlowColor
  /** @deprecated 渐变描边已移除。 */
  gradientBorder?: boolean
  /** @deprecated 圆点纹理已移除。 */
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
  base: 0,
  elevated: 0,
  glass: 0,
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
  border-radius: var(--radius-2xl);
  transition-property: transform, box-shadow, border-color, background-color, opacity;
  transition-duration: var(--ui-card-duration, var(--motion-standard-duration));
  transition-timing-function: var(--ui-card-ease, var(--motion-standard-ease));
}

.ui-card--disabled {
  @apply cursor-not-allowed opacity-50;
}

.ui-card--base {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-subtle);
  box-shadow: none;
}

.ui-card--elevated {
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border-subtle);
  box-shadow: none;
}

.ui-card--glass {
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border-subtle);
  box-shadow: none;
}

.ui-card--outline {
  background: transparent;
  border: 1px solid var(--color-border-subtle);
  box-shadow: none;
}

.ui-card--surface-modal.ui-card--elevated,
.ui-card--surface-modal.ui-card--glass {
  background: var(--surface-modal-bg);
  border-color: var(--surface-modal-border);
  box-shadow: var(--surface-modal-shadow);
}

.ui-card--interactive {
  @apply cursor-pointer;
}

.ui-card--interactive:hover {
  transform: translateY(var(--ui-card-hover-translate, -1px));
  border-color: var(--color-border-strong);
  box-shadow: var(--ui-card-hover-shadow, var(--shadow-sm));
}

.ui-card--elevation-0 {
  --ui-card-shadow: none;
  --ui-card-hover-shadow: var(--shadow-sm);
}

.ui-card--elevation-1 {
  --ui-card-shadow: var(--elevation-1);
  --ui-card-hover-shadow: var(--shadow-sm);
}

.ui-card--elevation-2,
.ui-card--elevation-3,
.ui-card--elevation-4 {
  --ui-card-shadow: var(--shadow-sm);
  --ui-card-hover-shadow: var(--shadow-sm);
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
  --ui-card-hover-translate: -1px;
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
