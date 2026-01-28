<template>
  <div class="w-full">
    <!-- Label -->
    <label
      v-if="label"
      :for="inputId"
      :class="labelClasses"
    >
      {{ label }}
      <span
        v-if="required"
        class="text-accent-danger ml-0.5"
        aria-hidden="true"
      >*</span>
    </label>

    <!-- Input wrapper for icon support -->
    <div class="relative">
      <!-- Leading icon slot -->
      <div
        v-if="$slots.leading"
        :class="leadingIconClasses"
      >
        <slot name="leading" />
      </div>

      <!-- Input element -->
      <input
        :id="inputId"
        ref="inputRef"
        :type="type"
        :value="modelValue"
        :placeholder="placeholder"
        :disabled="disabled"
        :readonly="readonly"
        :required="required"
        :aria-invalid="!!error"
        :aria-describedby="ariaDescribedBy"
        :class="inputClasses"
        @input="handleInput"
        @focus="handleFocus"
        @blur="handleBlur"
      >

      <!-- Trailing icon slot -->
      <div
        v-if="$slots.trailing"
        :class="trailingIconClasses"
      >
        <slot name="trailing" />
      </div>

      <!-- Clear button -->
      <button
        v-if="clearable && modelValue && !disabled && !readonly"
        type="button"
        :class="clearButtonClasses"
        aria-label="清除输入"
        @click="handleClear"
      >
        <svg
          class="w-4 h-4"
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

    <!-- Help text -->
    <p
      v-if="help && !error"
      :id="`${inputId}-help`"
      class="mt-1.5 text-xs text-text-muted"
    >
      {{ help }}
    </p>

    <!-- Error message -->
    <p
      v-if="error"
      :id="`${inputId}-error`"
      class="mt-1.5 text-xs text-accent-danger flex items-center gap-1"
      role="alert"
    >
      <svg
        class="w-3.5 h-3.5 flex-shrink-0"
        fill="currentColor"
        viewBox="0 0 20 20"
      >
        <path
          fill-rule="evenodd"
          d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"
          clip-rule="evenodd"
        />
      </svg>
      {{ error }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, useSlots } from 'vue'

// Props interface
interface Props {
  /** 唯一标识符 */
  id?: string
  /** 标签文本 */
  label?: string
  /** 输入类型 */
  type?: 'text' | 'password' | 'email' | 'number' | 'tel' | 'url' | 'search'
  /** 占位符文本 */
  placeholder?: string
  /** 绑定值 (v-model) */
  modelValue?: string | number
  /** 是否禁用 */
  disabled?: boolean
  /** 是否只读 */
  readonly?: boolean
  /** 是否必填 */
  required?: boolean
  /** 帮助文本 */
  help?: string
  /** 错误信息 */
  error?: string
  /** 尺寸变体 */
  size?: 'sm' | 'md' | 'lg'
  /** 是否显示清除按钮 */
  clearable?: boolean
  /** 自定义 ARIA 标签 */
  ariaLabel?: string
}

// Emits
const emit = defineEmits<{
  'update:modelValue': [value: string | number]
  'focus': [event: FocusEvent]
  'blur': [event: FocusEvent]
  'clear': []
}>()

// Props with defaults
const props = withDefaults(defineProps<Props>(), {
  id: undefined,
  label: undefined,
  type: 'text',
  placeholder: undefined,
  modelValue: '',
  disabled: false,
  readonly: false,
  required: false,
  help: undefined,
  error: undefined,
  size: 'md',
  clearable: false,
  ariaLabel: undefined,
})

// Slots
const slots = useSlots()

// Refs
const inputRef = ref<HTMLInputElement | null>(null)
const isFocused = ref(false)

// Generate unique ID
const inputId = computed(() => props.id || `input-${Math.random().toString(36).substr(2, 9)}`)

// ARIA describedby
const ariaDescribedBy = computed(() => {
  const ids: string[] = []
  if (props.help && !props.error) ids.push(`${inputId.value}-help`)
  if (props.error) ids.push(`${inputId.value}-error`)
  return ids.length > 0 ? ids.join(' ') : undefined
})

// Size classes - following Button.vue pattern
const sizeClasses = {
  sm: {
    input: 'text-sm px-3 py-1.5 rounded-lg',
    label: 'text-xs mb-1',
    iconSize: 'w-4 h-4',
    leadingPadding: 'pl-9',
    trailingPadding: 'pr-9',
    iconLeft: 'left-3',
    iconRight: 'right-3',
  },
  md: {
    input: 'text-base px-4 py-2.5 rounded-xl',
    label: 'text-sm mb-1.5',
    iconSize: 'w-5 h-5',
    leadingPadding: 'pl-11',
    trailingPadding: 'pr-11',
    iconLeft: 'left-3.5',
    iconRight: 'right-3.5',
  },
  lg: {
    input: 'text-lg px-5 py-3.5 rounded-xl',
    label: 'text-base mb-2',
    iconSize: 'w-6 h-6',
    leadingPadding: 'pl-14',
    trailingPadding: 'pr-14',
    iconLeft: 'left-4',
    iconRight: 'right-4',
  },
}

// Label classes
const labelClasses = computed(() => [
  'block font-medium text-text-secondary',
  sizeClasses[props.size].label,
])

// Input classes
const inputClasses = computed(() => {
  const size = sizeClasses[props.size]
  const hasLeading = !!slots.leading
  const hasTrailing = !!slots.trailing || (props.clearable && props.modelValue)

  return [
    // Base styles
    'w-full border transition-all duration-200',
    'bg-bg-card text-text-primary placeholder:text-text-muted',
    'focus:outline-none',

    // Size
    size.input,

    // Padding adjustments for icons
    hasLeading ? size.leadingPadding : '',
    hasTrailing ? size.trailingPadding : '',

    // State styles
    props.disabled
      ? 'opacity-50 cursor-not-allowed bg-bg-tertiary'
      : props.readonly
        ? 'cursor-default bg-bg-secondary'
        : 'cursor-text',

    // Border & focus styles
    props.error
      ? 'border-accent-danger focus:border-accent-danger focus:ring-2 focus:ring-accent-danger/20'
      : isFocused.value
        ? 'border-accent-primary ring-2 ring-accent-primary/20'
        : 'border-border-color hover:border-border-hover focus:border-accent-primary focus:ring-2 focus:ring-accent-primary/20',

    // Shadow
    'shadow-sm',
  ]
})

// Leading icon classes
const leadingIconClasses = computed(() => {
  const size = sizeClasses[props.size]
  return [
    'absolute top-1/2 -translate-y-1/2 text-text-muted pointer-events-none',
    size.iconLeft,
    size.iconSize,
  ]
})

// Trailing icon classes
const trailingIconClasses = computed(() => {
  const size = sizeClasses[props.size]
  return [
    'absolute top-1/2 -translate-y-1/2 text-text-muted pointer-events-none',
    size.iconRight,
    size.iconSize,
  ]
})

// Clear button classes
const clearButtonClasses = computed(() => {
  const size = sizeClasses[props.size]
  return [
    'absolute top-1/2 -translate-y-1/2 text-text-muted',
    'hover:text-text-secondary focus:text-text-primary',
    'transition-colors duration-150',
    'focus:outline-none focus:ring-2 focus:ring-accent-primary/30 rounded',
    size.iconRight,
  ]
})

// Event handlers
function handleInput(event: Event) {
  const target = event.target as HTMLInputElement
  emit('update:modelValue', props.type === 'number' ? Number(target.value) : target.value)
}

function handleFocus(event: FocusEvent) {
  isFocused.value = true
  emit('focus', event)
}

function handleBlur(event: FocusEvent) {
  isFocused.value = false
  emit('blur', event)
}

function handleClear() {
  emit('update:modelValue', '')
  emit('clear')
  inputRef.value?.focus()
}

// Expose methods
defineExpose({
  focus: () => inputRef.value?.focus(),
  blur: () => inputRef.value?.blur(),
  select: () => inputRef.value?.select(),
})
</script>
