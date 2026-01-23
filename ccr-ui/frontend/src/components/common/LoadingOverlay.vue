<template>
  <Transition name="overlay-fade">
    <div
      v-if="visible"
      class="loading-overlay"
      :class="overlayClasses"
      :style="overlayStyle"
      role="progressbar"
      aria-live="polite"
      aria-busy="true"
      aria-label="Loading"
    >
      <!-- Spinner 模式 -->
      <div
        v-if="mode === 'spinner'"
        class="loading-spinner"
      >
        <Loader2
          :size="48"
          :stroke-width="2"
          class="spinner-icon"
        />
        <span
          v-if="$slots.default"
          class="loading-text"
        >
          <slot />
        </span>
      </div>

      <!-- Skeleton 模式 -->
      <div
        v-else-if="mode === 'skeleton'"
        class="loading-skeleton-container"
      >
        <slot name="skeleton">
          <!-- 默认骨架屏 -->
          <Skeleton
            variant="card"
            height="200px"
          />
        </slot>
      </div>

      <!-- 自定义模式 -->
      <div
        v-else
        class="loading-custom"
      >
        <slot />
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Loader2 } from 'lucide-vue-next'
import Skeleton from './Skeleton.vue'

// Props 定义
interface Props {
  /** 是否显示遮罩层 */
  visible: boolean
  /** 加载模式 */
  mode?: 'spinner' | 'skeleton' | 'custom'
  /** 遮罩层不透明度 (0-1) */
  opacity?: number
  /** 是否启用背景模糊效果 */
  blur?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  mode: 'spinner',
  opacity: 0.7,
  blur: false,
})

// 计算遮罩层类名
const overlayClasses = computed(() => {
  const classes = []
  
  if (props.blur) {
    classes.push('overlay-blur')
  }
  
  return classes.join(' ')
})

// 计算遮罩层样式
const overlayStyle = computed(() => {
  return {
    '--overlay-opacity': props.opacity.toString(),
  }
})
</script>

<style scoped>
.loading-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgb(var(--bg-primary-rgb, 255, 255, 255), var(--overlay-opacity, 0.7));
  z-index: 1000;
  transition: opacity 0.22s ease, backdrop-filter 0.22s ease;
}

.overlay-blur {
  backdrop-filter: blur(8px);
}

/* Spinner 容器 */
.loading-spinner {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-md);
}

/* Spinner 图标旋转动画 */
.spinner-icon {
  color: var(--state-loading);
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

/* 加载文本 */
.loading-text {
  font-size: var(--font-size-base);
  font-weight: var(--font-weight-medium);
  color: var(--text-secondary);
}

/* Skeleton 容器 */
.loading-skeleton-container {
  width: 100%;
  height: 100%;
  padding: var(--space-md);
}

/* 自定义加载内容容器 */
.loading-custom {
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 淡入淡出过渡动画 */
.overlay-fade-enter-active,
.overlay-fade-leave-active {
  transition: opacity 0.22s ease;
}

.overlay-fade-enter-from,
.overlay-fade-leave-to {
  opacity: 0;
}

/* 尊重用户的动画偏好 */
@media (prefers-reduced-motion: reduce) {
  .spinner-icon {
    animation-duration: 2s; /* 减缓旋转速度 */
  }

  .overlay-fade-enter-active,
  .overlay-fade-leave-active {
    transition-duration: 0.01ms;
  }

  .loading-overlay {
    transition: none;
  }
}
</style>
