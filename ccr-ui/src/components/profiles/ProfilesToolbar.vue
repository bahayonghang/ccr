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

    <span
      v-if="allTags.length > 0"
      class="cp-toolbar__sep"
    />

    <div
      v-if="allTags.length > 0"
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

    <div class="cp-toolbar__right">
      <span class="cp-toolbar__meta">{{ resultCount }}/{{ total }}</span>

      <select
        v-if="allProviders && allProviders.length > 1"
        :value="providerFilter ?? ''"
        class="cp-toolbar__sort"
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

      <select
        :value="sortBy"
        class="cp-toolbar__sort"
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
import { computed, ref } from 'vue'
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

const props = defineProps<Props>()

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

@media (prefers-reduced-motion: reduce) {
  .cp-pill, .cp-seg__btn, .cp-search__input { transition: none; }
}
</style>
