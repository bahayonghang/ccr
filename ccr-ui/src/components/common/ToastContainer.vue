<template>
  <Teleport to="body">
    <TransitionGroup
      name="toast"
      tag="div"
      class="toast-container"
    >
      <div
        v-for="toast in toasts"
        :key="toast.id"
        :class="['toast', `toast-${toast.type}`]"
        @click="removeToast(toast.id)"
      >
        <SIcon
          :name="getIcon(toast.type)"
          class="toast-icon"
        />
        <span class="toast-message">{{ toast.message }}</span>
        <SIcon
          name="X"
          class="toast-close"
          size="w-4 h-4"
        />
      </div>
    </TransitionGroup>
  </Teleport>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { storeToRefs } from 'pinia'
import { useUIStore } from '@/stores/ui'
const uiStore = useUIStore()
const { toasts } = storeToRefs(uiStore)
const { removeToast } = uiStore

const getIcon = (type: 'success' | 'error' | 'warning' | 'info') => {
  const icons = {
    success: 'CheckCircle',
    error: 'XCircle',
    warning: 'AlertTriangle',
    info: 'Info',
  }
  return icons[type]
}
</script>

<style scoped>
.toast-container {
  position: fixed;
  top: 1rem;
  right: 1rem;
  z-index: var(--layer-toast);
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  pointer-events: none;
}

.toast {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.875rem 1rem;
  border-radius: 1rem;
  background: var(--surface-modal-bg);
  backdrop-filter: var(--surface-modal-blur);
  color: var(--color-text-primary);
  box-shadow: var(--shadow-lg);
  cursor: pointer;
  pointer-events: auto;
  min-width: 280px;
  max-width: 400px;
  border: 1px solid var(--color-border-default);
  border-left: 3px solid transparent;
  transition:
    transform var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    opacity var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.toast:hover {
  transform: translateX(-4px);
  box-shadow: var(--shadow-xl);
  border-color: var(--color-border-strong);
}

.toast-success {
  border-left-color: var(--accent-success);
}

.toast-success .toast-icon {
  color: var(--accent-success);
}

.toast-error {
  border-left-color: var(--accent-danger);
}

.toast-error .toast-icon {
  color: var(--accent-danger);
}

.toast-warning {
  border-left-color: var(--accent-warning);
}

.toast-warning .toast-icon {
  color: var(--accent-warning);
}

.toast-info {
  border-left-color: var(--accent-info);
}

.toast-info .toast-icon {
  color: var(--accent-info);
}

.toast-icon {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
}

.toast-message {
  flex: 1;
  font-size: 0.875rem;
  line-height: 1.4;
  color: inherit;
}

.toast-close {
  flex-shrink: 0;
  opacity: 0.5;
  transition: opacity 0.2s;
}

.toast:hover .toast-close {
  opacity: 1;
}

.toast-enter-active,
.toast-leave-active {
  transition:
    transform var(--motion-enter-duration) var(--motion-enter-ease),
    opacity var(--motion-enter-duration) var(--motion-enter-ease);
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(100%);
}

.toast-enter-to,
.toast-leave-from {
  opacity: 1;
  transform: translateX(0);
}

@media (prefers-reduced-motion: reduce) {
  .toast,
  .toast-enter-active,
  .toast-leave-active {
    animation: none;
    transition: none;
  }

  .toast:hover {
    transform: none;
  }
}
</style>
