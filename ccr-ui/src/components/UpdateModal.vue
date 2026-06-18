<template>
  <!-- 背景遮罩 -->
  <Teleport to="body">
    <Transition name="modal">
      <div
        v-if="isOpen"
        class="fixed inset-0 z-50 flex items-center justify-center p-4"
        :style="{
          background: 'rgba(0, 0, 0, 0.6)',
          backdropFilter: 'blur(8px)'
        }"
        @click="handleBackdropClick"
      >
        <!-- 模态框 -->
        <div
          ref="modalRef"
          role="dialog"
          aria-modal="true"
          :aria-labelledby="titleId"
          class="relative rounded-2xl shadow-2xl w-full max-w-2xl max-h-[80vh] overflow-hidden"
          :style="{
            background: 'var(--bg-primary)',
            border: '1px solid var(--border-color)',
            boxShadow: '0 25px 50px -12px rgba(0, 0, 0, 0.5)'
          }"
          @click.stop
        >
          <!-- 顶部装饰线 -->
          <div
            class="h-1"
            :style="{
              background: stage === 'error'
                ? 'var(--accent-danger)'
                : stage === 'success'
                  ? 'var(--accent-success)'
                  : 'var(--accent-primary)'
            }"
          />

          <!-- 头部 -->
          <div
            class="px-6 py-5 flex items-center justify-between border-b"
            :style="{ borderColor: 'var(--border-color)' }"
          >
            <div class="flex items-center space-x-3">
              <SIcon
                v-if="stage === 'confirm'"
                name="AlertTriangle"
                size="w-6 h-6"
                
                :style="{ color: 'var(--accent-warning)' }"
              />
              <SIcon
                v-if="stage === 'updating'"
                name="Loader2"
                size="w-6 h-6"
                class="animate-spin"
                
                :style="{ color: 'var(--accent-primary)' }"
              />
              <SIcon
                v-if="stage === 'success'"
                name="CheckCircle"
                size="w-6 h-6"
                
                :style="{ color: 'var(--accent-success)' }"
              />
              <SIcon
                v-if="stage === 'error'"
                name="AlertCircle"
                size="w-6 h-6"
                
                :style="{ color: 'var(--accent-danger)' }"
              />

              <h2
                :id="titleId"
                class="text-xl font-bold"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ getTitle() }}
              </h2>
            </div>

            <button
              v-if="stage !== 'updating'"
              class="p-2 rounded-lg transition-transform hover:scale-110"
              :style="{
                background: 'var(--bg-tertiary)',
                color: 'var(--text-secondary)'
              }"
              :aria-label="t('common.close')"
              @click="$emit('close')"
            >
              <SIcon
                name="X"
                size="w-5 h-5"
              />
            </button>
          </div>

          <!-- 内容区域 -->
          <div class="px-6 py-5 overflow-y-auto max-h-[60vh]">
            <!-- 确认阶段 -->
            <div
              v-if="stage === 'confirm'"
              class="space-y-4"
            >
              <p
                class="text-base leading-relaxed"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ t('common.updateModal.confirmMessage') }}
              </p>
              <div
                class="rounded-lg p-4 space-y-2"
                :style="{
                  background: 'var(--bg-secondary)',
                  border: '1px solid var(--border-color)'
                }"
              >
                <p
                  class="text-sm font-semibold"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  <span class="inline-flex items-center gap-1.5">
                    <SIcon
                      name="AlertTriangle"
                      size="w-4 h-4"
                    />
                    {{ t('common.updateModal.notesTitle') }}
                  </span>
                </p>
                <ul
                  class="text-sm space-y-1.5 ml-6 list-disc"
                  :style="{ color: 'var(--text-muted)' }"
                >
                  <li>{{ t('common.updateModal.noteDuration') }}</li>
                  <li>{{ t('common.updateModal.noteDoNotClose') }}</li>
                  <li>{{ t('common.updateModal.noteRefresh') }}</li>
                  <li>{{ t('common.updateModal.noteSaveWork') }}</li>
                </ul>
              </div>
            </div>

            <!-- 更新中 -->
            <div
              v-if="stage === 'updating'"
              class="space-y-4"
            >
              <div class="flex items-center space-x-3">
                <SIcon
                  name="Loader2"
                  size="w-5 h-5"
                  class="animate-spin"
                  :style="{ color: 'var(--accent-primary)' }"
                />
                <p
                  class="text-base font-medium"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ t('common.updateModal.runningMessage') }}
                </p>
              </div>

              <!-- 输出日志 -->
              <div
                v-if="output"
                class="rounded-lg p-4 font-mono text-xs overflow-x-auto"
                :style="{
                  background: 'var(--bg-secondary)',
                  border: '1px solid var(--border-color)',
                  color: 'var(--text-secondary)',
                  maxHeight: '300px',
                  overflowY: 'auto'
                }"
              >
                <pre class="whitespace-pre-wrap">{{ output }}</pre>
              </div>

              <!-- 进度动画 -->
              <div
                class="relative h-2 rounded-full overflow-hidden"
                :style="{ background: 'var(--bg-tertiary)' }"
              >
                <div
                  class="h-full progress-bar-animation"
                  :style="{
                    background: 'var(--accent-primary)'
                  }"
                />
              </div>
            </div>

            <!-- 成功 -->
            <div
              v-if="stage === 'success'"
              class="space-y-4"
            >
              <div
                class="rounded-lg p-4 flex items-start space-x-3"
                :style="{
                  background: 'rgba(var(--color-success-rgb), 0.1)',
                  border: '1px solid var(--accent-success)'
                }"
              >
                <SIcon
                  name="CheckCircle"
                  size="w-5 h-5"
                  class="mt-0.5 flex-shrink-0"
                  :style="{ color: 'var(--accent-success)' }"
                />
                <div class="space-y-2 flex-1">
                  <p
                    class="text-base font-semibold"
                    :style="{ color: 'var(--accent-success)' }"
                  >
                    {{ t('common.updateModal.successMessage') }}
                  </p>
                  <p
                    class="text-sm"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ t('common.updateModal.successHint') }}
                  </p>
                </div>
              </div>

              <!-- 输出日志 -->
              <details
                v-if="output"
                class="cursor-pointer"
              >
                <summary
                  class="text-sm font-medium mb-2"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ t('common.updateModal.viewLog') }}
                </summary>
                <div
                  class="rounded-lg p-4 font-mono text-xs overflow-x-auto"
                  :style="{
                    background: 'var(--bg-secondary)',
                    border: '1px solid var(--border-color)',
                    color: 'var(--text-secondary)',
                    maxHeight: '200px',
                    overflowY: 'auto'
                  }"
                >
                  <pre class="whitespace-pre-wrap">{{ output }}</pre>
                </div>
              </details>
            </div>

            <!-- 错误 -->
            <div
              v-if="stage === 'error'"
              class="space-y-4"
            >
              <div
                class="rounded-lg p-4 flex items-start space-x-3"
                :style="{
                  background: 'rgba(var(--color-danger-rgb), 0.1)',
                  border: '1px solid var(--accent-danger)'
                }"
              >
                <SIcon
                  name="AlertCircle"
                  size="w-5 h-5"
                  class="mt-0.5 flex-shrink-0"
                  :style="{ color: 'var(--accent-danger)' }"
                />
                <div class="space-y-2 flex-1">
                  <p
                    class="text-base font-semibold"
                    :style="{ color: 'var(--accent-danger)' }"
                  >
                    {{ t('common.updateModal.errorTitle') }}
                  </p>
                  <p
                    class="text-sm"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ t('common.updateModal.errorMessage') }}
                  </p>
                </div>
              </div>

              <!-- 错误信息 -->
              <div
                v-if="error"
                class="rounded-lg p-4 font-mono text-xs overflow-x-auto"
                :style="{
                  background: 'var(--bg-secondary)',
                  border: '1px solid var(--accent-danger)',
                  color: 'var(--accent-danger)',
                  maxHeight: '200px',
                  overflowY: 'auto'
                }"
              >
                <pre class="whitespace-pre-wrap">{{ error }}</pre>
              </div>

              <!-- 输出日志 -->
              <details
                v-if="output"
                class="cursor-pointer"
              >
                <summary
                  class="text-sm font-medium mb-2"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ t('common.updateModal.viewDetailedLog') }}
                </summary>
                <div
                  class="rounded-lg p-4 font-mono text-xs overflow-x-auto"
                  :style="{
                    background: 'var(--bg-secondary)',
                    border: '1px solid var(--border-color)',
                    color: 'var(--text-secondary)',
                    maxHeight: '200px',
                    overflowY: 'auto'
                  }"
                >
                  <pre class="whitespace-pre-wrap">{{ output }}</pre>
                </div>
              </details>
            </div>
          </div>

          <!-- 底部按钮 -->
          <div
            class="px-6 py-4 flex items-center justify-end space-x-3 border-t"
            :style="{
              borderColor: 'var(--border-color)',
              background: 'var(--bg-secondary)'
            }"
          >
            <!-- 确认阶段 -->
            <template v-if="stage === 'confirm'">
              <button
                class="px-5 py-2.5 rounded-lg font-semibold text-sm transition-transform hover:scale-105"
                :style="{
                  background: 'var(--bg-tertiary)',
                  color: 'var(--text-primary)',
                  border: '1px solid var(--border-color)'
                }"
                @click="$emit('close')"
              >
                {{ t('common.cancel') }}
              </button>
              <button
                class="px-5 py-2.5 rounded-lg font-semibold text-sm transition-transform text-white hover:scale-105"
                :style="{
                  background: 'var(--accent-primary)'
                }"
                @click="$emit('confirm')"
              >
                {{ t('common.updateModal.confirmAction') }}
              </button>
            </template>

            <!-- 更新中 -->
            <p
              v-if="stage === 'updating'"
              class="text-sm"
              :style="{ color: 'var(--text-muted)' }"
            >
              {{ t('common.updateModal.runningHint') }}
            </p>

            <!-- 成功或错误 -->
            <template v-if="stage === 'success' || stage === 'error'">
              <button
                class="px-5 py-2.5 rounded-lg font-semibold text-sm transition-transform hover:scale-105"
                :style="{
                  background: 'var(--bg-tertiary)',
                  color: 'var(--text-primary)',
                  border: '1px solid var(--border-color)'
                }"
                @click="$emit('close')"
              >
                {{ t('common.close') }}
              </button>
              <button
                v-if="stage === 'success'"
                class="px-5 py-2.5 rounded-lg font-semibold text-sm transition-transform text-white hover:scale-105"
                :style="{
                  background: 'var(--accent-success)'
                }"
                @click="handleRefresh"
              >
                {{ t('common.updateModal.refreshPage') }}
              </button>
            </template>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFocusTrap, useEscapeKey, useUniqueId } from '@/composables/useAccessibility'
import { MODAL_FOCUS_DELAY_MS } from '@/config/constants'

interface Props {
  isOpen: boolean
  stage: 'confirm' | 'updating' | 'success' | 'error'
  output?: string
  error?: string
}

const props = withDefaults(defineProps<Props>(), {
  output: '',
  error: ''
})

const emit = defineEmits<{
  close: []
  confirm: []
}>()
const { t } = useI18n()

// Accessibility enhancements
const titleId = useUniqueId('update-modal-title')
const modalRef = ref<HTMLElement | null>(null)
const isOpenRef = ref(props.isOpen)

// Close handler for composables
const handleClose = () => {
  if (props.stage !== 'updating') {
    emit('close')
  }
}

watch(() => props.isOpen, (newValue) => {
  isOpenRef.value = newValue
})

const { focusFirstElement } = useFocusTrap(modalRef, isOpenRef)
useEscapeKey(handleClose, isOpenRef)

watch(isOpenRef, (isOpen) => {
  if (isOpen) {
    setTimeout(() => focusFirstElement(), MODAL_FOCUS_DELAY_MS)
  }
})

const getTitle = () => {
  switch (props.stage) {
    case 'confirm':
      return t('common.updateModal.confirmTitle')
    case 'updating':
      return t('common.updateModal.updatingTitle')
    case 'success':
      return t('common.updateModal.successTitle')
    case 'error':
      return t('common.updateModal.errorTitle')
    default:
      return ''
  }
}

const handleBackdropClick = () => {
  if (props.stage !== 'updating') {
    emit('close')
  }
}

const handleRefresh = () => {
  window.location.reload()
}
</script>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active > div,
.modal-leave-active > div {
  transition: transform 0.3s ease, opacity 0.3s ease;
}

.modal-enter-from > div,
.modal-leave-to > div {
  opacity: 0;
  transform: scale(0.95) translateY(-10px);
}

.progress-bar-animation {
  animation: progress-bar 2s ease-in-out infinite;
}

@keyframes progress-bar {
  0% {
    width: 0%;
  }

  100% {
    width: 100%;
  }
}
</style>
