<template>
  <div
    class="mp-card group"
    :class="{
      'mp-card--selected': isSelected,
      'mp-card--installing': isInstalling,
      'mp-card--installed': isInstalled
    }"
    @click="handleClick"
  >
    <!-- 批量模式复选框 -->
    <div
      v-if="batchMode"
      class="mp-card__checkbox"
      @click.stop="$emit('select', item)"
    >
      <div
        class="mp-card__check"
        :class="{ 'mp-card__check--active': isSelected }"
      >
        <Check
          v-if="isSelected"
          class="w-3 h-3"
        />
      </div>
    </div>

    <!-- 头部: 作者头像 + 名称 + 星标 -->
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
        <Star class="w-3.5 h-3.5" />
        <span>{{ formatStars(item.stars) }}</span>
      </div>
    </div>

    <!-- 标题 -->
    <h3 class="mp-card__name">
      {{ displayName }}
    </h3>

    <!-- 描述 -->
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
      {{ $t('skills.noDescription') }}
    </p>

    <!-- 底部: 源码链接 + 安装按钮 -->
    <div class="mp-card__footer">
      <a
        :href="sourceUrl"
        target="_blank"
        rel="noopener noreferrer"
        class="mp-card__source-link"
        @click.stop
      >
        <Github class="w-3.5 h-3.5" />
        <span>{{ $t('skills.source') }}</span>
      </a>

      <button
        v-if="isInstalled"
        class="mp-card__status mp-card__status--installed"
        disabled
      >
        <CheckCircle class="w-4 h-4" />
        <span>{{ $t('skills.installed') }}</span>
      </button>
      <button
        v-else-if="isInstalling"
        class="mp-card__status mp-card__status--installing"
        disabled
      >
        <Loader2 class="w-4 h-4 animate-spin" />
        <span>{{ $t('skills.installing') }}</span>
      </button>
      <button
        v-else
        class="mp-card__install-btn"
        @click.stop="$emit('install', item)"
      >
        <Download class="w-4 h-4" />
        <span>{{ $t('skills.install') }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import {
  Star,
  Github,
  Download,
  Loader2,
  Check,
  CheckCircle
} from 'lucide-vue-next'
import type { MarketplaceItem } from '@/types/skills'

const props = defineProps<{
  item: MarketplaceItem
  isInstalled?: boolean
  isInstalling?: boolean
  batchMode?: boolean
  isSelected?: boolean
}>()

defineEmits<{
  (e: 'install', item: MarketplaceItem): void
  (e: 'select', item: MarketplaceItem): void
  (e: 'view-source', item: MarketplaceItem): void
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

const sourceUrl = computed(() => {
  return `https://github.com/${props.item.owner}/${props.item.repo}`
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
    // noop — checkbox handles it
  }
}
</script>

<style scoped>
.mp-card {
  @apply relative flex flex-col gap-3 p-4 rounded-2xl cursor-default
         border border-border-subtle
         transition-all duration-200 ease-out
         overflow-hidden;

  background: rgb(var(--color-bg-elevated-rgb) / 50%);
  backdrop-filter: blur(8px);
}

.mp-card:hover {
  @apply border-border-default;

  background: rgb(var(--color-bg-elevated-rgb) / 75%);
  box-shadow: 0 4px 16px rgb(0 0 0 / 8%);
  transform: translateY(-1px);
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

/* 批量复选框 */
.mp-card__checkbox {
  @apply absolute top-3 right-3 z-10;
}

.mp-card__check {
  @apply flex items-center justify-center w-5 h-5 rounded-md
         border-2 border-border-default
         transition-all duration-150 cursor-pointer;
}

.mp-card__check--active {
  @apply border-accent-primary bg-accent-primary text-white;
}

/* 头部 */
.mp-card__header {
  @apply flex items-center justify-between;
}

.mp-card__author {
  @apply flex items-center gap-2 min-w-0;
}

.mp-card__avatar {
  @apply w-6 h-6 rounded-full bg-bg-surface shrink-0;
}

.mp-card__owner {
  @apply text-xs font-medium text-text-secondary truncate;
}

.mp-card__stars {
  @apply flex items-center gap-1 text-xs font-medium shrink-0;

  color: rgb(var(--color-warning-rgb));
}

/* 标题 */
.mp-card__name {
  @apply text-base font-bold text-text-primary truncate;
}

/* 描述 */
.mp-card__description {
  @apply text-sm text-text-secondary leading-relaxed flex-1;

  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.mp-card__description--empty {
  @apply text-text-muted italic;
}

/* 底部 */
.mp-card__footer {
  @apply flex items-center justify-between mt-auto pt-2
         border-t border-border-subtle;
}

.mp-card__source-link {
  @apply flex items-center gap-1.5 text-xs text-text-muted
         hover:text-text-primary transition-colors;
}

.mp-card__install-btn {
  @apply flex items-center gap-1.5 px-3 py-1.5 rounded-lg
         text-xs font-semibold
         bg-accent-primary/10 text-accent-primary
         hover:bg-accent-primary hover:text-white
         transition-all duration-200;
}

.mp-card__status {
  @apply flex items-center gap-1.5 px-3 py-1.5 rounded-lg
         text-xs font-semibold cursor-default;
}

.mp-card__status--installed {
  color: rgb(var(--color-success-rgb));
  background: rgb(var(--color-success-rgb) / 10%);
}

.mp-card__status--installing {
  @apply text-text-secondary bg-bg-surface;
}
</style>
