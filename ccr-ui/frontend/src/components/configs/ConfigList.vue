<template>
  <div class="space-y-4">
    <!-- Loading State -->
    <div
      v-if="loading"
      class="flex flex-col items-center justify-center py-20 text-text-muted"
    >
      <Spinner
        size="xl"
        class="text-accent-primary mb-4"
      />
      <span class="text-sm font-mono animate-pulse">Loading configurations...</span>
    </div>

    <!-- Error State -->
    <div
      v-else-if="error"
      class="p-4 rounded-xl bg-accent-danger/10 border border-accent-danger/20 flex items-center gap-3 text-accent-danger"
    >
      <AlertCircle class="w-5 h-5 shrink-0" />
      <span>{{ $t('configs.operationFailed') }}: {{ error }}</span>
    </div>

    <!-- Empty State -->
    <div
      v-else-if="configs.length === 0"
      class="py-20 flex flex-col items-center justify-center text-text-muted"
    >
      <div class="w-16 h-16 rounded-2xl bg-bg-surface flex items-center justify-center mb-4">
        <Settings class="w-8 h-8 opacity-20" />
      </div>
      <p>{{ $t('configs.noConfigsInCategory') }}</p>
    </div>

    <!-- Config Grid -->
    <div
      v-else
      class="grid grid-cols-1 xl:grid-cols-2 gap-4"
    >
      <ConfigCard
        v-for="config in configs"
        :key="config.name"
        :config="config"
        @switch="$emit('switch', $event)"
        @edit="$emit('edit', $event)"
        @delete="$emit('delete', $event)"
        @enable="$emit('enable', $event)"
        @disable="$emit('disable', $event)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { AlertCircle, Settings } from 'lucide-vue-next'
import type { ConfigItem } from '@/types'
import ConfigCard from '@/components/ConfigCard.vue'
import Spinner from '@/components/ui/Spinner.vue'

defineProps<{
  configs: ConfigItem[]
  loading: boolean
  error: string | null
}>()

defineEmits<{
  switch: [name: string]
  edit: [name: string]
  delete: [name: string]
  enable: [name: string]
  disable: [name: string]
}>()
</script>
