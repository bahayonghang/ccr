<template>
  <div class="space-y-4">
    <div class="flex items-center gap-2">
      <input
        :value="logModelFilter"
        :placeholder="$t('usage.dashboard.logs.filterPlaceholder')"
        class="toolbar-select max-w-xs flex-1"
        @input="updateLogModelFilter(($event.target as HTMLInputElement).value)"
        @keyup.enter="loadLogs('reset')"
      >
      <button
        class="rounded-lg bg-accent-primary/20 px-3 py-1.5 text-xs font-medium text-accent-primary transition-colors hover:bg-accent-primary/30"
        @click="loadLogs('reset')"
      >
        {{ $t('usage.dashboard.logs.search') }}
      </button>
    </div>

    <div class="glass-panel overflow-x-auto rounded-xl">
      <div class="grid grid-cols-[2fr,1fr,2fr,1fr,1fr,1fr] border-b border-border-subtle text-left text-sm text-text-muted">
        <div class="p-3">
          {{ $t('usage.dashboard.table.time') }}
        </div>
        <div class="p-3">
          {{ $t('usage.dashboard.table.platform') }}
        </div>
        <div class="p-3">
          {{ $t('usage.dashboard.table.model') }}
        </div>
        <div class="p-3 text-right">
          {{ $t('usage.dashboard.table.input') }}
        </div>
        <div class="p-3 text-right">
          {{ $t('usage.dashboard.table.output') }}
        </div>
        <div class="p-3 text-right">
          {{ $t('usage.dashboard.table.cost') }}
        </div>
      </div>
      <div
        :ref="setLogsScrollRef"
        class="max-h-[420px] overflow-auto"
      >
        <div
          class="relative"
          :style="{ height: `${logsVirtualizer.getTotalSize()}px` }"
        >
          <div
            v-for="virtualRow in logsVirtualizer.getVirtualItems()"
            :key="logsRecords[virtualRow.index]?.id ?? virtualRow.index"
            class="absolute left-0 right-0 grid grid-cols-[2fr,1fr,2fr,1fr,1fr,1fr] border-b border-border-subtle/50 text-sm transition-colors hover:bg-accent-primary/5"
            :style="{ transform: `translateY(${virtualRow.start}px)` }"
          >
            <div class="whitespace-nowrap p-3 text-xs text-text-muted">
              {{ new Date(logsRecords[virtualRow.index].recorded_at).toLocaleString() }}
            </div>
            <div class="p-3 text-text-secondary">
              {{ logsRecords[virtualRow.index].platform }}
            </div>
            <div class="truncate p-3 font-medium text-text-primary">
              {{ logsRecords[virtualRow.index].model || '-' }}
            </div>
            <div class="p-3 text-right text-text-secondary">
              {{ formatTokens(logsRecords[virtualRow.index].input_tokens) }}
            </div>
            <div class="p-3 text-right text-text-secondary">
              {{ formatTokens(logsRecords[virtualRow.index].output_tokens) }}
            </div>
            <div class="p-3 text-right text-text-secondary">
              {{ formatCost(logsRecords[virtualRow.index].cost_usd) }}
            </div>
          </div>
        </div>
      </div>
      <div
        v-if="!logsRecords.length"
        class="p-6 text-center text-sm text-text-muted"
      >
        {{ $t('usage.dashboard.logs.noLogs') }}
      </div>
    </div>

    <div
      v-if="showPager"
      class="flex items-center justify-center gap-2"
    >
      <button
        class="glass-surface rounded px-3 py-1 text-xs text-text-secondary transition-colors hover:text-text-primary disabled:opacity-40"
        :disabled="!canPrevLogs"
        @click="loadLogs('prev')"
      >
        {{ $t('usage.dashboard.logs.prev') }}
      </button>
      <span
        v-if="hasLogsTotal"
        class="text-xs text-text-muted"
      >{{ logsPage }} / {{ logsTotalPages }}</span>
      <span
        v-else
        class="text-xs text-text-muted"
      >{{ logsPage }}</span>
      <button
        class="glass-surface rounded px-3 py-1 text-xs text-text-secondary transition-colors hover:text-text-primary disabled:opacity-40"
        :disabled="!canNextLogs"
        @click="loadLogs('next')"
      >
        {{ $t('usage.dashboard.logs.next') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ComponentPublicInstance } from 'vue'
import type { UsageRecordV2 } from '@/types/usage'

type LogsDirection = 'reset' | 'next' | 'prev' | 'same'

type VirtualRow = {
  index: number
  start: number
}

type LogsVirtualizer = {
  getTotalSize: () => number
  getVirtualItems: () => VirtualRow[]
}

interface Props {
  logModelFilter: string
  logsRecords: UsageRecordV2[]
  logsVirtualizer: LogsVirtualizer
  logsPage: number
  logsTotalPages: number
  canPrevLogs: boolean
  canNextLogs: boolean
  hasLogsTotal: boolean
  showPager: boolean
  formatCost: (value: number) => string
  formatTokens: (value: number) => string
  loadLogs: (direction?: LogsDirection) => void
  setLogsScrollRef: (element: Element | ComponentPublicInstance | null) => void
  updateLogModelFilter: (value: string) => void
}

defineProps<Props>()
</script>
