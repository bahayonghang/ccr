<template>
  <div class="sessions-view space-y-6">
    <!-- 页面标题 -->
    <div class="sessions-header flex items-center justify-between gap-4">
      <div>
        <h1 class="text-3xl font-bold text-text-primary inline-flex items-center gap-3">
          <SIcon
            name="BookOpen"
            size="w-8 h-8"
            class="text-accent-primary"
          />
          <span>Sessions</span>
        </h1>
        <p class="mt-2 text-sm text-text-secondary">
          管理和浏览 AI CLI 会话记录
        </p>
      </div>
      <div class="sessions-toolbar flex items-center gap-3">
        <!-- 平台筛选 -->
        <select
          v-model="selectedPlatform"
          class="sessions-toolbar-select"
          @change="loadSessions"
        >
          <option value="">
            全部平台
          </option>
          <option value="claude">
            Claude
          </option>
          <option value="codex">
            Codex
          </option>
          <option value="gemini">
            Gemini
          </option>
        </select>

        <!-- 重建索引按钮 -->
        <button
          :disabled="reindexing"
          class="sessions-toolbar-button sessions-toolbar-button--secondary"
          @click="reindex"
        >
          <svg
            class="w-5 h-5"
            :class="{ 'animate-spin': reindexing }"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
            />
          </svg>
          <span>重建索引</span>
        </button>

        <!-- 刷新按钮 -->
        <button
          :disabled="loading"
          class="sessions-toolbar-button sessions-toolbar-button--primary"
          @click="loadSessions"
        >
          <svg
            class="w-5 h-5"
            :class="{ 'animate-spin': loading }"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
            />
          </svg>
          <span>刷新</span>
        </button>
      </div>
    </div>

    <!-- 统计卡片 -->
    <div class="grid grid-cols-1 gap-6 md:grid-cols-4">
      <div class="sessions-stat-card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-text-secondary">
              总会话数
            </p>
            <p class="mt-2 text-3xl font-bold text-text-primary">
              {{ stats?.total || 0 }}
            </p>
          </div>
          <div class="sessions-stat-icon sessions-stat-icon--primary">
            <svg
              class="w-8 h-8 text-blue-600 dark:text-blue-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
              />
            </svg>
          </div>
        </div>
      </div>
      <div
        v-for="(count, platform) in stats?.by_platform || {}"
        :key="platform"
        class="sessions-stat-card"
      >
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-text-secondary">
              {{ platform }}
            </p>
            <p class="mt-2 text-3xl font-bold text-text-primary">
              {{ count }}
            </p>
          </div>
          <div
            class="p-3 rounded-full"
            :class="getPlatformColor(platform)"
          >
            <SIcon
              :name="getPlatformIconName(platform)"
              size="w-7 h-7"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- 加载状态 -->
    <div
      v-if="loading"
      class="flex items-center justify-center py-12"
    >
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600" />
    </div>

    <!-- 错误提示 -->
    <div
      v-if="error"
      class="sessions-feedback sessions-feedback--error"
    >
      <div class="flex">
        <svg
          class="h-5 w-5 text-red-400"
          fill="currentColor"
          viewBox="0 0 20 20"
        >
          <path
            fill-rule="evenodd"
            d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
            clip-rule="evenodd"
          />
        </svg>
        <div class="ml-3">
          <h3 class="text-sm font-medium text-red-800 dark:text-red-200">
            加载失败
          </h3>
          <p class="mt-2 text-sm text-red-700 dark:text-red-300">
            {{ error }}
          </p>
        </div>
      </div>
    </div>

    <!-- Session 列表 -->
    <div
      v-if="!loading && !error"
      class="sessions-table-shell"
    >
      <table class="min-w-full">
        <thead class="sessions-table-head">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider">
              平台
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider">
              标题/ID
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider">
              目录
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider">
              消息数
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-text-muted uppercase tracking-wider">
              更新时间
            </th>
          </tr>
        </thead>
        <tbody class="sessions-table-body">
          <tr
            v-for="session in sessions"
            :key="session.id"
            class="sessions-table-row cursor-pointer"
            @click="showSessionDetail(session)"
          >
            <td class="px-6 py-4 whitespace-nowrap">
              <SIcon
                :name="getPlatformIconName(session.platform)"
                size="w-6 h-6"
              />
            </td>
            <td class="px-6 py-4">
              <div class="text-sm font-medium text-text-primary">
                {{ session.title || session.id.substring(0, 16) + '...' }}
              </div>
            </td>
            <td class="px-6 py-4">
              <div
                class="max-w-xs truncate text-sm text-text-muted"
                :title="session.cwd"
              >
                {{ shortenPath(session.cwd) }}
              </div>
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
              <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300">
                {{ session.message_count }}
              </span>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-text-muted">
              {{ formatDate(session.updated_at) }}
            </td>
          </tr>
          <tr v-if="sessions.length === 0">
            <td
              colspan="5"
              class="px-6 py-12 text-center text-text-muted"
            >
              暂无会话记录
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Session 详情弹窗 -->
    <BaseModal
      :model-value="Boolean(selectedSession)"
      title="Session 详情"
      description="Session metadata detail panel"
      size="xl"
      surface="solid"
      @update:model-value="handleSessionModalChange"
    >
      <div
        v-if="selectedSession"
        class="sessions-detail-grid"
      >
        <div class="sessions-detail-item sessions-detail-item--wide">
          <label class="sessions-detail-label">ID</label>
          <p class="sessions-detail-value sessions-detail-value--mono">
            {{ selectedSession.id }}
          </p>
        </div>
        <div class="sessions-detail-item">
          <label class="sessions-detail-label">平台</label>
          <p class="sessions-detail-value">
            {{ selectedSession.platform }}
          </p>
        </div>
        <div class="sessions-detail-item">
          <label class="sessions-detail-label">消息数</label>
          <p class="sessions-detail-value">
            {{ selectedSession.message_count }}
          </p>
        </div>
        <div
          v-if="selectedSession.title"
          class="sessions-detail-item sessions-detail-item--wide"
        >
          <label class="sessions-detail-label">标题</label>
          <p class="sessions-detail-value">
            {{ selectedSession.title }}
          </p>
        </div>
        <div class="sessions-detail-item sessions-detail-item--wide">
          <label class="sessions-detail-label">工作目录</label>
          <p class="sessions-detail-value sessions-detail-value--mono break-all">
            {{ selectedSession.cwd }}
          </p>
        </div>
        <div class="sessions-detail-item">
          <label class="sessions-detail-label">更新时间</label>
          <p class="sessions-detail-value">
            {{ formatDate(selectedSession.updated_at) }}
          </p>
        </div>
        <div class="sessions-detail-item">
          <label class="sessions-detail-label">创建时间</label>
          <p class="sessions-detail-value">
            {{ formatDate(selectedSession.created_at) }}
          </p>
        </div>
      </div>
    </BaseModal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import BaseModal from '@/components/common/BaseModal.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { logger } from '@/utils/logger'

interface SessionSummary {
  id: string
  platform: string
  title: string | null
  cwd: string
  created_at: string
  updated_at: string
  message_count: number
}

interface SessionStats {
  total: number
  by_platform: Record<string, number>
}

const sessions = ref<SessionSummary[]>([])
const stats = ref<SessionStats | null>(null)
const loading = ref(false)
const reindexing = ref(false)
const error = ref<string | null>(null)
const selectedPlatform = ref('')
const selectedSession = ref<SessionSummary | null>(null)

const API_BASE = '/api'

const loadSessions = async () => {
  loading.value = true
  error.value = null
  
  try {
    const params = new URLSearchParams()
    if (selectedPlatform.value) {
      params.set('platform', selectedPlatform.value)
    }
    params.set('limit', '50')
    
    const [sessionsRes, statsRes] = await Promise.all([
      fetch(`${API_BASE}/sessions?${params}`),
      fetch(`${API_BASE}/sessions/stats`)
    ])
    
    if (!sessionsRes.ok) throw new Error('Failed to load sessions')
    if (!statsRes.ok) throw new Error('Failed to load stats')
    
    sessions.value = await sessionsRes.json()
    stats.value = await statsRes.json()
  } catch (e) {
    error.value = (e instanceof Error ? e.message : "Error") || '加载失败'
    logger.error('Failed to load sessions:', e)
  } finally {
    loading.value = false
  }
}

const reindex = async () => {
  reindexing.value = true
  try {
    const res = await fetch(`${API_BASE}/sessions/reindex`, { method: 'POST' })
    if (!res.ok) throw new Error('Reindex failed')
    await loadSessions()
  } catch (e) {
    error.value = (e instanceof Error ? e.message : "Error") || '重建索引失败'
  } finally {
    reindexing.value = false
  }
}

const showSessionDetail = (session: SessionSummary) => {
  selectedSession.value = session
}

const handleSessionModalChange = (isOpen: boolean) => {
  if (!isOpen) {
    selectedSession.value = null
  }
}

const getPlatformIconName = (platform: string): string => {
  const icons: Record<string, string> = {
    Claude: 'Bot',
    Codex: 'Terminal',
    Gemini: 'Gem',
  }
  return icons[platform] || 'Package'
}

const getPlatformColor = (platform: string): string => {
  const colors: Record<string, string> = {
    Claude: 'bg-purple-100 dark:bg-purple-900/20',
    Codex: 'bg-gray-100 dark:bg-gray-700/50',
    Gemini: 'bg-blue-100 dark:bg-blue-900/20',
  }
  return colors[platform] || 'bg-gray-100 dark:bg-gray-900/20'
}

const shortenPath = (path: string): string => {
  const parts = path.split(/[/\\]/)
  if (parts.length <= 3) return path
  return '...' + parts.slice(-2).join('/')
}

const formatDate = (dateStr: string): string => {
  try {
    const date = new Date(dateStr)
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    
    if (diff < 60000) return '刚刚'
    if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`
    if (diff < 86400000) return `${Math.floor(diff / 3600000)} 小时前`
    if (diff < 604800000) return `${Math.floor(diff / 86400000)} 天前`
    
    return date.toLocaleDateString('zh-CN')
  } catch {
    return dateStr
  }
}

onMounted(() => {
  loadSessions()
})
</script>

<style scoped>
.sessions-view {
  min-height: calc(100vh - 64px);
  padding: 1rem;
}

.sessions-header {
  align-items: flex-start;
}

.sessions-toolbar {
  flex-wrap: wrap;
}

.sessions-toolbar-select,
.sessions-toolbar-button {
  min-height: 44px;
  border-radius: 0.875rem;
  transition:
    background-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    border-color var(--motion-subtle-duration) var(--motion-subtle-ease),
    box-shadow var(--motion-subtle-duration) var(--motion-subtle-ease),
    transform var(--motion-subtle-duration) var(--motion-subtle-ease),
    color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.sessions-toolbar-select {
  min-width: 11rem;
  padding: 0.625rem 1rem;
  color: var(--color-text-primary);
  background: var(--surface-status-bg);
  border: 1px solid var(--surface-status-border);
  backdrop-filter: var(--surface-status-blur);
  box-shadow: var(--elevation-1);
}

.sessions-toolbar-select:hover,
.sessions-toolbar-select:focus {
  border-color: rgb(var(--color-accent-primary-rgb) / 30%);
  box-shadow: var(--elevation-2);
  outline: none;
}

.sessions-toolbar-button {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.625rem 1rem;
  border: 1px solid transparent;
  font-weight: 600;
}

.sessions-toolbar-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.sessions-toolbar-button--primary {
  color: var(--color-text-inverted);
  background: linear-gradient(135deg, rgb(var(--color-accent-primary-rgb) / 96%), rgb(var(--color-accent-secondary-rgb) / 84%));
  box-shadow: var(--elevation-2);
}

.sessions-toolbar-button--primary:hover:not(:disabled),
.sessions-toolbar-button--secondary:hover:not(:disabled) {
  transform: translateY(-1px);
}

.sessions-toolbar-button--secondary {
  color: var(--color-accent-secondary);
  background: var(--surface-status-bg);
  border-color: rgb(var(--color-accent-secondary-rgb) / 24%);
  backdrop-filter: var(--surface-status-blur);
  box-shadow: var(--elevation-1);
}

.sessions-stat-card {
  border-radius: 1rem;
  padding: 1.5rem;
  background: var(--surface-card-bg);
  border: 1px solid var(--surface-card-border);
  backdrop-filter: var(--surface-card-blur);
  box-shadow: var(--elevation-2);
}

.sessions-stat-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.75rem;
  border-radius: 9999px;
  background: rgb(var(--color-bg-surface-rgb) / 70%);
}

.sessions-stat-icon--primary {
  background: rgb(var(--color-info-rgb) / 12%);
}

.sessions-feedback {
  border-radius: 1rem;
  padding: 1rem;
}

.sessions-feedback--error {
  background: rgb(var(--color-danger-rgb) / 10%);
  border: 1px solid rgb(var(--color-danger-rgb) / 24%);
}

.sessions-table-shell {
  overflow: hidden;
  border-radius: 1rem;
  background: var(--surface-workspace-bg);
  border: 1px solid var(--surface-workspace-border);
  backdrop-filter: var(--surface-workspace-blur);
  box-shadow: var(--elevation-2);
}

.sessions-table-head {
  background: rgb(var(--color-bg-base-rgb) / 58%);
}

.sessions-table-head th {
  padding: 0.75rem 1.5rem;
  text-align: left;
  font-size: 0.75rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-text-muted);
}

.sessions-table-body {
  background: transparent;
}

.sessions-table-body tr + tr {
  border-top: 1px solid rgb(var(--color-border-default-rgb) / 35%);
}

.sessions-table-row {
  transition: background-color var(--motion-subtle-duration) var(--motion-subtle-ease);
}

.sessions-table-row:hover {
  background: rgb(var(--color-bg-surface-rgb) / 48%);
}

.sessions-detail-grid {
  display: grid;
  gap: 1rem;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.sessions-detail-item {
  border-radius: 1rem;
  padding: 0.875rem 1rem;
  background: var(--surface-status-bg);
  border: 1px solid var(--surface-status-border);
  backdrop-filter: var(--surface-status-blur);
}

.sessions-detail-item--wide {
  grid-column: 1 / -1;
}

.sessions-detail-label {
  display: block;
  margin-bottom: 0.375rem;
  font-size: 0.75rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-text-muted);
}

.sessions-detail-value {
  color: var(--color-text-primary);
}

.sessions-detail-value--mono {
  font-family: var(--font-mono);
  font-size: 0.875rem;
}

@media (width >= 640px) {
  .sessions-view {
    padding: 1.5rem;
  }
}

@media (width <= 768px) {
  .sessions-detail-grid {
    grid-template-columns: 1fr;
  }

  .sessions-header {
    flex-direction: column;
  }
}
</style>
