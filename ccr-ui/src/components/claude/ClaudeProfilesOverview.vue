<template>
  <div class="cpd">
    <!-- ── Band 1: Identity ─────────────────────────────────────────── -->
    <section
      class="cpd-identity"
      :aria-label="identityAriaLabel"
    >
      <div class="cpd-identity__head">
        <span class="cpd-identity__eyebrow">{{ $t('claudeProfiles.currentProfile') }}</span>

        <div class="cpd-identity__title-row">
          <span
            v-if="currentProfileName"
            aria-hidden="true"
            class="cpd-identity__state-dot"
            :class="stateDotClass"
          />
          <h2
            class="cpd-identity__name"
            :title="identityNameTitle"
          >
            {{ identityNameTitle }}
          </h2>
          <span
            v-if="currentProfileName"
            class="cpd-identity__status"
            :class="statusPillClass"
          >
            <span
              aria-hidden="true"
              class="cpd-identity__status-dot"
            />
            {{ currentProfileStatusLabel }}
          </span>

          <template v-if="identityInlineChips.length > 0">
            <span
              aria-hidden="true"
              class="cpd-identity__divider"
            >·</span>
            <span
              v-for="chip in identityInlineChips"
              :key="chip.key"
              class="cpd-identity__chip"
              :class="chip.toneClass"
            >
              {{ chip.text }}
            </span>
          </template>
        </div>
      </div>

      <p class="cpd-identity__desc">
        {{ currentProfileDescription }}
      </p>

      <dl
        v-if="identityMeta.length > 0"
        class="cpd-identity__meta"
      >
        <div
          v-for="item in identityMeta"
          :key="item.key"
          class="cpd-identity__meta-item"
        >
          <dt>{{ item.label }}</dt>
          <dd>
            <span
              v-for="(fragment, idx) in item.fragments"
              :key="idx"
              :class="fragment.className"
            >{{ fragment.text }}</span>
          </dd>
        </div>
      </dl>
    </section>

    <!-- ── Band 2: Metric Ticker ────────────────────────────────────── -->
    <section
      class="cpd-metrics"
      role="list"
      :aria-label="metricsAriaLabel"
    >
      <article
        v-for="(metric, index) in metricItems"
        :key="metric.key"
        role="listitem"
        class="cpd-metric"
        :class="[`cpd-metric--${metric.tier}`]"
        :style="{ '--i': index }"
      >
        <span class="cpd-metric__label">{{ metric.label }}</span>
        <span
          class="cpd-metric__value"
          :class="metric.valueToneClass"
        >{{ metric.value }}</span>
        <span class="cpd-metric__detail">{{ metric.detail }}</span>
      </article>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { ClaudeProfile } from '@/types'
import {
  isCustomClaudeProfileBaseUrl,
  isOfficialClaudeProfileBaseUrl,
  type ClaudeProfilesOverviewSummary,
} from '@/utils/claudeProfiles'
import { translateWithFallback } from '@/i18n/formatMessage'

interface IdentityChip {
  key: string
  text: string
  toneClass: string
}

interface IdentityMetaFragment {
  text: string
  className: string
}

interface IdentityMetaItem {
  key: 'endpoint' | 'model' | 'tags'
  label: string
  fragments: IdentityMetaFragment[]
}

interface MetricItem {
  key: string
  tier: 'primary' | 'secondary'
  label: string
  value: string
  detail: string
  valueToneClass: string
}

const props = defineProps<{
  currentProfile: ClaudeProfile | null
  providerUnsetLabel: string
  summary: ClaudeProfilesOverviewSummary
}>()

const { t } = useI18n()

const EM_DASH = '—'

const currentProfileName = computed(() => props.currentProfile?.name ?? null)

const identityNameTitle = computed(() => currentProfileName.value || t('claudeProfiles.notSet'))

const currentProfileStatusLabel = computed(() => (
  props.currentProfile?.enabled === false
    ? t('claudeProfiles.currentDisabled')
    : t('claudeProfiles.currentlyActive')
))

const stateDotClass = computed(() => {
  if (!props.currentProfile) return 'cpd-identity__state-dot--idle'
  return props.currentProfile.enabled === false
    ? 'cpd-identity__state-dot--disabled'
    : 'cpd-identity__state-dot--active'
})

const statusPillClass = computed(() => (
  props.currentProfile?.enabled === false
    ? 'cpd-identity__status--disabled'
    : 'cpd-identity__status--active'
))

const formatAuthMode = (authMode?: string | null): string | null => {
  if (authMode === 'api_key') return t('claudeProfiles.authModeApiKey')
  if (authMode === 'subscription') return t('claudeProfiles.authModeSubscription')
  return null
}

const currentProfileDescription = computed(() => {
  const description = props.currentProfile?.description?.trim()
  if (description) return description

  return props.currentProfile
    ? t('claudeProfiles.currentProfileSummaryFallback')
    : t('claudeProfiles.currentProfileMissingHint')
})

const identityInlineChips = computed<IdentityChip[]>(() => {
  const profile = props.currentProfile
  if (!profile) return []

  const chips: IdentityChip[] = []

  const authMode = formatAuthMode(profile.auth_mode)
  if (authMode) {
    chips.push({
      key: 'auth-mode',
      text: authMode,
      toneClass: 'cpd-identity__chip--accent',
    })
  }

  const provider = profile.provider?.trim()
  chips.push({
    key: 'provider',
    text: provider || props.providerUnsetLabel,
    toneClass: provider ? 'cpd-identity__chip--neutral' : 'cpd-identity__chip--muted',
  })

  const providerType = profile.provider_type?.trim()
  if (providerType) {
    chips.push({
      key: 'provider-type',
      text: providerType,
      toneClass: 'cpd-identity__chip--neutral',
    })
  }

  const account = profile.account?.trim()
  if (account) {
    chips.push({
      key: 'account',
      text: `@${account}`,
      toneClass: 'cpd-identity__chip--neutral',
    })
  }

  return chips
})

const identityMeta = computed<IdentityMetaItem[]>(() => {
  const profile = props.currentProfile
  if (!profile) return []

  const items: IdentityMetaItem[] = []

  // ENDPOINT
  const baseUrl = profile.base_url?.trim()
  const endpointLabel = translateWithFallback(t, 'claudeProfiles.identityFieldEndpoint', 'ENDPOINT')
  if (baseUrl) {
    const custom = isCustomClaudeProfileBaseUrl(baseUrl)
    const official = isOfficialClaudeProfileBaseUrl(baseUrl)
    const badgeText = official
      ? translateWithFallback(t, 'claudeProfiles.identityEndpointOfficial', '官方')
      : custom
        ? translateWithFallback(t, 'claudeProfiles.identityEndpointCustom', '自定义')
        : null

    const fragments: IdentityMetaFragment[] = [
      { text: baseUrl, className: 'cpd-identity__frag cpd-identity__frag--mono' },
    ]

    if (badgeText) {
      fragments.push({
        text: badgeText,
        className: official
          ? 'cpd-identity__frag cpd-identity__frag--badge cpd-identity__frag--badge-info'
          : 'cpd-identity__frag cpd-identity__frag--badge cpd-identity__frag--badge-ok',
      })
    }

    items.push({ key: 'endpoint', label: endpointLabel, fragments })
  } else {
    items.push({
      key: 'endpoint',
      label: endpointLabel,
      fragments: [{ text: EM_DASH, className: 'cpd-identity__frag cpd-identity__frag--muted' }],
    })
  }

  // MODEL
  const primary = profile.model?.trim()
  const fast = profile.small_fast_model?.trim()
  const modelLabel = translateWithFallback(t, 'claudeProfiles.identityFieldModel', 'MODEL')
  if (primary || fast) {
    const fragments: IdentityMetaFragment[] = []
    if (primary) {
      fragments.push({ text: primary, className: 'cpd-identity__frag cpd-identity__frag--mono' })
    }
    if (primary && fast) {
      fragments.push({ text: '→', className: 'cpd-identity__frag cpd-identity__frag--arrow' })
    }
    if (fast) {
      fragments.push({ text: fast, className: 'cpd-identity__frag cpd-identity__frag--mono' })
    }

    items.push({ key: 'model', label: modelLabel, fragments })
  } else {
    items.push({
      key: 'model',
      label: modelLabel,
      fragments: [{ text: EM_DASH, className: 'cpd-identity__frag cpd-identity__frag--muted' }],
    })
  }

  // TAGS
  const tags = (profile.tags ?? []).filter(tag => tag.trim().length > 0)
  const tagsLabel = translateWithFallback(t, 'claudeProfiles.identityFieldTags', 'TAGS')
  if (tags.length > 0) {
    const preview = tags.slice(0, 3).join(', ')
    const rest = tags.length > 3 ? ` +${tags.length - 3}` : ''
    items.push({
      key: 'tags',
      label: tagsLabel,
      fragments: [
        { text: `${tags.length}`, className: 'cpd-identity__frag cpd-identity__frag--count' },
        { text: `${preview}${rest}`, className: 'cpd-identity__frag cpd-identity__frag--muted' },
      ],
    })
  } else {
    items.push({
      key: 'tags',
      label: tagsLabel,
      fragments: [{ text: EM_DASH, className: 'cpd-identity__frag cpd-identity__frag--muted' }],
    })
  }

  return items
})

const metricItems = computed<MetricItem[]>(() => {
  const s = props.summary

  return [
    // Primary tier (原 overviewTiles)
    {
      key: 'profiles',
      tier: 'primary',
      label: t('claudeProfiles.overviewProfilesLabel'),
      value: String(s.totalProfiles),
      detail: translateWithFallback(
        t,
        'claudeProfiles.overviewProfilesDetail',
        '{enabled} 已启用 · {disabled} 已停用',
        {
          enabled: s.enabledProfilesCount,
          disabled: s.disabledProfilesCount,
        },
      ),
      valueToneClass: 'cpd-metric__value--default',
    },
    {
      key: 'providers',
      tier: 'primary',
      label: t('claudeProfiles.overviewProvidersLabel'),
      value: String(s.providerSectionsCount),
      detail: translateWithFallback(
        t,
        'claudeProfiles.overviewProvidersDetail',
        '{sections} 个分组 · {missing} 未设置 Provider',
        {
          sections: s.providerSectionsCount,
          missing: s.unsetProviderProfilesCount,
        },
      ),
      valueToneClass: 'cpd-metric__value--info',
    },
    {
      key: 'models',
      tier: 'primary',
      label: t('claudeProfiles.overviewModelsLabel'),
      value: String(s.configuredModelProfilesCount),
      detail: translateWithFallback(
        t,
        'claudeProfiles.overviewModelsDetail',
        '{primary} 主模型 · {fast} 快速模型',
        {
          primary: s.configuredModelProfilesCount,
          fast: s.configuredFastModelProfilesCount,
        },
      ),
      valueToneClass: 'cpd-metric__value--accent',
    },
    {
      key: 'access',
      tier: 'primary',
      label: t('claudeProfiles.overviewAccessLabel'),
      value: String(s.accountProfilesCount),
      detail: translateWithFallback(
        t,
        'claudeProfiles.overviewAccessDetail',
        '{subscription} 订阅 · {apiKey} API Key · {accounts} 账号',
        {
          subscription: s.subscriptionProfilesCount,
          apiKey: s.apiKeyProfilesCount,
          accounts: s.accountProfilesCount,
        },
      ),
      valueToneClass: 'cpd-metric__value--success',
    },
    // Secondary tier (原 ribbon；短 label 新增，中文 detail 复用既有 i18n)
    {
      key: 'custom-endpoint',
      tier: 'secondary',
      label: translateWithFallback(t, 'claudeProfiles.metricsSecondaryEndpointLabel', 'ENDPOINT'),
      value: String(s.customBaseUrlProfilesCount),
      detail: t('claudeProfiles.overviewRibbonCustomBaseUrl'),
      valueToneClass: 'cpd-metric__value--default',
    },
    {
      key: 'tagged',
      tier: 'secondary',
      label: translateWithFallback(t, 'claudeProfiles.metricsSecondaryTaggedLabel', 'TAGGED'),
      value: String(s.taggedProfilesCount),
      detail: t('claudeProfiles.overviewRibbonTagged'),
      valueToneClass: 'cpd-metric__value--default',
    },
    {
      key: 'missing-model',
      tier: 'secondary',
      label: translateWithFallback(t, 'claudeProfiles.metricsSecondaryNoModelLabel', 'NO MODEL'),
      value: String(s.missingModelProfilesCount),
      detail: t('claudeProfiles.overviewRibbonMissingModel'),
      valueToneClass: s.missingModelProfilesCount > 0
        ? 'cpd-metric__value--warning'
        : 'cpd-metric__value--default',
    },
    {
      key: 'missing-account',
      tier: 'secondary',
      label: translateWithFallback(t, 'claudeProfiles.metricsSecondaryNoAccountLabel', 'NO ACCT'),
      value: String(s.missingAccountProfilesCount),
      detail: t('claudeProfiles.overviewRibbonMissingAccount'),
      valueToneClass: s.missingAccountProfilesCount > 0
        ? 'cpd-metric__value--warning'
        : 'cpd-metric__value--default',
    },
  ]
})

const identityAriaLabel = computed(() => translateWithFallback(
  t,
  'claudeProfiles.identityAriaLabel',
  '当前 Profile 身份摘要',
))

const metricsAriaLabel = computed(() => translateWithFallback(
  t,
  'claudeProfiles.metricsAriaLabel',
  'Profile 指标总览',
))
</script>

<style scoped>
/* ── 外层节奏 ──────────────────────────────────────────────── */
.cpd {
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
}

/* ── Band 1: Identity ─────────────────────────────────────── */
.cpd-identity {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 0.65rem;
  padding: 0.95rem 1.15rem 1.05rem;
  border-radius: 22px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 38%);
  background:
    linear-gradient(180deg,
      rgb(var(--color-bg-elevated-rgb) / 72%),
      rgb(var(--color-bg-surface-rgb) / 60%));
  box-shadow:
    0 14px 30px rgb(15 12 18 / 5%),
    inset 0 1px 0 rgb(255 255 255 / 5%);
  backdrop-filter: blur(18px) saturate(130%);
}

.cpd-identity__head {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  min-width: 0;
}

.cpd-identity__eyebrow {
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.24em;
  text-transform: uppercase;
  color: var(--color-text-muted);
}

.cpd-identity__title-row {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 0.55rem 0.65rem;
  min-width: 0;
}

.cpd-identity__state-dot {
  flex-shrink: 0;
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 9999px;
  transform: translateY(-0.1rem);
}

.cpd-identity__state-dot--active {
  background: rgb(var(--color-accent-secondary-rgb));
  box-shadow: 0 0 0 4px rgb(var(--color-accent-secondary-rgb) / 18%);
}

.cpd-identity__state-dot--disabled {
  background: rgb(var(--color-danger-rgb));
  box-shadow: 0 0 0 4px rgb(var(--color-danger-rgb) / 18%);
}

.cpd-identity__state-dot--idle {
  background: rgb(var(--color-text-muted-rgb) / 70%);
}

.cpd-identity__name {
  margin: 0;
  max-width: 100%;
  overflow: hidden;
  font-size: 1.55rem;
  line-height: 1.15;
  font-weight: 620;
  letter-spacing: -0.024em;
  color: var(--color-text-primary);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cpd-identity__status {
  display: inline-flex;
  align-items: center;
  gap: 0.38rem;
  padding: 0.18rem 0.62rem;
  border-radius: 9999px;
  border: 1px solid transparent;
  font-size: 0.72rem;
  font-weight: 600;
}

.cpd-identity__status--active {
  border-color: rgb(var(--color-accent-secondary-rgb) / 26%);
  background: rgb(var(--color-accent-secondary-rgb) / 10%);
  color: rgb(var(--color-accent-secondary-rgb));
}

.cpd-identity__status--disabled {
  border-color: rgb(var(--color-danger-rgb) / 26%);
  background: rgb(var(--color-danger-rgb) / 10%);
  color: rgb(var(--color-danger-rgb));
}

.cpd-identity__status-dot {
  width: 0.38rem;
  height: 0.38rem;
  border-radius: 9999px;
  background: currentcolor;
  opacity: 0.85;
}

.cpd-identity__divider {
  color: var(--color-text-muted);
  opacity: 0.55;
  user-select: none;
}

.cpd-identity__chip {
  display: inline-flex;
  align-items: center;
  min-height: 1.55rem;
  padding: 0.14rem 0.6rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 45%);
  border-radius: 9999px;
  background: rgb(var(--color-bg-elevated-rgb) / 62%);
  color: var(--color-text-secondary);
  font-size: 0.72rem;
  font-weight: 500;
  line-height: 1rem;
}

.cpd-identity__chip--accent {
  border-color: rgb(var(--color-accent-primary-rgb) / 28%);
  background: rgb(var(--color-accent-primary-rgb) / 8%);
  color: rgb(var(--color-accent-primary-rgb));
}

.cpd-identity__chip--muted {
  color: var(--color-text-muted);
  opacity: 0.85;
}

.cpd-identity__desc {
  margin: 0;
  max-width: 64rem;
  font-size: 0.86rem;
  line-height: 1.55;
  color: var(--color-text-secondary);
}

/* ── Identity 元数据行 ─────────────────────────────────────── */
.cpd-identity__meta {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 0.7rem 1.25rem;
  margin: 0;
  padding-top: 0.6rem;
  border-top: 1px solid rgb(var(--color-border-default-rgb) / 18%);
}

.cpd-identity__meta-item {
  display: flex;
  align-items: baseline;
  gap: 0.55rem;
  min-width: 0;
}

.cpd-identity__meta-item dt {
  flex-shrink: 0;
  font-size: 0.64rem;
  font-weight: 700;
  letter-spacing: 0.22em;
  text-transform: uppercase;
  color: var(--color-text-muted);
}

.cpd-identity__meta-item dd {
  display: inline-flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 0.35rem;
  margin: 0;
  min-width: 0;
  font-size: 0.82rem;
  color: var(--color-text-primary);
}

.cpd-identity__frag--mono {
  max-width: 22rem;
  overflow: hidden;
  font-family: var(--font-mono, monospace);
  font-size: 0.78rem;
  letter-spacing: 0.01em;
  color: var(--color-text-primary);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cpd-identity__frag--muted {
  color: var(--color-text-muted);
}

.cpd-identity__frag--count {
  font-weight: 620;
  color: var(--color-text-primary);
}

.cpd-identity__frag--arrow {
  color: var(--color-text-muted);
  opacity: 0.78;
  font-family: var(--font-mono, monospace);
}

.cpd-identity__frag--badge {
  display: inline-flex;
  align-items: center;
  padding: 0.05rem 0.5rem;
  border-radius: 9999px;
  border: 1px solid currentcolor;
  font-size: 0.68rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.cpd-identity__frag--badge-ok {
  color: rgb(var(--color-success-rgb));
  border-color: rgb(var(--color-success-rgb) / 32%);
  background: rgb(var(--color-success-rgb) / 8%);
}

.cpd-identity__frag--badge-info {
  color: rgb(var(--color-info-rgb));
  border-color: rgb(var(--color-info-rgb) / 32%);
  background: rgb(var(--color-info-rgb) / 8%);
}

/* ── Band 2: Metric Ticker ────────────────────────────────── */
.cpd-metrics {
  position: relative;
  display: grid;
  grid-template-columns: repeat(8, minmax(0, 1fr));
  gap: 0;
  border-radius: 22px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 38%);
  background:
    linear-gradient(180deg,
      rgb(var(--color-bg-elevated-rgb) / 68%),
      rgb(var(--color-bg-surface-rgb) / 56%));
  box-shadow:
    0 14px 30px rgb(15 12 18 / 5%),
    inset 0 1px 0 rgb(255 255 255 / 5%);
  backdrop-filter: blur(18px) saturate(130%);
  overflow: hidden;
}

.cpd-metric {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 0.22rem;
  padding: 0.85rem 0.95rem;
  min-width: 0;
  animation: cpd-metric-enter 360ms ease both;
  animation-delay: calc(120ms + var(--i, 0) * 30ms);
}

.cpd-metric + .cpd-metric::before {
  content: '';
  position: absolute;
  top: 18%;
  bottom: 18%;
  left: 0;
  width: 1px;
  background: rgb(var(--color-border-default-rgb) / 22%);
}

.cpd-metric__label {
  font-size: 0.64rem;
  font-weight: 700;
  letter-spacing: 0.22em;
  text-transform: uppercase;
  color: var(--color-text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.cpd-metric__value {
  font-size: 1.45rem;
  font-weight: 620;
  letter-spacing: -0.02em;
  line-height: 1.08;
  color: var(--color-text-primary);
}

.cpd-metric__value--info {
  color: rgb(var(--color-info-rgb));
}

.cpd-metric__value--accent {
  color: rgb(var(--color-accent-primary-rgb));
}

.cpd-metric__value--success {
  color: rgb(var(--color-success-rgb));
}

.cpd-metric__value--warning {
  color: rgb(var(--color-warning-rgb, var(--color-danger-rgb)));
}

.cpd-metric__detail {
  display: -webkit-box;
  font-size: 0.7rem;
  line-height: 1.35;
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  line-clamp: 2;
}

/* Secondary tier: smaller value, muted label, still dense */
.cpd-metric--secondary .cpd-metric__value {
  font-size: 1.08rem;
  opacity: 0.92;
}

.cpd-metric--secondary .cpd-metric__label {
  opacity: 0.82;
}

.cpd-metric--secondary .cpd-metric__detail {
  font-size: 0.68rem;
  opacity: 0.82;
}

/* ── 断点降级 ─────────────────────────────────────────────── */
@media (width < 1280px) {
  .cpd-metrics {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }

  .cpd-metric:nth-child(4n+1)::before {
    display: none;
  }

  .cpd-metric:nth-child(n+5) {
    border-top: 1px solid rgb(var(--color-border-default-rgb) / 18%);
  }
}

@media (width < 640px) {
  .cpd-metrics {
    grid-template-columns: repeat(8, minmax(140px, 1fr));
    overflow-x: auto;
  }

  .cpd-metric:nth-child(n+5) {
    border-top: none;
  }

  .cpd-metric:nth-child(4n+1)::before {
    display: block;
  }

  .cpd-metric:first-child::before {
    display: none;
  }
}

/* ── Reduced motion ──────────────────────────────────────── */
@media (prefers-reduced-motion: reduce) {
  .cpd-metric {
    animation: none;
  }
}

@keyframes cpd-metric-enter {
  from {
    opacity: 0;
    transform: translateY(6px);
  }

  to {
    opacity: 1;
    transform: none;
  }
}
</style>
