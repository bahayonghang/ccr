<!--
  Profiles 右侧上下文侧栏：
  1) 当前 Profile 详情
  2) 分布洞察 (Provider / Auth 模式 / Tag)
  3) 健康审计 (已弃用 auth〔Codex〕/ 缺失字段 / 重复运行时)
  仅 ≥1280px 视口可见；窄屏自动隐藏，不影响主列。
  平台差异通过 i18nPrefix + descriptor（洞察来源 / 字段列表 / 文案 / 图标）注入；样式靠视图 --cp-* 继承换肤。
-->
<template>
  <aside
    class="cp-rail"
    :aria-label="t(`${i18nPrefix}.ariaLabel`)"
  >
    <!-- ========================================================
         面板 1：当前 Profile 详情
         ======================================================== -->
    <section
      class="cp-rail-card surface-card"
      :aria-labelledby="activeHeadingId"
    >
      <header class="cp-rail-card__head">
        <SIcon
          name="Sparkles"
          size="w-3.5 h-3.5"
          class="cp-rail-card__icon"
        />
        <h3
          :id="activeHeadingId"
          class="cp-rail-card__title"
        >
          {{ t(`${i18nPrefix}.activeTitle`) }}
        </h3>
      </header>

      <div
        v-if="activeProfile"
        class="cp-rail-active"
      >
        <div class="cp-rail-active__name">
          {{ activeProfile.name }}
        </div>
        <p
          v-if="activeProfile.description"
          class="cp-rail-active__desc"
        >
          {{ activeProfile.description }}
        </p>

        <dl class="cp-rail-fields">
          <div
            v-for="field in activeFields"
            :key="field.label"
            class="cp-rail-field"
          >
            <dt class="cp-rail-field__label">
              {{ field.label }}
            </dt>
            <dd
              class="cp-rail-field__value"
              :class="{
                'cp-rail-field__value--accent': field.variant === 'accent',
                'cp-rail-field__value--muted': field.variant === 'muted',
              }"
            >
              {{ field.value }}
            </dd>
          </div>
        </dl>

        <div
          v-if="activeTags.length > 0"
          class="cp-rail-tags"
        >
          <span
            v-for="tag in activeTags"
            :key="tag"
            class="cp-rail-tag"
          >#{{ tag }}</span>
        </div>

        <button
          type="button"
          class="cp-rail-action"
          @click="onEdit(activeProfile.name)"
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
        class="cp-rail-empty"
      >
        <SIcon
          name="Folder"
          size="w-4 h-4"
        />
        <div class="cp-rail-empty__title">
          {{ t(`${i18nPrefix}.activeEmpty`) }}
        </div>
        <div class="cp-rail-empty__hint">
          {{ t(`${i18nPrefix}.activeEmptyHint`) }}
        </div>
      </div>
    </section>

    <!-- ========================================================
         面板 2：分布洞察
         ======================================================== -->
    <section
      class="cp-rail-card surface-card"
      :aria-labelledby="distributionHeadingId"
    >
      <header class="cp-rail-card__head">
        <SIcon
          name="BarChart3"
          size="w-3.5 h-3.5"
          class="cp-rail-card__icon"
        />
        <h3
          :id="distributionHeadingId"
          class="cp-rail-card__title"
        >
          {{ t(`${i18nPrefix}.distributionTitle`) }}
        </h3>
      </header>

      <!-- 2.1 Provider -->
      <div class="cp-rail-section">
        <div class="cp-rail-section__head">
          {{ t(`${i18nPrefix}.providerSection`) }}
        </div>
        <ul
          v-if="providerBreakdown.length > 0"
          class="cp-rail-bars"
          role="presentation"
        >
          <li
            v-for="item in providerBreakdown"
            :key="item.provider"
            class="cp-rail-bar"
          >
            <div class="cp-rail-bar__label">
              {{
                item.provider === 'Unknown'
                  ? t(`${i18nPrefix}.unknownProvider`)
                  : item.provider
              }}
            </div>
            <div class="cp-rail-bar__track">
              <div
                class="cp-rail-bar__fill"
                :style="{ width: `${Math.max(item.pct, 4)}%` }"
              />
            </div>
            <div class="cp-rail-bar__value">
              {{ item.count }}
            </div>
          </li>
        </ul>
        <div
          v-else
          class="cp-rail-section__empty"
        >
          —
        </div>
      </div>

      <!-- 2.2 Auth 模式 -->
      <div class="cp-rail-section">
        <div class="cp-rail-section__head">
          {{ t(`${i18nPrefix}.authSection`) }}
        </div>
        <ul
          class="cp-rail-bars"
          role="presentation"
        >
          <li
            v-for="item in authModeBreakdown"
            :key="item.mode"
            class="cp-rail-bar"
            :class="{ 'cp-rail-bar--zero': item.count === 0 }"
          >
            <div class="cp-rail-bar__label">
              {{ descriptor.authModeLabel(item.mode) }}
            </div>
            <div class="cp-rail-bar__track">
              <div
                class="cp-rail-bar__fill"
                :class="{ 'cp-rail-bar__fill--warn': descriptor.isDeprecatedMode(item.mode) }"
                :style="{ width: `${item.count === 0 ? 0 : Math.max(item.pct, 4)}%` }"
              />
            </div>
            <div class="cp-rail-bar__value">
              {{ item.count }}
            </div>
          </li>
        </ul>
      </div>

      <!-- 2.3 Top Tags -->
      <div class="cp-rail-section">
        <div class="cp-rail-section__head">
          {{ t(`${i18nPrefix}.tagsSection`) }}
        </div>
        <div
          v-if="topTags.length > 0"
          class="cp-rail-tagcloud"
        >
          <span
            v-for="item in topTags"
            :key="item.tag"
            class="cp-rail-tag cp-rail-tag--count"
          >
            <span>#{{ item.tag }}</span>
            <span class="cp-rail-tag__count">{{ item.count }}</span>
          </span>
        </div>
        <div
          v-else
          class="cp-rail-section__empty"
        >
          {{ t(`${i18nPrefix}.noTags`) }}
        </div>
      </div>
    </section>

    <!-- ========================================================
         面板 3：健康审计
         ======================================================== -->
    <section
      class="cp-rail-card surface-card"
      :aria-labelledby="auditHeadingId"
    >
      <header class="cp-rail-card__head">
        <SIcon
          name="ShieldCheck"
          size="w-3.5 h-3.5"
          class="cp-rail-card__icon"
        />
        <h3
          :id="auditHeadingId"
          class="cp-rail-card__title"
        >
          {{ t(`${i18nPrefix}.auditTitle`) }}
        </h3>
        <span
          class="cp-rail-card__count"
          :class="{ 'cp-rail-card__count--warn': totalIssueCount > 0 }"
        >
          {{ totalIssueCount }}
        </span>
      </header>

      <div
        v-if="totalIssueCount === 0"
        class="cp-rail-clean"
      >
        <SIcon
          name="CheckCircle"
          size="w-4 h-4"
        />
        <span>{{ t(`${i18nPrefix}.auditClean`) }}</span>
      </div>

      <ul
        v-else
        class="cp-rail-issues"
      >
        <!-- 已弃用 auth（Codex 专属；Claude deprecatedAuthIssues 恒空 → 不渲染） -->
        <li
          v-for="profile in deprecatedAuthIssues"
          :key="`dep-${profile.name}`"
        >
          <button
            type="button"
            class="cp-rail-issue cp-rail-issue--warn"
            @click="onEdit(profile.name)"
          >
            <SIcon
              name="AlertCircle"
              size="w-3.5 h-3.5"
              class="cp-rail-issue__icon"
            />
            <div class="cp-rail-issue__body">
              <div class="cp-rail-issue__name">
                {{ profile.name }}
              </div>
              <div class="cp-rail-issue__msg">
                {{ descriptor.deprecatedMessage?.(profile) }}
              </div>
            </div>
            <SIcon
              :name="descriptor.editIcon"
              size="w-3.5 h-3.5"
              class="cp-rail-issue__edit"
            />
          </button>
        </li>

        <li
          v-for="issue in missingFieldIssues"
          :key="`miss-${issue.profile.name}`"
        >
          <button
            type="button"
            class="cp-rail-issue cp-rail-issue--danger"
            @click="onEdit(issue.profile.name)"
          >
            <SIcon
              name="AlertTriangle"
              size="w-3.5 h-3.5"
              class="cp-rail-issue__icon"
            />
            <div class="cp-rail-issue__body">
              <div class="cp-rail-issue__name">
                {{ issue.profile.name }}
              </div>
              <div class="cp-rail-issue__msg">
                {{ descriptor.missingMessage(issue.missing) }}
              </div>
            </div>
            <SIcon
              :name="descriptor.editIcon"
              size="w-3.5 h-3.5"
              class="cp-rail-issue__edit"
            />
          </button>
        </li>

        <li
          v-for="group in duplicateRuntimeIssues"
          :key="`dup-${group.key}`"
          class="cp-rail-issue-group"
        >
          <div class="cp-rail-issue-group__head">
            <SIcon
              name="Copy"
              size="w-3.5 h-3.5"
              class="cp-rail-issue__icon"
            />
            <span>
              {{
                tf(
                  `${i18nPrefix}.issues.duplicateRuntime`,
                  '运行时重复 · {count} 个 Profile',
                  { count: group.profiles.length },
                )
              }}
            </span>
          </div>
          <button
            v-for="profile in group.profiles"
            :key="`dup-${group.key}-${profile.name}`"
            type="button"
            class="cp-rail-issue cp-rail-issue--info cp-rail-issue--nested"
            @click="onEdit(profile.name)"
          >
            <div class="cp-rail-issue__body">
              <div class="cp-rail-issue__name">
                {{ profile.name }}
              </div>
              <div class="cp-rail-issue__msg cp-rail-issue__msg--mono">
                {{ descriptor.runtimeSummary(profile) }}
              </div>
            </div>
            <SIcon
              :name="descriptor.editIcon"
              size="w-3.5 h-3.5"
              class="cp-rail-issue__edit"
            />
          </button>
        </li>
      </ul>
    </section>
  </aside>
</template>

<script setup lang="ts" generic="T extends ContextRailProfile">
import { computed, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import { useTf } from '@/composables/useTf'
import type { ProfilesInsights } from '@/composables/useProfilesInsights'

/** ContextRail 直接读取的最小 profile 形状（两平台共有字段） */
export interface ContextRailProfile {
  name: string
  description?: string | null
  tags?: string[] | null
}

/** 当前 profile 详情面板的单个字段（平台决定字段集合/顺序/样式） */
export interface ContextRailActiveField {
  label: string
  value: string
  variant?: 'accent' | 'muted'
}

/** 平台注入的侧栏策略：洞察来源 + 字段列表 + 文案 + 图标 */
export interface ContextRailDescriptor<P extends ContextRailProfile> {
  /** 编辑按钮图标：Claude 'Pencil' / Codex 'Edit2' */
  editIcon: string
  /** 平台洞察 composable（在组件 setup 内调用一次） */
  useInsights: (profiles: Ref<P[]>) => ProfilesInsights<P, string, string>
  /** 当前 profile 的字段列表（activeProfile 非空时调用） */
  activeFields: (profile: P) => ContextRailActiveField[]
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
  current: string | null
  activeProfile: T | null
  /** i18n key 前缀，例如 'claudeProfiles.contextRail' / 'codex.profiles.contextRail' */
  i18nPrefix: string
  descriptor: ContextRailDescriptor<T>
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'edit', name: string): void
}>()

const { t } = useI18n()

const tf = useTf()

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

const activeHeadingId = 'cp-rail-active-heading'
const distributionHeadingId = 'cp-rail-distribution-heading'
const auditHeadingId = 'cp-rail-audit-heading'

const activeFields = computed<ContextRailActiveField[]>(() =>
  props.activeProfile ? props.descriptor.activeFields(props.activeProfile) : [],
)

const activeTags = computed(() => props.activeProfile?.tags ?? [])

const onEdit = (name: string) => emit('edit', name)
</script>

<style scoped>
/* ===========================================================
   容器：仅 ≥1280px 显示，主视图 grid 已经把它放在第二列
   =========================================================== */
.cp-rail {
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
  .cp-rail { display: flex; }
}

/* ===========================================================
   通用卡片
   =========================================================== */
.cp-rail-card {
  /* 背景/边框由 surface-card 工具类提供 */
  border-radius: 12px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.cp-rail-card__head {
  display: flex;
  align-items: center;
  gap: 8px;
}

.cp-rail-card__icon {
  color: var(--cp-accent);
  flex-shrink: 0;
}

.cp-rail-card__title {
  margin: 0;
  font-size: 11.5px;
  font-weight: 600;
  letter-spacing: 0.6px;
  text-transform: uppercase;
  color: var(--cp-ink-2);
  flex: 1;
  min-width: 0;
}

.cp-rail-card__count {
  flex-shrink: 0;
  padding: 1px 7px;
  border-radius: 999px;
  background: var(--cp-bg-3);
  color: var(--cp-ink-3);
  border: 1px solid var(--cp-line-2);
  font-family: var(--cp-mono);
  font-size: 10.5px;
}

.cp-rail-card__count--warn {
  background: rgb(var(--color-warning-rgb) / 14%);
  color: var(--cp-warn);
  border-color: rgb(var(--color-warning-rgb) / 35%);
}

/* ===========================================================
   面板 1：当前 Profile 详情
   =========================================================== */
.cp-rail-active {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.cp-rail-active__name {
  font-family: var(--cp-mono);
  font-size: 15px;
  font-weight: 600;
  color: var(--cp-ink-0);
  letter-spacing: -0.3px;
  word-break: break-all;
}

.cp-rail-active__desc {
  margin: 0;
  font-size: 12px;
  color: var(--cp-ink-2);
  line-height: 1.5;
}

.cp-rail-fields {
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.cp-rail-field {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.cp-rail-field__label {
  font-family: var(--cp-mono);
  font-size: 9.5px;
  letter-spacing: 1px;
  text-transform: uppercase;
  color: var(--cp-ink-3);
}

.cp-rail-field__value {
  margin: 0;
  padding: 5px 8px;
  background: var(--cp-bg-0);
  border: 1px solid var(--cp-line);
  border-radius: 5px;
  color: var(--cp-ink-0);
  font-family: var(--cp-mono);
  font-size: 11.5px;
  word-break: break-all;
}

.cp-rail-field__value--accent { color: var(--cp-accent); }
.cp-rail-field__value--muted { color: var(--cp-ink-2); }

.cp-rail-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.cp-rail-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 7px;
  border-radius: 4px;
  background: var(--cp-bg-3);
  color: var(--cp-ink-1);
  border: 1px solid var(--cp-line-2);
  font-family: var(--cp-mono);
  font-size: 10.5px;
  white-space: nowrap;
}

.cp-rail-tag--count {
  background: var(--cp-bg-0);
}

.cp-rail-tag__count {
  color: var(--cp-ink-3);
  font-size: 10px;
}

.cp-rail-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 7px 12px;
  border-radius: 7px;
  border: 1px solid var(--cp-accent);
  background: var(--cp-accent-soft);
  color: var(--cp-accent);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: background 120ms ease, color 120ms ease;
}

.cp-rail-action:hover {
  background: var(--cp-accent);
  color: var(--cp-on-accent);
}

.cp-rail-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 6px;
  padding: 20px 4px;
  color: var(--cp-ink-3);
}

.cp-rail-empty__title {
  font-size: 12.5px;
  color: var(--cp-ink-1);
}

.cp-rail-empty__hint {
  font-size: 11px;
  color: var(--cp-ink-3);
}

/* ===========================================================
   面板 2：分布洞察
   =========================================================== */
.cp-rail-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.cp-rail-section__head {
  font-family: var(--cp-mono);
  font-size: 9.5px;
  letter-spacing: 1px;
  text-transform: uppercase;
  color: var(--cp-ink-3);
}

.cp-rail-section__empty {
  font-size: 11.5px;
  color: var(--cp-ink-4);
  font-family: var(--cp-mono);
}

.cp-rail-bars {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.cp-rail-bar {
  display: grid;
  grid-template-columns: minmax(60px, 1fr) minmax(60px, 1.6fr) auto;
  align-items: center;
  gap: 8px;
  font-size: 11px;
}

.cp-rail-bar--zero { opacity: 0.55; }

.cp-rail-bar__label {
  color: var(--cp-ink-1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--cp-mono);
}

.cp-rail-bar__track {
  height: 6px;
  background: var(--cp-bg-3);
  border-radius: 999px;
  overflow: hidden;
}

.cp-rail-bar__fill {
  height: 100%;
  background: var(--cp-accent);
  border-radius: inherit;
  transition: width 200ms ease;
}

.cp-rail-bar__fill--warn { background: var(--cp-warn); }

.cp-rail-bar__value {
  font-family: var(--cp-mono);
  color: var(--cp-ink-2);
  font-size: 11px;
  min-width: 20px;
  text-align: right;
}

.cp-rail-tagcloud {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

/* ===========================================================
   面板 3：健康审计
   =========================================================== */
.cp-rail-clean {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 8px;
  background: rgb(var(--color-success-rgb) / 10%);
  border: 1px solid rgb(var(--color-success-rgb) / 25%);
  color: var(--cp-good);
  font-size: 12px;
  font-weight: 500;
}

.cp-rail-issues {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.cp-rail-issue {
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

.cp-rail-issue:hover {
  background: var(--cp-bg-3);
  border-color: var(--cp-line-2);
}

.cp-rail-issue--warn { border-color: rgb(var(--color-warning-rgb) / 30%); }
.cp-rail-issue--danger { border-color: rgb(var(--color-danger-rgb) / 30%); }
.cp-rail-issue--info { border-color: rgb(var(--color-info-rgb) / 25%); }
.cp-rail-issue--nested { margin-left: 8px; }

.cp-rail-issue__icon {
  flex-shrink: 0;
}

.cp-rail-issue--warn .cp-rail-issue__icon { color: var(--cp-warn); }
.cp-rail-issue--danger .cp-rail-issue__icon { color: var(--cp-danger); }
.cp-rail-issue--info .cp-rail-issue__icon { color: var(--cp-info); }

.cp-rail-issue__body {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.cp-rail-issue__name {
  font-family: var(--cp-mono);
  font-size: 11.5px;
  font-weight: 600;
  color: var(--cp-ink-0);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cp-rail-issue__msg {
  font-size: 11px;
  color: var(--cp-ink-3);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cp-rail-issue__msg--mono {
  font-family: var(--cp-mono);
}

.cp-rail-issue__edit {
  flex-shrink: 0;
  color: var(--cp-ink-3);
  opacity: 0.7;
}

.cp-rail-issue:hover .cp-rail-issue__edit {
  color: var(--cp-ink-0);
  opacity: 1;
}

.cp-rail-issue-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 6px 0 4px;
  border-top: 1px dashed var(--cp-line);
}

.cp-rail-issue-group:first-child {
  border-top: none;
  padding-top: 0;
}

.cp-rail-issue-group__head {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 10.5px;
  font-family: var(--cp-mono);
  letter-spacing: 0.4px;
  text-transform: uppercase;
  color: var(--cp-info);
}

@media (prefers-reduced-motion: reduce) {
  .cp-rail-action,
  .cp-rail-bar__fill,
  .cp-rail-issue { transition: none; }
}
</style>
