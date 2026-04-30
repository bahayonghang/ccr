<template>
  <main class="home-view">
    <div class="home-workbench">
      <div class="home-workbench__top">
        <HomeSystemPulse
          :system-info="systemInfo"
          :installed-cli-count="installedCliCount"
          :runtime-cli-count="runtimeCliCount"
          :overview="overview"
          :usage-loading="usageLoading"
          :actions="quickActions"
        />
        <HomeActivityStream
          :entries="logs"
          :limit="6"
        />
      </div>

      <HomePlatformRegistry
        :platforms="platforms"
        :cli-versions="cliVersions"
        :overview="overview"
        :installed-cli-count="installedCliCount"
        :runtime-cli-count="runtimeCliCount"
      />

      <HomeUsageSnapshot
        :overview="overview"
        :loading="usageLoading"
        :error="usageError"
        :active-days="activeDays"
        @change-days="loadUsageOverview"
      />
    </div>
  </main>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { storeToRefs } from 'pinia'
import HomeActivityStream from '@/components/home/HomeActivityStream.vue'
import HomePlatformRegistry from '@/components/home/HomePlatformRegistry.vue'
import HomeSystemPulse from '@/components/home/HomeSystemPulse.vue'
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
  gemini: 'gemini-cli',
  'gemini-cli': 'gemini-cli',
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
    path: '/gemini-cli',
    icon: 'Sparkles',
    iconClass: 'text-platform-gemini',
    platformKey: 'gemini-cli',
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
    role: t('home.platformRoleManaged'),
    mode: 'managed',
    isRuntimeCli: false,
  },
  {
    title: t('home.factoryDroidTitle'),
    desc: t('home.factoryDroidDesc'),
    path: '/droid',
    icon: 'Bot',
    iconClass: 'text-accent-secondary',
    platformKey: 'droid',
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
  gap: 1rem;
  width: min(100%, 1440px);
  margin: 0 auto;
}

.home-workbench__top {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(20rem, 0.34fr);
  gap: 1rem;
  align-items: stretch;
}

@media (width <= 1180px) {
  .home-workbench__top {
    grid-template-columns: 1fr;
  }
}
</style>
