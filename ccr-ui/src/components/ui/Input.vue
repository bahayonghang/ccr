<template>
  <div
    class="relative group"
    :class="{ 'w-full': fullWidth }"
  >
    <!-- Label -->
    <label 
      v-if="label" 
      class="mb-1.5 ml-1 block text-xs font-semibold tracking-wide text-text-muted transition-colors group-hover:text-text-secondary group-focus-within:text-accent-primary"
      :for="id"
    >
      {{ label }}
    </label>

    <div class="relative">
      <!-- Leading Icon -->
      <div 
        v-if="$slots.leading" 
        class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3 text-text-muted transition-colors group-focus-within:text-accent-primary"
      >
        <slot name="leading" />
      </div>

      <!-- Input Field -->
      <input
        :id="id"
        ref="inputRef"
        v-bind="$attrs"
        :value="modelValue"
        :type="type"
        :disabled="disabled"
        :placeholder="placeholder"
        class="peer w-full rounded-xl border border-border-default/70 bg-bg-elevated/84 px-4 py-2.5 text-sm text-text-primary shadow-sm transition-[background-color,border-color,box-shadow,color] duration-300 placeholder:text-text-muted/80 focus:outline-none focus:ring-2 focus:ring-accent-primary/18 focus:border-accent-primary/42 focus:bg-bg-elevated/96 disabled:cursor-not-allowed disabled:opacity-50 hover:border-border-strong hover:bg-bg-elevated/92"
        :class="[
          $slots.leading ? 'pl-10' : '',
          $slots.trailing ? 'pr-10' : '',
          error ? '!border-accent-danger !focus:ring-accent-danger/50' : '',
          fullWidth ? 'w-full' : ''
        ]"
        :aria-invalid="Boolean(error)"
        @input="handleInput"
      >

      <!-- Trailing Icon -->
      <div 
        v-if="$slots.trailing" 
        class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3 text-text-muted"
      >
        <slot name="trailing" />
      </div>

      <!-- Neo Glow Effect on Focus -->
      <div 
        class="absolute -inset-0.5 bg-accent-primary/16 rounded-xl blur opacity-0 transition-opacity duration-300 peer-focus:opacity-100 -z-10 pointer-events-none"
        :class="error ? 'bg-accent-danger/20' : ''"
      />
    </div>

    <!-- Error Message -->
    <div 
      v-if="error" 
      class="mt-1.5 ml-1 flex items-center gap-1 text-xs text-accent-danger animate-slide-up"
    >
      <span>•</span>
      {{ error }}
    </div>
    <div 
      v-else-if="hint" 
      class="mt-1.5 ml-1 text-xs text-text-muted"
    >
      {{ hint }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

interface Props {
  modelValue?: string | number
  label?: string
  id?: string
  type?: string
  placeholder?: string
  disabled?: boolean
  error?: string
  hint?: string
  fullWidth?: boolean
}

withDefaults(defineProps<Props>(), {
  modelValue: '',
  type: 'text',
  disabled: false,
  fullWidth: true,
  placeholder: ''
})

const emit = defineEmits(['update:modelValue'])
const inputRef = ref<HTMLInputElement | null>(null)

const handleInput = (event: Event) => {
  const target = event.target as HTMLInputElement
  emit('update:modelValue', target.value)
}

defineExpose({
  focus: () => inputRef.value?.focus()
})
</script>

<script lang="ts">
export default {
  inheritAttrs: false
}
</script>
