<!-- -->
<template>
  <div class="home-view">
    <div class="home-shell">
      <!-- HEADER SECTION -->
      <header class="home-section animate-slide-up">
        <div class="home-hero">
          <div class="hero-overlay hero-overlay--accent" />
          <div class="hero-overlay hero-overlay--shade" />

          <div class="hero-content">
            <div class="hero-copy">
              <span class="hero-badge">
                <span class="hero-badge__dot" />
                {{ $t('common.shell.tagline') }}
              </span>
              <div class="hero-copy__body">
                <h1 class="hero-title">
                  {{ $t('home.welcomeBack') }},
                  <span class="hero-title__accent">{{ $t('home.roleEngineer') }}</span>
                </h1>
                <p class="hero-description">
                  {{ $t('home.statusMsg') }}
                </p>
              </div>
            </div>

            <div class="hero-stats">
              <Card
                variant="glass"
                class="hero-metric"
              >
                <div class="flex items-center gap-3">
                  <div class="hero-metric__icon hero-metric__icon--success">
                    <div class="hero-metric__dot hero-metric__dot--success animate-pulse" />
                  </div>
                  <div class="hero-metric__body">
                    <span class="hero-metric__label">{{ $t('home.cpuUsage') }}</span>
                    <span class="hero-metric__value">{{ systemInfo?.cpu_usage?.toFixed(1) || '12.4' }}%</span>
                  </div>
                </div>
              </Card>
              <Card
                variant="glass"
                class="hero-metric"
              >
                <div class="flex items-center gap-3">
                  <div class="hero-metric__icon hero-metric__icon--info">
                    <div class="hero-metric__dot hero-metric__dot--info" />
                  </div>
                  <div class="hero-metric__body">
                    <span class="hero-metric__label">{{ $t('home.memoryUsage') }}</span>
                    <span class="hero-metric__value">{{ systemInfo?.memory_usage_percent?.toFixed(1) || '42.8' }}%</span>
                  </div>
                </div>
              </Card>
            </div>
          </div>
        </div>
      </header>

      <!-- QUICK ACTIONS GRID -->
      <section
        class="home-section animate-slide-up"
        style="animation-delay: 100ms"
      >
        <div class="section-heading">
          <SIcon
            name="Terminal"
            size="w-4 h-4"
            class="text-accent-primary"
          />
          <h2 class="section-eyebrow">
            {{ $t('home.quickActions') }}
          </h2>
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
              glow
              class="quick-action-card"
            >
              <div 
                class="quick-action-icon"
                :class="action.bgClass"
              >
                <SIcon
                  :name="action.icon"
                  size="w-5 h-5"
                  class="transition-transform group-hover:scale-110"
                  :class="action.textClass"
                />
              </div>
              <div>
                <h3 class="quick-action-title">
                  {{ action.title }}
                </h3>
                <p class="quick-action-desc">
                  {{ action.desc }}
                </p>
              </div>
              <SIcon
                name="ArrowRight"
                size="h-4 w-4"
                class="quick-action-arrow"
              />
            </Card>
          </RouterLink>
        </div>
      </section>

      <!-- MAIN MODULES -->
      <section
        class="home-section animate-slide-up"
        style="animation-delay: 200ms"
      >
        <div class="section-heading">
          <SIcon
            name="Grid"
            size="w-4 h-4"
            class="text-accent-secondary"
          />
          <h2 class="section-eyebrow">
            {{ $t('home.platformModules') }}
          </h2>
        </div>

        <div class="modules-grid">
          <RouterLink
            v-for="module in mainModules"
            :key="module.path"
            :to="module.path"
            class="group h-full"
          >
            <Card
              variant="glass"
              hover
              glow
              class="module-card"
            >
              <!-- Background Icon Watermark -->
              <SIcon
                :name="module.icon"
                size="w-32 h-32"
                class="module-watermark"
              />
              
              <div class="module-card__header">
                <div class="module-icon-shell">
                  <SIcon
                    :name="module.icon"
                    size="w-6 h-6"
                    :class="module.iconClass"
                  />
                </div>
                <div class="flex items-center gap-2">
                  <div
                    class="module-version-badge"
                    :class="getVersionClass(module.platformKey)"
                  >
                    {{ getVersionLabel(module.platformKey) }}
                  </div>
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

      <!-- STATS DASHBOARD -->
      <section
        ref="usageStatsSection"
        class="home-section animate-slide-up"
        style="animation-delay: 300ms"
      >
        <div class="section-header">
          <div class="section-heading">
            <SIcon
              name="Activity"
              size="w-4 h-4"
              class="text-accent-info"
            />
            <h2 class="section-eyebrow">
              {{ $t('home.systemActivity') }}
            </h2>
          </div>
          <Button
            variant="ghost"
            size="sm"
            @click="$router.push('/usage')"
          >
            {{ $t('home.fullReport') }} <SIcon
              name="ArrowRight"
              size="w-3 h-3"
              class="ml-1"
            />
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
import SIcon from '@/components/ui/SIcon.vue'
import { ref, computed, onMounted, onBeforeUnmount, defineAsyncComponent } from 'vue'
import { useI18n } from 'vue-i18n'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import { getSystemInfo, getCliVersions } from '@/api/runtime/system'
import { scheduleWhenIdle } from '@/utils/scheduling'
import { logger } from '@/utils/logger'
import type { CliVersionEntry, CliVersionsResponse, SystemInfo } from '@/types'

const UsageStatsDashboard = defineAsyncComponent({
  loader: () => import('@/components/UsageStatsDashboard.vue'),
  suspensible: false,
})

const { t } = useI18n()

const systemInfo = ref<SystemInfo | null>(null)
const cliVersions = ref<Map<string, CliVersionEntry>>(new Map())
const usageStatsSection = ref<HTMLElement | null>(null)
const shouldRenderUsageStats = ref(false)

const markPerf = (name: string) => {
  if (!import.meta.env.DEV || typeof performance === 'undefined') return
  performance.mark(name)
}

const applyCliVersions = (entries: CliVersionEntry[]) => {
  for (const entry of entries) {
    cliVersions.value.set(entry.platform, entry)
  }
  markPerf('home:cli-badges-updated')
}

const loadSystemInfo = async () => {
  try {
    const sysInfo = await getSystemInfo<SystemInfo>().catch(() => null)
    systemInfo.value = sysInfo
    markPerf('home:system-ready')
  } catch (e) {
    logger.error('[HomeView] failed to load system info', e)
  }
}

const loadCliVersions = async () => {
  try {
    const versions = await getCliVersions<CliVersionsResponse>({ mode: 'fast', timeoutMs: 3500, parallelism: 4 }).catch(() => null)
    if (versions) {
      applyCliVersions(versions.versions)
    }
  } catch (e) {
    logger.error('[HomeView] failed to load CLI versions', e)
  }
}

let cancelHomeDeferredTasks: (() => void) | null = null
let usageStatsObserver: IntersectionObserver | null = null
let usageStatsFallbackTimer: number | null = null

const revealUsageStats = () => {
  if (shouldRenderUsageStats.value) return

  shouldRenderUsageStats.value = true
  markPerf('home:usage-dashboard-revealed')

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
  if (!import.meta.env.DEV || typeof performance === 'undefined') return

  setTimeout(() => {
    const resources = performance.getEntriesByType('resource') as PerformanceResourceTiming[]
    const relevant = resources
      .filter((entry) =>
        entry.name.includes('get_system_info') ||
        entry.name.includes('get_cli_versions') ||
        entry.name.includes('get_home_usage_overview_v2')
      )
      .map((entry) => ({
        name: entry.name,
        responseEnd: Math.round(entry.responseEnd),
        duration: Math.round(entry.duration),
      }))

    const badgeMarks = performance.getEntriesByName('home:cli-badges-updated')
    const lastBadgeMark = badgeMarks.length > 0 ? Math.round(badgeMarks[badgeMarks.length - 1].startTime) : null

    logger.info('[HomePerf]', {
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
  if (!entry) return 'glass-surface border border-border-default/50 text-text-muted'
  if (entry.status === 'timeout') return 'bg-amber-500/10 text-amber-400'
  if (entry.status === 'error') return 'bg-orange-500/10 text-orange-400'
  if (entry.status === 'not_installed' || !entry.installed) return 'bg-red-500/10 text-red-400'
  return 'glass-surface border border-border-default/50 text-text-secondary'
}

const quickActions = computed(() => [
  { 
    title: t('home.actionCommandRunner'), 
    desc: t('home.actionCommandRunnerDesc'), 
    path: '/commands', 
    icon: 'Terminal', 
    bgClass: 'bg-blue-500/10',
    textClass: 'text-blue-500'
  },
  { 
    title: t('home.actionConfigManager'), 
    desc: t('home.actionConfigManagerDesc'), 
    path: '/configs', 
    icon: 'Settings', 
    bgClass: 'bg-purple-500/10',
    textClass: 'text-purple-500'
  },
  { 
    title: t('home.actionCloudSync'), 
    desc: t('home.actionCloudSyncDesc'), 
    path: '/sync', 
    icon: 'Cloud', 
    bgClass: 'bg-cyan-500/10',
    textClass: 'text-cyan-500'
  },
  { 
    title: t('home.actionUsageStats'), 
    desc: t('home.actionUsageStatsDesc'), 
    path: '/usage', 
    icon: 'Activity', 
    bgClass: 'bg-emerald-500/10',
    textClass: 'text-emerald-500'
  },
])

const mainModules = computed(() => [
  {
    title: t('home.claudeCodeTitle'),
    desc: t('home.claudeCodeDesc'),
    path: '/claude-code',
    icon: 'Code2',
    iconClass: 'text-platform-claude',
    platformKey: 'claude-code'
  },
  {
    title: t('home.codexTitle'),
    desc: t('home.codexDesc'),
    path: '/codex',
    icon: 'Settings',
    iconClass: 'text-platform-codex',
    platformKey: 'codex'
  },
  {
    title: t('home.geminiTitle'),
    desc: t('home.geminiDesc'),
    path: '/gemini-cli',
    icon: 'Sparkles',
    iconClass: 'text-platform-gemini',
    platformKey: 'gemini-cli'
  },
  {
    title: t('home.qwenTitle'),
    desc: t('home.qwenDesc'),
    path: '/qwen',
    icon: 'Zap',
    iconClass: 'text-platform-qwen',
    platformKey: 'qwen'
  },
  {
    title: t('home.qoderTitle'),
    desc: t('home.qoderDesc'),
    path: '/qoder',
    icon: 'Workflow',
    iconClass: 'text-platform-qoder',
    platformKey: 'qoder'
  },
  {
    title: t('home.factoryDroidTitle'),
    desc: t('home.factoryDroidDesc'),
    path: '/droid',
    icon: 'Bot',
    iconClass: 'text-accent-secondary',
    platformKey: 'droid'
  }
])
</script>

<style scoped>
.home-view {
  @apply relative min-h-full overflow-hidden p-6 lg:p-10;
}

.home-shell {
  @apply mx-auto max-w-7xl space-y-10;
}

.home-section {
  @apply relative;
}

.home-hero {
  @apply relative overflow-hidden border border-white/25 p-6 shadow-2xl shadow-slate-950/20 backdrop-blur-xl md:p-8;

  border-radius: 2rem;
  background: linear-gradient(145deg, rgb(16 18 36 / 74%), rgb(56 27 77 / 62%));
}

.hero-overlay {
  @apply absolute inset-0;
}

.hero-overlay--accent {
  background:
    radial-gradient(circle at top left, rgb(244 114 182 / 20%), transparent 42%),
    radial-gradient(circle at bottom right, rgb(168 85 247 / 16%), transparent 38%);
}

.hero-overlay--shade {
  background: linear-gradient(120deg, rgb(8 10 24 / 46%), transparent 55%);
}

.hero-content {
  @apply relative flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between;
}

.hero-copy {
  @apply max-w-3xl space-y-4;
}

.hero-copy__body {
  @apply space-y-2;
}

.hero-badge {
  @apply inline-flex items-center gap-2 rounded-full border border-white/15 px-3 py-1 font-semibold uppercase text-pink-100/90;

  background: rgb(255 255 255 / 8%);
  font-size: 11px;
  letter-spacing: 0.28em;
}

.hero-badge__dot {
  @apply h-2 w-2 rounded-full bg-accent-primary;

  box-shadow: 0 0 12px rgb(var(--color-accent-primary-rgb) / 75%);
}

.hero-title {
  @apply text-4xl font-bold tracking-tight text-white md:text-5xl;

  font-family: MapleBright, 'Microsoft YaHei UI', system-ui, sans-serif;
}

.hero-title__accent {
  @apply text-pink-100;
}

.hero-description {
  @apply max-w-2xl text-base leading-7 md:text-lg;

  color: rgb(241 245 249 / 88%);
}

.hero-stats {
  @apply grid gap-3 sm:grid-cols-2;
}

.hero-metric {
  @apply border-white/15 bg-white/10 px-4 py-3;

  min-height: 72px;
  min-width: 150px;
}

.hero-metric__icon {
  @apply flex h-10 w-10 items-center justify-center rounded-2xl;
}

.hero-metric__icon--success {
  @apply bg-accent-success/15 text-accent-success;
}

.hero-metric__icon--info {
  @apply bg-accent-info/15 text-accent-info;
}

.hero-metric__dot {
  @apply h-2.5 w-2.5 rounded-full;
}

.hero-metric__dot--success {
  @apply bg-accent-success;

  box-shadow: 0 0 12px rgb(var(--color-success-rgb) / 65%);
}

.hero-metric__dot--info {
  @apply bg-accent-info;

  box-shadow: 0 0 12px rgb(var(--color-info-rgb) / 65%);
}

.hero-metric__body {
  @apply font-mono text-xs;
}

.hero-metric__label {
  @apply block text-slate-200/70;
}

.hero-metric__value {
  @apply mt-1 block text-xl font-bold text-white;
}

.section-header {
  @apply mb-4 flex items-center justify-between;
}

.section-heading {
  @apply mb-4 flex items-center gap-2;
}

.section-eyebrow {
  @apply text-xs font-bold uppercase tracking-widest text-text-muted;
}

.quick-actions-grid {
  @apply grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4;
}

.quick-action-card {
  @apply flex h-full flex-col items-start gap-4 p-4 transition-colors;
}

.quick-action-icon {
  @apply flex h-10 w-10 items-center justify-center rounded-lg transition-colors duration-300;
}

.quick-action-title {
  @apply mb-1 font-bold text-text-primary;
}

.quick-action-desc {
  @apply line-clamp-2 text-xs leading-relaxed text-text-secondary;
}

.quick-action-arrow {
  @apply mt-auto self-end -translate-x-2 text-text-muted opacity-0 group-hover:translate-x-0 group-hover:opacity-100 group-hover:text-accent-primary;

  transition:
    opacity 200ms ease,
    transform 200ms ease,
    color 200ms ease;
}

.modules-grid {
  @apply grid grid-cols-1 gap-6 md:grid-cols-3;
}

.module-card {
  @apply relative flex h-full flex-col gap-6 overflow-hidden p-6;
}

.module-watermark {
  @apply absolute -bottom-6 -right-6 rotate-12 transition-opacity;

  opacity: 0.03;
}

.module-card__header {
  @apply z-10 flex items-start justify-between;
}

.module-icon-shell {
  @apply rounded-xl border border-border-default/50 bg-bg-elevated/70 p-3 backdrop-blur-md;
}

.module-version-badge {
  @apply rounded-md border border-border-default/50 px-2 py-1 font-bold uppercase;

  font-size: 10px;
}

.module-copy {
  @apply z-10;
}

.module-title {
  @apply mb-2 text-xl font-bold text-text-primary transition-colors group-hover:text-accent-primary;
}

.module-desc {
  @apply text-sm leading-relaxed text-text-secondary;
}

.usage-placeholder {
  @apply flex items-center justify-center p-6;

  min-height: 420px;
}

.group:hover .module-watermark {
  opacity: 0.07;
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
