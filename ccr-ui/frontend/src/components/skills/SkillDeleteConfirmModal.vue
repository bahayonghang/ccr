<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div
        v-if="modelValue && skill"
        class="fixed inset-0 z-50 flex items-center justify-center p-4"
        @click.self="close"
      >
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" />

        <!-- Modal Content -->
        <div class="modal-content relative w-full max-w-md">
          <!-- Warning Icon -->
          <div class="modal-warning-icon">
            <div class="warning-icon-bg">
              <AlertTriangle class="w-8 h-8" />
            </div>
          </div>

          <!-- Title -->
          <h2 class="modal-title">
            {{ $t('skills.deleteConfirmTitle') }}
          </h2>

          <!-- Skill Info -->
          <div class="skill-info">
            <div class="skill-info__name">
              {{ skill.name }}
            </div>
            <div class="skill-info__meta">
              <span class="skill-info__platform">
                <component
                  :is="platformIcon"
                  class="w-3.5 h-3.5"
                  :style="{ color: platformColor }"
                />
                {{ skill.platformName }}
              </span>
              <span
                v-if="skill.skillDir"
                class="skill-info__dir"
                :title="skill.skillDir"
              >
                <FolderOpen class="w-3.5 h-3.5" />
                {{ shortenPath(skill.skillDir) }}
              </span>
            </div>
          </div>

          <!-- Warning Message -->
          <div class="warning-message">
            <p class="warning-message__desc">
              {{ $t('skills.deleteDescription', { platform: skill.platformName }) }}
            </p>
            <p class="warning-message__danger">
              {{ $t('skills.deleteWarning') }}
            </p>
          </div>

          <!-- Action Buttons -->
          <div class="modal-actions">
            <!-- Keep Skill - Primary, large, emphasized -->
            <button
              class="btn-keep"
              @click="close"
            >
              <Shield class="w-4 h-4" />
              {{ $t('skills.keepSkill') }}
            </button>

            <!-- Delete - Secondary, small, de-emphasized -->
            <button
              class="btn-delete"
              @click="handleConfirm"
            >
              {{ $t('skills.confirmDeleteBtn') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  AlertTriangle,
  Shield,
  FolderOpen,
  Code2,
  Settings,
  Sparkles,
  Zap,
  Activity,
  Bot
} from 'lucide-vue-next'
import type { UnifiedSkill, Platform } from '@/types/skills'
import { PLATFORM_CONFIG } from '@/types/skills'

const props = defineProps<{
  modelValue: boolean
  skill: UnifiedSkill | undefined
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'confirm', skill: UnifiedSkill): void
}>()

const platformColor = computed(() => {
  if (!props.skill) return '#A78BFA'
  const config = PLATFORM_CONFIG[props.skill.platform as Platform]
  return config?.color || '#A78BFA'
})

const platformIcon = computed(() => {
  const iconMap: Record<string, any> = {
    'claude-code': Code2,
    'codex': Settings,
    'gemini': Sparkles,
    'qwen': Zap,
    'iflow': Activity,
    'droid': Bot
  }
  return iconMap[props.skill?.platform || ''] || Code2
})

function close() {
  emit('update:modelValue', false)
}

function handleConfirm() {
  if (props.skill) {
    emit('confirm', props.skill)
  }
}

function shortenPath(path: string): string {
  const segments = path.replace(/\\/g, '/').split('/')
  if (segments.length <= 3) return path
  return '.../' + segments.slice(-3).join('/')
}
</script>

<style scoped>
.modal-content {
  @apply flex flex-col items-center gap-4 p-6 rounded-2xl
         backdrop-blur-xl
         border border-border-subtle
         shadow-2xl;

  background: rgb(var(--color-bg-base-rgb) / 95%);
}

/* Warning icon */
.modal-warning-icon {
  @apply flex items-center justify-center;
}

.warning-icon-bg {
  @apply flex items-center justify-center w-16 h-16 rounded-full;

  background: rgb(var(--color-danger-rgb) / 12%);
  color: rgb(var(--color-danger-rgb));
}

/* Title */
.modal-title {
  @apply text-lg font-bold text-text-primary text-center;
}

/* Skill info card */
.skill-info {
  @apply w-full flex flex-col gap-2 p-3 rounded-xl
         border border-border-subtle;

  background: rgb(var(--color-bg-surface-rgb) / 50%);
}

.skill-info__name {
  @apply text-base font-bold text-text-primary text-center;
}

.skill-info__meta {
  @apply flex flex-col items-center gap-1.5;
}

.skill-info__platform {
  @apply flex items-center gap-1.5 text-xs text-text-secondary;
}

.skill-info__dir {
  @apply flex items-center gap-1.5 text-xs text-text-muted font-mono truncate max-w-full;
}

/* Warning message */
.warning-message {
  @apply w-full flex flex-col items-center gap-1 text-center;
}

.warning-message__desc {
  @apply text-sm text-text-secondary;
}

.warning-message__danger {
  @apply text-sm font-bold;

  color: rgb(var(--color-danger-rgb));
}

/* Action buttons */
.modal-actions {
  @apply w-full flex items-center gap-3 mt-2;
}

/* Keep button - large, primary, emphasized */
.btn-keep {
  @apply flex-1 flex items-center justify-center gap-2
         px-5 py-3 rounded-xl
         text-sm font-bold text-white
         transition-all duration-200;

  background: rgb(var(--color-accent-primary-rgb));
}

.btn-keep:hover {
  background: rgb(var(--color-accent-primary-rgb) / 85%);
  box-shadow: 0 4px 16px rgb(var(--color-accent-primary-rgb) / 30%);
}

/* Delete button - small, de-emphasized */
.btn-delete {
  @apply px-4 py-3 rounded-xl
         text-sm font-medium text-text-muted
         border border-border-subtle
         transition-all duration-200;

  background: transparent;
}

.btn-delete:hover {
  color: rgb(var(--color-danger-rgb));
  border-color: rgb(var(--color-danger-rgb) / 40%);
  background: rgb(var(--color-danger-rgb) / 8%);
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
