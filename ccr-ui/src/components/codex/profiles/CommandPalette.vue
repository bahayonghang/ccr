<!--
  ⌘K 命令面板：模糊搜索切换 profile 与执行常用命令。
  仅在 Codex Profiles 页面内挂载，关闭后由父组件释放快捷键监听。
-->
<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="cp-palette-backdrop"
      role="presentation"
      @click="emit('update:open', false)"
    >
      <div
        class="cp-palette"
        role="dialog"
        aria-modal="true"
        :aria-label="$t('codex.profiles.commandPalette.title')"
        @click.stop
      >
        <input
          ref="inputRef"
          v-model="query"
          class="cp-palette__search"
          :placeholder="$t('codex.profiles.commandPalette.placeholder')"
          :aria-label="$t('codex.profiles.commandPalette.placeholder')"
          @keydown="onKeyDown"
        >

        <div class="cp-palette__hint">
          {{ $t('codex.profiles.commandPalette.itemsHint', { count: items.length }) }}
        </div>

        <div
          ref="listRef"
          class="cp-palette__list"
          role="listbox"
          :aria-label="$t('codex.profiles.commandPalette.title')"
        >
          <div
            v-for="(item, i) in items"
            :key="item.id"
            :data-index="i"
            class="cp-palette__row"
            :class="{ 'cp-palette__row--active': i === activeIdx }"
            role="option"
            :aria-selected="i === activeIdx"
            @mouseenter="activeIdx = i"
            @click="fire(item)"
          >
            <SIcon
              :name="item.icon"
              size="w-3.5 h-3.5"
              :class="i === activeIdx ? 'cp-palette__icon--accent' : 'cp-palette__icon'"
            />
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
          <div
            v-if="items.length === 0"
            class="cp-palette__empty"
          >
            {{ $t('codex.profiles.commandPalette.empty') }}
          </div>
        </div>

        <div class="cp-palette__foot">
          <span><kbd class="cp-palette__kbd">↵</kbd> {{ $t('codex.profiles.commandPalette.execute') }}</span>
          <span><kbd class="cp-palette__kbd">↑↓</kbd> {{ $t('codex.profiles.commandPalette.select') }}</span>
          <span><kbd class="cp-palette__kbd">Esc</kbd> {{ $t('codex.profiles.commandPalette.close') }}</span>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
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

const items = computed<PaletteItem[]>(() => {
  const all = [...baseCommands.value, ...switchItems.value]
  const q = query.value.trim().toLowerCase()
  if (!q) return all
  return all.filter(it =>
    `${it.label} ${it.hint ?? ''}`.toLowerCase().includes(q),
  )
})

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
    activeIdx.value = Math.min(items.value.length - 1, activeIdx.value + 1)
    void nextTick(scrollActiveIntoView)
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    activeIdx.value = Math.max(0, activeIdx.value - 1)
    void nextTick(scrollActiveIntoView)
  } else if (event.key === 'Enter') {
    event.preventDefault()
    const item = items.value[activeIdx.value]
    if (item) fire(item)
  } else if (event.key === 'Escape') {
    event.preventDefault()
    emit('update:open', false)
  }
}
</script>

<style scoped>
.cp-palette-backdrop {
  position: fixed;
  inset: 0;
  z-index: 100;
  background: rgb(8 7 5 / 60%);
  backdrop-filter: blur(4px);
  display: grid;
  place-items: start center;
  padding-top: 100px;
}

.cp-palette {
  width: min(560px, calc(100vw - 32px));
  background: var(--cp-bg-1);
  border: 1px solid var(--cp-line-2);
  border-radius: 12px;
  box-shadow: 0 24px 80px rgb(0 0 0 / 60%);
  overflow: hidden;
}

.cp-palette__search {
  width: 100%;
  padding: 14px 18px;
  background: transparent;
  border: none;
  border-bottom: 1px solid var(--cp-line);
  color: var(--cp-ink-0);
  font-size: 14px;
  font-family: inherit;
  outline: none;
}

.cp-palette__search::placeholder { color: var(--cp-ink-4); }

.cp-palette__hint {
  padding: 8px 18px;
  color: var(--cp-ink-4);
  font-family: var(--cp-mono);
  font-size: 10.5px;
  letter-spacing: 1px;
  text-transform: uppercase;
  border-bottom: 1px solid var(--cp-line);
}

.cp-palette__list {
  max-height: 360px;
  overflow-y: auto;
}

.cp-palette__row {
  display: grid;
  grid-template-columns: 20px 1fr auto;
  gap: 12px;
  align-items: center;
  padding: 10px 18px;
  cursor: pointer;
  border-bottom: 1px solid rgb(60 55 40 / 40%);
}

.cp-palette__row--active { background: var(--cp-bg-3); }

.cp-palette__icon { color: var(--cp-ink-3); }
.cp-palette__icon--accent { color: var(--cp-accent); }

.cp-palette__main { min-width: 0; }

.cp-palette__label {
  color: var(--cp-ink-0);
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cp-palette__label--mono { font-family: var(--cp-mono); }

.cp-palette__sub {
  margin-top: 1px;
  color: var(--cp-ink-3);
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cp-palette__badge {
  color: var(--cp-ink-4);
  font-family: var(--cp-mono);
  font-size: 10.5px;
  text-transform: uppercase;
  letter-spacing: 0.8px;
}

.cp-palette__empty {
  padding: 30px;
  text-align: center;
  color: var(--cp-ink-3);
  font-size: 12px;
}

.cp-palette__foot {
  display: flex;
  gap: 14px;
  padding: 8px 18px;
  border-top: 1px solid var(--cp-line);
  color: var(--cp-ink-4);
  font-family: var(--cp-mono);
  font-size: 11px;
}

.cp-palette__kbd {
  padding: 1px 5px;
  border-radius: 3px;
  background: var(--cp-bg-0);
  border: 1px solid var(--cp-line-2);
  color: var(--cp-ink-2);
  font-family: var(--cp-mono);
  font-size: 10.5px;
}
</style>
