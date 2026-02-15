<template>
  <div class="min-h-full p-6 lg:p-10 relative overflow-hidden">
    <!-- Background Mesh -->
    <!-- Standard Animated Background -->
    <AnimatedBackground variant="complex" />

    <div class="max-w-7xl mx-auto space-y-5">
      <!-- HEADER -->
      <section class="grid grid-cols-1 lg:grid-cols-3 gap-4 animate-slide-up">
        <!-- Hero Card -->
        <Card
          variant="glass"
          class="lg:col-span-2 relative overflow-hidden p-5 flex flex-col"
        >
          <div class="absolute top-0 right-0 w-48 h-48 bg-gradient-to-bl from-pink-500/10 to-transparent -mr-12 -mt-12 rounded-bl-full pointer-events-none" />
           
          <div class="relative z-10">
            <div class="flex items-center gap-3 mb-3">
              <div class="w-12 h-12 rounded-xl bg-pink-500/10 flex items-center justify-center border border-pink-500/20 shadow-lg backdrop-blur-md">
                <Code2 class="w-6 h-6 text-pink-500" />
              </div>
              <div>
                <h1 class="text-3xl font-bold font-display text-text-primary tracking-tight">
                  Codex
                </h1>
                <p class="text-text-secondary text-base max-w-md">
                  {{ $t('codex.overview.subtitle') }}
                </p>
              </div>
            </div>
             
            <div class="flex flex-wrap gap-2">
              <span class="px-3 py-1 rounded-full text-xs font-bold uppercase tracking-wider bg-pink-500/10 text-pink-500 border border-pink-500/20 flex items-center gap-2">
                <Server class="w-3 h-3" /> {{ $t('codex.overview.features.mcpProtocol') }}
              </span>
              <span class="px-3 py-1 rounded-full text-xs font-bold uppercase tracking-wider bg-accent-secondary/10 text-accent-secondary border border-accent-secondary/20">
                {{ codexVersion }}
              </span>
            </div>
          </div>
        </Card>

        <!-- Status Grid -->
        <div class="grid grid-cols-1 gap-3">
          <!-- Active Profile -->
          <Card
            variant="elevated"
            class="p-3 flex items-center gap-3 border-l-4 border-l-yellow-500"
          >
            <div class="w-10 h-10 rounded-lg bg-yellow-500/10 flex items-center justify-center text-yellow-500 shrink-0">
              <Zap class="w-5 h-5" />
            </div>
            <div class="min-w-0">
              <p class="text-xs font-bold text-text-muted uppercase tracking-wider mb-0.5">
                {{ $t('codex.status.currentConfig') }}
              </p>
              <p
                class="text-base font-bold text-text-primary truncate"
                :title="currentProfile || ''"
              >
                {{ currentProfile || $t('codex.status.notSet') }}
              </p>
            </div>
          </Card>

          <!-- Total Profiles -->
          <Card
            variant="elevated"
            class="p-3 flex items-center gap-3 border-l-4 border-l-blue-500"
          >
            <div class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center text-blue-500 shrink-0">
              <Settings class="w-5 h-5" />
            </div>
            <div>
              <p class="text-xs font-bold text-text-muted uppercase tracking-wider mb-0.5">
                {{ $t('codex.status.totalProfiles') }}
              </p>
              <p class="text-base font-bold text-text-primary">
                {{ profilesCount }}
              </p>
            </div>
          </Card>

          <!-- System Status -->
          <Card
            variant="elevated"
            class="p-3 flex items-center gap-3 border-l-4 border-l-emerald-500"
          >
            <div class="w-10 h-10 rounded-lg bg-emerald-500/10 flex items-center justify-center text-emerald-500 shrink-0">
              <Activity class="w-5 h-5" />
            </div>
            <div>
              <p class="text-xs font-bold text-text-muted uppercase tracking-wider mb-0.5">
                System Status
              </p>
              <p class="text-base font-bold text-text-primary">
                Online
              </p>
            </div>
          </Card>
        </div>
      </section>

      <!-- MODULES GRID -->
      <section
        class="animate-slide-up"
        style="animation-delay: 200ms"
      >
        <div class="flex items-center gap-3 mb-3">
          <Boxes class="w-5 h-5 text-pink-500" />
          <h2 class="text-lg font-bold uppercase tracking-widest text-text-muted">
            {{ $t('codex.overview.modulesTitle') }}
          </h2>
          <div class="h-px flex-1 bg-border-subtle" />
        </div>

        <div class="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-5 gap-3">
          <RouterLink 
            v-for="module in modules" 
            :key="module.path" 
            :to="module.path"
            class="group h-full"
          >
            <Card
              variant="glass"
              hover
              glow
              class="h-full p-4 flex flex-col relative overflow-hidden"
            >
              <div class="flex items-start justify-between mb-2">
                <div
                  class="w-10 h-10 rounded-lg flex items-center justify-center transition-transform duration-300 group-hover:scale-110 border border-white/5"
                  :class="module.bgClass"
                >
                  <component
                    :is="module.icon"
                    class="w-6 h-6"
                    :class="module.textClass"
                  />
                </div>
                <span
                  class="px-2 py-1 rounded text-[10px] font-bold uppercase tracking-wide border bg-bg-base/50"
                  :class="module.badgeBorderClass"
                >
                  {{ module.badge }}
                </span>
              </div>
                  
              <h3 class="text-base font-bold text-text-primary mb-1 group-hover:text-pink-500 transition-colors">
                {{ module.title }}
              </h3>
              <p class="text-sm text-text-secondary leading-relaxed flex-grow">
                {{ module.description }}
              </p>

              <div
                class="mt-2 flex items-center text-sm font-bold opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-300"
                :class="module.textClass"
              >
                Open Module <ArrowRight class="w-4 h-4 ml-1" />
              </div>
            </Card>
          </RouterLink>
        </div>
      </section>

      <!-- USAGE AND TIPS -->
      <section
        class="grid grid-cols-1 lg:grid-cols-2 gap-4 animate-slide-up"
        style="animation-delay: 300ms"
      >
        <!-- Usage Panel -->
        <Card
          variant="glass"
          class="p-4"
        >
          <div class="flex items-center justify-between mb-3">
            <div class="flex items-center gap-3">
              <div class="p-2 rounded-lg bg-pink-500/10 text-pink-500">
                <BarChart3 class="w-5 h-5" />
              </div>
              <h3 class="text-base font-bold text-text-primary">
                {{ $t('codex.overview.usageTitle') }}
              </h3>
            </div>
            <Button
              variant="ghost"
              size="icon"
              :disabled="usageLoading"
              @click="refreshUsage"
            >
              <RefreshCw
                class="w-4 h-4"
                :class="{ 'animate-spin': usageLoading }"
              />
            </Button>
          </div>

          <div
            v-if="usageLoading"
            class="space-y-4"
          >
            <div class="h-20 bg-bg-elevated animate-pulse rounded-xl" />
            <div class="h-20 bg-bg-elevated animate-pulse rounded-xl" />
          </div>

          <div
            v-else-if="usageError"
            class="text-center py-6"
          >
            <AlertCircle class="w-10 h-10 text-text-muted mx-auto mb-3" />
            <p class="text-sm text-text-muted mb-3">
              {{ $t('codex.overview.usageError') }}
            </p>
            <Button
              variant="outline"
              size="sm"
              @click="refreshUsage"
            >
              {{ $t('common.retry') }}
            </Button>
          </div>

          <div
            v-else-if="!usageData || usageData.all_time.total_requests === 0"
            class="text-center py-6"
          >
            <Clock class="w-10 h-10 text-text-muted mx-auto mb-3" />
            <p class="text-sm text-text-muted">
              {{ $t('codex.overview.noUsageData') }}
            </p>
          </div>

          <div
            v-else
            class="space-y-3"
          >
            <!-- 5H Usage -->
            <div class="p-3 rounded-xl bg-bg-surface/50 border border-border-subtle">
              <div class="flex justify-between items-center mb-2">
                <span class="text-xs font-bold text-text-muted uppercase tracking-wider">{{ $t('codex.overview.usage5h') }}</span>
                <span class="text-xs font-mono text-text-secondary">{{ usageData.five_hour.total_requests }} reqs</span>
              </div>
              <div class="flex items-baseline gap-2">
                <span class="text-xl font-bold text-text-primary font-mono">{{ formatTokens(usageData.five_hour.total_input_tokens + usageData.five_hour.total_output_tokens) }}</span>
                <span class="text-xs text-text-muted">tokens</span>
              </div>
              <div class="w-full bg-bg-base rounded-full h-1.5 mt-2 overflow-hidden">
                <div
                  class="bg-pink-500 h-full rounded-full"
                  style="width: 45%"
                /> <!-- Mock width for visual -->
              </div>
            </div>
               
            <!-- 7D Usage -->
            <div class="p-3 rounded-xl bg-bg-surface/50 border border-border-subtle">
              <div class="flex justify-between items-center mb-2">
                <span class="text-xs font-bold text-text-muted uppercase tracking-wider">{{ $t('codex.overview.usage7d') }}</span>
                <span class="text-xs font-mono text-text-secondary">{{ usageData.seven_day.total_requests }} reqs</span>
              </div>
              <div class="flex items-baseline gap-2">
                <span class="text-xl font-bold text-text-primary font-mono">{{ formatTokens(usageData.seven_day.total_input_tokens + usageData.seven_day.total_output_tokens) }}</span>
                <span class="text-xs text-text-muted">tokens</span>
              </div>
              <div class="w-full bg-bg-base rounded-full h-1.5 mt-2 overflow-hidden">
                <div
                  class="bg-purple-500 h-full rounded-full"
                  style="width: 75%"
                /> <!-- Mock width -->
              </div>
            </div>
          </div>
        </Card>

        <!-- Capabilities & Tips -->
        <div class="space-y-3">
          <Card
            variant="glass"
            class="p-4"
          >
            <div class="flex items-center gap-3 mb-4">
              <div class="p-2 rounded-lg bg-pink-500/10 text-pink-500">
                <Cpu class="w-5 h-5" />
              </div>
              <h3 class="text-base font-bold text-text-primary">
                System Capabilities
              </h3>
            </div>
            <div class="space-y-2">
              <div
                v-for="(feature, index) in features"
                :key="index"
                class="flex items-center gap-2 p-2 rounded-lg bg-bg-surface/30 border border-border-subtle/50"
              >
                <div class="w-2 h-2 rounded-full bg-pink-500 shadow-[0_0_8px_rgba(236,72,153,0.5)]" />
                <span class="text-sm text-text-secondary">{{ feature }}</span>
              </div>
            </div>
          </Card>

          <Card
            variant="outline"
            class="p-4 bg-amber-500/5 border-amber-500/20"
          >
            <div class="flex gap-3">
              <Lightbulb class="w-5 h-5 text-amber-500 shrink-0 mt-0.5" />
              <div class="space-y-1">
                <h4 class="text-sm font-bold text-amber-500">
                  {{ $t('codex.overview.tipsTitle') }}
                </h4>
                <p class="text-xs text-text-secondary leading-relaxed opacity-80">
                  {{ $t('codex.overview.tips.mcpConnection') }}
                </p>
              </div>
            </div>
          </Card>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  Settings, Settings2, Server, Command, Code2, Boxes,
  Zap, Activity, ArrowRight, Lightbulb, Cpu, KeyRound,
  BarChart3, RefreshCw, AlertCircle, Clock
} from 'lucide-vue-next'
// ...
// const router = useRouter() DELETE THIS LINE if it exists below in user code
// I cannot replace non-contiguous lines easily.
// I will just replace the top block.
// And I will assume `const router = ...` is further down.
// Wait, I need to see where `const router` is.


import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import { listCodexProfiles, getCodexUsage, getCliVersions } from '@/api'
import type { CodexUsageResponse } from '@/types'

const { t } = useI18n()

// State
const profilesCount = ref(0)
const currentProfile = ref<string | null>(null)
const codexVersion = ref('v1.0.0') // Default fallback

// Usage State
const usageData = ref<CodexUsageResponse | null>(null)
const usageLoading = ref(false)
const usageError = ref(false)

const formatTokens = (tokens: number): string => {
  if (tokens >= 1_000_000) {
    return `${(tokens / 1_000_000).toFixed(1)}M`
  } else if (tokens >= 1_000) {
    return `${(tokens / 1_000).toFixed(1)}K`
  }
  return tokens.toString()
}

const refreshUsage = async () => {
  usageLoading.value = true
  usageError.value = false
  try {
    usageData.value = await getCodexUsage()
  } catch (error) {
    console.error('Failed to load usage data:', error)
    usageError.value = true
  } finally {
    usageLoading.value = false
  }
}

const modules = computed(() => [
  {
    path: '/codex/profiles',
    title: t('codex.overview.modules.profiles.title'),
    description: t('codex.overview.modules.profiles.description'),
    badge: t('codex.overview.modules.profiles.badge'),
    icon: Settings,
    textClass: 'text-pink-500', 
    bgClass: 'bg-pink-500/10',
    badgeBorderClass: 'border-pink-500/20 text-pink-500'
  },
  {
    path: '/codex/mcp',
    title: t('codex.overview.modules.mcp.title'),
    description: t('codex.overview.modules.mcp.description'),
    badge: t('codex.overview.modules.mcp.badge'),
    icon: Server,
    textClass: 'text-indigo-500',
    bgClass: 'bg-indigo-500/10',
    badgeBorderClass: 'border-indigo-500/20 text-indigo-500'
  },
  {
    path: '/codex/slash-commands',
    title: t('codex.overview.modules.slashCommands.title'),
    description: t('codex.overview.modules.slashCommands.description'),
    badge: t('codex.overview.modules.slashCommands.badge'),
    icon: Command,
    textClass: 'text-rose-500',
    bgClass: 'bg-rose-500/10',
    badgeBorderClass: 'border-rose-500/20 text-rose-500'
  },
  {
    path: '/codex/auth',
    title: t('codex.overview.modules.auth.title'),
    description: t('codex.overview.modules.auth.description'),
    badge: t('codex.overview.modules.auth.badge'),
    icon: KeyRound,
    textClass: 'text-amber-500',
    bgClass: 'bg-amber-500/10',
    badgeBorderClass: 'border-amber-500/20 text-amber-500'
  },
  {
    path: '/codex/settings',
    title: t('codex.overview.modules.settings.title'),
    description: t('codex.overview.modules.settings.description'),
    badge: t('codex.overview.modules.settings.badge'),
    icon: Settings2,
    textClass: 'text-emerald-500',
    bgClass: 'bg-emerald-500/10',
    badgeBorderClass: 'border-emerald-500/20 text-emerald-500'
  }
])

const features = [
  'Advanced MCP Protocol Integration (Stdio & SSE)',
  'Multi-Profile Configuration Management',
  'Custom Slash Commands Support',
  'Context-Aware AI Coding Assistance'
]

onMounted(async () => {
  try {
    const data = await listCodexProfiles()
    if (data.profiles) {
      profilesCount.value = data.profiles.length
      currentProfile.value = data.current_profile ?? null
    }
    
    // Fetch Codex version
    const versions = await getCliVersions()
    const codex = versions.versions.find(v => v.platform === 'codex')
    if (codex && codex.version) {
      codexVersion.value = codex.version
    }
  } catch (error) {
    console.error('Failed to load profile status or version:', error)
  }
  refreshUsage()
})
</script>
