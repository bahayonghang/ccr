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
        <div class="modal-content relative w-full max-w-lg">
          <!-- Header -->
          <div class="modal-header">
            <div class="flex items-center gap-3">
              <div class="modal-icon">
                <Download class="w-5 h-5" />
              </div>
              <div>
                <h2 class="text-lg font-bold text-text-primary">
                  {{ $t('skills.installSkill') }}
                </h2>
                <p class="text-sm text-text-secondary">
                  {{ skill?.name || marketplaceItem?.package }}
                </p>
              </div>
            </div>
            <button
              class="p-2 rounded-lg text-text-muted hover:text-text-primary
                     hover:bg-bg-surface transition-colors"
              @click="close"
            >
              <X class="w-5 h-5" />
            </button>
          </div>

          <!-- Body -->
          <div class="modal-body">
            <!-- Skill Info -->
            <div
              v-if="marketplaceItem"
              class="skill-info"
            >
              <div class="flex items-center gap-2 text-sm text-text-secondary">
                <Github class="w-4 h-4" />
                <a
                  :href="`https://github.com/${marketplaceItem.owner}/${marketplaceItem.repo}`"
                  target="_blank"
                  class="hover:text-accent-primary transition-colors"
                >
                  {{ marketplaceItem.owner }}/{{ marketplaceItem.repo }}
                </a>
              </div>
            </div>

            <!-- Platform Selection -->
            <div class="platform-selection">
              <label class="text-sm font-semibold text-text-primary mb-3 block">
                {{ $t('skills.selectPlatforms') }}
              </label>
              <div class="grid grid-cols-2 gap-2">
                <button
                  v-for="platform in availablePlatforms"
                  :key="platform.id"
                  class="platform-option"
                  :class="{
                    'platform-option--selected': selectedPlatforms.includes(platform.id),
                    'platform-option--disabled': !platform.detected
                  }"
                  :disabled="!platform.detected"
                  @click="togglePlatform(platform)"
                >
                  <div class="flex items-center gap-2">
                    <div
                      class="w-3 h-3 rounded-full border-2 transition-all"
                      :class="selectedPlatforms.includes(platform.id)
                        ? 'border-accent-primary bg-accent-primary'
                        : 'border-border-default'"
                    />
                    <component
                      :is="getPlatformIcon(platform.id)"
                      class="w-4 h-4"
                      :style="{ color: getPlatformColor(platform.id) }"
                    />
                    <span>{{ platform.display_name }}</span>
                  </div>
                  <span
                    v-if="!platform.detected"
                    class="text-[10px] text-warning"
                  >
                    {{ $t('skills.notConfigured') }}
                  </span>
                </button>
              </div>
            </div>

            <!-- Quick Actions -->
            <div class="quick-actions">
              <button
                class="quick-action"
                @click="selectAllDetected"
              >
                <CheckSquare class="w-4 h-4" />
                {{ $t('skills.selectAllDetected') }}
              </button>
              <button
                class="quick-action"
                @click="selectedPlatforms = []"
              >
                <Square class="w-4 h-4" />
                {{ $t('common.clearSelection') }}
              </button>
            </div>

            <!-- Error Message -->
            <div
              v-if="error"
              class="error-message"
            >
              <AlertCircle class="w-4 h-4" />
              <span>{{ error }}</span>
            </div>
          </div>

          <!-- Footer -->
          <div class="modal-footer">
            <button
              class="btn-secondary"
              @click="close"
            >
              {{ $t('common.cancel') }}
            </button>
            <button
              class="btn-primary"
              :disabled="selectedPlatforms.length === 0 || isInstalling"
              @click="handleInstall"
            >
              <Loader2
                v-if="isInstalling"
                class="w-4 h-4 animate-spin"
              />
              <Download
                v-else
                class="w-4 h-4"
              />
              <span>
                {{ isInstalling ? $t('skills.installing') : $t('skills.installTo', { count: selectedPlatforms.length }) }}
              </span>
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  Download,
  X,
  Github,
  CheckSquare,
  Square,
  AlertCircle,
  Loader2,
  Code2,
  Settings,
  Sparkles,
  Zap,
  Activity,
  Bot
} from 'lucide-vue-next'
import type { UnifiedSkill, MarketplaceItem, Platform, PlatformSummary } from '@/types/skills'
import { PLATFORM_CONFIG } from '@/types/skills'

const props = defineProps<{
  modelValue: boolean
  skill?: UnifiedSkill
  marketplaceItem?: MarketplaceItem
  platforms: PlatformSummary[]
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'install', platforms: Platform[]): void
}>()

const selectedPlatforms = ref<Platform[]>([])
const isInstalling = ref(false)
const error = ref<string | null>(null)

const availablePlatforms = computed(() => {
  return props.platforms
})

watch(() => props.modelValue, (isOpen) => {
  if (isOpen) {
    // Auto-select all detected platforms
    selectedPlatforms.value = props.platforms
      .filter(p => p.detected)
      .map(p => p.id as Platform)
    error.value = null
  }
})

function close() {
  emit('update:modelValue', false)
}

function togglePlatform(platform: PlatformSummary) {
  const id = platform.id as Platform
  const index = selectedPlatforms.value.indexOf(id)
  if (index === -1) {
    selectedPlatforms.value.push(id)
  } else {
    selectedPlatforms.value.splice(index, 1)
  }
}

function selectAllDetected() {
  selectedPlatforms.value = props.platforms
    .filter(p => p.detected)
    .map(p => p.id as Platform)
}

function getPlatformColor(platformId: string): string {
  const config = PLATFORM_CONFIG[platformId as Platform]
  return config?.color || '#A78BFA'
}

function getPlatformIcon(platformId: string) {
  const iconMap: Record<string, any> = {
    'claude-code': Code2,
    'codex': Settings,
    'gemini': Sparkles,
    'qwen': Zap,
    'iflow': Activity,
    'droid': Bot
  }
  return iconMap[platformId] || Code2
}

async function handleInstall() {
  if (selectedPlatforms.value.length === 0) return

  isInstalling.value = true
  error.value = null

  try {
    emit('install', [...selectedPlatforms.value])
    close()
  } catch (err: any) {
    error.value = err.message || 'Installation failed'
  } finally {
    isInstalling.value = false
  }
}
</script>

<style scoped>
.modal-content {
  @apply bg-bg-base border border-border-default rounded-2xl
         shadow-2xl overflow-hidden;
}

.modal-header {
  @apply flex items-center justify-between p-4
         border-b border-border-subtle;
}

.modal-icon {
  @apply flex items-center justify-center w-10 h-10 rounded-xl
         text-accent-primary;

  background: rgb(var(--color-accent-primary-rgb) / 10%);
}

.modal-body {
  @apply p-4 space-y-4;
}

.skill-info {
  @apply p-3 rounded-xl border border-border-subtle;

  background: rgb(var(--color-bg-surface-rgb) / 50%);
}

.platform-selection {
  @apply p-4 rounded-xl border border-border-subtle;

  background: rgb(var(--color-bg-elevated-rgb) / 30%);
}

.platform-option {
  @apply flex items-center justify-between p-3 rounded-xl
         border border-border-subtle
         text-sm text-text-secondary
         transition-all duration-200 cursor-pointer
         hover:border-border-default hover:text-text-primary;

  background: rgb(var(--color-bg-surface-rgb) / 50%);
}

.platform-option--selected {
  @apply text-text-primary;

  border-color: rgb(var(--color-accent-primary-rgb) / 50%);
  background: rgb(var(--color-accent-primary-rgb) / 5%);
}

.platform-option--disabled {
  @apply opacity-50 cursor-not-allowed hover:border-border-subtle;
}

.quick-actions {
  @apply flex gap-2;
}

.quick-action {
  @apply flex items-center gap-2 px-3 py-2 rounded-lg
         text-xs font-medium text-text-secondary
         bg-bg-surface hover:bg-bg-elevated
         transition-colors;
}

.error-message {
  @apply flex items-center gap-2 p-3 rounded-xl text-sm;

  background: rgb(var(--color-danger-rgb) / 10%);
  color: rgb(var(--color-danger-rgb));
}

.modal-footer {
  @apply flex items-center justify-end gap-2 p-4
         border-t border-border-subtle;

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
