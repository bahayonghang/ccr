<!-- Claude Profile 卡片：密排信息卡（名称 + 状态 + provider 色点 + 关键字段）。
     统一消费视图注入的 --cp-* 令牌，不再按 provider 给整卡染色；
     主操作固定为 Apply，编辑/删除收进角落 ··· 菜单（保留 aria-label）。 -->
<template>
  <article
    class="cp-card surface-status"
    :class="{
      'cp-card--active': profile.is_current,
      'cp-card--off': !isEnabled,
    }"
  >
    <div class="cp-card__head">
      <span
        class="cp-card__dot"
        :class="{ 'cp-card__dot--good': profile.is_current }"
      />
      <!-- eslint-disable vue/no-v-html -- highlightedName 由 claudeProfiles 转义后仅注入 <mark> -->
      <h3
        class="cp-card__name"
        :title="profile.name"
        v-html="highlightedName"
      />
      <!-- eslint-enable vue/no-v-html -->

      <span
        class="cp-card__state"
        :class="stateClass"
      >{{ stateLabel }}</span>

      <span
        class="cp-card__provider"
        :title="providerLabel"
      >
        <span
          class="cp-card__provider-dot"
          :style="{ background: `rgb(var(${providerColor.rgbVar}))` }"
        />
        {{ providerLabel }}
      </span>

      <div class="cp-card__actions">
        <button
          v-if="!profile.is_current"
          type="button"
          class="cp-card__apply"
          :disabled="disabled || !isEnabled"
          :title="t('claudeProfiles.applyProfile')"
          @click="emit('apply')"
        >
          <SIcon
            :name="busyAction === 'apply' ? 'RefreshCw' : 'Play'"
            size="w-3 h-3"
            :class="{ 'cp-card__spin': busyAction === 'apply' }"
          />
          {{ t('claudeProfiles.applyProfile') }}
        </button>
        <span
          v-else
          class="cp-card__current"
        >
          <span class="cp-card__current-dot" />
          {{ currentStatusLabel }}
        </span>

        <div class="cp-card__menu">
          <button
            ref="menuBtnRef"
            type="button"
            class="cp-card__icon-btn"
            :disabled="disabled"
            :aria-expanded="menuOpen"
            aria-haspopup="menu"
            :aria-label="t('claudeProfiles.overflowMenu')"
            :title="t('claudeProfiles.overflowMenu')"
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
            class="cp-card__menu-pop"
            role="menu"
            :aria-label="t('claudeProfiles.overflowMenu')"
            @keydown="onMenuKeydown"
          >
            <button
              type="button"
              role="menuitem"
              class="cp-card__menu-item"
              @click="onMenuItem('edit')"
            >
              <SIcon
                name="Pencil"
                size="w-3.5 h-3.5"
              />
              <span>{{ t('claudeProfiles.editTooltip') }}</span>
            </button>
            <button
              type="button"
              role="menuitem"
              class="cp-card__menu-item cp-card__menu-item--danger"
              @click="onMenuItem('delete')"
            >
              <SIcon
                :name="busyAction === 'delete' ? 'RefreshCw' : 'Trash2'"
                size="w-3.5 h-3.5"
                :class="{ 'cp-card__spin': busyAction === 'delete' }"
              />
              <span>{{ t('claudeProfiles.deleteTooltip') }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- eslint-disable vue/no-v-html -- highlightedDescription 由 claudeProfiles 转义后仅注入 <mark> -->
    <p
      class="cp-card__desc"
      v-html="highlightedDescription"
    />
    <!-- eslint-enable vue/no-v-html -->

    <dl class="cp-card__fields">
      <div
        v-for="field in fields"
        :key="field.label"
        class="cp-card__field"
      >
        <dt class="cp-card__field-label">
          {{ field.label }}
        </dt>
        <dd
          class="cp-card__field-value"
          :class="{ 'cp-card__field-value--accent': field.accent }"
          :title="field.title ?? field.value"
        >
          {{ field.value }}
        </dd>
      </div>
    </dl>

    <div
      v-if="tagList.length > 0"
      class="cp-card__tags"
    >
      <span
        v-for="tag in tagList"
        :key="tag"
        class="cp-card__tag"
      >#{{ tag }}</span>
    </div>
  </article>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type { ClaudeProfile } from '@/types'
import {
  claudeAuthModeLabel,
  formatClaudeBaseUrlDisplay,
  highlightSearchMatch,
  resolveClaudePrimaryModel,
  resolveClaudeDisplayBaseUrl,
  resolveProviderColor,
} from '@/utils/claudeProfiles'

interface Props {
  profile: ClaudeProfile
  searchQuery?: string
  disabled?: boolean
  /** 进行中的操作，驱动图标转圈 */
  busyAction?: 'apply' | 'delete' | null
}

const props = withDefaults(defineProps<Props>(), {
  searchQuery: '',
  disabled: false,
  busyAction: null,
})

const emit = defineEmits<{
  edit: []
  delete: []
  apply: []
}>()

const { t } = useI18n()

const isEnabled = computed(() => props.profile.enabled !== false)
const tagList = computed(() => props.profile.tags ?? [])

// provider 只用色点弱表达，不再按 provider 给整卡/按钮染色
const providerColor = computed(() => resolveProviderColor(props.profile.provider))
const providerLabel = computed(
  () => props.profile.provider?.trim() || t('claudeProfiles.providerUnset'),
)

const stateLabel = computed(() => {
  if (props.profile.is_current) {
    return isEnabled.value
      ? t('claudeProfiles.currentBadge')
      : t('claudeProfiles.currentDisabled')
  }
  return isEnabled.value ? t('claudeProfiles.enabledText') : t('claudeProfiles.disabledText')
})

const stateClass = computed(() =>
  isEnabled.value ? 'cp-card__state--on' : 'cp-card__state--off',
)

const currentStatusLabel = computed(() =>
  isEnabled.value ? t('claudeProfiles.currentlyActive') : t('claudeProfiles.currentDisabled'),
)

const highlightedName = computed(() =>
  highlightSearchMatch(props.profile.name, props.searchQuery),
)

const highlightedDescription = computed(() =>
  highlightSearchMatch(
    props.profile.description || t('claudeProfiles.descriptionFallback'),
    props.searchQuery,
  ),
)

interface CardField {
  label: string
  value: string
  title?: string
  accent?: boolean
}

const fields = computed<CardField[]>(() => {
  const fullBaseUrl = resolveClaudeDisplayBaseUrl(props.profile, t)
  const items: CardField[] = [
    {
      label: t('claudeProfiles.fields.baseUrl'),
      value: formatClaudeBaseUrlDisplay(fullBaseUrl),
      title: fullBaseUrl,
    },
    {
      label: t('claudeProfiles.fields.model'),
      value: resolveClaudePrimaryModel(props.profile),
      accent: true,
    },
    {
      label: t('claudeProfiles.fields.authMode'),
      value: claudeAuthModeLabel(t, props.profile.auth_mode),
    },
  ]

  if (props.profile.account?.trim()) {
    items.push({ label: t('claudeProfiles.accountLabel'), value: props.profile.account.trim() })
  }

  return items
})

/* ========================================================================
 * 角落 ··· 菜单：Esc 关闭并还焦触发按钮；外部点击关闭；选中即关闭。
 * ======================================================================== */

const menuOpen = ref(false)
const menuBtnRef = ref<HTMLButtonElement | null>(null)
const menuPopRef = ref<HTMLElement | null>(null)

const menuItems = () =>
  Array.from(menuPopRef.value?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? [])

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

const onMenuItem = (action: 'edit' | 'delete') => {
  if (action === 'edit') emit('edit')
  else emit('delete')
  closeMenu(true)
}
</script>

<style scoped>
.cp-card {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0.75rem 0.875rem;

  /* 背景/边框由 surface-status 工具类提供 */
  border-radius: 12px;
  color: var(--cp-ink-1);
}

.cp-card--active {
  border-left: 2px solid var(--cp-accent);
}

.cp-card--off {
  opacity: 0.6;
}

.cp-card__head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
}

.cp-card__dot {
  width: 0.5rem;
  height: 0.5rem;
  flex-shrink: 0;
  border-radius: 999px;
  background: var(--cp-ink-4);
}

.cp-card__dot--good {
  background: var(--cp-good);
}

.cp-card__name {
  margin: 0;
  min-width: 0;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--cp-mono);
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--cp-ink-0);
}

.cp-card__state {
  flex-shrink: 0;
  padding: 0.0625rem 0.4375rem;
  border-radius: 999px;
  border: 1px solid var(--cp-line-2);
  font-size: 0.75rem;
}

.cp-card__state--on {
  color: var(--cp-ink-2);
}

.cp-card__state--off {
  color: var(--cp-danger);
  border-color: rgb(var(--color-danger-rgb) / 30%);
}

.cp-card__provider {
  display: inline-flex;
  align-items: center;
  gap: 0.3125rem;
  flex-shrink: 0;
  max-width: 10rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--cp-ink-3);
  font-size: 0.75rem;
}

.cp-card__provider-dot {
  width: 0.375rem;
  height: 0.375rem;
  flex-shrink: 0;
  border-radius: 999px;
}

.cp-card__actions {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  margin-left: auto;
}

.cp-card__apply {
  display: inline-flex;
  align-items: center;
  gap: 0.3125rem;
  padding: 0.25rem 0.5625rem;
  border-radius: 6px;
  border: 1px solid var(--cp-accent-line);
  background: var(--cp-accent-soft);
  color: var(--cp-accent);
  font-family: inherit;
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: background 120ms ease, border-color 120ms ease;
}

.cp-card__apply:hover:not(:disabled) {
  background: var(--cp-accent);
  color: var(--cp-on-accent);
}

.cp-card__apply:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.cp-card__current {
  display: inline-flex;
  align-items: center;
  gap: 0.3125rem;
  padding: 0.25rem 0.5625rem;
  border-radius: 6px;
  color: var(--cp-good);
  font-size: 0.75rem;
  font-weight: 500;
}

.cp-card__current-dot {
  width: 0.375rem;
  height: 0.375rem;
  border-radius: 999px;
  background: var(--cp-good);
}

.cp-card__icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  border-radius: 6px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--cp-ink-3);
  cursor: pointer;
  transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
}

.cp-card__icon-btn:hover:not(:disabled) {
  background: var(--cp-bg-3);
  border-color: var(--cp-line-2);
  color: var(--cp-ink-0);
}

.cp-card__icon-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.cp-card__menu {
  position: relative;
}

.cp-card__menu-pop {
  position: absolute;
  top: calc(100% + 0.25rem);
  right: 0;
  z-index: 20;
  min-width: 8rem;
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  padding: 0.25rem;
  border-radius: 8px;
  border: 1px solid var(--cp-line-2);
  background: var(--cp-bg-1);
  box-shadow: 0 12px 24px rgb(0 0 0 / 12%);
}

.cp-card__menu-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.375rem 0.5rem;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--cp-ink-1);
  font-family: inherit;
  font-size: 0.8125rem;
  text-align: left;
  cursor: pointer;
}

.cp-card__menu-item:hover {
  background: var(--cp-bg-3);
  color: var(--cp-ink-0);
}

.cp-card__menu-item--danger:hover {
  color: var(--cp-danger);
}

.cp-card__desc {
  margin: 0;
  overflow: hidden;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  color: var(--cp-ink-2);
  font-size: 0.8125rem;
  line-height: 1.45;
}

.cp-card__fields {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(9rem, 1fr));
  gap: 0.375rem 0.75rem;
  margin: 0;
}

.cp-card__field {
  min-width: 0;
}

.cp-card__field-label {
  font-family: var(--cp-mono);
  font-size: 0.75rem;
  letter-spacing: 0.0625rem;
  text-transform: uppercase;
  color: var(--cp-ink-3);
}

.cp-card__field-value {
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--cp-mono);
  font-size: 0.8125rem;
  color: var(--cp-ink-0);
}

.cp-card__field-value--accent {
  color: var(--cp-accent);
}

.cp-card__tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.cp-card__tag {
  padding: 0.0625rem 0.375rem;
  border-radius: 4px;
  border: 1px solid var(--cp-line-2);
  background: var(--cp-bg-3);
  color: var(--cp-ink-2);
  font-family: var(--cp-mono);
  font-size: 0.75rem;
  white-space: nowrap;
}

.cp-card__spin {
  animation: cp-card-spin 1s linear infinite;
}

@keyframes cp-card-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (prefers-reduced-motion: reduce) {
  .cp-card__apply,
  .cp-card__icon-btn {
    transition: none;
  }

  .cp-card__spin {
    animation: none;
  }
}
</style>
