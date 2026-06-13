<!--
  ⌘K 命令面板：模糊搜索切换 profile 与执行常用命令。
  仅在 Codex Profiles 页面内挂载，关闭后由父组件释放快捷键监听。
-->
<template>
  <BaseModal
    :model-value="open"
    :title="$t('codex.profiles.commandPalette.title')"
    :description="resultSummary"
    size="full"
    surface="solid"
    content-class="cp-palette-modal"
    :show-close="false"
    @update:model-value="emit('update:open', $event)"
  >
    <template #header="{ titleId }">
      <div class="cp-palette__header">
        <div class="cp-palette__heading">
          <div class="cp-palette__eyebrow">
            {{ $t('codex.profiles.commandPalette.eyebrow') }}
          </div>
          <h2
            :id="titleId"
            class="cp-palette__title"
          >
            {{ $t('codex.profiles.commandPalette.title') }}
          </h2>
        </div>
        <div
          class="cp-palette__summary"
          aria-live="polite"
        >
          {{ resultSummary }}
        </div>
      </div>
    </template>

    <div class="cp-palette">
      <label
        class="sr-only"
        for="codex-command-palette-search"
      >
        {{ $t('codex.profiles.commandPalette.placeholder') }}
      </label>
      <div class="cp-palette__search-wrap">
        <SIcon
          name="Search"
          size="w-4 h-4"
          class="cp-palette__search-icon"
        />
        <input
          id="codex-command-palette-search"
          ref="inputRef"
          v-model="query"
          class="cp-palette__search"
          :placeholder="$t('codex.profiles.commandPalette.placeholder')"
          :aria-label="$t('codex.profiles.commandPalette.placeholder')"
          :aria-activedescendant="activeOptionId"
          aria-controls="codex-command-palette-list"
          @keydown="onKeyDown"
        >
      </div>

      <div
        id="codex-command-palette-list"
        ref="listRef"
        class="cp-palette__list"
        role="listbox"
        :aria-label="$t('codex.profiles.commandPalette.title')"
      >
        <template
          v-for="group in groups"
          :key="group.id"
        >
          <section
            v-if="group.items.length > 0"
            class="cp-palette__group"
            :aria-label="group.title"
          >
            <div class="cp-palette__group-head">
              <span>{{ group.title }}</span>
              <span class="cp-palette__group-count">{{ group.items.length }}</span>
            </div>

            <div
              v-for="item in group.items"
              :id="optionId(item.index)"
              :key="item.id"
              :data-index="item.index"
              class="cp-palette__row"
              :class="{ 'cp-palette__row--active': item.index === activeIdx }"
              role="option"
              :aria-selected="item.index === activeIdx"
              @mouseenter="activeIdx = item.index"
              @click="fire(item)"
            >
              <span class="cp-palette__icon-box">
                <SIcon
                  :name="item.icon"
                  size="w-3.5 h-3.5"
                  :class="item.index === activeIdx ? 'cp-palette__icon--accent' : 'cp-palette__icon'"
                />
              </span>
              <div class="cp-palette__main">
                <div
                  class="cp-palette__label"
                  :class="{ 'cp-palette__label--mono': item.kind === 'switch' }"
                >
                  {{ item.label }}
                </div>
                <div
                  v-if="item.hint"
                  class="cp-palette__sub"
                >
                  {{ item.hint }}
                </div>
              </div>
              <span class="cp-palette__badge">
                {{ item.kind === 'switch'
                  ? $t('codex.profiles.commandPalette.kindSwitch')
                  : $t('codex.profiles.commandPalette.kindCommand') }}
              </span>
            </div>
          </section>
        </template>

        <div
          v-if="items.length === 0"
          class="cp-palette__empty"
        >
          {{ $t('codex.profiles.commandPalette.empty') }}
        </div>
      </div>
    </div>

    <template #footer>
      <div class="cp-palette__foot">
        <span><kbd class="cp-palette__kbd">↵</kbd> {{ $t('codex.profiles.commandPalette.execute') }}</span>
        <span><kbd class="cp-palette__kbd">↑↓</kbd> {{ $t('codex.profiles.commandPalette.select') }}</span>
        <span><kbd class="cp-palette__kbd">Esc</kbd> {{ $t('codex.profiles.commandPalette.close') }}</span>
      </div>
    </template>
  </BaseModal>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import BaseModal from '@/components/common/BaseModal.vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { IconName } from '@/config/icons'
import type { CodexProfile } from '@/types'

interface Props {
  open: boolean
  profiles: CodexProfile[]
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'update:open', value: boolean): void
  (e: 'apply', name: string): void
  (e: 'add'): void
  (e: 'import'): void
  (e: 'export'): void
  (e: 'reload'): void
}>()

const { t } = useI18n()

interface PaletteItem {
  id: string
  kind: 'cmd' | 'switch'
  label: string
  hint?: string
  icon: IconName
  action: () => void
}

interface IndexedPaletteItem extends PaletteItem {
  index: number
}

interface PaletteGroup {
  id: 'commands' | 'profiles'
  title: string
  items: IndexedPaletteItem[]
}

const query = ref('')
const activeIdx = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)
const listRef = ref<HTMLDivElement | null>(null)

watch(
  () => props.open,
  async open => {
    if (open) {
      query.value = ''
      activeIdx.value = 0
      await nextTick()
      inputRef.value?.focus()
    }
  },
)

const baseCommands = computed<PaletteItem[]>(() => [
  {
    id: '__add',
    kind: 'cmd',
    label: t('codex.profiles.commandPalette.actionAdd'),
    icon: 'Plus',
    action: () => emit('add'),
  },
  {
    id: '__reload',
    kind: 'cmd',
    label: t('codex.profiles.commandPalette.actionReload'),
    icon: 'RefreshCw',
    action: () => emit('reload'),
  },
  {
    id: '__export',
    kind: 'cmd',
    label: t('codex.profiles.commandPalette.actionExport'),
    icon: 'Download',
    action: () => emit('export'),
  },
])

const switchItems = computed<PaletteItem[]>(() =>
  props.profiles
    .filter(p => p.enabled !== false)
    .map(p => ({
      id: p.name,
      kind: 'switch',
      label: t('codex.profiles.commandPalette.actionApply', { name: p.name }),
      hint: p.description || p.base_url || undefined,
      icon: 'Play',
      action: () => emit('apply', p.name),
    })),
)

const matchesQuery = (item: PaletteItem) => {
  const q = query.value.trim().toLowerCase()
  if (!q) return true
  return `${item.label} ${item.hint ?? ''}`.toLowerCase().includes(q)
}

const filteredBuckets = computed(() => ({
  commands: baseCommands.value.filter(matchesQuery),
  profiles: switchItems.value.filter(matchesQuery),
}))

const items = computed<PaletteItem[]>(() => [
  ...filteredBuckets.value.commands,
  ...filteredBuckets.value.profiles,
])

const groups = computed<PaletteGroup[]>(() => {
  const commandItems = filteredBuckets.value.commands.map((item, index) => ({
    ...item,
    index,
  }))
  const profileItems = filteredBuckets.value.profiles.map((item, index) => ({
    ...item,
    index: commandItems.length + index,
  }))

  return [
    {
      id: 'commands',
      title: t('codex.profiles.commandPalette.groupCommands'),
      items: commandItems,
    },
    {
      id: 'profiles',
      title: t('codex.profiles.commandPalette.groupProfiles'),
      items: profileItems,
    },
  ]
})

const resultSummary = computed(() => t('codex.profiles.commandPalette.resultSummary', {
  commands: filteredBuckets.value.commands.length,
  profiles: filteredBuckets.value.profiles.length,
}))

const optionId = (index: number) => `codex-command-palette-option-${index}`

const activeOptionId = computed(() => (
  items.value.length > 0 ? optionId(activeIdx.value) : undefined
))

watch(items, () => {
  if (activeIdx.value >= items.value.length) {
    activeIdx.value = Math.max(0, items.value.length - 1)
  }
})

const fire = (item: PaletteItem) => {
  item.action()
  emit('update:open', false)
}

const scrollActiveIntoView = () => {
  const el = listRef.value?.querySelector<HTMLElement>(`[data-index="${activeIdx.value}"]`)
  el?.scrollIntoView({ block: 'nearest' })
}

const onKeyDown = (event: KeyboardEvent) => {
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    if (items.value.length === 0) return
    activeIdx.value = Math.min(items.value.length - 1, activeIdx.value + 1)
    void nextTick(scrollActiveIntoView)
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    if (items.value.length === 0) return
    activeIdx.value = Math.max(0, activeIdx.value - 1)
    void nextTick(scrollActiveIntoView)
  } else if (event.key === 'Enter') {
    event.preventDefault()
    const item = items.value[activeIdx.value]
    if (item) fire(item)
  } else if (event.key === 'Escape') {
    event.preventDefault()
    event.stopPropagation()
    emit('update:open', false)
  }
}
</script>

<style scoped>
:global(.cp-palette-modal) {
  max-width: min(720px, calc(100vw - 32px)) !important;
  border-radius: 18px !important;
  background: var(--color-bg-elevated) !important;
  border: 1px solid var(--surface-modal-border, var(--color-border-default)) !important;
  box-shadow: var(--surface-modal-shadow, 0 28px 80px rgb(0 0 0 / 28%)) !important;
  backdrop-filter: none !important;
}

.cp-palette,
.cp-palette__header,
.cp-palette__foot {
  --palette-bg: var(--color-bg-elevated);
  --palette-bg-soft: var(--color-bg-surface);
  --palette-bg-muted: var(--color-bg-overlay);
  --palette-ink: var(--color-text-primary);
  --palette-ink-soft: var(--color-text-secondary);
  --palette-ink-muted: var(--color-text-muted);
  --palette-ink-faint: var(--color-text-disabled);
  --palette-line: var(--color-border-subtle);
  --palette-line-strong: var(--color-border-default);
  --palette-accent: var(--color-accent-primary);
  --palette-accent-soft: rgb(var(--color-accent-primary-rgb) / 12%);
  --palette-accent-line: rgb(var(--color-accent-primary-rgb) / 32%);
  --palette-mono: var(--font-mono, 'MapleBright', monospace);
}

.cp-palette__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.cp-palette__heading {
  min-width: 0;
}

.cp-palette__eyebrow {
  color: var(--palette-ink-muted);
  font-family: var(--palette-mono);
  font-size: 10.5px;
  letter-spacing: 1.1px;
  line-height: 1.2;
  text-transform: uppercase;
}

.cp-palette__title {
  margin: 4px 0 0;
  color: var(--palette-ink);
  font-size: 18px;
  font-weight: 650;
  letter-spacing: -0.3px;
  line-height: 1.25;
}

.cp-palette__summary {
  flex-shrink: 0;
  max-width: 240px;
  padding: 5px 9px;
  border: 1px solid var(--palette-line);
  border-radius: 999px;
  background: var(--palette-bg-soft);
  color: var(--palette-ink-muted);
  font-family: var(--palette-mono);
  font-size: 11px;
  line-height: 1.2;
  text-align: right;
}

.cp-palette {
  color: var(--palette-ink-soft);
}

.cp-palette__search-wrap {
  position: relative;
}

.cp-palette__search-icon {
  position: absolute;
  top: 50%;
  left: 13px;
  color: var(--palette-ink-muted);
  pointer-events: none;
  transform: translateY(-50%);
}

.cp-palette__search {
  width: 100%;
  padding: 12px 14px 12px 40px;
  background: var(--palette-bg-soft);
  border: 1px solid var(--palette-line-strong);
  border-radius: 12px;
  color: var(--palette-ink);
  font-size: 15px;
  font-family: inherit;
  outline: none;
  transition: border-color 120ms ease, box-shadow 120ms ease, background 120ms ease;
}

.cp-palette__search:focus {
  border-color: var(--palette-accent-line);
  background: var(--palette-bg);
  box-shadow: 0 0 0 3px rgb(var(--color-accent-primary-rgb) / 10%);
}

.cp-palette__search::placeholder {
  color: var(--palette-ink-faint);
}

.cp-palette__list {
  margin-top: 14px;
  max-height: min(60vh, 520px);
  overflow-y: auto;
  padding-right: 2px;
  scrollbar-color: var(--palette-line-strong) transparent;
}

.cp-palette__group + .cp-palette__group {
  margin-top: 14px;
}

.cp-palette__group-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 0 4px 7px;
  color: var(--palette-ink-muted);
  font-family: var(--palette-mono);
  font-size: 11px;
  letter-spacing: 0.9px;
  line-height: 1.2;
  text-transform: uppercase;
}

.cp-palette__group-count {
  color: var(--palette-ink-faint);
}

.cp-palette__row {
  position: relative;
  display: grid;
  grid-template-columns: 28px minmax(0, 1fr) auto;
  gap: 12px;
  align-items: center;
  min-height: 58px;
  padding: 10px 12px;
  border: 1px solid transparent;
  border-radius: 12px;
  cursor: pointer;
  transition: background 120ms ease, border-color 120ms ease, box-shadow 120ms ease;
}

.cp-palette__row + .cp-palette__row {
  margin-top: 4px;
}

.cp-palette__row::before {
  position: absolute;
  top: 12px;
  bottom: 12px;
  left: 0;
  width: 3px;
  border-radius: 999px;
  background: transparent;
  content: '';
}

.cp-palette__row--active {
  background: var(--palette-accent-soft);
  border-color: var(--palette-accent-line);
  box-shadow: 0 8px 22px rgb(var(--color-accent-primary-rgb) / 8%);
}

.cp-palette__row--active::before {
  background: var(--palette-accent);
}

.cp-palette__icon-box {
  display: grid;
  place-items: center;
  width: 28px;
  height: 28px;
  border: 1px solid var(--palette-line);
  border-radius: 9px;
  background: var(--palette-bg-soft);
}

.cp-palette__icon {
  color: var(--palette-ink-muted);
}

.cp-palette__icon--accent {
  color: var(--palette-accent);
}

.cp-palette__main {
  min-width: 0;
}

.cp-palette__label {
  overflow: hidden;
  color: var(--palette-ink);
  font-size: 14.5px;
  font-weight: 550;
  line-height: 1.25;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cp-palette__label--mono {
  font-family: var(--palette-mono);
  font-size: 14px;
}

.cp-palette__sub {
  overflow: hidden;
  margin-top: 3px;
  color: var(--palette-ink-muted);
  font-size: 12.5px;
  line-height: 1.25;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cp-palette__badge {
  align-self: center;
  padding: 3px 7px;
  border: 1px solid var(--palette-line);
  border-radius: 999px;
  background: var(--palette-bg-soft);
  color: var(--palette-ink-muted);
  font-family: var(--palette-mono);
  font-size: 10.5px;
  letter-spacing: 0.6px;
  line-height: 1.2;
  text-transform: uppercase;
  white-space: nowrap;
}

.cp-palette__empty {
  padding: 34px 16px;
  border: 1px dashed var(--palette-line-strong);
  border-radius: 12px;
  color: var(--palette-ink-muted);
  font-size: 13px;
  text-align: center;
}

.cp-palette__foot {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 14px;
  justify-content: flex-start;
  width: 100%;
  color: var(--palette-ink-muted);
  font-family: var(--palette-mono);
  font-size: 11px;
  line-height: 1.5;
}

.cp-palette__kbd {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 22px;
  padding: 1px 6px;
  border: 1px solid var(--palette-line-strong);
  border-radius: 5px;
  background: var(--palette-bg-soft);
  color: var(--palette-ink-soft);
  font-family: var(--palette-mono);
  font-size: 10.5px;
  line-height: 1.4;
  box-shadow: inset 0 -1px 0 rgb(0 0 0 / 8%);
}

@media (width <= 640px) {
  :global(.cp-palette-modal) {
    max-width: calc(100vw - 20px) !important;
  }

  .cp-palette__header {
    flex-direction: column;
    gap: 10px;
  }

  .cp-palette__summary {
    max-width: none;
    text-align: left;
  }

  .cp-palette__row {
    grid-template-columns: 28px minmax(0, 1fr);
  }

  .cp-palette__badge {
    grid-column: 2;
    justify-self: start;
  }
}

@media (prefers-reduced-motion: reduce) {
  .cp-palette__search,
  .cp-palette__row {
    transition: none;
  }
}
</style>
