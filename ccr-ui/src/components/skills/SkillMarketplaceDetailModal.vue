<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div
        v-if="show"
        class="marketplace-detail-overlay"
        @click.self="close"
      >
        <Transition name="modal-scale">
          <div
            v-if="show && item"
            class="marketplace-detail-modal"
          >
            <div class="marketplace-detail__header">
              <div class="marketplace-detail__title-wrap">
                <div class="marketplace-detail__eyebrow">
                  <SIcon
                    name="Store"
                    size="w-4 h-4"
                  />
                  <span>{{ $t('skills.marketplaceDetailTitle') }}</span>
                </div>
                <h3 class="marketplace-detail__title">
                  {{ displayName }}
                </h3>
                <p class="marketplace-detail__subtitle">
                  {{ packageName }}
                </p>
              </div>
              <button
                class="marketplace-detail__close"
                @click="close"
              >
                <SIcon
                  name="X"
                  size="w-5 h-5"
                />
              </button>
            </div>

            <div class="marketplace-detail__content">
              <div class="marketplace-detail__meta">
                <div class="marketplace-detail__owner">
                  <img
                    v-if="avatarUrl"
                    :src="avatarUrl"
                    :alt="ownerName"
                    class="marketplace-detail__avatar"
                  >
                  <div>
                    <p class="marketplace-detail__label">
                      {{ $t('skills.author') }}
                    </p>
                    <p class="marketplace-detail__value">
                      {{ ownerName }}
                    </p>
                  </div>
                </div>

                <div
                  v-if="stars != null"
                  class="marketplace-detail__stat"
                >
                  <SIcon
                    name="Star"
                    size="w-4 h-4"
                  />
                  <span>{{ formatStars(stars) }}</span>
                </div>

                <div class="marketplace-detail__status">
                  <span
                    v-if="isInstalled"
                    class="marketplace-detail__badge marketplace-detail__badge--installed"
                  >
                    {{ $t('skills.installed') }}
                  </span>
                  <span
                    v-else
                    class="marketplace-detail__badge"
                  >
                    {{ $t('skills.marketplaceNotInstalled') }}
                  </span>
                </div>
              </div>

              <div class="marketplace-detail__section">
                <p class="marketplace-detail__label">
                  {{ $t('skills.description') }}
                </p>
                <p class="marketplace-detail__description">
                  {{ description || $t('skills.search.noDescription') }}
                </p>
              </div>

              <div class="marketplace-detail__links">
                <a
                  :href="githubUrl"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="marketplace-detail__link"
                >
                  <SIcon
                    name="Github"
                    size="w-4 h-4"
                  />
                  <span>{{ $t('skills.viewSource') }}</span>
                </a>
                <a
                  v-if="skillsShUrl"
                  :href="skillsShUrl"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="marketplace-detail__link"
                >
                  <SIcon
                    name="ExternalLink"
                    size="w-4 h-4"
                  />
                  <span>skills.sh</span>
                </a>
              </div>
            </div>

            <div class="marketplace-detail__footer">
              <button
                class="btn-secondary"
                @click="close"
              >
                {{ $t('common.close') }}
              </button>
              <button
                v-if="!isInstalled"
                class="btn-primary"
                :disabled="installDisabled"
                @click="handleInstall"
              >
                <SIcon
                  name="Download"
                  size="w-4 h-4"
                />
                <span>{{ $t('skills.install') }}</span>
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
import SIcon from '@/components/ui/SIcon.vue'
import type { MarketplaceItem } from '@/types/skills'

const props = defineProps<{
  show: boolean
  item: MarketplaceItem | null
  isInstalled: boolean
  installDisabled: boolean
}>()

const emit = defineEmits<{
  close: []
  install: [item: MarketplaceItem]
}>()

const avatarUrl = computed(() => props.item?.authorAvatar || '')
const displayName = computed(() => props.item?.skill || props.item?.repo || '')
const packageName = computed(() => props.item?.package || '')
const ownerName = computed(() => props.item?.owner || '')
const stars = computed(() => props.item?.stars)
const description = computed(() => props.item?.description || '')
const skillsShUrl = computed(() => props.item?.skillsShUrl || '')
const githubUrl = computed(() => {
  if (!props.item) return '#'
  return `https://github.com/${props.item.owner}/${props.item.repo}`
})

const close = () => emit('close')

function handleInstall() {
  if (!props.item) {
    return
  }

  emit('install', props.item)
}

function formatStars(stars: number): string {
  if (stars >= 1_000_000) return `${(stars / 1_000_000).toFixed(1)}M`
  if (stars >= 1000) return `${(stars / 1000).toFixed(1)}k`
  return stars.toString()
}
</script>

<style scoped>
.marketplace-detail-overlay {
  @apply fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 backdrop-blur-md;
}

.marketplace-detail-modal {
  @apply flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden rounded-3xl border border-border-default/15 shadow-2xl;

  background: rgb(var(--color-bg-base-rgb));
}

.marketplace-detail__header,
.marketplace-detail__footer {
  @apply flex items-center justify-between gap-3 border-border-default/10 px-6 py-4;
}

.marketplace-detail__header {
  @apply border-b;
}

.marketplace-detail__footer {
  @apply border-t;
}

.marketplace-detail__title-wrap {
  @apply min-w-0;
}

.marketplace-detail__eyebrow {
  @apply mb-2 flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-accent-primary;
}

.marketplace-detail__title {
  @apply truncate text-2xl font-bold text-white;
}

.marketplace-detail__subtitle {
  @apply mt-1 truncate font-mono text-sm text-text-muted;
}

.marketplace-detail__close {
  @apply rounded-xl p-2 text-text-muted transition-colors hover:bg-bg-surface/70 hover:text-white;
}

.marketplace-detail__content {
  @apply flex flex-col gap-5 overflow-y-auto px-6 py-5;
}

.marketplace-detail__meta {
  @apply grid gap-3 md:grid-cols-[1fr_auto_auto];
}

.marketplace-detail__owner,
.marketplace-detail__stat,
.marketplace-detail__status,
.marketplace-detail__section,
.marketplace-detail__links {
  @apply rounded-2xl border border-border-default/10 bg-black/20;
}

.marketplace-detail__owner {
  @apply flex items-center gap-3 px-4 py-3;
}

.marketplace-detail__avatar {
  @apply h-10 w-10 rounded-full bg-bg-elevated/80;
}

.marketplace-detail__stat,
.marketplace-detail__status {
  @apply flex items-center justify-center gap-2 px-4 py-3 text-sm font-semibold text-white;
}

.marketplace-detail__stat {
  color: rgb(var(--color-warning-rgb));
}

.marketplace-detail__badge {
  @apply rounded-full bg-bg-elevated/80 px-3 py-1 text-xs font-semibold text-text-primary;
}

.marketplace-detail__badge--installed {
  color: rgb(var(--color-success-rgb));
  background: rgb(var(--color-success-rgb) / 12%);
}

.marketplace-detail__section {
  @apply px-4 py-4;
}

.marketplace-detail__label {
  @apply mb-1 text-xs font-semibold uppercase tracking-wide text-text-muted;
}

.marketplace-detail__value {
  @apply text-sm font-medium text-white;
}

.marketplace-detail__description {
  @apply text-sm leading-6 text-text-primary;
}

.marketplace-detail__links {
  @apply flex flex-wrap gap-3 px-4 py-4;
}

.marketplace-detail__link {
  @apply flex items-center gap-2 text-sm font-medium text-accent-primary transition-colors hover:text-white;
}

.btn-primary,
.btn-secondary {
  @apply flex items-center gap-2 rounded-xl px-4 py-2.5 text-sm font-semibold transition-colors;
}

.btn-primary {
  @apply bg-accent-primary text-white hover:bg-accent-primary/90 disabled:cursor-not-allowed disabled:opacity-50;
}

.btn-secondary {
  @apply text-text-primary hover:bg-bg-surface/70 hover:text-white;
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
  transition: all 0.2s ease;
}

.modal-scale-enter-from,
.modal-scale-leave-to {
  opacity: 0;
  transform: scale(0.96);
}
</style>

