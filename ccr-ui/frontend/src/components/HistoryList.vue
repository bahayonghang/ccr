<template>
  <div class="h-[600px] flex flex-col">
    <!-- Header -->
    <div class="flex items-center justify-between mb-4 flex-shrink-0">
      <div>
        <h2 class="text-xl font-bold text-text-primary">
          Operation History
        </h2>
        <p class="text-sm text-text-secondary">
          {{ entries.length }} records found
        </p>
      </div>
    </div>

    <!-- Loading State -->
    <div
      v-if="loading"
      class="flex-1 flex items-center justify-center"
    >
      <Spinner
        size="xl"
        class="text-accent-primary"
      />
    </div>

    <!-- Empty State -->
    <div
      v-else-if="entries.length === 0"
      class="flex-1 flex flex-col items-center justify-center text-text-muted"
    >
      <div class="p-6 rounded-full bg-bg-surface mb-4">
        <History class="w-8 h-8 opacity-20" />
      </div>
      <p class="text-lg font-medium text-text-secondary">
        No history records
      </p>
      <p class="text-sm">
        Operations will appear here.
      </p>
    </div>

    <!-- Virtual List -->
    <div
      v-else
      ref="parentRef"
      class="flex-1 overflow-auto pr-2 scrollbar-thin"
    >
      <div
        :style="{
          height: `${rowVirtualizer.getTotalSize()}px`,
          width: '100%',
          position: 'relative',
        }"
      >
        <div
          v-for="virtualRow in rowVirtualizer.getVirtualItems()"
          :key="entries[virtualRow.index].id"
          class="absolute top-0 left-0 w-full"
          :style="{ transform: `translateY(${virtualRow.start}px)` }"
        >
          <div
            :ref="(el) => measureElement(el)"
            class="pb-3"
          >
            <Card 
              variant="glass" 
              hover 
              class="p-4 group transition-all duration-300 border-l-4"
              :style="{ borderLeftColor: getOperationColor(entries[virtualRow.index].operation) }"
            >
              <!-- Timeline Line -->
              <div
                v-if="virtualRow.index < entries.length - 1"
                class="absolute left-8 top-full w-px h-4 bg-border-subtle -z-10"
              />

              <div class="flex gap-4">
                <!-- Icon -->
                <div 
                  class="w-10 h-10 rounded-lg flex items-center justify-center shrink-0 border"
                  :style="{ 
                    borderColor: getOperationColor(entries[virtualRow.index].operation) + '40',
                    backgroundColor: getOperationColor(entries[virtualRow.index].operation) + '15',
                    color: getOperationColor(entries[virtualRow.index].operation)
                  }"
                >
                  <component
                    :is="getOperationIcon(entries[virtualRow.index].operation)"
                    class="w-5 h-5"
                  />
                </div>

                <!-- Content -->
                <div class="flex-1 min-w-0">
                  <div class="flex justify-between items-start mb-2">
                    <div>
                      <h3 class="font-bold text-text-primary">
                        {{ getOperationLabel(entries[virtualRow.index].operation) }}
                      </h3>
                      <div class="flex items-center gap-3 text-xs text-text-secondary mt-1">
                        <span class="flex items-center gap-1"><Clock class="w-3 h-3" /> {{ formatRelativeTime(entries[virtualRow.index].timestamp) }}</span>
                        <span class="flex items-center gap-1"><User class="w-3 h-3" /> {{ entries[virtualRow.index].actor }}</span>
                      </div>
                    </div>
                    <span 
                      class="px-2 py-0.5 rounded text-[10px] uppercase font-bold tracking-wider"
                      :style="{ 
                        backgroundColor: getOperationColor(entries[virtualRow.index].operation) + '20',
                        color: getOperationColor(entries[virtualRow.index].operation)
                      }"
                    >
                      {{ entries[virtualRow.index].operation }}
                    </span>
                  </div>

                  <!-- Config Change -->
                  <div
                    v-if="entries[virtualRow.index].from_config && entries[virtualRow.index].to_config"
                    class="flex items-center gap-2 p-2 rounded bg-bg-surface/50 border border-border-default mb-2"
                  >
                    <code class="text-xs text-accent-danger bg-accent-danger/10 px-1.5 py-0.5 rounded">{{ entries[virtualRow.index].from_config }}</code>
                    <ArrowRight class="w-3 h-3 text-text-muted" />
                    <code class="text-xs text-accent-success bg-accent-success/10 px-1.5 py-0.5 rounded">{{ entries[virtualRow.index].to_config }}</code>
                  </div>

                  <!-- Env Changes -->
                  <div
                    v-if="entries[virtualRow.index].changes?.length"
                    class="space-y-1 my-2"
                  >
                    <div
                      v-for="change in entries[virtualRow.index].changes.slice(0, 3)"
                      :key="change.key"
                      class="text-xs font-mono p-1.5 rounded bg-bg-surface/30 border border-border-subtle grid grid-cols-[auto_1fr] gap-2"
                    >
                      <span class="font-bold text-text-primary">{{ change.key }}</span>
                      <div class="flex items-center gap-1 truncate text-text-muted">
                        <span class="truncate">{{ change.old_value || '_' }}</span>
                        <span>→</span>
                        <span class="text-text-primary truncate">{{ change.new_value || '_' }}</span>
                      </div>
                    </div>
                    <button
                      v-if="entries[virtualRow.index].changes.length > 3"
                      class="text-[10px] text-accent-primary hover:underline"
                    >
                      + {{ entries[virtualRow.index].changes.length - 3 }} more changes
                    </button>
                  </div>
                    
                  <div class="mt-2 pt-2 border-t border-border-subtle text-[10px] text-text-muted font-mono">
                    ID: {{ entries[virtualRow.index].id }}
                  </div>
                </div>
              </div>
            </Card>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ArrowRight, Clock, User, History, GitBranch, CheckCircle, RefreshCw, FileEdit, Trash2 } from 'lucide-vue-next'
import { useVirtualizer } from '@tanstack/vue-virtual'
import type { ComponentPublicInstance } from 'vue'
import type { HistoryEntry } from '@/types'
import { formatRelativeTime } from '@/utils/codexHelpers'
import Spinner from '@/components/ui/Spinner.vue'
import Card from '@/components/ui/Card.vue'

const props = withDefaults(defineProps<{
  entries: HistoryEntry[]
  loading?: boolean
}>(), { loading: false })

const parentRef = ref<HTMLElement | null>(null)

const rowVirtualizer = useVirtualizer(computed(() => ({
  count: props.entries.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 160,
  overscan: 5,
})))

const measureElement = (el: Element | ComponentPublicInstance | null) => {
  if (el instanceof Element) rowVirtualizer.value.measureElement(el)
}

const getOperationLabel = (op: string) => ({
  'switch': 'Switched Config',
  'init': 'Initialized',
  'update': 'Updated Config',
  'delete': 'Deleted Config',
  'validate': 'Validation Run',
  'clean': 'Cleaned Backups',
  'import': 'Imported',
  'export': 'Exported'
}[op] || op)

const getOperationIcon = (op: string) => ({
  'switch': GitBranch,
  'init': CheckCircle,
  'update': FileEdit,
  'delete': Trash2,
  'validate': CheckCircle,
  'clean': RefreshCw,
  'import': ArrowRight,
  'export': ArrowRight
}[op] || GitBranch)

const getOperationColor = (op: string) => ({
  'switch': '#8b5cf6',
  'init': '#10b981',
  'update': '#3b82f6',
  'delete': '#ef4444',
  'validate': '#f59e0b',
  'clean': '#6366f1',
  'import': '#06b6d4',
  'export': '#ec4899'
}[op] || '#64748b')
</script>
