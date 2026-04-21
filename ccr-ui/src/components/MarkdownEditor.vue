<template>
  <div class="flex flex-col h-full gap-4">
    <!-- Toolbar -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <button
          v-for="mode in viewModes"
          :key="mode.value"
          class="px-3 py-1.5 text-xs rounded-lg transition-colors"
          :class="currentMode === mode.value 
            ? 'bg-accent-secondary/20 text-accent-secondary border border-accent-secondary/30' 
            : 'bg-bg-surface text-text-muted hover:text-text-primary border border-transparent'"
          @click="currentMode = mode.value as 'edit' | 'preview' | 'split'"
        >
          <SIcon
            :name="mode.icon"
            size="w-3.5 h-3.5"
            class="inline mr-1"
          />
          {{ mode.label }}
        </button>
      </div>
      
      <!-- Variable Insert -->
      <div
        v-if="showVariables"
        class="flex items-center gap-2"
      >
        <span class="text-xs text-text-muted">{{ $t('editor.insertVariable') }}:</span>
        <button
          v-for="variable in availableVariables"
          :key="variable.key"
          class="px-2 py-1 text-[10px] rounded bg-accent-primary/10 text-accent-primary hover:bg-accent-primary/20 transition-colors"
          :title="variable.description"
          @click="insertVariable(variable.key)"
        >
          {{ variable.key }}
        </button>
      </div>
    </div>

    <!-- Editor Area -->
    <div
      class="flex-1 flex gap-4"
      :class="{ 'flex-col md:flex-row': currentMode === 'split' }"
    >
      <!-- Code Editor -->
      <div
        v-if="currentMode !== 'preview'"
        class="flex-1 relative"
        :class="{ 'md:w-1/2': currentMode === 'split' }"
      >
        <textarea
          ref="editorRef"
          v-model="internalContent"
          class="w-full h-full min-h-[300px] p-4 text-sm font-mono rounded-xl bg-bg-surface border border-border-default text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent-secondary resize-none"
          :placeholder="placeholder"
          @input="handleInput"
        />
        <div class="absolute bottom-3 right-3 text-[10px] text-text-muted">
          {{ lineCount }} {{ $t('editor.lines') }} · {{ charCount }} {{ $t('editor.chars') }}
        </div>
      </div>

      <!-- Preview -->
      <div
        v-if="currentMode !== 'edit'"
        class="flex-1 p-4 rounded-xl bg-bg-surface border border-border-default overflow-auto"
        :class="{ 'md:w-1/2': currentMode === 'split' }"
      >
        <div
          class="prose prose-invert prose-sm max-w-none"
          v-html="renderedHtml"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { renderMarkdown } from '@/composables/useMarkdownRender'

const { t } = useI18n({ useScope: 'global' })

interface Props {
  modelValue: string
  placeholder?: string
  showVariables?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: '',
  showVariables: true
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

const editorRef = ref<HTMLTextAreaElement | null>(null)
const internalContent = ref(props.modelValue)
const currentMode = ref<'edit' | 'preview' | 'split'>('split')

// 视图模式配置
const viewModes = [
  { value: 'edit', label: t('editor.edit'), icon: 'Code' },
  { value: 'split', label: t('editor.split'), icon: 'Columns' },
  { value: 'preview', label: t('editor.preview'), icon: 'Eye' }
]

// 可用的变量占位符
const availableVariables = [
  { key: '{{PROJECT_NAME}}', description: t('editor.variables.projectName') },
  { key: '{{LANGUAGE}}', description: t('editor.variables.language') },
  { key: '{{DATE}}', description: t('editor.variables.date') },
  { key: '{{AUTHOR}}', description: t('editor.variables.author') },
  { key: '{{FILE_PATH}}', description: t('editor.variables.filePath') }
]

// 统计信息
const lineCount = computed(() => internalContent.value.split('\n').length)
const charCount = computed(() => internalContent.value.length)

// 渲染后的 HTML
const renderedHtml = computed(() => {
  let content = internalContent.value

  // 替换变量占位符为高亮显示
  content = content.replace(
    /\{\{(\w+)\}\}/g,
    '<code class="px-1.5 py-0.5 rounded bg-accent-primary/20 text-accent-primary text-xs">{{$1}}</code>'
  )

  // 使用统一的 markdown 渲染入口（内置 sanitize + highlight 注册）
  return renderMarkdown(content, { breaks: true, gfm: true })
})

// 同步外部值到内部
watch(() => props.modelValue, (newVal) => {
  if (newVal !== internalContent.value) {
    internalContent.value = newVal
  }
})

// 处理输入
const handleInput = () => {
  emit('update:modelValue', internalContent.value)
}

// 插入变量占位符
const insertVariable = (key: string) => {
  if (!editorRef.value) return

  const textarea = editorRef.value
  const start = textarea.selectionStart
  const end = textarea.selectionEnd
  const before = internalContent.value.substring(0, start)
  const after = internalContent.value.substring(end)

  internalContent.value = before + key + after
  emit('update:modelValue', internalContent.value)

  // 恢复光标位置
  setTimeout(() => {
    textarea.focus()
    textarea.setSelectionRange(start + key.length, start + key.length)
  }, 0)
}
</script>

<style scoped>
.prose :deep(h1) { @apply text-xl font-bold text-text-primary mb-4; }
.prose :deep(h2) { @apply text-lg font-bold text-text-primary mb-3; }
.prose :deep(h3) { @apply text-base font-bold text-text-primary mb-2; }
.prose :deep(p) { @apply text-sm text-text-secondary mb-3; }
.prose :deep(ul) { @apply list-disc list-inside mb-3 text-sm text-text-secondary; }
.prose :deep(ol) { @apply list-decimal list-inside mb-3 text-sm text-text-secondary; }
.prose :deep(li) { @apply mb-1; }
.prose :deep(code) { @apply px-1.5 py-0.5 rounded bg-bg-base text-accent-primary text-xs font-mono; }
.prose :deep(pre) { @apply p-4 rounded-xl bg-bg-base text-sm font-mono overflow-x-auto mb-3; }
.prose :deep(blockquote) { @apply border-l-4 border-accent-secondary/50 pl-4 italic text-text-muted; }
.prose :deep(a) { @apply text-accent-secondary hover:underline; }
.prose :deep(hr) { @apply border-border-default my-4; }
.prose :deep(table) { @apply w-full text-sm mb-3; }
.prose :deep(th) { @apply border border-border-default px-3 py-2 bg-bg-base text-left; }
.prose :deep(td) { @apply border border-border-default px-3 py-2; }
</style>
