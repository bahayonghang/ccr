<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition-opacity duration-200 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition-opacity duration-150 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
      @after-enter="handleAfterEnter"
      @after-leave="handleAfterLeave"
    >
      <div
        v-if="modelValue"
        class="fixed inset-0 z-50 flex items-center justify-center p-4"
        @click.self="handleBackdropClick"
      >
        <!-- 背景遮罩 -->
        <div
          class="absolute inset-0 bg-black/50 backdrop-blur-sm"
          aria-hidden="true"
        />

        <!-- 模态框内容 -->
        <Transition
          enter-active-class="transition-all duration-200 ease-out"
          enter-from-class="opacity-0 scale-95 translate-y-4"
          enter-to-class="opacity-100 scale-100 translate-y-0"
          leave-active-class="transition-all duration-150 ease-in"
          leave-from-class="opacity-100 scale-100 translate-y-0"
          leave-to-class="opacity-0 scale-95 translate-y-4"
        >
          <div
            v-if="modelValue"
            ref="modalRef"
            role="dialog"
            aria-modal="true"
            :aria-labelledby="titleId"
            :aria-describedby="hasDescription ? descId : undefined"
            :class="modalClasses"
          >
            <!-- Header -->
            <div
              v-if="$slots.header || title"
              :class="headerClasses"
            >
              <slot name="header">
                <h2
                  :id="titleId"
                  class="text-lg font-semibold text-text-primary"
                >
                  {{ title }}
                </h2>
              </slot>

              <!-- 关闭按钮 -->
              <button
                v-if="showClose"
                type="button"
                class="absolute top-4 right-4 p-1.5 rounded-lg text-text-muted
                       hover:text-text-primary hover:bg-bg-hover
                       focus:outline-none focus:ring-2 focus:ring-accent-primary/30
                       transition-colors duration-150"
                aria-label="关闭"
                @click="handleClose"
              >
                <svg
                  class="w-5 h-5"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            </div>

            <!-- Body -->
            <div :class="bodyClasses">
              <p
                v-if="hasDescription"
                :id="descId"
                class="sr-only"
              >
                {{ description }}
              </p>
              <slot />
            </div>

            <!-- Footer -->
            <div
              v-if="$slots.footer"
              :class="footerClasses"
            >
              <slot name="footer" />
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, ref, watch, useSlots } from 'vue'
import { useFocusTrap, useEscapeKey, useUniqueId, focusUtils } from '@/composables/useAccessibility'

// Props interface
interface Props {
  /** 控制模态框显示 (v-model) */
  modelValue: boolean
  /** 标题文本 */
  title?: string
  /** 描述文本 (用于 ARIA) */
  description?: string
  /** 尺寸变体 */
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full'
  /** 是否显示关闭按钮 */
  showClose?: boolean
  /** 点击遮罩是否关闭 */
  closeOnBackdrop?: boolean
  /** 按 Escape 是否关闭 */
  closeOnEscape?: boolean
  /** 是否为持久模态框 (不可关闭) */
  persistent?: boolean
  /** 自定义类名 */
  contentClass?: string
}

// Emits
const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'close': []
  'open': []
}>()

// Props with defaults
const props = withDefaults(defineProps<Props>(), {
  title: undefined,
  description: undefined,
  size: 'md',
  showClose: true,
  closeOnBackdrop: true,
  closeOnEscape: true,
  persistent: false,
  contentClass: undefined,
})

// Slots
const slots = useSlots()

// Refs
const modalRef = ref<HTMLElement | null>(null)
const isOpenRef = computed(() => props.modelValue)

// 生成唯一 ID
const titleId = useUniqueId('modal-title')
const descId = useUniqueId('modal-desc')

// 焦点管理
const focusStore = focusUtils.createFocusStore()
const { focusFirstElement } = useFocusTrap(modalRef, isOpenRef)

// Escape 键关闭
useEscapeKey(() => {
  if (props.closeOnEscape && !props.persistent) {
    handleClose()
  }
}, isOpenRef)

// 计算属性
const hasDescription = computed(() => !!props.description)

// 尺寸类
const sizeClasses = {
  sm: 'max-w-sm',
  md: 'max-w-md',
  lg: 'max-w-lg',
  xl: 'max-w-xl',
  full: 'max-w-[calc(100vw-2rem)] max-h-[calc(100vh-2rem)]',
}

// 模态框容器类
const modalClasses = computed(() => [
  // 基础样式
  'relative w-full overflow-hidden rounded-2xl',
  // 玻璃态效果
  'bg-bg-card/95 backdrop-blur-xl backdrop-saturate-150',
  'border border-border-color/50',
  'shadow-xl shadow-black/10',
  // 暗黑模式调整
  'dark:bg-bg-card/90 dark:border-border-color/30',
  'dark:shadow-2xl dark:shadow-black/30',
  // 尺寸
  sizeClasses[props.size],
  // 自定义类
  props.contentClass,
])

// Header 类
const headerClasses = computed(() => [
  'relative px-6 pt-6 pb-4',
  props.showClose ? 'pr-12' : '',
])

// Body 类
const bodyClasses = computed(() => [
  'px-6 py-2',
  // 如果有 footer，减少底部 padding
  slots.footer ? 'pb-4' : 'pb-6',
])

// Footer 类
const footerClasses = computed(() => [
  'px-6 py-4',
  'border-t border-border-color/50',
  'flex items-center justify-end gap-3',
])

// 事件处理
function handleClose() {
  if (props.persistent) return
  emit('update:modelValue', false)
  emit('close')
}

function handleBackdropClick() {
  if (props.closeOnBackdrop && !props.persistent) {
    handleClose()
  }
}

function handleAfterEnter() {
  focusFirstElement()
  emit('open')
}

function handleAfterLeave() {
  focusStore.restore()
}

// 监听打开状态，保存焦点
watch(() => props.modelValue, (isOpen) => {
  if (isOpen) {
    focusStore.save()
    // 锁定 body 滚动
    document.body.style.overflow = 'hidden'
  } else {
    // 恢复 body 滚动
    document.body.style.overflow = ''
  }
})

// 暴露方法
defineExpose({
  close: handleClose,
})
</script>
