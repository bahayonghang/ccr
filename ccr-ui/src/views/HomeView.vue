<template>
  <main class="home-view">
    <div class="home-workbench">
      <section class="home-section home-section--hero">
        <HomeEditorialHero />
      </section>

      <section class="home-section home-section--status">
        <HomeStatusBar
          :system-info="systemInfo"
          :installed-cli-count="installedCliCount"
          :runtime-cli-count="runtimeCliCount"
          :overview="overview"
          :usage-loading="usageLoading"
        />
      </section>

      <section class="home-section home-section--workbench">
        <HomeQuickActions :actions="quickActions" />
        <HomeActivityStream
          :entries="logs"
          :limit="6"
        />
      </section>

      <section class="home-section">
        <HomePlatformRegistry
          :platforms="platforms"
          :cli-versions="cliVersions"
          :overview="overview"
          :installed-cli-count="installedCliCount"
          :runtime-cli-count="runtimeCliCount"
        />
      </section>

      <section class="home-section">
        <HomeUsageSnapshot
          :overview="overview"
          :loading="usageLoading"
          :error="usageError"
          :active-days="activeDays"
          @change-days="loadUsageOverview"
        />
      </section>
    </div>
  </main>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { storeToRefs } from 'pinia'
import HomeActivityStream from '@/components/home/HomeActivityStream.vue'
import HomeEditorialHero from '@/components/home/HomeEditorialHero.vue'
import HomePlatformRegistry from '@/components/home/HomePlatformRegistry.vue'
import HomeQuickActions from '@/components/home/HomeQuickActions.vue'
import HomeStatusBar from '@/components/home/HomeStatusBar.vue'
import HomeUsageSnapshot from '@/components/home/HomeUsageSnapshot.vue'
import { getCliVersions, getSystemInfo } from '@/api/runtime/system'
import { useMonitoringFeed } from '@/composables/useMonitoringFeed'
import { useHomeUsageOverviewStore } from '@/stores/homeUsageOverview'
import { logger } from '@/utils/logger'
import { perfMark, shouldLogPerfTelemetry } from '@/utils/perfTelemetry'
import { scheduleWhenIdle } from '@/utils/scheduling'
import { isTauriRuntime } from '@/utils/tauriRuntime'
import type { CliVersionEntry, CliVersionsResponse, SystemInfo } from '@/types'
import type { HomePlatformRecord, HomeQuickAction } from '@/components/home/types'

const { t } = useI18n()
const usageOverviewStore = useHomeUsageOverviewStore()
const { overview, loading: usageLoading, error: usageError, activeDays } = storeToRefs(usageOverviewStore)
const { logs } = useMonitoringFeed({ initialCount: 6, maxEntries: 24 })

const systemInfo = ref<SystemInfo | null>(null)
const cliVersions = ref<Map<string, CliVersionEntry>>(new Map())

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

const normalizeHomeCliPlatform = (platform: string) => {
  return CLI_PLATFORM_ALIASES[platform.trim().toLowerCase()] ?? null
}

const applyCliVersions = (entries: CliVersionEntry[]) => {
  const normalizedEntries = new Map<string, CliVersionEntry>()

  for (const entry of entries) {
    const platformKey = normalizeHomeCliPlatform(entry.platform)
    if (!platformKey) continue

    normalizedEntries.set(platformKey, {
      ...entry,
      platform: platformKey,
    })
  }

  cliVersions.value = normalizedEntries
  perfMark('home:cli-badges-updated')
}

const loadSystemInfo = async () => {
  try {
    systemInfo.value = await getSystemInfo<SystemInfo>()
    perfMark('home:system-ready')
  } catch (error) {
    logger.error('[HomeView] failed to load system info', error)
  }
}

const loadCliVersions = async () => {
  try {
    const versions = await getCliVersions<CliVersionsResponse>({
      mode: 'fast',
      timeoutMs: 3500,
      parallelism: 4,
    })
    applyCliVersions(versions.versions ?? [])
  } catch (error) {
    logger.error('[HomeView] failed to load CLI versions', error)
  }
}

const loadUsageOverview = async (days: number) => {
  try {
    await usageOverviewStore.loadOverview(days)
    perfMark('home:usage-overview-ready')
  } catch (error) {
    logger.error('[HomeView] failed to load usage overview', error)
  }
}

let cancelHomeDeferredTasks: (() => void) | null = null

const logHomePerfSnapshot = () => {
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

    const badgeMarks = performance.getEntriesByName('home:cli-badges-updated')
    const usageMarks = performance.getEntriesByName('home:usage-overview-ready')

    logger.info('[Perf]', {
      scope: 'home',
      apiResponses: relevant,
      cliBadgeUpdatedAt: badgeMarks.length > 0 ? Math.round(badgeMarks[badgeMarks.length - 1].startTime) : null,
      usageOverviewReadyAt: usageMarks.length > 0 ? Math.round(usageMarks[usageMarks.length - 1].startTime) : null,
    })
  }, 4500)
}

onMounted(() => {
  cancelHomeDeferredTasks = scheduleWhenIdle(() => {
    if (!isTauriRuntime()) {
      perfMark('home:web-preview-ready')
      return
    }

    void loadSystemInfo()
    void loadCliVersions()
    void loadUsageOverview(activeDays.value)
  }, { timeout: 1400, fallbackDelay: 280 })
  logHomePerfSnapshot()
})

onBeforeUnmount(() => {
  cancelHomeDeferredTasks?.()
  cancelHomeDeferredTasks = null
  void usageOverviewStore.teardown()
})

const quickActions = computed<HomeQuickAction[]>(() => [
  {
    title: t('home.actionCommandRunner'),
    desc: t('home.actionCommandRunnerDesc'),
    path: '/commands',
    icon: 'Terminal',
    tone: 'command',
  },
  {
    title: t('home.actionConfigManager'),
    desc: t('home.actionConfigManagerDesc'),
    path: '/configs',
    icon: 'Settings',
    tone: 'config',
  },
  {
    title: t('home.actionCloudSync'),
    desc: t('home.actionCloudSyncDesc'),
    path: '/sync',
    icon: 'Cloud',
    tone: 'sync',
  },
  {
    title: t('home.actionUsageStats'),
    desc: t('home.actionUsageStatsDesc'),
    path: '/usage',
    icon: 'Activity',
    tone: 'usage',
  },
])

const platforms = computed<HomePlatformRecord[]>(() => [
  {
    title: t('home.claudeCodeTitle'),
    desc: t('home.claudeCodeDesc'),
    path: '/claude-code',
    icon: 'Code2',
    iconClass: 'text-platform-claude',
    platformKey: 'claude-code',
    usageKey: 'claude',
    role: t('home.platformRoleCoreCli'),
    mode: 'cli',
    isRuntimeCli: true,
  },
  {
    title: t('home.codexTitle'),
    desc: t('home.codexDesc'),
    path: '/codex',
    icon: 'Settings',
    iconClass: 'text-platform-codex',
    platformKey: 'codex',
    usageKey: 'codex',
    role: t('home.platformRoleCoreCli'),
    mode: 'cli',
    isRuntimeCli: true,
  },
  {
    title: t('home.geminiTitle'),
    desc: t('home.geminiDesc'),
    path: '/antigravity',
    icon: 'Sparkles',
    iconClass: 'text-platform-gemini',
    platformKey: 'antigravity',
    usageKey: 'gemini',
    role: t('home.platformRoleCoreCli'),
    mode: 'cli',
    isRuntimeCli: true,
  },
  {
    title: t('home.opencodeTitle'),
    desc: t('home.opencodeDesc'),
    path: '/opencode',
    icon: 'TerminalSquare',
    iconClass: 'text-accent-info',
    platformKey: 'opencode',
    usageKey: 'opencode',
    role: t('home.platformRoleManaged'),
    mode: 'managed',
    isRuntimeCli: false,
  },
])

const runtimeCliCount = computed(() => platforms.value.filter((platform) => platform.isRuntimeCli).length)

const installedCliCount = computed(() => (
  platforms.value.filter((platform) => {
    if (!platform.isRuntimeCli) return false
    const entry = cliVersions.value.get(platform.platformKey)
    return Boolean(entry?.installed && entry.status !== 'error' && entry.status !== 'timeout')
  }).length
))
</script>

<style scoped>
.home-view {
  min-height: 100%;
  color: var(--color-text-primary);
}

.home-workbench {
  display: grid;
  gap: var(--home-section-gap);
  width: min(100%, 1440px);
  margin: 0 auto;
}

.home-section {
  display: block;
  min-width: 0;
}

.home-section--workbench {
  display: grid;
  grid-template-columns: repeat(12, minmax(0, 1fr));
  gap: var(--home-section-gap-tight);
  align-items: stretch;
}

.home-section--workbench > * {
  min-width: 0;
}

.home-section--workbench > :nth-child(1) {
  grid-column: span 7;
}

.home-section--workbench > :nth-child(2) {
  grid-column: span 5;
}

@media (width <= 1080px) {
  .home-section--workbench > :nth-child(1),
  .home-section--workbench > :nth-child(2) {
    grid-column: 1 / -1;
  }
}

@media (width <= 960px) {
  .home-workbench {
    gap: var(--home-section-gap-tight);
  }
}
</style>
