<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div
        v-if="modelValue"
        class="fixed inset-0 z-50 flex items-center justify-center p-4"
        @click.self="close"
      >
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" />

        <!-- Modal Content -->
        <div class="modal-content relative w-full max-w-3xl max-h-[85vh] flex flex-col">
          <!-- Header -->
          <div class="modal-header">
            <div class="flex items-center gap-3 min-w-0">
              <div
                class="modal-icon shrink-0"
                :style="{ backgroundColor: platformColor + '15', color: platformColor }"
              >
                <component
                  :is="platformIcon"
                  class="w-5 h-5"
                />
              </div>
              <div class="min-w-0">
                <h2 class="text-lg font-bold text-text-primary truncate">
                  {{ skillContent?.name || skill?.name }}
                </h2>
                <p class="text-xs text-text-muted truncate">
                  {{ skill?.platformName }}
                </p>
              </div>
            </div>
            <div class="flex items-center gap-1 shrink-0">
              <!-- Mode Toggle -->
              <button
                class="p-2 rounded-lg text-text-muted hover:text-text-primary
                       hover:bg-bg-surface transition-colors"
                :title="isEditMode ? $t('skills.previewMode') : $t('skills.editMode')"
                @click="toggleMode"
              >
                <Eye
                  v-if="isEditMode"
                  class="w-5 h-5"
                />
                <Edit3
                  v-else
                  class="w-5 h-5"
                />
              </button>
              <!-- Close -->
              <button
                class="p-2 rounded-lg text-text-muted hover:text-text-primary
                       hover:bg-bg-surface transition-colors"
                @click="close"
              >
                <X class="w-5 h-5" />
              </button>
            </div>
          </div>

          <!-- Body (Scrollable) -->
          <div class="modal-body flex-1 overflow-y-auto">
            <!-- Loading State -->
            <div
              v-if="isContentLoading"
              class="flex flex-col items-center justify-center py-16"
            >
              <Loader2 class="w-8 h-8 animate-spin text-accent-primary" />
              <p class="text-text-secondary text-sm mt-3">
                {{ $t('skills.loadingContent') }}
              </p>
            </div>

            <!-- Error State -->
            <div
              v-else-if="contentError"
              class="flex flex-col items-center justify-center py-16"
            >
              <AlertCircle class="w-8 h-8 text-danger" />
              <p
                class="text-sm mt-2"
                style="color: rgb(var(--color-danger-rgb));"
              >
                {{ contentError }}
              </p>
              <button
                class="btn-retry mt-4"
                @click="loadContent"
              >
                {{ $t('common.retry') }}
              </button>
            </div>

            <!-- Content Loaded -->
            <template v-else-if="skillContent">
              <!-- Metadata Section -->
              <div class="metadata-section">
                <!-- Description -->
                <p
                  v-if="skillContent.description"
                  class="text-sm text-text-secondary leading-relaxed"
                >
                  {{ skillContent.description }}
                </p>

                <!-- Meta Grid -->
                <div class="meta-grid">
                  <!-- Category -->
                  <div
                    v-if="skillContent.category"
                    class="meta-item"
                  >
                    <Folder class="w-3.5 h-3.5 text-text-muted shrink-0" />
                    <span class="meta-label">{{ $t('skills.categoryLabel') }}</span>
                    <span class="meta-value">{{ skillContent.category }}</span>
                  </div>

                  <!-- Platform -->
                  <div class="meta-item">
                    <component
                      :is="platformIcon"
                      class="w-3.5 h-3.5 shrink-0"
                      :style="{ color: platformColor }"
                    />
                    <span class="meta-label">{{ $t('skills.platform') }}</span>
                    <span class="meta-value">{{ skill?.platformName }}</span>
                  </div>

                  <!-- Version -->
                  <div
                    v-if="skill?.version"
                    class="meta-item"
                  >
                    <Tag class="w-3.5 h-3.5 text-text-muted shrink-0" />
                    <span class="meta-label">{{ $t('skills.version') }}</span>
                    <span class="meta-value">v{{ skill.version }}</span>
                  </div>

                  <!-- Author -->
                  <div
                    v-if="skill?.author"
                    class="meta-item"
                  >
                    <User class="w-3.5 h-3.5 text-text-muted shrink-0" />
                    <span class="meta-label">{{ $t('skills.author') }}</span>
                    <span class="meta-value">{{ skill.author }}</span>
                  </div>

                  <!-- Source -->
                  <div
                    v-if="skill?.source"
                    class="meta-item"
                  >
                    <component
                      :is="sourceIconMap[skill.source] || HardDrive"
                      class="w-3.5 h-3.5 text-text-muted shrink-0"
                    />
                    <span class="meta-label">{{ $t('skills.sourceLabel') }}</span>
                    <span class="meta-value flex items-center gap-1">
                      {{ sourceLabelMap[skill.source] || skill.source }}
                      <a
                        v-if="skill.sourceUrl"
                        :href="skill.sourceUrl"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="text-accent-primary hover:underline"
                        @click.stop
                      >
                        <ExternalLink class="w-3 h-3" />
                      </a>
                    </span>
                  </div>

                  <!-- Install Date -->
                  <div
                    v-if="skill?.installDate"
                    class="meta-item"
                  >
                    <Clock class="w-3.5 h-3.5 text-text-muted shrink-0" />
                    <span class="meta-label">{{ $t('skills.installedAt') }}</span>
                    <span class="meta-value">{{ formatDate(skill.installDate) }}</span>
                  </div>

                  <!-- Directory -->
                  <div class="meta-item">
                    <FolderOpen class="w-3.5 h-3.5 text-text-muted shrink-0" />
                    <span class="meta-label">{{ $t('skills.directory') }}</span>
                    <span
                      class="meta-value font-mono text-[11px]"
                      :title="skillContent.skillDir"
                    >
                      {{ shortenPath(skillContent.skillDir) }}
                    </span>
                  </div>
                </div>

                <!-- Tags -->
                <div
                  v-if="skillContent.tags && skillContent.tags.length > 0"
                  class="flex flex-wrap gap-1 mt-2"
                >
                  <span
                    v-for="tag in skillContent.tags"
                    :key="tag"
                    class="tag-badge"
                  >
                    #{{ tag }}
                  </span>
                </div>
              </div>

              <!-- Divider -->
              <div class="content-divider">
                <span class="content-divider__label">
                  {{ $t('skills.skillContent') }}
                </span>
              </div>

              <!-- View Mode: Rendered Markdown -->
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
                  class="text-sm text-text-muted italic py-8 text-center"
                >
                  {{ $t('skills.noContent') }}
                </p>
              </div>

              <!-- Edit Mode: Textarea -->
              <div
                v-else
                class="edit-content"
              >
                <textarea
                  v-model="editBuffer"
                  class="edit-textarea"
                  spellcheck="false"
                  :placeholder="$t('skills.instructionLabel')"
                />
              </div>
            </template>
          </div>

          <!-- Footer (only in edit mode) -->
          <div
            v-if="isEditMode && skillContent"
            class="modal-footer"
          >
            <button
              class="btn-secondary"
              @click="cancelEdit"
            >
              {{ $t('common.cancel') }}
            </button>
            <button
              class="btn-primary"
              :disabled="isSaving"
              @click="handleSave"
            >
              <Loader2
                v-if="isSaving"
                class="w-4 h-4 animate-spin"
              />
              <Save
                v-else
                class="w-4 h-4"
              />
              <span>{{ $t('skills.saveSkill') }}</span>
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import {
  X,
  Eye,
  Edit3,
  Save,
  Loader2,
  AlertCircle,
  Folder,
  FolderOpen,
  Code2,
  Settings,
  Sparkles,
  Zap,
  Activity,
  Bot,
  Tag,
  User,
  Clock,
  Store,
  Github,
  HardDrive,
  ExternalLink
} from 'lucide-vue-next'
import { marked } from 'marked'
import hljs from 'highlight.js'
import { useUnifiedSkills } from '@/composables/useUnifiedSkills'
import type { UnifiedSkill, SkillContent, Platform } from '@/types/skills'
import { PLATFORM_CONFIG } from '@/types/skills'

const GeminiIcon = Sparkles

const props = defineProps<{
  modelValue: boolean
  skill?: UnifiedSkill
  initialMode?: 'view' | 'edit'
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'saved'): void
}>()

const { fetchSkillContent, saveSkillContent } = useUnifiedSkills()

// State
const skillContent = ref<SkillContent | null>(null)
const isContentLoading = ref(false)
const contentError = ref<string | null>(null)
const isEditMode = ref(false)
const editBuffer = ref('')
const isSaving = ref(false)
const markdownRef = ref<HTMLElement | null>(null)

// Platform styling
const platformColor = computed(() => {
  if (!props.skill) return '#A78BFA'
  const config = PLATFORM_CONFIG[props.skill.platform as Platform]
  return config?.color || '#A78BFA'
})

const platformIcon = computed(() => {
  const iconMap: Record<string, any> = {
    'claude-code': Code2,
    'codex': Settings,
    'gemini': GeminiIcon,
    'qwen': Zap,
    'iflow': Activity,
    'droid': Bot
  }
  return iconMap[props.skill?.platform || ''] || Code2
})

// Rendered markdown HTML
const renderedHtml = computed(() => {
  if (!skillContent.value?.content) return ''
  return marked.parse(skillContent.value.content) as string
})

// Watch for modal open
watch(() => props.modelValue, (isOpen) => {
  if (isOpen && props.skill) {
    isEditMode.value = props.initialMode === 'edit'
    contentError.value = null
    skillContent.value = null
    loadContent()
  }
})

// Apply syntax highlighting after markdown renders
watch(renderedHtml, () => {
  if (!isEditMode.value && renderedHtml.value) {
    nextTick(() => {
      if (markdownRef.value) {
        markdownRef.value.querySelectorAll('pre code').forEach((block) => {
          hljs.highlightElement(block as HTMLElement)
        })
      }
    })
  }
})

async function loadContent() {
  if (!props.skill) return

  isContentLoading.value = true
  contentError.value = null

  try {
    skillContent.value = await fetchSkillContent(props.skill.skillDir)
    editBuffer.value = skillContent.value.raw
  } catch (err: any) {
    contentError.value = err.message || 'Failed to load skill content'
  } finally {
    isContentLoading.value = false
  }
}

function toggleMode() {
  if (isEditMode.value) {
    // Switching from edit to view: reset buffer if unchanged
    isEditMode.value = false
  } else {
    // Switching from view to edit
    if (skillContent.value) {
      editBuffer.value = skillContent.value.raw
    }
    isEditMode.value = true
  }
}

function cancelEdit() {
  if (skillContent.value) {
    editBuffer.value = skillContent.value.raw
  }
  isEditMode.value = false
}

async function handleSave() {
  if (!props.skill || !skillContent.value) return

  isSaving.value = true
  try {
    await saveSkillContent(props.skill.skillDir, editBuffer.value)
    // Reload to reflect changes
    skillContent.value = await fetchSkillContent(props.skill.skillDir)
    editBuffer.value = skillContent.value.raw
    isEditMode.value = false
    emit('saved')
  } catch (err: any) {
    contentError.value = err.message || 'Failed to save'
  } finally {
    isSaving.value = false
  }
}

function close() {
  emit('update:modelValue', false)
}

function shortenPath(path: string): string {
  const segments = path.replace(/\\/g, '/').split('/')
  if (segments.length <= 3) return path
  return '.../' + segments.slice(-3).join('/')
}

// Source display helpers
const sourceIconMap: Record<string, any> = {
  marketplace: Store,
  github: Github,
  local: HardDrive,
}

const sourceLabelMap: Record<string, string> = {
  marketplace: 'Marketplace',
  github: 'GitHub',
  local: 'Local',
}

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}
</script>

<style scoped>
.modal-content {
  @apply bg-bg-base border border-border-default rounded-2xl
         shadow-2xl overflow-hidden;
}

.modal-header {
  @apply flex items-center justify-between p-4
         border-b border-border-subtle shrink-0;
}

.modal-icon {
  @apply flex items-center justify-center w-10 h-10 rounded-xl;
}

.modal-body {
  @apply p-4 space-y-4;
}

/* Metadata */
.metadata-section {
  @apply p-4 rounded-xl border border-border-subtle space-y-3;

  background: rgb(var(--color-bg-surface-rgb) / 50%);
}

.meta-grid {
  @apply grid grid-cols-1 sm:grid-cols-3 gap-2;
}

.meta-item {
  @apply flex items-center gap-2 text-sm;
}

.meta-label {
  @apply text-text-muted text-xs;
}

.meta-value {
  @apply text-text-primary font-medium text-xs;
}

.tag-badge {
  @apply px-2 py-0.5 rounded-md text-[10px] font-medium
         bg-bg-surface text-text-muted;
}

/* Content Divider */
.content-divider {
  @apply flex items-center gap-3;
}

.content-divider__label {
  @apply text-xs font-bold uppercase tracking-wide text-text-muted whitespace-nowrap;
}

.content-divider::after {
  content: '';

  @apply flex-1 h-px;

  background: rgb(var(--color-border-subtle-rgb) / 50%);
}

/* Markdown Content */
.markdown-content {
  @apply rounded-xl border border-border-subtle p-4 overflow-x-auto;

  background: rgb(var(--color-bg-elevated-rgb) / 30%);
}

.markdown-content .prose {
  @apply text-sm text-text-primary leading-relaxed max-w-none;
}

.markdown-content .prose :deep(h1) {
  @apply text-xl font-bold text-text-primary mt-4 mb-2;
}

.markdown-content .prose :deep(h2) {
  @apply text-lg font-bold text-text-primary mt-4 mb-2;
}

.markdown-content .prose :deep(h3) {
  @apply text-base font-semibold text-text-primary mt-3 mb-1.5;
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
  @apply px-1.5 py-0.5 rounded text-xs font-mono
         bg-bg-surface text-accent-primary;
}

.markdown-content .prose :deep(pre) {
  @apply my-3 p-3 rounded-lg overflow-x-auto text-xs;

  background: rgb(var(--color-bg-base-rgb) / 80%);
}

.markdown-content .prose :deep(pre code) {
  @apply bg-transparent p-0 text-text-primary;
}

.markdown-content .prose :deep(blockquote) {
  @apply my-3 pl-4 border-l-2 text-text-secondary italic;

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
  @apply w-full my-3 text-xs;
}

.markdown-content .prose :deep(th) {
  @apply px-3 py-2 text-left font-semibold border-b border-border-subtle;
}

.markdown-content .prose :deep(td) {
  @apply px-3 py-2 border-b border-border-subtle;
}

/* Edit Mode */
.edit-content {
  @apply rounded-xl border border-border-subtle overflow-hidden;

  background: rgb(var(--color-bg-elevated-rgb) / 30%);
}

.edit-textarea {
  @apply w-full min-h-[400px] p-4 text-sm font-mono leading-relaxed
         text-text-primary bg-transparent
         border-0 outline-none resize-y;
}

.edit-textarea::placeholder {
  color: rgb(var(--color-text-muted-rgb) / 50%);
}

/* Footer */
.modal-footer {
  @apply flex items-center justify-end gap-2 p-4
         border-t border-border-subtle shrink-0;

  background: rgb(var(--color-bg-surface-rgb) / 30%);
}

.btn-secondary {
  @apply px-4 py-2 rounded-xl text-sm font-semibold
         text-text-secondary hover:text-text-primary
         hover:bg-bg-elevated transition-colors;
}

.btn-primary {
  @apply flex items-center gap-2 px-4 py-2 rounded-xl
         text-sm font-bold text-white
         bg-accent-primary
         disabled:opacity-50 disabled:cursor-not-allowed
         transition-colors;
}

.btn-primary:hover {
  background: rgb(var(--color-accent-primary-rgb) / 90%);
}

.btn-retry {
  @apply px-4 py-2 rounded-xl text-sm font-semibold
         bg-accent-primary text-white hover:bg-accent-primary/90
         transition-colors;
}

/* Modal Transition */
.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: opacity 0.2s ease;
}

.modal-fade-enter-active .modal-content,
.modal-fade-leave-active .modal-content {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}

.modal-fade-enter-from .modal-content,
.modal-fade-leave-to .modal-content {
  transform: scale(0.95);
  opacity: 0;
}
</style>
