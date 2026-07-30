<!--
  Profiles 右侧检查器：
  1) Profile 预览面板（hover/focus 驱动，目标 ≠ 当前时展示与当前的 diff 高亮）
  2) Health Audit（问题项点击 → @locate 定位卡片，而非直接开编辑器）
  3) Distribution（<details> 默认折叠；tag cloud 可点击 → @tag-select 写筛选）
  仅 ≥1280px 视口可见；预览区 aria-live="polite" 仅报 profile 名，不打断屏幕阅读器。
  平台差异通过 i18nPrefix + descriptor 注入；样式靠视图的 --cp-* 继承换肤。
-->
<template>
  <aside
    class="cp-inspector"
    :aria-label="t(`${i18nPrefix}.ariaLabel`)"
  >
    <!-- ========================================================
         面板 1：Profile 预览（hover/focus 驱动 + diff 高亮）
         ======================================================== -->
    <section
      class="cp-inspector-card surface-card"
      :aria-labelledby="previewHeadingId"
    >
      <header class="cp-inspector-card__head">
        <SIcon
          name="Sparkles"
          size="w-3.5 h-3.5"
          class="cp-inspector-card__icon"
        />
        <h3
          :id="previewHeadingId"
          class="cp-inspector-card__title"
        >
          {{ t(`${i18nPrefix}.previewTitle`) }}
        </h3>
        <span
          v-if="isPreviewingCurrent && previewProfile"
          class="cp-inspector-badge"
        >{{ t(`${i18nPrefix}.currentBadge`) }}</span>
      </header>

      <!-- 屏幕阅读器只报预览目标名，字段变化不播报 -->
      <span
        class="sr-only"
        aria-live="polite"
      >{{ previewProfile?.name ?? '' }}</span>

      <div
        v-if="previewProfile"
        class="cp-inspector-preview"
      >
        <div class="cp-inspector-preview__name">
          {{ previewProfile.name }}
        </div>
        <p
          v-if="previewProfile.description"
          class="cp-inspector-preview__desc"
        >
          {{ previewProfile.description }}
        </p>

        <dl class="cp-inspector-fields">
          <div
            v-for="field in previewFields"
            :key="field.label"
            class="cp-inspector-field"
          >
            <dt class="cp-inspector-field__label">
              {{ field.label }}
            </dt>
            <dd
              class="cp-inspector-field__value"
              :class="{
                'cp-inspector-field__value--accent': field.variant === 'accent',
                'cp-inspector-field__value--muted': field.variant === 'muted',
              }"
            >
              {{ field.value }}
            </dd>
          </div>
        </dl>

        <div
          v-if="diffRows.length > 0"
          class="cp-inspector-diff"
        >
          <div class="cp-inspector-section__head">
            {{ t(`${i18nPrefix}.diffTitle`) }}
          </div>
          <ProfileDiffRows :rows="diffRows" />
        </div>

        <div
          v-if="previewTags.length > 0"
          class="cp-inspector-tags"
        >
          <span
            v-for="tag in previewTags"
            :key="tag"
            class="cp-inspector-tag"
          >#{{ tag }}</span>
        </div>

        <div
          v-if="sessionWriteAt"
          class="cp-inspector-session"
        >
          {{ t(`${i18nPrefix}.sessionWrite`, { time: sessionWriteAt }) }}
        </div>

        <button
          type="button"
          class="cp-inspector-action"
          @click="emit('edit', previewProfile.name)"
        >
          <SIcon
            :name="descriptor.editIcon"
            size="w-3.5 h-3.5"
          />
          <span>{{ t(`${i18nPrefix}.editAction`) }}</span>
        </button>
      </div>

      <div
        v-else
        class="cp-inspector-empty"
      >
        <SIcon
          name="Folder"
          size="w-4 h-4"
        />
        <div class="cp-inspector-empty__title">
          {{ t(`${i18nPrefix}.previewEmpty`) }}
        </div>
        <div class="cp-inspector-empty__hint">
          {{ t(`${i18nPrefix}.previewEmptyHint`) }}
        </div>
      </div>
    </section>

    <!-- ========================================================
         面板 2：Health Audit（点击 → @locate 定位卡片）
         ======================================================== -->
    <section
      class="cp-inspector-card surface-card"
      :aria-labelledby="auditHeadingId"
    >
      <header class="cp-inspector-card__head">
        <SIcon
          name="ShieldCheck"
          size="w-3.5 h-3.5"
          class="cp-inspector-card__icon"
        />
        <h3
          :id="auditHeadingId"
          class="cp-inspector-card__title"
        >
          {{ t(`${i18nPrefix}.auditTitle`) }}
        </h3>
        <span
          class="cp-inspector-card__count"
          :class="{ 'cp-inspector-card__count--warn': totalIssueCount > 0 }"
        >
          {{ totalIssueCount }}
        </span>
      </header>

      <div
        v-if="totalIssueCount === 0"
        class="cp-inspector-clean"
      >
        <SIcon
          name="CheckCircle"
          size="w-4 h-4"
        />
        <span>{{ t(`${i18nPrefix}.auditClean`) }}</span>
      </div>

      <ul
        v-else
        class="cp-inspector-issues"
      >
        <!-- 已弃用 auth（Codex 专属；Claude deprecatedAuthIssues 恒空 → 不渲染） -->
        <li
          v-for="profile in deprecatedAuthIssues"
          :key="`dep-${profile.name}`"
        >
          <button
            type="button"
            class="cp-inspector-issue cp-inspector-issue--warn"
            :aria-label="t(`${i18nPrefix}.locateAction`, { name: profile.name })"
            @click="emit('locate', profile.name)"
          >
            <SIcon
              name="AlertCircle"
              size="w-3.5 h-3.5"
              class="cp-inspector-issue__icon"
            />
            <div class="cp-inspector-issue__body">
              <div class="cp-inspector-issue__name">
                {{ profile.name }}
              </div>
              <div class="cp-inspector-issue__msg">
                {{ descriptor.deprecatedMessage?.(profile) }}
              </div>
            </div>
            <SIcon
              name="Target"
              size="w-3.5 h-3.5"
              class="cp-inspector-issue__locate"
            />
          </button>
        </li>

        <li
          v-for="issue in missingFieldIssues"
          :key="`miss-${issue.profile.name}`"
        >
          <button
            type="button"
            class="cp-inspector-issue cp-inspector-issue--danger"
            :aria-label="t(`${i18nPrefix}.locateAction`, { name: issue.profile.name })"
            @click="emit('locate', issue.profile.name)"
          >
            <SIcon
              name="AlertTriangle"
              size="w-3.5 h-3.5"
              class="cp-inspector-issue__icon"
            />
            <div class="cp-inspector-issue__body">
              <div class="cp-inspector-issue__name">
                {{ issue.profile.name }}
              </div>
              <div class="cp-inspector-issue__msg">
                {{ descriptor.missingMessage(issue.missing) }}
              </div>
            </div>
            <SIcon
              name="Target"
              size="w-3.5 h-3.5"
              class="cp-inspector-issue__locate"
            />
          </button>
        </li>

        <li
          v-for="group in duplicateRuntimeIssues"
          :key="`dup-${group.key}`"
          class="cp-inspector-issue-group"
        >
          <div class="cp-inspector-issue-group__head">
            <SIcon
              name="Copy"
              size="w-3.5 h-3.5"
              class="cp-inspector-issue__icon"
            />
            <span>
              {{ t(`${i18nPrefix}.issues.duplicateRuntime`, { count: group.profiles.length }) }}
            </span>
          </div>
          <button
            v-for="profile in group.profiles"
            :key="`dup-${group.key}-${profile.name}`"
            type="button"
            class="cp-inspector-issue cp-inspector-issue--info cp-inspector-issue--nested"
            :aria-label="t(`${i18nPrefix}.locateAction`, { name: profile.name })"
            @click="emit('locate', profile.name)"
          >
            <div class="cp-inspector-issue__body">
              <div class="cp-inspector-issue__name">
                {{ profile.name }}
              </div>
              <div class="cp-inspector-issue__msg cp-inspector-issue__msg--mono">
                {{ descriptor.runtimeSummary(profile) }}
              </div>
            </div>
            <SIcon
              name="Target"
              size="w-3.5 h-3.5"
              class="cp-inspector-issue__locate"
            />
          </button>
        </li>
      </ul>
    </section>

    <!-- ========================================================
         面板 3：Distribution（默认折叠；tag cloud 可点击写筛选）
         ======================================================== -->
    <details class="cp-inspector-card cp-inspector-details surface-card">
      <summary
        class="cp-inspector-card__head cp-inspector-details__summary"
        :aria-label="t(`${i18nPrefix}.distributionTitle`)"
      >
        <SIcon
          name="BarChart3"
          size="w-3.5 h-3.5"
          class="cp-inspector-card__icon"
        />
        <span class="cp-inspector-card__title">
          {{ t(`${i18nPrefix}.distributionTitle`) }}
        </span>
        <SIcon
          name="ChevronDown"
          size="w-3.5 h-3.5"
          class="cp-inspector-details__chevron"
        />
      </summary>

      <div class="cp-inspector-details__body">
        <!-- Provider -->
        <div class="cp-inspector-section">
          <div class="cp-inspector-section__head">
            {{ t(`${i18nPrefix}.providerSection`) }}
          </div>
          <ul
            v-if="providerBreakdown.length > 0"
            class="cp-inspector-bars"
            role="presentation"
          >
            <li
              v-for="item in providerBreakdown"
              :key="item.provider"
              class="cp-inspector-bar"
            >
              <div class="cp-inspector-bar__label">
                {{
                  item.provider === 'Unknown'
                    ? t(`${i18nPrefix}.unknownProvider`)
                    : item.provider
                }}
              </div>
              <div class="cp-inspector-bar__track">
                <div
                  class="cp-inspector-bar__fill"
                  :style="{ width: `${Math.max(item.pct, 4)}%` }"
                />
              </div>
              <div class="cp-inspector-bar__value">
                {{ item.count }}
              </div>
            </li>
          </ul>
          <div
            v-else
            class="cp-inspector-section__empty"
          >
            —
          </div>
        </div>

        <!-- Auth 模式（隐藏 0 值条目） -->
        <div
          v-if="visibleAuthModeBreakdown.length > 0"
          class="cp-inspector-section"
        >
          <div class="cp-inspector-section__head">
            {{ t(`${i18nPrefix}.authSection`) }}
          </div>
          <ul
            class="cp-inspector-bars"
            role="presentation"
          >
            <li
              v-for="item in visibleAuthModeBreakdown"
              :key="item.mode"
              class="cp-inspector-bar"
            >
              <div class="cp-inspector-bar__label">
                {{ descriptor.authModeLabel(item.mode) }}
              </div>
              <div class="cp-inspector-bar__track">
                <div
                  class="cp-inspector-bar__fill"
                  :class="{ 'cp-inspector-bar__fill--warn': descriptor.isDeprecatedMode(item.mode) }"
                  :style="{ width: `${Math.max(item.pct, 4)}%` }"
                />
              </div>
              <div class="cp-inspector-bar__value">
                {{ item.count }}
              </div>
            </li>
          </ul>
        </div>

        <!-- Top Tags（可点击 → @tag-select） -->
        <div class="cp-inspector-section">
          <div class="cp-inspector-section__head">
            {{ t(`${i18nPrefix}.tagsSection`) }}
          </div>
          <div
            v-if="topTags.length > 0"
            class="cp-inspector-tagcloud"
          >
            <button
              v-for="item in topTags"
              :key="item.tag"
              type="button"
              class="cp-inspector-tag cp-inspector-tag--count cp-inspector-tag--clickable"
              :aria-pressed="item.tag === selectedTag"
              @click="emit('tag-select', item.tag)"
            >
              <span>#{{ item.tag }}</span>
              <span class="cp-inspector-tag__count">{{ item.count }}</span>
            </button>
          </div>
          <div
            v-else
            class="cp-inspector-section__empty"
          >
            {{ t(`${i18nPrefix}.noTags`) }}
          </div>
        </div>
      </div>
    </details>
  </aside>
</template>

<script setup lang="ts" generic="T extends ProfilesInspectorProfile">
import { computed, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import ProfileDiffRows from '@/components/profiles/ProfileDiffRows.vue'
import type { ProfilesInsights } from '@/composables/useProfilesInsights'
import { buildProfileDiff, type ProfileDiffField } from '@/utils/profileDiff'

/** Inspector 直接读取的最小 profile 形状（两平台共有字段） */
export interface ProfilesInspectorProfile {
  name: string
  description?: string | null
  tags?: string[] | null
}

/** 预览面板的单个字段（平台决定字段集合/顺序/样式） */
export interface ProfilesInspectorField {
  label: string
  value: string
  variant?: 'accent' | 'muted'
}

/** 平台注入的检查器策略：洞察来源 + 字段列表 + diff 字段 + 文案 + 图标 */
export interface ProfilesInspectorDescriptor<P extends ProfilesInspectorProfile> {
  /** 编辑按钮图标：Claude 'Pencil' / Codex 'Edit2' */
  editIcon: string
  /** 平台洞察 composable（在组件 setup 内调用一次） */
  useInsights: (profiles: Ref<P[]>) => ProfilesInsights<P, string, string>
  /** 预览 profile 的字段列表（previewProfile 非空时调用） */
  activeFields: (profile: P) => ProfilesInspectorField[]
  /** 参与「当前 → 预览目标」diff 的字段（base_url/model/auth_mode 三行） */
  diffFields: readonly ProfileDiffField<P>[]
  /** auth 分布条标签 */
  authModeLabel: (mode: string) => string
  /** 该 auth 模式是否弃用（Claude 恒 false → 不加 warn 类） */
  isDeprecatedMode: (mode: string) => boolean
  /** 缺失字段消息（已 join） */
  missingMessage: (missing: string[]) => string
  /** 重复运行时条目摘要 */
  runtimeSummary: (profile: P) => string
  /** 已弃用 auth 条目消息（Codex 提供；Claude 无弃用概念，可省略） */
  deprecatedMessage?: (profile: P) => string
}

interface Props {
  profiles: T[]
  /** 预览目标（视图按 hoveredName ?? focusedName ?? current 解析后传入） */
  previewProfile: T | null
  /** 当前激活 profile（diff 的 from 侧） */
  currentProfile: T | null
  /** i18n key 前缀，指向 inspector 子对象，例如 'claudeProfiles.inspector' */
  i18nPrefix: string
  descriptor: ProfilesInspectorDescriptor<T>
  /** 本次会话最近一次写入时间（仅预览目标=当前时由视图传入） */
  sessionWriteAt?: string | null
  /** 当前生效的标签筛选（tag cloud aria-pressed 同步） */
  selectedTag?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  sessionWriteAt: null,
  selectedTag: null,
})

const emit = defineEmits<{
  (e: 'edit', name: string): void
  (e: 'locate', name: string): void
  (e: 'tag-select', tag: string): void
}>()

const { t } = useI18n()

const profilesRef = computed(() => props.profiles)

const {
  providerBreakdown,
  authModeBreakdown,
  topTags,
  deprecatedAuthIssues,
  missingFieldIssues,
  duplicateRuntimeIssues,
  totalIssueCount,
} = props.descriptor.useInsights(profilesRef)

const previewHeadingId = 'cp-inspector-preview-heading'
const auditHeadingId = 'cp-inspector-audit-heading'

const previewFields = computed<ProfilesInspectorField[]>(() =>
  props.previewProfile ? props.descriptor.activeFields(props.previewProfile) : [],
)

const previewTags = computed(() => props.previewProfile?.tags ?? [])

const isPreviewingCurrent = computed(() =>
  Boolean(
    props.previewProfile
      && props.currentProfile
      && props.previewProfile.name === props.currentProfile.name,
  ),
)

// 预览目标 ≠ 当前时给出「当前 → 目标」三行 diff，与 Apply 确认框共用同一数据
const diffRows = computed(() => {
  const preview = props.previewProfile
  const current = props.currentProfile
  if (!preview || !current || preview.name === current.name) return []
  return buildProfileDiff(current, preview, props.descriptor.diffFields)
})

// 分布条只展示实际出现过的 auth 模式
const visibleAuthModeBreakdown = computed(() =>
  authModeBreakdown.value.filter(item => item.count > 0),
)
</script>

<style scoped>
/* ===========================================================
   容器：仅 ≥1280px 显示，主视图 grid 已经把它放在第二列
   =========================================================== */
.cp-inspector {
  display: none;
  flex-direction: column;
  gap: 14px;
  position: sticky;
  top: 16px;
  align-self: start;
  max-height: calc(100vh - 32px);
  overflow-y: auto;
  scrollbar-color: var(--cp-line-2) transparent;
}

@media (width >= 1280px) {
  .cp-inspector { display: flex; }
}

/* ===========================================================
   通用卡片
   =========================================================== */
.cp-inspector-card {
  /* 背景/边框由 surface-card 工具类提供 */
  border-radius: 12px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.cp-inspector-card__head {
  display: flex;
  align-items: center;
  gap: 8px;
}

.cp-inspector-card__icon {
  color: var(--cp-accent);
  flex-shrink: 0;
}

.cp-inspector-card__title {
  margin: 0;
  font-size: 0.8125rem;
  font-weight: 600;
  letter-spacing: 0.0375rem;
  text-transform: uppercase;
  color: var(--cp-ink-2);
  flex: 1;
  min-width: 0;
}

.cp-inspector-card__count {
  flex-shrink: 0;
  padding: 0.0625rem 0.4375rem;
  border-radius: 999px;
  background: var(--cp-bg-3);
  color: var(--cp-ink-3);
  border: 1px solid var(--cp-line-2);
  font-family: var(--cp-mono);
  font-size: 0.75rem;
}

.cp-inspector-card__count--warn {
  background: rgb(var(--color-warning-rgb) / 14%);
  color: var(--cp-warn);
  border-color: rgb(var(--color-warning-rgb) / 35%);
}

.cp-inspector-badge {
  flex-shrink: 0;
  padding: 0.0625rem 0.4375rem;
  border-radius: 999px;
  background: var(--cp-accent-soft);
  color: var(--cp-accent);
  border: 1px solid var(--cp-accent-line);
  font-size: 0.75rem;
  font-weight: 600;
}

/* ===========================================================
   面板 1：预览
   =========================================================== */
.cp-inspector-preview {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.cp-inspector-preview__name {
  font-family: var(--cp-mono);
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--cp-ink-0);
  letter-spacing: -0.0187rem;
  word-break: break-all;
}

.cp-inspector-preview__desc {
  margin: 0;
  font-size: 0.8125rem;
  color: var(--cp-ink-2);
  line-height: 1.5;
}

.cp-inspector-fields {
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.cp-inspector-field {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.cp-inspector-field__label {
  font-family: var(--cp-mono);
  font-size: 0.75rem;
  letter-spacing: 0.0625rem;
  text-transform: uppercase;
  color: var(--cp-ink-3);
}

.cp-inspector-field__value {
  margin: 0;
  padding: 0.3125rem 0.5rem;
  background: var(--cp-bg-0);
  border: 1px solid var(--cp-line);
  border-radius: 6px;
  color: var(--cp-ink-0);
  font-family: var(--cp-mono);
  font-size: 0.75rem;
  word-break: break-all;
}

.cp-inspector-field__value--accent { color: var(--cp-accent); }
.cp-inspector-field__value--muted { color: var(--cp-ink-2); }

.cp-inspector-diff {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.cp-inspector-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.cp-inspector-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 0.125rem 0.4375rem;
  border-radius: 6px;
  background: var(--cp-bg-3);
  color: var(--cp-ink-1);
  border: 1px solid var(--cp-line-2);
  font-family: var(--cp-mono);
  font-size: 0.75rem;
  white-space: nowrap;
}

.cp-inspector-tag--count {
  background: var(--cp-bg-0);
}

.cp-inspector-tag--clickable {
  cursor: pointer;
  transition: background 120ms ease, border-color 120ms ease, color 120ms ease;
}

.cp-inspector-tag--clickable:hover {
  background: var(--cp-bg-3);
  color: var(--cp-ink-0);
}

.cp-inspector-tag--clickable[aria-pressed='true'] {
  background: var(--cp-accent-soft);
  border-color: var(--cp-accent-line);
  color: var(--cp-accent);
}

.cp-inspector-tag__count {
  color: var(--cp-ink-3);
}

.cp-inspector-session {
  font-family: var(--cp-mono);
  font-size: 0.75rem;
  color: var(--cp-ink-3);
}

.cp-inspector-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 0.4375rem 0.75rem;
  border-radius: 8px;
  border: 1px solid var(--cp-accent);
  background: var(--cp-accent-soft);
  color: var(--cp-accent);
  font-size: 0.8125rem;
  font-weight: 600;
  cursor: pointer;
  transition: background 120ms ease, color 120ms ease;
}

.cp-inspector-action:hover {
  background: var(--cp-accent);
  color: var(--cp-on-accent);
}

.cp-inspector-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 6px;
  padding: 20px 4px;
  color: var(--cp-ink-3);
}

.cp-inspector-empty__title {
  font-size: 0.8125rem;
  color: var(--cp-ink-1);
}

.cp-inspector-empty__hint {
  font-size: 0.75rem;
  color: var(--cp-ink-3);
}

/* ===========================================================
   面板 2：Health Audit
   =========================================================== */
.cp-inspector-clean {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 8px;
  background: rgb(var(--color-success-rgb) / 10%);
  border: 1px solid rgb(var(--color-success-rgb) / 25%);
  color: var(--cp-good);
  font-size: 0.8125rem;
  font-weight: 500;
}

.cp-inspector-issues {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.cp-inspector-issue {
  width: 100%;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 8px;
  background: var(--cp-bg-2);
  border: 1px solid var(--cp-line);
  color: var(--cp-ink-1);
  text-align: left;
  cursor: pointer;
  transition: background 120ms ease, border-color 120ms ease;
  font: inherit;
}

.cp-inspector-issue:hover {
  background: var(--cp-bg-3);
  border-color: var(--cp-line-2);
}

.cp-inspector-issue--warn { border-color: rgb(var(--color-warning-rgb) / 30%); }
.cp-inspector-issue--danger { border-color: rgb(var(--color-danger-rgb) / 30%); }
.cp-inspector-issue--info { border-color: rgb(var(--color-info-rgb) / 25%); }
.cp-inspector-issue--nested { margin-left: 8px; }

.cp-inspector-issue__icon { flex-shrink: 0; }

.cp-inspector-issue--warn .cp-inspector-issue__icon { color: var(--cp-warn); }
.cp-inspector-issue--danger .cp-inspector-issue__icon { color: var(--cp-danger); }
.cp-inspector-issue--info .cp-inspector-issue__icon { color: var(--cp-info); }

.cp-inspector-issue__body {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.cp-inspector-issue__name {
  font-family: var(--cp-mono);
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--cp-ink-0);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cp-inspector-issue__msg {
  font-size: 0.75rem;
  color: var(--cp-ink-3);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cp-inspector-issue__msg--mono {
  font-family: var(--cp-mono);
}

.cp-inspector-issue__locate {
  flex-shrink: 0;
  color: var(--cp-ink-3);
  opacity: 0.7;
}

.cp-inspector-issue:hover .cp-inspector-issue__locate {
  color: var(--cp-ink-0);
  opacity: 1;
}

.cp-inspector-issue-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 6px 0 4px;
  border-top: 1px dashed var(--cp-line);
}

.cp-inspector-issue-group:first-child {
  border-top: none;
  padding-top: 0;
}

.cp-inspector-issue-group__head {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.75rem;
  font-family: var(--cp-mono);
  letter-spacing: 0.025rem;
  text-transform: uppercase;
  color: var(--cp-info);
}

/* ===========================================================
   面板 3：Distribution（details 折叠）
   =========================================================== */
.cp-inspector-details { padding: 0; }

.cp-inspector-details__summary {
  padding: 14px;
  cursor: pointer;
  list-style: none;
  user-select: none;
}

.cp-inspector-details__summary::-webkit-details-marker { display: none; }

.cp-inspector-details__chevron {
  color: var(--cp-ink-3);
  transition: transform 120ms ease;
}

.cp-inspector-details[open] .cp-inspector-details__chevron {
  transform: rotate(180deg);
}

.cp-inspector-details__body {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 0 14px 14px;
}

.cp-inspector-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.cp-inspector-section__head {
  font-family: var(--cp-mono);
  font-size: 0.75rem;
  letter-spacing: 0.0625rem;
  text-transform: uppercase;
  color: var(--cp-ink-3);
}

.cp-inspector-section__empty {
  font-size: 0.75rem;
  color: var(--cp-ink-4);
  font-family: var(--cp-mono);
}

.cp-inspector-bars {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.cp-inspector-bar {
  display: grid;
  grid-template-columns: minmax(60px, 1fr) minmax(60px, 1.6fr) auto;
  align-items: center;
  gap: 8px;
  font-size: 0.75rem;
}

.cp-inspector-bar__label {
  color: var(--cp-ink-1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--cp-mono);
}

.cp-inspector-bar__track {
  height: 6px;
  background: var(--cp-bg-3);
  border-radius: 999px;
  overflow: hidden;
}

.cp-inspector-bar__fill {
  height: 100%;
  background: var(--cp-accent);
  border-radius: inherit;
}

.cp-inspector-bar__fill--warn { background: var(--cp-warn); }

.cp-inspector-bar__value {
  font-family: var(--cp-mono);
  color: var(--cp-ink-2);
  font-size: 0.75rem;
  min-width: 20px;
  text-align: right;
}

.cp-inspector-tagcloud {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

@media (prefers-reduced-motion: reduce) {
  .cp-inspector-action,
  .cp-inspector-bar__fill,
  .cp-inspector-details__chevron,
  .cp-inspector-issue,
  .cp-inspector-tag--clickable { transition: none; }
}
</style>
