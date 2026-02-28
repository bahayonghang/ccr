<template>
  <Teleport to="body">
    <Transition name="toast-slide">
      <div
        v-if="progress"
        class="install-toast"
      >
        <div class="install-toast__icon">
          <Loader2
            v-if="progress.phase === 'downloading' || progress.phase === 'installing'"
            class="w-5 h-5 animate-spin"
          />
          <CheckCircle
            v-else-if="progress.phase === 'done'"
            class="w-5 h-5 text-success"
          />
          <XCircle
            v-else-if="progress.phase === 'error'"
            class="w-5 h-5 text-danger"
          />
        </div>

        <div class="install-toast__body">
          <p class="install-toast__title">
            {{ phaseLabel }}
          </p>
          <p class="install-toast__package">
            {{ progress.package }}
          </p>
          <p
            v-if="progress.message"
            class="install-toast__message"
          >
            {{ progress.message }}
          </p>
        </div>

        <button
          v-if="progress.phase === 'done' || progress.phase === 'error'"
          class="install-toast__close"
          @click="dismiss"
        >
          <X class="w-4 h-4" />
        </button>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Loader2, CheckCircle, XCircle, X } from 'lucide-vue-next'
import type { InstallProgress } from '@/types/skills'

const props = defineProps<{
  progress: InstallProgress | null
}>()

const emit = defineEmits<{
  (e: 'dismiss'): void
}>()

const { t } = useI18n()

const phaseLabel = computed(() => {
  if (!props.progress) return ''
  switch (props.progress.phase) {
    case 'downloading': return t('skills.downloadingPhase')
    case 'installing': return t('skills.installingPhase')
    case 'done': return t('skills.donePhase')
    case 'error': return t('skills.errorPhase')
    default: return ''
  }
})

// 安装成功后 5 秒自动消失
watch(
  () => props.progress?.phase,
  (phase) => {
    if (phase === 'done') {
      setTimeout(() => {
        dismiss()
      }, 5000)
    }
  }
)

function dismiss() {
  emit('dismiss')
}
</script>

<style scoped>
.install-toast {
  @apply fixed bottom-6 right-6 z-50 flex items-start gap-3
         px-4 py-3 rounded-2xl
         border border-border-subtle
         shadow-2xl max-w-sm;

  background: rgb(var(--color-bg-elevated-rgb) / 90%);
  backdrop-filter: blur(16px);
}

.install-toast__icon {
  @apply shrink-0 mt-0.5;

  color: rgb(var(--color-accent-primary-rgb));
}

.install-toast__body {
  @apply flex flex-col gap-0.5 min-w-0;
}

.install-toast__title {
  @apply text-sm font-semibold text-text-primary;
}

.install-toast__package {
  @apply text-xs text-text-secondary font-mono truncate;
}

.install-toast__message {
  @apply text-xs text-text-muted mt-1;
}

.install-toast__close {
  @apply shrink-0 p-1 rounded-lg text-text-muted
         hover:text-text-primary hover:bg-bg-surface
         transition-colors;
}

.text-success {
  color: rgb(var(--color-success-rgb));
}

.text-danger {
  color: rgb(var(--color-danger-rgb));
}

/* Toast 动画 */
.toast-slide-enter-active,
.toast-slide-leave-active {
  transition: all 0.3s ease;
}

.toast-slide-enter-from {
  opacity: 0;
  transform: translateX(100%) translateY(20px);
}

.toast-slide-leave-to {
  opacity: 0;
  transform: translateX(100%);
}
</style>
