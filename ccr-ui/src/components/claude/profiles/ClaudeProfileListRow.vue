<!-- 列表密度行：紧凑显示，列含 Claude 多模型摘要 -->
<template>
  <div
    class="cp-row surface-status"
    :class="{
      'cp-row--active': isCurrent,
      'cp-row--off': !isEnabled,
    }"
  >
    <span
      class="cp-row__dot"
      :class="{ 'cp-row__dot--good': isCurrent }"
    />
    <span
      class="cp-row__name"
      :title="profile.name"
    >{{ profile.name }}</span>
    <span class="cp-row__label">{{ profile.description || '—' }}</span>
    <span
      class="cp-row__url"
      :title="baseUrlText"
    >{{ baseUrlText }}</span>
    <span
      class="cp-row__model"
      :title="modelText"
    >{{ modelText }}</span>
    <span class="cp-row__meta">{{ authModeText }}</span>
    <div class="cp-row__tags">
      <span
        v-for="tag in tagList.slice(0, 3)"
        :key="tag"
        class="cp-tag"
      >#{{ tag }}</span>
    </div>
    <div class="cp-row__actions">
      <button
        v-if="!isCurrent && isEnabled"
        type="button"
        class="cp-icon-btn cp-icon-btn--accent"
        :title="$t('claudeProfiles.applyProfile')"
        :aria-label="$t('claudeProfiles.applyProfile')"
        :disabled="disabled"
        @click="emit('apply', profile.name)"
      >
        <SIcon
          name="Play"
          size="w-3 h-3"
        />
      </button>
      <button
        type="button"
        class="cp-icon-btn"
        :title="$t('claudeProfiles.editTooltip')"
        :aria-label="$t('claudeProfiles.editTooltip')"
        :disabled="disabled"
        @click="emit('edit', profile.name)"
      >
        <SIcon
          name="Pencil"
          size="w-3 h-3"
        />
      </button>
      <button
        type="button"
        class="cp-icon-btn cp-icon-btn--danger"
        :title="$t('claudeProfiles.deleteTooltip')"
        :aria-label="$t('claudeProfiles.deleteTooltip')"
        :disabled="disabled"
        @click="emit('delete', profile.name)"
      >
        <SIcon
          name="Trash2"
          size="w-3 h-3"
        />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type { ClaudeProfile } from '@/types'

interface Props {
  profile: ClaudeProfile
  isCurrent: boolean
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
})

const emit = defineEmits<{
  (e: 'apply', name: string): void
  (e: 'edit', name: string): void
  (e: 'delete', name: string): void
}>()

const { t } = useI18n()

const isEnabled = computed(() => props.profile.enabled !== false)
const tagList = computed(() => props.profile.tags ?? [])

const baseUrlText = computed(() => {
  const raw = props.profile.base_url?.trim()
  return raw && raw.length > 0 ? raw : t('claudeProfiles.officialBaseUrl')
})

// Claude 多模型：优先主模型，回退 sonnet/opus/haiku/subagent 映射
const modelText = computed(() => {
  const profile = props.profile
  return profile.model?.trim()
    || profile.default_sonnet_model?.trim()
    || profile.default_opus_model?.trim()
    || profile.default_haiku_model?.trim()
    || profile.subagent_model?.trim()
    || '—'
})

const authModeText = computed(() =>
  props.profile.auth_mode === 'api_key'
    ? t('claudeProfiles.authModeApiKey')
    : t('claudeProfiles.authModeSubscription'),
)
</script>

<style scoped>
.cp-row {
  display: grid;
  grid-template-columns: 12px minmax(120px, 160px) minmax(0, 1.2fr) minmax(0, 1.5fr) minmax(80px, 110px) minmax(80px, 120px) minmax(60px, 1fr) auto;
  gap: 12px;
  align-items: center;
  padding: 9px 14px;

  /* 背景/边框由 surface-status 工具类提供 */
  font-size: 12px;
  color: var(--cp-ink-1);
}

.cp-row--active { border-left: 2px solid var(--cp-accent); }
.cp-row--off { opacity: 0.55; }

.cp-row__dot {
  width: 8px;
  height: 8px;
  border-radius: 999px;
  background: var(--cp-ink-4);
}

.cp-row__dot--good { background: var(--cp-good); }

.cp-row__name {
  font-family: var(--cp-mono);
  color: var(--cp-ink-0);
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cp-row__label {
  color: var(--cp-ink-2);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cp-row__url {
  font-family: var(--cp-mono);
  color: var(--cp-ink-2);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cp-row__model {
  font-family: var(--cp-mono);
  color: var(--cp-accent);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cp-row__meta { color: var(--cp-ink-3); }

.cp-row__tags {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
  min-width: 0;
}

.cp-row__actions {
  display: flex;
  gap: 4px;
}

.cp-tag {
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--cp-bg-3);
  color: var(--cp-ink-1);
  border: 1px solid var(--cp-line-2);
  font-family: var(--cp-mono);
  font-size: 10.5px;
  white-space: nowrap;
}

.cp-icon-btn {
  width: 24px;
  height: 24px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 5px;
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

@media (prefers-reduced-motion: reduce) {
  .cp-icon-btn { transition: none; }
}
</style>
