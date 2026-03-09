<template>
  <div class="min-h-screen bg-bg-base">
    <Navbar />
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <!-- Header -->
      <div class="flex items-center justify-between mb-8">
        <div class="flex items-center gap-4">
          <Breadcrumb :items="breadcrumbs" />
        </div>
        <div class="flex items-center gap-3">
          <!-- Connection Status -->
          <div 
            class="flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-medium"
            :class="isConnected 
              ? 'bg-accent-success/10 text-accent-success' 
              : 'bg-accent-danger/10 text-accent-danger'"
          >
            <span
              class="w-2 h-2 rounded-full"
              :class="isConnected ? 'bg-accent-success animate-pulse' : 'bg-accent-danger'"
            />
            {{ isConnected ? $t('monitoring.connected') : $t('monitoring.disconnected') }}
          </div>
          <button
            class="text-xs px-3 py-1.5 rounded-lg bg-bg-surface hover:bg-accent-secondary/10 text-text-secondary hover:text-accent-secondary transition-colors flex items-center gap-1.5"
            @click="clearLogs"
          >
            <Trash2 class="w-3.5 h-3.5" />
            {{ $t('monitoring.clearLogs') }}
          </button>
        </div>
      </div>

      <!-- Stats Cards -->
      <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
        <div class="glass-effect rounded-2xl p-4 border border-white/10">
          <div class="flex items-center gap-3">
            <div class="p-2 rounded-xl bg-accent-secondary/10">
              <MessageSquare class="w-5 h-5 text-accent-secondary" />
            </div>
            <div>
              <p class="text-xs text-text-muted">
                {{ $t('monitoring.totalLogs') }}
              </p>
              <p class="text-lg font-bold text-text-primary">
                {{ logs.length }}
              </p>
            </div>
          </div>
        </div>

        <div class="glass-effect rounded-2xl p-4 border border-white/10">
          <div class="flex items-center gap-3">
            <div class="p-2 rounded-xl bg-accent-success/10">
              <ArrowUpCircle class="w-5 h-5 text-accent-success" />
            </div>
            <div>
              <p class="text-xs text-text-muted">
                {{ $t('monitoring.inputTokens') }}
              </p>
              <p class="text-lg font-bold text-text-primary">
                {{ formatNumber(tokenStats?.input_tokens || 0) }}
              </p>
            </div>
          </div>
        </div>

        <div class="glass-effect rounded-2xl p-4 border border-white/10">
          <div class="flex items-center gap-3">
            <div class="p-2 rounded-xl bg-accent-primary/10">
              <ArrowDownCircle class="w-5 h-5 text-accent-primary" />
            </div>
            <div>
              <p class="text-xs text-text-muted">
                {{ $t('monitoring.outputTokens') }}
              </p>
              <p class="text-lg font-bold text-text-primary">
                {{ formatNumber(tokenStats?.output_tokens || 0) }}
              </p>
            </div>
          </div>
        </div>

        <div class="glass-effect rounded-2xl p-4 border border-white/10">
          <div class="flex items-center gap-3">
            <div class="p-2 rounded-xl bg-accent-warning/10">
              <DollarSign class="w-5 h-5 text-accent-warning" />
            </div>
            <div>
              <p class="text-xs text-text-muted">
                {{ $t('monitoring.estimatedCost') }}
              </p>
              <p class="text-lg font-bold text-text-primary">
                ${{ formatCost(tokenStats?.estimated_cost_cents || 0) }}
              </p>
            </div>
          </div>
        </div>
      </div>

      <!-- Logs Panel -->
      <div class="glass-effect rounded-3xl p-6 border border-white/20">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-base font-bold text-text-primary flex items-center gap-2">
            <Terminal class="w-4 h-4" />
            {{ $t('monitoring.realTimeLogs') }}
          </h3>
          <div class="flex items-center gap-2">
            <select
              v-model="filterLevel"
              class="text-xs px-2 py-1 rounded-lg bg-bg-surface text-text-secondary border border-border-default focus:outline-none focus:border-accent-secondary"
            >
              <option value="all">
                {{ $t('monitoring.allLevels') }}
              </option>
              <option value="error">
                Error
              </option>
              <option value="warn">
                Warning
              </option>
              <option value="info">
                Info
              </option>
              <option value="debug">
                Debug
              </option>
            </select>
          </div>
        </div>

        <!-- Log List -->
        <div
          ref="logContainer"
          class="h-96 overflow-y-auto space-y-1 font-mono text-xs"
        >
          <div
            v-for="log in filteredLogs"
            :key="log.id"
            class="flex items-start gap-2 px-3 py-2 rounded-lg hover:bg-bg-surface/50 transition-colors"
          >
            <span class="text-text-muted flex-shrink-0">{{ formatTime(log.timestamp) }}</span>
            <span
              class="px-1.5 py-0.5 rounded text-[10px] font-bold uppercase flex-shrink-0"
              :class="getLevelClass(log.level)"
            >
              {{ log.level }}
            </span>
            <span class="px-1.5 py-0.5 rounded bg-bg-surface text-text-muted flex-shrink-0">
              {{ log.source }}
            </span>
            <span class="text-text-secondary break-all">{{ log.message }}</span>
          </div>

          <!-- Empty state -->
          <div
            v-if="logs.length === 0"
            class="flex flex-col items-center justify-center h-full text-text-muted"
          >
            <Monitor class="w-12 h-12 opacity-30 mb-2" />
            <p>{{ $t('monitoring.noLogs') }}</p>
            <p class="text-xs mt-1">
              {{ $t('monitoring.waitingForLogs') }}
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  MessageSquare,
  ArrowUpCircle,
  ArrowDownCircle,
  DollarSign,
  Terminal,
  Trash2,
  Monitor
} from 'lucide-vue-next'
import Navbar from '@/components/Navbar.vue'
import { Breadcrumb } from '@/components/ui'
import {
  useMonitoringFeed,
  type MonitoringEntry,
  type MonitoringLevel,
} from '@/composables/useMonitoringFeed'

const { t } = useI18n({ useScope: 'global' })

const { isConnected, logs, tokenStats, clearLogs } = useMonitoringFeed()

// Filter state
const filterLevel = ref<'all' | MonitoringLevel>('all')
const logContainer = ref<HTMLElement | null>(null)

// Breadcrumbs
const breadcrumbs = computed(() => [
  { label: t('common.home'), path: '/' },
  { label: t('monitoring.title'), path: '/monitoring' }
])

// Filtered logs
const filteredLogs = computed(() => {
  if (filterLevel.value === 'all') return logs.value
  return logs.value.filter((log: MonitoringEntry) => log.level === filterLevel.value)
})

// Auto-scroll to bottom on new logs
watch(logs, async () => {
  await nextTick()
  if (logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight
  }
}, { deep: true })

// Helpers
const formatNumber = (num: number) => {
  if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`
  if (num >= 1000) return `${(num / 1000).toFixed(1)}K`
  return num.toString()
}

const formatCost = (cents: number) => {
  return (cents / 100).toFixed(4)
}

const formatTime = (timestamp: string) => {
  const date = new Date(timestamp)
  return date.toLocaleTimeString('zh-CN', { 
    hour: '2-digit', 
    minute: '2-digit', 
    second: '2-digit',
    hour12: false 
  })
}

const getLevelClass = (level: string) => {
  switch (level) {
    case 'error': return 'bg-accent-danger/20 text-accent-danger'
    case 'warn': return 'bg-accent-warning/20 text-accent-warning'
    case 'info': return 'bg-accent-primary/20 text-accent-primary'
    case 'debug': return 'bg-text-muted/20 text-text-muted'
    default: return 'bg-bg-surface text-text-secondary'
  }
}
</script>
