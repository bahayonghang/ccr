<template>
  <div
    class="ui-card group relative overflow-hidden transition-interactive duration-300"
    :class="[
      `ui-card--${normalizedVariant}`,
      isInteractive ? 'ui-card--interactive' : '',
      disabled ? 'ui-card--disabled' : '',
      className,
    ]"
    :style="style"
    :aria-disabled="disabled || undefined"
    @click="handleClick"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <div
      v-if="pattern"
      class="absolute inset-0 ui-card-pattern pointer-events-none"
      :style="{ backgroundImage: backgroundPattern }"
    />

    <div
      v-if="showGlow"
      class="absolute inset-0 opacity-0 pointer-events-none transition-opacity duration-500 group-hover:opacity-100"
      :style="glowStyle"
    />

    <div
      v-if="gradientBorder"
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
      <div class="rounded-full bg-white/10 p-1.5 backdrop-blur-md text-text-primary">
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

    <template v-if="normalizedVariant === 'neko'">
      <div
        class="absolute -top-3 left-5 z-10 h-6 w-6 bg-accent-primary transition-transform duration-300 hover:scale-110"
        style="clip-path: polygon(50% 0%, 0% 100%, 100% 100%); transform: rotate(-15deg);"
      />
      <div
        class="absolute -top-3 right-5 z-10 h-6 w-6 bg-accent-primary transition-transform duration-300 hover:scale-110"
        style="clip-path: polygon(50% 0%, 0% 100%, 100% 100%); transform: rotate(15deg);"
      />
    </template>

    <div
      class="relative z-10 h-full"
      :class="[paddingClasses, bodyClass]"
    >
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { CSSProperties } from 'vue'

type CardVariant = 'default' | 'base' | 'elevated' | 'glass' | 'outline' | 'neko'
type PaddingSize = 'none' | 'sm' | 'md' | 'lg'
type GlowColor = 'primary' | 'secondary' | 'success' | 'warning' | 'danger'

interface Props {
  variant?: CardVariant
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

const props = withDefaults(defineProps<Props>(), {
  variant: 'elevated',
  hover: false,
  interactive: false,
  glow: false,
  glowEffect: false,
  glowColor: 'primary',
  gradientBorder: false,
  pattern: false,
  padding: 'md',
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
  props.variant === 'default' ? 'elevated' : props.variant
)

const isInteractive = computed(() => !props.disabled && (props.interactive || props.hover))
const showGlow = computed(() => props.glow || props.glowEffect || normalizedVariant.value === 'neko')

const paddingClasses = computed(() => {
  const map: Record<PaddingSize, string> = {
    none: '',
    sm: 'p-4',
    md: 'p-6',
    lg: 'p-8',
  }
  return map[props.padding]
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
.ui-card--disabled {
  @apply cursor-not-allowed opacity-50;
}

.ui-card--base {
  @apply rounded-xl border border-white/10 shadow-sm;

  background: var(--color-bg-elevated);
}

.ui-card--elevated {
  @apply rounded-2xl;

  background: var(--surface-card-bg);
  border: 1px solid var(--surface-card-border);
  backdrop-filter: var(--surface-card-blur);
  box-shadow: var(--surface-card-shadow), var(--glass-inner-glow);
}

.ui-card--glass {
  @apply rounded-2xl;

  background: var(--surface-workspace-bg);
  border: 1px solid var(--surface-workspace-border);
  backdrop-filter: var(--surface-workspace-blur);
  box-shadow: var(--surface-workspace-shadow);
}

.ui-card--outline {
  @apply rounded-xl border border-white/20 bg-transparent backdrop-blur-md;
}

.ui-card--neko {
  @apply mt-4 overflow-visible rounded-2xl border border-accent-primary/20;

  background: var(--surface-card-bg);
  border-color: rgb(var(--color-accent-primary-rgb) / 20%);
  backdrop-filter: var(--surface-card-blur);
  box-shadow:
    var(--surface-card-shadow),
    var(--glass-inner-glow),
    0 0 24px rgb(var(--color-accent-primary-rgb) / 18%);
}

.ui-card--interactive {
  @apply cursor-pointer;
}

.ui-card--interactive:hover {
  transform: translateY(-0.18rem);
  box-shadow:
    var(--surface-card-shadow),
    0 18px 40px rgb(var(--color-accent-primary-rgb) / 12%);
  border-color: rgb(var(--color-accent-primary-rgb) / 28%);
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
