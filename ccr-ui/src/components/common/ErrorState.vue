<template>
  <div
    class="error-state"
    role="alert"
    aria-live="assertive"
  >
    <!-- 错误图标 -->
    <div class="error-state-icon">
      <component
        :is="iconComponent"
        :size="iconSize"
        :stroke-width="1.5"
      />
    </div>

    <!-- 标题 -->
    <h3 class="error-state-title">
      {{ titleText }}
    </h3>

    <!-- 错误消息 -->
    <div class="error-state-message">
      {{ message }}
    </div>

    <!-- 重试按钮 (可选) -->
    <div
      v-if="retryLabel && retryHandler !== undefined"
      class="error-state-action"
    >
      <button
        class="error-state-button"
        @click="handleRetry"
      >
        <RotateCw
          :size="16"
          :stroke-width="2"
        />
        {{ retryLabel }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, type Component } from 'vue'
import { AlertCircle, RotateCw } from 'lucide-vue-next'

// Props 定义
interface Props {
  /** 图标组件 (Lucide 图标或自定义组件) */
  icon?: Component
  /** 标题文本 (可选,默认 "出错了") */
  title?: string
  /** 错误消息 */
  message: string
  /** 重试按钮文本 (可选,默认 "重试") */
  retryLabel?: string
  /** 重试按钮回调 (可选) */
  retryHandler?: () => void
  /** 图标大小 */
  iconSize?: number
}

const props = withDefaults(defineProps<Props>(), {
  icon: undefined,
  title: undefined,
  retryLabel: '重试',
  retryHandler: undefined,
  iconSize: 48,
})

// 计算使用的图标组件 (默认为 AlertCircle)
const iconComponent = computed(() => {
  return props.icon || AlertCircle
})

// 计算标题文本 (默认 "出错了")
const titleText = computed(() => {
  return props.title || '出错了'
})

// 处理重试按钮点击
const handleRetry = () => {
  props.retryHandler?.()
}
</script>

<style scoped>
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-3xl) var(--space-xl);
  text-align: center;
  min-height: 300px;
}

.error-state-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 80px;
  height: 80px;
  margin-bottom: var(--space-lg);
  border-radius: var(--radius-full);
  background: rgb(var(--accent-danger-rgb), 0.1);
  color: var(--state-error);
  transition: all 0.2s ease;
}

.error-state-title {
  font-size: var(--font-size-xl);
  font-weight: var(--font-weight-semibold);
  color: var(--text-primary);
  margin-bottom: var(--space-sm);
  line-height: var(--line-height-tight);
}

.error-state-message {
  font-size: var(--font-size-base);
  color: var(--text-secondary);
  max-width: 480px;
  margin: 0 auto var(--space-lg);
  line-height: var(--line-height-relaxed);
  padding: var(--space-md);
  background: rgb(var(--accent-danger-rgb), 0.05);
  border-left: 3px solid var(--accent-danger);
  border-radius: var(--radius-sm);
  text-align: left;
}

.error-state-action {
  margin-top: var(--space-md);
}

.error-state-button {
  display: inline-flex;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-sm) var(--space-lg);
  font-size: var(--font-size-base);
  font-weight: var(--font-weight-medium);
  color: white;
  background: var(--accent-danger);
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 2px 8px rgb(var(--accent-danger-rgb), 0.2);
}

.error-state-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgb(var(--accent-danger-rgb), 0.3);
  filter: brightness(1.1);
}

.error-state-button:active {
  transform: translateY(0);
  box-shadow: 0 1px 4px rgb(var(--accent-danger-rgb), 0.2);
}

.error-state-button:focus-visible {
  outline: 2px solid var(--accent-danger);
  outline-offset: 2px;
}

/* 尊重用户的动画偏好 */
@media (prefers-reduced-motion: reduce) {
  .error-state-icon,
  .error-state-button {
    transition: none;
  }

  .error-state-button:hover {
    transform: none;
  }

  .error-state-button:active {
    transform: none;
  }
}
</style>
