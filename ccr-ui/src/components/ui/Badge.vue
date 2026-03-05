<template>
  <span
    :class="badgeClasses"
    v-bind="$attrs"
  >
    <!-- Leading icon slot -->
    <span
      v-if="$slots.leading"
      :class="iconClasses"
    >
      <slot name="leading" />
    </span>

    <!-- Dot indicator -->
    <span
      v-if="dot"
      :class="dotClasses"
      aria-hidden="true"
    />

    <!-- Content -->
    <slot>{{ label }}</slot>

    <!-- Trailing icon slot -->
    <span
      v-if="$slots.trailing"
      :class="iconClasses"
    >
      <slot name="trailing" />
    </span>

    <!-- Removable button -->
    <button
      v-if="removable"
      type="button"
      :class="removeButtonClasses"
      aria-label="移除"
      @click.stop="$emit('remove')"
    >
      <svg
        class="w-3 h-3"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M6 18L18 6M6 6l12 12"
        />
      </svg>
    </button>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'

// Props interface
interface Props {
  /** 徽章文本 */
  label?: string
  /** 变体样式 */
  variant?: 'default' | 'primary' | 'secondary' | 'success' | 'warning' | 'danger' | 'info' | 'outline'
  /** 尺寸 */
  size?: 'xs' | 'sm' | 'md' | 'lg'
  /** 是否显示圆点指示器 */
  dot?: boolean
  /** 是否可移除 */
  removable?: boolean
  /** 是否为圆角药丸形状 */
  pill?: boolean
  /** 平台专属颜色 */
  platform?: 'claude' | 'codex' | 'gemini' | 'qwen' | 'iflow'
}

// Emits
defineEmits<{
  remove: []
}>()

// Props with defaults
const props = withDefaults(defineProps<Props>(), {
  label: undefined,
  variant: 'default',
  size: 'sm',
  dot: false,
  removable: false,
  pill: false,
  platform: undefined,
})

// Variant classes
const variantClasses = {
  default: 'bg-bg-tertiary text-text-secondary border-border-color',
  primary: 'bg-accent-primary/10 text-accent-primary border-accent-primary/20',
  secondary: 'bg-accent-secondary/10 text-accent-secondary border-accent-secondary/20',
  success: 'bg-accent-success/10 text-accent-success border-accent-success/20',
  warning: 'bg-accent-warning/10 text-accent-warning border-accent-warning/20',
  danger: 'bg-accent-danger/10 text-accent-danger border-accent-danger/20',
  info: 'bg-accent-info/10 text-accent-info border-accent-info/20',
  outline: 'bg-transparent text-text-secondary border-border-color',
}

// Platform-specific classes
const platformClasses = {
  claude: 'bg-platform-claude/10 text-platform-claude border-platform-claude/20',
  codex: 'bg-platform-codex/10 text-platform-codex border-platform-codex/20',
  gemini: 'bg-platform-gemini/10 text-platform-gemini border-platform-gemini/20',
  qwen: 'bg-platform-qwen/10 text-platform-qwen border-platform-qwen/20',
  iflow: 'bg-platform-iflow/10 text-platform-iflow border-platform-iflow/20',
}

// Size classes
const sizeClasses = {
  xs: 'text-[10px] px-1.5 py-0.5 gap-1',
  sm: 'text-xs px-2 py-0.5 gap-1',
  md: 'text-sm px-2.5 py-1 gap-1.5',
  lg: 'text-base px-3 py-1.5 gap-2',
}

// Badge classes
const badgeClasses = computed(() => [
  // Base styles
  'inline-flex items-center font-medium border transition-colors duration-150',

  // Size
  sizeClasses[props.size],

  // Shape
  props.pill ? 'rounded-full' : 'rounded-md',

  // Variant or Platform
  props.platform
    ? platformClasses[props.platform]
    : variantClasses[props.variant],
])

// Icon classes
const iconClasses = computed(() => {
  const iconSizes = {
    xs: 'w-2.5 h-2.5',
    sm: 'w-3 h-3',
    md: 'w-3.5 h-3.5',
    lg: 'w-4 h-4',
  }
  return ['flex-shrink-0', iconSizes[props.size]]
})

// Dot classes
const dotClasses = computed(() => {
  const dotSizes = {
    xs: 'w-1 h-1',
    sm: 'w-1.5 h-1.5',
    md: 'w-2 h-2',
    lg: 'w-2.5 h-2.5',
  }

  // Dot color based on variant
  const dotColors = {
    default: 'bg-text-muted',
    primary: 'bg-accent-primary',
    secondary: 'bg-accent-secondary',
    success: 'bg-accent-success',
    warning: 'bg-accent-warning',
    danger: 'bg-accent-danger',
    info: 'bg-accent-info',
    outline: 'bg-text-muted',
  }

  const platformDotColors = {
    claude: 'bg-platform-claude',
    codex: 'bg-platform-codex',
    gemini: 'bg-platform-gemini',
    qwen: 'bg-platform-qwen',
    iflow: 'bg-platform-iflow',
  }

  return [
    'rounded-full animate-pulse',
    dotSizes[props.size],
    props.platform
      ? platformDotColors[props.platform]
      : dotColors[props.variant],
  ]
})

// Remove button classes
const removeButtonClasses = computed(() => [
  'ml-0.5 -mr-1 p-0.5 rounded-full',
  'hover:bg-black/10 dark:hover:bg-white/10',
  'focus:outline-none focus:ring-2 focus:ring-current/30',
  'transition-colors duration-150',
])
</script>
