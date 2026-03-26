<template>
  <div class="h-[600px] flex flex-col">
    <!-- Header -->
    <div class="flex items-center justify-between mb-4 flex-shrink-0">
      <div>
        <h2 class="text-xl font-bold text-white">
          Operation History
        </h2>
        <p class="text-sm text-white/80">
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
      class="flex-1 flex flex-col items-center justify-center text-white/50"
    >
      <div class="p-6 rounded-full glass-surface mb-4">
        <SIcon
          name="History"
          size="w-8 h-8"
          class="opacity-20"
        />
      </div>
      <p class="text-lg font-medium text-white/80">
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
            :data-index="virtualRow.index"
            class="pb-3"
          >
            <Card 
              variant="glass" 
              hover 
              class="p-4 group transition-colors duration-300 border-l-4"
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
                  <SIcon
                    :name="getOperationIcon(entries[virtualRow.index].operation)"
                    size="w-5 h-5"
                  />
                </div>

                <!-- Content -->
                <div class="flex-1 min-w-0">
                  <div class="flex justify-between items-start mb-2">
                    <div>
                      <h3 class="font-bold text-white">
                        {{ getOperationLabel(entries[virtualRow.index].operation) }}
                      </h3>
                      <div class="flex items-center gap-3 text-xs text-white/80 mt-1">
                        <span class="flex items-center gap-1"><SIcon
                          name="Clock"
                          size="w-3 h-3"
                        /> {{ formatRelativeTime(entries[virtualRow.index].timestamp) }}</span>
                        <span class="flex items-center gap-1"><SIcon
                          name="User"
                          size="w-3 h-3"
                        /> {{ entries[virtualRow.index].actor }}</span>
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
                    class="flex items-center gap-2 p-2 rounded bg-white/5/50 border border-white/10 mb-2"
                  >
                    <code class="text-xs text-accent-danger bg-accent-danger/10 px-1.5 py-0.5 rounded">{{ entries[virtualRow.index].from_config }}</code>
                    <SIcon
                      name="ArrowRight"
                      size="w-3 h-3"
                      class="text-white/50"
                    />
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
                      class="text-xs font-mono p-1.5 rounded bg-white/5/30 border border-white/5 grid grid-cols-[auto_1fr] gap-2"
                    >
                      <span class="font-bold text-white">{{ change.key }}</span>
                      <div class="flex items-center gap-1 truncate text-white/50">
                        <span class="truncate">{{ change.old_value || '_' }}</span>
                        <span>→</span>
                        <span class="text-white truncate">{{ change.new_value || '_' }}</span>
                      </div>
                    </div>
                    <button
                      v-if="entries[virtualRow.index].changes.length > 3"
                      class="text-[10px] text-accent-primary hover:underline"
                    >
                      + {{ entries[virtualRow.index].changes.length - 3 }} more changes
                    </button>
                  </div>
                    
                  <div class="mt-2 pt-2 border-t border-white/5 text-[10px] text-white/50 font-mono">
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
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
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

const measureElement = (element: unknown) => {
  rowVirtualizer.value.measureElement(element instanceof Element ? element : null)
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
  'switch': 'GitBranch',
  'init': 'CheckCircle',
  'update': 'FileEdit',
  'delete': 'Trash2',
  'validate': 'CheckCircle',
  'clean': 'RefreshCw',
  'import': 'ArrowRight',
  'export': 'ArrowRight'
}[op] || 'GitBranch')

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
