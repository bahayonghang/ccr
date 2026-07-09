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
          class="dashboard-platform__accent"
          aria-hidden="true"
        />
        <span class="dashboard-platform__icon">
          <SIcon
            :name="platform.icon"
            size="w-4 h-4"
            :class="platform.iconClass"
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
  font-size: var(--home-text-meta);
  font-weight: 800;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.dashboard-platforms__title {
  margin: 0;
  color: var(--color-text-primary);
  font-family: var(--font-brand);
  font-size: var(--home-text-section);
  font-weight: 640;
  letter-spacing: var(--home-tracking-display);
}

.dashboard-platforms__description {
  max-width: 54rem;
  margin: 0;
  color: var(--color-text-secondary);
  font-size: var(--home-text-body);
  line-height: var(--home-leading-body);
}

.dashboard-platforms__count {
  display: inline-flex;
  align-items: baseline;
  gap: 0.5rem;
  padding: 0.34rem 0.75rem;
  border: 1px solid var(--home-border-hairline);
  border-radius: 999px;
  background: var(--home-surface-sunk);
  box-shadow: var(--home-elevation-sunk);
  white-space: nowrap;
}

.dashboard-platforms__count strong {
  color: var(--color-text-primary);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-mono);
}

.dashboard-platforms__count span {
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 800;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.dashboard-platforms__matrix {
  display: grid;
  gap: 0.45rem;
}

.dashboard-platform {
  position: relative;
  display: grid;
  grid-template-columns: auto minmax(9rem, 1.2fr) auto minmax(6rem, 0.6fr) minmax(12rem, 2fr) repeat(3, minmax(4.5rem, 0.55fr)) auto;
  align-items: center;
  gap: 0.65rem;
  min-width: 0;
  padding: 0.72rem 0.85rem 0.72rem 1rem;
  border: 1px solid var(--home-border-card);
  border-radius: 12px;
  background: var(--home-surface-card);
  box-shadow: var(--home-elevation-raised);
  color: var(--color-text-primary);
  text-decoration: none;
  transition:
    border-color var(--home-motion-duration) var(--home-motion-ease),
    background-color var(--home-motion-duration) var(--home-motion-ease),
    transform var(--home-motion-duration) var(--home-motion-ease);
}

.dashboard-platform:hover {
  border-color: var(--home-border-card-hover);
  background: var(--home-surface-card-hover);
  transform: translateY(var(--home-motion-lift));
}

.dashboard-platform:focus-visible {
  outline: 0;
  box-shadow: var(--home-focus-ring);
}

.dashboard-platform__accent {
  position: absolute;
  top: 0.75rem;
  bottom: 0.75rem;
  left: 0.38rem;
  width: 3px;
  border-radius: 999px;
  background: var(--color-text-disabled);
}

.dashboard-platform--claude-code .dashboard-platform__accent { background: var(--color-platform-claude); }
.dashboard-platform--codex .dashboard-platform__accent { background: var(--color-platform-codex); }
.dashboard-platform--antigravity .dashboard-platform__accent { background: var(--color-platform-gemini); }
.dashboard-platform--opencode .dashboard-platform__accent { background: var(--color-info); }

.dashboard-platform__icon {
  display: grid;
  place-items: center;
  width: 1.7rem;
  height: 1.7rem;
  border: 1px solid var(--home-border-card);
  border-radius: 8px;
  background: rgb(var(--color-bg-elevated-rgb) / 94%);
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
  font-size: var(--home-text-body);
  font-weight: 650;
}

.dashboard-platform__identity span,
.dashboard-platform__role,
.dashboard-platform__metric span {
  color: var(--color-text-muted);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-meta);
  font-weight: 800;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.dashboard-platform__version-skeleton {
  display: inline-block;
  width: 48px;
  height: 12px;
  border-radius: 4px;
  background: linear-gradient(
    90deg,
    rgb(var(--color-border-default-rgb) / 16%) 25%,
    rgb(var(--color-border-default-rgb) / 34%) 50%,
    rgb(var(--color-border-default-rgb) / 16%) 75%
  );
  background-size: 200% 100%;
  animation: dashboard-platform-skeleton-pulse 1.4s ease-in-out infinite;
}

@keyframes dashboard-platform-skeleton-pulse {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

.dashboard-platform__status {
  display: inline-flex;
  align-items: center;
  justify-self: start;
  gap: 0.35rem;
  padding: 0.18rem 0.5rem;
  border: 1px solid var(--home-border-hairline);
  border-radius: 999px;
  background: var(--home-surface-sunk);
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 800;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
  white-space: nowrap;
}

.dashboard-platform__status-dot {
  width: 0.42rem;
  height: 0.42rem;
  border-radius: 999px;
  background: var(--color-text-disabled);
}

.dashboard-platform__status[data-state='ready'],
.dashboard-platform__status[data-state='ready'] .dashboard-platform__status-dot {
  color: var(--color-success);
  background-color: var(--color-success);
}

.dashboard-platform__status[data-state='ready'] {
  background: var(--home-surface-sunk);
}

.dashboard-platform__status[data-state='attention'],
.dashboard-platform__status[data-state='attention'] .dashboard-platform__status-dot {
  color: var(--color-danger);
  background-color: var(--color-danger);
}

.dashboard-platform__status[data-state='attention'] {
  background: var(--home-surface-sunk);
}

.dashboard-platform__status[data-state='scanning'],
.dashboard-platform__status[data-state='scanning'] .dashboard-platform__status-dot {
  color: var(--color-warning);
  background-color: var(--color-warning);
}

.dashboard-platform__status[data-state='scanning'] {
  background: var(--home-surface-sunk);
}

.dashboard-platform__status[data-state='managed'],
.dashboard-platform__status[data-state='managed'] .dashboard-platform__status-dot {
  color: var(--color-accent-primary);
  background-color: var(--color-accent-primary);
}

.dashboard-platform__status[data-state='managed'] {
  background: var(--home-surface-sunk);
}

.dashboard-platform__desc {
  color: var(--color-text-secondary);
  font-size: var(--home-text-meta);
}

.dashboard-platform__metric {
  display: grid;
  gap: 0.1rem;
  min-width: 0;
}

.dashboard-platform__metric strong {
  color: var(--color-text-primary);
  font-family: var(--font-mono);
  font-feature-settings: var(--home-mono-feature);
  font-size: var(--home-text-mono);
  font-weight: 800;
}

.dashboard-platform__cta {
  color: var(--color-text-muted);
  opacity: 0;
  transform: translateX(-3px);
  transition:
    color var(--home-motion-duration) var(--home-motion-ease),
    opacity var(--home-motion-duration) var(--home-motion-ease),
    transform var(--home-motion-duration) var(--home-motion-ease);
}

.dashboard-platform:hover .dashboard-platform__cta,
.dashboard-platform:focus-visible .dashboard-platform__cta {
  color: var(--color-accent-primary);
  opacity: 1;
  transform: translateX(0);
}

@media (width <= 1180px) {
  .dashboard-platform {
    grid-template-columns: auto minmax(0, 1fr) auto;
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
    grid-template-columns: auto minmax(0, 1fr);
  }

  .dashboard-platform__status,
  .dashboard-platform__cta {
    grid-column: 2;
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

