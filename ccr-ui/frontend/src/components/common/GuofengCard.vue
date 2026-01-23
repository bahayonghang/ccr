<template>
  <div
    class="guofeng-card group relative overflow-hidden transition-all"
    :class="[
      interactive && !disabled ? 'hover:-translate-y-1 cursor-pointer animate-gpu-accelerated' : '',
      variant === 'glass' ? 'glass-effect' : 'bg-[var(--bg-secondary)]',
      disabled ? 'opacity-50 cursor-not-allowed' : '',
      paddingClasses,
      className
    ]"
    :style="{
      borderRadius: 'var(--radius-xl)',
      border: '1px solid var(--border-color)',
      transition: `all var(--duration-normal) var(--ease-out-cubic)`,
      ...style
    }"
    @click="handleClick"
    @mouseenter="onMouseEnter"
    @mouseleave="onMouseLeave"
  >
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

    <!-- Enhanced hover glow effect -->
    <div
      v-if="(interactive || glowEffect) && !disabled"
      class="absolute inset-0 opacity-0 pointer-events-none transition-opacity"
      :class="{ 'opacity-100': isHovered }"
      :style="{
        boxShadow: glowColors[glowColor],
        transition: `opacity var(--duration-normal) var(--ease-out-cubic)`
      }"
    />

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

// 发光颜色映射
const glowColors = {
  primary: 'var(--glow-primary)',
  secondary: 'var(--glow-secondary)',
  success: 'var(--glow-success)',
  warning: 'var(--glow-warning)',
  danger: 'var(--glow-danger)',
}

// 渐变边框样式
const gradientBorderStyle = computed(() => ({
  background: 'linear-gradient(135deg, var(--color-accent-primary), var(--color-accent-secondary))',
  padding: '1px',
  WebkitMask: 'linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0)',
  WebkitMaskComposite: 'xor',
  maskComposite: 'exclude',
}))

const BACKGROUND_PATTERN = `url("data:image/svg+xml,%3Csvg width='60' height='60' viewBox='0 0 60 60' xmlns='http://www.w3.org/2000/svg'%3E%3Cg fill='none' fill-rule='evenodd'%3E%3Cg fill='%23000000' fill-opacity='1'%3E%3Cpath d='M36 34v-4h-2v4h-4v2h4v4h2v-4h4v-2h-4zm0-30V0h-2v4h-4v2h4v4h2V6h4V4h-4zM6 34v-4H4v4H0v2h4v4h2v-4h4v-2H6zM6 4V0H4v4H0v2h4v4h2V6h4V4H6z'/%3E%3C/g%3E%3C/g%3E%3C/svg%3E")`

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
