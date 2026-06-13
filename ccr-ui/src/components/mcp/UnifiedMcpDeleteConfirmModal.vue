<template>
  <Teleport to="body">
    <div
      v-if="show"
      class="modal-overlay"
      @click.self="close"
    >
      <div class="modal-content modal-content--sm">
        <div class="modal-header">
          <h2>确认删除</h2>
          <button
            class="modal-close"
            @click="close"
          >
            <SIcon
              name="X"
              size="w-5 h-5"
            />
          </button>
        </div>
        <div class="delete-confirm-body">
          <SIcon
            name="AlertCircle"
            size="w-10 h-10"
            class="text-[var(--color-danger)]"
          />
          <p>
            确定要从 <strong>{{ platformLabel }}</strong>
            删除服务器 <strong>{{ serverName }}</strong> 吗？
          </p>
          <p class="text-sm text-[var(--color-text-muted)]">
            此操作不可撤销
          </p>
        </div>
        <div class="form-actions">
          <button
            class="btn-cancel"
            @click="close"
          >
            取消
          </button>
          <button
            class="btn-danger"
            @click="confirm"
          >
            删除
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'

defineProps<{
  show: boolean
  platformLabel: string
  serverName: string
  close: () => void
  confirm: () => void
}>()
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: var(--layer-popover);
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgb(0 0 0 / 50%);
  backdrop-filter: blur(4px);
  padding: var(--space-4);
}

.modal-content {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-xl);
  width: 100%;
  box-shadow: var(--shadow-2xl);
}

.modal-content--sm {
  max-width: 420px;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-4) var(--space-5);
  border-bottom: 1px solid var(--color-border-default);
}

.modal-header h2 {
  font-size: 1.125rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

.modal-close {
  display: flex;
  padding: 4px;
  border: none;
  background: none;
  color: var(--color-text-muted);
  cursor: pointer;
  border-radius: var(--radius-sm);
}

.modal-close:hover {
  color: var(--color-text-primary);
  background: var(--glass-bg-medium);
}

.delete-confirm-body {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-6) var(--space-5) var(--space-2);
  text-align: center;
  color: var(--color-text-primary);
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-5) var(--space-4);
}

.btn-cancel {
  padding: 8px 16px;
  font-size: 0.8125rem;
  font-weight: 500;
  border-radius: var(--radius-md);
  background: var(--glass-bg-light);
  border: 1px solid var(--color-border-default);
  color: var(--color-text-secondary);
  cursor: pointer;
}

.btn-cancel:hover { background: var(--glass-bg-medium); }

.btn-danger {
  padding: 8px 20px;
  font-size: 0.8125rem;
  font-weight: 500;
  border-radius: var(--radius-md);
  background: var(--color-danger);
  color: white;
  border: none;
  cursor: pointer;
}

.btn-danger:hover { opacity: 0.85; }
</style>
