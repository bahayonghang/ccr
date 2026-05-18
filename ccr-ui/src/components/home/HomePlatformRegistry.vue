<template>
  <section
    class="home-platforms"
    data-home-platforms
  >
    <header class="home-platforms__header">
      <div class="home-platforms__lede">
        <p class="home-platforms__eyebrow">
          {{ t('home.platformsEyebrow') }}
        </p>
        <h2 class="home-platforms__title">
          {{ t('home.platformsTitle') }}
        </h2>
        <p class="home-platforms__description">
          {{ t('home.platformsDescription') }}
        </p>
      </div>
      <span class="home-platforms__count">
        <span class="home-platforms__count-value">{{ installedCliCount }}/{{ runtimeCliCount }}</span>
        <span class="home-platforms__count-label">{{ t('home.visualBadge') }}</span>
      </span>
    </header>

    <div class="home-platforms__grid">
      <RouterLink
        v-for="platform in platforms"
        :key="platform.platformKey"
        :to="platform.path"
        class="home-platform"
        :class="`home-platform--${platform.platformKey}`"
      >
        <span
          class="home-platform__accent"
          aria-hidden="true"
        />

        <div class="home-platform__head">
          <span class="home-platform__icon">
            <SIcon
              :name="platform.icon"
              size="w-4 h-4"
              :class="platform.iconClass"
            />
          </span>
          <span class="home-platform__identity">
            <strong class="home-platform__title">{{ platform.title }}</strong>
            <span class="home-platform__version">{{ getVersionLabel(platform) }}</span>
          </span>
          <span
            class="home-platform__status"
            :data-state="getPlatformState(platform)"
          >
            <span
              class="home-platform__status-dot"
              aria-hidden="true"
            />
            {{ getStateLabel(platform) }}
          </span>
        </div>

        <p class="home-platform__lede">
          <span class="home-platform__role">{{ platform.role }}</span>
          <span
            class="home-platform__sep"
            aria-hidden="true"
          >·</span>
          <span class="home-platform__desc">{{ platform.desc }}</span>
        </p>

        <div class="home-platform__metrics">
          <div class="home-platform__metric home-platform__metric--primary">
            <span class="home-platform__metric-label">{{ t('home.platformStatRequests') }}</span>
            <span class="home-platform__metric-value">{{ getPlatformMetric(platform, 'requests') }}</span>
          </div>
          <div class="home-platform__metric">
            <span class="home-platform__metric-label">{{ t('home.platformStatSessions') }}</span>
            <span class="home-platform__metric-value">{{ getPlatformMetric(platform, 'sessions') }}</span>
          </div>
          <div class="home-platform__metric">
            <span class="home-platform__metric-label">{{ t('home.platformStatTokens') }}</span>
            <span class="home-platform__metric-value">{{ getPlatformMetric(platform, 'tokens') }}</span>
          </div>
        </div>

        <span
          class="home-platform__cta"
          aria-hidden="true"
        >
          <SIcon
            name="ArrowRight"
            size="w-4 h-4"
          />
        </span>
      </RouterLink>
    </div>
  </section>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type { CliVersionEntry } from '@/types'
import type { HomeUsageOverviewResponse } from '@/types/usage'
import type { HomePlatformRecord, HomeUsageMetric } from './types'

const props = defineProps<{
  platforms: HomePlatformRecord[]
  cliVersions: Map<string, CliVersionEntry>
  overview: HomeUsageOverviewResponse | null
  installedCliCount: number
  runtimeCliCount: number
}>()

const { t } = useI18n()

const formatCompact = (value?: number) => {
  if (typeof value !== 'number') return t('home.platformUsageUntracked')
  return new Intl.NumberFormat(undefined, {
    notation: 'compact',
    maximumFractionDigits: 1,
  }).format(value)
}

const getEntry = (platform: HomePlatformRecord) => props.cliVersions.get(platform.platformKey)

const getPlatformState = (platform: HomePlatformRecord) => {
  if (platform.mode === 'managed') return 'managed'

  const entry = getEntry(platform)
  if (!entry) return 'scanning'
  if (entry.status === 'timeout') return 'scanning'
  if (entry.status === 'error' || entry.status === 'not_installed' || !entry.installed) return 'attention'
  return 'ready'
}

const getStateLabel = (platform: HomePlatformRecord) => {
  switch (getPlatformState(platform)) {
    case 'managed':
      return t('home.moduleStateManaged')
    case 'scanning':
      return t('home.moduleStateScanning')
    case 'attention':
      return t('home.moduleStateAttention')
    default:
      return t('home.moduleStateReady')
  }
}

const getVersionLabel = (platform: HomePlatformRecord) => {
  if (platform.mode === 'managed') return t('home.moduleManagedLabel')

  const entry = getEntry(platform)
  if (!entry || entry.status === 'timeout' || entry.status === 'error') return t('home.moduleStateScanning')
  if (entry.status === 'not_installed' || !entry.installed) return t('home.notInstalled')
  return entry.version ? `v${entry.version}` : t('common.installed')
}

const getPlatformMetric = (platform: HomePlatformRecord, metric: HomeUsageMetric) => {
  if (!platform.usageKey) return t('home.platformUsageUntracked')
  const stats = props.overview?.by_platform[platform.usageKey]
  return formatCompact(stats?.[metric])
}
</script>

<style scoped>
.home-platforms {
  display: grid;
  gap: 0.85rem;
}

.home-platforms__header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 1.25rem;
}

.home-platforms__lede {
  display: grid;
  gap: 0.18rem;
  min-width: 0;
}

.home-platforms__eyebrow {
  margin: 0;
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.home-platforms__title {
  margin: 0;
  color: var(--color-text-primary);
  font-family: var(--font-brand);
  font-size: var(--home-text-section);
  font-weight: 620;
  letter-spacing: var(--home-tracking-display);
}

.home-platforms__description {
  max-width: 48rem;
  margin: 0;
  color: var(--color-text-secondary);
  font-size: var(--home-text-body);
  letter-spacing: var(--home-tracking-body);
  line-height: var(--home-leading-body);
}

.home-platforms__count {
  flex: 0 0 auto;
  display: inline-flex;
  align-items: baseline;
  gap: 0.5rem;
  padding: 0.32rem 0.8rem;
  border: 1px solid var(--home-border-hairline);
  border-radius: 999px;
  background: var(--home-surface-sunk);
  box-shadow: var(--home-elevation-sunk);
}

.home-platforms__count-value {
  color: var(--color-text-primary);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-mono);
  font-weight: 700;
}

.home-platforms__count-label {
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.home-platforms__grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.75rem;
}

.home-platform {
  position: relative;
  display: grid;
  grid-template-rows: auto auto 1fr;
  gap: 0.7rem;
  padding: 0.95rem 1rem 0.95rem 1.15rem;
  border: 1px solid var(--home-border-card);
  border-radius: var(--home-card-radius);
  background: var(--home-surface-card);
  box-shadow: var(--home-elevation-raised);
  color: var(--color-text-primary);
  text-decoration: none;
  transition:
    border-color var(--home-motion-duration) var(--home-motion-ease),
    background-color var(--home-motion-duration) var(--home-motion-ease),
    transform var(--home-motion-duration) var(--home-motion-ease);
}

.home-platform:hover {
  border-color: var(--home-border-card-hover);
  background: var(--home-surface-card-hover);
  transform: translateY(var(--home-motion-lift));
}

.home-platform:focus-visible {
  outline: 0;
  box-shadow: var(--home-focus-ring);
}

.home-platform:active {
  transform: translateY(0);
  box-shadow: var(--home-elevation-sunk);
}

.home-platform__accent {
  position: absolute;
  top: 0.95rem;
  bottom: 0.95rem;
  left: 0.45rem;
  width: 3px;
  border-radius: 999px;
  background: var(--color-text-disabled);
  transition:
    width var(--home-motion-duration) var(--home-motion-ease),
    background-color var(--home-motion-duration) var(--home-motion-ease);
}

.home-platform--claude-code .home-platform__accent { background: var(--color-platform-claude); }
.home-platform--codex .home-platform__accent { background: var(--color-platform-codex); }
.home-platform--gemini-cli .home-platform__accent { background: var(--color-platform-gemini); }
.home-platform--opencode .home-platform__accent { background: var(--color-info); }

.home-platform:hover .home-platform__accent,
.home-platform:focus-visible .home-platform__accent {
  width: 4px;
}

.home-platform__head {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.65rem;
}

.home-platform__icon {
  display: grid;
  place-items: center;
  width: 1.7rem;
  height: 1.7rem;
  border: 1px solid var(--home-border-card);
  border-radius: 7px;
  background: rgb(var(--color-bg-elevated-rgb) / 96%);
}

.home-platform__identity {
  display: grid;
  gap: 0.1rem;
  min-width: 0;
}

.home-platform__title {
  overflow: hidden;
  color: var(--color-text-primary);
  font-size: var(--home-text-body);
  font-weight: 620;
  letter-spacing: var(--home-tracking-body);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.home-platform__version {
  color: var(--color-text-muted);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-meta);
  font-weight: 700;
}

.home-platform__status {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.18rem 0.5rem;
  border: 1px solid var(--home-border-hairline);
  border-radius: 999px;
  background: var(--home-surface-sunk);
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
  white-space: nowrap;
}

.home-platform__status-dot {
  width: 0.42rem;
  height: 0.42rem;
  border-radius: 999px;
  background: var(--color-text-disabled);
}

.home-platform__status[data-state='ready'] .home-platform__status-dot { background: var(--color-success); }
.home-platform__status[data-state='ready'] { color: var(--color-success); }
.home-platform__status[data-state='attention'] .home-platform__status-dot { background: var(--color-danger); }
.home-platform__status[data-state='attention'] { color: var(--color-danger); }
.home-platform__status[data-state='scanning'] .home-platform__status-dot { background: var(--color-warning); }
.home-platform__status[data-state='scanning'] { color: var(--color-warning); }
.home-platform__status[data-state='managed'] .home-platform__status-dot { background: var(--color-accent-primary); }
.home-platform__status[data-state='managed'] { color: var(--color-accent-primary); }

.home-platform__lede {
  display: -webkit-box;
  overflow: hidden;
  margin: 0;
  color: var(--color-text-secondary);
  font-size: var(--home-text-body);
  letter-spacing: var(--home-tracking-body);
  line-height: var(--home-leading-body);
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.home-platform__role {
  color: var(--color-text-muted);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.home-platform__sep {
  margin: 0 0.4rem;
  color: var(--color-text-disabled);
}

.home-platform__desc {
  color: var(--color-text-secondary);
}

.home-platform__metrics {
  display: grid;
  grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr) minmax(0, 1fr);
  align-items: end;
  gap: 0.55rem;
}

.home-platform__metric {
  display: grid;
  gap: 0.12rem;
  padding: 0.32rem 0;
}

.home-platform__metric-label {
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 700;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.home-platform__metric-value {
  color: var(--color-text-primary);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-mono);
  font-weight: 700;
}

.home-platform__metric--primary .home-platform__metric-value {
  font-size: var(--home-text-mono-lg);
}

.home-platform__cta {
  position: absolute;
  top: 1rem;
  right: 1rem;
  display: grid;
  place-items: center;
  width: 1.5rem;
  height: 1.5rem;
  border-radius: 999px;
  background: rgb(var(--color-accent-primary-rgb) / 0%);
  color: var(--color-text-muted);
  opacity: 0;
  transform: translateX(-4px);
  transition:
    opacity var(--home-motion-duration) var(--home-motion-ease),
    transform var(--home-motion-duration) var(--home-motion-ease),
    color var(--home-motion-duration) var(--home-motion-ease);
}

.home-platform:hover .home-platform__cta,
.home-platform:focus-visible .home-platform__cta {
  opacity: 1;
  transform: translateX(0);
  color: var(--color-accent-primary);
}

@media (width <= 720px) {
  .home-platforms__header {
    flex-direction: column;
    align-items: flex-start;
  }

  .home-platforms__grid {
    grid-template-columns: 1fr;
  }
}

@media (prefers-reduced-motion: reduce) {
  .home-platform,
  .home-platform__accent,
  .home-platform__cta {
    transition: none;
  }
}
</style>
