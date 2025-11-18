<template>
  <div
    class="terminal-output"
    :class="{ streaming: isStreaming }"
  >
    <!-- 工具栏 -->
    <div class="terminal-toolbar">
      <div class="terminal-status">
        <span
          v-if="isStreaming"
          class="status-indicator streaming"
        >
          <span class="pulse" />
          流式输出中...
        </span>
        <span
          v-else-if="isComplete"
          class="status-indicator complete"
        >
          ✓ 完成
        </span>
        <span
          v-else
          class="status-indicator"
        >准备就绪</span>
      </div>
      <div class="terminal-actions">
        <button 
          v-if="isStreaming" 
          class="btn-action btn-stop" 
          title="停止"
          @click="handleStop"
        >
          停止
        </button>
        <button 
          class="btn-action" 
          :disabled="lines.length === 0"
          title="清空"
          @click="handleClear"
        >
          清空
        </button>
        <button 
          class="btn-action" 
          :disabled="lines.length === 0"
          title="复制"
          @click="handleCopy"
        >
          复制
        </button>
      </div>
    </div>

    <!-- 终端内容 -->
    <div 
      ref="terminalRef" 
      class="terminal-content"
      :class="{ 'auto-scroll': autoScroll }"
    >
      <div
        v-if="error"
        class="terminal-error"
      >
        ❌ {{ error }}
      </div>
      
      <div
        v-else-if="lines.length === 0"
        class="terminal-empty"
      >
        <div class="empty-icon">
          📟
        </div>
        <div class="empty-text">
          {{ emptyText }}
        </div>
      </div>

      <div
        v-else
        class="terminal-lines"
      >
        <div 
          v-for="(line, index) in lines" 
          :key="index" 
          class="terminal-line"
        >
          <span class="line-number">{{ index + 1 }}</span>
          <span class="line-content">{{ line }}</span>
        </div>
      </div>
    </div>

    <!-- 行数提示 -->
    <div
      v-if="lines.length > 0"
      class="terminal-footer"
    >
      <span class="line-count">共 {{ lines.length }} 行</span>
      <span
        v-if="lines.length >= maxLines"
        class="line-limit-warning"
      >
        (已达上限 {{ maxLines }} 行，最旧的行将被自动移除)
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from 'vue'
import { useStream } from '@/composables/useStream'
import { useUIStore } from '@/store'

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
  emptyText: '等待命令输出...'
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
  uiStore.showInfo('已停止输出')
}

// 清空输出
const handleClear = () => {
  clear()
  uiStore.showInfo('已清空输出')
}

// 复制输出
const handleCopy = async () => {
  try {
    const text = lines.value.join('\n')
    await navigator.clipboard.writeText(text)
    uiStore.showSuccess('已复制到剪贴板')
  } catch (err) {
    uiStore.showError('复制失败')
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
  background: #1e1e1e;
  border-radius: 8px;
  overflow: hidden;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
}

.terminal-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: #2d2d2d;
  border-bottom: 1px solid #404040;
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
  color: #888;
}

.status-indicator.streaming {
  color: #4caf50;
}

.status-indicator.complete {
  color: #2196f3;
}

.pulse {
  width: 8px;
  height: 8px;
  background: #4caf50;
  border-radius: 50%;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.3;
  }
}

.terminal-actions {
  display: flex;
  gap: 8px;
}

.btn-action {
  padding: 4px 12px;
  font-size: 12px;
  background: #404040;
  color: #fff;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.2s;
}

.btn-action:hover:not(:disabled) {
  background: #505050;
}

.btn-action:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-stop {
  background: #f44336;
}

.btn-stop:hover {
  background: #d32f2f;
}

.terminal-content {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  background: #1e1e1e;
  color: #d4d4d4;
  font-size: 13px;
  line-height: 1.6;
}

.terminal-content.auto-scroll {
  scroll-behavior: smooth;
}

.terminal-error {
  color: #f44336;
  padding: 12px;
  background: rgba(244, 67, 54, 0.1);
  border-radius: 4px;
}

.terminal-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #666;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 12px;
}

.empty-text {
  font-size: 14px;
}

.terminal-lines {
  font-size: 13px;
}

.terminal-line {
  display: flex;
  padding: 2px 0;
}

.terminal-line:hover {
  background: rgba(255, 255, 255, 0.05);
}

.line-number {
  flex-shrink: 0;
  width: 50px;
  color: #858585;
  text-align: right;
  padding-right: 12px;
  user-select: none;
}

.line-content {
  flex: 1;
  word-break: break-all;
  white-space: pre-wrap;
}

.terminal-footer {
  padding: 6px 12px;
  background: #2d2d2d;
  border-top: 1px solid #404040;
  font-size: 11px;
  color: #888;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.line-limit-warning {
  color: #ff9800;
}

/* 自定义滚动条 */
.terminal-content::-webkit-scrollbar {
  width: 8px;
}

.terminal-content::-webkit-scrollbar-track {
  background: #1e1e1e;
}

.terminal-content::-webkit-scrollbar-thumb {
  background: #555;
  border-radius: 4px;
}

.terminal-content::-webkit-scrollbar-thumb:hover {
  background: #666;
}
</style>
