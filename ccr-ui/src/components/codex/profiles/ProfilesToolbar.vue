<!-- 粘性工具条：搜索 / 状态pill / 标签pill / 排序 / 视图 / 结果数 -->
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
        :placeholder="$t('codex.profiles.toolbar.searchPlaceholder')"
        :aria-label="$t('codex.profiles.toolbar.searchPlaceholder')"
        class="cp-search__input"
        @input="onQueryInput"
      >
      <kbd class="cp-search__kbd">/</kbd>
    </div>

    <span class="cp-toolbar__sep" />

    <div
      class="cp-pill-row"
      role="group"
      :aria-label="$t('codex.profiles.toolbar.statusGroupLabel')"
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

    <span
      v-if="allTags.length > 0"
      class="cp-toolbar__sep"
    />

    <div
      v-if="allTags.length > 0"
      class="cp-pill-row"
      role="group"
      :aria-label="$t('codex.profiles.toolbar.tagGroupLabel')"
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

    <div class="cp-toolbar__right">
      <span class="cp-toolbar__meta">{{ resultCount }}/{{ total }}</span>

      <select
        :value="sortBy"
        class="cp-toolbar__sort"
        :aria-label="$t('codex.profiles.toolbar.sortLabel')"
        @change="onSortChange"
      >
        <option value="recent">
          {{ $t('codex.profiles.toolbar.sortRecent') }}
        </option>
        <option value="name">
          {{ $t('codex.profiles.toolbar.sortName') }}
        </option>
        <option value="requests">
          {{ $t('codex.profiles.toolbar.sortRequests') }}
        </option>
        <option value="enabled">
          {{ $t('codex.profiles.toolbar.sortEnabled') }}
        </option>
      </select>

      <div
        class="cp-seg"
        role="group"
        :aria-label="$t('codex.profiles.toolbar.viewLabel')"
      >
        <button
          type="button"
          class="cp-seg__btn"
          :class="{ 'cp-seg__btn--active': viewMode === 'card' }"
          :title="$t('codex.profiles.toolbar.viewCard')"
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
          :title="$t('codex.profiles.toolbar.viewList')"
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
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type {
  CodexProfilesSortBy,
  CodexProfilesStatusFilter,
} from '@/composables/useCodexProfilesFilter'

export type CodexProfilesViewMode = 'card' | 'list'

interface Props {
  query: string
  statusFilter: CodexProfilesStatusFilter
  tagFilter: string | null
  sortBy: CodexProfilesSortBy
  viewMode: CodexProfilesViewMode
  resultCount: number
  total: number
  allTags: string[]
}

defineProps<Props>()

const emit = defineEmits<{
  (e: 'update:query', value: string): void
  (e: 'update:statusFilter', value: CodexProfilesStatusFilter): void
  (e: 'update:tagFilter', value: string | null): void
  (e: 'update:sortBy', value: CodexProfilesSortBy): void
  (e: 'update:viewMode', value: CodexProfilesViewMode): void
}>()

const { t } = useI18n()
const searchRef = ref<HTMLInputElement | null>(null)

const statusOptions = computed<{ id: CodexProfilesStatusFilter; label: string }[]>(() => [
  { id: 'all',      label: t('codex.profiles.toolbar.statusAll') },
  { id: 'active',   label: t('codex.profiles.toolbar.statusActive') },
  { id: 'enabled',  label: t('codex.profiles.toolbar.statusEnabled') },
  { id: 'disabled', label: t('codex.profiles.toolbar.statusDisabled') },
])

const onQueryInput = (event: Event) => {
  emit('update:query', (event.target as HTMLInputElement).value)
}

const onSortChange = (event: Event) => {
  emit('update:sortBy', (event.target as HTMLSelectElement).value as CodexProfilesSortBy)
}

const focusSearch = () => searchRef.value?.focus()

defineExpose({ focusSearch })
</script>

<style scoped>
.cp-toolbar {
  position: sticky;
  top: 0;
  z-index: 5;
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
  font-size: 13px;
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
  font-size: 10px;
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
  font-size: 11px;
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
  font-size: 11.5px;
  font-family: var(--cp-mono);
}

.cp-toolbar__sort {
  padding: 6px 8px;
  background: var(--cp-bg-0);
  border: 1px solid var(--cp-line-2);
  border-radius: 7px;
  color: var(--cp-ink-1);
  font-family: var(--cp-mono);
  font-size: 12px;
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

@media (prefers-reduced-motion: reduce) {
  .cp-pill, .cp-seg__btn, .cp-search__input { transition: none; }
}
</style>
