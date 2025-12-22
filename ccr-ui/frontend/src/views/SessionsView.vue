<template>
  <div class="sessions-view p-6 space-y-6">
    <!-- 页面标题 -->
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">
          📚 Sessions
        </h1>
        <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
          管理和浏览 AI CLI 会话记录
        </p>
      </div>
      <div class="flex items-center space-x-4">
        <!-- 平台筛选 -->
        <select
          v-model="selectedPlatform"
          class="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
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
          class="px-4 py-2 border border-blue-200 dark:border-blue-500 text-blue-700 dark:text-blue-200 rounded-lg flex items-center space-x-2 hover:bg-blue-50 dark:hover:bg-blue-900/30 disabled:opacity-50"
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
          class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg flex items-center space-x-2 disabled:opacity-50"
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
    <div class="grid grid-cols-1 md:grid-cols-4 gap-6">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-600 dark:text-gray-400">
              总会话数
            </p>
            <p class="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
              {{ stats?.total || 0 }}
            </p>
          </div>
          <div class="p-3 bg-blue-100 dark:bg-blue-900/20 rounded-full">
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
        class="bg-white dark:bg-gray-800 rounded-lg shadow p-6"
      >
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-600 dark:text-gray-400">
              {{ platform }}
            </p>
            <p class="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
              {{ count }}
            </p>
          </div>
          <div
            class="p-3 rounded-full"
            :class="getPlatformColor(platform)"
          >
            <span class="text-2xl">{{ getPlatformIcon(platform) }}</span>
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
      class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4"
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
      class="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden"
    >
      <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
        <thead class="bg-gray-50 dark:bg-gray-900/50">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              平台
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              标题/ID
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              目录
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              消息数
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              更新时间
            </th>
          </tr>
        </thead>
        <tbody class="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
          <tr
            v-for="session in sessions"
            :key="session.id"
            class="hover:bg-gray-50 dark:hover:bg-gray-700/50 cursor-pointer"
            @click="showSessionDetail(session)"
          >
            <td class="px-6 py-4 whitespace-nowrap">
              <span class="text-2xl">{{ getPlatformIcon(session.platform) }}</span>
            </td>
            <td class="px-6 py-4">
              <div class="text-sm font-medium text-gray-900 dark:text-white">
                {{ session.title || session.id.substring(0, 16) + '...' }}
              </div>
            </td>
            <td class="px-6 py-4">
              <div
                class="text-sm text-gray-500 dark:text-gray-400 truncate max-w-xs"
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
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
              {{ formatDate(session.updated_at) }}
            </td>
          </tr>
          <tr v-if="sessions.length === 0">
            <td
              colspan="5"
              class="px-6 py-12 text-center text-gray-500 dark:text-gray-400"
            >
              暂无会话记录
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Session 详情弹窗 -->
    <div
      v-if="selectedSession"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click.self="selectedSession = null"
    >
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-2xl w-full mx-4 p-6">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-xl font-bold text-gray-900 dark:text-white">
            Session 详情
          </h3>
          <button
            class="text-gray-500 hover:text-gray-800 dark:hover:text-white"
            @click="selectedSession = null"
          >
            ✕
          </button>
        </div>
        <div class="space-y-4">
          <div>
            <label class="text-sm font-medium text-gray-600 dark:text-gray-400">ID</label>
            <p class="text-gray-900 dark:text-white font-mono text-sm">
              {{ selectedSession.id }}
            </p>
          </div>
          <div>
            <label class="text-sm font-medium text-gray-600 dark:text-gray-400">平台</label>
            <p class="text-gray-900 dark:text-white">
              {{ selectedSession.platform }}
            </p>
          </div>
          <div v-if="selectedSession.title">
            <label class="text-sm font-medium text-gray-600 dark:text-gray-400">标题</label>
            <p class="text-gray-900 dark:text-white">
              {{ selectedSession.title }}
            </p>
          </div>
          <div>
            <label class="text-sm font-medium text-gray-600 dark:text-gray-400">工作目录</label>
            <p class="text-gray-900 dark:text-white font-mono text-sm break-all">
              {{ selectedSession.cwd }}
            </p>
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="text-sm font-medium text-gray-600 dark:text-gray-400">消息数</label>
              <p class="text-gray-900 dark:text-white">
                {{ selectedSession.message_count }}
              </p>
            </div>
            <div>
              <label class="text-sm font-medium text-gray-600 dark:text-gray-400">更新时间</label>
              <p class="text-gray-900 dark:text-white">
                {{ formatDate(selectedSession.updated_at) }}
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

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
  } catch (e: any) {
    error.value = e.message || '加载失败'
    console.error('Failed to load sessions:', e)
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
  } catch (e: any) {
    error.value = e.message || '重建索引失败'
  } finally {
    reindexing.value = false
  }
}

const showSessionDetail = (session: SessionSummary) => {
  selectedSession.value = session
}

const getPlatformIcon = (platform: string): string => {
  const icons: Record<string, string> = {
    Claude: '🔮',
    Codex: '🐙',
    Gemini: '💎',
    Qwen: '🌟',
    IFlow: '🌊'
  }
  return icons[platform] || '📦'
}

const getPlatformColor = (platform: string): string => {
  const colors: Record<string, string> = {
    Claude: 'bg-purple-100 dark:bg-purple-900/20',
    Codex: 'bg-gray-100 dark:bg-gray-700/50',
    Gemini: 'bg-blue-100 dark:bg-blue-900/20',
    Qwen: 'bg-orange-100 dark:bg-orange-900/20'
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
}
</style>
