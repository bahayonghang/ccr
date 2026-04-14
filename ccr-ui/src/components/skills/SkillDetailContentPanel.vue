<template>
  <div class="modal-body flex-1 overflow-y-auto">
    <div
      v-if="isContentLoading"
      class="flex flex-col items-center justify-center py-16"
    >
      <SIcon
        name="Loader2"
        size="w-8 h-8"
        class="animate-spin text-accent-primary"
      />
      <p class="mt-3 text-sm text-text-primary">
        {{ loadingLabel }}
      </p>
    </div>

    <div
      v-else-if="contentError"
      class="flex flex-col items-center justify-center py-16"
    >
      <SIcon
        name="AlertCircle"
        size="w-8 h-8"
        class="text-danger"
      />
      <p
        class="mt-2 text-sm"
        style="color: rgb(var(--color-danger-rgb));"
      >
        {{ contentError }}
      </p>
      <button
        class="btn-retry mt-4"
        @click="$emit('retry')"
      >
        {{ retryLabel }}
      </button>
    </div>

    <template v-else-if="skillContent">
      <SkillDetailMetadata
        :description="skillContent.description"
        :items="metadataItems"
        :tags="skillContent.tags"
      />

      <div class="content-divider">
        <span class="content-divider__label">
          {{ skillContentLabel }}
        </span>
      </div>

      <div
        v-if="!isEditMode"
        class="markdown-content"
      >
        <div
          v-if="renderedHtml"
          ref="markdownRef"
          class="prose"
          v-html="renderedHtml"
        />
        <p
          v-else
          class="py-8 text-center text-sm italic text-text-muted"
        >
          {{ noContentLabel }}
        </p>
      </div>

      <div
        v-else
        class="edit-content"
      >
        <textarea
          :value="editBuffer"
          class="edit-textarea"
          spellcheck="false"
          :placeholder="editPlaceholder"
          @input="handleInput"
        />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import SkillDetailMetadata from '@/components/skills/SkillDetailMetadata.vue'
import type { SkillContent } from '@/types/skills'
import type { SkillDetailMetaItem } from '@/types/skillDetailModal'
import { sanitizeMarkdown } from '@/utils/sanitize'
import { computed, nextTick, ref, watch } from 'vue'
import { marked } from 'marked'
import hljs from 'highlight.js/lib/core'
import javascript from 'highlight.js/lib/languages/javascript'
import typescript from 'highlight.js/lib/languages/typescript'
import python from 'highlight.js/lib/languages/python'
import bash from 'highlight.js/lib/languages/bash'
import json from 'highlight.js/lib/languages/json'
import yaml from 'highlight.js/lib/languages/yaml'
import xml from 'highlight.js/lib/languages/xml'
import css from 'highlight.js/lib/languages/css'
import rust from 'highlight.js/lib/languages/rust'
import go from 'highlight.js/lib/languages/go'
import sql from 'highlight.js/lib/languages/sql'
import markdown from 'highlight.js/lib/languages/markdown'
import diff from 'highlight.js/lib/languages/diff'

hljs.registerLanguage('javascript', javascript)
hljs.registerLanguage('js', javascript)
hljs.registerLanguage('typescript', typescript)
hljs.registerLanguage('ts', typescript)
hljs.registerLanguage('python', python)
hljs.registerLanguage('py', python)
hljs.registerLanguage('bash', bash)
hljs.registerLanguage('sh', bash)
hljs.registerLanguage('shell', bash)
hljs.registerLanguage('json', json)
hljs.registerLanguage('yaml', yaml)
hljs.registerLanguage('yml', yaml)
hljs.registerLanguage('xml', xml)
hljs.registerLanguage('html', xml)
hljs.registerLanguage('css', css)
hljs.registerLanguage('rust', rust)
hljs.registerLanguage('rs', rust)
hljs.registerLanguage('go', go)
hljs.registerLanguage('golang', go)
hljs.registerLanguage('sql', sql)
hljs.registerLanguage('markdown', markdown)
hljs.registerLanguage('md', markdown)
hljs.registerLanguage('diff', diff)

const props = defineProps<{
  isContentLoading: boolean
  contentError: string | null
  skillContent: SkillContent | null
  metadataItems: SkillDetailMetaItem[]
  isEditMode: boolean
  editBuffer: string
  loadingLabel: string
  retryLabel: string
  skillContentLabel: string
  noContentLabel: string
  editPlaceholder: string
}>()

const emit = defineEmits<{
  (e: 'retry'): void
  (e: 'update:editBuffer', value: string): void
}>()

const markdownRef = ref<HTMLElement | null>(null)

const renderedHtml = computed(() => {
  if (!props.skillContent?.content) return ''
  const html = marked.parse(props.skillContent.content) as string
  return sanitizeMarkdown(html)
})

watch(renderedHtml, () => {
  if (!props.isEditMode && renderedHtml.value) {
    nextTick(() => {
      markdownRef.value?.querySelectorAll('pre code').forEach((block) => {
        hljs.highlightElement(block as HTMLElement)
      })
    })
  }
})

function handleInput(event: Event) {
  emit('update:editBuffer', (event.target as HTMLTextAreaElement).value)
}
</script>

<style scoped>
.modal-body {
  @apply p-4 space-y-4;
}

.content-divider {
  @apply flex items-center gap-3;
}

.content-divider__label {
  @apply whitespace-nowrap text-xs font-bold uppercase tracking-wide text-text-muted;
}

.content-divider::after {
  content: '';

  @apply flex-1 h-px;

  background: rgb(var(--color-border-subtle-rgb) / 50%);
}

.markdown-content {
  @apply rounded-xl border border-border-default/10 p-4 overflow-x-auto;

  background: rgb(0 0 0 / 30%);
}

.markdown-content .prose {
  @apply max-w-none text-sm leading-relaxed text-white;
}

.markdown-content .prose :deep(h1) {
  @apply mt-4 mb-2 text-xl font-bold text-white;
}

.markdown-content .prose :deep(h2) {
  @apply mt-4 mb-2 text-lg font-bold text-white;
}

.markdown-content .prose :deep(h3) {
  @apply mt-3 mb-1.5 text-base font-semibold text-white;
}

.markdown-content .prose :deep(p) {
  @apply my-2;
}

.markdown-content .prose :deep(ul),
.markdown-content .prose :deep(ol) {
  @apply my-2 pl-5;
}

.markdown-content .prose :deep(li) {
  @apply my-0.5;
}

.markdown-content .prose :deep(code) {
  @apply px-1.5 py-0.5 rounded text-xs font-mono glass-surface text-accent-primary;
}

.markdown-content .prose :deep(pre) {
  @apply my-3 rounded-lg p-3 overflow-x-auto text-xs;

  background: rgb(0 0 0 / 40%);
}

.markdown-content .prose :deep(pre code) {
  @apply bg-transparent p-0 text-white;
}

.markdown-content .prose :deep(blockquote) {
  @apply my-3 border-l-2 pl-4 text-text-primary italic;

  border-color: rgb(var(--color-accent-primary-rgb) / 30%);
}

.markdown-content .prose :deep(a) {
  @apply text-accent-primary hover:underline;
}

.markdown-content .prose :deep(hr) {
  @apply my-4;

  border-color: rgb(var(--color-border-subtle-rgb) / 50%);
}

.markdown-content .prose :deep(table) {
  @apply my-3 w-full text-xs;
}

.markdown-content .prose :deep(th) {
  @apply border-b border-border-default/10 px-3 py-2 text-left font-semibold;
}

.markdown-content .prose :deep(td) {
  @apply border-b border-border-default/10 px-3 py-2;
}

.edit-content {
  @apply rounded-xl border border-border-default/10 overflow-hidden;

  background: rgb(0 0 0 / 30%);
}

.edit-textarea {
  @apply min-h-[400px] w-full resize-y border-0 bg-transparent p-4 text-sm font-mono leading-relaxed text-white outline-none;
}

.edit-textarea::placeholder {
  color: rgb(var(--color-text-muted-rgb) / 50%);
}

.btn-retry {
  @apply px-4 py-2 rounded-xl text-sm font-semibold bg-accent-primary text-white transition-colors hover:bg-accent-primary/90;
}
</style>

