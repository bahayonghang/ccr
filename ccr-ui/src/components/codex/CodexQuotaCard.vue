<template>
  <Card
    variant="glass"
    padding="lg"
  >
    <div class="flex items-center justify-between mb-4">
      <div class="flex items-center gap-2">
        <SIcon
          name="Gauge"
          size="w-5 h-5"
          class="text-platform-codex"
        />
        <h3 class="text-base font-semibold text-white">
          配额余额
        </h3>
      </div>
      <button
        class="btn btn-secondary btn-sm"
        :disabled="loading"
        @click="fetchQuotas"
      >
        <SIcon
          name="RefreshCw"
          size="w-4 h-4"
          :class="{ 'animate-spin': loading }"
        />
        {{ loading ? '查询中...' : '查询配额' }}
      </button>
    </div>

    <!-- 加载状态 -->
    <div
      v-if="loading"
      class="flex items-center gap-3 py-4"
    >
      <div class="w-5 h-5 rounded-full border-2 border-transparent border-t-platform-codex animate-spin" />
      <span class="text-sm text-white/60">正在查询配额余额...</span>
    </div>

    <!-- 错误状态 -->
    <div
      v-else-if="error"
      class="p-3 rounded-lg bg-red-500/10 border border-red-500/20 text-sm text-red-400"
    >
      {{ error }}
    </div>

    <!-- 无数据 -->
    <div
      v-else-if="quotas.length === 0 && !hasQueried"
      class="text-sm text-white/50 py-2"
    >
      点击"查询配额"获取账号 API 用量信息
    </div>

    <!-- 配额列表 -->
    <div
      v-else
      class="space-y-4"
    >
      <div
        v-for="aq in quotas"
        :key="aq.account_name"
        class="p-3 rounded-xl glass-surface border border-white/5"
      >
        <!-- 账号名 -->
        <div class="flex items-center justify-between mb-2">
          <div class="flex items-center gap-2">
            <span class="text-sm font-semibold text-white">{{ aq.account_name }}</span>
            <span
              v-if="aq.email"
              class="text-xs text-white/40"
            >{{ aq.email }}</span>
          </div>
          <span
            v-if="aq.quota?.plan_type"
            class="px-2 py-0.5 text-xs font-medium rounded-full"
            :class="planBadgeClass(aq.quota.plan_type)"
          >
            {{ aq.quota.plan_type }}
          </span>
        </div>

        <!-- 配额进度条 -->
        <div
          v-if="aq.quota"
          class="space-y-2"
        >
          <!-- 5h 窗口 -->
          <div class="flex items-center gap-3">
            <span class="text-xs text-white/60 w-14 shrink-0">5h 限额</span>
            <div class="flex-1 h-2 rounded-full bg-white/5 overflow-hidden">
              <div
                class="h-full rounded-full transition-all duration-500"
                :class="barColorClass(aq.quota.hourly_percentage)"
                :style="{ width: `${aq.quota.hourly_percentage}%` }"
              />
            </div>
            <span
              class="text-xs font-mono w-10 text-right"
              :class="textColorClass(aq.quota.hourly_percentage)"
            >{{ aq.quota.hourly_percentage }}%</span>
            <span
              v-if="aq.quota.hourly_reset_time"
              class="text-xs text-white/30 w-16 text-right"
            >{{ formatReset(aq.quota.hourly_reset_time) }}</span>
          </div>

          <!-- 周限额 -->
          <div class="flex items-center gap-3">
            <span class="text-xs text-white/60 w-14 shrink-0">周限额</span>
            <div class="flex-1 h-2 rounded-full bg-white/5 overflow-hidden">
              <div
                class="h-full rounded-full transition-all duration-500"
                :class="barColorClass(aq.quota.weekly_percentage)"
                :style="{ width: `${aq.quota.weekly_percentage}%` }"
              />
            </div>
            <span
              class="text-xs font-mono w-10 text-right"
              :class="textColorClass(aq.quota.weekly_percentage)"
            >{{ aq.quota.weekly_percentage }}%</span>
            <span
              v-if="aq.quota.weekly_reset_time"
              class="text-xs text-white/30 w-16 text-right"
            >{{ formatReset(aq.quota.weekly_reset_time) }}</span>
          </div>
        </div>

        <!-- 错误 -->
        <div
          v-else-if="aq.error"
          class="text-xs text-red-400 mt-1 truncate"
        >
          {{ aq.error }}
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import Card from '@/components/ui/Card.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { getCodexAllQuotas } from '@/api'
import type { CodexAccountQuota } from '@/types'
import { logger } from '@/utils/logger'

const loading = ref(false)
const error = ref<string | null>(null)
const quotas = ref<CodexAccountQuota[]>([])
const hasQueried = ref(false)

const fetchQuotas = async () => {
  try {
    loading.value = true
    error.value = null
    const data = await getCodexAllQuotas<CodexAccountQuota[]>()
    quotas.value = data
    hasQueried.value = true
  } catch (e) {
    logger.error('Failed to fetch codex quotas:', e)
    error.value = typeof e === 'string' ? e : '配额查询失败'
  } finally {
    loading.value = false
  }
}

const barColorClass = (pct: number) => {
  if (pct >= 60) return 'bg-emerald-500'
  if (pct >= 30) return 'bg-yellow-500'
  return 'bg-red-500'
}

const textColorClass = (pct: number) => {
  if (pct >= 60) return 'text-emerald-400'
  if (pct >= 30) return 'text-yellow-400'
  return 'text-red-400'
}

const planBadgeClass = (plan: string) => {
  const lower = plan.toLowerCase()
  if (lower === 'pro') return 'bg-purple-500/20 text-purple-400 border border-purple-500/30'
  if (lower === 'plus') return 'bg-blue-500/20 text-blue-400 border border-blue-500/30'
  if (lower === 'team') return 'bg-teal-500/20 text-teal-400 border border-teal-500/30'
  return 'bg-white/10 text-white/60 border border-white/10'
}

const formatReset = (timestamp: number) => {
  const now = Math.floor(Date.now() / 1000)
  const remaining = timestamp - now
  if (remaining <= 0) return '即将'

  const hours = Math.floor(remaining / 3600)
  const minutes = Math.floor((remaining % 3600) / 60)

  if (hours > 24) {
    const days = Math.floor(hours / 24)
    return `${days}d${hours % 24}h`
  }
  if (hours > 0) return `${hours}h${minutes}m`
  return `${minutes}m`
}
</script>
