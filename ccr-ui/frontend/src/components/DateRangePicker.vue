<template>
  <div class="date-range-picker relative">
    <button
      class="px-4 py-2 rounded-lg glass-card hover:scale-105 transition-all duration-200 flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300"
      @click="togglePicker"
    >
      <Calendar class="w-4 h-4" />
      <span v-if="!modelValue.startDate && !modelValue.endDate">
        Select Date Range
      </span>
      <span v-else>
        {{ formatDate(modelValue.startDate) }} - {{ formatDate(modelValue.endDate) }}
      </span>
      <ChevronDown
        class="w-4 h-4"
        :class="{ 'rotate-180': isOpen }"
      />
    </button>

    <!-- Picker Dropdown -->
    <div
      v-if="isOpen"
      class="absolute top-full left-0 mt-2 z-50 glass-card rounded-lg shadow-2xl p-4 min-w-[320px]"
      @click.stop
    >
      <!-- Quick Presets -->
      <div class="mb-4 pb-4 border-b border-gray-200 dark:border-gray-700">
        <h4 class="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
          Quick Select
        </h4>
        <div class="grid grid-cols-2 gap-2">
          <button
            v-for="preset in presets"
            :key="preset.label"
            class="px-3 py-2 text-sm rounded-lg bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 transition-colors"
            @click="selectPreset(preset)"
          >
            {{ preset.label }}
          </button>
        </div>
      </div>

      <!-- Custom Range -->
      <div class="space-y-3">
        <h4 class="text-sm font-semibold text-gray-700 dark:text-gray-300">
          Custom Range
        </h4>

        <!-- Start Date -->
        <div>
          <label class="block text-xs text-gray-600 dark:text-gray-400 mb-1">
            Start Date
          </label>
          <input
            v-model="localStartDate"
            type="date"
            class="w-full px-3 py-2 rounded-lg bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-white text-sm border border-gray-300 dark:border-gray-600 focus:ring-2 focus:ring-blue-500 focus:outline-none"
            :max="localEndDate || today"
          >
        </div>

        <!-- End Date -->
        <div>
          <label class="block text-xs text-gray-600 dark:text-gray-400 mb-1">
            End Date
          </label>
          <input
            v-model="localEndDate"
            type="date"
            class="w-full px-3 py-2 rounded-lg bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-white text-sm border border-gray-300 dark:border-gray-600 focus:ring-2 focus:ring-blue-500 focus:outline-none"
            :min="localStartDate"
            :max="today"
          >
        </div>

        <!-- Action Buttons -->
        <div class="flex gap-2 pt-2">
          <button
            class="flex-1 px-4 py-2 bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 text-white rounded-lg text-sm font-medium transition-all duration-300 hover:scale-105"
            @click="applyCustomRange"
          >
            Apply
          </button>
          <button
            class="px-4 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 rounded-lg text-sm font-medium transition-colors"
            @click="clearRange"
          >
            Clear
          </button>
        </div>
      </div>
    </div>

    <!-- Backdrop -->
    <div
      v-if="isOpen"
      class="fixed inset-0 z-40"
      @click="isOpen = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { Calendar, ChevronDown } from 'lucide-vue-next'

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

// Quick presets
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

// Sync local values with prop changes
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

.rotate-180 {
  transform: rotate(180deg);
  transition: transform 0.3s ease;
}
</style>
