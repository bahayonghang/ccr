<template>
  <div>
    <!-- 历史记录标题 -->
    <div class="flex items-center justify-between mb-6">
      <div>
        <h2 class="text-2xl font-bold mb-2" :style="{ color: 'var(--text-primary)' }">
          操作历史记录
        </h2>
        <p class="text-sm" :style="{ color: 'var(--text-secondary)' }">
          共 {{ entries.length }} 条记录
        </p>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="flex items-center justify-center py-20">
      <div
        class="w-12 h-12 rounded-full border-4 border-transparent animate-spin"
        :style="{
          borderTopColor: 'var(--accent-primary)',
          borderRightColor: 'var(--accent-secondary)'
        }"
        aria-label="加载中"
      />
    </div>

    <!-- 空状态 -->
    <div v-else-if="entries.length === 0" class="text-center py-20">
      <div class="inline-flex p-6 rounded-full mb-4" :style="{ background: 'var(--bg-tertiary)' }">
        <History class="w-12 h-12" :style="{ color: 'var(--text-muted)' }" />
      </div>
      <p class="text-lg font-medium mb-2" :style="{ color: 'var(--text-secondary)' }">
        暂无历史记录
      </p>
      <p class="text-sm" :style="{ color: 'var(--text-muted)' }">
        配置切换和管理操作将在此处显示
      </p>
    </div>

    <!-- 历史记录时间线 -->
    <div v-else class="space-y-4">
      <div
        v-for="(entry, index) in entries"
        :key="entry.id"
        class="relative glass-card p-5 rounded-xl transition-all duration-300 hover:scale-[1.01]"
        :style="{
          background: 'rgba(255, 255, 255, 0.03)',
          border: '1.5px solid rgba(139, 92, 246, 0.2)',
          boxShadow: '0 4px 16px rgba(0, 0, 0, 0.2)',
          animationDelay: `${index * 0.05}s`
        }"
      >
        <!-- 时间线连接线 -->
        <div
          v-if="index < entries.length - 1"
          class="absolute left-8 top-full w-0.5 h-4 -mb-4"
          :style="{ background: 'var(--border-color)' }"
        />

        <div class="flex gap-4">
          <!-- 图标区域 -->
          <div class="flex-shrink-0">
            <div
              class="w-12 h-12 rounded-xl flex items-center justify-center"
              :style="{
                background: getOperationColor(entry.operation, 0.15),
                border: `2px solid ${getOperationColor(entry.operation, 0.4)}`
              }"
            >
              <component
                :is="getOperationIcon(entry.operation)"
                class="w-6 h-6"
                :style="{ color: getOperationColor(entry.operation, 1) }"
              />
            </div>
          </div>

          <!-- 内容区域 -->
          <div class="flex-1 min-w-0">
            <!-- 操作标题和时间 -->
            <div class="flex items-start justify-between mb-3">
              <div>
                <h3 class="text-lg font-bold mb-1" :style="{ color: 'var(--text-primary)' }">
                  {{ getOperationLabel(entry.operation) }}
                </h3>
                <div class="flex items-center gap-3 text-sm">
                  <span
                    class="inline-flex items-center gap-1.5"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    <Clock class="w-4 h-4" />
                    {{ formatTimestamp(entry.timestamp) }}
                  </span>
                  <span
                    class="inline-flex items-center gap-1.5"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    <User class="w-4 h-4" />
                    {{ entry.actor }}
                  </span>
                </div>
              </div>

              <!-- 操作类型徽章 -->
              <span
                class="px-3 py-1 rounded-lg text-xs font-bold uppercase tracking-wide"
                :style="{
                  background: getOperationColor(entry.operation, 0.2),
                  color: getOperationColor(entry.operation, 1),
                  border: `1px solid ${getOperationColor(entry.operation, 0.4)}`
                }"
              >
                {{ entry.operation }}
              </span>
            </div>

            <!-- 配置切换信息 -->
            <div
              v-if="entry.from_config && entry.to_config"
              class="flex items-center gap-3 mb-4 p-3 rounded-lg"
              :style="{
                background: 'var(--bg-secondary)',
                border: '1px solid var(--border-color)'
              }"
            >
              <div
                class="px-3 py-1.5 rounded-md text-sm font-mono font-semibold"
                :style="{
                  background: 'rgba(239, 68, 68, 0.15)',
                  color: '#ef4444'
                }"
              >
                {{ entry.from_config }}
              </div>
              <ArrowRight class="w-5 h-5" :style="{ color: 'var(--text-muted)' }" />
              <div
                class="px-3 py-1.5 rounded-md text-sm font-mono font-semibold"
                :style="{
                  background: 'rgba(16, 185, 129, 0.15)',
                  color: '#10b981'
                }"
              >
                {{ entry.to_config }}
              </div>
            </div>

            <!-- 环境变量变化 -->
            <div v-if="entry.changes && entry.changes.length > 0" class="space-y-2">
              <h4
                class="text-xs font-semibold uppercase tracking-wide mb-2"
                :style="{ color: 'var(--text-muted)' }"
              >
                环境变量变化 ({{ entry.changes.length }})
              </h4>
              <div
                v-for="change in entry.changes.slice(0, 5)"
                :key="change.key"
                class="p-2.5 rounded-md text-xs font-mono"
                :style="{
                  background: 'var(--bg-tertiary)',
                  border: '1px solid var(--border-color)'
                }"
              >
                <div class="flex items-center gap-2 mb-1">
                  <span
                    class="font-bold"
                    :style="{ color: 'var(--accent-secondary)' }"
                  >
                    {{ change.key }}
                  </span>
                </div>
                <div class="flex items-start gap-2 text-[11px]">
                  <div class="flex-1">
                    <span :style="{ color: 'var(--text-muted)' }">旧值:</span>
                    <span
                      class="ml-1"
                      :style="{ color: 'var(--text-secondary)' }"
                    >
                      {{ change.old_value || '(空)' }}
                    </span>
                  </div>
                  <ArrowRight class="w-3 h-3 mt-0.5 flex-shrink-0" :style="{ color: 'var(--text-muted)' }" />
                  <div class="flex-1">
                    <span :style="{ color: 'var(--text-muted)' }">新值:</span>
                    <span
                      class="ml-1"
                      :style="{ color: 'var(--text-secondary)' }"
                    >
                      {{ change.new_value || '(空)' }}
                    </span>
                  </div>
                </div>
              </div>
              <button
                v-if="entry.changes.length > 5"
                class="text-xs font-medium px-3 py-1.5 rounded-md transition-all hover:scale-105"
                :style="{
                  background: 'var(--bg-tertiary)',
                  color: 'var(--accent-primary)',
                  border: '1px solid var(--border-color)'
                }"
              >
                查看全部 {{ entry.changes.length }} 项变化
              </button>
            </div>

            <!-- 操作 ID -->
            <div class="mt-3 pt-3" :style="{ borderTop: '1px solid var(--border-color)' }">
              <div class="flex items-center justify-between">
                <span class="text-xs" :style="{ color: 'var(--text-muted)' }">
                  ID: <code class="font-mono">{{ entry.id }}</code>
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ArrowRight, Clock, User, History, GitBranch, CheckCircle, RefreshCw, FileEdit, Trash2 } from 'lucide-vue-next'
import type { HistoryEntry } from '@/types'

interface Props {
  entries: HistoryEntry[]
  loading?: boolean
}

withDefaults(defineProps<Props>(), {
  loading: false
})

// 操作类型映射
const getOperationLabel = (operation: string): string => {
  const labels: Record<string, string> = {
    'switch': '配置切换',
    'init': '初始化配置',
    'update': '更新配置',
    'delete': '删除配置',
    'validate': '验证配置',
    'clean': '清理备份',
    'import': '导入配置',
    'export': '导出配置'
  }
  return labels[operation] || operation
}

// 操作图标映射
const getOperationIcon = (operation: string) => {
  const icons: Record<string, any> = {
    'switch': GitBranch,
    'init': CheckCircle,
    'update': FileEdit,
    'delete': Trash2,
    'validate': CheckCircle,
    'clean': RefreshCw,
    'import': ArrowRight,
    'export': ArrowRight
  }
  return icons[operation] || GitBranch
}

// 操作颜色映射
const getOperationColor = (operation: string, alpha: number): string => {
  const colors: Record<string, string> = {
    'switch': '#6366f1',      // 紫色
    'init': '#10b981',        // 绿色
    'update': '#3b82f6',      // 蓝色
    'delete': '#ef4444',      // 红色
    'validate': '#f59e0b',    // 橙色
    'clean': '#8b5cf6',       // 紫罗兰
    'import': '#06b6d4',      // 青色
    'export': '#ec4899'       // 粉色
  }
  
  const color = colors[operation] || '#6366f1'
  
  if (alpha === 1) return color
  
  // 将十六进制颜色转换为 rgba
  const r = parseInt(color.slice(1, 3), 16)
  const g = parseInt(color.slice(3, 5), 16)
  const b = parseInt(color.slice(5, 7), 16)
  
  return `rgba(${r}, ${g}, ${b}, ${alpha})`
}

// 格式化时间戳
const formatTimestamp = (timestamp: string): string => {
  const date = new Date(timestamp)
  const now = new Date()
  const diffInSeconds = Math.floor((now.getTime() - date.getTime()) / 1000)
  
  if (diffInSeconds < 60) {
    return '刚刚'
  } else if (diffInSeconds < 3600) {
    const minutes = Math.floor(diffInSeconds / 60)
    return `${minutes} 分钟前`
  } else if (diffInSeconds < 86400) {
    const hours = Math.floor(diffInSeconds / 3600)
    return `${hours} 小时前`
  } else if (diffInSeconds < 604800) {
    const days = Math.floor(diffInSeconds / 86400)
    return `${days} 天前`
  } else {
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    })
  }
}
</script>
