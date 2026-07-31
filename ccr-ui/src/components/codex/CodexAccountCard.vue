<template>
  <Card
    variant="glass"
    padding="md"
    :hover="false"
    :glow="isCurrent"
    glow-color="success"
    :style="isCurrent ? { borderLeft: '3px solid rgb(96, 165, 250)' } : {}"
    class="codex-account-card"
  >
    <!-- 头部：账户信息 + 徽章 -->
    <div class="flex items-start justify-between gap-2 mb-2">
      <div class="flex items-center gap-2 min-w-0">
        <span class="text-base font-semibold text-text-primary truncate">
          {{ account.email || account.name }}
        </span>
      </div>
      <div class="flex items-center gap-1.5 flex-shrink-0 flex-wrap justify-end">
        <Badge
          v-if="isCurrent"
          variant="success"
          size="xs"
          pill
        >
          {{ $t('codex.auth.currentBadge') }}
        </Badge>
        <span
          v-if="quota?.quota?.plan_type"
          class="px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider rounded-full"
          :class="planBadgeClass(quota.quota.plan_type)"
        >
          {{ quota.quota.plan_type }}
        </span>
      </div>
    </div>

    <!-- 账户名与描述 -->
    <div class="mb-3 space-y-0.5">
      <div class="flex items-center gap-2 text-sm text-text-muted">
        <span class="font-mono font-medium">{{ account.name }}</span>
        <span
          v-if="account.is_virtual"
          class="text-text-ghost"
        >({{ $t('codex.auth.virtual') }})</span>
      </div>
      <p
        v-if="account.description"
        class="text-xs text-text-ghost truncate"
      >
        {{ account.description }}
      </p>
      <div class="flex flex-wrap items-center gap-1.5 text-[11px] text-text-muted">
        <span class="codex-account-chip">
          {{ authMethodLabel(account.auth_method) }}
        </span>
        <span
          v-if="account.api_provider_name"
          class="codex-account-chip"
        >
          {{ account.api_provider_name }}
        </span>
        <span
          v-else-if="account.api_base_url"
          class="codex-account-chip codex-account-chip--muted"
        >
          {{ account.api_base_url }}
        </span>
      </div>
    </div>

    <!-- 配额区域：骨架屏加载态 -->
    <div
      v-if="quotaLoading"
      class="space-y-3 mb-3"
    >
      <div
        v-for="i in 2"
        :key="i"
        class="space-y-1"
      >
        <div class="flex justify-between">
          <div class="h-3 w-16 rounded bg-bg-elevated animate-pulse" />
          <div class="h-3 w-8 rounded bg-bg-elevated animate-pulse" />
        </div>
        <div class="h-1.5 rounded-full bg-bg-elevated overflow-hidden">
          <div class="h-full w-1/2 rounded-full bg-bg-surface animate-pulse" />
        </div>
      </div>
    </div>

    <!-- 配额区域：已加载 -->
    <div
      v-else-if="quota?.quota"
      class="space-y-2.5 mb-3"
    >
      <!-- 5h 窗口 -->
      <div>
        <div class="flex items-center justify-between mb-1">
          <div class="flex items-center gap-1.5">
            <SIcon
              name="Clock"
              size="w-3 h-3"
              class="text-text-ghost"
            />
            <span class="text-xs text-text-muted">{{ $t('codex.auth.hourlyQuota') }}</span>
          </div>
          <span
            class="text-xs font-mono font-semibold"
            :class="textColorClass(quota.quota.hourly_percentage)"
          >{{ quota.quota.hourly_percentage }}%</span>
        </div>
        <div class="h-1.5 rounded-full bg-bg-elevated overflow-hidden">
          <div
            class="h-full w-full rounded-full origin-left transition-transform duration-500"
            :class="barColorClass(quota.quota.hourly_percentage)"
            :style="{ transform: `scaleX(${Math.min(quota.quota.hourly_percentage, 100) / 100})` }"
          />
        </div>
        <div
          v-if="quota.quota.hourly_reset_time"
          class="flex justify-end mt-0.5"
        >
          <span class="text-[11px] text-text-disabled">
            {{ formatReset(quota.quota.hourly_reset_time) }}
          </span>
        </div>
      </div>

      <!-- 周限额 -->
      <div>
        <div class="flex items-center justify-between mb-1">
          <div class="flex items-center gap-1.5">
            <SIcon
              name="CalendarDays"
              size="w-3 h-3"
              class="text-text-ghost"
            />
            <span class="text-xs text-text-muted">{{ $t('codex.auth.weeklyQuota') }}</span>
          </div>
          <span
            class="text-xs font-mono font-semibold"
            :class="textColorClass(quota.quota.weekly_percentage)"
          >{{ quota.quota.weekly_percentage }}%</span>
        </div>
        <div class="h-1.5 rounded-full bg-bg-elevated overflow-hidden">
          <div
            class="h-full w-full rounded-full origin-left transition-transform duration-500"
            :class="barColorClass(quota.quota.weekly_percentage)"
            :style="{ transform: `scaleX(${Math.min(quota.quota.weekly_percentage, 100) / 100})` }"
          />
        </div>
        <div
          v-if="quota.quota.weekly_reset_time"
          class="flex justify-end mt-0.5"
        >
          <span class="text-[11px] text-text-disabled">
            {{ formatResetDetailed(quota.quota.weekly_reset_time) }}
          </span>
        </div>
      </div>
    </div>

    <!-- 配额错误 -->
    <div
      v-else-if="quota?.error"
      class="mb-3 px-2 py-1.5 rounded-lg bg-red-500/10 border border-red-500/20 text-xs text-red-400 truncate"
    >
      {{ quota.error }}
    </div>

    <!-- 配额未加载 -->
    <div
      v-else-if="!quotaLoading"
      class="mb-3 text-xs text-text-disabled italic"
    >
      {{ $t('codex.auth.quotaNotQueried') }}
    </div>

    <!-- 底部：最后使用 + 操作按钮 -->
    <div class="flex items-center justify-between gap-3 pt-2 border-t border-border-default/10">
      <div class="min-w-0 space-y-0.5">
        <p class="text-xs text-text-disabled truncate">
          {{ formatLastUsed(account.last_used) }}
        </p>
        <p
          v-if="account.saved_at"
          class="text-[11px] text-text-ghost truncate"
        >
          {{ $t('codex.auth.fields.savedAt') }} {{ formatSavedAt(account.saved_at) }}
        </p>
      </div>
      <div class="flex items-center gap-1">
        <!-- 标签管理 -->
        <button
          class="card-action-btn"
          :title="$t('codex.auth.tagAccount')"
          :disabled="disabled"
          @click.stop="$emit('tag', account.name)"
        >
          <SIcon
            name="Tag"
            size="w-4 h-4"
          />
        </button>

        <!-- 重命名账户 -->
        <button
          v-if="!account.is_virtual"
          class="card-action-btn hover:text-accent-primary hover:bg-accent-primary/10"
          :title="$t('codex.auth.renameAccount')"
          :disabled="disabled"
          @click.stop="$emit('rename', account.name)"
        >
          <SIcon
            name="Pencil"
            size="w-4 h-4"
          />
        </button>

        <!-- 切换账户 -->
        <button
          v-if="!isCurrent"
          class="card-action-btn hover:text-accent-success hover:bg-accent-success/10"
          :title="$t('codex.auth.switch')"
          :disabled="disabled"
          @click.stop="$emit('switch', account.name)"
        >
          <SIcon
            :name="busyAction === 'switch' ? 'RefreshCw' : 'Play'"
            size="w-4 h-4"
            :class="{ 'animate-spin': busyAction === 'switch' }"
          />
        </button>

        <!-- 刷新配额 -->
        <button
          class="card-action-btn hover:text-platform-codex hover:bg-platform-codex/10"
          :title="$t('codex.auth.refreshQuota')"
          :disabled="disabled"
          @click.stop="$emit('refresh', account.name)"
        >
          <SIcon
            name="RefreshCw"
            size="w-4 h-4"
          />
        </button>

        <!-- 导出账户 -->
        <button
          class="card-action-btn hover:text-accent-secondary hover:bg-accent-secondary/10"
          :title="$t('codex.auth.exportAccount')"
          :disabled="disabled"
          @click.stop="$emit('export', account.name)"
        >
          <SIcon
            name="Upload"
            size="w-4 h-4"
          />
        </button>

        <!-- 删除账户 -->
        <button
          v-if="!account.is_virtual"
          class="card-action-btn hover:text-accent-danger hover:bg-accent-danger/10"
          :title="$t('codex.actions.delete')"
          :disabled="disabled"
          @click.stop="$emit('delete', account.name)"
        >
          <SIcon
            :name="busyAction === 'delete' ? 'RefreshCw' : 'Trash2'"
            size="w-4 h-4"
            :class="{ 'animate-spin': busyAction === 'delete' }"
          />
        </button>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import Card from '@/components/ui/Card.vue'
import Badge from '@/components/ui/Badge.vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { CodexAuthAccountItem, CodexAccountQuota } from '@/types'

interface Props {
  account: CodexAuthAccountItem
  quota?: CodexAccountQuota | null
  quotaLoading?: boolean
  isCurrent?: boolean
  busyAction?: 'switch' | 'delete' | null
  disabled?: boolean
}

withDefaults(defineProps<Props>(), {
  quota: null,
  quotaLoading: false,
  isCurrent: false,
  busyAction: null,
  disabled: false,
})

defineEmits<{
  switch: [name: string]
  delete: [name: string]
  refresh: [name: string]
  tag: [name: string]
  export: [name: string]
  rename: [name: string]
}>()

// ── 配额进度条颜色 ──

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

// ── Plan 类型 badge 样式 ──

const planBadgeClass = (plan: string) => {
  const lower = plan.toLowerCase()
  if (lower === 'pro') return 'bg-purple-500/20 text-purple-400 border border-purple-500/30'
  if (lower === 'plus') return 'bg-blue-500/20 text-blue-400 border border-blue-500/30'
  if (lower === 'team') return 'bg-teal-500/20 text-teal-400 border border-teal-500/30'
  return 'bg-bg-elevated text-text-muted border border-border-default/15'
}

const authMethodLabel = (method?: string) => {
  switch (method) {
    case 'chatgpt':
      return 'ChatGPT OAuth'
    case 'api':
      return 'API Key'
    default:
      return 'Managed Auth'
  }
}

// ── 重置时间格式化 ──

const formatReset = (timestamp: number) => {
  const now = Math.floor(Date.now() / 1000)
  const remaining = timestamp - now
  if (remaining <= 0) return ''

  const hours = Math.floor(remaining / 3600)
  const minutes = Math.floor((remaining % 3600) / 60)

  if (hours > 0) return `${hours}h${minutes}m`
  return `${minutes}m`
}

const formatResetDetailed = (timestamp: number) => {
  const now = Math.floor(Date.now() / 1000)
  const remaining = timestamp - now
  if (remaining <= 0) return ''

  const hours = Math.floor(remaining / 3600)
  const minutes = Math.floor((remaining % 3600) / 60)

  let relative: string
  if (hours > 24) {
    const days = Math.floor(hours / 24)
    relative = `${days}d ${hours % 24}h ${minutes}m`
  } else if (hours > 0) {
    relative = `${hours}h ${minutes}m`
  } else {
    relative = `${minutes}m`
  }

  const date = new Date(timestamp * 1000)
  const mm = String(date.getMonth() + 1).padStart(2, '0')
  const dd = String(date.getDate()).padStart(2, '0')
  const hh = String(date.getHours()).padStart(2, '0')
  const mi = String(date.getMinutes()).padStart(2, '0')

  return `${relative} (${mm}/${dd} ${hh}:${mi})`
}

const formatLastUsed = (raw?: string | null) => {
  if (!raw) return '—'
  try {
    const date = new Date(raw)
    if (isNaN(date.getTime())) return raw
    const y = date.getFullYear()
    const mm = String(date.getMonth() + 1).padStart(2, '0')
    const dd = String(date.getDate()).padStart(2, '0')
    const hh = String(date.getHours()).padStart(2, '0')
    const mi = String(date.getMinutes()).padStart(2, '0')
    return `${y}-${mm}-${dd} ${hh}:${mi}`
  } catch {
    return raw
  }
}

const formatSavedAt = (raw?: string | null) => {
  if (!raw) return ''
  try {
    const date = new Date(raw)
    if (isNaN(date.getTime())) return raw
    const y = date.getFullYear()
    const mm = String(date.getMonth() + 1).padStart(2, '0')
    const dd = String(date.getDate()).padStart(2, '0')
    const hh = String(date.getHours()).padStart(2, '0')
    const mi = String(date.getMinutes()).padStart(2, '0')
    return `${y}-${mm}-${dd} ${hh}:${mi}`
  } catch {
    return raw
  }
}
</script>

<style scoped>
.card-action-btn {
  @apply p-2 rounded-lg text-text-ghost transition-colors duration-200
         hover:text-text-secondary hover:bg-bg-surface/70
         disabled:opacity-30 disabled:cursor-not-allowed;
}

.codex-account-chip {
  @apply inline-flex items-center rounded-full border border-border-default/20 bg-bg-elevated px-2 py-0.5;
}

.codex-account-chip--muted {
  @apply max-w-full truncate;
}

.codex-account-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px -5px rgb(0 0 0 / 20%);
}
</style>
