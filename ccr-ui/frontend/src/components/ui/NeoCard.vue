<template>
  <div
    :class="cardClasses"
    @click="handleClick"
    @mouseenter="isHovered = true"
    @mouseleave="isHovered = false"
  >
    <!-- 渐变边框效果 -->
    <div
      v-if="gradientBorder"
      class="absolute inset-0 rounded-[inherit] pointer-events-none overflow-hidden"
    >
      <div
        class="absolute inset-0 rounded-[inherit] opacity-0 transition-opacity duration-300"
        :class="{ 'opacity-100': isHovered }"
        :style="gradientBorderStyle"
      />
    </div>

    <!-- 微光效果 (悬停时) -->
    <div
      v-if="glowEffect && isHovered"
      class="absolute inset-0 rounded-[inherit] pointer-events-none"
      :style="{ boxShadow: glowShadow }"
    />

    <!-- 卡片内容 -->
    <div class="relative z-10">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

interface Props {
  /** 是否可交互（悬停效果） */
  interactive?: boolean
  /** 是否启用渐变边框 */
  gradientBorder?: boolean
  /** 是否启用发光效果 */
  glowEffect?: boolean
  /** 发光颜色 */
  glowColor?: 'primary' | 'secondary' | 'success' | 'warning' | 'danger'
  /** 内边距大小 */
  padding?: 'none' | 'sm' | 'md' | 'lg'
  /** 是否禁用 */
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  interactive: true,
  gradientBorder: false,
  glowEffect: true,
  glowColor: 'primary',
  padding: 'md',
  disabled: false,
})

const emit = defineEmits<{
  click: [event: MouseEvent]
}>()

const isHovered = ref(false)

// 内边距映射
const paddingStyles = {
  none: '',
  sm: 'p-4',
  md: 'p-6',
  lg: 'p-8',
}

// 发光颜色映射
const glowColors = {
  primary: 'var(--glow-primary)',
  secondary: 'var(--glow-secondary)',
  success: 'var(--glow-success)',
  warning: 'var(--glow-warning)',
  danger: 'var(--glow-danger)',
}

const glowShadow = computed(() => glowColors[props.glowColor])

// 渐变边框样式
const gradientBorderStyle = computed(() => ({
  background: 'linear-gradient(135deg, var(--color-accent-primary), var(--color-accent-secondary))',
  padding: '1px',
  WebkitMask: 'linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0)',
  WebkitMaskComposite: 'xor',
  maskComposite: 'exclude',
}))

// 卡片类计算
const cardClasses = computed(() => [
  // 基础样式
  'relative rounded-xl bg-bg-elevated border border-border-subtle',
  'transition-all duration-200 ease-out',
  
  // 内边距
  paddingStyles[props.padding],
  
  // 交互样式
  props.interactive && !props.disabled
    ? 'cursor-pointer hover:border-border-default hover:-translate-y-0.5 hover:shadow-lg'
    : '',
  
  // 禁用样式
  props.disabled
    ? 'opacity-50 cursor-not-allowed'
    : '',
])

function handleClick(event: MouseEvent) {
  if (!props.disabled) {
    emit('click', event)
  }
}
</script>

<style scoped>
/* 确保卡片内容可以正确继承圆角 */
.rounded-\[inherit\] {
  border-radius: inherit;
}
</style>
