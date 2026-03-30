<template>
  <div class="date-range-picker relative w-full sm:w-auto">
    <button
      type="button"
      class="glass-card inline-flex min-h-[44px] w-full items-center justify-between gap-2 rounded-xl px-4 py-2.5 text-left text-sm font-medium text-text-primary transition-[color,background-color,border-color,transform] duration-200 hover:border-accent-primary/40 hover:text-accent-primary focus:outline-none focus:ring-2 focus:ring-accent-primary/30 sm:w-auto sm:min-w-[17rem]"
      :aria-expanded="isOpen"
      aria-haspopup="dialog"
      @click="togglePicker"
    >
      <span class="flex min-w-0 items-center gap-2">
        <SIcon
          name="Calendar"
          size="h-4 w-4"
          class="flex-none"
        />
        <span class="truncate">
          <span v-if="!modelValue.startDate && !modelValue.endDate">
            Select Date Range
          </span>
          <span v-else>
            {{ formatDate(modelValue.startDate) }} - {{ formatDate(modelValue.endDate) }}
          </span>
        </span>
      </span>
      <SIcon
        name="ChevronDown"
        size="h-4 w-4"
        class="flex-none transition-transform duration-300"
        :class="{ 'rotate-180': isOpen }"
      />
    </button>

    <div
      v-if="isOpen"
      class="date-range-picker__panel absolute left-0 right-0 top-full mt-2 w-full sm:right-auto sm:w-[22rem]"
      @click.stop
    >
      <div class="date-range-picker__surface rounded-2xl p-4">
        <div class="mb-4 border-b border-border-default/60 pb-4">
          <h4 class="mb-3 text-sm font-semibold text-text-primary">
            Quick Select
          </h4>
          <div class="grid grid-cols-1 gap-2 sm:grid-cols-2">
            <button
              v-for="preset in presets"
              :key="preset.label"
              type="button"
              class="date-range-picker__preset min-h-[40px] rounded-xl px-3 py-2 text-sm"
              @click="selectPreset(preset)"
            >
              {{ preset.label }}
            </button>
          </div>
        </div>

        <div class="space-y-4">
          <h4 class="text-sm font-semibold text-text-primary">
            Custom Range
          </h4>

          <div>
            <label
              for="date-range-start"
              class="mb-1 block text-xs font-medium text-text-secondary"
            >
              Start Date
            </label>
            <input
              id="date-range-start"
              v-model="localStartDate"
              type="date"
              class="date-range-picker__input w-full rounded-xl px-3 py-2.5 text-sm text-text-primary"
              :max="localEndDate || today"
            >
          </div>

          <div>
            <label
              for="date-range-end"
              class="mb-1 block text-xs font-medium text-text-secondary"
            >
              End Date
            </label>
            <input
              id="date-range-end"
              v-model="localEndDate"
              type="date"
              class="date-range-picker__input w-full rounded-xl px-3 py-2.5 text-sm text-text-primary"
              :min="localStartDate"
              :max="today"
            >
          </div>

          <div class="flex flex-col gap-2 pt-1 sm:flex-row">
            <button
              type="button"
              class="date-range-picker__action date-range-picker__action--primary inline-flex min-h-[44px] flex-1 items-center justify-center rounded-xl px-4 py-2.5 text-sm font-semibold text-white"
              @click="applyCustomRange"
            >
              Apply
            </button>
            <button
              type="button"
              class="date-range-picker__action date-range-picker__action--secondary inline-flex min-h-[44px] items-center justify-center rounded-xl px-4 py-2.5 text-sm font-medium text-text-secondary"
              @click="clearRange"
            >
              Clear
            </button>
          </div>
        </div>
      </div>
    </div>

    <div
      v-if="isOpen"
      class="date-range-picker__scrim fixed inset-0"
      @click="isOpen = false"
    />
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, watch } from 'vue'
interface DateRange {
  startDate: string | null
  endDate: string | null
}

interface Props {
  modelValue: DateRange
}

interface Emits {
  (e: 'update:modelValue', value: DateRange): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const isOpen = ref(false)
const localStartDate = ref<string>('')
const localEndDate = ref<string>('')

const today = new Date().toISOString().split('T')[0]

const presets = [
  {
    label: 'Last 7 days',
    getDates: () => ({
      startDate: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
      endDate: today
    })
  },
  {
    label: 'Last 30 days',
    getDates: () => ({
      startDate: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
      endDate: today
    })
  },
  {
    label: 'Last 90 days',
    getDates: () => ({
      startDate: new Date(Date.now() - 90 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
      endDate: today
    })
  },
  {
    label: 'This month',
    getDates: () => {
      const now = new Date()
      const start = new Date(now.getFullYear(), now.getMonth(), 1)
      return {
        startDate: start.toISOString().split('T')[0],
        endDate: today
      }
    }
  },
  {
    label: 'Last month',
    getDates: () => {
      const now = new Date()
      const start = new Date(now.getFullYear(), now.getMonth() - 1, 1)
      const end = new Date(now.getFullYear(), now.getMonth(), 0)
      return {
        startDate: start.toISOString().split('T')[0],
        endDate: end.toISOString().split('T')[0]
      }
    }
  },
  {
    label: 'This year',
    getDates: () => {
      const now = new Date()
      const start = new Date(now.getFullYear(), 0, 1)
      return {
        startDate: start.toISOString().split('T')[0],
        endDate: today
      }
    }
  }
]

const togglePicker = () => {
  isOpen.value = !isOpen.value
  if (isOpen.value) {
    localStartDate.value = props.modelValue.startDate || ''
    localEndDate.value = props.modelValue.endDate || ''
  }
}

const selectPreset = (preset: typeof presets[0]) => {
  const dates = preset.getDates()
  emit('update:modelValue', dates)
  isOpen.value = false
}

const applyCustomRange = () => {
  if (localStartDate.value && localEndDate.value) {
    emit('update:modelValue', {
      startDate: localStartDate.value,
      endDate: localEndDate.value
    })
    isOpen.value = false
  }
}

const clearRange = () => {
  localStartDate.value = ''
  localEndDate.value = ''
  emit('update:modelValue', {
    startDate: null,
    endDate: null
  })
  isOpen.value = false
}

const formatDate = (date: string | null) => {
  if (!date) return ''
  return new Date(date).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric'
  })
}

watch(() => props.modelValue, (newValue) => {
  if (!isOpen.value) {
    localStartDate.value = newValue.startDate || ''
    localEndDate.value = newValue.endDate || ''
  }
})
</script>

<style scoped>
.date-range-picker {
  position: relative;
}

.date-range-picker__panel {
  z-index: var(--layer-dropdown);
}

.date-range-picker__surface {
  background: var(--surface-modal-bg);
  border: 1px solid var(--surface-modal-border);
  backdrop-filter: var(--surface-modal-blur);
  box-shadow: var(--surface-modal-shadow);
}

.date-range-picker__preset,
.date-range-picker__input,
.date-range-picker__action--secondary {
  background: var(--surface-status-bg);
  border: 1px solid var(--surface-status-border);
  backdrop-filter: var(--surface-status-blur);
  box-shadow: var(--elevation-1);
  transition:
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease),
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    transform var(--motion-subtle-duration) var(--motion-subtle-ease),
    color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.date-range-picker__preset:hover,
.date-range-picker__input:hover,
.date-range-picker__action--secondary:hover {
  border-color: rgb(var(--color-accent-primary-rgb) / 28%);
}

.date-range-picker__input:focus {
  outline: none;
  border-color: rgb(var(--color-accent-primary-rgb) / 40%);
  box-shadow: var(--elevation-2);
}

.date-range-picker__action--primary {
  background: linear-gradient(135deg, rgb(var(--color-accent-primary-rgb) / 96%), rgb(var(--color-accent-secondary-rgb) / 84%));
  box-shadow: var(--elevation-2);
  transition:
    transform var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.date-range-picker__action--primary:hover {
  transform: translateY(-1px);
  box-shadow: var(--elevation-3);
}

.date-range-picker__scrim {
  z-index: var(--layer-sticky);
}

@media (width <= 639px) {
  .date-range-picker {
    width: 100%;
  }
}
</style>
