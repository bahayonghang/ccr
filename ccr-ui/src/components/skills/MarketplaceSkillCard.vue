<template>
  <div
    class="mp-card group"
    :class="{
      'mp-card--selected': isSelected,
      'mp-card--installing': isInstalling,
      'mp-card--installed': isInstalled,
      'mp-card--clickable': !batchMode,
    }"
    @click="handleClick"
  >
    <div
      v-if="batchMode"
      class="mp-card__checkbox"
      @click.stop="emit('toggle-batch', item)"
    >
      <div
        class="mp-card__check"
        :class="{ 'mp-card__check--active': isSelected }"
      >
        <SIcon
          v-if="isSelected"
          name="Check"
          size="w-3 h-3"
        />
      </div>
    </div>

    <div class="mp-card__header">
      <div class="mp-card__author">
        <img
          :src="avatarUrl"
          :alt="item.owner"
          class="mp-card__avatar"
          loading="lazy"
          @error="onAvatarError"
        >
        <span class="mp-card__owner">{{ item.owner }}</span>
      </div>
      <div
        v-if="item.stars != null"
        class="mp-card__stars"
      >
        <SIcon
          name="Star"
          size="w-3.5 h-3.5"
        />
        <span>{{ formatStars(item.stars) }}</span>
      </div>
    </div>

    <h3 class="mp-card__name">
      {{ displayName }}
    </h3>

    <p
      v-if="item.description"
      class="mp-card__description"
    >
      {{ item.description }}
    </p>
    <p
      v-else
      class="mp-card__description mp-card__description--empty"
    >
      {{ $t('skills.search.noDescription') }}
    </p>

    <div class="mp-card__footer">
      <button
        class="mp-card__link-btn"
        @click.stop="emit('view-detail', item)"
      >
        <SIcon
          name="ExternalLink"
          size="w-3.5 h-3.5"
        />
        <span>{{ $t('skills.viewDetails') }}</span>
      </button>

      <button
        v-if="isInstalled"
        class="mp-card__status mp-card__status--installed"
        disabled
      >
        <SIcon
          name="CheckCircle"
          size="w-4 h-4"
        />
        <span>{{ $t('skills.installed') }}</span>
      </button>
      <button
        v-else-if="isInstalling"
        class="mp-card__status mp-card__status--installing"
        disabled
      >
        <SIcon
          name="Loader2"
          size="w-4 h-4"
          class="animate-spin"
        />
        <span>{{ $t('skills.installing') }}</span>
      </button>
      <button
        v-else
        class="mp-card__install-btn"
        :disabled="installDisabled"
        @click.stop="emit('install', item)"
      >
        <SIcon
          :name="installDisabled ? 'AlertTriangle' : 'Download'"
          size="w-4 h-4"
        />
        <span>{{ installDisabled ? $t('skills.noPlatformsDetectedShort') : $t('skills.install') }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { computed, ref } from 'vue'
import type { MarketplaceItem } from '@/types/skills'

const props = defineProps<{
  item: MarketplaceItem
  isInstalled?: boolean
  isInstalling?: boolean
  batchMode?: boolean
  isSelected?: boolean
  installDisabled?: boolean
}>()

const emit = defineEmits<{
  (e: 'install', item: MarketplaceItem): void
  (e: 'toggle-batch', item: MarketplaceItem): void
  (e: 'view-detail', item: MarketplaceItem): void
}>()

const avatarFailed = ref(false)

const avatarUrl = computed(() => {
  if (avatarFailed.value) {
    return ''
  }
  return props.item.authorAvatar || `https://avatars.githubusercontent.com/${props.item.owner}?s=64`
})

const displayName = computed(() => {
  return props.item.skill || props.item.repo
})

function formatStars(stars: number): string {
  if (stars >= 1_000_000) {
    return (stars / 1_000_000).toFixed(1) + 'M'
  }
  if (stars >= 1000) {
    return (stars / 1000).toFixed(1) + 'k'
  }
  return stars.toString()
}

function onAvatarError() {
  avatarFailed.value = true
}

function handleClick() {
  if (props.batchMode) {
    emit('toggle-batch', props.item)
    return
  }
  emit('view-detail', props.item)
}
</script>

<style scoped>
.mp-card {
  @apply relative flex flex-col gap-3 overflow-hidden rounded-2xl border border-border-default/15 p-4 text-white
         transition-interactive duration-200 ease-out;

  background: rgb(var(--color-bg-elevated-rgb) / 92%);
  contain: layout paint;
}

.mp-card--clickable {
  @apply cursor-pointer;
}

.mp-card:hover {
  @apply scale-[1.01] border-border-default/25;

  background: rgb(var(--color-bg-surface-rgb) / 96%);
  box-shadow: 0 4px 20px rgb(0 0 0 / 20%);
}

.mp-card--selected {
  border-color: rgb(var(--color-accent-primary-rgb));
  background: rgb(var(--color-accent-primary-rgb) / 5%);
}

.mp-card--installing {
  @apply opacity-80 pointer-events-none;
}

.mp-card--installed {
  @apply opacity-90;
}

.mp-card__checkbox {
  @apply absolute right-3 top-3 z-10;
}

.mp-card__check {
  @apply flex h-5 w-5 cursor-pointer items-center justify-center rounded-md border-2 border-border-default/25 bg-black/20 transition-colors duration-150;
}

.mp-card__check--active {
  @apply border-accent-primary bg-accent-primary text-white;
}

.mp-card__header {
  @apply flex items-center justify-between;
}

.mp-card__author {
  @apply flex min-w-0 items-center gap-2;
}

.mp-card__avatar {
  @apply h-6 w-6 shrink-0 rounded-full bg-bg-elevated/80;
}

.mp-card__owner {
  @apply truncate text-xs font-medium text-text-secondary;
}

.mp-card__stars {
  @apply flex shrink-0 items-center gap-1 text-xs font-medium;

  color: rgb(var(--color-warning-rgb));
}

.mp-card__name {
  @apply truncate text-base font-bold text-white;
}

.mp-card__description {
  @apply flex-1 text-sm leading-relaxed text-text-primary;

  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.mp-card__description--empty {
  @apply italic text-text-muted;
}

.mp-card__footer {
  @apply mt-auto flex items-center justify-between border-t border-border-default/15 pt-2;
}

.mp-card__link-btn {
  @apply flex items-center gap-1.5 text-xs text-text-muted transition-colors hover:text-white;
}

.mp-card__install-btn {
  @apply flex items-center gap-1.5 rounded-lg bg-accent-primary/10 px-3 py-1.5 text-xs font-semibold text-accent-primary transition-colors duration-200 hover:bg-accent-primary hover:text-white disabled:cursor-not-allowed disabled:opacity-60 disabled:hover:bg-accent-primary/10 disabled:hover:text-accent-primary;
}

.mp-card__status {
  @apply flex cursor-default items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-semibold;
}

.mp-card__status--installed {
  color: rgb(var(--color-success-rgb));
  background: rgb(var(--color-success-rgb) / 10%);
}

.mp-card__status--installing {
  @apply bg-bg-elevated/80 text-text-secondary;
}
</style>


