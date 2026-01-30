<template>
  <div
    class="relative group"
    :class="{ 'w-full': fullWidth }"
  >
    <!-- Label -->
    <label 
      v-if="label" 
      class="block text-xs font-bold uppercase tracking-wider text-text-muted mb-1.5 ml-1 transition-colors group-hover:text-text-secondary peer-focus:text-accent-primary"
      :for="id"
    >
      {{ label }}
    </label>

    <div class="relative">
      <!-- Leading Icon -->
      <div 
        v-if="$slots.leading" 
        class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-text-muted transition-colors peer-focus:text-text-primary"
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
        class="peer w-full bg-bg-surface/50 border border-border-default rounded-lg px-4 py-2.5 text-sm text-text-primary placeholder-text-muted/70 transition-all duration-300 focus:outline-none focus:ring-2 focus:ring-accent-primary/50 focus:border-accent-primary focus:bg-bg-elevated/80 disabled:opacity-50 disabled:cursor-not-allowed shadow-sm hover:border-border-strong hover:bg-bg-surface/80"
        :class="[
          $slots.leading ? 'pl-10' : '',
          $slots.trailing ? 'pr-10' : '',
          error ? '!border-accent-danger !focus:ring-accent-danger/50' : '',
          fullWidth ? 'w-full' : ''
        ]"
        @input="handleInput"
      >

      <!-- Trailing Icon -->
      <div 
        v-if="$slots.trailing" 
        class="absolute inset-y-0 right-0 pr-3 flex items-center pointer-events-none text-text-muted"
      >
        <slot name="trailing" />
      </div>

      <!-- Neo Glow Effect on Focus -->
      <div 
        class="absolute -inset-0.5 bg-accent-primary/20 rounded-xl blur opacity-0 transition-opacity duration-300 peer-focus:opacity-100 -z-10 pointer-events-none"
        :class="error ? 'bg-accent-danger/20' : ''"
      />
    </div>

    <!-- Error Message -->
    <div 
      v-if="error" 
      class="mt-1.5 ml-1 text-xs text-accent-danger flex items-center gap-1 animate-slide-up"
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
