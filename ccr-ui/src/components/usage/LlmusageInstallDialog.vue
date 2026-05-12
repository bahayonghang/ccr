<template>
  <BaseModal
    :model-value="isOpen"
    title="llmusage 未安装"
    :close-on-backdrop="false"
    :close-on-escape="state !== 'auto-running'"
    :show-close="state !== 'auto-running'"
    size="md"
    @update:model-value="handleClose"
    @close="handleClose"
  >
    <!-- Checking state -->
    <div v-if="state === 'checking'" class="flex flex-col items-center py-8">
      <div class="animate-spin h-8 w-8 border-2 border-accent-primary border-t-transparent rounded-full" />
      <p class="mt-4 text-sm text-text-secondary">正在检测 llmusage 安装状态...</p>
    </div>

    <!-- Absent: show choice -->
    <div v-else-if="state === 'absent-choice'" class="space-y-4">
      <div class="bg-bg-surface rounded-lg p-4 border border-border-default">
        <p class="text-sm text-text-primary leading-relaxed">
          <strong>llmusage</strong> 是 CCR 使用统计功能的核心依赖，用于解析和聚合各 AI 平台（Claude Code、Codex、Gemini、OpenCode）的 token 使用数据。
        </p>
        <p class="text-sm text-text-secondary mt-2">
          检测到当前系统未安装 llmusage，导入功能无法正常工作。请选择安装方式：
        </p>
      </div>

      <!-- Platform info -->
      <div class="flex items-center gap-2 text-xs text-text-tertiary">
        <span>检测到平台：</span>
        <span class="font-mono bg-bg-overlay px-2 py-0.5 rounded">{{ platformLabel }}</span>
        <span v-if="planOutcome?.kind === 'plan'">
          · 推荐：<span class="font-mono">{{ (planOutcome as any).package_manager }}</span>
        </span>
      </div>

      <!-- Action buttons -->
      <div class="flex gap-3">
        <button
          class="flex-1 rounded-xl px-4 py-3 text-sm font-medium bg-accent-primary text-white hover:bg-accent-primary/90 transition-colors"
          :disabled="planOutcome?.kind !== 'plan'"
          @click="startAutoInstall"
        >
          🚀 自动安装
        </button>
        <button
          class="flex-1 rounded-xl px-4 py-3 text-sm font-medium border border-border-default bg-bg-surface text-text-primary hover:bg-bg-overlay transition-colors"
          @click="showManualInstall"
        >
          📋 手动安装
        </button>
      </div>

      <p
        v-if="planOutcome?.kind === 'unsupported'"
        class="text-xs text-accent-warning"
      >
        未检测到可用的包管理器，请使用手动安装。
      </p>
    </div>

    <!-- Auto install running -->
    <div v-else-if="state === 'auto-running'" class="space-y-4">
      <div class="flex items-center justify-between">
        <p class="text-sm font-medium text-text-primary">正在安装 llmusage...</p>
        <button
          class="text-xs text-accent-danger hover:underline"
          @click="cancelInstall"
        >
          取消
        </button>
      </div>

      <!-- Progress -->
      <div v-if="currentStage" class="flex items-center gap-2 text-xs text-text-secondary">
        <div class="animate-spin h-3 w-3 border border-accent-primary border-t-transparent rounded-full" />
        <span>{{ stageLabel(currentStage) }}</span>
      </div>

      <!-- Log output -->
      <div
        ref="logContainer"
        class="bg-bg-overlay rounded-lg p-3 max-h-48 overflow-y-auto font-mono text-xs text-text-secondary border border-border-default"
      >
        <div v-for="log in recentLogs" :key="log.seq" class="leading-relaxed">
          {{ log.line }}
        </div>
        <div v-if="recentLogs.length === 0" class="text-text-tertiary italic">
          等待输出...
        </div>
      </div>
    </div>

    <!-- Auto install terminal -->
    <div v-else-if="state === 'auto-terminal'" class="space-y-4">
      <!-- Success -->
      <div v-if="terminalOutcome === 'succeeded'" class="text-center py-4">
        <div class="text-4xl mb-3">✅</div>
        <p class="text-sm font-medium text-text-primary">llmusage 安装成功！</p>
        <p class="text-xs text-text-secondary mt-1">
          版本：{{ installedVersion || '未知' }} · 耗时：{{ formatDuration(installDurationMs) }}
        </p>
        <p class="text-xs text-text-tertiary mt-2">正在自动重试导入...</p>
      </div>

      <!-- Failed -->
      <div v-else-if="terminalOutcome === 'failed'" class="space-y-3">
        <div class="text-center">
          <div class="text-4xl mb-3">❌</div>
          <p class="text-sm font-medium text-text-primary">安装失败</p>
        </div>
        <div class="bg-accent-danger/10 border border-accent-danger/20 rounded-lg p-3">
          <p class="text-xs text-accent-danger font-mono">{{ failureMessage }}</p>
        </div>
        <div class="flex gap-3">
          <button
            class="flex-1 rounded-xl px-4 py-2.5 text-sm font-medium bg-accent-primary text-white hover:bg-accent-primary/90"
            @click="retryAutoInstall"
          >
            重试
          </button>
          <button
            class="flex-1 rounded-xl px-4 py-2.5 text-sm font-medium border border-border-default bg-bg-surface text-text-primary hover:bg-bg-overlay"
            @click="showManualInstall"
          >
            手动安装
          </button>
        </div>
      </div>

      <!-- Cancelled -->
      <div v-else-if="terminalOutcome === 'cancelled'" class="text-center py-4">
        <div class="text-4xl mb-3">⏹️</div>
        <p class="text-sm font-medium text-text-primary">安装已取消</p>
        <button
          class="mt-4 rounded-xl px-6 py-2.5 text-sm font-medium border border-border-default bg-bg-surface text-text-primary hover:bg-bg-overlay"
          @click="state = 'absent-choice'"
        >
          返回
        </button>
      </div>
    </div>

    <!-- Manual install -->
    <div v-else-if="state === 'manual'" class="space-y-4">
      <p class="text-sm text-text-secondary">
        请在终端中执行以下命令安装 llmusage：
      </p>

      <!-- Commands grouped by platform -->
      <div v-if="manualCatalog" class="space-y-3">
        <div
          v-for="cmd in currentPlatformCommands"
          :key="cmd.command_line"
          class="bg-bg-overlay rounded-lg p-3 border border-border-default"
        >
          <div class="flex items-center justify-between mb-1">
            <span class="text-xs font-medium text-text-primary">{{ cmd.title }}</span>
            <button
              class="text-xs text-accent-primary hover:underline"
              @click="copyCommand(cmd.command_line)"
            >
              {{ copiedCommand === cmd.command_line ? '已复制 ✓' : '复制' }}
            </button>
          </div>
          <code class="block text-xs font-mono text-text-secondary bg-bg-surface rounded px-2 py-1.5">
            {{ cmd.command_line }}
          </code>
          <p v-if="cmd.notes" class="text-xs text-text-tertiary mt-1">
            {{ cmd.notes }}
          </p>
        </div>
      </div>

      <!-- Docs link -->
      <div v-if="manualCatalog" class="text-xs text-text-tertiary">
        📖 文档：<a
          :href="manualCatalog.docs_url"
          target="_blank"
          class="text-accent-primary hover:underline"
        >{{ manualCatalog.docs_url }}</a>
      </div>

      <!-- Re-check button -->
      <div class="flex gap-3 pt-2">
        <button
          class="flex-1 rounded-xl px-4 py-2.5 text-sm font-medium bg-accent-primary text-white hover:bg-accent-primary/90"
          :disabled="recheckLoading"
          @click="recheckInstallation"
        >
          {{ recheckLoading ? '检测中...' : '🔍 重新检测' }}
        </button>
        <button
          class="flex-1 rounded-xl px-4 py-2.5 text-sm font-medium border border-border-default bg-bg-surface text-text-primary hover:bg-bg-overlay"
          @click="state = 'absent-choice'"
        >
          返回
        </button>
      </div>
    </div>
  </BaseModal>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import BaseModal from '@/components/common/BaseModal.vue'
import {
  llmusageInstallCheck,
  llmusageInstallPlan,
  llmusageInstallExecute,
  llmusageInstallCancel,
  llmusageInstallManualCatalog,
  llmusageInstallDetect,
  type DetectionResult,
  type HostCapabilities,
  type PlanOutcome,
  type InstallEvent,
  type InstallPlan,
  type ManualCatalog,
  type ProgressStage,
} from '@/api/domains/install'

// ──────────────────────────────────────────────────────────────────────────────
// Props & Emits
// ──────────────────────────────────────────────────────────────────────────────

interface Props {
  isOpen: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:isOpen': [value: boolean]
  'install-success': []
  'retry-import': []
}>()

// ──────────────────────────────────────────────────────────────────────────────
// State
// ──────────────────────────────────────────────────────────────────────────────

type DialogState = 'checking' | 'absent-choice' | 'auto-running' | 'auto-terminal' | 'manual'

const state = ref<DialogState>('checking')
const detection = ref<DetectionResult | null>(null)
const capabilities = ref<HostCapabilities | null>(null)
const planOutcome = ref<PlanOutcome | null>(null)
const manualCatalog = ref<ManualCatalog | null>(null)

// Auto install state
const currentAttemptId = ref<string | null>(null)
const recentLogs = ref<Array<{ line: string; seq: number }>>([])
const currentStage = ref<ProgressStage | null>(null)
const terminalOutcome = ref<'succeeded' | 'failed' | 'cancelled' | null>(null)
const installedVersion = ref<string | null>(null)
const installDurationMs = ref(0)
const failureMessage = ref('')
const autoRetryFired = ref<Set<string>>(new Set())

// Manual install state
const copiedCommand = ref<string | null>(null)
const recheckLoading = ref(false)

// Log container ref for auto-scroll
const logContainer = ref<HTMLElement | null>(null)

// Event listener cleanup
let unlisten: UnlistenFn | null = null

// ──────────────────────────────────────────────────────────────────────────────
// Computed
// ──────────────────────────────────────────────────────────────────────────────

const platformLabel = computed(() => {
  if (!capabilities.value) return '未知'
  const labels: Record<string, string> = {
    macos: 'macOS',
    linux: 'Linux',
    windows: 'Windows',
  }
  return labels[capabilities.value.platform] || capabilities.value.platform
})

const currentPlatformCommands = computed(() => {
  if (!manualCatalog.value || !capabilities.value) return []
  return manualCatalog.value.entries.filter(
    (e) => e.platform === capabilities.value!.platform,
  )
})

// ──────────────────────────────────────────────────────────────────────────────
// Lifecycle
// ──────────────────────────────────────────────────────────────────────────────

watch(() => props.isOpen, async (open) => {
  if (open) {
    await startCheck()
  } else {
    cleanup()
  }
})

onUnmounted(() => {
  cleanup()
})

// ──────────────────────────────────────────────────────────────────────────────
// Methods
// ──────────────────────────────────────────────────────────────────────────────

async function startCheck() {
  state.value = 'checking'
  try {
    const [det, caps] = await llmusageInstallCheck()
    detection.value = det
    capabilities.value = caps

    if (det.status === 'available') {
      // Already available — close and proceed with import
      emit('retry-import')
      handleClose()
      return
    }

    // Generate plan
    planOutcome.value = await llmusageInstallPlan(det, caps)

    // Load manual catalog
    try {
      manualCatalog.value = await llmusageInstallManualCatalog()
    } catch {
      // Non-fatal
    }

    state.value = 'absent-choice'
  } catch (e) {
    console.error('[install-dialog] check failed:', e)
    state.value = 'absent-choice'
  }
}

async function startAutoInstall() {
  if (!planOutcome.value || planOutcome.value.kind !== 'plan') return

  state.value = 'auto-running'
  recentLogs.value = []
  currentStage.value = null
  terminalOutcome.value = null

  // Subscribe to events
  await setupEventListener()

  try {
    const plan = planOutcome.value as unknown as InstallPlan
    const attemptId = await llmusageInstallExecute(plan)
    currentAttemptId.value = typeof attemptId === 'string' ? attemptId : (attemptId as any)[0] || String(attemptId)
  } catch (e) {
    state.value = 'auto-terminal'
    terminalOutcome.value = 'failed'
    failureMessage.value = String(e)
  }
}

function retryAutoInstall() {
  startAutoInstall()
}

async function cancelInstall() {
  if (currentAttemptId.value) {
    try {
      await llmusageInstallCancel(currentAttemptId.value)
    } catch {
      // Best effort
    }
  }
}

function showManualInstall() {
  state.value = 'manual'
}

async function recheckInstallation() {
  recheckLoading.value = true
  try {
    const det = await llmusageInstallDetect()
    if (det.status === 'available') {
      emit('install-success')
      emit('retry-import')
      handleClose()
    }
  } catch {
    // Stay on manual page
  } finally {
    recheckLoading.value = false
  }
}

async function copyCommand(cmd: string) {
  try {
    await navigator.clipboard.writeText(cmd)
    copiedCommand.value = cmd
    setTimeout(() => {
      copiedCommand.value = null
    }, 2000)
  } catch {
    // Fallback: select text
  }
}

function handleClose() {
  emit('update:isOpen', false)
  cleanup()
}

function cleanup() {
  if (unlisten) {
    unlisten()
    unlisten = null
  }
  state.value = 'checking'
  recentLogs.value = []
  currentStage.value = null
  terminalOutcome.value = null
  currentAttemptId.value = null
}

async function setupEventListener() {
  if (unlisten) {
    unlisten()
  }
  unlisten = await listen<InstallEvent>('llmusage.install', (event) => {
    handleInstallEvent(event.payload)
  })
}

function handleInstallEvent(ev: InstallEvent) {
  switch (ev.type) {
    case 'log':
      recentLogs.value.push({ line: ev.line, seq: ev.seq })
      // Keep only last 100 lines in UI
      if (recentLogs.value.length > 100) {
        recentLogs.value = recentLogs.value.slice(-100)
      }
      // Auto-scroll
      nextTick(() => {
        if (logContainer.value) {
          logContainer.value.scrollTop = logContainer.value.scrollHeight
        }
      })
      break

    case 'progress':
      currentStage.value = ev.stage
      break

    case 'succeeded':
      state.value = 'auto-terminal'
      terminalOutcome.value = 'succeeded'
      installedVersion.value = ev.installed_version
      installDurationMs.value = ev.duration_ms
      // Auto-retry (once per attempt)
      if (ev.attempt_id && !autoRetryFired.value.has(ev.attempt_id)) {
        autoRetryFired.value.add(ev.attempt_id)
        setTimeout(() => {
          emit('install-success')
          emit('retry-import')
          handleClose()
        }, 1500)
      }
      break

    case 'failed':
      state.value = 'auto-terminal'
      terminalOutcome.value = 'failed'
      failureMessage.value = ev.error_message
      break

    case 'cancelled':
      state.value = 'auto-terminal'
      terminalOutcome.value = 'cancelled'
      break
  }
}

function stageLabel(stage: ProgressStage): string {
  const labels: Record<ProgressStage, string> = {
    resolving: '解析依赖...',
    downloading: '下载中...',
    compiling: '编译中...',
    installing: '安装中...',
    finalizing: '完成中...',
  }
  return labels[stage] || stage
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  const seconds = Math.round(ms / 1000)
  if (seconds < 60) return `${seconds}s`
  const minutes = Math.floor(seconds / 60)
  const remaining = seconds % 60
  return `${minutes}m ${remaining}s`
}
</script>
