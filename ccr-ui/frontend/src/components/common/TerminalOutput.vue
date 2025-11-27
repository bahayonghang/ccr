<template>
  <div
    class="terminal-output glass-effect"
    :class="{ streaming: isStreaming }"
  >
    <!-- 工具栏 -->
    <div class="terminal-toolbar border-b border-white/5 bg-white/5 backdrop-blur-sm">
      <div class="terminal-status">
        <span
          v-if="isStreaming"
          class="status-indicator streaming text-guofeng-jade"
        >
          <span class="pulse bg-guofeng-jade" />
          {{ $t('common.processing') || 'Processing...' }}
        </span>
        <span
          v-else-if="isComplete"
          class="status-indicator complete text-guofeng-jade"
        >
          <CheckCircle class="w-3.5 h-3.5" />
          {{ $t('common.completed') || 'Completed' }}
        </span>
        <span
          v-else
          class="status-indicator text-guofeng-text-muted"
        >
          <Terminal class="w-3.5 h-3.5" />
          {{ $t('common.ready') || 'Ready' }}
        </span>
      </div>
      <div class="terminal-actions">
        <button 
          v-if="isStreaming" 
          class="btn-action btn-stop hover:bg-red-500/20 hover:text-red-400" 
          :title="$t('common.stop')"
          @click="handleStop"
        >
          <Square class="w-3 h-3 fill-current" />
        </button>
        <button 
          class="btn-action hover:bg-white/10 hover:text-white" 
          :disabled="lines.length === 0"
          :title="$t('common.clear')"
          @click="handleClear"
        >
          <Trash2 class="w-3.5 h-3.5" />
        </button>
        <button 
          class="btn-action hover:bg-white/10 hover:text-white" 
          :disabled="lines.length === 0"
          :title="$t('common.copy')"
          @click="handleCopy"
        >
          <Copy class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- 终端内容 -->
    <div 
      ref="terminalRef" 
      class="terminal-content custom-scrollbar"
      :class="{ 'auto-scroll': autoScroll }"
    >
      <div
        v-if="error"
        class="terminal-error bg-red-500/10 text-red-400 border border-red-500/20"
      >
        <AlertTriangle class="w-4 h-4 shrink-0" />
        <span>{{ error }}</span>
      </div>
      
      <div
        v-else-if="lines.length === 0"
        class="terminal-empty text-guofeng-text-muted/30"
      >
        <div class="empty-icon mb-3">
          <Terminal class="w-12 h-12 opacity-20" />
        </div>
        <div class="empty-text text-sm">
          {{ emptyText }}
        </div>
      </div>

      <div
        v-else
        class="terminal-lines font-mono text-sm"
      >
        <div 
          v-for="(line, index) in lines" 
          :key="index" 
          class="terminal-line hover:bg-white/5 transition-colors"
        >
          <span class="line-number text-guofeng-text-muted/30 select-none w-8 text-right mr-3 text-xs">{{ index + 1 }}</span>
          <span class="line-content" v-html="renderAnsi(line)"></span>
        </div>
      </div>
    </div>

    <!-- 行数提示 -->
    <div
      v-if="lines.length > 0"
      class="terminal-footer border-t border-white/5 bg-white/5 backdrop-blur-sm text-xs text-guofeng-text-muted"
    >
      <span class="line-count">{{ lines.length }} lines</span>
      <span
        v-if="lines.length >= maxLines"
        class="line-limit-warning text-amber-500 flex items-center gap-1"
      >
        <AlertTriangle class="w-3 h-3" />
        Max lines reached
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'
import { useStream } from '@/composables/useStream'
import { useUIStore } from '@/store'
import { AnsiUp } from 'ansi_up'
import { 
  Terminal, 
  CheckCircle, 
  Square, 
  Trash2, 
  Copy, 
  AlertTriangle 
} from 'lucide-vue-next'

interface Props {
  streamUrl?: string
  maxLines?: number
  autoScroll?: boolean
  emptyText?: string
}

const props = withDefaults(defineProps<Props>(), {
  streamUrl: '',
  maxLines: 2000,
  autoScroll: true,
  emptyText: 'Waiting for output...'
})

const emit = defineEmits<{
  (e: 'complete'): void
  (e: 'error', error: string): void
}>()

const uiStore = useUIStore()
const terminalRef = ref<HTMLElement | null>(null)

// 使用流式 composable
const { lines, isStreaming, isComplete, error, start, stop, clear } = useStream(
  props.streamUrl,
  props.maxLines
)

// Render ANSI to HTML
// Create a new instance for each line to prevent state leakage between renders
const renderAnsi = (text: string) => {
  const ansiUp = new AnsiUp()
  ansiUp.use_classes = true
  return ansiUp.ansi_to_html(text)
}

// 自动启动流式读取
onMounted(() => {
  if (props.streamUrl) {
    start()
  }
})

// 监听完成状态
watch(isComplete, (complete) => {
  if (complete) {
    emit('complete')
  }
})

// 监听错误
watch(error, (err) => {
  if (err) {
    emit('error', err)
  }
})

// 自动滚动到底部
watch(
  () => lines.value.length,
  async () => {
    if (props.autoScroll && terminalRef.value) {
      await nextTick()
      terminalRef.value.scrollTop = terminalRef.value.scrollHeight
    }
  }
)

// 停止流式输出
const handleStop = () => {
  stop()
  uiStore.showInfo('Output stopped')
}

// 清空输出
const handleClear = () => {
  clear()
  uiStore.showInfo('Output cleared')
}

// 复制输出
const handleCopy = async () => {
  try {
    // Copy raw text without HTML tags
    const text = lines.value.map(line => line.replace(/\x1b\[[0-9;]*m/g, '')).join('\n')
    await navigator.clipboard.writeText(text)
    uiStore.showSuccess('Copied to clipboard')
  } catch (err) {
    uiStore.showError('Failed to copy')
  }
}

// 暴露方法给父组件
defineExpose({
  start,
  stop,
  clear,
  lines
})
</script>

<style scoped>
.terminal-output {
  display: flex;
  flex-direction: column;
  height: 100%;
  border-radius: 12px;
  overflow: hidden;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

.terminal-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
}

.terminal-status {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 500;
}

.pulse {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.4; transform: scale(0.8); }
}

.terminal-actions {
  display: flex;
  gap: 4px;
}

.btn-action {
  padding: 6px;
  border-radius: 6px;
  color: var(--text-muted);
  transition: all 0.2s;
}

.btn-action:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.terminal-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  /* Use a very dark transparent background for better contrast of colored text */
  background: rgba(10, 14, 39, 0.6); 
  color: #e5e7eb;
  font-size: 13px;
  line-height: 1.6;
}

.terminal-content.auto-scroll {
  scroll-behavior: smooth;
}

.terminal-error {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  border-radius: 8px;
  margin-bottom: 8px;
  font-size: 13px;
}

.terminal-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.terminal-line {
  display: flex;
  padding: 1px 0;
  border-radius: 2px;
}

.line-content {
  flex: 1;
  word-break: break-all;
  white-space: pre-wrap;
}

.terminal-footer {
  padding: 6px 12px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

/* ANSI Colors - matching standard terminal colors but tweaked for the theme */
:deep(.ansi-black-fg) { color: #3e4451; }
:deep(.ansi-red-fg) { color: #e06c75; }
:deep(.ansi-green-fg) { color: #98c379; }
:deep(.ansi-yellow-fg) { color: #e5c07b; }
:deep(.ansi-blue-fg) { color: #61afef; }
:deep(.ansi-magenta-fg) { color: #c678dd; }
:deep(.ansi-cyan-fg) { color: #56b6c2; }
:deep(.ansi-white-fg) { color: #abb2bf; }

:deep(.ansi-bright-black-fg) { color: #5c6370; }
:deep(.ansi-bright-red-fg) { color: #e06c75; }
:deep(.ansi-bright-green-fg) { color: #98c379; }
:deep(.ansi-bright-yellow-fg) { color: #e5c07b; }
:deep(.ansi-bright-blue-fg) { color: #61afef; }
:deep(.ansi-bright-magenta-fg) { color: #c678dd; }
:deep(.ansi-bright-cyan-fg) { color: #56b6c2; }
:deep(.ansi-bright-white-fg) { color: #ffffff; }

/* Custom Scrollbar */
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}
</style>
