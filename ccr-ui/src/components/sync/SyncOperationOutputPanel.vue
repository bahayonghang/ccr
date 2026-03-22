<template>
  <div
    v-if="output"
    class="glass-card p-6"
  >
    <div class="mb-4 flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div
          class="rounded-xl p-2"
          :style="{ background: 'rgba(var(--color-accent-primary-rgb), 0.1)' }"
        >
          <SIcon
            name="Terminal"
            size="w-5 h-5"
            :style="{ color: 'var(--accent-primary)' }"
          />
        </div>
        <h2
          class="text-xl font-bold"
          :style="{ color: 'var(--text-primary)' }"
        >
          {{ $t('sync.output.title') }}
        </h2>
      </div>
      <button
        class="glass-card rounded-lg p-2 transition-[color,background-color,border-color,transform] duration-300 hover:scale-110"
        :style="{ background: 'rgba(var(--color-gray-rgb), 0.1)' }"
        @click="clearOutput"
      >
        <SIcon
          name="XCircle"
          size="w-4 h-4"
          :style="{ color: 'var(--text-muted)' }"
        />
      </button>
    </div>
    <pre
      class="glass-card overflow-x-auto whitespace-pre-wrap rounded-lg p-4 font-mono text-sm"
      :style="{ color: 'var(--text-primary)', background: 'rgba(255, 255, 255, 0.5)' }"
    >{{ output }}</pre>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'

interface Props {
  output: string
  clearOutput: () => void
}

defineProps<Props>()
</script>
