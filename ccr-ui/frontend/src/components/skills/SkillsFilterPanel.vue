<template>
  <aside
    class="skills-filter-panel transition-all duration-300"
    :class="[
      collapsed ? 'w-12 p-2' : 'w-[280px] p-4',
      { 'select-none': collapsed }
    ]"
  >
    <!-- Collapsed View -->
    <template v-if="collapsed">
      <div class="flex flex-col items-center gap-3 pt-1">
        <button
          class="p-2 rounded-lg text-text-muted hover:text-accent-primary hover:bg-bg-surface/50 transition-colors"
          :title="$t('skills.expandPanel')"
          @click="$emit('toggle-collapse')"
        >
          <PanelLeftOpen class="w-4 h-4" />
        </button>

        <div class="w-8 h-px bg-border-subtle" />

        <button
          class="p-2 rounded-lg hover:bg-bg-surface/50 transition-colors"
          :title="$t('skills.searchPlaceholder')"
        >
          <Search class="w-4 h-4 text-text-muted" />
        </button>

        <button
          class="p-2 rounded-lg hover:bg-bg-surface/50 transition-colors"
          :title="$t('skills.platforms')"
        >
          <Layers class="w-4 h-4 text-fuchsia-400" />
        </button>

        <div class="w-8 h-px bg-border-subtle" />

        <!-- Platform icons (collapsed) -->
        <div
          v-for="platform in platforms"
          :key="platform.id"
          class="w-8 h-8 flex items-center justify-center rounded-lg cursor-pointer
                 hover:bg-bg-surface/50 transition-all duration-200 group relative"
          :class="selectedPlatform === platform.id ? 'bg-accent-primary/10' : ''"
          @click="selectPlatform(platform.id)"
        >
          <component
            :is="getPlatformIcon(platform.id)"
            class="w-4 h-4"
            :style="{ color: getPlatformColor(platform.id) }"
          />
          <!-- Tooltip -->
          <div
            class="absolute left-full ml-2 px-2 py-1 bg-bg-overlay border border-border-subtle
                      rounded text-xs whitespace-nowrap opacity-0 group-hover:opacity-100
                      pointer-events-none transition-opacity z-50"
          >
            {{ platform.display_name }} ({{ platform.installed_count }})
          </div>
        </div>
      </div>
    </template>

    <!-- Expanded View -->
    <template v-else>
      <!-- Header with collapse button -->
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-sm font-bold flex items-center gap-2 text-text-primary">
          <Layers class="w-4 h-4 text-fuchsia-400" />
          {{ $t('skills.filters') }}
        </h2>
        <button
          class="p-1.5 rounded-lg text-text-muted hover:text-accent-primary
                 hover:bg-bg-surface/80 transition-all duration-200"
          :title="$t('skills.collapsePanel')"
          @click="$emit('toggle-collapse')"
        >
          <PanelLeftClose class="w-4 h-4" />
        </button>
      </div>

      <!-- Search Input -->
      <div class="relative mb-4">
        <Search
          class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-muted"
        />
        <input
          v-model="localSearch"
          type="text"
          :placeholder="$t('skills.searchPlaceholder')"
          class="w-full pl-10 pr-8 py-2.5 text-sm rounded-xl
                 bg-bg-surface/50 border border-border-subtle
                 text-text-primary placeholder:text-text-muted/50
                 focus:border-accent-primary/50 focus:bg-bg-surface
                 focus:outline-none focus:ring-2 focus:ring-accent-primary/20
                 transition-all duration-200"
          @input="debouncedSearchUpdate"
        >
        <button
          v-if="localSearch"
          class="absolute right-3 top-1/2 -translate-y-1/2 p-1
                 hover:bg-bg-elevated rounded-full text-text-muted transition-colors"
          @click="clearSearch"
        >
          <X class="w-3 h-3" />
        </button>
      </div>

      <!-- Platforms Section -->
      <div class="filter-section">
        <div class="filter-section__header">
          <Package class="w-3.5 h-3.5 text-fuchsia-400" />
          <span>{{ $t('skills.platforms') }}</span>
          <span class="filter-section__count">{{ totalInstalled }}</span>
        </div>

        <div class="space-y-1">
          <!-- All Platforms Option -->
          <button
            class="platform-item"
            :class="{ 'platform-item--active': selectedPlatform === 'all' }"
            @click="selectPlatform('all')"
          >
            <div class="flex items-center gap-2">
              <div
                class="w-3 h-3 rounded-full border-2 transition-all"
                :class="selectedPlatform === 'all'
                  ? 'border-accent-primary bg-accent-primary'
                  : 'border-border-default'"
              />
              <LayoutGrid class="w-4 h-4 text-emerald-400" />
              <span>{{ $t('skills.allPlatforms') }}</span>
            </div>
            <span class="platform-item__count">{{ totalInstalled }}</span>
          </button>

          <!-- Individual Platforms -->
          <button
            v-for="platform in platforms"
            :key="platform.id"
            class="platform-item"
            :class="{
              'platform-item--active': selectedPlatform === platform.id,
              'platform-item--disabled': !platform.detected
            }"
            :disabled="!platform.detected"
            @click="selectPlatform(platform.id)"
          >
            <div class="flex items-center gap-2">
              <div
                class="w-3 h-3 rounded-full border-2 transition-all"
                :class="selectedPlatform === platform.id
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
            <div class="flex items-center gap-2">
              <span
                v-if="!platform.detected"
                class="text-[10px] text-warning"
              >
                {{ $t('skills.notConfigured') }}
              </span>
              <span class="platform-item__count">{{ platform.installed_count }}</span>
            </div>
          </button>
        </div>
      </div>

      <!-- Source Section -->
      <div class="filter-section">
        <div class="filter-section__header">
          <FolderOpen class="w-3.5 h-3.5 text-cyan-400" />
          <span>{{ $t('skills.source') }}</span>
        </div>

        <div class="flex gap-1 p-1 rounded-xl bg-bg-surface/30 border border-border-subtle">
          <button
            v-for="opt in sourceOptions"
            :key="opt.value"
            class="source-btn"
            :class="{ 'source-btn--active': localSource === opt.value }"
            @click="selectSource(opt.value)"
          >
            {{ opt.label }}
          </button>
        </div>
      </div>

      <!-- Categories Section -->
      <div
        v-if="categories.length > 0"
        class="filter-section"
      >
        <div
          class="filter-section__header cursor-pointer"
          @click="expandedSections.categories = !expandedSections.categories"
        >
          <Folder class="w-3.5 h-3.5 text-violet-400" />
          <span>{{ $t('skills.categories') }}</span>
          <span class="filter-section__count">{{ categories.length }}</span>
          <ChevronDown
            class="w-3.5 h-3.5 ml-auto transition-transform"
            :class="{ 'rotate-180': expandedSections.categories }"
          />
        </div>

        <Transition name="collapse">
          <div
            v-if="expandedSections.categories"
            class="space-y-1 mt-2"
          >
            <button
              v-for="cat in displayCategories"
              :key="cat"
              class="category-item"
              :class="{ 'category-item--active': localCategory === cat }"
              @click="toggleCategory(cat)"
            >
              <span class="truncate">{{ cat }}</span>
            </button>
            <button
              v-if="categories.length > maxDisplayCategories"
              class="text-xs text-text-muted hover:text-accent-primary
                     px-2 py-1 transition-colors"
              @click="showAllCategories = !showAllCategories"
            >
              {{ showAllCategories ? $t('common.less') : `+${categories.length - maxDisplayCategories} ${$t('common.more')}` }}
            </button>
          </div>
        </Transition>
      </div>

      <!-- Tags Section -->
      <div
        v-if="tags.length > 0"
        class="filter-section"
      >
        <div
          class="filter-section__header cursor-pointer"
          @click="expandedSections.tags = !expandedSections.tags"
        >
          <Tag class="w-3.5 h-3.5 text-pink-400" />
          <span>{{ $t('skills.tags') }}</span>
          <span
            v-if="localTags.length > 0"
            class="px-1.5 py-0.5 rounded bg-accent-primary/20 text-accent-primary text-[10px] font-bold"
          >
            {{ localTags.length }}
          </span>
          <ChevronDown
            class="w-3.5 h-3.5 ml-auto transition-transform"
            :class="{ 'rotate-180': expandedSections.tags }"
          />
        </div>

        <Transition name="collapse">
          <div
            v-if="expandedSections.tags"
            class="flex flex-wrap gap-1.5 mt-2"
          >
            <button
              v-for="tag in displayTags"
              :key="tag"
              class="tag-item"
              :class="{ 'tag-item--active': localTags.includes(tag) }"
              @click="toggleTag(tag)"
            >
              #{{ tag }}
            </button>
            <button
              v-if="tags.length > maxDisplayTags"
              class="text-xs text-text-muted hover:text-accent-primary
                     px-2 py-1 transition-colors"
              @click="showAllTags = !showAllTags"
            >
              {{ showAllTags ? $t('common.less') : `+${tags.length - maxDisplayTags}` }}
            </button>
          </div>
        </Transition>
      </div>

      <!-- Clear Filters Button -->
      <button
        v-if="hasActiveFilters"
        class="clear-filters-btn"
        @click="clearAllFilters"
      >
        <FilterX class="w-4 h-4" />
        <span>{{ $t('common.clearFilters') }}</span>
      </button>
    </template>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed, watch, reactive } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  Search,
  X,
  Layers,
  Package,
  FolderOpen,
  Folder,
  Tag,
  ChevronDown,
  FilterX,
  LayoutGrid,
  PanelLeftOpen,
  PanelLeftClose,
  Code2,
  Settings,
  Sparkles,
  Zap,
  Activity,
  Bot
} from 'lucide-vue-next'
import type { Platform, SkillSource, PlatformSummary, SkillFilters } from '@/types/skills'
import { PLATFORM_CONFIG } from '@/types/skills'

const { t } = useI18n()

const props = defineProps<{
  platforms: PlatformSummary[]
  categories: string[]
  tags: string[]
  modelValue: SkillFilters
  collapsed?: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: SkillFilters): void
  (e: 'toggle-collapse'): void
}>()

// Local state
const localSearch = ref(props.modelValue.search)
const localSource = ref<SkillSource>(props.modelValue.source)
const localCategory = ref<string | null>(props.modelValue.category)
const localTags = ref<string[]>([...props.modelValue.tags])
const selectedPlatform = ref<Platform | 'all'>(props.modelValue.platform)

// UI state
const showAllCategories = ref(false)
const showAllTags = ref(false)
const maxDisplayCategories = 8
const maxDisplayTags = 12
const expandedSections = reactive({
  categories: true,
  tags: true
})

// Source options
const sourceOptions = computed(() => [
  { value: 'all' as SkillSource, label: t('skills.sourceAll') },
  { value: 'user' as SkillSource, label: t('skills.sourceUser') },
  { value: 'plugin' as SkillSource, label: t('skills.sourcePlugin') }
])

// Display categories (limited or all)
const displayCategories = computed(() => {
  if (showAllCategories.value) return props.categories
  return props.categories.slice(0, maxDisplayCategories)
})

// Display tags (limited or all)
const displayTags = computed(() => {
  if (showAllTags.value) return props.tags
  return props.tags.slice(0, maxDisplayTags)
})

// Total installed skills count
const totalInstalled = computed(() => {
  return props.platforms.reduce((sum, p) => sum + p.installed_count, 0)
})

// Check if any filters are active
const hasActiveFilters = computed(() => {
  return localSearch.value !== '' ||
         localSource.value !== 'all' ||
         localCategory.value !== null ||
         localTags.value.length > 0 ||
         selectedPlatform.value !== 'all'
})

// Get platform icon component
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

// Get platform color
function getPlatformColor(platformId: string): string {
  const config = PLATFORM_CONFIG[platformId as Platform]
  return config?.color || '#A78BFA'
}

// Emit update to parent
function emitUpdate() {
  emit('update:modelValue', {
    search: localSearch.value,
    source: localSource.value,
    category: localCategory.value,
    tags: [...localTags.value],
    platform: selectedPlatform.value
  })
}

// Debounced search (300ms) to avoid excessive filter recalculations
let searchTimer: ReturnType<typeof setTimeout> | null = null
function debouncedSearchUpdate() {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => emitUpdate(), 300)
}

// Event handlers
function clearSearch() {
  localSearch.value = ''
  emitUpdate()
}

function selectPlatform(platform: Platform | 'all') {
  selectedPlatform.value = platform
  emitUpdate()
}

function selectSource(source: SkillSource) {
  localSource.value = source
  emitUpdate()
}

function toggleCategory(category: string) {
  if (localCategory.value === category) {
    localCategory.value = null
  } else {
    localCategory.value = category
  }
  emitUpdate()
}

function toggleTag(tag: string) {
  const index = localTags.value.indexOf(tag)
  if (index === -1) {
    localTags.value.push(tag)
  } else {
    localTags.value.splice(index, 1)
  }
  emitUpdate()
}

function clearAllFilters() {
  localSearch.value = ''
  localSource.value = 'all'
  localCategory.value = null
  localTags.value = []
  selectedPlatform.value = 'all'
  emitUpdate()
}

// Watch for external changes
watch(() => props.modelValue, (newVal) => {
  localSearch.value = newVal.search
  localSource.value = newVal.source
  localCategory.value = newVal.category
  localTags.value = [...newVal.tags]
  selectedPlatform.value = newVal.platform
}, { deep: true })
</script>

<style scoped>
/* Panel Container - Glassmorphism */
.skills-filter-panel {
  @apply sticky top-6 h-fit rounded-2xl overflow-hidden;

  background: var(--glass-bg-light, rgb(255 255 255 / 60%));
  backdrop-filter: blur(16px) saturate(180%);
  border: 1px solid rgb(244 114 182 / 15%);
  max-height: calc(100vh - 180px);
  overflow-y: auto;
}

/* Dark mode glass effect */
:root[data-theme="dark"] .skills-filter-panel,
.dark .skills-filter-panel {
  background: linear-gradient(
    180deg,
    rgb(26 10 32 / 85%) 0%,
    rgb(38 18 50 / 88%) 50%,
    rgb(26 10 32 / 90%) 100%
  );
  border-color: rgb(249 168 212 / 10%);
  box-shadow:
    4px 0 24px rgb(0 0 0 / 20%),
    inset 0 1px 0 rgb(249 168 212 / 5%);
}

/* Filter Section */
.filter-section {
  @apply py-3 border-b border-border-subtle last:border-b-0;
}

.filter-section__header {
  @apply flex items-center gap-2 text-xs font-bold uppercase tracking-wide
         text-text-secondary mb-2 select-none;
}

.filter-section__count {
  @apply px-1.5 py-0.5 rounded text-[10px] font-bold font-mono
         bg-bg-surface text-text-muted;
}

/* Platform Item */
.platform-item {
  @apply w-full flex items-center justify-between px-3 py-2 rounded-xl
         text-sm text-text-secondary cursor-pointer
         transition-all duration-200;
}

.platform-item:hover {
  @apply text-text-primary;

  background: rgb(var(--color-bg-surface-rgb) / 50%);
}

.platform-item--active {
  @apply text-text-primary;

  background: rgb(var(--color-accent-primary-rgb) / 8%);
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 20%);
}

.platform-item--disabled {
  @apply opacity-50 cursor-not-allowed;
}

.platform-item--disabled:hover {
  @apply bg-transparent text-text-secondary;
}

.platform-item__count {
  @apply text-xs font-mono font-bold px-1.5 py-0.5 rounded
         bg-bg-surface text-text-muted;
}

.platform-item--active .platform-item__count {
  @apply bg-accent-primary/20 text-accent-primary;
}

/* Source Button */
.source-btn {
  @apply flex-1 px-3 py-1.5 rounded-lg text-xs font-semibold
         text-text-secondary transition-all duration-200;
}

.source-btn:hover {
  @apply text-text-primary;
}

.source-btn--active {
  @apply text-text-primary bg-bg-elevated shadow-sm;
}

/* Category Item */
.category-item {
  @apply w-full text-left px-3 py-1.5 rounded-lg text-xs font-medium
         text-text-secondary hover:text-text-primary
         transition-all duration-200;
}

.category-item:hover {
  background: rgb(var(--color-bg-surface-rgb) / 50%);
}

.category-item--active {
  @apply text-accent-primary bg-accent-primary/10;
}

/* Tag Item */
.tag-item {
  @apply px-2 py-1 rounded-md text-xs font-medium
         bg-bg-surface text-text-secondary
         hover:bg-bg-elevated hover:text-text-primary
         transition-all duration-200;
}

.tag-item--active {
  @apply bg-accent-primary text-white;
}

/* Clear Filters Button */
.clear-filters-btn {
  @apply w-full flex items-center justify-center gap-2 mt-4 px-4 py-2.5
         rounded-xl text-sm font-semibold border
         transition-all duration-200;

  color: var(--color-danger);
  background: rgb(var(--color-danger-rgb) / 10%);
  border-color: rgb(var(--color-danger-rgb) / 20%);
}

.clear-filters-btn:hover {
  background: rgb(var(--color-danger-rgb) / 20%);
  border-color: rgb(var(--color-danger-rgb) / 30%);
}

/* Collapse Transition */
.collapse-enter-active,
.collapse-leave-active {
  transition: all 0.2s ease;
  overflow: hidden;
}

.collapse-enter-from,
.collapse-leave-to {
  opacity: 0;
  max-height: 0;
}

.collapse-enter-to,
.collapse-leave-from {
  opacity: 1;
  max-height: 500px;
}

/* Custom Scrollbar */
.skills-filter-panel::-webkit-scrollbar {
  width: 4px;
}

.skills-filter-panel::-webkit-scrollbar-track {
  background: transparent;
}

.skills-filter-panel::-webkit-scrollbar-thumb {
  background: rgb(var(--color-border-subtle-rgb) / 50%);
  border-radius: 2px;
}

.skills-filter-panel::-webkit-scrollbar-thumb:hover {
  background: var(--color-border-default);
}
</style>
