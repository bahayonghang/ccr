<template>
  <div
    class="empty-state"
    role="status"
    aria-live="polite"
  >
    <!-- 图标 -->
    <div class="empty-state-icon">
      <component
        :is="iconComponent"
        :size="iconSize"
        :stroke-width="1.5"
      />
    </div>

    <!-- 标题 -->
    <h3 class="empty-state-title">
      {{ title }}
    </h3>

    <!-- 描述 (可选) -->
    <p
      v-if="description"
      class="empty-state-description"
    >
      {{ description }}
    </p>

    <!-- 操作按钮 (可选) -->
    <div
      v-if="actionLabel && actionHandler !== undefined"
      class="empty-state-action"
    >
      <button
        class="empty-state-button"
        @click="handleAction"
      >
        <Plus
          v-if="showPlusIcon"
          :size="16"
          :stroke-width="2"
        />
        {{ actionLabel }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, type Component } from 'vue'
import { FileX, Plus } from 'lucide-vue-next'

// Props 定义
interface Props {
  /** 图标组件 (Lucide 图标或自定义组件) */
  icon?: Component
  /** 标题文本 */
  title: string
  /** 描述文本 (可选) */
  description?: string
  /** 操作按钮文本 (可选) */
  actionLabel?: string
  /** 操作按钮回调 (可选) */
  actionHandler?: () => void
  /** 图标大小 */
  iconSize?: number
  /** 是否在按钮中显示加号图标 */
  showPlusIcon?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  icon: undefined,
  description: undefined,
  actionLabel: undefined,
  actionHandler: undefined,
  iconSize: 48,
  showPlusIcon: true,
})

// 计算使用的图标组件 (默认为 FileX)
const iconComponent = computed(() => {
  return props.icon || FileX
})

// 处理按钮点击
const handleAction = () => {
  props.actionHandler?.()
}
</script>

<style scoped>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-3xl) var(--space-xl);
  text-align: center;
  min-height: 300px;
}

.empty-state-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 80px;
  height: 80px;
  margin-bottom: var(--space-lg);
  border-radius: var(--radius-full);
  background: var(--bg-tertiary);
  color: var(--state-empty);
  transition: all 0.2s ease;
}

.empty-state-title {
  font-size: var(--font-size-xl);
  font-weight: var(--font-weight-semibold);
  color: var(--text-primary);
  margin-bottom: var(--space-sm);
  line-height: var(--line-height-tight);
}

.empty-state-description {
  font-size: var(--font-size-base);
  color: var(--text-secondary);
  max-width: 480px;
  margin: 0 auto var(--space-lg);
  line-height: var(--line-height-relaxed);
}

.empty-state-action {
  margin-top: var(--space-md);
}

.empty-state-button {
  display: inline-flex;
  align-items: center;
  gap: var(--space-xs);
  padding: var(--space-sm) var(--space-lg);
  font-size: var(--font-size-base);
  font-weight: var(--font-weight-medium);
  color: white;
  background: var(--accent-primary);
  border: none;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 2px 8px rgba(var(--accent-primary-rgb), 0.2);
}

.empty-state-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(var(--accent-primary-rgb), 0.3);
}

.empty-state-button:active {
  transform: translateY(0);
  box-shadow: 0 1px 4px rgba(var(--accent-primary-rgb), 0.2);
}

.empty-state-button:focus-visible {
  outline: 2px solid var(--accent-primary);
  outline-offset: 2px;
}

/* 尊重用户的动画偏好 */
@media (prefers-reduced-motion: reduce) {
  .empty-state-icon,
  .empty-state-button {
    transition: none;
  }

  .empty-state-button:hover {
    transform: none;
  }

  .empty-state-button:active {
    transform: none;
  }
}
</style>
