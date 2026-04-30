<template>
  <section
    class="home-platforms"
    data-home-platforms
  >
    <header class="home-platforms__header">
      <div>
        <p class="home-section-kicker">
          {{ t('home.platformsEyebrow') }}
        </p>
        <h2>{{ t('home.platformsTitle') }}</h2>
        <p>{{ t('home.platformsDescription') }}</p>
      </div>
      <span class="home-platforms__count">
        {{ installedCliCount }}/{{ runtimeCliCount }} {{ t('home.visualBadge') }}
      </span>
    </header>

    <div class="home-platforms__grid">
      <RouterLink
        v-for="platform in platforms"
        :key="platform.platformKey"
        :to="platform.path"
        class="home-platform"
        :class="[
          `home-platform--${platform.mode}`,
          `home-platform--${getPlatformState(platform)}`,
        ]"
      >
        <div class="home-platform__top">
          <span class="home-platform__icon">
            <SIcon
              :name="platform.icon"
              size="w-5 h-5"
              :class="platform.iconClass"
            />
          </span>
          <span
            class="home-platform__status"
            :data-state="getPlatformState(platform)"
          >
            <span class="home-platform__dot" />
            {{ getStateLabel(platform) }}
          </span>
        </div>

        <div>
          <h3>{{ platform.title }}</h3>
          <p class="home-platform__role">
            {{ platform.role }} · {{ getVersionLabel(platform) }}
          </p>
        </div>

        <p class="home-platform__description">
          {{ platform.desc }}
        </p>

        <dl class="home-platform__stats">
          <div>
            <dt>{{ t('home.platformStatRequests') }}</dt>
            <dd>{{ getPlatformMetric(platform, 'requests') }}</dd>
          </div>
          <div>
            <dt>{{ t('home.platformStatSessions') }}</dt>
            <dd>{{ getPlatformMetric(platform, 'sessions') }}</dd>
          </div>
          <div>
            <dt>{{ t('home.platformStatTokens') }}</dt>
            <dd>{{ getPlatformMetric(platform, 'tokens') }}</dd>
          </div>
        </dl>
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
  gap: 1rem;
}

.home-platforms__header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 1rem;
}

.home-section-kicker {
  color: var(--color-text-muted);
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.home-platforms h2 {
  margin: 0.25rem 0 0;
  color: var(--color-text-primary);
  font-size: 1.2rem;
  font-weight: 650;
  letter-spacing: 0;
}

.home-platforms__header p:not(.home-section-kicker) {
  max-width: 48rem;
  margin: 0.35rem 0 0;
  color: var(--color-text-secondary);
  font-size: 0.86rem;
  line-height: 1.6;
}

.home-platforms__count {
  flex: 0 0 auto;
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  border-radius: 999px;
  background: rgb(var(--color-bg-elevated-rgb) / 80%);
  color: var(--color-text-secondary);
  font-family: var(--font-mono);
  font-size: 0.72rem;
  font-weight: 700;
  padding: 0.42rem 0.65rem;
}

.home-platforms__grid {
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 0.7rem;
}

.home-platform {
  display: grid;
  gap: 0.75rem;
  min-height: 13rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 14%);
  border-radius: 12px;
  background: rgb(var(--color-bg-elevated-rgb) / 86%);
  color: var(--color-text-primary);
  padding: 0.85rem;
  transition:
    border-color 160ms ease,
    background-color 160ms ease,
    transform 160ms ease;
}

.home-platform:hover,
.home-platform:focus-visible {
  border-color: rgb(var(--color-accent-primary-rgb) / 28%);
  background: rgb(var(--color-bg-surface-rgb) / 78%);
  transform: translateY(-1px);
}

.home-platform__top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
}

.home-platform__icon {
  display: grid;
  place-items: center;
  width: 2.15rem;
  height: 2.15rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 12%);
  border-radius: 8px;
  background: rgb(var(--color-bg-surface-rgb) / 72%);
}

.home-platform__status {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  border: 1px solid rgb(var(--color-border-default-rgb) / 12%);
  border-radius: 999px;
  background: rgb(var(--color-bg-surface-rgb) / 58%);
  color: var(--color-text-secondary);
  font-size: 0.68rem;
  font-weight: 700;
  padding: 0.25rem 0.45rem;
  white-space: nowrap;
}

.home-platform__dot {
  width: 0.42rem;
  height: 0.42rem;
  border-radius: 999px;
  background: var(--color-text-muted);
}

.home-platform__status[data-state='ready'] .home-platform__dot {
  background: var(--accent-success);
}

.home-platform__status[data-state='attention'] .home-platform__dot {
  background: var(--color-danger);
}

.home-platform__status[data-state='scanning'] .home-platform__dot {
  background: var(--accent-warning);
}

.home-platform__status[data-state='managed'] .home-platform__dot {
  background: var(--color-accent-primary);
}

.home-platform h3 {
  margin: 0;
  color: var(--color-text-primary);
  font-size: 0.98rem;
  font-weight: 650;
  letter-spacing: 0;
}

.home-platform__role {
  margin: 0.2rem 0 0;
  color: var(--color-text-muted);
  font-family: var(--font-mono);
  font-size: 0.68rem;
  text-transform: uppercase;
}

.home-platform__description {
  display: -webkit-box;
  overflow: hidden;
  min-height: 2.8rem;
  color: var(--color-text-secondary);
  font-size: 0.78rem;
  line-height: 1.55;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.home-platform__stats {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 1px;
  overflow: hidden;
  align-self: end;
  margin: 0;
  border: 1px solid rgb(var(--color-border-default-rgb) / 10%);
  border-radius: 8px;
  background: rgb(var(--color-border-default-rgb) / 8%);
}

.home-platform__stats div {
  min-width: 0;
  background: rgb(var(--color-bg-surface-rgb) / 58%);
  padding: 0.48rem;
}

.home-platform__stats dt {
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 0.58rem;
  font-weight: 700;
  letter-spacing: 0.07em;
  text-overflow: ellipsis;
  text-transform: uppercase;
  white-space: nowrap;
}

.home-platform__stats dd {
  overflow: hidden;
  margin: 0.15rem 0 0;
  color: var(--color-text-primary);
  font-family: var(--font-mono);
  font-size: 0.78rem;
  font-weight: 700;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (width <= 1500px) {
  .home-platforms__grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}

@media (width <= 900px) {
  .home-platforms__grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (width <= 640px) {
  .home-platforms__header {
    align-items: flex-start;
    flex-direction: column;
  }

  .home-platforms__grid {
    grid-template-columns: 1fr;
  }
}

@media (prefers-reduced-motion: reduce) {
  .home-platform {
    transition: none;
  }
}
</style>
