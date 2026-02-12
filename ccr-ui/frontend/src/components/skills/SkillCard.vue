<template>
  <div
    class="skill-card group"
    @click="$emit('click', skill)"
  >
    <!-- Left: Platform Icon Area -->
    <div class="skill-card__platform">
      <div
        class="skill-card__platform-icon"
        :style="{ backgroundColor: platformColor + '15', color: platformColor }"
      >
        <component
          :is="platformIcon"
          class="w-5 h-5"
        />
      </div>
      <div
        class="skill-card__platform-badge"
        :style="{ backgroundColor: platformColor + '20', color: platformColor }"
      >
        {{ skill.platformName }}
      </div>
    </div>

    <!-- Center: Skill Info Area -->
    <div class="skill-card__body">
      <!-- Row 1: Name + Source + Category -->
      <div class="skill-card__title-row">
        <h3 class="skill-card__name">
          {{ skill.name }}
        </h3>
        <span
          v-if="skill.source && sourceIcon"
          class="skill-card__source-badge"
          :class="`skill-card__source--${skill.source}`"
        >
          <component
            :is="sourceIcon"
            class="w-3 h-3"
          />
          {{ sourceLabel }}
        </span>
        <span
          v-if="skill.category"
          class="skill-card__category"
        >
          <Folder class="w-3 h-3" />
          {{ skill.category }}
        </span>
        <span
          v-else
          class="skill-card__category skill-card__category--empty"
        >
          <Folder class="w-3 h-3" />
          {{ $t('skills.uncategorized') }}
        </span>
      </div>

      <!-- Row 2: Description (expanded) -->
      <p
        v-if="skill.description"
        class="skill-card__description"
      >
        {{ truncateDescription(skill.description) }}
      </p>

      <!-- Row 3: Tags -->
      <div
        v-if="skill.tags && skill.tags.length > 0"
        class="skill-card__tags"
      >
        <span
          v-for="tag in displayTags"
          :key="tag"
          class="skill-card__tag"
        >
          #{{ tag }}
        </span>
        <span
          v-if="skill.tags.length > maxDisplayTags"
          class="skill-card__tag skill-card__tag--more"
        >
          +{{ skill.tags.length - maxDisplayTags }}
        </span>
      </div>

      <!-- Row 4: Meta info (source/version/author/install date/path) -->
      <div class="skill-card__meta">
        <span
          v-if="isMarketplace && marketplaceItem"
          class="skill-card__meta-item"
        >
          <Github class="w-3 h-3" />
          {{ marketplaceItem.owner }}/{{ marketplaceItem.repo }}
        </span>
        <template v-else>
          <span
            v-if="skill.version"
            class="skill-card__meta-item"
          >
            <Tag class="w-3 h-3" />
            v{{ skill.version }}
          </span>
          <span
            v-if="skill.author"
            class="skill-card__meta-item"
          >
            <User class="w-3 h-3" />
            {{ skill.author }}
          </span>
          <span
            v-if="skill.installDate"
            class="skill-card__meta-item"
          >
            <Clock class="w-3 h-3" />
            {{ formatRelativeTime(skill.installDate) }}
          </span>
          <span
            v-if="skill.skillDir && !skill.version && !skill.author && !skill.installDate"
            class="skill-card__meta-item"
            :title="skill.skillDir"
          >
            <FolderOpen class="w-3 h-3" />
            {{ shortenPath(skill.skillDir) }}
          </span>
        </template>
      </div>
    </div>

    <!-- Right: Action Buttons -->
    <div class="skill-card__actions">
      <button
        v-if="!isMarketplace"
        class="skill-card__action"
        :title="$t('common.edit')"
        @click.stop="$emit('edit', skill)"
      >
        <Edit3 class="w-4 h-4" />
      </button>
      <button
        v-if="!isMarketplace"
        class="skill-card__action skill-card__action--danger"
        :title="$t('common.delete')"
        @click.stop="$emit('delete', skill)"
      >
        <Trash2 class="w-4 h-4" />
      </button>
      <button
        v-if="isMarketplace"
        class="skill-card__action skill-card__action--install"
        :title="$t('skills.install')"
        @click.stop="$emit('install', skill)"
      >
        <Download class="w-4 h-4" />
      </button>
    </div>

    <!-- Subtle left accent line (platform color) -->
    <div
      class="skill-card__accent"
      :style="{ '--accent-color': platformColor }"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  Edit3,
  Trash2,
  Download,
  Sparkles,
  Folder,
  FolderOpen,
  Github,
  Code2,
  Settings,
  Zap,
  Activity,
  Bot,
  Tag,
  User,
  Clock,
  Store,
  HardDrive
} from 'lucide-vue-next'
import type { UnifiedSkill, MarketplaceItem, Platform } from '@/types/skills'
import { PLATFORM_CONFIG } from '@/types/skills'

// Use a custom Sparkles icon for Gemini since lucide has it
const GeminiIcon = Sparkles

const props = defineProps<{
  skill: UnifiedSkill
  featured?: boolean
  isMarketplace?: boolean
  marketplaceItem?: MarketplaceItem
}>()

defineEmits<{
  (e: 'click', skill: UnifiedSkill): void
  (e: 'edit', skill: UnifiedSkill): void
  (e: 'delete', skill: UnifiedSkill): void
  (e: 'install', skill: UnifiedSkill): void
}>()

const maxDisplayTags = 8
const maxDescriptionLength = 300

const platformColor = computed(() => {
  const config = PLATFORM_CONFIG[props.skill.platform as Platform]
  return config?.color || '#A78BFA'
})

const platformIcon = computed(() => {
  const iconMap: Record<string, any> = {
    'claude-code': Code2,
    'codex': Settings,
    'gemini': GeminiIcon,
    'qwen': Zap,
    'iflow': Activity,
    'droid': Bot
  }
  return iconMap[props.skill.platform] || Code2
})

const displayTags = computed(() => {
  if (!props.skill.tags) return []
  return props.skill.tags.slice(0, maxDisplayTags)
})

function truncateDescription(desc: string): string {
  if (desc.length <= maxDescriptionLength) return desc
  return desc.substring(0, maxDescriptionLength) + '...'
}

function shortenPath(path: string): string {
  // Show last 2-3 segments of the path for readability
  const segments = path.replace(/\\/g, '/').split('/')
  if (segments.length <= 3) return path
  return '.../' + segments.slice(-3).join('/')
}

// Source display helpers
const sourceIcon = computed(() => {
  switch (props.skill.source) {
    case 'marketplace': return Store
    case 'github': return Github
    case 'local': return HardDrive
    default: return null
  }
})

const sourceLabel = computed(() => {
  switch (props.skill.source) {
    case 'marketplace': return 'Market'
    case 'github': return 'GitHub'
    case 'local': return 'Local'
    default: return ''
  }
})

function formatRelativeTime(timestamp: number): string {
  const now = Date.now()
  const diff = now - timestamp
  const minutes = Math.floor(diff / 60000)
  const hours = Math.floor(diff / 3600000)
  const days = Math.floor(diff / 86400000)
  if (days > 30) return new Date(timestamp).toLocaleDateString()
  if (days > 0) return `${days}d ago`
  if (hours > 0) return `${hours}h ago`
  if (minutes > 0) return `${minutes}m ago`
  return 'just now'
}
</script>

<style scoped>
.skill-card {
  @apply relative flex flex-row items-start gap-4 p-4 pl-5 rounded-2xl cursor-pointer
         backdrop-blur-sm
         border border-border-subtle
         transition-all duration-200 ease-out
         overflow-hidden;

  background: rgb(var(--color-bg-elevated-rgb) / 50%);
}

.skill-card:hover {
  @apply border-border-default;

  background: rgb(var(--color-bg-elevated-rgb) / 75%);
  box-shadow: 0 2px 12px rgb(0 0 0 / 6%);
}

/* Platform column */
.skill-card__platform {
  @apply flex flex-col items-center gap-2 shrink-0 w-16;
}

.skill-card__platform-icon {
  @apply flex items-center justify-center w-12 h-12 rounded-xl;
}

.skill-card__platform-badge {
  @apply px-2 py-0.5 rounded-full
         text-[9px] font-bold uppercase tracking-wide text-center
         whitespace-nowrap;
}

/* Body / info column */
.skill-card__body {
  @apply flex flex-col gap-1.5 flex-1 min-w-0;
}

.skill-card__title-row {
  @apply flex items-center gap-2 flex-wrap;
}

.skill-card__name {
  @apply text-base font-bold text-text-primary;
}

.skill-card__category {
  @apply flex items-center gap-1 px-2 py-0.5 rounded-md
         text-[10px] font-medium text-text-secondary
         bg-bg-surface;
}

.skill-card__category--empty {
  @apply text-text-muted italic;
}

.skill-card__description {
  @apply text-sm text-text-secondary leading-relaxed;
}

.skill-card__tags {
  @apply flex flex-wrap gap-1 mt-1;
}

.skill-card__tag {
  @apply px-2 py-0.5 rounded-md text-[10px] font-medium
         bg-bg-surface text-text-muted;
}

.skill-card__tag--more {
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: rgb(var(--color-accent-primary-rgb));
}

.skill-card__meta {
  @apply mt-1 flex flex-wrap items-center gap-3;
}

.skill-card__meta-item {
  @apply flex items-center gap-1.5 text-xs text-text-muted font-mono;
}

/* Source badge */
.skill-card__source-badge {
  @apply flex items-center gap-1 px-2 py-0.5 rounded-md
         text-[10px] font-semibold;
}

.skill-card__source--marketplace {
  background: rgb(var(--color-success-rgb) / 12%);
  color: rgb(var(--color-success-rgb));
}

.skill-card__source--github {
  background: rgb(var(--color-accent-primary-rgb) / 12%);
  color: rgb(var(--color-accent-primary-rgb));
}

.skill-card__source--local {
  @apply bg-bg-surface text-text-muted;
}

/* Actions column */
.skill-card__actions {
  @apply flex flex-col items-center gap-1 shrink-0
         opacity-50 group-hover:opacity-100
         transition-opacity duration-200;
}

.skill-card__action {
  @apply p-2 rounded-lg text-text-muted
         hover:bg-bg-surface hover:text-text-primary
         transition-colors;
}

.skill-card__action--danger {
  &:hover {
    background: rgb(var(--color-danger-rgb) / 10%);
    color: rgb(var(--color-danger-rgb));
  }
}

.skill-card__action--install {
  &:hover {
    background: rgb(var(--color-success-rgb) / 10%);
    color: rgb(var(--color-success-rgb));
  }
}

/* Subtle left accent line - platform color indicator */
.skill-card__accent {
  @apply absolute left-0 top-3 bottom-3 w-[3px] rounded-full
         transition-all duration-200 pointer-events-none;

  background: var(--accent-color, #A78BFA);
  opacity: 0.25;
}

.skill-card:hover .skill-card__accent {
  @apply top-2 bottom-2;

  opacity: 0.7;
}
</style>
