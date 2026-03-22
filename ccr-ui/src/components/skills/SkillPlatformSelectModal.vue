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
              <h3 class="platform-modal__title">
                {{ $t('skills.installSkill') }}
              </h3>
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
            <p class="platform-modal__pkg">
              {{ pendingPackage }}
            </p>
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
                {{ $t('skills.installTo', { count: selectedPlatforms.length }) }}
              </button>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import type { PlatformSummary } from '@/types/skills'

interface Props {
  show: boolean
  pendingPackage: string
  platforms: PlatformSummary[]
  selectedPlatforms: string[]
  closeModal: () => void
  selectDetected: () => void
  updateSelectedPlatforms: (value: string[]) => void
  confirmInstall: () => void
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
.checkbox-input {
  @apply rounded border-white/10 text-accent-primary focus:ring-accent-primary/20;
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

.btn-install {
  @apply flex items-center gap-2 px-5 py-2.5 rounded-xl
         text-sm font-semibold text-white
         bg-accent-primary hover:bg-accent-primary/90
         disabled:opacity-50 disabled:cursor-not-allowed transition-colors;
}

.btn-cancel {
  @apply px-4 py-2 rounded-xl text-sm font-medium
         text-white/80 hover:text-white
         hover:bg-white/5 transition-colors;
}

.platform-modal-overlay {
  @apply fixed inset-0 z-50 flex items-center justify-center
         bg-black/50 backdrop-blur-md;
}

.platform-modal {
  @apply flex flex-col gap-4 w-full max-w-md mx-4 p-6 rounded-2xl
         border border-white/5 shadow-2xl;

  background: rgb(var(--color-bg-base-rgb));
}

.platform-modal__header {
  @apply flex items-center justify-between;
}

.platform-modal__title {
  @apply text-lg font-bold text-white;
}

.platform-modal__close {
  @apply p-2 rounded-lg text-white/50
         hover:text-white hover:bg-white/5 transition-colors;
}

.platform-modal__pkg {
  @apply text-sm text-white/80 font-mono truncate
         px-3 py-2 rounded-lg glass-surface;
}

.platform-modal__footer {
  @apply flex items-center justify-end gap-3 pt-3 border-t border-white/5;
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
