<template>
  <div class="home-view">
    <div class="home-shell">
      <section
        class="home-poster"
        data-home-hero
      >
        <div class="home-poster__mesh" />
        <div class="home-poster__copy">
          <div class="home-poster__eyebrow">
            <span class="home-poster__eyebrow-line" />
            {{ $t('home.posterEyebrow') }}
          </div>
          <div class="home-poster__title-wrap">
            <p class="home-poster__welcome">
              {{ `${$t('home.welcomeBack')}，${$t('home.roleEngineer')}` }}
            </p>
            <h1 class="home-poster__title">
              {{ $t('home.posterTitle') }}
            </h1>
          </div>
          <p class="home-poster__lead">
            {{ $t('home.posterLead') }}
          </p>
          <p class="home-poster__description">
            {{ $t('home.posterDescription') }}
          </p>

          <div class="home-poster__status-rail">
            <div
              v-for="item in posterStatusItems"
              :key="item.label"
              class="home-status-pill"
            >
              <span
                class="home-status-pill__dot"
                :class="item.toneClass"
              />
              <span class="home-status-pill__label">{{ item.label }}</span>
              <strong class="home-status-pill__value">{{ item.value }}</strong>
            </div>
          </div>

          <div class="home-poster__actions">
            <Button
              variant="primary"
              density="compact"
              surface="card"
              motion="standard"
              @click="router.push('/commands')"
            >
              {{ $t('home.actionCommandRunner') }}
            </Button>
            <Button
              variant="secondary"
              density="compact"
              surface="card"
              motion="subtle"
              @click="router.push('/skills?tab=marketplace')"
            >
              {{ $t('nav.skills') }}
            </Button>
          </div>
        </div>

        <div class="home-poster__visual">
          <div class="home-visual-panel">
            <div class="home-visual-panel__header">
              <div>
                <p class="home-visual-panel__eyebrow">
                  {{ $t('home.visualEyebrow') }}
                </p>
                <h2 class="home-visual-panel__title">
                  {{ $t('home.visualTitle') }}
                </h2>
              </div>
              <div class="home-visual-panel__badge">
                {{ installedCliCount }}/{{ mainModules.length }} {{ $t('home.visualBadge') }}
              </div>
            </div>

            <div class="home-visual-panel__fabric">
              <div class="home-visual-panel__grid" />
              <div class="home-visual-panel__signal-ring" />
              <div class="home-visual-panel__signal-ring home-visual-panel__signal-ring--secondary" />

              <div class="home-signal-list">
                <div
                  v-for="module in posterModules"
                  :key="module.path"
                  class="home-signal-node"
                >
                  <div class="home-signal-node__meta">
                    <SIcon
                      :name="module.icon"
                      size="w-4 h-4"
                      :class="module.iconClass"
                    />
                    <span>{{ module.title }}</span>
                  </div>
                  <div class="home-signal-node__status">
                    <span class="home-signal-node__version">{{ getVersionLabel(module.platformKey) }}</span>
                    <span
                      class="home-signal-node__state"
                      :class="getNodeStateClass(module.platformKey)"
                    />
                  </div>
                </div>
              </div>
            </div>

            <div
              class="home-usage-preview"
              data-home-usage-preview
            >
              <div class="home-usage-preview__copy">
                <p class="home-usage-preview__eyebrow">
                  {{ $t('home.usagePreviewEyebrow') }}
                </p>
                <p class="home-usage-preview__title">
                  {{ $t('home.usagePreviewTitle') }}
                </p>
              </div>
              <div class="home-usage-preview__bars">
                <span
                  v-for="(bar, index) in usagePreviewBars"
                  :key="`${bar}-${index}`"
                  class="home-usage-preview__bar"
                  :style="{ height: `${bar}%` }"
                />
              </div>
            </div>
          </div>
        </div>
      </section>

      <section
        class="section-block"
        data-home-actions
      >
        <div class="section-row">
          <div>
            <p class="section-kicker">
              {{ $t('home.actionsEyebrow') }}
            </p>
            <h2 class="section-title">
              {{ $t('home.actionsTitle') }}
            </h2>
            <p class="section-description">
              {{ $t('home.actionsDescription') }}
            </p>
          </div>
        </div>

        <div class="command-strip">
          <RouterLink
            v-for="action in quickActions"
            :key="action.path"
            :to="action.path"
            class="command-strip__item group"
          >
            <div class="command-strip__slot">
              <SIcon
                :name="action.icon"
                size="w-5 h-5"
                :class="action.textClass"
              />
              <div class="command-strip__copy">
                <h3 class="command-strip__title">
                  {{ action.title }}
                </h3>
                <p class="command-strip__desc">
                  {{ action.desc }}
                </p>
              </div>
              <SIcon
                name="ArrowRight"
                size="w-4 h-4"
                class="command-strip__arrow"
              />
            </div>
          </RouterLink>
        </div>
      </section>

      <section
        class="section-block"
        data-home-platforms
      >
        <div class="section-row">
          <div>
            <p class="section-kicker">
              {{ $t('home.platformsEyebrow') }}
            </p>
            <h2 class="section-title">
              {{ $t('home.platformsTitle') }}
            </h2>
            <p class="section-description">
              {{ $t('home.platformsDescription') }}
            </p>
          </div>
        </div>

        <div class="platform-matrix">
          <div class="platform-matrix__primary">
            <RouterLink
              v-for="module in featuredModules"
              :key="module.path"
              :to="module.path"
              class="platform-feature group"
            >
              <div class="platform-feature__shell">
                <div class="platform-feature__header">
                  <div class="platform-feature__brand">
                    <div class="platform-feature__icon">
                      <SIcon
                        :name="module.icon"
                        size="w-5 h-5"
                        :class="module.iconClass"
                      />
                    </div>
                    <div>
                      <p class="platform-feature__eyebrow">
                        {{ $t('home.platformFeatureLabel') }}
                      </p>
                      <h3 class="platform-feature__title">
                        {{ module.title }}
                      </h3>
                    </div>
                  </div>
                  <div
                    class="module-version-badge"
                    :class="getVersionClass(module.platformKey)"
                  >
                    {{ getVersionLabel(module.platformKey) }}
                  </div>
                </div>
                <p class="platform-feature__desc">
                  {{ module.desc }}
                </p>
                <div class="platform-feature__footer">
                  <span class="platform-feature__state">
                    {{ getModuleStateLabel(module.platformKey) }}
                  </span>
                  <SIcon
                    name="ArrowUpRight"
                    size="w-4 h-4"
                    class="platform-feature__arrow"
                  />
                </div>
              </div>
            </RouterLink>
          </div>

          <div class="platform-matrix__secondary">
            <RouterLink
              v-for="module in secondaryModules"
              :key="module.path"
              :to="module.path"
              class="platform-rail group"
            >
              <div class="platform-rail__shell">
                <div class="platform-rail__meta">
                  <SIcon
                    :name="module.icon"
                    size="w-4 h-4"
                    :class="module.iconClass"
                  />
                  <div>
                    <h3 class="platform-rail__title">
                      {{ module.title }}
                    </h3>
                    <p class="platform-rail__desc">
                      {{ module.desc }}
                    </p>
                  </div>
                </div>
                <div class="platform-rail__status">
                  <span
                    class="module-version-badge"
                    :class="getVersionClass(module.platformKey)"
                  >
                    {{ getVersionLabel(module.platformKey) }}
                  </span>
                  <SIcon
                    name="ArrowRight"
                    size="w-4 h-4"
                    class="platform-rail__arrow"
                  />
                </div>
              </div>
            </RouterLink>
          </div>
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
              {{ $t('home.usageSectionTitle') }}
            </h2>
            <p class="section-description">
              {{ $t('home.usageSectionDescription') }}
            </p>
          </div>
          <Button
            variant="ghost"
            density="compact"
            surface="status"
            motion="subtle"
            @click="router.push('/usage')"
          >
            {{ $t('home.fullReport') }}
          </Button>
        </div>

        <UsageStatsDashboard v-if="shouldRenderUsageStats" />
        <Card
          v-else
          surface="workspace"
          :elevation="2"
          motion="subtle"
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

const posterStatusItems = computed(() => [
  {
    label: t('home.statusReadyLabel'),
    value: t('common.ready'),
    toneClass: 'home-status-pill__dot--success',
  },
  {
    label: t('home.cpuUsage'),
    value: `${systemInfo.value?.cpu_usage?.toFixed(1) || '0.0'}%`,
    toneClass: 'home-status-pill__dot--info',
  },
  {
    label: t('home.memoryUsage'),
    value: `${systemInfo.value?.memory_usage_percent?.toFixed(1) || '0.0'}%`,
    toneClass: 'home-status-pill__dot--secondary',
  },
  {
    label: t('home.statusCliLabel'),
    value: `${installedCliCount.value}/${mainModules.value.length}`,
    toneClass: 'home-status-pill__dot--primary',
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

const featuredModules = computed(() => mainModules.value.slice(0, 2))
const secondaryModules = computed(() => mainModules.value.slice(2))
const posterModules = computed(() => mainModules.value.slice(0, 4))

const usagePreviewBars = computed(() => {
  const cpu = Math.round(systemInfo.value?.cpu_usage ?? 0)
  const memory = Math.round(systemInfo.value?.memory_usage_percent ?? 0)
  const baseBars = [28, 44, 58, 72, 52, 80, 64, 48]

  return baseBars.map((base, index) => {
    if (index === 1) return Math.max(18, Math.min(96, cpu + 16))
    if (index === 5) return Math.max(22, Math.min(96, memory + 10))
    return Math.max(16, Math.min(94, base + installedCliCount.value * 3 - index * 2))
  })
})

const getModuleStateLabel = (platformKey: string) => {
  const entry = cliVersions.value.get(platformKey)
  if (!entry) return t('common.loading')
  if (entry.status === 'timeout') return t('home.moduleStateScanning')
  if (entry.status === 'error' || entry.status === 'not_installed' || !entry.installed) {
    return t('home.moduleStateAttention')
  }
  return t('home.moduleStateReady')
}

const getNodeStateClass = (platformKey: string) => {
  const entry = cliVersions.value.get(platformKey)
  if (!entry) return 'home-signal-node__state--idle'
  if (entry.status === 'timeout') return 'home-signal-node__state--warning'
  if (entry.status === 'error' || entry.status === 'not_installed' || !entry.installed) {
    return 'home-signal-node__state--danger'
  }
  return 'home-signal-node__state--ready'
}
</script>

<style scoped>
.home-view {
  @apply relative min-h-full px-4 py-4 sm:px-6 sm:py-6;
}

.home-shell {
  @apply mx-auto flex max-w-[1480px] flex-col gap-8;
}

.home-poster {
  position: relative;
  overflow: hidden;
  border-radius: 2rem;
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 12%);
  background:
    radial-gradient(circle at top left, rgb(var(--color-accent-primary-rgb) / 16%), transparent 32%),
    radial-gradient(circle at bottom right, rgb(var(--color-accent-secondary-rgb) / 14%), transparent 30%),
    linear-gradient(135deg, rgb(var(--color-bg-elevated-rgb) / 96%), rgb(var(--color-bg-base-rgb) / 98%));
  box-shadow:
    0 24px 80px rgb(15 23 42 / 12%),
    inset 0 1px 0 rgb(255 255 255 / 14%);
  display: grid;
  gap: 2rem;
  padding: 1.5rem;
}

.home-poster__mesh {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgb(255 255 255 / 5%) 1px, transparent 1px),
    linear-gradient(90deg, rgb(255 255 255 / 5%) 1px, transparent 1px);
  background-size: 36px 36px;
  mask-image: radial-gradient(circle at center, black 40%, transparent 92%);
  opacity: 0.36;
  pointer-events: none;
}

.home-poster__copy,
.home-poster__visual {
  position: relative;
  z-index: 1;
}

.home-poster__copy {
  @apply flex flex-col gap-5;
}

.home-poster__eyebrow,
.home-visual-panel__eyebrow,
.home-usage-preview__eyebrow {
  @apply flex items-center gap-3 text-[11px] font-semibold uppercase tracking-[0.18em] text-text-muted;
}

.home-poster__eyebrow-line {
  @apply h-px w-10 bg-accent-primary/40;
}

.home-poster__welcome {
  @apply text-sm font-medium uppercase tracking-[0.16em] text-accent-primary;
}

.home-poster__title-wrap {
  @apply flex flex-col gap-2;
}

.home-poster__title {
  font-family: var(--font-brand, inherit);

  @apply max-w-[9ch] text-4xl font-semibold tracking-[-0.04em] text-text-primary sm:text-5xl lg:text-6xl;
}

.home-poster__lead {
  @apply max-w-[32rem] text-lg font-medium leading-8 text-text-primary;
}

.home-poster__description {
  @apply max-w-[34rem] text-sm leading-7 text-text-secondary;
}

.home-poster__status-rail {
  @apply flex flex-wrap gap-3;
}

.home-status-pill {
  @apply inline-flex items-center gap-2 rounded-full border border-border-default/55 bg-bg-elevated/70 px-3 py-2 backdrop-blur;
}

.home-status-pill__dot {
  @apply h-2.5 w-2.5 rounded-full;
}

.home-status-pill__dot--success,
.home-signal-node__state--ready {
  @apply bg-accent-success;
}

.home-status-pill__dot--info {
  @apply bg-accent-info;
}

.home-status-pill__dot--secondary {
  @apply bg-accent-secondary;
}

.home-status-pill__dot--primary {
  @apply bg-accent-primary;
}

.home-status-pill__label {
  @apply text-[11px] uppercase tracking-[0.14em] text-text-muted;
}

.home-status-pill__value {
  @apply text-sm font-semibold text-text-primary;
}

.home-poster__actions {
  @apply flex flex-wrap items-center gap-3;
}

.home-poster__visual {
  @apply flex items-stretch;
}

.home-visual-panel {
  @apply flex w-full flex-col gap-5 rounded-[1.75rem] border border-border-default/55 bg-bg-base/75 p-5 backdrop-blur-xl;

  box-shadow: inset 0 1px 0 rgb(255 255 255 / 10%);
}

.home-visual-panel__header {
  @apply flex items-start justify-between gap-4;
}

.home-visual-panel__title {
  @apply mt-1 text-xl font-semibold tracking-tight text-text-primary;
}

.home-visual-panel__badge {
  @apply rounded-full border border-accent-primary/25 bg-accent-primary/10 px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.14em] text-accent-primary;
}

.home-visual-panel__fabric {
  position: relative;
  overflow: hidden;
  border-radius: 1.5rem;
  min-height: 280px;
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 12%);
  background:
    linear-gradient(180deg, rgb(255 255 255 / 3%), transparent),
    linear-gradient(135deg, rgb(var(--color-bg-elevated-rgb) / 88%), rgb(var(--color-bg-base-rgb) / 90%));
}

.home-visual-panel__grid {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgb(255 255 255 / 4%) 1px, transparent 1px),
    linear-gradient(90deg, rgb(255 255 255 / 4%) 1px, transparent 1px);
  background-size: 30px 30px;
}

.home-visual-panel__signal-ring {
  position: absolute;
  inset: auto auto 1.25rem 1.5rem;
  width: 220px;
  height: 220px;
  border-radius: 9999px;
  border: 1px solid rgb(var(--color-accent-primary-rgb) / 24%);
  box-shadow: 0 0 90px rgb(var(--color-accent-primary-rgb) / 14%);
  animation: hero-pulse 9s ease-in-out infinite;
}

.home-visual-panel__signal-ring--secondary {
  inset: 2.25rem 2rem auto auto;
  width: 160px;
  height: 160px;
  border-color: rgb(var(--color-accent-secondary-rgb) / 22%);
  box-shadow: 0 0 70px rgb(var(--color-accent-secondary-rgb) / 12%);
  animation-delay: 1.4s;
}

.home-signal-list {
  position: relative;
  z-index: 1;

  @apply flex h-full flex-col justify-center gap-3 p-5;
}

.home-signal-node {
  @apply flex items-center justify-between gap-3 rounded-2xl border border-border-default/55 bg-bg-surface/70 px-4 py-3 backdrop-blur;

  transform: translateZ(0);
  transition: transform 220ms ease, border-color 220ms ease, background-color 220ms ease;
}

.home-signal-node:hover {
  transform: translateX(4px);
  border-color: rgb(var(--color-accent-primary-rgb) / 26%);
  background-color: rgb(var(--color-bg-elevated-rgb) / 82%);
}

.home-signal-node__meta {
  @apply flex items-center gap-3 text-sm font-medium text-text-primary;
}

.home-signal-node__status {
  @apply flex items-center gap-3;
}

.home-signal-node__version {
  @apply text-xs font-mono uppercase tracking-[0.12em] text-text-muted;
}

.home-signal-node__state {
  @apply h-2.5 w-2.5 rounded-full bg-text-muted/40;
}

.home-signal-node__state--idle {
  @apply bg-text-muted/40;
}

.home-signal-node__state--warning {
  @apply bg-accent-warning;
}

.home-signal-node__state--danger {
  @apply bg-accent-danger;
}

.home-usage-preview {
  @apply flex items-end justify-between gap-4 rounded-[1.4rem] border border-border-default/55 px-4 py-4 backdrop-blur;

  background-color: rgb(var(--color-bg-surface-rgb) / 72%);
}

.home-usage-preview__title {
  @apply mt-1 text-base font-semibold text-text-primary;
}

.home-usage-preview__bars {
  @apply flex min-w-[10rem] items-end gap-1.5;
}

.home-usage-preview__bar {
  @apply w-3 rounded-full bg-gradient-to-t from-accent-primary/35 via-accent-primary/60 to-accent-info/80;

  min-height: 14px;
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

.section-description {
  @apply mt-2 max-w-2xl text-sm leading-6 text-text-secondary;
}

.command-strip {
  @apply grid gap-3 lg:grid-cols-2;
}

.command-strip__item {
  @apply block;
}

.command-strip__slot {
  @apply flex h-full items-center gap-4 rounded-[1.45rem] border border-border-default/55 px-4 py-4 backdrop-blur transition-all duration-200;

  background-color: rgb(var(--color-bg-surface-rgb) / 72%);
}

.command-strip__slot:hover {
  border-color: rgb(var(--color-accent-primary-rgb) / 26%);
  transform: translateY(-2px);
}

.command-strip__copy {
  @apply min-w-0 flex-1;
}

.command-strip__title {
  @apply text-base font-semibold text-text-primary;
}

.command-strip__desc {
  @apply mt-1 text-sm leading-6 text-text-secondary;
}

.command-strip__arrow {
  @apply text-text-muted transition-transform duration-200 group-hover:translate-x-1 group-hover:text-accent-primary;
}

.platform-matrix {
  @apply grid gap-4 xl:grid-cols-[minmax(0,1.2fr)_minmax(0,0.9fr)];
}

.platform-matrix__primary,
.platform-matrix__secondary {
  @apply flex flex-col gap-4;
}

.platform-feature,
.platform-rail {
  @apply block;
}

.platform-feature__shell,
.platform-rail__shell {
  @apply rounded-[1.6rem] border border-border-default/55 p-5 backdrop-blur transition-all duration-200;

  background-color: rgb(var(--color-bg-surface-rgb) / 72%);
}

.platform-feature__shell:hover,
.platform-rail__shell:hover {
  border-color: rgb(var(--color-accent-primary-rgb) / 26%);
  transform: translateY(-2px);
}

.platform-feature__header,
.platform-feature__footer,
.platform-rail__shell,
.platform-rail__status {
  @apply flex items-center justify-between gap-4;
}

.platform-feature__brand,
.platform-rail__meta {
  @apply flex items-start gap-3;
}

.platform-feature__icon {
  @apply flex h-12 w-12 items-center justify-center rounded-2xl border border-border-default/55 bg-bg-elevated/80;
}

.platform-feature__eyebrow {
  @apply text-[11px] font-semibold uppercase tracking-[0.14em] text-text-muted;
}

.platform-feature__title,
.platform-rail__title {
  @apply mt-1 text-lg font-semibold tracking-tight text-text-primary;
}

.platform-feature__desc,
.platform-rail__desc {
  @apply mt-4 text-sm leading-7 text-text-secondary;
}

.platform-feature__state {
  @apply text-xs font-semibold uppercase tracking-[0.14em] text-accent-primary;
}

.platform-feature__arrow,
.platform-rail__arrow {
  @apply text-text-muted transition-transform duration-200 group-hover:translate-x-1 group-hover:text-accent-primary;
}

.platform-rail__meta {
  @apply min-w-0 flex-1;
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

@keyframes hero-pulse {
  0%,
  100% {
    transform: scale(1);
    opacity: 0.45;
  }

  50% {
    transform: scale(1.05);
    opacity: 0.85;
  }
}

@media (width >= 1024px) {
  .home-poster {
    grid-template-columns: minmax(0, 1.05fr) minmax(360px, 0.95fr);
    padding: 2rem;
  }
}

@media (width <= 767px) {
  .home-poster {
    border-radius: 1.5rem;
    padding: 1.25rem;
  }

  .home-poster__title {
    max-width: 12ch;
  }

  .home-visual-panel__fabric {
    min-height: 240px;
  }

  .home-usage-preview {
    @apply flex-col items-start;
  }
}
</style>
