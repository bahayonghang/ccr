<template>
  <article
    class="cp-card surface-status"
    :class="{ 'cp-card--active': isCurrent, 'cp-card--off': !profile.enabled }"
    :data-profile-name="profile.name"
  >
    <div class="cp-card__head">
      <span
        class="cp-card__dot"
        :class="{ 'cp-card__dot--good': isCurrent }"
      />
      <h3
        class="cp-card__name"
        :title="profile.name"
      >
        {{ profile.name }}
      </h3>
      <span class="cp-card__kind">{{ t(`grok.profiles.profileKinds.${profile.profile_kind}`) }}</span>
      <div class="cp-card__actions">
        <button
          v-if="!isCurrent"
          type="button"
          class="cp-card__apply"
          :disabled="disabled || !profile.enabled"
          @click="emit('apply', profile.name)"
        >
          <SIcon
            :name="busyAction === 'apply' ? 'RefreshCw' : 'Play'"
            size="w-3 h-3"
            :class="{ 'cp-card__spin': busyAction === 'apply' }"
          />
          {{ t('grok.profiles.actions.apply') }}
        </button>
        <span
          v-else
          class="cp-card__current"
        >{{ t('grok.profiles.currentActive') }}</span>
        <div class="cp-card__menu">
          <button
            ref="menuButton"
            type="button"
            class="cp-card__icon-btn"
            :disabled="disabled"
            :aria-expanded="menuOpen"
            :aria-label="t('grok.profiles.overflowMenu')"
            @click="toggleMenu"
          >
            <SIcon
              name="MenuDots"
              size="w-4 h-4"
            />
          </button>
          <div
            v-if="menuOpen"
            ref="menuPanel"
            class="cp-card__menu-pop"
            role="menu"
            @keydown="onMenuKeydown"
          >
            <button
              role="menuitem"
              type="button"
              class="cp-card__menu-item"
              @click="selectAction('edit')"
            >
              <SIcon
                name="Edit2"
                size="w-4 h-4"
              />{{ t('grok.profiles.actions.edit') }}
            </button>
            <button
              role="menuitem"
              type="button"
              class="cp-card__menu-item"
              @click="selectAction('toggle')"
            >
              <SIcon
                :name="profile.enabled ? 'Pause' : 'Play'"
                size="w-4 h-4"
              />
              {{ profile.enabled ? t('grok.profiles.actions.disable') : t('grok.profiles.actions.enable') }}
            </button>
            <button
              role="menuitem"
              type="button"
              class="cp-card__menu-item cp-card__menu-item--danger"
              @click="selectAction('delete')"
            >
              <SIcon
                :name="busyAction === 'delete' ? 'RefreshCw' : 'Trash2'"
                size="w-4 h-4"
              />
              {{ t('grok.profiles.actions.delete') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <p
      v-if="profile.description"
      class="cp-card__desc"
    >
      {{ profile.description }}
    </p>

    <dl class="cp-card__fields">
      <div
        v-for="field in fields"
        :key="field.label"
        class="cp-card__field"
      >
        <dt>{{ field.label }}</dt>
        <dd :title="field.title ?? field.value">
          {{ field.value }}
        </dd>
      </div>
    </dl>

    <div
      v-if="profile.tags.length > 0"
      class="cp-card__tags"
    >
      <span
        v-for="tag in profile.tags"
        :key="tag"
      >#{{ tag }}</span>
    </div>
  </article>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type { GrokProfileDto } from '@/types'
import { GROK_FIELD_PLACEHOLDER, grokAuthModeLabel, resolveGrokBaseUrl } from '@/utils/grokProfiles'
import { formatBaseUrlDisplay } from '@/utils/text'

const props = withDefaults(defineProps<{
  profile: GrokProfileDto
  isCurrent: boolean
  disabled?: boolean
  busyAction?: 'apply' | 'delete' | null
}>(), {
  disabled: false,
  busyAction: null,
})

const emit = defineEmits<{
  apply: [name: string]
  edit: [name: string]
  delete: [name: string]
  toggle: [name: string, enabled: boolean]
}>()

const { t } = useI18n()
const fields = computed(() => [
  {
    label: t('grok.profiles.fields.baseUrl'),
    value: formatBaseUrlDisplay(resolveGrokBaseUrl(props.profile, t)),
    title: resolveGrokBaseUrl(props.profile, t),
  },
  { label: t('grok.profiles.fields.model'), value: props.profile.model || GROK_FIELD_PLACEHOLDER },
  { label: t('grok.profiles.fields.authMode'), value: grokAuthModeLabel(t, props.profile.auth_mode) },
  { label: t('grok.profiles.fields.apiBackend'), value: props.profile.api_backend || GROK_FIELD_PLACEHOLDER },
  { label: t('grok.profiles.fields.reasoningEffort'), value: props.profile.reasoning_effort || GROK_FIELD_PLACEHOLDER },
  {
    label: t('grok.profiles.fields.contextWindow'),
    value: props.profile.context_window?.toLocaleString() || GROK_FIELD_PLACEHOLDER,
  },
])

const menuOpen = ref(false)
const menuButton = ref<HTMLButtonElement | null>(null)
const menuPanel = ref<HTMLElement | null>(null)
const menuItems = () => Array.from(menuPanel.value?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]') ?? [])

const closeMenu = (restoreFocus = false) => {
  menuOpen.value = false
  if (restoreFocus) menuButton.value?.focus()
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
    closeMenu(true)
    return
  }
  if (items.length === 0) return
  const index = items.indexOf(document.activeElement as HTMLButtonElement)
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    items[(index + 1) % items.length]?.focus()
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    items[(index - 1 + items.length) % items.length]?.focus()
  }
}

const onDocumentPointerDown = (event: MouseEvent) => {
  const target = event.target as Node
  if (!menuOpen.value || menuPanel.value?.contains(target) || menuButton.value?.contains(target)) return
  closeMenu()
}

const selectAction = (action: 'edit' | 'toggle' | 'delete') => {
  if (action === 'edit') emit('edit', props.profile.name)
  else if (action === 'toggle') emit('toggle', props.profile.name, !props.profile.enabled)
  else emit('delete', props.profile.name)
  closeMenu(true)
}

watch(menuOpen, (open) => {
  if (open) document.addEventListener('mousedown', onDocumentPointerDown)
  else document.removeEventListener('mousedown', onDocumentPointerDown)
})

onBeforeUnmount(() => document.removeEventListener('mousedown', onDocumentPointerDown))
</script>

<style scoped>
.cp-card {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 0.625rem;
  padding: 0.875rem;
  color: var(--cp-ink-1);
  border-radius: var(--radius-lg);
}

.cp-card--active { border-left: 2px solid var(--cp-accent); }
.cp-card--off { opacity: 0.62; }

.cp-card__head,
.cp-card__actions {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 0.5rem;
}

.cp-card__dot {
  width: 0.5rem;
  height: 0.5rem;
  flex: 0 0 auto;
  background: var(--cp-ink-4);
  border-radius: 999px;
}

.cp-card__dot--good { background: var(--cp-good); }

.cp-card__name {
  min-width: 0;
  flex: 1;
  margin: 0;
  overflow: hidden;
  color: var(--cp-ink-0);
  font-family: var(--cp-mono);
  font-size: 0.9375rem;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cp-card__kind,
.cp-card__current,
.cp-card__tags span {
  color: var(--cp-ink-2);
  font-size: 0.75rem;
}

.cp-card__actions { margin-left: auto; }

.cp-card__apply,
.cp-card__icon-btn,
.cp-card__menu-item {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.375rem;
  color: var(--cp-ink-2);
  background: transparent;
  border: 1px solid var(--cp-line-2);
  border-radius: var(--radius-sm);
  cursor: pointer;
}

.cp-card__apply {
  padding: 0.3rem 0.55rem;
  color: var(--cp-accent);
  font-size: 0.75rem;
}

.cp-card__icon-btn {
  width: 1.75rem;
  height: 1.75rem;
}

.cp-card__apply:disabled,
.cp-card__icon-btn:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.cp-card__menu {
  position: relative;
}

.cp-card__menu-pop {
  position: absolute;
  top: calc(100% + 0.25rem);
  right: 0;
  z-index: var(--layer-popover);
  display: flex;
  min-width: 10rem;
  flex-direction: column;
  gap: 0.125rem;
  padding: 0.25rem;
  background: var(--cp-bg-1);
  border: 1px solid var(--cp-line-2);
  border-radius: var(--radius-md);
  box-shadow: 0 12px 24px rgb(0 0 0 / 16%);
}

.cp-card__menu-item {
  justify-content: flex-start;
  width: 100%;
  padding: 0.45rem 0.55rem;
  border-color: transparent;
  font-size: 0.8125rem;
}

.cp-card__menu-item:hover {
  background: var(--cp-bg-3);
}

.cp-card__menu-item--danger:hover {
  color: var(--cp-danger);
}

.cp-card__desc {
  margin: 0;
  color: var(--cp-ink-2);
  font-size: 0.8125rem;
  line-height: 1.5;
}

.cp-card__fields {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.5rem 1rem;
  margin: 0;
}

.cp-card__field {
  min-width: 0;
}

.cp-card__field dt {
  color: var(--cp-ink-3);
  font-size: 0.75rem;
}

.cp-card__field dd {
  margin: 0.2rem 0 0;
  overflow: hidden;
  color: var(--cp-ink-0);
  font-family: var(--cp-mono);
  font-size: 0.8125rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cp-card__tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
}

.cp-card__tags span {
  padding: 0.1rem 0.4rem;
  background: var(--cp-bg-3);
  border: 1px solid var(--cp-line-2);
  border-radius: var(--radius-sm);
}

.cp-card__spin {
  animation: grok-card-spin 1s linear infinite;
}

@keyframes grok-card-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (prefers-reduced-motion: reduce) {
  .cp-card__spin {
    animation: none;
  }
}
</style>
