<template>
  <div
    :class="skeletonClasses"
    :style="skeletonStyle"
    role="status"
    aria-live="polite"
    aria-label="Loading content"
  >
    <!-- 多行骨架 (table variant) -->
    <template v-if="variant === 'table' && rows && rows > 1">
      <div
        v-for="i in rows"
        :key="i"
        :class="rowClasses"
        :style="rowStyle"
      />
    </template>

    <span class="sr-only">Loading...</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

// Props 定义
interface Props {
  /** 骨架屏变体类型 */
  variant?: 'line' | 'circle' | 'card' | 'table' | 'custom'
  /** 宽度 (CSS 值, 如 '100%', '200px', '50rem') */
  width?: string
  /** 高度 (CSS 值, 如 '16px', '100px', '10rem') */
  height?: string
  /** table 变体的行数 */
  rows?: number
  /** 是否启用动画 (默认 true) */
  animated?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'line',
  width: undefined,
  height: undefined,
  rows: 1,
  animated: true,
})

// 计算骨架屏基础类名
const skeletonClasses = computed(() => {
  const classes = ['skeleton']

  // 添加动画类 (支持 prefers-reduced-motion)
  if (props.animated) {
    classes.push('skeleton-animated')
  }

  // 根据 variant 添加基础形状类
  if (props.variant !== 'table') {
    classes.push(`skeleton-${props.variant}`)
  }

  return classes.join(' ')
})

// 计算骨架屏样式
const skeletonStyle = computed(() => {
  const styles: Record<string, string> = {}

  // 对于非 table variant,设置宽高
  if (props.variant !== 'table') {
    if (props.width) {
      styles.width = props.width
    } else {
      // 默认宽度
      styles.width = props.variant === 'circle' ? '48px' : '100%'
    }

    if (props.height) {
      styles.height = props.height
    } else {
      // 默认高度
      if (props.variant === 'circle') {
        styles.height = '48px'
      } else if (props.variant === 'card') {
        styles.height = '200px'
      } else {
        styles.height = '16px' // line 和 custom
      }
    }
  }

  return styles
})

// table variant 的行类名
const rowClasses = computed(() => {
  return 'skeleton skeleton-line skeleton-animated'
})

// table variant 的行样式
const rowStyle = computed(() => {
  const styles: Record<string, string> = {
    width: props.width || '100%',
    height: props.height || '16px',
    marginBottom: '8px',
  }
  return styles
})
</script>

<style scoped>
/* 基础骨架屏样式 */
.skeleton {
  background: var(--bg-tertiary);
  position: relative;
  overflow: hidden;
  border-radius: var(--radius-sm);
}

/* line 变体 */
.skeleton-line {
  border-radius: var(--radius-xs);
}

/* circle 变体 */
.skeleton-circle {
  border-radius: var(--radius-full);
}

/* card 变体 */
.skeleton-card {
  border-radius: var(--radius-md);
}

/* shimmer 动画效果 */
.skeleton-animated::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: linear-gradient(
    90deg,
    transparent 0%,
    rgba(var(--accent-primary-rgb), 0.08) 50%,
    transparent 100%
  );
  animation: shimmer 1.5s ease-in-out infinite;
  transform: translateX(-100%);
}

@keyframes shimmer {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(100%);
  }
}

/* 尊重用户的动画偏好 - 禁用或减缓动画 */
@media (prefers-reduced-motion: reduce) {
  .skeleton-animated::before {
    animation-duration: 3s; /* 减缓动画速度 */
    animation-iteration-count: 1; /* 只播放一次 */
  }
}

/* 屏幕阅读器专用文本 */
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border-width: 0;
}
</style>
