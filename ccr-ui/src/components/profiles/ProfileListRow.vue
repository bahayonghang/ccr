<!-- 列表密度行：紧凑显示。平台差异（base_url/model/authMode 解析、编辑图标、操作文案）
     通过 descriptor 注入；busyAction 驱动 apply/delete 的加载态。样式靠视图 --cp-* 继承换肤。 -->
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
    >{{ truncateMiddle(baseUrlText, 20, 12) }}</span>
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
        :title="descriptor.labels.apply"
        :aria-label="descriptor.labels.apply"
        :disabled="disabled"
        @click="emit('apply', profile.name)"
      >
        <SIcon
          :name="busyAction === 'apply' ? 'RefreshCw' : 'Play'"
          size="w-3 h-3"
          :class="{ 'cp-spin': busyAction === 'apply' }"
        />
      </button>
      <button
        type="button"
        class="cp-icon-btn"
        :title="descriptor.labels.edit"
        :aria-label="descriptor.labels.edit"
        :disabled="disabled"
        @click="emit('edit', profile.name)"
      >
        <SIcon
          :name="descriptor.editIcon"
          size="w-3 h-3"
        />
      </button>
      <button
        type="button"
        class="cp-icon-btn cp-icon-btn--danger"
        :title="descriptor.labels.delete"
        :aria-label="descriptor.labels.delete"
        :disabled="disabled"
        @click="emit('delete', profile.name)"
      >
        <SIcon
          :name="busyAction === 'delete' ? 'RefreshCw' : 'Trash2'"
          size="w-3 h-3"
          :class="{ 'cp-spin': busyAction === 'delete' }"
        />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts" generic="T extends ProfileRowProfile">
import { computed } from 'vue'
import SIcon from '@/components/ui/SIcon.vue'
import { truncateMiddle } from '@/utils/text'

/** Row 渲染所需的最小 profile 形状（两平台共有字段） */
export interface ProfileRowProfile {
  name: string
  description?: string | null
  enabled?: boolean | null
  tags?: string[] | null
}

/** 平台注入的行渲染策略：解析展示字段 + 操作文案 + 编辑图标 */
export interface ProfileRowDescriptor<P> {
  /** 解析展示用 base_url（空回退官方文案） */
  baseUrl: (profile: P) => string
  /** 解析展示用主模型（Claude 多模型回退；Codex 单 model） */
  model: (profile: P) => string
  /** 解析展示用 auth 模式标签 */
  authMode: (profile: P) => string
  /** 编辑按钮图标：Claude 'Pencil' / Codex 'Edit2' */
  editIcon: string
  labels: { apply: string; edit: string; delete: string }
}

interface Props {
  profile: T
  descriptor: ProfileRowDescriptor<T>
  isCurrent: boolean
  disabled?: boolean
  /** 进行中的操作（Codex 用，驱动图标转圈）；Claude 省略 → 恒 null */
  busyAction?: 'apply' | 'delete' | null
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  busyAction: null,
})

const emit = defineEmits<{
  (e: 'apply', name: string): void
  (e: 'edit', name: string): void
  (e: 'delete', name: string): void
}>()

const isEnabled = computed(() => props.profile.enabled !== false)
const tagList = computed(() => props.profile.tags ?? [])
const baseUrlText = computed(() => props.descriptor.baseUrl(props.profile))
const modelText = computed(() => props.descriptor.model(props.profile))
const authModeText = computed(() => props.descriptor.authMode(props.profile))
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

.cp-spin { animation: cp-spin 1s linear infinite; }

@keyframes cp-spin { to { transform: rotate(360deg); } }

@media (prefers-reduced-motion: reduce) {
  .cp-icon-btn { transition: none; }
  .cp-spin { animation: none; }
}
</style>
