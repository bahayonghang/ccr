<!-- Profiles 页面顶部标题区：标题/副标题 + （可选命令面板）/重载/导出/添加。
     平台差异通过 icon + backTo + labels + 可选 palette 注入；样式靠视图的 --cp-* 继承换肤。 -->
<template>
  <div class="cp-header">
    <div class="cp-header__intro">
      <div class="cp-header__icon">
        <SIcon
          :name="icon"
          size="w-5 h-5"
        />
      </div>
      <div class="cp-header__text">
        <h1 class="cp-header__title">
          {{ labels.title }}
        </h1>
        <p class="cp-header__subtitle">
          {{ labels.subtitle }}
        </p>
      </div>
    </div>

    <div class="cp-header__actions">
      <RouterLink
        :to="backTo"
        class="cp-header__back"
      >
        <button
          type="button"
          class="cp-btn cp-btn--ghost"
        >
          <SIcon
            name="ArrowLeft"
            size="w-3.5 h-3.5"
          />
          <span>{{ labels.back }}</span>
        </button>
      </RouterLink>

      <template v-if="actionsMenu">
        <!-- actionsMenu 模式：Add / ⌘K / ··· 溢出（Reload / Export / Edit TOML） -->
        <button
          v-if="palette"
          type="button"
          class="cp-btn cp-btn--ghost"
          :class="{ 'cp-btn--palette-open': paletteOpen }"
          :disabled="loading"
          :aria-pressed="paletteOpen"
          aria-haspopup="dialog"
          :title="palette.title"
          @click="emit('openPalette')"
        >
          <SIcon
            name="Command"
            size="w-3.5 h-3.5"
          />
          <span>{{ palette.label }}</span>
          <kbd class="cp-btn__kbd">{{ palette.shortcut }}</kbd>
        </button>

        <div class="cp-menu">
          <button
            ref="menuBtnRef"
            type="button"
            class="cp-btn cp-btn--ghost"
            :disabled="loading"
            :aria-expanded="menuOpen"
            aria-haspopup="menu"
            :aria-label="labels.overflow ?? '···'"
            :title="labels.overflow"
            @click="toggleMenu"
          >
            <SIcon
              name="MenuDots"
              size="w-3.5 h-3.5"
            />
          </button>

          <div
            v-if="menuOpen"
            ref="menuPopRef"
            class="cp-menu__pop"
            role="menu"
            :aria-label="labels.overflow"
            @keydown="onMenuKeydown"
          >
            <button
              type="button"
              role="menuitem"
              class="cp-menu__item"
              :disabled="loading"
              @click="onMenuItem('reload')"
            >
              <SIcon
                name="RefreshCw"
                size="w-3.5 h-3.5"
                :class="{ 'cp-spin': loading }"
              />
              <span>{{ labels.reload }}</span>
            </button>
            <button
              type="button"
              role="menuitem"
              class="cp-menu__item"
              :disabled="exporting || loading"
              @click="onMenuItem('export')"
            >
              <SIcon
                name="Download"
                size="w-3.5 h-3.5"
              />
              <span>{{ labels.export }}</span>
            </button>
            <button
              v-if="labels.source"
              type="button"
              role="menuitem"
              class="cp-menu__item"
              :disabled="loading || sourceDisabled"
              :title="sourceTitle"
              @click="onMenuItem('editSource')"
            >
              <SIcon
                name="FileCode2"
                size="w-3.5 h-3.5"
              />
              <span>{{ labels.source }}</span>
            </button>
          </div>
        </div>

        <button
          type="button"
          class="cp-btn cp-btn--primary"
          :disabled="loading"
          @click="emit('add')"
        >
          <SIcon
            name="Plus"
            size="w-3.5 h-3.5"
          />
          <span>{{ labels.add }}</span>
        </button>
      </template>

      <!-- 旧平铺分支（缺省）：渲染与行为保持原样。
           TODO(profiles-redesign): 集成步骤删除 -->
      <template v-else>
      <button
        v-if="palette"
        type="button"
        class="cp-btn cp-btn--ghost"
        :class="{ 'cp-btn--palette-open': paletteOpen }"
        :disabled="loading"
        :aria-pressed="paletteOpen"
        aria-haspopup="dialog"
        :title="palette.title"
        @click="emit('openPalette')"
      >
        <SIcon
          name="Command"
          size="w-3.5 h-3.5"
        />
        <span>{{ palette.label }}</span>
        <kbd class="cp-btn__kbd">{{ palette.shortcut }}</kbd>
      </button>

      <button
        v-if="labels.source"
        type="button"
        class="cp-btn cp-btn--ghost"
        :disabled="loading || sourceDisabled"
        :title="sourceTitle"
        @click="emit('editSource')"
      >
        <SIcon
          name="FileCode2"
          size="w-3.5 h-3.5"
        />
        <span>{{ labels.source }}</span>
      </button>

      <button
        type="button"
        class="cp-btn cp-btn--ghost"
        :disabled="loading"
        @click="emit('reload')"
      >
        <SIcon
          name="RefreshCw"
          size="w-3.5 h-3.5"
          :class="{ 'cp-spin': loading }"
        />
        <span>{{ labels.reload }}</span>
      </button>

      <button
        type="button"
        class="cp-btn cp-btn--ghost"
        :disabled="exporting || loading"
        @click="emit('export')"
      >
        <SIcon
          name="Download"
          size="w-3.5 h-3.5"
        />
        <span>{{ labels.export }}</span>
      </button>

      <button
        type="button"
        class="cp-btn cp-btn--primary"
        :disabled="loading"
        @click="emit('add')"
      >
        <SIcon
          name="Plus"
          size="w-3.5 h-3.5"
        />
        <span>{{ labels.add }}</span>
      </button>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from 'vue'
import SIcon from '@/components/ui/SIcon.vue'
import { RouterLink } from 'vue-router'

export interface ProfilesHeaderLabels {
  title: string
  subtitle: string
  back: string
  reload: string
  export: string
  add: string
  source?: string
  /** ··· 溢出菜单文案（actionsMenu 模式） */
  overflow?: string
}

/** 命令面板按钮（Codex 用，Claude 省略 → 不渲染） */
export interface ProfilesHeaderPalette {
  label: string
  shortcut: string
  title: string
}

interface Props {
  icon: string
  backTo: string
  labels: ProfilesHeaderLabels
  loading?: boolean
  exporting?: boolean
  palette?: ProfilesHeaderPalette | null
  paletteOpen?: boolean
  sourceDisabled?: boolean
  sourceTitle?: string
  /** 溢出菜单模式：Add / ⌘K / ···；缺省 false 保持现有平铺按钮 */
  actionsMenu?: boolean
}

withDefaults(defineProps<Props>(), {
  loading: false,
  exporting: false,
  palette: null,
  paletteOpen: false,
  sourceDisabled: false,
  sourceTitle: undefined,
  actionsMenu: false,
})

const emit = defineEmits<{
  (e: 'add'): void
  (e: 'export'): void
  (e: 'reload'): void
  (e: 'openPalette'): void
  (e: 'editSource'): void
}>()

/* ========================================================================
 * actionsMenu 模式：··· 溢出菜单
 * 行为契约（与 Filters 弹层一致）：Esc 关闭并还焦触发按钮；外部点击关闭；
 * 打开时焦点进第一项；Tab 在菜单内循环；方向键在选项间移动；选中即关闭。
 * ======================================================================== */

const menuOpen = ref(false)
const menuBtnRef = ref<HTMLButtonElement | null>(null)
const menuPopRef = ref<HTMLElement | null>(null)

const menuItems = () =>
  Array.from(menuPopRef.value?.querySelectorAll<HTMLElement>('[role="menuitem"]:not(:disabled)') ?? [])

const closeMenu = (restoreFocus: boolean) => {
  if (!menuOpen.value) return
  menuOpen.value = false
  if (restoreFocus) menuBtnRef.value?.focus()
}

const toggleMenu = async () => {
  menuOpen.value = !menuOpen.value
  if (menuOpen.value) {
    await nextTick()
    menuItems()[0]?.focus()
  }
}

const onMenuKeydown = (event: KeyboardEvent) => {
  const items = menuItems()
  if (event.key === 'Escape') {
    event.preventDefault()
    event.stopPropagation()
    closeMenu(true)
    return
  }
  if (items.length === 0) return
  const idx = items.indexOf(document.activeElement as HTMLElement)
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    items[(idx + 1) % items.length]?.focus()
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    items[(idx - 1 + items.length) % items.length]?.focus()
  } else if (event.key === 'Tab') {
    const first = items[0]
    const last = items[items.length - 1]
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault()
      last.focus()
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault()
      first.focus()
    }
  }
}

const onDocumentPointerDown = (event: MouseEvent) => {
  if (!menuOpen.value) return
  const target = event.target as Node
  if (menuPopRef.value?.contains(target)) return
  if (menuBtnRef.value?.contains(target)) return
  closeMenu(false)
}

watch(menuOpen, (open) => {
  if (open) document.addEventListener('mousedown', onDocumentPointerDown)
  else document.removeEventListener('mousedown', onDocumentPointerDown)
})

onBeforeUnmount(() => document.removeEventListener('mousedown', onDocumentPointerDown))

const onMenuItem = (action: 'editSource' | 'export' | 'reload') => {
  if (action === 'reload') emit('reload')
  else if (action === 'export') emit('export')
  else emit('editSource')
  closeMenu(true)
}
</script>

<style scoped>
.cp-header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 16px;
  padding: 4px 0 14px;
}

.cp-header__intro {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.cp-header__icon {
  display: grid;
  place-items: center;
  width: 38px;
  height: 38px;
  flex-shrink: 0;
  border-radius: 10px;

  /* 平台识别色：视图可选覆盖 --cp-icon-*，未设置时退回共享 accent */
  background: var(--cp-icon-soft, var(--cp-accent-soft));
  border: 1px solid var(--cp-icon-line, var(--cp-accent-line));
  color: var(--cp-icon-color, var(--cp-accent));
}

.cp-header__text { min-width: 0; }

.cp-header__title {
  font-size: 18px;
  font-weight: 600;
  color: var(--cp-ink-0);
  letter-spacing: -0.2px;
  margin: 0;
  line-height: 1.25;
}

.cp-header__subtitle {
  margin: 2px 0 0;
  font-size: 12px;
  color: var(--cp-ink-3);
}

.cp-header__actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.cp-header__back { display: inline-flex; }

/* 共享按钮样式：靠 --cp-* 令牌着色，每个视图各自设定 accent（Claude 暖中性次色 / Codex 主色） */
.cp-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 12px;
  border-radius: 7px;
  font-size: 12.5px;
  font-weight: 500;
  font-family: inherit;
  background: var(--cp-bg-2);
  border: 1px solid var(--cp-line-2);
  color: var(--cp-ink-1);
  cursor: pointer;
  transition: background 120ms ease, border-color 120ms ease, color 120ms ease;
}

.cp-btn:hover:not(:disabled) {
  background: var(--cp-bg-3);
  color: var(--cp-ink-0);
}

.cp-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.cp-btn--ghost {
  background: transparent;
  border-color: var(--cp-line);
  color: var(--cp-ink-2);
}

.cp-btn--palette-open {
  background: var(--cp-accent-soft);
  border-color: var(--cp-accent-line);
  color: var(--cp-accent);
  box-shadow: inset 0 0 0 1px var(--cp-accent-line);
}

.cp-btn--palette-open .cp-btn__kbd {
  border-color: var(--cp-accent-line);
  color: var(--cp-accent);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
}

.cp-btn--primary {
  background: var(--cp-accent);
  border-color: var(--cp-accent);
  color: var(--cp-on-accent);
  font-weight: 600;
}

.cp-btn--primary:hover:not(:disabled) {
  background: var(--cp-accent-hover);
  border-color: var(--cp-accent-hover);
}

.cp-btn__kbd {
  margin-left: 4px;
  padding: 1px 5px;
  font-family: var(--cp-mono);
  font-size: 10px;
  color: var(--cp-ink-4);
  background: var(--cp-bg-0);
  border: 1px solid var(--cp-line-2);
  border-radius: 3px;
}

.cp-spin { animation: cp-spin 1s linear infinite; }

@keyframes cp-spin { to { transform: rotate(360deg); } }

/* actionsMenu 模式：··· 溢出菜单弹层（锚定触发按钮，右对齐防溢出视口） */
.cp-menu {
  position: relative;
  display: inline-flex;
}

.cp-menu__pop {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  z-index: var(--layer-popover);
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 180px;
  padding: 6px;
  background: var(--cp-bg-1);
  border: 1px solid var(--cp-line-2);
  border-radius: 12px;
  box-shadow: 0 16px 40px rgb(0 0 0 / 22%);
}

.cp-menu__item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 7px 10px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--cp-ink-1);
  font-family: inherit;
  font-size: 0.8125rem;
  text-align: left;
  cursor: pointer;
  transition: background 120ms ease, color 120ms ease;
}

.cp-menu__item:hover:not(:disabled) {
  background: var(--cp-bg-3);
  color: var(--cp-ink-0);
}

.cp-menu__item:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

@media (width <= 720px) {
  .cp-menu__pop {
    position: fixed;
    inset: auto 12px 12px;
  }
}

@media (prefers-reduced-motion: reduce) {
  .cp-btn, .cp-menu__item { transition: none; }
  .cp-spin { animation: none; }
}
</style>
