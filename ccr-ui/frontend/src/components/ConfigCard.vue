<template>
  <Card
    :variant="config.is_current ? 'glass' : 'elevated'"
    :glow="config.is_current"
    v-bind="{ 'data-config-name': config.name }"
    class="config-card group transition-all duration-300"
    :class="{ 
      'config-card--expanded': expanded,
      'config-card--current': config.is_current,
      'ring-2 ring-accent-primary/30': config.is_current
    }"
  >
    <div class="p-5 flex flex-col">
      <!-- Header (Always visible) -->
      <div class="flex items-start justify-between">
        <div class="flex-1 min-w-0">
          <!-- Name & Status -->
          <div class="flex items-center gap-2 mb-1">
            <div
              class="w-7 h-7 rounded-lg bg-bg-surface flex items-center justify-center text-xs font-bold shrink-0"
              :class="config.is_current ? 'bg-accent-primary/20 text-accent-primary' : 'text-text-muted'"
            >
              {{ config.provider?.[0]?.toUpperCase() || 'C' }}
            </div>
            <h3 
              class="text-base font-bold font-display truncate transition-colors duration-300"
              :class="titleColorClass"
            >
              {{ config.name }}
            </h3>
            
            <!-- Status Badges -->
            <div class="flex gap-1 shrink-0">
              <span
                v-if="config.is_current"
                class="relative flex items-center gap-1 text-[10px] font-bold uppercase tracking-wider text-accent-success bg-accent-success/10 px-2 py-0.5 rounded-full"
              >
                <span class="w-1.5 h-1.5 bg-accent-success rounded-full animate-pulse" />
                Active
              </span>
              <span
                v-if="config.is_default"
                class="text-[10px] font-bold uppercase tracking-wider text-accent-warning bg-accent-warning/10 px-2 py-0.5 rounded-full"
              >Default</span>
            </div>
          </div>
          
          <!-- Description (Always visible) -->
          <p class="text-xs text-text-secondary line-clamp-1 mb-2 ml-9">
            {{ config.description || 'No description provided.' }}
          </p>
          
          <!-- Provider (Always visible) -->
          <div class="flex items-center gap-2 ml-9 text-xs text-text-muted">
            <Building2 class="w-3 h-3" />
            <span class="font-mono">{{ config.provider || '-' }}</span>
          </div>
        </div>
        
        <!-- Expand Toggle -->
        <button 
          class="p-1.5 rounded-lg hover:bg-bg-surface transition-colors shrink-0"
          @click="$emit('toggleExpand')"
        >
          <ChevronDown 
            class="w-4 h-4 text-text-muted transition-transform duration-300"
            :class="{ 'rotate-180': expanded }"
          />
        </button>
      </div>

      <!-- Expanded Content -->
      <Transition name="expand">
        <div
          v-if="expanded"
          class="mt-4 pt-4 border-t border-border-subtle space-y-4"
        >
          <!-- Meta Grid -->
          <div class="grid grid-cols-2 gap-3 text-sm">
            <div class="flex flex-col gap-0.5">
              <span class="text-[10px] text-text-muted uppercase font-bold">Model</span>
              <span
                class="font-mono text-xs text-text-primary truncate"
                :title="config.model"
              >
                {{ config.model || '-' }}
              </span>
            </div>
            <div class="flex flex-col gap-0.5">
              <span class="text-[10px] text-text-muted uppercase font-bold">Calls</span>
              <span class="font-mono text-xs text-accent-primary font-bold">
                {{ config.usage_count || 0 }}
              </span>
            </div>
            <div class="flex flex-col gap-0.5">
              <span class="text-[10px] text-text-muted uppercase font-bold">Type</span>
              <span class="font-mono text-xs text-text-primary">
                {{ config.provider_type || 'Unknown' }}
              </span>
            </div>
            <div
              v-if="config.account"
              class="flex flex-col gap-0.5"
            >
              <span class="text-[10px] text-text-muted uppercase font-bold">Account</span>
              <span class="font-mono text-xs text-text-primary truncate">
                {{ config.account }}
              </span>
            </div>
          </div>

          <!-- Actions -->
          <div class="flex items-center gap-2 pt-2">
            <Button 
              v-if="!config.is_current" 
              size="sm" 
              variant="primary" 
              class="flex-1"
              @click="$emit('switch', config.name)"
            >
              <ArrowRightLeft class="w-3.5 h-3.5 mr-1.5" />
              Switch
            </Button>
            <Button 
              v-else 
              size="sm" 
              variant="glass" 
              class="flex-1 cursor-default text-accent-success border-accent-success/20 bg-accent-success/5"
            >
              <CheckCircle class="w-3.5 h-3.5 mr-1.5" />
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
      </Transition>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Settings, ChevronDown, Building2, ArrowRightLeft, CheckCircle } from 'lucide-vue-next'
import type { ConfigItem } from '@/types'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'

interface Props {
  config: ConfigItem
  expanded?: boolean
}

const props = defineProps<Props>()

defineEmits<{
  switch: [name: string]
  edit: [name: string]
  delete: [name: string]
  enable: [name: string]
  disable: [name: string]
  toggleExpand: []
}>()

const titleColorClass = computed(() => {
  const type = props.config.provider_type?.toLowerCase() || ''
  if (type.includes('official')) return 'text-cyan-400 group-hover:text-cyan-300'
  if (type.includes('third')) return 'text-violet-400 group-hover:text-violet-300'
  return 'text-amber-400 group-hover:text-amber-300'
})
</script>

<style scoped>
/* Expand transition */
.expand-enter-active,
.expand-leave-active {
  transition: all 0.3s ease;
  overflow: hidden;
}

.expand-enter-from,
.expand-leave-to {
  opacity: 0;
  max-height: 0;
  margin-top: 0;
  padding-top: 0;
}

.expand-enter-to,
.expand-leave-from {
  opacity: 1;
  max-height: 300px;
}

/* Highlight pulse animation for scroll-to */
.config-card.highlight-pulse {
  animation: highlight-pulse 1.5s ease-out;
}

@keyframes highlight-pulse {
  0% {
    box-shadow: 0 0 0 0 rgb(6 182 212 / 40%);
  }

  50% {
    box-shadow: 0 0 0 8px rgb(6 182 212 / 10%);
  }

  100% {
    box-shadow: 0 0 0 0 rgb(6 182 212 / 0%);
  }
}

/* Current config glow */
.config-card--current {
  background: linear-gradient(135deg, 
    rgb(6 182 212 / 5%) 0%, 
    rgb(139 92 246 / 5%) 100%
  );
}
</style>
