<template>
  <div class="w-full">
    <label
      v-if="label"
      :for="id"
      class="block text-sm font-medium mb-2"
      :style="{ color: 'var(--text-secondary)' }"
    >
      {{ label }}
      <span
        v-if="required"
        class="text-accent-danger"
      >*</span>
    </label>
    <input
      :id="id"
      :type="type"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :class="inputClasses"
      :style="inputStyle"
      @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    >
    <p
      v-if="help"
      class="mt-1 text-xs"
      :style="{ color: 'var(--text-muted)' }"
    >
      {{ help }}
    </p>
    <p
      v-if="error"
      class="mt-1 text-xs text-accent-danger"
    >
      {{ error }}
    </p>
  </div>
</template>

<script setup lang="ts">
interface Props {
  id?: string;
  label?: string;
  type?: string;
  placeholder?: string;
  modelValue: string | number;
  disabled?: boolean;
  required?: boolean;
  help?: string;
  error?: string;
  fullWidth?: boolean;
}

withDefaults(defineProps<Props>(), {
  id: () => `input-${Math.random().toString(36).substr(2, 9)}`,
  type: 'text',
  disabled: false,
  required: false,
  fullWidth: true,
})

defineEmits(['update:modelValue'])

const inputClasses = [
  'w-full',
  'px-4 py-3',
  'rounded-xl',
  'border',
  'focus:outline-none',
  'focus:ring-2',
  'transition-all',
  'glass-effect',
]

const inputStyle = {
  width: '100%',
  borderWidth: '1px',
  borderColor: 'var(--border-color)',
  color: 'var(--text-primary)',
  boxShadow: 'var(--shadow-sm), inset 0 1px 2px rgba(0,0,0,0.05)',
}
</script>