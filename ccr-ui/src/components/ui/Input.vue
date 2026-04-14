<template>
  <div
    class="relative group"
    :class="{ 'w-full': props.fullWidth }"
  >
    <!-- Label -->
    <label 
      v-if="props.label"
      class="mb-1.5 ml-1 block text-xs font-semibold tracking-wide text-text-muted transition-colors group-hover:text-text-secondary group-focus-within:text-accent-primary"
      :for="props.id"
    >
      {{ props.label }}
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
        :id="props.id"
        ref="inputRef"
        v-bind="$attrs"
        :value="props.modelValue"
        :type="props.type"
        :disabled="props.disabled"
        :placeholder="props.placeholder"
        class="peer rounded-2xl border border-border-default/70 text-text-primary placeholder:text-text-muted/80 focus:outline-none focus:ring-2 focus:ring-accent-primary/14 focus:border-accent-primary/28 disabled:cursor-not-allowed disabled:opacity-50"
        :class="[
          ...inputClasses,
          inputDensityClass,
          $slots.leading ? 'pl-10' : '',
          $slots.trailing ? 'pr-10' : '',
          props.error ? '!border-accent-danger !focus:ring-accent-danger/50' : '',
          props.fullWidth ? 'w-full' : ''
        ]"
        :data-surface="props.surface"
        :data-elevation="props.elevation"
        :data-motion="props.motion"
        :data-density="props.density"
        :aria-invalid="Boolean(props.error)"
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
        class="absolute -inset-0.5 rounded-2xl bg-accent-primary/10 blur-md opacity-0 transition-opacity duration-300 peer-focus:opacity-100 -z-10 pointer-events-none"
        :class="props.error ? 'bg-accent-danger/20' : ''"
      />
    </div>

    <!-- Error Message -->
    <div 
      v-if="props.error"
      class="mt-1.5 ml-1 flex items-center gap-1 text-xs text-accent-danger animate-slide-up"
    >
      <span>•</span>
      {{ props.error }}
    </div>
    <div 
      v-else-if="props.hint"
      class="mt-1.5 ml-1 text-xs text-text-muted"
    >
      {{ props.hint }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

type InputSurface = 'workspace' | 'card' | 'modal' | 'status'
type InputElevation = 0 | 1 | 2 | 3 | 4
type InputMotion = 'none' | 'subtle' | 'standard'
type InputDensity = 'compact' | 'default'

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
  surface?: InputSurface
  elevation?: InputElevation
  motion?: InputMotion
  density?: InputDensity
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  type: 'text',
  disabled: false,
  fullWidth: true,
  placeholder: '',
  surface: 'workspace',
  elevation: 1,
  motion: 'subtle',
  density: 'default',
})

const emit = defineEmits(['update:modelValue'])
const inputRef = ref<HTMLInputElement | null>(null)
const inputDensityClass = computed(() => (props.density === 'compact' ? 'px-3 py-2 text-sm' : 'px-4 py-2.5 text-sm'))
const inputClasses = computed(() => [
  'ui-input',
  `ui-input--surface-${props.surface}`,
  `ui-input--elevation-${props.elevation}`,
  `ui-input--motion-${props.motion}`,
  `ui-input--density-${props.density}`,
])

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

<style scoped>
.ui-input {
  transition-property: background-color, border-color, box-shadow, color, transform;
  transition-duration: var(--ui-input-duration, var(--motion-subtle-duration));
  transition-timing-function: var(--ui-input-ease, var(--motion-subtle-ease));
  box-shadow:
    var(--ui-input-shadow, var(--shadow-sm)),
    inset 0 1px 0 rgb(255 255 255 / 14%);
}

.ui-input:hover:not(:disabled) {
  border-color: rgb(var(--color-border-strong-rgb) / 28%);
}

.ui-input--surface-workspace {
  background: var(--surface-workspace-bg);
  backdrop-filter: var(--surface-workspace-blur);
}

.ui-input--surface-card {
  background: var(--surface-card-bg);
  backdrop-filter: var(--surface-card-blur);
}

.ui-input--surface-modal {
  background: var(--surface-modal-bg);
  backdrop-filter: var(--surface-modal-blur);
}

.ui-input--surface-status {
  background: var(--surface-status-bg);
  backdrop-filter: var(--surface-status-blur);
}

.ui-input--elevation-0 {
  --ui-input-shadow: none;
}

.ui-input--elevation-1 {
  --ui-input-shadow: var(--elevation-1);
}

.ui-input--elevation-2 {
  --ui-input-shadow: var(--elevation-2);
}

.ui-input--elevation-3 {
  --ui-input-shadow: var(--elevation-3);
}

.ui-input--elevation-4 {
  --ui-input-shadow: var(--elevation-4);
}

.ui-input--motion-none {
  --ui-input-duration: var(--motion-none-duration);
}

.ui-input--motion-subtle {
  --ui-input-duration: var(--motion-subtle-duration);
  --ui-input-ease: var(--motion-subtle-ease);
}

.ui-input--motion-standard {
  --ui-input-duration: var(--motion-standard-duration);
  --ui-input-ease: var(--motion-standard-ease);
}
</style>
