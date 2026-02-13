<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div
        v-if="modelValue"
        class="add-skill-overlay"
        @click.self="close"
      >
        <Transition name="modal-scale">
          <div
            v-if="modelValue"
            class="add-skill-modal"
          >
            <!-- Header -->
            <div class="add-skill-modal__header">
              <h2 class="add-skill-modal__title">
                <Plus class="w-5 h-5" />
                {{ $t('skills.addSkill') }}
              </h2>
              <button
                class="add-skill-modal__close"
                @click="close"
              >
                <X class="w-5 h-5" />
              </button>
            </div>

            <!-- Tab 切换 -->
            <div class="add-skill-modal__tabs">
              <button
                v-for="tab in tabs"
                :key="tab.id"
                class="add-skill-tab"
                :class="{ 'add-skill-tab--active': activeSource === tab.id }"
                @click="activeSource = tab.id"
              >
                <component
                  :is="tab.icon"
                  class="w-4 h-4"
                />
                <span>{{ $t(tab.label) }}</span>
              </button>
            </div>

            <!-- Tab 内容 -->
            <div class="add-skill-modal__body">
              <!-- 市场 Tab -->
              <div
                v-if="activeSource === 'marketplace'"
                class="tab-content"
              >
                <div class="input-group">
                  <Search class="input-icon" />
                  <input
                    v-model="marketplaceQuery"
                    type="text"
                    class="text-input"
                    :placeholder="$t('skills.searchMarketplace')"
                    @input="onMarketplaceSearch"
                  >
                </div>
                <p class="tab-hint">
                  {{ $t('skills.marketplaceHint') }}
                </p>
              </div>

              <!-- GitHub Tab -->
              <div
                v-if="activeSource === 'github'"
                class="tab-content"
              >
                <div class="input-group">
                  <Github class="input-icon" />
                  <input
                    v-model="githubUrl"
                    type="text"
                    class="text-input"
                    :placeholder="$t('skills.githubUrlPlaceholder')"
                  >
                </div>
                <p class="tab-hint">
                  {{ $t('skills.githubFormats') }}
                </p>
              </div>

              <!-- 本地 Tab -->
              <div
                v-if="activeSource === 'local'"
                class="tab-content"
              >
                <div class="input-group">
                  <FolderOpen class="input-icon" />
                  <input
                    v-model="localPath"
                    type="text"
                    class="text-input"
                    :placeholder="$t('skills.localPathPlaceholder')"
                  >
                  <button
                    v-if="isTauri"
                    class="browse-btn"
                    @click="handleBrowse"
                  >
                    <Folder class="w-4 h-4" />
                    {{ $t('skills.browse') }}
                  </button>
                </div>
                <p class="tab-hint">
                  {{ $t('skills.localHint') }}
                </p>
              </div>

              <!-- npx Tab -->
              <div
                v-if="activeSource === 'npx'"
                class="tab-content"
              >
                <div class="npx-status">
                  <div
                    class="npx-indicator"
                    :class="npxAvailable ? 'npx-indicator--ok' : 'npx-indicator--no'"
                  />
                  <span class="text-xs">
                    {{ npxAvailable ? $t('skills.npxAvailable') : $t('skills.npxNotAvailable') }}
                    <span
                      v-if="npxVersion"
                      class="text-text-muted"
                    >(v{{ npxVersion }})</span>
                  </span>
                </div>
                <div class="input-group">
                  <Zap class="input-icon" />
                  <input
                    v-model="npxPackage"
                    type="text"
                    class="text-input"
                    :placeholder="$t('skills.npxPackagePlaceholder')"
                  >
                </div>
                <label class="checkbox-label">
                  <input
                    v-model="npxGlobal"
                    type="checkbox"
                    class="checkbox-input"
                  >
                  <span>{{ $t('skills.npxGlobal') }}</span>
                </label>
                <p class="tab-hint">
                  {{ $t('skills.npxHint') }}
                </p>
              </div>

              <!-- 目标平台选择 -->
              <div class="platform-section">
                <div class="platform-section__header">
                  <h3 class="platform-section__title">
                    {{ $t('skills.targetPlatforms') }}
                  </h3>
                  <div class="platform-section__actions">
                    <button
                      class="platform-action"
                      @click="selectDetected"
                    >
                      {{ $t('skills.selectDetected') }}
                    </button>
                    <button
                      class="platform-action"
                      @click="selectedPlatforms = []"
                    >
                      {{ $t('skills.clearAll') }}
                    </button>
                  </div>
                </div>
                <div class="platform-grid">
                  <label
                    v-for="p in platforms"
                    :key="p.id"
                    class="platform-item"
                    :class="{ 'platform-item--disabled': !p.detected }"
                  >
                    <input
                      v-model="selectedPlatforms"
                      type="checkbox"
                      :value="p.id"
                      class="checkbox-input"
                    >
                    <span class="platform-item__name">{{ p.display_name }}</span>
                    <span
                      v-if="!p.detected"
                      class="platform-item__badge"
                    >{{ $t('skills.notDetected') }}</span>
                  </label>
                </div>
              </div>
            </div>

            <!-- Footer -->
            <div class="add-skill-modal__footer">
              <button
                class="btn-cancel"
                @click="close"
              >
                {{ $t('common.cancel') }}
              </button>
              <button
                class="btn-install"
                :disabled="!canInstall || isInstalling"
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
                  {{ isInstalling
                    ? $t('skills.installing')
                    : $t('skills.installTo', { count: selectedPlatforms.length })
                  }}
                </span>
              </button>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  Plus, X, Search, Github, FolderOpen, Folder, Zap, Download, Loader2, Store
} from 'lucide-vue-next'
import { useUnifiedSkills } from '@/composables/useUnifiedSkills'
import type { ImportSource, PlatformSummary } from '@/types/skills'

const props = defineProps<{
  modelValue: boolean
  platforms: PlatformSummary[]
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'installed'): void
}>()

const { t: _t } = useI18n()
const {
  importFromGithub,
  importFromLocal,
  importViaNpx,
  checkNpxStatus,
  browseFolder,
  npxStatus,
  setInstalling,
  setInstallProgress
} = useUnifiedSkills()

// Tab 定义
const tabs = [
  { id: 'marketplace' as ImportSource, label: 'skills.tabMarketplace', icon: Store },
  { id: 'github' as ImportSource, label: 'skills.github', icon: Github },
  { id: 'local' as ImportSource, label: 'skills.local', icon: FolderOpen },
  { id: 'npx' as ImportSource, label: 'skills.npx', icon: Zap },
]

// 状态
const activeSource = ref<ImportSource>('marketplace')
const selectedPlatforms = ref<string[]>([])
const isInstalling = ref(false)

// 市场搜索
const marketplaceQuery = ref('')

// GitHub 输入
const githubUrl = ref('')

// 本地路径
const localPath = ref('')

// npx 输入
const npxPackage = ref('')
const npxGlobal = ref(false)

// Tauri 环境检测
const isTauri = computed(() => !!(window as any).__TAURI__)

// npx 状态
const npxAvailable = computed(() => npxStatus.value?.available ?? false)
const npxVersion = computed(() => npxStatus.value?.version)

// 是否可安装
const canInstall = computed(() => {
  if (selectedPlatforms.value.length === 0) return false

  switch (activeSource.value) {
    case 'marketplace':
      return marketplaceQuery.value.trim().length > 0
    case 'github':
      return githubUrl.value.trim().length > 0
    case 'local':
      return localPath.value.trim().length > 0
    case 'npx':
      return npxPackage.value.trim().length > 0
    default:
      return false
  }
})

// 自动选中已检测平台
function selectDetected() {
  selectedPlatforms.value = props.platforms
    .filter(p => p.detected)
    .map(p => p.id)
}

// 弹窗打开时初始化
watch(() => props.modelValue, (open) => {
  if (open) {
    selectDetected()
    checkNpxStatus()
  }
})

function close() {
  emit('update:modelValue', false)
}

async function handleBrowse() {
  const path = await browseFolder()
  if (path) {
    localPath.value = path
  }
}

function onMarketplaceSearch() {
  // 防抖在父组件处理
}

async function handleInstall() {
  isInstalling.value = true
  setInstalling(true)

  try {
    switch (activeSource.value) {
      case 'github': {
        setInstallProgress({
          phase: 'downloading',
          package: githubUrl.value,
          startedAt: Date.now()
        })
        await importFromGithub({
          url: githubUrl.value.trim(),
          agents: selectedPlatforms.value,
          force: false
        })
        setInstallProgress({
          phase: 'done',
          package: githubUrl.value,
          startedAt: Date.now()
        })
        break
      }
      case 'local': {
        setInstallProgress({
          phase: 'installing',
          package: localPath.value,
          startedAt: Date.now()
        })
        await importFromLocal({
          sourcePath: localPath.value.trim(),
          agents: selectedPlatforms.value
        })
        setInstallProgress({
          phase: 'done',
          package: localPath.value,
          startedAt: Date.now()
        })
        break
      }
      case 'npx': {
        setInstallProgress({
          phase: 'downloading',
          package: npxPackage.value,
          startedAt: Date.now()
        })
        await importViaNpx({
          package: npxPackage.value.trim(),
          agents: selectedPlatforms.value,
          global: npxGlobal.value
        })
        setInstallProgress({
          phase: 'done',
          package: npxPackage.value,
          startedAt: Date.now()
        })
        break
      }
      case 'marketplace': {
        // 市场安装走现有 SkillInstallModal 流程
        break
      }
    }

    emit('installed')
    close()
  } catch (err: any) {
    setInstallProgress({
      phase: 'error',
      package: activeSource.value === 'github' ? githubUrl.value :
               activeSource.value === 'local' ? localPath.value :
               npxPackage.value,
      message: err.message || 'Installation failed',
      startedAt: Date.now()
    })
  } finally {
    isInstalling.value = false
    setInstalling(false)
  }
}
</script>

<style scoped>
.add-skill-overlay {
  @apply fixed inset-0 z-50 flex items-center justify-center
         bg-black/50 backdrop-blur-sm;
}

.add-skill-modal {
  @apply flex flex-col w-full max-w-lg mx-4 rounded-2xl
         border border-border-subtle shadow-2xl
         overflow-hidden;

  max-height: 85vh;
  background: rgb(var(--color-bg-base-rgb));
}

.add-skill-modal__header {
  @apply flex items-center justify-between px-6 py-4
         border-b border-border-subtle;
}

.add-skill-modal__title {
  @apply flex items-center gap-2 text-lg font-bold text-text-primary;
}

.add-skill-modal__close {
  @apply p-2 rounded-lg text-text-muted
         hover:text-text-primary hover:bg-bg-surface
         transition-colors;
}

/* Tabs */
.add-skill-modal__tabs {
  @apply flex gap-1 px-6 pt-4 pb-2;
}

.add-skill-tab {
  @apply flex items-center gap-1.5 px-3 py-2 rounded-lg
         text-sm font-medium text-text-secondary
         hover:text-text-primary hover:bg-bg-surface
         transition-all duration-200;
}

.add-skill-tab--active {
  @apply text-text-primary;

  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: rgb(var(--color-accent-primary-rgb));
}

/* Body */
.add-skill-modal__body {
  @apply flex-1 overflow-y-auto px-6 py-4 flex flex-col gap-4;
}

.tab-content {
  @apply flex flex-col gap-3;
}

.input-group {
  @apply relative flex items-center;
}

.input-icon {
  @apply absolute left-3 w-4 h-4 text-text-muted pointer-events-none;
}

.text-input {
  @apply w-full pl-10 pr-4 py-2.5 rounded-xl
         text-sm text-text-primary
         bg-bg-surface border border-border-subtle
         focus:border-accent-primary focus:outline-none
         placeholder:text-text-muted
         transition-colors;
}

.browse-btn {
  @apply ml-2 flex items-center gap-1.5 px-3 py-2.5 rounded-xl shrink-0
         text-sm font-medium text-text-secondary
         bg-bg-surface border border-border-subtle
         hover:border-border-default hover:text-text-primary
         transition-colors;
}

.tab-hint {
  @apply text-xs text-text-muted leading-relaxed;
}

/* npx 状态 */
.npx-status {
  @apply flex items-center gap-2 px-3 py-2 rounded-lg bg-bg-surface;
}

.npx-indicator {
  @apply w-2 h-2 rounded-full;
}

.npx-indicator--ok {
  background: rgb(var(--color-success-rgb));
  box-shadow: 0 0 6px rgb(var(--color-success-rgb) / 50%);
}

.npx-indicator--no {
  background: rgb(var(--color-danger-rgb));
}

/* Checkbox */
.checkbox-label {
  @apply flex items-center gap-2 text-sm text-text-secondary cursor-pointer;
}

.checkbox-input {
  @apply rounded border-border-default text-accent-primary
         focus:ring-accent-primary/20;
}

/* Platform 选择 */
.platform-section {
  @apply flex flex-col gap-3 pt-3 border-t border-border-subtle;
}

.platform-section__header {
  @apply flex items-center justify-between;
}

.platform-section__title {
  @apply text-sm font-semibold text-text-primary;
}

.platform-section__actions {
  @apply flex items-center gap-2;
}

.platform-action {
  @apply text-xs text-accent-primary hover:underline cursor-pointer;
}

.platform-grid {
  @apply grid grid-cols-2 gap-2;
}

.platform-item {
  @apply flex items-center gap-2 px-3 py-2 rounded-lg
         bg-bg-surface text-sm cursor-pointer
         hover:bg-bg-elevated transition-colors;
}

.platform-item--disabled {
  @apply opacity-50;
}

.platform-item__name {
  @apply text-text-primary font-medium;
}

.platform-item__badge {
  @apply ml-auto text-[10px] text-text-muted;
}

/* Footer */
.add-skill-modal__footer {
  @apply flex items-center justify-end gap-3 px-6 py-4
         border-t border-border-subtle;
}

.btn-cancel {
  @apply px-4 py-2 rounded-xl text-sm font-medium
         text-text-secondary hover:text-text-primary
         hover:bg-bg-surface transition-colors;
}

.btn-install {
  @apply flex items-center gap-2 px-5 py-2 rounded-xl
         text-sm font-semibold text-white
         bg-accent-primary hover:bg-accent-primary/90
         disabled:opacity-50 disabled:cursor-not-allowed
         transition-all;
}

/* Modal 动画 */
.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: opacity 0.2s ease;
}

.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}

.modal-scale-enter-active,
.modal-scale-leave-active {
  transition: all 0.25s ease;
}

.modal-scale-enter-from {
  opacity: 0;
  transform: scale(0.95);
}

.modal-scale-leave-to {
  opacity: 0;
  transform: scale(0.95);
}
</style>
