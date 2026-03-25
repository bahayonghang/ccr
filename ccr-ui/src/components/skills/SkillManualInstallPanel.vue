<template>
  <section class="manual-section">
    <h2 class="section-title">
      <SIcon
        name="Terminal"
        size="w-5 h-5"
        class="text-accent-primary"
      />
      {{ $t('skills.manualInstall') }}
    </h2>

    <div class="manual-tabs">
      <button
        v-for="tab in manualTabs"
        :key="tab.id"
        class="manual-tab"
        :class="{ 'manual-tab--active': activeSource === tab.id }"
        @click="setActiveSource(tab.id)"
      >
        <SIcon
          :name="tab.icon"
          size="w-4 h-4"
        />
        <span>{{ $t(tab.label) }}</span>
      </button>
    </div>

    <div class="manual-body">
      <div
        v-if="activeSource === 'github'"
        class="tab-content"
      >
        <div class="input-group">
          <SIcon
            name="Github"
            class="input-icon"
          />
          <input
            :value="githubUrl"
            type="text"
            class="text-input"
            :placeholder="$t('skills.githubUrlPlaceholder')"
            @input="updateGithubUrl(($event.target as HTMLInputElement).value)"
          >
        </div>
        <p class="tab-hint">
          {{ $t('skills.githubFormats') }}
        </p>
      </div>

      <div
        v-if="activeSource === 'local'"
        class="tab-content"
      >
        <div class="input-group">
          <SIcon
            name="FolderOpen"
            class="input-icon"
          />
          <input
            :value="localPath"
            type="text"
            class="text-input"
            :placeholder="$t('skills.localPathPlaceholder')"
            @input="updateLocalPath(($event.target as HTMLInputElement).value)"
          >
          <button
            class="browse-btn"
            @click="handleBrowse"
          >
            <SIcon
              name="Folder"
              size="w-4 h-4"
            />
            {{ $t('skills.browse') }}
          </button>
        </div>
        <p class="tab-hint">
          {{ $t('skills.localHint') }}
        </p>
      </div>

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
              class="text-white/50"
            >(v{{ npxVersion }})</span>
          </span>
        </div>
        <div class="input-group">
          <SIcon
            name="Zap"
            class="input-icon"
          />
          <input
            :value="npxPackage"
            type="text"
            class="text-input"
            :placeholder="$t('skills.npxPackagePlaceholder')"
            @input="updateNpxPackage(($event.target as HTMLInputElement).value)"
          >
        </div>
        <label class="checkbox-label">
          <input
            :checked="npxGlobal"
            type="checkbox"
            class="checkbox-input"
            @change="updateNpxGlobal(($event.target as HTMLInputElement).checked)"
          >
          <span>{{ $t('skills.npxGlobal') }}</span>
        </label>
        <p class="tab-hint">
          {{ $t('skills.npxHint') }}
        </p>
      </div>

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
              @click="clearSelectedPlatforms"
            >
              {{ $t('skills.clearAll') }}
            </button>
          </div>
        </div>
        <div class="platform-grid">
          <label
            v-for="platform in platforms"
            :key="platform.id"
            class="platform-item"
            :class="{ 'platform-item--disabled': !platform.detected }"
          >
            <input
              :checked="selectedPlatforms.includes(platform.id)"
              type="checkbox"
              :value="platform.id"
              class="checkbox-input"
              @change="toggleSelectedPlatform(platform.id)"
            >
            <span class="platform-item__name">{{ platform.display_name }}</span>
            <span
              v-if="!platform.detected"
              class="platform-item__badge"
            >{{ $t('skills.notDetected') }}</span>
          </label>
        </div>
      </div>

      <div class="manual-footer">
        <button
          class="btn-install"
          :disabled="!canManualInstall || manualInstalling"
          @click="handleManualInstall"
        >
          <SIcon
            v-if="manualInstalling"
            name="Loader2"
            size="w-4 h-4"
            class="animate-spin"
          />
          <SIcon
            v-else
            name="Download"
            size="w-4 h-4"
          />
          <span>
            {{ manualInstalling
              ? $t('skills.installing')
              : $t('skills.installTo', { count: selectedPlatforms.length })
            }}
          </span>
        </button>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import type { ImportSource, PlatformSummary } from '@/types/skills'

type ManualSource = Exclude<ImportSource, 'marketplace'>

type ManualTab = {
  id: ManualSource
  label: string
  icon: string
}

interface Props {
  activeSource: ManualSource
  manualTabs: ManualTab[]
  githubUrl: string
  localPath: string
  npxPackage: string
  npxGlobal: boolean
  npxAvailable: boolean
  npxVersion?: string
  platforms: PlatformSummary[]
  selectedPlatforms: string[]
  canManualInstall: boolean
  manualInstalling: boolean
  hasDetectedPlatforms: boolean
  noPlatformHint: string
  setActiveSource: (source: ManualSource) => void
  updateGithubUrl: (value: string) => void
  updateLocalPath: (value: string) => void
  updateNpxPackage: (value: string) => void
  updateNpxGlobal: (value: boolean) => void
  updateSelectedPlatforms: (value: string[]) => void
  selectDetected: () => void
  clearSelectedPlatforms: () => void
  handleBrowse: () => void
  handleManualInstall: () => void
}

const props = defineProps<Props>()

const toggleSelectedPlatform = (platformId: string) => {
  const next = props.selectedPlatforms.includes(platformId)
    ? props.selectedPlatforms.filter((item) => item !== platformId)
    : [...props.selectedPlatforms, platformId]

  props.updateSelectedPlatforms(next)
}
</script>

<style scoped>
.manual-section {
  @apply flex flex-col gap-4 p-5 rounded-2xl border border-white/5;

  background: rgb(0 0 0 / 30%);
}

.section-title {
  @apply flex items-center gap-2 text-lg font-bold text-white;
}

.manual-tabs {
  @apply flex gap-1;
}

.manual-tab {
  @apply flex items-center gap-1.5 px-4 py-2.5 rounded-xl
         text-sm font-medium text-white/80
         hover:text-white hover:bg-white/5
         transition-colors duration-200;
}

.manual-tab--active {
  @apply text-white;

  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: rgb(var(--color-accent-primary-rgb));
}

.manual-body {
  @apply flex flex-col gap-4;
}

.tab-content {
  @apply flex flex-col gap-3;
}

.input-group {
  @apply relative flex items-center;
}

.input-icon {
  @apply absolute left-3 w-4 h-4 text-white/50 pointer-events-none;
}

.text-input {
  @apply w-full pl-10 pr-4 py-2.5 rounded-xl
         text-sm text-white
         glass-surface border border-white/5
         focus:border-accent-primary focus:outline-none
         placeholder:text-white/50 transition-colors;
}

.browse-btn {
  @apply ml-2 flex items-center gap-1.5 px-3 py-2.5 rounded-xl shrink-0
         text-sm font-medium text-white/80
         glass-surface border border-white/5
         hover:border-white/10 hover:text-white transition-colors;
}

.tab-hint {
  @apply text-xs text-white/50 leading-relaxed;
}

.npx-status {
  @apply flex items-center gap-2 px-3 py-2 rounded-lg glass-surface;
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

.checkbox-label {
  @apply flex items-center gap-2 text-sm text-white/80 cursor-pointer;
}

.checkbox-input {
  @apply rounded border-white/10 text-accent-primary focus:ring-accent-primary/20;
}

.platform-section {
  @apply flex flex-col gap-3 pt-3 border-t border-white/5;
}

.platform-section__header {
  @apply flex items-center justify-between;
}

.platform-section__title {
  @apply text-sm font-semibold text-white;
}

.platform-section__actions {
  @apply flex items-center gap-2;
}

.platform-action {
  @apply text-xs text-accent-primary hover:underline cursor-pointer;
}

.platform-grid {
  @apply grid grid-cols-2 sm:grid-cols-3 gap-2;
}

.platform-item {
  @apply flex items-center gap-2 px-3 py-2 rounded-lg
         glass-surface text-sm cursor-pointer
         hover:bg-white/5 transition-colors;
}

.platform-item--disabled {
  @apply opacity-50;
}

.platform-item__name {
  @apply text-white font-medium;
}

.platform-item__badge {
  @apply ml-auto text-[10px] text-white/50;
}

.manual-footer {
  @apply flex justify-end pt-3 border-t border-white/5;
}

.btn-install {
  @apply flex items-center gap-2 px-5 py-2.5 rounded-xl
         text-sm font-semibold text-white
         bg-accent-primary hover:bg-accent-primary/90
         disabled:opacity-50 disabled:cursor-not-allowed transition-colors;
}
</style>
