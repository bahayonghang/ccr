<template>
  <Card 
    :variant="config.is_current ? 'glass' : 'elevated'"
    :glow="config.is_current"
    class="h-full flex flex-col group"
  >
    <div class="p-5 flex flex-col h-full">
      <!-- Header -->
      <div class="flex items-start justify-between mb-4">
        <div class="flex flex-col gap-1">
          <div class="flex items-center gap-2">
            <!-- Provider Icon (Mock) -->
            <div class="w-6 h-6 rounded bg-bg-surface flex items-center justify-center text-xs">
              {{ config.provider?.[0]?.toUpperCase() || 'C' }}
            </div>
            <h3 class="text-lg font-bold font-display text-text-primary tracking-tight">
              {{ config.name }}
            </h3>
          </div>
          
          <div class="flex flex-wrap gap-2 mt-1">
            <span
              v-if="config.is_current"
              class="text-[10px] font-bold uppercase tracking-wider text-accent-success bg-accent-success/10 px-2 py-0.5 rounded-full"
            >Active</span>
            <span
              v-if="config.is_default"
              class="text-[10px] font-bold uppercase tracking-wider text-accent-warning bg-accent-warning/10 px-2 py-0.5 rounded-full"
            >Default</span>
          </div>
        </div>
        
        <!-- Usage Count (Neo Style) -->
        <div class="text-right">
          <div class="text-[10px] text-text-muted uppercase tracking-wider font-bold">
            Calls
          </div>
          <div class="font-mono text-accent-primary font-bold">
            {{ config.usage_count || 0 }}
          </div>
        </div>
      </div>

      <!-- Content Grid -->
      <div class="grid grid-cols-2 gap-y-2 gap-x-4 text-sm mb-6 flex-1">
        <div class="col-span-2 text-text-secondary line-clamp-2 text-xs mb-2">
          {{ config.description || 'No description provided.' }}
        </div>
        
        <div class="flex flex-col">
          <span class="text-[10px] text-text-muted uppercase">Provider</span>
          <span class="font-mono text-xs">{{ config.provider }}</span>
        </div>
        <div class="flex flex-col">
          <span class="text-[10px] text-text-muted uppercase">Model</span>
          <span
            class="font-mono text-xs truncate"
            :title="config.model"
          >{{ config.model || '-' }}</span>
        </div>
      </div>

      <!-- Actions Footer -->
      <div class="flex items-center gap-2 pt-4 border-t border-border-subtle mt-auto opacity-80 group-hover:opacity-100 transition-opacity">
        <Button 
          v-if="!config.is_current" 
          size="sm" 
          variant="primary" 
          class="flex-1"
          @click="$emit('switch', config.name)"
        >
          Switch
        </Button>
        <Button 
          v-else 
          size="sm" 
          variant="glass" 
          class="flex-1 cursor-default text-accent-success border-accent-success/20 bg-accent-success/5"
        >
          Active
        </Button>
        
        <Button
          size="sm"
          variant="ghost"
          class="px-2"
          @click="$emit('edit', config.name)"
        >
          <Settings class="w-4 h-4" />
        </Button>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { Settings } from 'lucide-vue-next'
import type { ConfigItem } from '@/types'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'

interface Props {
  config: ConfigItem
}

defineProps<Props>()

defineEmits<{
  switch: [name: string]
  edit: [name: string]
  delete: [name: string]
  enable: [name: string]
  disable: [name: string]
}>()
</script>
