<template>
  <div
    class="rounded-xl mb-5 glass-effect overflow-hidden relative"
    :style="{
      border: '2px solid var(--accent-primary)',
      boxShadow: '0 0 30px rgba(var(--color-accent-secondary-rgb), 0.25), 0 8px 32px rgba(0, 0, 0, 0.2)',
      background: 'linear-gradient(135deg, rgba(var(--color-accent-secondary-rgb), 0.05), rgba(var(--color-accent-secondary-rgb), 0.03))'
    }"
  >
    <!-- 顶部装饰条 -->
    <div
      class="absolute top-0 left-0 w-full h-1"
      :style="{
        background: 'linear-gradient(90deg, var(--accent-primary), var(--accent-secondary), var(--accent-primary))',
        opacity: 0.8
      }"
      aria-hidden="true"
    />

    <!-- 折叠按钮和标题栏 -->
    <div
      class="flex items-center justify-between px-5 py-4 cursor-pointer hover:bg-opacity-50 transition-all"
      :style="{
        borderBottom: isCollapsed ? 'none' : '1px solid var(--border-color)'
      }"
      @click="toggleCollapsed"
    >
      <div class="flex items-center gap-3">
        <div
          class="w-2 h-2 rounded-full animate-pulse"
          :style="{
            background: 'var(--accent-success)',
            boxShadow: '0 0 10px var(--glow-success)'
          }"
          aria-label="状态：活跃"
        />
        <h2
          class="text-base font-bold uppercase tracking-wider"
          :style="{ color: 'var(--text-primary)' }"
        >
          系统状态面板
        </h2>
        <span
          class="px-2.5 py-0.5 rounded-lg text-xs font-semibold uppercase"
          :style="{
            background: 'var(--accent-primary)',
            color: 'white'
          }"
        >
          实时监控
        </span>
      </div>
      <button
        class="p-2 rounded-lg transition-all hover:bg-opacity-20 hover:scale-110"
        :style="{
          background: 'rgba(var(--color-accent-secondary-rgb), 0.1)',
          color: 'var(--accent-primary)'
        }"
        :aria-label="isCollapsed ? '展开' : '收起'"
        @click.stop="toggleCollapsed"
      >
        <ChevronDown
          v-if="isCollapsed"
          class="w-5 h-5"
        />
        <ChevronUp
          v-else
          class="w-5 h-5"
        />
      </button>
    </div>

    <!-- 可折叠内容 -->
    <div
      class="transition-all duration-300 ease-in-out overflow-hidden"
      :style="{
        maxHeight: isCollapsed ? '0' : '1000px',
        opacity: isCollapsed ? 0 : 1
      }"
    >
      <div class="p-5">
        <!-- 第一行：当前配置、统计信息、版本管理 -->
        <div class="grid grid-cols-1 lg:grid-cols-[2fr_1fr_1fr] gap-4 mb-4">
          <!-- 当前激活配置 - 占据更大空间 -->
          <div
            class="relative overflow-hidden rounded-lg p-5"
            :style="{
              background: 'linear-gradient(135deg, var(--bg-tertiary), var(--bg-secondary))',
              border: '2px solid var(--accent-primary)',
              boxShadow: '0 0 20px rgba(var(--color-accent-secondary-rgb), 0.3), inset 0 0 20px rgba(var(--color-accent-secondary-rgb), 0.05)'
            }"
          >
            <div
              class="absolute top-0 left-0 w-full h-0.5 scan-animation"
              :style="{
                background: 'linear-gradient(90deg, transparent, var(--accent-primary), transparent)'
              }"
              aria-hidden="true"
            />
            <div class="flex items-center justify-between mb-3">
              <span
                class="text-xs font-semibold uppercase tracking-wider"
                :style="{ color: 'var(--text-secondary)' }"
              >
                当前配置
              </span>
              <span
                class="w-2 h-2 rounded-full animate-pulse"
                :style="{
                  background: 'var(--accent-success)',
                  boxShadow: '0 0 10px var(--glow-success)'
                }"
                aria-label="状态：活跃"
              />
            </div>
            <div
              class="text-2xl font-bold uppercase font-mono tracking-wide truncate"
              :style="{ color: 'var(--text-primary)' }"
              :title="currentConfig"
            >
              {{ currentConfig || '-' }}
            </div>
          </div>

          <!-- 统计信息卡片 -->
          <div
            class="rounded-lg p-4"
            :style="{
              background: 'var(--bg-tertiary)',
              border: '2px solid var(--accent-secondary)',
              boxShadow: '0 0 15px rgba(var(--color-accent-secondary-rgb), 0.2), inset 0 0 15px rgba(var(--color-accent-secondary-rgb), 0.05)'
            }"
          >
            <div
              class="text-xs font-semibold uppercase tracking-wider mb-3"
              :style="{ color: 'var(--text-secondary)' }"
            >
              统计信息
            </div>
            <div class="space-y-3">
              <StatItem
                :icon="Server"
                label="总配置"
                :value="totalConfigs"
              />
              <StatItem
                :icon="Activity"
                label="历史记录"
                :value="historyCount"
              />
            </div>
          </div>

          <!-- 版本管理 -->
          <VersionManager />
        </div>

        <!-- 第二行：系统信息 -->
        <div
          v-if="systemInfo"
          class="rounded-lg p-4"
          :style="{
            background: 'var(--bg-tertiary)',
            border: '2px solid rgba(var(--color-success-rgb), 0.5)',
            boxShadow: '0 0 15px rgba(var(--color-success-rgb), 0.15), inset 0 0 15px rgba(var(--color-success-rgb), 0.05)'
          }"
        >
          <div class="flex items-center justify-between mb-3">
            <div
              class="text-xs font-semibold uppercase tracking-wider"
              :style="{ color: 'var(--text-secondary)' }"
            >
              系统信息
            </div>
            <div
              class="text-xs"
              :style="{ color: 'var(--text-muted)' }"
            >
              每5秒自动刷新
            </div>
          </div>
          <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4">
            <SystemMetric
              :icon="Server"
              label="主机"
              :value="systemInfo.hostname"
            />
            <SystemMetric
              :icon="Cpu"
              label="CPU"
              :value="`${systemInfo.cpu_cores} 核 · ${Math.round(systemInfo.cpu_usage)}%`"
              :progress="systemInfo.cpu_usage"
            />
            <SystemMetric
              :icon="HardDrive"
              label="内存"
              :value="`${systemInfo.used_memory_gb.toFixed(1)}/${systemInfo.total_memory_gb.toFixed(1)} GB`"
              :progress="systemInfo.memory_usage_percent"
            />
            <SystemMetric
              :icon="Activity"
              label="系统"
              :value="systemInfo.os"
            />
            <SystemMetric
              :icon="Clock"
              label="运行时间"
              :value="formatUptime(systemInfo.uptime_seconds)"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import {
  Activity,
  Cpu,
  HardDrive,
  Clock,
  Server,
  ChevronDown,
  ChevronUp
} from 'lucide-vue-next'
import { getSystemInfo } from '@/api/client'
import type { SystemInfo as SystemInfoType } from '@/types'
import VersionManager from './VersionManager.vue'

interface Props {
  currentConfig: string
  totalConfigs: number
  historyCount: number
}

defineProps<Props>()

const systemInfo = ref<SystemInfoType | null>(null)
const isCollapsed = ref(false)
let refreshInterval: number | null = null

onMounted(() => {
  // 从 localStorage 读取折叠状态
  const saved = localStorage.getItem('ccr-status-header-collapsed')
  if (saved === 'true') {
    isCollapsed.value = true
  }

  // 加载系统信息
  loadSystemInfo()
  // 每 5 秒刷新
  refreshInterval = window.setInterval(loadSystemInfo, 5000)
})

onBeforeUnmount(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})

const toggleCollapsed = () => {
  isCollapsed.value = !isCollapsed.value
  localStorage.setItem('ccr-status-header-collapsed', String(isCollapsed.value))
}

const loadSystemInfo = async () => {
  try {
    const data = await getSystemInfo()
    systemInfo.value = data
  } catch (err) {
    console.error('Failed to load system info:', err)
  }
}

const formatUptime = (seconds: number): string => {
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)

  if (days > 0) {
    return `${days}天 ${hours}时`
  } else if (hours > 0) {
    return `${hours}时 ${minutes}分`
  } else {
    return `${minutes}分钟`
  }
}
</script>

<script lang="ts">
import { defineComponent, h } from 'vue'

// StatItem 子组件
const StatItem = defineComponent({
  props: {
    icon: { type: Object, required: true },
    label: { type: String, required: true },
    value: { type: Number, required: true }
  },
  setup(props) {
    return () => h('div', { class: 'flex items-center justify-between' }, [
      h('div', { 
        class: 'flex items-center gap-2',
        style: { color: 'var(--text-muted)' }
      }, [
        h(props.icon, { class: 'w-4 h-4' }),
        h('span', { class: 'text-xs font-medium' }, props.label)
      ]),
      h('div', {
        class: 'text-xl font-bold font-mono',
        style: { color: 'var(--accent-primary)' }
      }, props.value)
    ])
  }
})

// SystemMetric 子组件
const SystemMetric = defineComponent({
  props: {
    icon: { type: Object, required: true },
    label: { type: String, required: true },
    value: { type: String, required: true },
    progress: { type: Number, default: undefined }
  },
  setup(props) {
    const getBorderColor = () => {
      if (props.progress === undefined) {
        return 'rgba(var(--color-accent-secondary-rgb), 0.4)'
      }
      if (props.progress > 80) {
        return 'rgba(var(--color-danger-rgb), 0.5)'
      }
      return 'rgba(var(--color-success-rgb), 0.5)'
    }

    const getGlowColor = () => {
      if (props.progress === undefined) {
        return 'rgba(var(--color-accent-secondary-rgb), 0.15)'
      }
      if (props.progress > 80) {
        return 'rgba(var(--color-danger-rgb), 0.2)'
      }
      return 'rgba(var(--color-success-rgb), 0.15)'
    }

    const getProgressBackground = () => {
      if (props.progress === undefined) return ''
      if (props.progress > 80) {
        return 'linear-gradient(90deg, var(--accent-warning), var(--accent-danger))'
      }
      return 'linear-gradient(90deg, var(--accent-primary), var(--accent-secondary))'
    }

    const getProgressShadow = () => {
      if (props.progress === undefined) return ''
      if (props.progress > 80) {
        return '0 0 8px var(--accent-danger)'
      }
      return '0 0 8px var(--accent-primary)'
    }

    return () => h('div', {
      class: 'rounded-lg p-3 space-y-2 transition-all duration-300 hover:scale-[1.02]',
      style: {
        background: 'linear-gradient(135deg, var(--bg-secondary), var(--bg-tertiary))',
        border: `1.5px solid ${getBorderColor()}`,
        boxShadow: `0 0 12px ${getGlowColor()}, inset 0 0 12px rgba(255, 255, 255, 0.02)`
      }
    }, [
      h('div', {
        class: 'flex items-center gap-1.5',
        style: { color: 'var(--text-muted)' }
      }, [
        h(props.icon, { class: 'w-4 h-4' }),
        h('span', { class: 'text-xs font-medium uppercase tracking-wide' }, props.label)
      ]),
      props.progress !== undefined && h('div', {
        class: 'w-full h-1.5 rounded-full overflow-hidden',
        style: {
          background: 'var(--bg-primary)',
          border: '1px solid rgba(255, 255, 255, 0.05)'
        },
        role: 'progressbar',
        'aria-valuenow': Math.min(props.progress, 100),
        'aria-valuemin': 0,
        'aria-valuemax': 100
      }, [
        h('div', {
          class: 'h-full rounded-full transition-all duration-500',
          style: {
            width: `${Math.min(props.progress, 100)}%`,
            background: getProgressBackground(),
            boxShadow: getProgressShadow()
          }
        })
      ]),
      h('div', {
        class: 'text-sm font-semibold font-mono truncate',
        style: { color: 'var(--text-primary)' },
        title: props.value
      }, props.value)
    ])
  }
})

export { StatItem, SystemMetric }
</script>

<style scoped>
@keyframes scan {
  0% {
    left: -100%;
  }

  100% {
    left: 100%;
  }
}

.scan-animation {
  animation: scan 3s linear infinite;
}
</style>
