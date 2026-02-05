<script setup lang="ts">
import { computed, ref, watch, nextTick } from 'vue'
import {
  FileText,
  CheckCircle,
  XCircle,
  Clock,
  Loader2,
  Circle,
} from 'lucide-vue-next'
import BaseModal from '@/components/common/BaseModal.vue'
import type { CheckinLogEntry } from '@/types/checkin'

// Props interface
interface Props {
  /** 是否显示 */
  isOpen: boolean
  /** 总账号数 */
  total: number
  /** 当前进度 */
  current: number
  /** 当前处理的账号名 */
  currentAccountName: string
  /** 签到日志 */
  logs: CheckinLogEntry[]
  /** 是否完成 */
  isFinished?: boolean
}

const props = defineProps<Props>()
const emit = defineEmits(['close'])

// 日志容器引用，用于自动滚动
const logContainerRef = ref<HTMLElement | null>(null)

// 圆形进度条计算
const radius = 42
const circumference = 2 * Math.PI * radius

const progressPercent = computed(() => {
  if (props.total === 0) return 0
  return Math.round((props.current / props.total) * 100)
})

const progressOffset = computed(() => {
  const progress = props.total === 0 ? 0 : props.current / props.total
  return circumference * (1 - progress)
})

// 日志文本颜色
const getLogTextClass = (status: CheckinLogEntry['status']) => {
  switch (status) {
    case 'success':
      return 'text-green-400'
    case 'already_checked_in':
      return 'text-yellow-400'
    case 'failed':
      return 'text-red-400'
    case 'processing':
      return 'text-blue-400'
    default:
      return 'text-text-secondary'
  }
}

// 监听日志变化，自动滚动到底部
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
    :title="isFinished ? '签到完成' : '正在执行签到'"
    :close-on-backdrop="isFinished"
    :close-on-escape="isFinished"
    :show-close="false"
    :persistent="true"
    size="md"
  >
    <div class="py-4 space-y-6">
      <!-- 进度信息 -->
      <div class="text-center space-y-3">
        <!-- 圆形进度指示 -->
        <div class="relative inline-flex items-center justify-center w-24 h-24">
          <svg class="w-24 h-24 transform -rotate-90">
            <circle
              cx="48"
              cy="48"
              r="42"
              stroke="currentColor"
              stroke-width="6"
              fill="none"
              class="text-bg-tertiary"
            />
            <circle
              cx="48"
              cy="48"
              r="42"
              stroke="currentColor"
              stroke-width="6"
              fill="none"
              class="text-accent-primary transition-all duration-300"
              :stroke-dasharray="circumference"
              :stroke-dashoffset="progressOffset"
              stroke-linecap="round"
            />
          </svg>
          <span class="absolute text-2xl font-bold text-text-primary">
            <template v-if="isFinished">
              <CheckCircle class="w-10 h-10 text-green-500" />
            </template>
            <template v-else>
              {{ progressPercent }}%
            </template>
          </span>
        </div>

        <!-- 进度文本 -->
        <div class="space-y-1">
          <p class="text-sm text-text-secondary">
            {{ current }} / {{ total }} 账号
          </p>
          <p
            v-if="currentAccountName && !isFinished"
            class="text-sm font-medium text-accent-primary"
          >
            正在签到: {{ currentAccountName }}
          </p>
          <p
            v-if="isFinished"
            class="text-sm font-medium text-green-500"
          >
            全部任务执行完毕
          </p>
        </div>
      </div>

      <!-- 签到日志 -->
      <div class="space-y-2">
        <h4 class="text-sm font-medium text-text-secondary flex items-center gap-2">
          <FileText class="w-4 h-4" />
          签到日志
        </h4>
        <div
          ref="logContainerRef"
          class="h-48 overflow-y-auto rounded-lg bg-bg-secondary/50 border border-border-color/30 p-3 space-y-1.5"
        >
          <!-- (Log items logic remains same, just ensuring correct context) -->
          <div
            v-for="log in logs"
            :key="`${log.accountId}-${log.timestamp}`"
            class="flex items-start gap-2 text-sm"
          >
            <span class="flex-shrink-0 mt-0.5">
              <Loader2
                v-if="log.status === 'processing'"
                class="w-4 h-4 text-blue-500 animate-spin"
              />
              <CheckCircle
                v-else-if="log.status === 'success'"
                class="w-4 h-4 text-green-500"
              />
              <Clock
                v-else-if="log.status === 'already_checked_in'"
                class="w-4 h-4 text-yellow-500"
              />
              <XCircle
                v-else-if="log.status === 'failed'"
                class="w-4 h-4 text-red-500"
              />
              <Circle
                v-else
                class="w-4 h-4 text-text-muted"
              />
            </span>
            <div class="flex-1 min-w-0">
              <span
                class="font-medium"
                :class="getLogTextClass(log.status)"
              >
                {{ log.accountName }}
              </span>
              <span class="text-text-muted ml-1">
                ({{ log.providerName }})
              </span>
              <p
                v-if="log.message"
                class="text-xs mt-0.5 break-all"
                :class="log.status === 'failed' ? 'text-red-400' : 'text-text-muted'"
              >
                {{ log.message }}
              </p>
            </div>
          </div>
          <div
            v-if="logs.length === 0"
            class="flex items-center justify-center h-full text-text-muted text-sm"
          >
            等待开始签到...
          </div>
        </div>
      </div>

      <!-- 确认按钮 -->
      <div
        v-if="isFinished"
        class="pt-2"
      >
        <button
          class="w-full py-2.5 px-4 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors flex items-center justify-center gap-2"
          @click="emit('close')"
        >
          <CheckCircle class="w-5 h-5" />
          确定
        </button>
      </div>
    </div>
  </BaseModal>
</template>
