<template>
  <div class="home-view">
    <div class="home-shell">
      <PageHeaderCard
        :title="`${$t('home.welcomeBack')}，${$t('home.roleEngineer')}`"
        :description="$t('home.statusMsg')"
        badge="Neko Console"
        icon="Sparkles"
        tone="secondary"
      >
        <template #actions>
          <Button
            variant="primary"
            size="sm"
            @click="router.push('/commands')"
          >
            {{ $t('home.actionCommandRunner') }}
          </Button>
          <Button
            variant="secondary"
            size="sm"
            @click="router.push('/skills?tab=marketplace')"
          >
            {{ $t('nav.skills') }}
          </Button>
        </template>

        <div class="hero-metrics">
          <div class="hero-metric">
            <div class="hero-metric__icon hero-metric__icon--success">
              <div class="hero-metric__dot hero-metric__dot--success" />
            </div>
            <div class="hero-metric__body">
              <span class="hero-metric__label">{{ $t('home.cpuUsage') }}</span>
              <strong class="hero-metric__value">{{ systemInfo?.cpu_usage?.toFixed(1) || '0.0' }}%</strong>
            </div>
          </div>

          <div class="hero-metric">
            <div class="hero-metric__icon hero-metric__icon--info">
              <div class="hero-metric__dot hero-metric__dot--info" />
            </div>
            <div class="hero-metric__body">
              <span class="hero-metric__label">{{ $t('home.memoryUsage') }}</span>
              <strong class="hero-metric__value">{{ systemInfo?.memory_usage_percent?.toFixed(1) || '0.0' }}%</strong>
            </div>
          </div>

          <div class="hero-metric">
            <div class="hero-metric__icon hero-metric__icon--secondary">
              <SIcon
                name="Package"
                size="w-4 h-4"
              />
            </div>
            <div class="hero-metric__body">
              <span class="hero-metric__label">CLI footprint</span>
              <strong class="hero-metric__value">{{ installedCliCount }}/{{ mainModules.length }}</strong>
            </div>
          </div>
        </div>
      </PageHeaderCard>

      <section class="section-block">
        <div class="section-row">
          <div>
            <p class="section-kicker">
              {{ $t('home.quickActions') }}
            </p>
            <h2 class="section-title">
              Operate fast
            </h2>
          </div>
        </div>

        <div class="quick-actions-grid">
          <RouterLink
            v-for="action in quickActions"
            :key="action.path"
            :to="action.path"
            class="group"
          >
            <Card
              variant="elevated"
              hover
              class="quick-action-card"
            >
              <div
                class="quick-action-icon"
                :class="action.bgClass"
              >
                <SIcon
                  :name="action.icon"
                  size="w-5 h-5"
                  :class="action.textClass"
                />
              </div>
              <div class="quick-action-copy">
                <h3 class="quick-action-title">
                  {{ action.title }}
                </h3>
                <p class="quick-action-desc">
                  {{ action.desc }}
                </p>
              </div>
              <SIcon
                name="ArrowRight"
                size="w-4 h-4"
                class="quick-action-arrow"
              />
            </Card>
          </RouterLink>
        </div>
      </section>

      <section class="section-block">
        <div class="section-row">
          <div>
            <p class="section-kicker">
              {{ $t('home.platformModules') }}
            </p>
            <h2 class="section-title">
              Platform surfaces
            </h2>
          </div>
        </div>

        <div class="modules-grid">
          <RouterLink
            v-for="module in mainModules"
            :key="module.path"
            :to="module.path"
            class="group h-full"
          >
            <Card
              variant="elevated"
              hover
              class="module-card"
            >
              <div class="module-card__header">
                <div class="module-icon-shell">
                  <SIcon
                    :name="module.icon"
                    size="w-5 h-5"
                    :class="module.iconClass"
                  />
                </div>
                <div
                  class="module-version-badge"
                  :class="getVersionClass(module.platformKey)"
                >
                  {{ getVersionLabel(module.platformKey) }}
                </div>
              </div>

              <div class="module-copy">
                <h3 class="module-title">
                  {{ module.title }}
                </h3>
                <p class="module-desc">
                  {{ module.desc }}
                </p>
              </div>
            </Card>
          </RouterLink>
        </div>
      </section>

      <section
        ref="usageStatsSection"
        class="section-block"
      >
        <div class="section-row">
          <div>
            <p class="section-kicker">
              {{ $t('home.systemActivity') }}
            </p>
            <h2 class="section-title">
              Usage overview
            </h2>
          </div>
          <Button
            variant="ghost"
            size="sm"
            @click="router.push('/usage')"
          >
            {{ $t('home.fullReport') }}
          </Button>
        </div>

        <UsageStatsDashboard v-if="shouldRenderUsageStats" />
        <Card
          v-else
          variant="glass"
          class="usage-placeholder"
        >
          <div class="usage-placeholder__content">
            <div class="usage-placeholder__spinner" />
            <div>
              <p class="usage-placeholder__title">
                {{ $t('usageStats.title') }}
              </p>
              <p class="usage-placeholder__subtitle">
                {{ $t('common.loading') }}
              </p>
            </div>
          </div>
        </Card>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, onBeforeUnmount, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import PageHeaderCard from '@/components/PageHeaderCard.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { getCliVersions, getSystemInfo } from '@/api/runtime/system'
import { logger } from '@/utils/logger'
import { perfMark, shouldLogPerfTelemetry } from '@/utils/perfTelemetry'
import { scheduleWhenIdle } from '@/utils/scheduling'
import type { CliVersionEntry, CliVersionsResponse, SystemInfo } from '@/types'

const UsageStatsDashboard = defineAsyncComponent({
  loader: () => import('@/components/UsageStatsDashboard.vue'),
  suspensible: false,
})

const { t } = useI18n()
const router = useRouter()

const systemInfo = ref<SystemInfo | null>(null)
const cliVersions = ref<Map<string, CliVersionEntry>>(new Map())
const usageStatsSection = ref<HTMLElement | null>(null)
const shouldRenderUsageStats = ref(false)

const applyCliVersions = (entries: CliVersionEntry[]) => {
  for (const entry of entries) {
    cliVersions.value.set(entry.platform, entry)
  }
  perfMark('home:cli-badges-updated')
}

const loadSystemInfo = async () => {
  try {
    const sysInfo = await getSystemInfo<SystemInfo>().catch(() => null)
    systemInfo.value = sysInfo
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
    }).catch(() => null)
    if (versions) {
      applyCliVersions(versions.versions)
    }
  } catch (error) {
    logger.error('[HomeView] failed to load CLI versions', error)
  }
}

let cancelHomeDeferredTasks: (() => void) | null = null
let usageStatsObserver: IntersectionObserver | null = null
let usageStatsFallbackTimer: number | null = null

const revealUsageStats = () => {
  if (shouldRenderUsageStats.value) return

  shouldRenderUsageStats.value = true
  perfMark('home:usage-dashboard-revealed')

  if (usageStatsObserver) {
    usageStatsObserver.disconnect()
    usageStatsObserver = null
  }
  if (usageStatsFallbackTimer !== null) {
    window.clearTimeout(usageStatsFallbackTimer)
    usageStatsFallbackTimer = null
  }
}

const scheduleUsageStatsLoad = () => {
  if (typeof window === 'undefined') {
    revealUsageStats()
    return
  }

  if (typeof IntersectionObserver === 'function' && usageStatsSection.value) {
    usageStatsObserver = new IntersectionObserver((entries) => {
      if (entries.some((entry) => entry.isIntersecting)) {
        revealUsageStats()
      }
    }, { rootMargin: '240px 0px' })

    usageStatsObserver.observe(usageStatsSection.value)
  }

  usageStatsFallbackTimer = window.setTimeout(() => {
    revealUsageStats()
  }, 1800)
}

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
    const lastBadgeMark = badgeMarks.length > 0 ? Math.round(badgeMarks[badgeMarks.length - 1].startTime) : null

    logger.info('[Perf]', {
      scope: 'home',
      apiResponses: relevant,
      cliBadgeUpdatedAt: lastBadgeMark,
    })
  }, 4500)
}

onMounted(() => {
  cancelHomeDeferredTasks = scheduleWhenIdle(() => {
    void loadSystemInfo()
    void loadCliVersions()
    scheduleUsageStatsLoad()
  }, { timeout: 1400, fallbackDelay: 280 })
  logHomePerfSnapshot()
})

onBeforeUnmount(() => {
  cancelHomeDeferredTasks?.()
  cancelHomeDeferredTasks = null

  if (usageStatsObserver) {
    usageStatsObserver.disconnect()
    usageStatsObserver = null
  }
  if (usageStatsFallbackTimer !== null) {
    window.clearTimeout(usageStatsFallbackTimer)
    usageStatsFallbackTimer = null
  }
})

const getVersionLabel = (platformKey: string) => {
  const entry = cliVersions.value.get(platformKey)
  if (!entry) return '...'
  if (entry.status === 'timeout' || entry.status === 'error') return '...'
  if (entry.status === 'not_installed' || !entry.installed) return t('home.notInstalled')
  return entry.version ? `v${entry.version}` : t('common.installed')
}

const getVersionClass = (platformKey: string) => {
  const entry = cliVersions.value.get(platformKey)
  if (!entry) return 'module-version-badge--default'
  if (entry.status === 'timeout') return 'module-version-badge--warning'
  if (entry.status === 'error') return 'module-version-badge--danger'
  if (entry.status === 'not_installed' || !entry.installed) return 'module-version-badge--danger'
  return 'module-version-badge--default'
}

const quickActions = computed(() => [
  {
    title: t('home.actionCommandRunner'),
    desc: t('home.actionCommandRunnerDesc'),
    path: '/commands',
    icon: 'Terminal',
    bgClass: 'bg-accent-secondary/12',
    textClass: 'text-accent-secondary',
  },
  {
    title: t('home.actionConfigManager'),
    desc: t('home.actionConfigManagerDesc'),
    path: '/configs',
    icon: 'Settings',
    bgClass: 'bg-accent-primary/12',
    textClass: 'text-accent-primary',
  },
  {
    title: t('home.actionCloudSync'),
    desc: t('home.actionCloudSyncDesc'),
    path: '/sync',
    icon: 'Cloud',
    bgClass: 'bg-accent-info/12',
    textClass: 'text-accent-info',
  },
  {
    title: t('home.actionUsageStats'),
    desc: t('home.actionUsageStatsDesc'),
    path: '/usage',
    icon: 'Activity',
    bgClass: 'bg-accent-success/12',
    textClass: 'text-accent-success',
  },
])

const mainModules = computed(() => [
  {
    title: t('home.claudeCodeTitle'),
    desc: t('home.claudeCodeDesc'),
    path: '/claude-code',
    icon: 'Code2',
    iconClass: 'text-platform-claude',
    platformKey: 'claude-code',
  },
  {
    title: t('home.codexTitle'),
    desc: t('home.codexDesc'),
    path: '/codex',
    icon: 'Settings',
    iconClass: 'text-platform-codex',
    platformKey: 'codex',
  },
  {
    title: t('home.geminiTitle'),
    desc: t('home.geminiDesc'),
    path: '/gemini-cli',
    icon: 'Sparkles',
    iconClass: 'text-platform-gemini',
    platformKey: 'gemini-cli',
  },
  {
    title: t('home.qwenTitle'),
    desc: t('home.qwenDesc'),
    path: '/qwen',
    icon: 'Zap',
    iconClass: 'text-platform-qwen',
    platformKey: 'qwen',
  },
  {
    title: t('home.qoderTitle'),
    desc: t('home.qoderDesc'),
    path: '/qoder',
    icon: 'Workflow',
    iconClass: 'text-platform-qoder',
    platformKey: 'qoder',
  },
  {
    title: t('home.factoryDroidTitle'),
    desc: t('home.factoryDroidDesc'),
    path: '/droid',
    icon: 'Bot',
    iconClass: 'text-accent-secondary',
    platformKey: 'droid',
  },
])

const installedCliCount = computed(() => (
  mainModules.value.filter((module) => {
    const entry = cliVersions.value.get(module.platformKey)
    return Boolean(entry?.installed && entry.status !== 'error' && entry.status !== 'timeout')
  }).length
))
</script>

<style scoped>
.home-view {
  @apply relative min-h-full px-4 py-4 sm:px-6 sm:py-6;
}

.home-shell {
  @apply mx-auto flex max-w-[1440px] flex-col gap-6;
}

.section-block {
  @apply flex flex-col gap-4;
}

.section-row {
  @apply flex flex-wrap items-end justify-between gap-4;
}

.section-kicker {
  @apply text-xs font-semibold tracking-[0.14em] text-text-muted;
}

.section-title {
  @apply mt-1 text-xl font-semibold tracking-tight text-text-primary;
}

.hero-metrics {
  @apply grid gap-3 md:grid-cols-3;
}

.hero-metric {
  @apply flex items-center gap-3 rounded-2xl border border-border-default/60 px-4 py-3;

  background-color: rgb(var(--color-bg-elevated-rgb) / 70%);
  backdrop-filter: blur(14px);
}

.hero-metric__icon {
  @apply flex h-10 w-10 items-center justify-center rounded-xl border border-border-default/40;
}

.hero-metric__icon--success {
  @apply bg-accent-success/10 text-accent-success;
}

.hero-metric__icon--info {
  @apply bg-accent-info/10 text-accent-info;
}

.hero-metric__icon--secondary {
  @apply bg-accent-secondary/10 text-accent-secondary;
}

.hero-metric__dot {
  @apply h-2.5 w-2.5 rounded-full;
}

.hero-metric__dot--success {
  @apply bg-accent-success;
}

.hero-metric__dot--info {
  @apply bg-accent-info;
}

.hero-metric__body {
  @apply min-w-0 font-mono;
}

.hero-metric__label {
  @apply block text-[11px] uppercase tracking-[0.12em] text-text-muted;
}

.hero-metric__value {
  @apply mt-1 block text-lg font-semibold tracking-tight text-text-primary;

  font-variant-numeric: tabular-nums;
}

.quick-actions-grid {
  @apply grid gap-4 md:grid-cols-2 xl:grid-cols-4;
}

.quick-action-card {
  @apply flex h-full flex-col items-start gap-4 p-5;
}

.quick-action-icon {
  @apply flex h-11 w-11 items-center justify-center rounded-xl border border-border-default/40;
}

.quick-action-copy {
  @apply flex-1;
}

.quick-action-title {
  @apply mb-1 font-semibold text-text-primary;
}

.quick-action-desc {
  @apply text-sm leading-relaxed text-text-secondary;
}

.quick-action-arrow {
  @apply mt-auto text-text-muted transition-all duration-200 group-hover:translate-x-1 group-hover:text-accent-primary;
}

.modules-grid {
  @apply grid gap-4 md:grid-cols-2 xl:grid-cols-3;
}

.module-card {
  @apply flex h-full flex-col gap-5 p-5;
}

.module-card__header {
  @apply flex items-start justify-between gap-3;
}

.module-icon-shell {
  @apply flex h-11 w-11 items-center justify-center rounded-xl border border-border-default/40;

  background-color: rgb(var(--color-bg-elevated-rgb) / 75%);
  backdrop-filter: blur(12px);
}

.module-version-badge {
  @apply rounded-full border px-2.5 py-1 text-[10px] font-semibold tracking-[0.12em];
}

.module-version-badge--default {
  @apply border-border-default/60 text-text-secondary;

  background-color: rgb(var(--color-bg-elevated-rgb) / 72%);
}

.module-version-badge--warning {
  @apply border-accent-warning/30 bg-accent-warning/10 text-accent-warning;
}

.module-version-badge--danger {
  @apply border-accent-danger/30 bg-accent-danger/10 text-accent-danger;
}

.module-copy {
  @apply flex-1;
}

.module-title {
  @apply mb-2 text-lg font-semibold text-text-primary transition-colors group-hover:text-accent-primary;
}

.module-desc {
  @apply text-sm leading-relaxed text-text-secondary;
}

.usage-placeholder {
  @apply flex items-center justify-center p-6;

  min-height: 320px;
}

.usage-placeholder__content {
  @apply flex flex-col items-center gap-3 text-center;
}

.usage-placeholder__spinner {
  @apply h-8 w-8 animate-spin rounded-full border-2 border-accent-info/20 border-t-accent-info;
}

.usage-placeholder__title {
  @apply text-sm font-semibold text-text-primary;
}

.usage-placeholder__subtitle {
  @apply mt-1 text-xs text-text-muted;
}
</style>
