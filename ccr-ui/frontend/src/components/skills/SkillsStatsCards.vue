<template>
  <div class="stats-container">
    <!-- Left: Summary Cards -->
    <div class="stats-summary">
      <!-- Installed Skills Card -->
      <div class="stats-card stats-card--primary">
        <div class="stats-card__icon">
          <Package class="w-5 h-5" />
        </div>
        <div class="stats-card__content">
          <span class="stats-card__value">{{ stats.installed }}</span>
          <span class="stats-card__label">{{ $t('skills.statsInstalled') }}</span>
        </div>
        <div
          v-if="stats.installed > 0"
          class="stats-card__trend"
        >
          <Sparkles class="w-3 h-3" />
        </div>
      </div>

      <!-- Available Skills Card -->
      <div class="stats-card stats-card--secondary">
        <div class="stats-card__icon">
          <Globe class="w-5 h-5" />
        </div>
        <div class="stats-card__content">
          <span class="stats-card__value">
            {{ stats.available > 1000 ? formatNumber(stats.available) : stats.available }}
          </span>
          <span class="stats-card__label">{{ $t('skills.statsAvailable') }}</span>
        </div>
        <div
          v-if="cached"
          class="stats-card__badge"
        >
          <Clock class="w-3 h-3" />
          <span>{{ $t('skills.cached') }}</span>
        </div>
      </div>
    </div>

    <!-- Right: Platform Overview -->
    <div class="platform-overview">
      <div class="platform-overview__header">
        <Cpu class="w-3.5 h-3.5 text-text-muted" />
        <span class="text-xs font-bold uppercase tracking-wide text-text-secondary">
          {{ $t('skills.allPlatformStats') }}
        </span>
        <span class="platform-overview__count">
          {{ stats.activePlatforms }}/{{ stats.totalPlatforms }}
        </span>
      </div>
      <div class="platform-chips">
        <div
          v-for="platform in platformIndicators"
          :key="platform.id"
          class="platform-chip"
          :class="{
            'platform-chip--active': activePlatform === platform.id,
            'platform-chip--all-active': activePlatform === 'all' && platform.active,
            'platform-chip--inactive': !platform.active
          }"
        >
          <component
            :is="getPlatformIcon(platform.id)"
            class="w-3.5 h-3.5"
            :style="{ color: platform.active ? platform.color : undefined }"
          />
          <span class="platform-chip__name">{{ platform.name }}</span>
          <span
            class="platform-chip__count"
            :class="{ 'platform-chip__count--highlight': activePlatform === platform.id }"
          >
            {{ platform.count }}
          </span>
          <span
            v-if="!platform.detected"
            class="platform-chip__status"
          >
            {{ $t('skills.notConfigured') }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  Package, Globe, Cpu, Sparkles, Clock,
  Code2, Settings, Zap, Activity, Bot
} from 'lucide-vue-next'
import type { SkillsStats, PlatformSummary, Platform } from '@/types/skills'
import { PLATFORM_CONFIG } from '@/types/skills'

// Use Sparkles for Gemini icon
const GeminiIcon = Sparkles

const props = defineProps<{
  stats: SkillsStats
  platforms: PlatformSummary[]
  cached?: boolean
  activePlatform?: Platform | 'all'
}>()

const platformIndicators = computed(() => {
  return props.platforms.map(p => ({
    id: p.id,
    name: p.display_name,
    active: p.detected && p.installed_count > 0,
    detected: p.detected,
    color: PLATFORM_CONFIG[p.id as Platform]?.color || '#A78BFA',
    count: p.installed_count
  }))
})

function getPlatformIcon(platformId: string) {
  const iconMap: Record<string, any> = {
    'claude-code': Code2,
    'codex': Settings,
    'gemini': GeminiIcon,
    'qwen': Zap,
    'iflow': Activity,
    'droid': Bot
  }
  return iconMap[platformId] || Code2
}

function formatNumber(num: number): string {
  if (num >= 1000) {
    return (num / 1000).toFixed(1) + 'k'
  }
  return num.toString()
}
</script>

<style scoped>
.stats-container {
  @apply flex gap-4 items-stretch;
}

/* Left summary cards */
.stats-summary {
  @apply flex gap-3 shrink-0;
}

.stats-card {
  @apply relative flex items-center gap-3 px-4 py-3 rounded-xl
         backdrop-blur-sm
         border border-border-subtle
         transition-all duration-300
         hover:border-border-default hover:shadow-lg;

  background: rgb(var(--color-bg-elevated-rgb) / 50%);
  min-width: 140px;
}

.stats-card--primary {
  &:hover {
    border-color: rgb(var(--color-accent-primary-rgb) / 30%);
  }
}

.stats-card--primary .stats-card__icon {
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: rgb(var(--color-accent-primary-rgb));
}

.stats-card--secondary {
  &:hover {
    border-color: rgb(var(--color-accent-secondary-rgb) / 30%);
  }
}

.stats-card--secondary .stats-card__icon {
  background: rgb(var(--color-accent-secondary-rgb) / 10%);
  color: rgb(var(--color-accent-secondary-rgb));
}

.stats-card__icon {
  @apply flex items-center justify-center w-9 h-9 rounded-lg shrink-0;
}

.stats-card__content {
  @apply flex flex-col;
}

.stats-card__value {
  @apply text-xl font-bold text-text-primary leading-tight;
}

.stats-card__label {
  @apply text-[10px] text-text-secondary uppercase tracking-wide;
}

.stats-card__trend {
  @apply absolute top-2 right-2 text-accent-primary animate-pulse;
}

.stats-card__badge {
  @apply absolute top-2 right-2 flex items-center gap-1
         px-1.5 py-0.5 rounded-full text-[9px] font-medium
         bg-bg-surface text-text-muted;
}

/* Right platform overview */
.platform-overview {
  @apply flex-1 flex flex-col gap-2 px-4 py-3 rounded-xl
         backdrop-blur-sm border border-border-subtle;

  background: rgb(var(--color-bg-elevated-rgb) / 50%);
}

.platform-overview__header {
  @apply flex items-center gap-2;
}

.platform-overview__count {
  @apply ml-auto px-1.5 py-0.5 rounded text-[10px] font-bold font-mono
         bg-bg-surface text-text-muted;
}

.platform-chips {
  @apply flex flex-wrap gap-2;
}

.platform-chip {
  @apply flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg
         text-xs font-medium text-text-secondary
         border border-border-subtle
         transition-all duration-200;

  background: rgb(var(--color-bg-surface-rgb) / 50%);
}

.platform-chip--active {
  @apply text-text-primary;

  border-color: rgb(var(--color-accent-primary-rgb) / 40%);
  background: rgb(var(--color-accent-primary-rgb) / 8%);
  box-shadow: 0 0 12px rgb(var(--color-accent-primary-rgb) / 10%);
}

.platform-chip--all-active {
  @apply text-text-primary;

  border-color: rgb(var(--color-success-rgb) / 30%);
  background: rgb(var(--color-bg-surface-rgb) / 80%);
}

.platform-chip--inactive {
  @apply opacity-50;
}

.platform-chip__name {
  @apply whitespace-nowrap;
}

.platform-chip__count {
  @apply px-1.5 py-0.5 rounded text-[10px] font-bold font-mono
         bg-bg-surface text-text-muted;
}

.platform-chip__count--highlight {
  @apply bg-accent-primary/20 text-accent-primary;
}

.platform-chip__status {
  @apply text-[9px] italic;

  color: var(--color-warning);
}

/* Responsive: stack on small screens */
@media (width <= 768px) {
  .stats-container {
    @apply flex-col;
  }

  .stats-summary {
    @apply w-full;
  }

  .stats-card {
    @apply flex-1;

    min-width: auto;
  }
}
</style>
