<!--
  Codex Profile 单列宽卡片：只读视图 + 行动 icon。
  编辑动作通过 emits 触发外层调用现有 CodexProfileEditorModal，不在此组件内编辑。
-->
<template>
  <article
    class="cp-card surface-card"
    :class="{
      'cp-card--active': isCurrent,
      'cp-card--off': !isEnabled,
    }"
    :data-profile-name="profile.name"
  >
    <span
      v-if="isCurrent"
      class="cp-card__active-bar"
      aria-hidden="true"
    />

    <header class="cp-card__head">
      <h3 class="cp-card__name">
        {{ profile.name }}
      </h3>

      <span
        class="cp-card__badge"
        :class="isEnabled ? 'cp-card__badge--enabled' : 'cp-card__badge--off'"
      >
        {{ isEnabled ? $t('codex.states.enabled') : $t('codex.states.disabled') }}
      </span>

      <span
        v-if="isCurrent"
        class="cp-card__badge cp-card__badge--current"
      >
        {{ $t('codex.profiles.currentBadge') }}
      </span>

      <p
        v-if="profile.description"
        class="cp-card__label"
      >
        {{ profile.description }}
      </p>

      <div class="cp-card__actions">
        <button
          v-if="!isCurrent && isEnabled"
          type="button"
          class="cp-icon-btn cp-icon-btn--accent"
          :title="$t('codex.profiles.apply')"
          :aria-label="$t('codex.profiles.apply')"
          :disabled="disabled"
          @click="emit('apply', profile.name)"
        >
          <SIcon
            :name="busyAction === 'apply' ? 'RefreshCw' : 'Play'"
            size="w-3.5 h-3.5"
            :class="{ 'cp-spin': busyAction === 'apply' }"
          />
        </button>
        <button
          type="button"
          class="cp-icon-btn"
          :title="$t('codex.actions.edit')"
          :aria-label="$t('codex.actions.edit')"
          :disabled="disabled"
          @click="emit('edit', profile.name)"
        >
          <SIcon
            name="Edit2"
            size="w-3.5 h-3.5"
          />
        </button>
        <button
          type="button"
          class="cp-icon-btn cp-icon-btn--danger"
          :title="$t('codex.actions.delete')"
          :aria-label="$t('codex.actions.delete')"
          :disabled="disabled"
          @click="emit('delete', profile.name)"
        >
          <SIcon
            :name="busyAction === 'delete' ? 'RefreshCw' : 'Trash2'"
            size="w-3.5 h-3.5"
            :class="{ 'cp-spin': busyAction === 'delete' }"
          />
        </button>
      </div>
    </header>

    <div class="cp-card__fields">
      <div class="cp-field">
        <div class="cp-field__label">
          {{ $t('codex.profiles.fields.baseUrl') }}
        </div>
        <div
          class="cp-field__value"
          :title="profileBaseUrlText"
        >
          {{ truncatedBaseUrl }}
        </div>
      </div>

      <div class="cp-field">
        <div class="cp-field__label">
          {{ $t('codex.profiles.fields.model') }}
        </div>
        <div class="cp-field__value cp-field__value--accent">
          {{ profile.model || '—' }}
        </div>
      </div>

      <div class="cp-field">
        <div class="cp-field__label">
          {{ $t('codex.profiles.fields.authMode') }}
        </div>
        <div class="cp-field__value cp-field__value--muted">
          <span>{{ authModeText }}</span>
          <span
            v-if="profile.openai_login_method"
            class="cp-tag cp-tag--good"
          >{{ profile.openai_login_method }}</span>
        </div>
      </div>

      <div class="cp-field">
        <div class="cp-field__label">
          {{ $t('codex.profiles.fields.authSource') }}
        </div>
        <div class="cp-field__value cp-field__value--muted">
          <span>{{ profile.auth_source || $t('codex.profiles.notAvailable') }}</span>
          <span
            v-if="profile.credential_store"
            class="cp-tag cp-tag--info"
          >{{ profile.credential_store }}</span>
        </div>
      </div>

      <div
        v-if="profile.env_key"
        class="cp-field cp-field--full"
      >
        <div class="cp-field__label">
          {{ $t('codex.profiles.fields.envKey') }}
        </div>
        <div class="cp-field__value">
          {{ profile.env_key }}
        </div>
      </div>
    </div>

    <div
      v-if="profile.shell_export_script"
      class="cp-export"
    >
      <div class="cp-export__head">
        <div>
          <div class="cp-export__title">
            {{ $t('codex.profiles.envExportTitle') }}
          </div>
          <div class="cp-export__hint">
            {{ $t('codex.profiles.envExportHint') }}
          </div>
        </div>
        <button
          type="button"
          class="cp-btn cp-btn--ghost cp-btn--xs"
          @click="emit('copyEnv', profile)"
        >
          <SIcon
            name="Copy"
            size="w-3.5 h-3.5"
          />
          <span>{{ $t('codex.profiles.copyEnvExport') }}</span>
        </button>
      </div>
      <pre class="cp-export__code"><code>{{ profile.shell_export_script }}</code></pre>
    </div>

    <footer
      v-if="hasFooter"
      class="cp-card__foot"
    >
      <div
        v-if="tagList.length > 0"
        class="cp-tags"
      >
        <span
          v-for="tag in tagList"
          :key="tag"
          class="cp-tag"
        >#{{ tag }}</span>
      </div>
      <div class="cp-card__meta">
        <span
          v-if="profile.provider"
          class="cp-tag cp-tag--info"
        >{{ profile.provider }}</span>
        <span
          v-if="typeof profile.usage_count === 'number'"
          class="cp-card__meta-item"
          :title="$t('codex.profiles.toolbar.sortRequests')"
        >
          <SIcon
            name="BarChart3"
            size="w-3 h-3"
          />
          {{ profile.usage_count.toLocaleString() }}
        </span>
      </div>
    </footer>
  </article>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type { CodexProfile, CodexProfileAuthMode } from '@/types'
import { truncateMiddle } from '@/utils/text'

interface Props {
  profile: CodexProfile
  isCurrent: boolean
  busyAction?: 'apply' | 'delete' | null
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  busyAction: null,
  disabled: false,
})

const emit = defineEmits<{
  (e: 'apply', name: string): void
  (e: 'edit', name: string): void
  (e: 'delete', name: string): void
  (e: 'copyEnv', profile: CodexProfile): void
}>()

const { t } = useI18n()

const isEnabled = computed(() => props.profile.enabled !== false)

const profileBaseUrlText = computed(() => {
  const raw = props.profile.base_url?.trim()
  return raw && raw.length > 0 ? raw : t('codex.profiles.officialBaseUrl')
})

const truncatedBaseUrl = computed(() => truncateMiddle(profileBaseUrlText.value, 24, 14))

const authModeText = computed(() => {
  const mode = props.profile.auth_mode as CodexProfileAuthMode | undefined
  return t(`codex.profiles.authModes.${mode || 'no_auth'}`)
})

const tagList = computed(() => props.profile.tags ?? [])

const hasFooter = computed(
  () =>
    tagList.value.length > 0
    || Boolean(props.profile.provider)
    || typeof props.profile.usage_count === 'number',
)
</script>

<style scoped>
.cp-card {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px 18px;

  /* 背景/边框由 surface-card 工具类提供 */
  border-radius: 12px;
  transition: border-color 120ms ease, box-shadow 120ms ease;
}

.cp-card:hover .cp-card__actions { opacity: 1; }
.cp-card:focus-within .cp-card__actions { opacity: 1; }

.cp-card--active {
  border-color: var(--cp-accent-line);
  box-shadow: 0 0 0 1px rgb(var(--color-accent-primary-rgb) / 15%), 0 4px 18px rgb(0 0 0 / 25%);
}

.cp-card--off { opacity: 0.62; }

.cp-card__active-bar {
  position: absolute;
  left: 0;
  top: 14px;
  bottom: 14px;
  width: 3px;
  border-radius: 999px;
  background: var(--cp-accent);
}

.cp-card__head {
  display: grid;
  grid-template-columns: auto auto auto 1fr auto;
  align-items: center;
  gap: 10px;
}

.cp-card__name {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  font-family: var(--cp-mono);
  color: var(--cp-ink-0);
  letter-spacing: -0.3px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.cp-card__badge {
  flex-shrink: 0;
  padding: 2px 7px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.4px;
  text-transform: uppercase;
  white-space: nowrap;
}

.cp-card__badge--enabled {
  background: rgb(var(--color-success-rgb) / 15%);
  color: var(--cp-good);
  border: 1px solid rgb(var(--color-success-rgb) / 30%);
}

.cp-card__badge--off {
  background: var(--cp-bg-3);
  color: var(--cp-ink-3);
  border: 1px solid var(--cp-line-2);
}

.cp-card__badge--current {
  background: var(--cp-accent-soft);
  color: var(--cp-accent);
  border: 1px solid var(--cp-accent-line);
}

.cp-card__label {
  margin: 0;
  grid-column: 1 / -2;
  margin-top: 4px;
  color: var(--cp-ink-2);
  font-size: 12.5px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cp-card__actions {
  grid-column: -1;
  display: flex;
  gap: 4px;
  margin-left: auto;
  opacity: 0.55;
  transition: opacity 120ms ease;
}

.cp-icon-btn {
  width: 28px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--cp-ink-3);
  cursor: pointer;
  transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
}

.cp-icon-btn:hover:not(:disabled) {
  background: var(--cp-bg-3);
  border-color: var(--cp-line-2);
  color: var(--cp-ink-0);
}

.cp-icon-btn--accent:hover:not(:disabled) { color: var(--cp-accent); }
.cp-icon-btn--danger:hover:not(:disabled) { color: var(--cp-danger); }

.cp-icon-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.cp-card__fields {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px 12px;
}

.cp-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}
.cp-field--full { grid-column: 1 / -1; }

.cp-field__label {
  color: var(--cp-ink-3);
  font-family: var(--cp-mono);
  font-size: 10px;
  letter-spacing: 1px;
  text-transform: uppercase;
}

.cp-field__value {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  color: var(--cp-ink-0);
  font-family: var(--cp-mono);
  font-size: 12px;
  word-break: break-all;
}

.cp-field__value--accent { color: var(--cp-accent); }
.cp-field__value--muted { color: var(--cp-ink-2); }

.cp-card__foot {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  padding-top: 10px;
  border-top: 1px solid var(--cp-line);
}

.cp-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.cp-tag {
  padding: 2px 7px;
  border-radius: 4px;
  background: var(--cp-bg-3);
  color: var(--cp-ink-1);
  border: 1px solid var(--cp-line-2);
  font-family: var(--cp-mono);
  font-size: 10.5px;
  white-space: nowrap;
}

.cp-tag--good {
  background: rgb(var(--color-success-rgb) / 12%);
  color: var(--cp-good);
  border-color: rgb(var(--color-success-rgb) / 35%);
}

.cp-tag--info {
  background: rgb(var(--color-info-rgb) / 12%);
  color: var(--cp-info);
  border-color: rgb(var(--color-info-rgb) / 35%);
}

.cp-card__meta {
  margin-left: auto;
  display: flex;
  gap: 12px;
  align-items: center;
  color: var(--cp-ink-3);
  font-size: 11px;
  font-family: var(--cp-mono);
}

.cp-card__meta-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.cp-export {
  border: 1px solid rgb(var(--color-success-rgb) / 25%);
  background: rgb(var(--color-success-rgb) / 6%);
  border-radius: 10px;
  padding: 12px;
}

.cp-export__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.cp-export__title {
  color: var(--cp-good);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.5px;
  text-transform: uppercase;
}

.cp-export__hint {
  margin-top: 2px;
  color: var(--cp-ink-3);
  font-size: 11px;
}

.cp-export__code {
  margin: 10px 0 0;
  padding: 10px 12px;
  border-radius: 8px;
  background: var(--cp-bg-0);
  border: 1px solid var(--cp-line);
  color: var(--cp-ink-0);
  font-family: var(--cp-mono);
  font-size: 11.5px;
  overflow-x: auto;
  white-space: pre;
}

.cp-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  font-family: inherit;
  background: var(--cp-bg-2);
  border: 1px solid var(--cp-line-2);
  color: var(--cp-ink-1);
  cursor: pointer;
  transition: background 120ms ease, color 120ms ease;
}

.cp-btn--ghost {
  background: transparent;
  border-color: var(--cp-line);
  color: var(--cp-ink-2);
}

.cp-btn--xs {
  padding: 4px 8px;
  font-size: 11px;
}

.cp-btn:hover {
  background: var(--cp-bg-3);
  color: var(--cp-ink-0);
}

.cp-spin { animation: cp-spin 1s linear infinite; }

@keyframes cp-spin { to { transform: rotate(360deg); } }

@media (prefers-reduced-motion: reduce) {
  .cp-card, .cp-icon-btn, .cp-btn { transition: none; }
  .cp-spin { animation: none; }
}

@media (width <= 720px) {
  .cp-card__fields { grid-template-columns: minmax(0, 1fr); }
  .cp-card__head { grid-template-columns: auto auto 1fr auto; }
}
</style>
