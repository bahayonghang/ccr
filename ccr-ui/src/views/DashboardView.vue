<template>
  <main class="dashboard-view">
    <div class="dashboard-workbench">
      <section
        class="dashboard-hero"
        data-dashboard-hero
      >
        <div class="dashboard-hero__lede">
          <p class="dashboard-hero__eyebrow">
            <span class="dashboard-hero__eyebrow-anchor" />
            {{ t('dashboard.eyebrow') }}
          </p>
          <h1 class="dashboard-hero__title">
            {{ t('dashboard.title') }}
          </h1>
          <p class="dashboard-hero__description">
            {{ t('dashboard.description') }}
          </p>
        </div>
        <button
          type="button"
          class="dashboard-hero__badge"
          :data-tone="heroBadgeTone"
          :aria-label="t('dashboard.readiness.label')"
          @click="scrollToReadiness"
        >
          <span
            class="dashboard-hero__badge-dot"
            aria-hidden="true"
          />
          {{ t(heroBadgeLabelKey) }}
        </button>
      </section>

      <section class="dashboard-grid dashboard-grid--top">
        <DashboardNextActions
          class="dashboard-grid__actions"
          :actions="dashboardPresentation.actions"
          :show-onboarding="dashboardPresentation.isFirstRun"
        />
        <DashboardReadinessLedger
          class="dashboard-grid__readiness"
          :readiness="dashboardPresentation.readiness"
          :status-metrics="dashboardPresentation.statusMetrics"
        />
      </section>

      <section class="dashboard-grid dashboard-grid--middle">
        <DashboardUsageMovement
          class="dashboard-grid__usage"
          :overview="overview"
          :loading="usageLoading"
          :error="usageError"
          :active-days="activeDays"
          @change-days="loadUsageOverview"
        />
        <DashboardSignalStream
          class="dashboard-grid__signals"
          :entries="logs"
          :limit="6"
        />
      </section>

      <DashboardPlatformMatrix
        :rows="dashboardPresentation.platformRows"
        :installed-cli-count="dashboardPresentation.installedCliCount"
        :runtime-cli-count="dashboardPresentation.runtimeCliCount"
      />
    </div>
  </main>
</template>

<script setup lang="ts">
import { computed, onActivated, onBeforeUnmount, onDeactivated, onMounted, ref } from 'vue'
import { getErrorMessage } from '@/utils/errorHandler'
import { useI18n } from 'vue-i18n'
import { storeToRefs } from 'pinia'
import DashboardNextActions from '@/components/dashboard/DashboardNextActions.vue'
import DashboardPlatformMatrix from '@/components/dashboard/DashboardPlatformMatrix.vue'
import DashboardReadinessLedger from '@/components/dashboard/DashboardReadinessLedger.vue'
import DashboardSignalStream from '@/components/dashboard/DashboardSignalStream.vue'
import DashboardUsageMovement from '@/components/dashboard/DashboardUsageMovement.vue'
import { getCliVersions, getSystemInfo } from '@/api/runtime/system'
import { useMonitoringFeed } from '@/composables/useMonitoringFeed'
import { useHomeUsageOverviewStore } from '@/stores/homeUsageOverview'
import { logger } from '@/utils/logger'
import { perfMark, shouldLogPerfTelemetry } from '@/utils/perfTelemetry'
import { scheduleWhenIdle } from '@/utils/scheduling'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import {
  buildDashboardPresentation,
  type DashboardBackendStatus,
  type DashboardPlatformSource,
} from '@/views/dashboard/dashboardPresentation'
import type { CliVersionEntry, CliVersionsResponse, SystemInfo } from '@/types'

const { t } = useI18n()
const usageOverviewStore = useHomeUsageOverviewStore()
const { overview, loading: usageLoading, error: usageError, activeDays } = storeToRefs(usageOverviewStore)
const { logs, pause: pauseFeed, resume: resumeFeed } = useMonitoringFeed({
  initialCount: 6,
  maxEntries: 24,
})

const systemInfo = ref<SystemInfo | null>(null)
const systemInfoError = ref<string | null>(null)
const cliVersions = ref<Map<string, CliVersionEntry>>(new Map())
const cliVersionsLoaded = ref(false)
const isNativeRuntime = isTauriRuntime()

const CLI_PLATFORM_ALIASES: Record<string, string> = {
  claude: 'claude-code',
  'claude-code': 'claude-code',
  codex: 'codex',
  gemini: 'antigravity',
  agy: 'antigravity',
  antigravity: 'antigravity',
  'antigravity-cli': 'antigravity',
  'gemini-cli': 'antigravity',
  opencode: 'opencode',
  'open-code': 'opencode',
}

const normalizeDashboardCliPlatform = (platform: string) => {
  return CLI_PLATFORM_ALIASES[platform.trim().toLowerCase()] ?? null
}

const applyCliVersions = (entries: CliVersionEntry[]) => {
  const normalizedEntries = new Map<string, CliVersionEntry>()

  for (const entry of entries) {
    const platformKey = normalizeDashboardCliPlatform(entry.platform)
    if (!platformKey) continue

    normalizedEntries.set(platformKey, {
      ...entry,
      platform: platformKey,
    })
  }

  cliVersions.value = normalizedEntries
  cliVersionsLoaded.value = true
  perfMark('dashboard:cli-badges-updated')
}

const loadSystemInfo = async () => {
  try {
    systemInfo.value = await getSystemInfo()
    systemInfoError.value = null
    perfMark('dashboard:system-ready')
  } catch (error) {
    systemInfoError.value = getErrorMessage(error)
    logger.error('[DashboardView] failed to load system info', error)
  }
}

const loadCliVersions = async () => {
  try {
    cliVersionsLoaded.value = false
    const versions: CliVersionsResponse = await getCliVersions({
      mode: 'fast',
      timeoutMs: 3500,
      parallelism: 4,
    })
    applyCliVersions(versions.entries)
  } catch (error) {
    cliVersionsLoaded.value = true
    logger.error('[DashboardView] failed to load CLI versions', error)
  }
}

const loadUsageOverview = async (days: number) => {
  try {
    await usageOverviewStore.loadOverview(days)
    perfMark('dashboard:usage-overview-ready')
  } catch (error) {
    logger.error('[DashboardView] failed to load usage overview', error)
  }
}

let cancelDashboardDeferredTasks: (() => void) | null = null

const logDashboardPerfSnapshot = () => {
  if (!shouldLogPerfTelemetry() || typeof performance === 'undefined') return

  window.setTimeout(() => {
    const resources = performance.getEntriesByType('resource') as PerformanceResourceTiming[]
    const relevant = resources
      .filter((entry) =>
        entry.name.includes('get_system_info')
        || entry.name.includes('get_cli_versions')
        || entry.name.includes('get_home_usage_overview_v2'),
      )
      .map((entry) => ({
        name: entry.name,
        responseEnd: Math.round(entry.responseEnd),
        duration: Math.round(entry.duration),
      }))

    const badgeMarks = performance.getEntriesByName('dashboard:cli-badges-updated')
    const usageMarks = performance.getEntriesByName('dashboard:usage-overview-ready')

    logger.info('[Perf]', {
      scope: 'dashboard',
      apiResponses: relevant,
      cliBadgeUpdatedAt: badgeMarks.length > 0 ? Math.round(badgeMarks[badgeMarks.length - 1].startTime) : null,
      usageOverviewReadyAt: usageMarks.length > 0 ? Math.round(usageMarks[usageMarks.length - 1].startTime) : null,
    })
  }, 4500)
}

onMounted(() => {
  cancelDashboardDeferredTasks = scheduleWhenIdle(() => {
    if (!isNativeRuntime) {
      perfMark('dashboard:web-preview-ready')
      cliVersionsLoaded.value = true
      return
    }

    void loadSystemInfo()
    void loadCliVersions()
    void loadUsageOverview(activeDays.value)
  }, { timeout: 1400, fallbackDelay: 280 })
  logDashboardPerfSnapshot()
})

onBeforeUnmount(() => {
  cancelDashboardDeferredTasks?.()
  cancelDashboardDeferredTasks = null
  void usageOverviewStore.teardown()
})

// 本视图被 keep-alive 缓存：onBeforeUnmount 在导航离开时不触发，改由 deactivated 暂停
// 监控事件源、activated 恢复，避免切走后仍在后台持续合并事件。
onDeactivated(() => {
  pauseFeed()
})

onActivated(() => {
  resumeFeed()
})

const backendStatus = computed<DashboardBackendStatus>(() => {
  if (!isNativeRuntime) return 'unsupported'
  if (systemInfoError.value) return 'error'
  if (systemInfo.value) return 'ok'
  return 'checking'
})

const heroBadgeTone = computed<'ok' | 'checking' | 'error'>(() => {
  if (backendStatus.value === 'ok') return 'ok'
  if (backendStatus.value === 'error') return 'error'
  return 'checking'
})

const heroBadgeLabelKey = computed(() => {
  switch (backendStatus.value) {
    case 'ok':
      return 'dashboard.metrics.backendReady'
    case 'error':
      return 'dashboard.metrics.backendError'
    case 'checking':
      return 'dashboard.metrics.backendChecking'
    case 'unsupported':
      return 'dashboard.metrics.backendUnsupported'
    default:
      return 'dashboard.metrics.backendUnknown'
  }
})

const scrollToReadiness = () => {
  const target = document.querySelector('[data-dashboard-readiness]')
  const reduceMotion = typeof window.matchMedia === 'function'
    && window.matchMedia('(prefers-reduced-motion: reduce)').matches

  target?.scrollIntoView({ behavior: reduceMotion ? 'auto' : 'smooth', block: 'center' })
}

const platforms = computed<DashboardPlatformSource[]>(() => [
  {
    title: t('dashboard.platforms.claudeTitle'),
    desc: t('dashboard.platforms.claudeDesc'),
    path: '/claude-code',
    icon: 'Code2',
    iconClass: 'text-platform-claude',
    platformKey: 'claude-code',
    usageKey: 'claude',
    role: t('dashboard.platforms.roleCoreCli'),
    mode: 'cli',
    isRuntimeCli: true,
  },
  {
    title: t('dashboard.platforms.codexTitle'),
    desc: t('dashboard.platforms.codexDesc'),
    path: '/codex',
    icon: 'Settings',
    iconClass: 'text-platform-codex',
    platformKey: 'codex',
    usageKey: 'codex',
    role: t('dashboard.platforms.roleCoreCli'),
    mode: 'cli',
    isRuntimeCli: true,
  },
  {
    title: t('dashboard.platforms.antigravityTitle'),
    desc: t('dashboard.platforms.antigravityDesc'),
    path: '/antigravity',
    icon: 'Sparkles',
    iconClass: 'text-platform-gemini',
    platformKey: 'antigravity',
    usageKey: 'gemini',
    role: t('dashboard.platforms.roleCoreCli'),
    mode: 'cli',
    isRuntimeCli: true,
  },
  {
    title: t('dashboard.platforms.opencodeTitle'),
    desc: t('dashboard.platforms.opencodeDesc'),
    path: '/opencode',
    icon: 'TerminalSquare',
    iconClass: 'text-accent-info',
    platformKey: 'opencode',
    usageKey: 'opencode',
    role: t('dashboard.platforms.roleManaged'),
    mode: 'managed',
    isRuntimeCli: false,
  },
])

const dashboardPresentation = computed(() => buildDashboardPresentation({
  backendStatus: backendStatus.value,
  isNativeRuntime,
  systemInfo: systemInfo.value,
  cliVersions: cliVersions.value,
  cliVersionsLoaded: cliVersionsLoaded.value,
  platforms: platforms.value,
  overview: overview.value,
  usageLoading: usageLoading.value,
  usageError: usageError.value,
  logs: logs.value,
}))
</script>

<style scoped>
.dashboard-view {
  min-height: 100%;
  color: var(--color-text-primary);
}

.dashboard-workbench {
  display: grid;
  gap: var(--home-section-gap);
  width: min(100%, 1440px);
  margin: 0 auto;
}

.dashboard-hero {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 1rem;
  padding: 1rem 0 0.15rem;
  color: var(--color-text-primary);
}

.dashboard-hero__lede {
  display: grid;
  gap: 0.35rem;
  min-width: 0;
  flex: 1 1 28rem;
}

.dashboard-hero__badge {
  display: inline-flex;
  flex-shrink: 0;
  align-items: center;
  gap: 0.45rem;
  padding: 0.4rem 0.85rem;
  border: 1px solid var(--home-border-card);
  border-radius: 999px;
  background: var(--home-surface-card);
  box-shadow: var(--home-elevation-sunk);
  color: var(--color-text-secondary);
  font-size: var(--home-text-meta);
  font-weight: 800;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
  cursor: pointer;
  transition:
    border-color var(--home-motion-duration) var(--home-motion-ease),
    background-color var(--home-motion-duration) var(--home-motion-ease);
}

.dashboard-hero__badge:hover {
  border-color: var(--home-border-card-hover);
  background: var(--home-surface-card-hover);
}

.dashboard-hero__badge:focus-visible {
  outline: 0;
  box-shadow: var(--home-focus-ring);
}

.dashboard-hero__badge-dot {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 999px;
  background: var(--color-text-muted);
}

.dashboard-hero__badge[data-tone='ok'] {
  color: var(--color-success);
}

.dashboard-hero__badge[data-tone='ok'] .dashboard-hero__badge-dot {
  background: var(--color-success);
}

.dashboard-hero__badge[data-tone='error'] {
  color: var(--color-danger);
}

.dashboard-hero__badge[data-tone='error'] .dashboard-hero__badge-dot {
  background: var(--color-danger);
}

.dashboard-hero__eyebrow {
  display: inline-flex;
  align-items: center;
  gap: 0.55rem;
  margin: 0;
  color: var(--color-text-muted);
  font-size: var(--home-text-meta);
  font-weight: 800;
  letter-spacing: var(--home-tracking-eyebrow);
  text-transform: uppercase;
}

.dashboard-hero__eyebrow-anchor {
  display: inline-block;
  width: 16px;
  height: 2px;
  border-radius: 2px;
  background: var(--color-accent-primary);
}

.dashboard-hero__title {
  max-width: 42rem;
  margin: 0;
  color: var(--color-text-primary);
  font-family: var(--font-brand);
  font-size: clamp(1.8rem, 3.8vw, 3.2rem);
  font-weight: 640;
  letter-spacing: -0.06em;
  line-height: 0.96;
}

.dashboard-hero__description {
  max-width: 58rem;
  margin: 0.2rem 0 0;
  color: var(--color-text-secondary);
  font-size: var(--home-text-body);
  letter-spacing: var(--home-tracking-body);
  line-height: var(--home-leading-body);
}

.dashboard-grid {
  display: grid;
  grid-template-columns: repeat(12, minmax(0, 1fr));
  gap: var(--home-section-gap-tight);
  align-items: stretch;
}

.dashboard-grid > * {
  min-width: 0;
}

.dashboard-grid__actions,
.dashboard-grid__usage {
  grid-column: span 8;
}

.dashboard-grid__readiness,
.dashboard-grid__signals {
  grid-column: span 4;
}

@media (width <= 1180px) {
  .dashboard-hero {
    flex-direction: column;
    align-items: flex-start;
  }

  .dashboard-grid__readiness,
  .dashboard-grid__usage,
  .dashboard-grid__actions,
  .dashboard-grid__signals {
    grid-column: 1 / -1;
  }
}

@media (width <= 960px) {
  .dashboard-workbench {
    gap: var(--home-section-gap-tight);
  }
}
</style>

