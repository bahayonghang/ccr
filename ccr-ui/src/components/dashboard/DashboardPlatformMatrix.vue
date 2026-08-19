<template>
  <section
    class="dashboard-platforms"
    data-dashboard-platforms
  >
    <header class="dashboard-platforms__header">
      <div class="dashboard-platforms__lede">
        <p class="dashboard-platforms__eyebrow">
          {{ t('dashboard.platforms.eyebrow') }}
        </p>
        <h2 class="dashboard-platforms__title">
          {{ t('dashboard.platforms.title') }}
        </h2>
        <p class="dashboard-platforms__description">
          {{ t('dashboard.platforms.description') }}
        </p>
      </div>
      <span class="dashboard-platforms__count">
        <strong>{{ installedCliCount }}/{{ runtimeCliCount }}</strong>
        <span>{{ t('dashboard.platforms.detectedLabel') }}</span>
      </span>
    </header>

    <div class="dashboard-platforms__matrix">
      <RouterLink
        v-for="platform in rows"
        :key="platform.platformKey"
        :to="platform.path"
        class="dashboard-platform"
        :class="`dashboard-platform--${platform.platformKey}`"
      >
        <span
          class="dashboard-platform__mark"
          aria-hidden="true"
        />
        <span class="dashboard-platform__icon">
          <SIcon
            :name="platform.icon"
            size="w-4 h-4"
          />
        </span>
        <span class="dashboard-platform__identity">
          <strong>{{ platform.title }}</strong>
          <span
            v-if="platform.versionKey === 'dashboard.platforms.stateScanning'"
            class="dashboard-platform__version-skeleton"
            role="status"
            :aria-label="t(platform.versionKey)"
          />
          <span v-else>{{ resolveVersion(platform) }}</span>
        </span>
        <span
          class="dashboard-platform__status"
          :data-state="platform.state"
        >
          <span
            class="dashboard-platform__status-dot"
            aria-hidden="true"
          />
          {{ t(platform.stateKey) }}
        </span>
        <span class="dashboard-platform__role">{{ platform.role }}</span>
        <span class="dashboard-platform__desc">{{ platform.desc }}</span>
        <span
          v-for="metric in platform.metrics"
          :key="`${platform.platformKey}-${metric.labelKey}`"
          class="dashboard-platform__metric"
        >
          <span>{{ t(metric.labelKey) }}</span>
          <strong>{{ resolveMetric(metric) }}</strong>
        </span>
        <span
          class="dashboard-platform__cta"
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
import type {
  DashboardMetricValue,
  DashboardPlatformRow,
} from '@/views/dashboard/dashboardPresentation'

defineProps<{
  rows: DashboardPlatformRow[]
  installedCliCount: number
  runtimeCliCount: number
}>()

const { t } = useI18n()

const resolveMetric = (metric: DashboardMetricValue) => {
  if (metric.valueKey) return t(metric.valueKey)
  return metric.value ?? '…'
}

const resolveVersion = (platform: DashboardPlatformRow) => {
  if (platform.versionKey) return t(platform.versionKey)
  return platform.version ?? '…'
}
</script>

<style scoped>
.dashboard-platforms {
  display: grid;
  gap: 0.85rem;
}

.dashboard-platforms__header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 1rem;
}

.dashboard-platforms__lede {
  display: grid;
  gap: 0.18rem;
  min-width: 0;
}

.dashboard-platforms__eyebrow {
  margin: 0;
  color: var(--color-text-muted);
  font-size: 0.8125rem;
  font-weight: 500;
  line-height: 1.24;
  letter-spacing: 0;
}

.dashboard-platforms__title {
  margin: 0;
  color: var(--color-text-primary);
  font-size: 1.0625rem;
  font-weight: 600;
  line-height: 1.3;
  letter-spacing: 0;
}

.dashboard-platforms__description {
  max-width: 54rem;
  margin: 0;
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  line-height: 1.5;
}

.dashboard-platforms__count {
  display: inline-flex;
  align-items: baseline;
  gap: 0.5rem;
  padding: 0.3rem 0.65rem;
  border: 1px solid var(--color-border-subtle);
  border-radius: 8px;
  background: var(--color-bg-elevated);
  white-space: nowrap;
}

.dashboard-platforms__count strong {
  color: var(--color-text-primary);
  font-size: 0.875rem;
  font-variant-numeric: tabular-nums;
  font-weight: 600;
}

.dashboard-platforms__count span {
  color: var(--color-text-muted);
  font-size: 0.75rem;
  font-weight: 500;
  letter-spacing: 0;
}

.dashboard-platforms__matrix {
  display: grid;
  gap: 0.4rem;
}

.dashboard-platform {
  display: grid;
  grid-template-columns: auto auto minmax(9rem, 1.2fr) auto minmax(6rem, 0.6fr) minmax(12rem, 2fr) repeat(3, minmax(4.5rem, 0.55fr)) auto;
  align-items: center;
  gap: 0.65rem;
  min-width: 0;
  padding: 0.65rem 0.85rem;
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--home-card-radius);
  background: var(--color-bg-surface);
  color: var(--color-text-primary);
  text-decoration: none;
  transition:
    border-color var(--home-motion-duration) var(--home-motion-ease),
    background-color var(--home-motion-duration) var(--home-motion-ease);
}

.dashboard-platform:hover {
  border-color: var(--color-border-strong);
  background: var(--color-bg-elevated);
}

.dashboard-platform:focus-visible {
  outline: 2px solid var(--color-accent-primary);
  outline-offset: 2px;
}

.dashboard-platform__mark {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 2px;
  background: var(--color-text-disabled);
}

.dashboard-platform--claude-code .dashboard-platform__mark {
  background: var(--color-platform-claude);
}

.dashboard-platform--codex .dashboard-platform__mark {
  background: var(--color-platform-codex);
}

.dashboard-platform--antigravity .dashboard-platform__mark {
  background: var(--color-platform-gemini);
}

.dashboard-platform--opencode .dashboard-platform__mark {
  background: var(--color-info);
}

.dashboard-platform__icon {
  display: grid;
  place-items: center;
  width: 1.6rem;
  height: 1.6rem;
  color: var(--color-text-secondary);
}

.dashboard-platform__identity {
  display: grid;
  gap: 0.1rem;
  min-width: 0;
}

.dashboard-platform__identity strong,
.dashboard-platform__identity span,
.dashboard-platform__role,
.dashboard-platform__desc,
.dashboard-platform__metric strong {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dashboard-platform__identity strong {
  color: var(--color-text-primary);
  font-size: 0.875rem;
  font-weight: 600;
}

.dashboard-platform__identity span,
.dashboard-platform__role,
.dashboard-platform__metric span {
  color: var(--color-text-muted);
  font-size: 0.75rem;
  font-weight: 500;
  letter-spacing: 0;
}

.dashboard-platform__version-skeleton {
  display: inline-block;
  width: 48px;
  height: 12px;
  border-radius: 4px;
  background: rgb(var(--color-border-default-rgb) / 22%);
  animation: dashboard-platform-skeleton-pulse 1.4s ease-in-out infinite;
}

@keyframes dashboard-platform-skeleton-pulse {
  0%,
  100% { opacity: 0.45; }
  50% { opacity: 1; }
}

.dashboard-platform__status {
  display: inline-flex;
  align-items: center;
  justify-self: start;
  gap: 0.35rem;
  padding: 0.16rem 0.45rem;
  border: 1px solid var(--color-border-subtle);
  border-radius: 6px;
  background: var(--color-bg-elevated);
  color: var(--color-text-muted);
  font-size: 0.75rem;
  font-weight: 500;
  letter-spacing: 0;
  white-space: nowrap;
}

.dashboard-platform__status-dot {
  width: 0.4rem;
  height: 0.4rem;
  border-radius: 999px;
  background: var(--color-text-disabled);
}

.dashboard-platform__status[data-state='ready'] {
  color: var(--color-success);
}

.dashboard-platform__status[data-state='ready'] .dashboard-platform__status-dot {
  background: var(--color-success);
}

.dashboard-platform__status[data-state='attention'] {
  color: var(--color-danger);
}

.dashboard-platform__status[data-state='attention'] .dashboard-platform__status-dot {
  background: var(--color-danger);
}

.dashboard-platform__status[data-state='scanning'] {
  color: var(--color-warning);
}

.dashboard-platform__status[data-state='scanning'] .dashboard-platform__status-dot {
  background: var(--color-warning);
}

.dashboard-platform__status[data-state='managed'] {
  color: var(--color-info);
}

.dashboard-platform__status[data-state='managed'] .dashboard-platform__status-dot {
  background: var(--color-info);
}

.dashboard-platform__desc {
  color: var(--color-text-secondary);
  font-size: 0.75rem;
}

.dashboard-platform__metric {
  display: grid;
  gap: 0.1rem;
  min-width: 0;
}

.dashboard-platform__metric strong {
  color: var(--color-text-primary);
  font-size: 0.8125rem;
  font-variant-numeric: tabular-nums;
  font-weight: 600;
}

.dashboard-platform__cta {
  color: var(--color-text-muted);
  opacity: 0;
}

.dashboard-platform:hover .dashboard-platform__cta,
.dashboard-platform:focus-visible .dashboard-platform__cta {
  color: var(--color-text-primary);
  opacity: 1;
}

@media (width <= 1180px) {
  .dashboard-platform {
    grid-template-columns: auto auto minmax(0, 1fr) auto;
  }

  .dashboard-platform__role,
  .dashboard-platform__desc,
  .dashboard-platform__metric {
    display: none;
  }
}

@media (width <= 720px) {
  .dashboard-platforms__header {
    flex-direction: column;
    align-items: flex-start;
  }

  .dashboard-platform {
    grid-template-columns: auto auto minmax(0, 1fr);
  }

  .dashboard-platform__status,
  .dashboard-platform__cta {
    grid-column: 3;
  }
}

@media (prefers-reduced-motion: reduce) {
  .dashboard-platform,
  .dashboard-platform__cta {
    transition: none;
  }

  .dashboard-platform__version-skeleton {
    animation: none;
  }
}
</style>
