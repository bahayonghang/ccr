<template>
  <component
    :is="to ? 'router-link' : 'button'"
    :to="to"
    :class="itemClasses"
    :aria-current="isActive ? 'page' : undefined"
    @click="handleClick"
  >
    <!-- 活跃状态指示条 -->
    <div
      v-if="isActive"
      class="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-6 rounded-r-full bg-accent-primary"
      :class="{ 'animate-pulse-subtle': showActiveIndicator }"
    />

    <!-- 图标区域 -->
    <div
      class="flex-shrink-0 w-5 h-5 flex items-center justify-center transition-colors duration-200"
      :class="iconClasses"
    >
      <slot name="icon">
        <component
          :is="icon"
          v-if="icon"
          class="w-5 h-5"
        />
      </slot>
    </div>

    <!-- 标签文字 -->
    <span
      class="flex-1 truncate transition-colors duration-200"
      :class="labelClasses"
    >
      <slot>{{ label }}</slot>
    </span>

    <!-- 右侧徽章/附加内容 -->
    <div
      v-if="$slots.trailing || badge"
      class="flex-shrink-0"
    >
      <slot name="trailing">
        <span
          v-if="badge"
          class="px-2 py-0.5 text-xs font-medium rounded-full"
          :class="badgeClasses"
        >
          {{ badge }}
        </span>
      </slot>
    </div>

    <!-- 悬停背景效果 -->
    <div
      class="absolute inset-0 rounded-lg bg-bg-overlay opacity-0 transition-opacity duration-200 -z-10"
      :class="{ 'group-hover:opacity-100': !isActive }"
    />
  </component>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Component } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

interface Props {
  /** 路由链接地址 */
  to?: RouteLocationRaw
  /** 图标组件 */
  icon?: Component
  /** 标签文字 */
  label?: string
  /** 是否为活跃状态 */
  isActive?: boolean
  /** 是否显示活跃指示器动画 */
  showActiveIndicator?: boolean
  /** 右侧徽章内容 */
  badge?: string | number
  /** 徽章变体 */
  badgeVariant?: 'primary' | 'secondary' | 'success' | 'warning' | 'danger'
  /** 是否禁用 */
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  to: undefined,
  icon: undefined,
  label: undefined,
  isActive: false,
  showActiveIndicator: true,
  badge: undefined,
  badgeVariant: 'primary',
  disabled: false,
})

const emit = defineEmits<{
  click: [event: MouseEvent]
}>()

// 徽章颜色映射
const badgeVariants = {
  primary: 'bg-accent-primary/15 text-accent-primary',
  secondary: 'bg-accent-secondary/15 text-accent-secondary',
  success: 'bg-accent-success/15 text-accent-success',
  warning: 'bg-accent-warning/15 text-accent-warning',
  danger: 'bg-accent-danger/15 text-accent-danger',
}

// 项目整体类
const itemClasses = computed(() => [
  // 基础样式
  'group relative flex items-center gap-3 px-4 py-3 rounded-lg',
  'transition-all duration-200 ease-out',
  'focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30',
  
  // 活跃状态
  props.isActive
    ? 'bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary font-medium pl-5'
    : 'text-text-secondary hover:text-text-primary',
  
  // 禁用状态
  props.disabled
    ? 'opacity-50 cursor-not-allowed pointer-events-none'
    : 'cursor-pointer',
])

// 图标类
const iconClasses = computed(() => [
  props.isActive
    ? 'text-accent-primary'
    : 'text-text-muted group-hover:text-text-secondary',
])

// 标签类
const labelClasses = computed(() => [
  props.isActive ? 'text-accent-primary' : '',
])

// 徽章类
const badgeClasses = computed(() => badgeVariants[props.badgeVariant])

function handleClick(event: MouseEvent) {
  if (!props.disabled && !props.to) {
    emit('click', event)
  }
}
</script>
