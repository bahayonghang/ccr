<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { computed, ref, watch, nextTick } from 'vue'
import BaseModal from '@/components/common/BaseModal.vue'
import type { CheckinFlowPhase, CheckinLogEntry } from '@/types/checkin'

interface Props {
  isOpen: boolean
  total: number
  current: number
  currentAccountName: string
  logs: CheckinLogEntry[]
  phase: CheckinFlowPhase
  recoveryMessage?: string | null
  recoveryProviderName?: string | null
}

const props = defineProps<Props>()
const emit = defineEmits<{
  close: []
}>()

const logContainerRef = ref<HTMLElement | null>(null)

const radius = 42
const circumference = 2 * Math.PI * radius
const isRecovering = computed(() => props.phase === 'recovering')
const isFinished = computed(() => props.phase === 'finished')

const modalTitle = computed(() => {
  if (isRecovering.value) return '正在自动处理 WAF'
  return isFinished.value ? '签到完成' : '正在执行签到'
})

const progressPercent = computed(() => {
  if (props.total === 0) return 0
  return Math.round((props.current / props.total) * 100)
})

const progressOffset = computed(() => {
  const progress = props.total === 0 ? 0 : props.current / props.total
  return circumference * (1 - progress)
})

const getLogTextClass = (status: CheckinLogEntry['status']) => {
  switch (status) {
    case 'success':
      return 'text-accent-success'
    case 'already_checked_in':
      return 'text-accent-warning'
    case 'failed':
      return 'text-accent-danger'
    case 'processing':
      return 'text-accent-info'
    default:
      return 'text-text-secondary'
  }
}

const getRecoveryBadgeLabel = (log: CheckinLogEntry) => {
  if (!log.wafRecoveryAttempted) return null
  if (log.wafRecovered) {
    return log.status === 'already_checked_in' ? '自动补救后完成' : '自动补救后成功'
  }
  return '自动补救失败'
}

watch(
  () => props.logs.length,
  async () => {
    await nextTick()
    if (logContainerRef.value) {
      logContainerRef.value.scrollTop = logContainerRef.value.scrollHeight
    }
  }
)
</script>

<template>
  <BaseModal
    :model-value="isOpen"
    :title="modalTitle"
    :close-on-backdrop="isFinished"
    :close-on-escape="isFinished"
    :show-close="false"
    :persistent="!isFinished"
    surface="solid"
    size="md"
  >
    <div class="space-y-6 py-4">
      <div class="space-y-3 text-center">
        <div class="relative inline-flex h-24 w-24 items-center justify-center">
          <svg class="h-24 w-24 -rotate-90 transform">
            <circle
              cx="48"
              cy="48"
              r="42"
              stroke="currentColor"
              stroke-width="6"
              fill="none"
              class="text-border-default"
            />
            <circle
              cx="48"
              cy="48"
              r="42"
              stroke="currentColor"
              stroke-width="6"
              fill="none"
              class="text-accent-primary transition-[stroke-dashoffset,stroke-dasharray] duration-300"
              :stroke-dasharray="circumference"
              :stroke-dashoffset="progressOffset"
              stroke-linecap="round"
            />
          </svg>
          <span class="absolute text-2xl font-bold text-text-primary">
            <template v-if="isFinished">
              <SIcon
                name="CheckCircle"
                size="h-10 w-10"
                class="text-accent-success"
              />
            </template>
            <template v-else-if="isRecovering">
              <SIcon
                name="Loader2"
                size="h-10 w-10"
                class="animate-spin text-accent-info"
              />
            </template>
            <template v-else>
              {{ progressPercent }}%
            </template>
          </span>
        </div>

        <div class="space-y-1">
          <p class="text-sm text-text-secondary">
            {{ current }} / {{ total }} 个账号
          </p>
          <p
            v-if="currentAccountName && !isFinished && !isRecovering"
            class="text-sm font-medium text-accent-primary"
          >
            正在签到：{{ currentAccountName }}
          </p>
          <p
            v-else-if="isRecovering && recoveryMessage"
            class="text-sm font-medium text-accent-info"
          >
            {{ recoveryMessage }}
          </p>
          <p
            v-else-if="isFinished"
            class="text-sm font-medium text-accent-success"
          >
            全部任务执行完毕
          </p>
          <p
            v-if="isRecovering && recoveryProviderName"
            class="text-xs text-text-secondary"
          >
            当前提供商：{{ recoveryProviderName }}
          </p>
        </div>
      </div>

      <div class="space-y-2">
        <h4 class="flex items-center gap-2 text-sm font-medium text-text-secondary">
          <SIcon
            name="FileText"
            size="h-4 w-4"
          />
          签到日志
        </h4>
        <div
          ref="logContainerRef"
          class="h-48 space-y-1.5 overflow-y-auto rounded-lg border border-border-default bg-bg-base p-3 shadow-inner"
        >
          <div
            v-for="log in logs"
            :key="`${log.accountId}-${log.timestamp}`"
            class="flex items-start gap-2 text-sm"
          >
            <span class="mt-0.5 flex-shrink-0">
              <SIcon
                v-if="log.status === 'processing'"
                name="Loader2"
                size="h-4 w-4"
                class="animate-spin text-accent-info"
              />
              <SIcon
                v-else-if="log.status === 'success'"
                name="CheckCircle"
                size="h-4 w-4"
                class="text-accent-success"
              />
              <SIcon
                v-else-if="log.status === 'already_checked_in'"
                name="Clock"
                size="h-4 w-4"
                class="text-accent-warning"
              />
              <SIcon
                v-else-if="log.status === 'failed'"
                name="XCircle"
                size="h-4 w-4"
                class="text-accent-danger"
              />
              <SIcon
                v-else
                name="Circle"
                size="h-4 w-4"
                class="text-text-muted"
              />
            </span>
            <div class="min-w-0 flex-1">
              <span
                class="font-medium"
                :class="getLogTextClass(log.status)"
              >
                {{ log.accountName }}
              </span>
              <span class="ml-1 text-text-muted">
                ({{ log.providerName }})
              </span>
              <span
                v-if="getRecoveryBadgeLabel(log)"
                class="ml-2 inline-flex rounded-full px-2 py-0.5 text-[11px] font-medium"
                :class="log.wafRecovered
                  ? 'bg-sky-100 text-sky-700 dark:bg-sky-900/40 dark:text-sky-200'
                  : 'bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-200'"
              >
                {{ getRecoveryBadgeLabel(log) }}
              </span>
              <p
                v-if="log.message"
                class="mt-0.5 break-all text-xs"
                :class="log.status === 'failed' ? 'text-accent-danger' : 'text-text-secondary'"
              >
                {{ log.message }}
              </p>
              <p
                v-if="log.wafRecoveryAttempted && log.wafRecovered === false && log.wafRecoveryError"
                class="mt-0.5 break-all text-xs text-amber-700 dark:text-amber-300"
              >
                {{ log.wafRecoveryError }}
              </p>
            </div>
          </div>
          <div
            v-if="logs.length === 0"
            class="flex h-full items-center justify-center text-sm text-text-muted"
          >
            等待开始签到...
          </div>
        </div>
      </div>

      <div
        v-if="isFinished"
        class="pt-2"
      >
        <button
          class="flex w-full items-center justify-center gap-2 rounded-lg bg-accent-primary px-4 py-2.5 font-medium text-white transition-colors hover:bg-accent-primary/90"
          @click="emit('close')"
        >
          <SIcon
            name="CheckCircle"
            size="h-5 w-5"
          />
          确定
        </button>
      </div>
    </div>
  </BaseModal>
</template>
