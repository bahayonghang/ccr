<template>
  <div
    class="guofeng-card group relative overflow-hidden transition-all duration-300"
    :class="[
      interactive && !disabled ? 'hover:-translate-y-1 cursor-pointer' : '',
      variant === 'glass' ? 'glass-effect' : 'bg-[var(--bg-secondary)]',
      disabled ? 'opacity-50 cursor-not-allowed' : '',
      paddingClasses,
      className
    ]"
    :role="role"
    :tabindex="tabindex"
    :style="{
      borderRadius: 'var(--radius-xl)',
      border: '1px solid var(--border-color)',
      background: interactive && !disabled && isHovered 
        ? `linear-gradient(145deg, rgba(255,255,255,0.05) 0%, ${glowColors[glowColor].replace(')', ', 0.05)')} 100%)`
        : 'rgba(255, 255, 255, 0.03)',
      ...style
    }"
    @click="handleClick"
    @keydown="onKeydown"
    @mouseenter="onMouseEnter"
    @mouseleave="onMouseLeave"
  >
    <!-- Noise Texture for "Premium" feel -->
    <div
      class="absolute inset-0 opacity-[0.03] pointer-events-none mix-blend-overlay"
      :style="{ backgroundImage: 'url(https://grainy-gradients.vercel.app/noise.svg)' }"
    />

    <!-- Modern Gradient Mask (Breathing Light Effect) -->
    <div
      v-if="interactive && !disabled"
      class="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none"
      :style="{
        background: `radial-gradient(circle at center, ${glowColors[glowColor]} 0%, transparent 70%)`,
        mixBlendMode: 'overlay',
        filter: 'blur(20px)',
      }"
    />

    <!-- Iconified Entry Indicator (Top Right) -->
    <div
      v-if="interactive && !disabled"
      class="absolute top-3 right-3 opacity-0 group-hover:opacity-100 transition-all duration-300 transform translate-x-2 group-hover:translate-x-0"
    >
      <div class="p-1.5 rounded-full bg-white/10 backdrop-blur-sm text-[var(--color-text-primary)]">
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

    <!-- Gradient border effect (optional) -->
    <div
      v-if="gradientBorder"
      class="absolute inset-0 rounded-[inherit] pointer-events-none overflow-hidden"
    >
      <div
        class="absolute inset-0 rounded-[inherit] opacity-0 transition-opacity duration-300"
        :class="{ 'opacity-100': isHovered && !disabled }"
        :style="gradientBorderStyle"
      />
    </div>

    <!-- Decorative background pattern (optional) -->
    <div
      v-if="pattern"
      class="absolute inset-0 opacity-5 pointer-events-none"
      :style="{
        backgroundImage: BACKGROUND_PATTERN
      }"
    />

    <!-- Content -->
    <div
      class="relative z-10"
      :class="bodyClass"
    >
      <slot />
    </div>

    <!-- Interactive ripple effect -->
    <div
      v-if="interactive && !disabled"
      class="absolute inset-0 ripple-effect"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

interface Props {
  /** 卡片变体：default 或 glass */
  variant?: 'default' | 'glass'
  /** 是否可交互（悬停效果） */
  interactive?: boolean
  /** 是否显示装饰图案 */
  pattern?: boolean
  /** 额外的 CSS 类名 */
  className?: string
  /** 额外的内联样式 */
  style?: Record<string, any>
  /** 内容区域的额外类名 */
  bodyClass?: string
  /** 内边距大小 */
  padding?: 'none' | 'sm' | 'md' | 'lg'
  role?: string
  tabindex?: number | string
  onKeydown?: (event: KeyboardEvent) => void
  /** 是否启用渐变边框 */
  gradientBorder?: boolean
  /** 是否启用发光效果 */
  glowEffect?: boolean
  /** 发光颜色 */
  glowColor?: 'primary' | 'secondary' | 'success' | 'warning' | 'danger'
  /** 是否禁用 */
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  interactive: false,
  pattern: false,
  className: '',
  style: () => ({}),
  bodyClass: '',
  padding: 'md',
  role: undefined,
  tabindex: undefined,
  onKeydown: undefined,
  gradientBorder: false,
  glowEffect: false,
  glowColor: 'primary',
  disabled: false
})

const emit = defineEmits<{
  click: [event: MouseEvent]
  mouseenter: [event: MouseEvent]
  mouseleave: [event: MouseEvent]
}>()

const isHovered = ref(false)

// 内边距映射
const paddingMap = {
  none: '',
  sm: 'p-4',
  md: 'p-[var(--space-lg)]',
  lg: 'p-8',
}

const paddingClasses = computed(() => paddingMap[props.padding])

// 发光颜色映射 - 猫娘粉紫色系
const glowColors = {
  primary: 'rgba(244, 114, 182, 0.15)',
  secondary: 'rgba(167, 139, 250, 0.15)',
  success: 'rgba(52, 211, 153, 0.15)',
  warning: 'rgba(251, 191, 36, 0.15)',
  danger: 'rgba(251, 113, 133, 0.15)',
}

// 渐变边框样式
const gradientBorderStyle = computed(() => ({
  background: 'linear-gradient(135deg, var(--color-accent-primary), var(--color-accent-secondary))',
  padding: '1px',
  WebkitMask: 'linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0)',
  WebkitMaskComposite: 'xor',
  maskComposite: 'exclude',
}))

// Dot pattern for modern look
const BACKGROUND_PATTERN = `url("data:image/svg+xml,%3Csvg width='20' height='20' viewBox='0 0 20 20' xmlns='http://www.w3.org/2000/svg'%3E%3Cg fill='%239C92AC' fill-opacity='0.15' fill-rule='evenodd'%3E%3Ccircle cx='3' cy='3' r='1'/%3E%3C/g%3E%3C/svg%3E")`

const handleClick = (e: MouseEvent) => {
  if (!props.disabled) {
    emit('click', e)
  }
}

const onMouseEnter = (e: MouseEvent) => {
  if (!props.disabled) {
    isHovered.value = true
    if (props.interactive) {
      emit('mouseenter', e)
    }
  }
}

const onMouseLeave = (e: MouseEvent) => {
  isHovered.value = false
  if (props.interactive && !props.disabled) {
    emit('mouseleave', e)
  }
}
</script>

<style scoped>
.guofeng-card {
  position: relative;

  /* 性能优化 */
  transform: var(--hardware-acceleration);
  will-change: var(--will-change-auto);
}

.guofeng-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg, transparent, var(--color-border-default), transparent);
  opacity: 0.5;
}

/* 增强的悬停效果 */
.guofeng-card:hover {
  box-shadow: var(--shadow-xl);
}

/* 专注状态增强 */
.guofeng-card:focus-visible {
  outline: 2px solid var(--color-accent-primary);
  outline-offset: 2px;
  border-radius: var(--radius-xl);
}

/* 暗色主题适配 */
[data-theme="dark"] .guofeng-card {
  border-color: var(--color-border-default);
}

/* 减少动画效果 */
@media (prefers-reduced-motion: reduce) {
  .guofeng-card {
    transition: none;
  }

  .guofeng-card:hover {
    transform: none;
  }
}
</style>
