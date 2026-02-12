<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div
        v-if="modelValue"
        class="fixed inset-0 z-50 flex items-center justify-center p-4"
        @click.self="close"
      >
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" />

        <!-- Modal Content -->
        <div class="modal-content relative w-full max-w-lg max-h-[80vh] flex flex-col">
          <!-- Header -->
          <div class="modal-header">
            <div class="flex items-center gap-2">
              <div class="header-icon">
                <ScrollText class="w-5 h-5" />
              </div>
              <h2 class="text-lg font-bold text-text-primary">
                {{ $t('skills.operationLog') }}
              </h2>
            </div>
            <button
              class="p-2 rounded-lg text-text-muted hover:text-text-primary
                     hover:bg-bg-surface transition-colors"
              @click="close"
            >
              <X class="w-5 h-5" />
            </button>
          </div>

          <!-- Body (Scrollable) -->
          <div class="modal-body flex-1 overflow-y-auto">
            <!-- Loading -->
            <div
              v-if="isLoading"
              class="flex flex-col items-center justify-center py-16"
            >
              <Loader2 class="w-8 h-8 animate-spin text-accent-primary" />
              <p class="text-text-secondary text-sm mt-3">
                {{ $t('common.loading') }}
              </p>
            </div>

            <!-- Error -->
            <div
              v-else-if="loadError"
              class="flex flex-col items-center justify-center py-16"
            >
              <AlertCircle
                class="w-8 h-8"
                style="color: rgb(var(--color-danger-rgb));"
              />
              <p
                class="text-sm mt-2"
                style="color: rgb(var(--color-danger-rgb));"
              >
                {{ loadError }}
              </p>
              <button
                class="btn-retry mt-4"
                @click="fetchLogs"
              >
                {{ $t('common.retry') }}
              </button>
            </div>

            <!-- Empty State -->
            <div
              v-else-if="filteredLogs.length === 0"
              class="flex flex-col items-center justify-center py-16"
            >
              <Inbox class="w-12 h-12 text-text-muted opacity-40" />
              <p class="text-sm text-text-muted mt-3">
                {{ $t('skills.noOperationLog') }}
              </p>
            </div>

            <!-- Log List -->
            <div
              v-else
              class="log-list"
            >
              <div
                v-for="log in filteredLogs"
                :key="log.id"
                class="log-item"
              >
                <!-- Action Icon -->
                <div
                  class="log-icon"
                  :class="getActionClass(log)"
                >
                  <component
                    :is="getActionIcon(log)"
                    class="w-4 h-4"
                  />
                </div>

                <!-- Content -->
                <div class="log-content">
                  <p class="log-message">
                    <span class="log-action-label">
                      {{ getActionLabel(log) }}
                    </span>
                    <span class="log-skill-name">
                      {{ getSkillName(log) }}
                    </span>
                    <span class="log-preposition">
                      {{ getPreposition(log) }}
                    </span>
                    <span class="log-agent-name">
                      {{ getAgentName(log) }}
                    </span>
                  </p>
                  <p class="log-time">
                    {{ formatTime(log.timestamp) }}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  X,
  ScrollText,
  Loader2,
  AlertCircle,
  Inbox,
  Trash2,
  Download,
  Activity
} from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import api from '@/api/core'

const { t } = useI18n()

interface LogMetadata {
  action?: string
  skill?: string
  package?: string
  agent?: string
  skill_dir?: string
}

interface LogEntry {
  id: string
  timestamp: string
  level: string
  source: string
  message: string
  metadata?: LogMetadata | null
}

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

// State
const logs = ref<LogEntry[]>([])
const isLoading = ref(false)
const loadError = ref<string | null>(null)

// Filter only SkillHub logs
const filteredLogs = computed(() =>
  logs.value.filter(log => log.source === 'SkillHub')
)

// Watch for modal open
watch(() => props.modelValue, (isOpen) => {
  if (isOpen) {
    fetchLogs()
  }
})

async function fetchLogs() {
  isLoading.value = true
  loadError.value = null

  try {
    const response = await api.get<{
      data: { logs: LogEntry[]; total: number }
    }>('/logs', {
      params: { limit: 100 }
    })
    logs.value = response.data.data.logs
  } catch (err: any) {
    loadError.value = err.message || 'Failed to fetch logs'
  } finally {
    isLoading.value = false
  }
}

function close() {
  emit('update:modelValue', false)
}

// Helper: extract metadata
function getMeta(log: LogEntry): LogMetadata {
  return log.metadata || {}
}

function getActionIcon(log: LogEntry) {
  const action = getMeta(log).action
  if (action === 'remove') return Trash2
  if (action === 'install') return Download
  return Activity
}

function getActionClass(log: LogEntry): string {
  const action = getMeta(log).action
  if (action === 'remove') return 'log-icon--remove'
  if (action === 'install') return 'log-icon--install'
  return 'log-icon--default'
}

function getActionLabel(log: LogEntry): string {
  const action = getMeta(log).action
  if (action === 'remove') return t('skills.logActionRemove')
  if (action === 'install') return t('skills.logActionInstall')
  return log.message
}

function getSkillName(log: LogEntry): string {
  const meta = getMeta(log)
  return meta.skill || meta.package || ''
}

function getPreposition(log: LogEntry): string {
  const action = getMeta(log).action
  if (action === 'remove') return t('skills.logFrom')
  if (action === 'install') return t('skills.logTo')
  return ''
}

function getAgentName(log: LogEntry): string {
  return getMeta(log).agent || ''
}

function formatTime(timestamp: string): string {
  try {
    const date = new Date(timestamp)
    return date.toLocaleString()
  } catch {
    return timestamp
  }
}
</script>

<style scoped>
.modal-content {
  @apply bg-bg-base border border-border-default rounded-2xl
         shadow-2xl overflow-hidden;
}

.modal-header {
  @apply flex items-center justify-between p-4
         border-b border-border-subtle shrink-0;
}

.header-icon {
  @apply flex items-center justify-center w-9 h-9 rounded-xl;

  background: rgb(var(--color-accent-primary-rgb) / 12%);
  color: rgb(var(--color-accent-primary-rgb));
}

.modal-body {
  @apply p-4;
}

/* Log List */
.log-list {
  @apply flex flex-col gap-1;
}

.log-item {
  @apply flex items-start gap-3 p-3 rounded-xl
         transition-colors duration-150;
}

.log-item:hover {
  background: rgb(var(--color-bg-surface-rgb) / 50%);
}

/* Action Icon */
.log-icon {
  @apply flex items-center justify-center w-8 h-8 rounded-lg shrink-0 mt-0.5;
}

.log-icon--remove {
  background: rgb(var(--color-danger-rgb) / 12%);
  color: rgb(var(--color-danger-rgb));
}

.log-icon--install {
  background: rgb(var(--color-success-rgb) / 12%);
  color: rgb(var(--color-success-rgb));
}

.log-icon--default {
  background: rgb(var(--color-accent-primary-rgb) / 12%);
  color: rgb(var(--color-accent-primary-rgb));
}

/* Log Content */
.log-content {
  @apply flex flex-col gap-0.5 min-w-0;
}

.log-message {
  @apply text-sm text-text-primary leading-snug;
}

.log-action-label {
  @apply font-semibold;
}

.log-skill-name {
  @apply font-bold text-accent-primary mx-1;
}

.log-preposition {
  @apply text-text-muted;
}

.log-agent-name {
  @apply font-medium text-text-secondary;
}

.log-time {
  @apply text-xs;

  color: rgb(var(--color-text-muted-rgb) / 60%);
}

/* Buttons */
.btn-retry {
  @apply px-4 py-2 rounded-xl text-sm font-semibold
         text-white transition-colors;

  background: rgb(var(--color-accent-primary-rgb));
}

.btn-retry:hover {
  background: rgb(var(--color-accent-primary-rgb) / 90%);
}

/* Modal Transition */
.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: opacity 0.2s ease;
}

.modal-fade-enter-active .modal-content,
.modal-fade-leave-active .modal-content {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}

.modal-fade-enter-from .modal-content,
.modal-fade-leave-to .modal-content {
  transform: scale(0.95);
  opacity: 0;
}
</style>
