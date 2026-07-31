<!-- 粘性工具条：搜索 / 状态pill / 标签pill / (可选)Provider下拉 / 排序 / 视图 / 结果数。
     平台差异通过 i18nPrefix + 可选 provider 维度注入；样式靠视图的 --cp-* 继承换肤。 -->
<template>
  <div class="cp-toolbar surface-workspace">
    <div class="cp-search">
      <SIcon
        name="Search"
        size="w-3.5 h-3.5"
        class="cp-search__icon"
      />
      <input
        ref="searchRef"
        :value="query"
        :placeholder="t(`${i18nPrefix}.searchPlaceholder`)"
        :aria-label="t(`${i18nPrefix}.searchPlaceholder`)"
        class="cp-search__input"
        @input="onQueryInput"
      >
      <kbd class="cp-search__kbd">/</kbd>
    </div>

    <span class="cp-toolbar__sep" />

    <div
      class="cp-pill-row"
      role="group"
      :aria-label="t(`${i18nPrefix}.statusGroupLabel`)"
    >
      <button
        v-for="opt in statusOptions"
        :key="opt.id"
        type="button"
        class="cp-pill"
        :class="{ 'cp-pill--active': statusFilter === opt.id }"
        :aria-pressed="statusFilter === opt.id"
        @click="emit('update:statusFilter', opt.id)"
      >
        {{ opt.label }}
      </button>
    </div>

    <span class="cp-toolbar__sep" />

    <div class="cp-filters">
      <button
        ref="filtersBtnRef"
        type="button"
        class="cp-pill cp-filters__trigger"
        :class="{ 'cp-pill--active': activeFilterCount > 0 || filtersOpen }"
        :aria-expanded="filtersOpen"
        aria-haspopup="dialog"
        @click="toggleFilters"
      >
        <SIcon
          name="SlidersHorizontal"
          size="w-3.5 h-3.5"
        />
        {{ t(`${i18nPrefix}.filtersButton`) }}
        <span
          v-if="activeFilterCount > 0"
          class="cp-filters__badge"
        >{{ activeFilterCount }}</span>
        <SIcon
          name="ChevronDown"
          size="w-3 h-3"
        />
      </button>

      <div
        v-if="filtersOpen"
        ref="filtersPopRef"
        class="cp-filters__pop"
        role="dialog"
        :aria-label="t(`${i18nPrefix}.filtersButton`)"
        @keydown="onFiltersKeydown"
      >
        <div
          v-if="allTags.length > 0"
          class="cp-filters__section"
        >
          <div class="cp-filters__label">
            {{ t(`${i18nPrefix}.tagGroupLabel`) }}
          </div>
          <div
            class="cp-pill-row"
            role="group"
            :aria-label="t(`${i18nPrefix}.tagGroupLabel`)"
          >
            <button
              v-for="tag in allTags"
              :key="tag"
              type="button"
              class="cp-pill"
              :class="{ 'cp-pill--active': tagFilter === tag }"
              :aria-pressed="tagFilter === tag"
              @click="emit('update:tagFilter', tagFilter === tag ? null : tag)"
            >
              #{{ tag }}
            </button>
          </div>
        </div>

        <div
          v-if="allProviders && allProviders.length > 1"
          class="cp-filters__section"
        >
          <div class="cp-filters__label">
            {{ t(`${i18nPrefix}.providerLabel`) }}
          </div>
          <select
            :value="providerFilter ?? ''"
            class="cp-toolbar__sort cp-filters__select"
            :aria-label="t(`${i18nPrefix}.providerLabel`)"
            @change="onProviderChange"
          >
            <option value="">
              {{ t(`${i18nPrefix}.providerAll`) }}
            </option>
            <option
              v-for="provider in allProviders"
              :key="provider.key"
              :value="provider.key"
            >
              {{ provider.label }}
            </option>
          </select>
        </div>

        <div class="cp-filters__section">
          <div class="cp-filters__label">
            {{ t(`${i18nPrefix}.sortLabel`) }}
          </div>
          <select
            :value="sortBy"
            class="cp-toolbar__sort cp-filters__select"
            :aria-label="t(`${i18nPrefix}.sortLabel`)"
            @change="onSortChange"
          >
            <option value="recent">
              {{ t(`${i18nPrefix}.sortRecent`) }}
            </option>
            <option value="name">
              {{ t(`${i18nPrefix}.sortName`) }}
            </option>
            <option value="requests">
              {{ t(`${i18nPrefix}.sortRequests`) }}
            </option>
            <option value="enabled">
              {{ t(`${i18nPrefix}.sortEnabled`) }}
            </option>
          </select>
        </div>

        <div class="cp-filters__foot">
          <button
            type="button"
            class="cp-pill"
            :disabled="activeFilterCount === 0"
            @click="clearAllFilters"
          >
            {{ t(`${i18nPrefix}.clearAll`) }}
          </button>
        </div>
      </div>
    </div>

    <div class="cp-toolbar__right">
      <span class="cp-toolbar__meta">{{ resultCount }}/{{ total }}</span>

      <div
        class="cp-seg"
        role="group"
        :aria-label="t(`${i18nPrefix}.viewLabel`)"
      >
        <button
          type="button"
          class="cp-seg__btn"
          :class="{ 'cp-seg__btn--active': viewMode === 'card' }"
          :title="t(`${i18nPrefix}.viewCard`)"
          :aria-pressed="viewMode === 'card'"
          @click="emit('update:viewMode', 'card')"
        >
          <SIcon
            name="Layers"
            size="w-3.5 h-3.5"
          />
        </button>
        <button
          type="button"
          class="cp-seg__btn"
          :class="{ 'cp-seg__btn--active': viewMode === 'list' }"
          :title="t(`${i18nPrefix}.viewList`)"
          :aria-pressed="viewMode === 'list'"
          @click="emit('update:viewMode', 'list')"
        >
          <SIcon
            name="List"
            size="w-3.5 h-3.5"
          />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type {
  ProfilesSortBy,
  ProfilesStatusFilter,
  ProviderOption,
} from '@/composables/useProfilesFilter'

export type ProfilesViewMode = 'card' | 'list'

interface Props {
  query: string
  statusFilter: ProfilesStatusFilter
  tagFilter: string | null
  sortBy: ProfilesSortBy
  viewMode: ProfilesViewMode
  resultCount: number
  total: number
  allTags: string[]
  /** i18n key 前缀，例如 'claudeProfiles.toolbar' / 'codex.profiles.toolbar' */
  i18nPrefix: string
  /** provider 维度（Claude 用，Codex 省略 → 不渲染 provider 下拉） */
  providerFilter?: string | null
  allProviders?: ProviderOption[]
}

const props = withDefaults(defineProps<Props>(), {
  providerFilter: null,
  allProviders: undefined,
})

const emit = defineEmits<{
  (e: 'update:query', value: string): void
  (e: 'update:statusFilter', value: ProfilesStatusFilter): void
  (e: 'update:tagFilter', value: string | null): void
  (e: 'update:providerFilter', value: string | null): void
  (e: 'update:sortBy', value: ProfilesSortBy): void
  (e: 'update:viewMode', value: ProfilesViewMode): void
}>()

const { t } = useI18n()
const searchRef = ref<HTMLInputElement | null>(null)

/* Filters: Esc restores trigger focus; outside click closes; selections keep it open. */

const filtersOpen = ref(false)
const filtersBtnRef = ref<HTMLButtonElement | null>(null)
const filtersPopRef = ref<HTMLElement | null>(null)

/** 生效筛选数徽标：标签 + provider + 非默认排序 */
const activeFilterCount = computed(() => {
  let count = 0
  if (props.tagFilter) count += 1
  if (props.providerFilter) count += 1
  if (props.sortBy !== 'recent') count += 1
  return count
})

const FOCUSABLE_SELECTOR = 'button:not(:disabled), select, input, [tabindex]:not([tabindex="-1"])'

const focusableInPopover = () =>
  Array.from(filtersPopRef.value?.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR) ?? [])

const closeFilters = (restoreFocus: boolean) => {
  if (!filtersOpen.value) return
  filtersOpen.value = false
  if (restoreFocus) filtersBtnRef.value?.focus()
}

const toggleFilters = async () => {
  filtersOpen.value = !filtersOpen.value
  if (filtersOpen.value) {
    await nextTick()
    focusableInPopover()[0]?.focus()
  }
}

const onFiltersKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    event.preventDefault()
    event.stopPropagation()
    closeFilters(true)
    return
  }
  if (event.key === 'Tab') {
    // 轻量 focus trap：Tab 在弹层内循环
    const focusable = focusableInPopover()
    if (focusable.length === 0) return
    const first = focusable[0]
    const last = focusable[focusable.length - 1]
    const active = document.activeElement
    if (event.shiftKey && active === first) {
      event.preventDefault()
      last.focus()
    } else if (!event.shiftKey && active === last) {
      event.preventDefault()
      first.focus()
    }
    return
  }
  if (
    event.target instanceof HTMLButtonElement
    && ['ArrowDown', 'ArrowRight', 'ArrowUp', 'ArrowLeft'].includes(event.key)
  ) {
    const focusable = focusableInPopover()
    if (focusable.length === 0) return
    const activeIndex = focusable.indexOf(document.activeElement as HTMLElement)
    if (activeIndex < 0) return
    const delta = event.key === 'ArrowDown' || event.key === 'ArrowRight' ? 1 : -1
    event.preventDefault()
    focusable[(activeIndex + delta + focusable.length) % focusable.length]?.focus()
  }
}

const onDocumentPointerDown = (event: MouseEvent) => {
  if (!filtersOpen.value) return
  const target = event.target as Node
  if (filtersPopRef.value?.contains(target)) return
  if (filtersBtnRef.value?.contains(target)) return
  closeFilters(false)
}

watch(filtersOpen, (open) => {
  if (open) document.addEventListener('mousedown', onDocumentPointerDown)
  else document.removeEventListener('mousedown', onDocumentPointerDown)
})

onBeforeUnmount(() => document.removeEventListener('mousedown', onDocumentPointerDown))

const clearAllFilters = () => {
  emit('update:tagFilter', null)
  emit('update:providerFilter', null)
  emit('update:sortBy', 'recent')
  closeFilters(true)
}

const statusOptions = computed<{ id: ProfilesStatusFilter; label: string }[]>(() => [
  { id: 'all', label: t(`${props.i18nPrefix}.statusAll`) },
  { id: 'active', label: t(`${props.i18nPrefix}.statusActive`) },
  { id: 'enabled', label: t(`${props.i18nPrefix}.statusEnabled`) },
  { id: 'disabled', label: t(`${props.i18nPrefix}.statusDisabled`) },
])

const onQueryInput = (event: Event) => {
  emit('update:query', (event.target as HTMLInputElement).value)
}

const onSortChange = (event: Event) => {
  emit('update:sortBy', (event.target as HTMLSelectElement).value as ProfilesSortBy)
}

const onProviderChange = (event: Event) => {
  const value = (event.target as HTMLSelectElement).value
  emit('update:providerFilter', value ? value : null)
}

const focusSearch = () => searchRef.value?.focus()

defineExpose({ focusSearch })
</script>

<style scoped>
.cp-toolbar {
  position: sticky;
  top: 0;
  z-index: var(--layer-raised);
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  margin-bottom: 14px;

  /* 背景/边框由 surface-workspace 工具类提供 */
  border-radius: 12px;
  flex-wrap: wrap;
}

.cp-search {
  position: relative;
  flex: 1 1 280px;
  min-width: 220px;
  max-width: 380px;
}

.cp-search__icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--cp-ink-3);
  pointer-events: none;
}

.cp-search__input {
  width: 100%;
  padding: 8px 38px 8px 32px;
  background: var(--cp-bg-0);
  border: 1px solid var(--cp-line-2);
  border-radius: 7px;
  color: var(--cp-ink-0);
  font-size: 0.8125rem;
  font-family: inherit;
  outline: none;
  transition: border-color 120ms ease;
}

.cp-search__input:focus { border-color: var(--cp-accent-line); }
.cp-search__input::placeholder { color: var(--cp-ink-4); }

.cp-search__kbd {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  padding: 2px 6px;
  font-family: var(--cp-mono);
  font-size: 0.75rem;
  color: var(--cp-ink-4);
  background: var(--cp-bg-2);
  border: 1px solid var(--cp-line-2);
  border-radius: 4px;
}

.cp-toolbar__sep {
  width: 1px;
  height: 22px;
  background: var(--cp-line);
}

.cp-pill-row {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.cp-pill {
  padding: 5px 10px;
  border-radius: 999px;
  border: 1px solid var(--cp-line-2);
  background: var(--cp-bg-2);
  color: var(--cp-ink-2);
  font-family: var(--cp-mono);
  font-size: 0.75rem;
  letter-spacing: 0.2px;
  cursor: pointer;
  transition: background 120ms ease, border-color 120ms ease, color 120ms ease;
}

.cp-pill:hover:not(.cp-pill--active) {
  background: var(--cp-bg-3);
  color: var(--cp-ink-0);
}

.cp-pill--active {
  background: var(--cp-accent-soft);
  border-color: var(--cp-accent-line);
  color: var(--cp-accent);
}

.cp-toolbar__right {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.cp-toolbar__meta {
  color: var(--cp-ink-3);
  font-size: 0.75rem;
  font-family: var(--cp-mono);
}

.cp-toolbar__sort {
  padding: 6px 8px;
  background: var(--cp-bg-0);
  border: 1px solid var(--cp-line-2);
  border-radius: 7px;
  color: var(--cp-ink-1);
  font-family: var(--cp-mono);
  font-size: 0.75rem;
  max-width: 160px;
}

.cp-seg {
  display: inline-flex;
  padding: 2px;
  gap: 2px;
  background: var(--cp-bg-0);
  border: 1px solid var(--cp-line-2);
  border-radius: 7px;
}

.cp-seg__btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 24px;
  border: none;
  background: transparent;
  color: var(--cp-ink-3);
  border-radius: 5px;
  cursor: pointer;
  transition: background 120ms ease, color 120ms ease;
}

.cp-seg__btn:hover { color: var(--cp-ink-0); }

.cp-seg__btn--active {
  background: var(--cp-bg-3);
  color: var(--cp-ink-0);
  box-shadow: inset 0 0 0 1px var(--cp-line-2);
}

/* Filters popover anchored to its trigger and right-aligned to avoid overflow. */
.cp-filters {
  position: relative;
  display: inline-flex;
}

.cp-filters__trigger {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}

.cp-filters__badge {
  display: inline-grid;
  place-items: center;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  border-radius: 999px;
  background: var(--cp-accent);
  color: var(--cp-on-accent);
  font-size: 0.75rem;
  font-weight: 600;
  line-height: 1;
}

.cp-filters__pop {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  z-index: var(--layer-popover);
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 260px;
  max-width: min(340px, calc(100vw - 24px));
  max-height: min(420px, calc(100vh - 96px));
  overflow-y: auto;
  padding: 12px;
  background: var(--cp-bg-1);
  border: 1px solid var(--cp-line-2);
  border-radius: 12px;
  box-shadow: 0 16px 40px rgb(0 0 0 / 22%);
}

.cp-filters__section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.cp-filters__label {
  font-family: var(--cp-mono);
  font-size: 0.75rem;
  letter-spacing: 0.05rem;
  text-transform: uppercase;
  color: var(--cp-ink-3);
}

.cp-filters__select { width: 100%; }

.cp-filters__foot {
  display: flex;
  justify-content: flex-end;
  padding-top: 8px;
  border-top: 1px solid var(--cp-line);
}

/* 窄窗口退化为全宽面板（跟随视口而非触发按钮宽度） */
@media (width <= 720px) {
  .cp-filters__pop {
    position: fixed;
    inset: auto 12px 12px;
    max-width: none;
    max-height: 60vh;
  }
}

@media (prefers-reduced-motion: reduce) {
  .cp-pill, .cp-seg__btn, .cp-search__input { transition: none; }
}
</style>
