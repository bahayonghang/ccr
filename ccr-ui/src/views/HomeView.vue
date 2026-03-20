<!-- -->
<template>
  <div class="min-h-full p-6 lg:p-10 relative overflow-hidden">
    <div class="max-w-7xl mx-auto space-y-10">
      <!-- HEADER SECTION -->
      <header class="animate-slide-up">
        <div class="relative overflow-hidden rounded-[2rem] border border-white/25 bg-[linear-gradient(145deg,rgba(16,18,36,0.74),rgba(56,27,77,0.62))] p-6 shadow-2xl shadow-slate-950/20 backdrop-blur-xl md:p-8">
          <div class="absolute inset-0 bg-[radial-gradient(circle_at_top_left,rgba(244,114,182,0.2),transparent_42%),radial-gradient(circle_at_bottom_right,rgba(168,85,247,0.16),transparent_38%)]" />
          <div class="absolute inset-0 bg-[linear-gradient(120deg,rgba(8,10,24,0.46),transparent_55%)]" />

          <div class="relative flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between">
            <div class="max-w-3xl space-y-4">
              <span class="inline-flex items-center gap-2 rounded-full border border-white/15 bg-white/8 px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.28em] text-pink-100/90">
                <span class="h-2 w-2 rounded-full bg-accent-primary shadow-[0_0_12px_rgba(var(--color-accent-primary-rgb),0.75)]" />
                {{ $t('common.shell.tagline') }}
              </span>
              <div class="space-y-2">
                <h1 class="text-4xl font-bold font-display tracking-tight text-white md:text-5xl">
                  {{ $t('home.welcomeBack') }},
                  <span class="text-pink-100">{{ $t('home.roleEngineer') }}</span>
                </h1>
                <p class="max-w-2xl text-base leading-7 text-slate-100/88 md:text-lg">
                  {{ $t('home.statusMsg') }}
                </p>
              </div>
            </div>

            <div class="grid gap-3 sm:grid-cols-2">
              <Card
                variant="glass"
                class="min-h-[72px] min-w-[150px] border-white/15 bg-white/10 px-4 py-3"
              >
                <div class="flex items-center gap-3">
                  <div class="flex h-10 w-10 items-center justify-center rounded-2xl bg-accent-success/15 text-accent-success">
                    <div class="h-2.5 w-2.5 rounded-full bg-accent-success shadow-glow-success animate-pulse" />
                  </div>
                  <div class="text-xs font-mono">
                    <span class="block text-slate-200/70">{{ $t('home.cpuUsage') }}</span>
                    <span class="mt-1 block text-xl font-bold text-white">{{ systemInfo?.cpu_usage?.toFixed(1) || '12.4' }}%</span>
                  </div>
                </div>
              </Card>
              <Card
                variant="glass"
                class="min-h-[72px] min-w-[150px] border-white/15 bg-white/10 px-4 py-3"
              >
                <div class="flex items-center gap-3">
                  <div class="flex h-10 w-10 items-center justify-center rounded-2xl bg-accent-info/15 text-accent-info">
                    <div class="h-2.5 w-2.5 rounded-full bg-accent-info shadow-glow-info" />
                  </div>
                  <div class="text-xs font-mono">
                    <span class="block text-slate-200/70">{{ $t('home.memoryUsage') }}</span>
                    <span class="mt-1 block text-xl font-bold text-white">{{ systemInfo?.memory_usage_percent?.toFixed(1) || '42.8' }}%</span>
                  </div>
                </div>
              </Card>
            </div>
          </div>
        </div>
      </header>

      <!-- QUICK ACTIONS GRID -->
      <section
        class="animate-slide-up"
        style="animation-delay: 100ms"
      >
        <div class="flex items-center gap-2 mb-4">
          <SIcon
            name="Terminal"
            size="w-4 h-4"
            class="text-accent-primary"
          />
          <h2 class="text-xs font-bold uppercase tracking-widest text-text-muted">
            {{ $t('home.quickActions') }}
          </h2>
        </div>
        
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
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
              class="h-full p-4 flex flex-col items-start gap-4 transition-colors"
            >
              <div 
                class="w-10 h-10 rounded-lg flex items-center justify-center transition-colors duration-300"
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
                <h3 class="mb-1 font-bold text-text-primary">
                  {{ action.title }}
                </h3>
                <p class="line-clamp-2 text-xs leading-relaxed text-text-secondary">
                  {{ action.desc }}
                </p>
              </div>
              <SIcon
                name="ArrowRight"
                size="h-4 w-4"
                class="mt-auto self-end -translate-x-2 text-text-muted opacity-0 transition-[opacity,transform,color] group-hover:translate-x-0 group-hover:opacity-100 group-hover:text-accent-primary"
              />
            </Card>
          </RouterLink>
        </div>
      </section>

      <!-- MAIN MODULES -->
      <section
        class="animate-slide-up"
        style="animation-delay: 200ms"
      >
        <div class="flex items-center gap-2 mb-4">
          <SIcon
            name="Grid"
            size="w-4 h-4"
            class="text-accent-secondary"
          />
          <h2 class="text-xs font-bold uppercase tracking-widest text-text-muted">
            {{ $t('home.platformModules') }}
          </h2>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
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
              class="h-full relative overflow-hidden p-6 flex flex-col gap-6"
            >
              <!-- Background Icon Watermark -->
              <SIcon
                :name="module.icon"
                size="w-32 h-32"
                class="absolute -right-6 -bottom-6 opacity-[0.03] group-hover:opacity-[0.07] transition-opacity rotate-12"
              />
              
              <div class="flex justify-between items-start z-10">
                <div class="rounded-xl border border-border-default/50 bg-bg-elevated/70 p-3 backdrop-blur-md">
                  <SIcon
                    :name="module.icon"
                    size="w-6 h-6"
                    :class="module.iconClass"
                  />
                </div>
                <div class="flex items-center gap-2">
                  <div
                    class="rounded-md border border-border-default/50 px-2 py-1 text-[10px] font-bold uppercase"
                    :class="getVersionClass(module.platformKey)"
                  >
                    {{ getVersionLabel(module.platformKey) }}
                  </div>
                </div>
              </div>

              <div class="z-10">
                <h3 class="mb-2 text-xl font-bold text-text-primary transition-colors group-hover:text-accent-primary">
                  {{ module.title }}
                </h3>
                <p class="text-sm leading-relaxed text-text-secondary">
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
        class="animate-slide-up"
        style="animation-delay: 300ms"
      >
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-2">
            <SIcon
              name="Activity"
              size="w-4 h-4"
              class="text-accent-info"
            />
            <h2 class="text-xs font-bold uppercase tracking-widest text-text-muted">
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
          class="min-h-[420px] p-6 flex items-center justify-center"
        >
          <div class="flex flex-col items-center gap-3 text-center">
            <div class="h-8 w-8 rounded-full border-2 border-accent-info/20 border-t-accent-info animate-spin" />
            <div>
              <p class="text-sm font-semibold text-text-primary">
                {{ $t('usageStats.title') }}
              </p>
              <p class="mt-1 text-xs text-text-muted">
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
import { getSystemInfo, getCliVersions } from '@/api'
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
        entry.name.includes('get_daily_stats')
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
    title: t('home.iflowTitle'),
    desc: t('home.iflowDesc'),
    path: '/iflow',
    icon: 'Workflow',
    iconClass: 'text-platform-iflow',
    platformKey: 'iflow'
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
