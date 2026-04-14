<template>
  <div class="codex-sessions-view">
    <div class="codex-sessions-shell">
      <ModuleSubnav module="codex" />

      <div class="codex-sessions-stack">
        <div class="codex-sessions-header">
          <div class="codex-sessions-header__copy">
            <div class="codex-sessions-header__icon">
              <SIcon
                name="MessagesSquare"
                size="w-6 h-6"
                class="text-platform-codex"
              />
            </div>
            <div>
              <h1 class="codex-sessions-title">
                Codex Sessions
              </h1>
              <p class="codex-sessions-subtitle">
                直接读取本地 `~/.codex/sessions`，集中查看会话上下文、导出记录和复制工作流样本。
              </p>
            </div>
          </div>

          <div class="codex-sessions-header__actions">
            <RouterLink
              to="/codex"
              class="btn btn-secondary"
            >
              <SIcon
                name="ArrowLeft"
                size="w-4 h-4"
              />
              <span>返回 Codex</span>
            </RouterLink>
            <button
              class="hidden"
              :disabled="loading"
              @click="refreshSessions()"
            />
            <Button
              variant="primary"
              surface="card"
              density="compact"
              motion="standard"
              :disabled="loading"
              @click="refreshSessions()"
            >
              <template #leading>
                <SIcon
                  name="RefreshCw"
                  size="w-4 h-4"
                  :class="{ 'animate-spin': loading }"
                />
              </template>
              刷新列表
            </Button>
          </div>
        </div>

        <div class="codex-sessions-stats">
          <Card
            surface="status"
            :elevation="2"
            motion="subtle"
            class="codex-sessions-stat"
          >
            <p class="codex-sessions-stat__label">
              已加载会话
            </p>
            <p class="codex-sessions-stat__value">
              {{ sessions.length }}
            </p>
            <p class="codex-sessions-stat__hint">
              当前窗口最多展示 {{ SESSION_LIMIT }} 条最近记录
            </p>
          </Card>
          <Card
            surface="status"
            :elevation="2"
            motion="subtle"
            class="codex-sessions-stat"
          >
            <p class="codex-sessions-stat__label">
              列表 Tokens
            </p>
            <p class="codex-sessions-stat__value">
              {{ formatTokenCount(totalTokens) }}
            </p>
            <p class="codex-sessions-stat__hint">
              来自当前已加载的会话摘要
            </p>
          </Card>
          <Card
            surface="status"
            :elevation="2"
            motion="subtle"
            class="codex-sessions-stat"
          >
            <p class="codex-sessions-stat__label">
              当前会话消息
            </p>
            <p class="codex-sessions-stat__value">
              {{ selectedSession?.message_count ?? 0 }}
            </p>
            <p class="codex-sessions-stat__hint">
              仅统计用户与助手消息
            </p>
          </Card>
        </div>

        <div
          v-if="loadError"
          class="codex-sessions-error"
        >
          <SIcon
            name="AlertTriangle"
            size="w-4 h-4"
          />
          <span>{{ loadError }}</span>
        </div>

        <div class="codex-sessions-workspace">
          <Card
            surface="workspace"
            :elevation="2"
            motion="subtle"
            class="codex-sessions-panel codex-sessions-panel--list"
          >
            <div class="codex-sessions-panel__header">
              <div>
                <h2 class="codex-sessions-panel__title">
                  最近会话
                </h2>
                <p class="codex-sessions-panel__subtitle">
                  左侧列表用于快速切换，右侧查看详情和导出
                </p>
              </div>

              <div class="codex-sessions-search">
                <Input
                  v-model="searchQuery"
                  type="text"
                  surface="status"
                  :elevation="1"
                  motion="subtle"
                  density="compact"
                  :full-width="true"
                  placeholder="搜索 session id / cwd / model"
                >
                  <template #leading>
                    <SIcon
                      name="Search"
                      size="w-4 h-4"
                    />
                  </template>
                </Input>
              </div>
            </div>

            <div
              v-if="loading"
              class="codex-sessions-loading"
            >
              <div class="codex-sessions-loading__spinner" />
              <span>正在读取本地会话记录…</span>
            </div>

            <EmptyState
              v-else-if="filteredSessions.length === 0"
              icon="Inbox"
              title="没有匹配的会话"
              description="当前过滤条件下没有找到会话，试试清空搜索或刷新列表。"
              action-text="清空搜索"
              action-icon="RotateCcw"
              :on-action="clearSearch"
            />

            <div
              v-else
              class="codex-sessions-list"
            >
              <button
                v-for="session in filteredSessions"
                :key="session.file_path"
                type="button"
                class="codex-session-row"
                :class="{
                  'codex-session-row--active': session.file_path === selectedFilePath,
                }"
                @click="openSession(session.file_path)"
              >
                <div class="codex-session-row__top">
                  <div class="min-w-0">
                    <p class="codex-session-row__id">
                      {{ session.session_id }}
                    </p>
                    <p class="codex-session-row__meta">
                      {{ session.model || 'unknown model' }} ·
                      {{ formatRelative(session.updated_at) }}
                    </p>
                  </div>

                  <span class="codex-session-row__badge"> {{ session.message_count }} msg </span>
                </div>

                <p
                  v-if="session.preview"
                  class="codex-session-row__preview"
                >
                  {{ session.preview }}
                </p>

                <div class="codex-session-row__footer">
                  <span class="truncate">{{ session.cwd || session.relative_path }}</span>
                  <span>{{
                    formatTokenCount(session.total_input_tokens + session.total_output_tokens)
                  }}</span>
                </div>
              </button>
            </div>
          </Card>

          <Card
            surface="workspace"
            :elevation="2"
            motion="subtle"
            class="codex-sessions-panel codex-sessions-panel--detail"
          >
            <div class="codex-sessions-panel__header">
              <div>
                <h2 class="codex-sessions-panel__title">
                  会话详情
                </h2>
                <p class="codex-sessions-panel__subtitle">
                  当前默认只展示用户与助手消息，避免把系统提示词刷满工作台
                </p>
              </div>

              <div class="codex-detail-actions">
                <Button
                  variant="glass"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  :disabled="!selectedSession || actionLoading"
                  @click="copyFilePath"
                >
                  <template #leading>
                    <SIcon
                      name="Copy"
                      size="w-4 h-4"
                    />
                  </template>
                  复制路径
                </Button>
                <Button
                  variant="glass"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  :disabled="!selectedSession || actionLoading"
                  @click="handleExport"
                >
                  <template #leading>
                    <SIcon
                      name="Download"
                      size="w-4 h-4"
                    />
                  </template>
                  导出
                </Button>
                <Button
                  variant="glass"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  :disabled="!selectedSession || actionLoading"
                  @click="handleClone"
                >
                  <template #leading>
                    <SIcon
                      name="CopyPlus"
                      size="w-4 h-4"
                    />
                  </template>
                  克隆
                </Button>
                <Button
                  variant="danger"
                  surface="status"
                  density="compact"
                  motion="subtle"
                  :disabled="!selectedSession || actionLoading"
                  @click="handleDelete"
                >
                  <template #leading>
                    <SIcon
                      name="Trash2"
                      size="w-4 h-4"
                    />
                  </template>
                  删除
                </Button>
              </div>
            </div>

            <div
              v-if="detailLoading"
              class="codex-sessions-loading"
            >
              <div class="codex-sessions-loading__spinner" />
              <span>正在读取会话详情…</span>
            </div>

            <EmptyState
              v-else-if="!detail"
              icon="MessagesSquare"
              title="还没有选中会话"
              description="从左侧选择一个最近会话，就可以在这里查看详细上下文。"
            />

            <div
              v-else
              class="codex-detail"
            >
              <div class="codex-detail-summary">
                <div class="codex-detail-summary__title-row">
                  <div>
                    <h3 class="codex-detail-summary__title">
                      {{ selectedSession?.session_id }}
                    </h3>
                    <p class="codex-detail-summary__meta">
                      {{ selectedSession?.model || 'unknown model' }} ·
                      {{ formatAbsolute(selectedSession?.updated_at) }}
                    </p>
                  </div>
                  <span class="codex-detail-summary__pill">
                    {{ selectedSession?.total_requests ?? 0 }} req
                  </span>
                </div>

                <div class="codex-detail-grid">
                  <div class="codex-detail-field">
                    <span class="codex-detail-field__label">工作目录</span>
                    <button
                      type="button"
                      class="codex-detail-field__value codex-detail-field__value--button"
                      @click="copyCwd"
                    >
                      {{ selectedSession?.cwd || 'N/A' }}
                    </button>
                  </div>
                  <div class="codex-detail-field">
                    <span class="codex-detail-field__label">相对路径</span>
                    <span class="codex-detail-field__value">
                      {{ selectedSession?.relative_path }}
                    </span>
                  </div>
                  <div class="codex-detail-field">
                    <span class="codex-detail-field__label">输入 / 输出</span>
                    <span class="codex-detail-field__value">
                      {{ formatTokenCount(selectedSession?.total_input_tokens ?? 0) }}
                      /
                      {{ formatTokenCount(selectedSession?.total_output_tokens ?? 0) }}
                    </span>
                  </div>
                  <div class="codex-detail-field">
                    <span class="codex-detail-field__label">CLI 版本</span>
                    <span class="codex-detail-field__value">
                      {{ selectedSession?.cli_version || 'N/A' }}
                    </span>
                  </div>
                </div>

                <div
                  v-if="detail.clipped"
                  class="codex-detail-tip"
                >
                  详情面板只展示最近 {{ detail.message_limit }} 条消息，导出会沿用同样的窗口上限。
                </div>
              </div>

              <div class="codex-detail-messages">
                <article
                  v-for="(message, index) in detail.messages"
                  :key="`${message.timestamp || 'none'}-${index}`"
                  class="codex-message"
                  :class="
                    message.role === 'assistant'
                      ? 'codex-message--assistant'
                      : 'codex-message--user'
                  "
                >
                  <div class="codex-message__meta">
                    <span class="codex-message__role">
                      {{ message.role === 'assistant' ? 'Assistant' : 'User' }}
                    </span>
                    <span class="codex-message__time">
                      {{ formatAbsolute(message.timestamp) }}
                    </span>
                  </div>
                  <pre class="codex-message__body"><code>{{ message.text }}</code></pre>
                </article>
              </div>
            </div>
          </Card>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onActivated, onMounted, ref } from 'vue'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import Input from '@/components/ui/Input.vue'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import SIcon from '@/components/ui/SIcon.vue'
import {
  cloneCodexSession,
  deleteCodexSession,
  exportCodexSession,
  getCodexSessionDetail,
  listCodexSessions,
} from '@/api'
import { useUIStore } from '@/stores/ui'
import { copyToClipboard, formatRelativeTime, formatTimestamp } from '@/utils/codexHelpers'
import { logger } from '@/utils/logger'
import type {
  CodexCloneSessionResponse,
  CodexSessionDetailResponse,
  CodexSessionExportResponse,
  CodexSessionSummary,
  CodexSessionsResponse,
} from '@/types'

defineOptions({ name: 'CodexSessionsView' })

const SESSION_LIMIT = 160
const DETAIL_LIMIT = 120
const EXPORT_LIMIT = 200

const uiStore = useUIStore()

const loading = ref(true)
const detailLoading = ref(false)
const actionLoading = ref(false)
const loadError = ref<string | null>(null)
const sessions = ref<CodexSessionSummary[]>([])
const detail = ref<CodexSessionDetailResponse | null>(null)
const selectedFilePath = ref('')
const searchQuery = ref('')

const selectedSession = computed(() => detail.value?.session ?? null)

const filteredSessions = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) {
    return sessions.value
  }

  return sessions.value.filter((session) => {
    return [
      session.session_id,
      session.cwd ?? '',
      session.model ?? '',
      session.preview ?? '',
      session.relative_path,
    ].some((value) => value.toLowerCase().includes(query))
  })
})

const totalTokens = computed(() => {
  return sessions.value.reduce((total, session) => {
    return total + session.total_input_tokens + session.total_output_tokens
  }, 0)
})

function formatTokenCount(value: number): string {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`
  return String(value)
}

function formatRelative(value?: string | null): string {
  if (!value) return '未知时间'
  return formatRelativeTime(value)
}

function formatAbsolute(value?: string | null): string {
  if (!value) return '未知时间'
  return formatTimestamp(value)
}

function extractErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

async function loadSessionDetail(filePath: string) {
  detailLoading.value = true
  try {
    selectedFilePath.value = filePath
    detail.value = await getCodexSessionDetail<CodexSessionDetailResponse>(filePath, DETAIL_LIMIT)
  } catch (error) {
    const message = extractErrorMessage(error)
    logger.error('Failed to load codex session detail:', error)
    uiStore.showError(message)
  } finally {
    detailLoading.value = false
  }
}

async function loadSessions(preferredFilePath?: string) {
  loading.value = true
  loadError.value = null

  try {
    const response = await listCodexSessions<CodexSessionsResponse>({ limit: SESSION_LIMIT })
    sessions.value = response.sessions ?? []

    const nextFilePath = preferredFilePath ?? selectedFilePath.value ?? sessions.value[0]?.file_path

    if (nextFilePath && sessions.value.some((session) => session.file_path === nextFilePath)) {
      await loadSessionDetail(nextFilePath)
    } else if (sessions.value[0]?.file_path) {
      await loadSessionDetail(sessions.value[0].file_path)
    } else {
      selectedFilePath.value = ''
      detail.value = null
    }
  } catch (error) {
    loadError.value = extractErrorMessage(error)
    logger.error('Failed to load codex sessions:', error)
  } finally {
    loading.value = false
  }
}

function refreshSessions() {
  void loadSessions(selectedFilePath.value || undefined)
}

function clearSearch() {
  searchQuery.value = ''
}

function openSession(filePath: string) {
  if (filePath === selectedFilePath.value && detail.value) {
    return
  }
  void loadSessionDetail(filePath)
}

async function handleExport() {
  if (!selectedSession.value) {
    return
  }

  actionLoading.value = true
  try {
    const payload = await exportCodexSession<CodexSessionExportResponse>(
      selectedSession.value.file_path,
      EXPORT_LIMIT
    )
    const blob = new Blob([payload.content], { type: 'text/markdown;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = payload.file_name
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)
    uiStore.showSuccess('Session Markdown 已导出')
  } catch (error) {
    const message = extractErrorMessage(error)
    logger.error('Failed to export codex session:', error)
    uiStore.showError(message)
  } finally {
    actionLoading.value = false
  }
}

async function handleClone() {
  if (!selectedSession.value) {
    return
  }

  actionLoading.value = true
  try {
    const payload = await cloneCodexSession<CodexCloneSessionResponse>(
      selectedSession.value.file_path
    )
    await loadSessions(payload.session.file_path)
    uiStore.showSuccess('Session 已克隆到本地会话目录')
  } catch (error) {
    const message = extractErrorMessage(error)
    logger.error('Failed to clone codex session:', error)
    uiStore.showError(message)
  } finally {
    actionLoading.value = false
  }
}

async function handleDelete() {
  if (!selectedSession.value) {
    return
  }

  const confirmed = await uiStore.requestConfirm({
    title: '删除 Session',
    message: `确认删除 ${selectedSession.value.session_id} 吗？这个操作会直接删除本地 JSONL 文件。`,
    confirmText: '删除',
    cancelText: '取消',
    type: 'danger',
  })

  if (!confirmed) {
    return
  }

  const deletingPath = selectedSession.value.file_path
  const fallbackFilePath = sessions.value.find(
    (session) => session.file_path !== deletingPath
  )?.file_path

  actionLoading.value = true
  try {
    await deleteCodexSession(deletingPath)
    await loadSessions(fallbackFilePath)
    uiStore.showSuccess('Session 已删除')
  } catch (error) {
    const message = extractErrorMessage(error)
    logger.error('Failed to delete codex session:', error)
    uiStore.showError(message)
  } finally {
    actionLoading.value = false
  }
}

async function copyFilePath() {
  if (!selectedSession.value) return
  const success = await copyToClipboard(selectedSession.value.file_path)
  if (success) {
    uiStore.showSuccess('已复制 session 文件路径')
  } else {
    uiStore.showError('复制失败')
  }
}

async function copyCwd() {
  if (!selectedSession.value?.cwd) return
  const success = await copyToClipboard(selectedSession.value.cwd)
  if (success) {
    uiStore.showSuccess('已复制工作目录')
  } else {
    uiStore.showError('复制失败')
  }
}

onMounted(() => {
  void loadSessions()
})

onActivated(() => {
  if (!sessions.value.length) {
    void loadSessions()
  }
})
</script>

<style scoped>
.codex-sessions-view {
  @apply min-h-full p-6 lg:p-8;
}

.codex-sessions-shell {
  @apply mx-auto max-w-[1800px] space-y-6;
}

.codex-sessions-stack {
  @apply space-y-4;
}

.codex-sessions-header {
  @apply flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between;
}

.codex-sessions-header__copy {
  @apply flex items-start gap-4;
}

.codex-sessions-header__icon {
  @apply flex h-14 w-14 items-center justify-center rounded-2xl border border-platform-codex/20 bg-platform-codex/10;
}

.codex-sessions-title {
  @apply text-2xl font-bold text-text-primary lg:text-3xl;

  font-family: var(--font-brand);
}

.codex-sessions-subtitle {
  @apply mt-1 max-w-3xl text-sm leading-7 text-text-secondary;
}

.codex-sessions-header__actions {
  @apply flex flex-wrap gap-3;
}

.codex-sessions-stats {
  @apply grid gap-4 md:grid-cols-3;
}

.codex-sessions-stat {
  @apply p-5;
}

.codex-sessions-stat__label {
  @apply text-xs uppercase tracking-[0.2em] text-text-ghost;
}

.codex-sessions-stat__value {
  @apply mt-2 text-2xl font-semibold text-text-primary;
}

.codex-sessions-stat__hint {
  @apply mt-2 text-sm text-text-muted;
}

.codex-sessions-error {
  @apply flex items-center gap-2 rounded-2xl border border-rose-500/20 bg-rose-500/10 px-4 py-3 text-sm text-rose-200;
}

.codex-sessions-workspace {
  @apply grid gap-4 xl:grid-cols-[minmax(360px,420px)_minmax(0,1fr)];
}

.codex-sessions-panel {
  @apply p-5;
}

.codex-sessions-panel--list,
.codex-sessions-panel--detail {
  min-height: 720px;
}

.codex-sessions-panel__header {
  @apply mb-4 flex flex-col gap-4;
}

.codex-sessions-panel__title {
  @apply text-base font-semibold text-text-primary;
}

.codex-sessions-panel__subtitle {
  @apply text-sm text-text-muted;
}

.codex-sessions-loading {
  @apply flex h-full min-h-[320px] flex-col items-center justify-center gap-3 text-sm text-text-muted;
}

.codex-sessions-loading__spinner {
  @apply h-9 w-9 animate-spin rounded-full border-4 border-transparent border-r-platform-codex border-t-platform-codex;
}

.codex-sessions-list {
  @apply space-y-3 overflow-y-auto pr-1;

  max-height: 620px;
}

.codex-session-row {
  @apply w-full rounded-2xl border border-border-default/15 bg-bg-surface/70 p-4 text-left transition-all duration-200 hover:border-platform-codex/25 hover:bg-bg-elevated/80;
}

.codex-session-row--active {
  @apply border-platform-codex/35 bg-platform-codex/10 shadow-lg shadow-platform-codex/10;
}

.codex-session-row__top {
  @apply flex items-start justify-between gap-3;
}

.codex-session-row__id {
  @apply truncate font-mono text-sm font-semibold text-text-primary;
}

.codex-session-row__meta {
  @apply mt-1 text-xs text-text-ghost;
}

.codex-session-row__badge {
  @apply shrink-0 rounded-full border border-border-default/15 bg-bg-surface/70 px-2.5 py-1 text-[11px] text-text-secondary;
}

.codex-session-row__preview {
  @apply mt-3 line-clamp-3 text-sm leading-6 text-text-secondary;
}

.codex-session-row__footer {
  @apply mt-3 flex items-center justify-between gap-3 text-xs text-text-ghost;
}

.codex-detail-actions {
  @apply flex flex-wrap gap-2;
}

.codex-detail {
  @apply flex h-full flex-col gap-4;
}

.codex-detail-summary {
  @apply rounded-2xl border border-border-default/15 bg-bg-surface/70 p-4;
}

.codex-detail-summary__title-row {
  @apply flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between;
}

.codex-detail-summary__title {
  @apply font-mono text-lg font-semibold text-text-primary;
}

.codex-detail-summary__meta {
  @apply mt-1 text-sm text-text-muted;
}

.codex-detail-summary__pill {
  @apply inline-flex items-center rounded-full border border-platform-codex/20 bg-platform-codex/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.18em] text-platform-codex;
}

.codex-detail-grid {
  @apply mt-4 grid gap-3 md:grid-cols-2;
}

.codex-detail-field {
  @apply rounded-2xl border border-border-default/15 bg-bg-base/35 px-3 py-3;
}

.codex-detail-field__label {
  @apply text-[11px] uppercase tracking-[0.18em] text-text-ghost;
}

.codex-detail-field__value {
  @apply mt-1 block break-all text-sm text-text-primary;
}

.codex-detail-field__value--button {
  @apply text-left transition-colors hover:text-platform-codex;
}

.codex-detail-tip {
  @apply mt-4 rounded-xl border border-amber-500/20 bg-amber-500/10 px-3 py-2 text-sm text-amber-200;
}

.codex-detail-messages {
  @apply flex-1 space-y-3 overflow-y-auto pr-1;

  max-height: 520px;
}

.codex-message {
  @apply rounded-2xl border p-4;
}

.codex-message--assistant {
  @apply border-indigo-500/20 bg-indigo-500/10;
}

.codex-message--user {
  @apply border-border-default/15 bg-bg-surface/70;
}

.codex-message__meta {
  @apply flex items-center justify-between gap-3 text-xs;
}

.codex-message__role {
  @apply font-semibold uppercase tracking-[0.18em] text-text-muted;
}

.codex-message__time {
  @apply text-text-ghost;
}

.codex-message__body {
  @apply mt-3 overflow-x-auto whitespace-pre-wrap break-words text-sm leading-7 text-text-primary;

  font-family: var(--font-mono);
}
</style>


