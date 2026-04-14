<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div
        v-if="show"
        class="platform-modal-overlay"
        @click.self="closeModal"
      >
        <Transition name="modal-scale">
          <div
            v-if="show"
            class="platform-modal"
          >
            <div class="platform-modal__header">
              <div>
                <h3 class="platform-modal__title">
                  {{ modalTitle }}
                </h3>
                <p class="platform-modal__subtitle">
                  {{ modalSubtitle }}
                </p>
              </div>
              <button
                class="platform-modal__close"
                @click="closeModal"
              >
                <SIcon
                  name="X"
                  size="w-5 h-5"
                />
              </button>
            </div>

            <div
              v-if="mode === 'single' && pendingItem"
              class="platform-summary"
            >
              <p class="platform-summary__eyebrow">
                {{ $t('skills.installSkill') }}
              </p>
              <p class="platform-summary__title">
                {{ pendingItem.skill || pendingItem.repo }}
              </p>
              <p class="platform-summary__package">
                {{ pendingItem.package }}
              </p>
            </div>

            <div
              v-else-if="mode === 'batch'"
              class="platform-summary"
            >
              <p class="platform-summary__eyebrow">
                {{ $t('skills.batchInstall') }}
              </p>
              <p class="platform-summary__title">
                {{ $t('skills.batchInstallCount', { count: batchPackages.length }) }}
              </p>
              <ul class="platform-summary__list">
                <li
                  v-for="pkg in batchPackages"
                  :key="pkg"
                >
                  {{ pkg }}
                </li>
              </ul>
            </div>

            <div class="platform-section">
              <div class="platform-section__header">
                <h3 class="platform-section__title">
                  {{ $t('skills.selectPlatforms') }}
                </h3>
                <div class="platform-section__actions">
                  <button
                    class="platform-action"
                    @click="selectDetected"
                  >
                    {{ $t('skills.selectAllDetected') }}
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
                    :disabled="!platform.detected"
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
            <div class="platform-modal__footer">
              <button
                class="btn-cancel"
                @click="closeModal"
              >
                {{ $t('common.cancel') }}
              </button>
              <button
                class="btn-install"
                :disabled="selectedPlatforms.length === 0"
                @click="confirmInstall"
              >
                <SIcon
                  name="Download"
                  size="w-4 h-4"
                />
                {{ confirmLabel }}
              </button>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type { MarketplaceItem, PlatformSummary } from '@/types/skills'

interface Props {
  show: boolean
  mode: 'single' | 'batch'
  pendingItem: MarketplaceItem | null
  batchPackages: string[]
  platforms: PlatformSummary[]
  selectedPlatforms: string[]
  closeModal: () => void
  selectDetected: () => void
  updateSelectedPlatforms: (value: string[]) => void
  confirmInstall: () => void
}

const props = defineProps<Props>()
const { t } = useI18n()

const modalTitle = computed(() => {
  return props.mode === 'batch' ? t('skills.batchInstall') : t('skills.installSkill')
})

const modalSubtitle = computed(() => {
  if (props.mode === 'batch') {
    return t('skills.batchInstallCount', { count: props.batchPackages.length })
  }

  return props.pendingItem?.package || ''
})

const confirmLabel = computed(() => {
  return t('skills.installTo', { count: props.selectedPlatforms.length })
})

const toggleSelectedPlatform = (platformId: string) => {
  const next = props.selectedPlatforms.includes(platformId)
    ? props.selectedPlatforms.filter((item) => item !== platformId)
    : [...props.selectedPlatforms, platformId]

  props.updateSelectedPlatforms(next)
}
</script>

<style scoped>
.checkbox-input {
  @apply rounded border-border-default/15 text-accent-primary focus:ring-accent-primary/20;
}

.platform-section {
  @apply flex flex-col gap-3;
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
  @apply cursor-pointer text-xs text-accent-primary hover:underline;
}

.platform-grid {
  @apply grid grid-cols-2 gap-2 sm:grid-cols-3;
}

.platform-item {
  @apply glass-surface flex cursor-pointer items-center gap-2 rounded-lg px-3 py-2 text-sm transition-colors hover:bg-bg-surface/70;
}

.platform-item--disabled {
  @apply opacity-50;
}

.platform-item__name {
  @apply text-white font-medium;
}

.platform-item__badge {
  @apply ml-auto text-[10px] text-text-muted;
}

.platform-summary {
  @apply glass-surface flex flex-col gap-2 rounded-2xl border border-border-default/10 px-4 py-4;
}

.platform-summary__eyebrow {
  @apply text-xs font-semibold uppercase tracking-wide text-accent-primary;
}

.platform-summary__title {
  @apply text-base font-semibold text-white;
}

.platform-summary__package {
  @apply truncate font-mono text-sm text-text-secondary;
}

.platform-summary__list {
  @apply max-h-32 list-disc space-y-1 overflow-y-auto pl-4 text-sm text-text-primary;
}

.btn-install {
  @apply flex items-center gap-2 rounded-xl bg-accent-primary px-5 py-2.5 text-sm font-semibold text-white transition-colors disabled:cursor-not-allowed disabled:opacity-50 hover:bg-accent-primary/90;
}

.btn-cancel {
  @apply rounded-xl px-4 py-2 text-sm font-medium text-text-primary transition-colors hover:bg-bg-surface/70 hover:text-white;
}

.platform-modal-overlay {
  @apply fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-md;
}

.platform-modal {
  @apply flex w-full max-w-xl flex-col gap-4 mx-4 rounded-2xl border border-border-default/10 p-6 shadow-2xl;

  background: rgb(var(--color-bg-base-rgb));
}

.platform-modal__header {
  @apply flex items-start justify-between gap-4;
}

.platform-modal__title {
  @apply text-lg font-bold text-white;
}

.platform-modal__subtitle {
  @apply mt-1 text-sm text-text-muted;
}

.platform-modal__close {
  @apply rounded-lg p-2 text-text-muted transition-colors hover:bg-bg-surface/70 hover:text-white;
}

.platform-modal__footer {
  @apply flex items-center justify-end gap-3 border-t border-border-default/10 pt-3;
}

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

