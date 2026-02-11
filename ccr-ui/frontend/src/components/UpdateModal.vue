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
                  : 'linear-gradient(90deg, var(--accent-primary), var(--accent-secondary))'
            }"
          />

          <!-- 头部 -->
          <div
            class="px-6 py-5 flex items-center justify-between border-b"
            :style="{ borderColor: 'var(--border-color)' }"
          >
            <div class="flex items-center space-x-3">
              <AlertTriangle
                v-if="stage === 'confirm'"
                class="w-6 h-6"
                :style="{ color: 'var(--accent-warning)' }"
              />
              <Loader2
                v-if="stage === 'updating'"
                class="w-6 h-6 animate-spin"
                :style="{ color: 'var(--accent-primary)' }"
              />
              <CheckCircle
                v-if="stage === 'success'"
                class="w-6 h-6"
                :style="{ color: 'var(--accent-success)' }"
              />
              <AlertCircle
                v-if="stage === 'error'"
                class="w-6 h-6"
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
              class="p-2 rounded-lg transition-all hover:scale-110"
              :style="{
                background: 'var(--bg-tertiary)',
                color: 'var(--text-secondary)'
              }"
              aria-label="关闭"
              @click="$emit('close')"
            >
              <X class="w-5 h-5" />
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
                确定要立即更新 CCR 吗？
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
                  ⚠️ 注意事项：
                </p>
                <ul
                  class="text-sm space-y-1.5 ml-6 list-disc"
                  :style="{ color: 'var(--text-muted)' }"
                >
                  <li>更新过程可能需要几分钟时间</li>
                  <li>更新期间请勿关闭此窗口</li>
                  <li>更新完成后需要刷新页面</li>
                  <li>建议在更新前保存当前工作</li>
                </ul>
              </div>
            </div>

            <!-- 更新中 -->
            <div
              v-if="stage === 'updating'"
              class="space-y-4"
            >
              <div class="flex items-center space-x-3">
                <Loader2
                  class="w-5 h-5 animate-spin"
                  :style="{ color: 'var(--accent-primary)' }"
                />
                <p
                  class="text-base font-medium"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  正在执行更新，请稍候...
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
                    background: 'linear-gradient(90deg, var(--accent-primary), var(--accent-secondary))'
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
                <CheckCircle
                  class="w-5 h-5 mt-0.5 flex-shrink-0"
                  :style="{ color: 'var(--accent-success)' }"
                />
                <div class="space-y-2 flex-1">
                  <p
                    class="text-base font-semibold"
                    :style="{ color: 'var(--accent-success)' }"
                  >
                    CCR 已成功更新！
                  </p>
                  <p
                    class="text-sm"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    更新已完成，建议刷新页面以使用最新版本。
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
                  查看更新日志
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
                <AlertCircle
                  class="w-5 h-5 mt-0.5 flex-shrink-0"
                  :style="{ color: 'var(--accent-danger)' }"
                />
                <div class="space-y-2 flex-1">
                  <p
                    class="text-base font-semibold"
                    :style="{ color: 'var(--accent-danger)' }"
                  >
                    更新失败
                  </p>
                  <p
                    class="text-sm"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    更新过程中出现错误，请查看错误信息并重试。
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
                  查看详细日志
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
                class="px-5 py-2.5 rounded-lg font-semibold text-sm transition-all hover:scale-105"
                :style="{
                  background: 'var(--bg-tertiary)',
                  color: 'var(--text-primary)',
                  border: '1px solid var(--border-color)'
                }"
                @click="$emit('close')"
              >
                取消
              </button>
              <button
                class="px-5 py-2.5 rounded-lg font-semibold text-sm transition-all text-white hover:scale-105"
                :style="{
                  background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
                  boxShadow: '0 0 20px var(--glow-primary)'
                }"
                @click="$emit('confirm')"
              >
                确认更新
              </button>
            </template>

            <!-- 更新中 -->
            <p
              v-if="stage === 'updating'"
              class="text-sm"
              :style="{ color: 'var(--text-muted)' }"
            >
              更新中，请勿关闭窗口...
            </p>

            <!-- 成功或错误 -->
            <template v-if="stage === 'success' || stage === 'error'">
              <button
                class="px-5 py-2.5 rounded-lg font-semibold text-sm transition-all hover:scale-105"
                :style="{
                  background: 'var(--bg-tertiary)',
                  color: 'var(--text-primary)',
                  border: '1px solid var(--border-color)'
                }"
                @click="$emit('close')"
              >
                关闭
              </button>
              <button
                v-if="stage === 'success'"
                class="px-5 py-2.5 rounded-lg font-semibold text-sm transition-all text-white hover:scale-105"
                :style="{
                  background: 'linear-gradient(135deg, var(--accent-success), var(--accent-primary))',
                  boxShadow: '0 0 20px var(--glow-success)'
                }"
                @click="handleRefresh"
              >
                刷新页面
              </button>
            </template>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  X,
  AlertTriangle,
  CheckCircle,
  Loader2,
  AlertCircle
} from 'lucide-vue-next'
import { useFocusTrap, useEscapeKey, useUniqueId } from '@/composables/useAccessibility'

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
    setTimeout(() => focusFirstElement(), 100)
  }
})

const getTitle = () => {
  switch (props.stage) {
    case 'confirm':
      return '确认更新'
    case 'updating':
      return '正在更新...'
    case 'success':
      return '更新成功'
    case 'error':
      return '更新失败'
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
