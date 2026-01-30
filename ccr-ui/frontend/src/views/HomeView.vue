<template>
  <div class="min-h-full p-6 lg:p-10 relative overflow-hidden">
    <!-- Animated Background Mesh -->
    <div class="fixed inset-0 pointer-events-none -z-10 bg-bg-base">
      <div class="absolute top-[-10%] right-[-5%] w-[500px] h-[500px] bg-accent-primary/5 rounded-full blur-[120px] animate-pulse-subtle" />
      <div
        class="absolute bottom-[-10%] left-[-5%] w-[600px] h-[600px] bg-accent-secondary/5 rounded-full blur-[100px] animate-pulse-subtle"
        style="animation-delay: 2s"
      />
    </div>

    <div class="max-w-7xl mx-auto space-y-10">
      <!-- HEADER SECTION -->
      <header class="flex flex-col md:flex-row md:items-center justify-between gap-6 animate-slide-up">
        <div class="space-y-1">
          <h1 class="text-4xl font-bold font-display tracking-tight text-text-primary">
            {{ $t('home.welcomeBack') }}, <span class="bg-gradient-to-r from-accent-primary to-accent-secondary bg-clip-text text-transparent">{{ $t('home.roleEngineer') }}</span>
          </h1>
          <p class="text-text-secondary text-lg">
            {{ $t('home.statusMsg') }}
          </p>
        </div>
        
        <!-- System Stats (Mock or Real) -->
        <div class="flex items-center gap-3">
          <Card
            variant="glass"
            class="px-4 py-2 flex items-center gap-3 !rounded-full"
          >
            <div class="w-2 h-2 rounded-full bg-accent-success shadow-glow-success animate-pulse" />
            <div class="text-xs font-mono">
              <span class="text-text-muted">{{ $t('home.cpuUsage') }}</span>
              <span class="ml-2 text-text-primary font-bold">{{ systemInfo?.cpu_usage?.toFixed(1) || '12.4' }}%</span>
            </div>
          </Card>
          <Card
            variant="glass"
            class="px-4 py-2 flex items-center gap-3 !rounded-full"
          >
            <div class="w-2 h-2 rounded-full bg-accent-info shadow-glow-info" />
            <div class="text-xs font-mono">
              <span class="text-text-muted">{{ $t('home.memoryUsage') }}</span>
              <span class="ml-2 text-text-primary font-bold">{{ systemInfo?.memory_usage_percent?.toFixed(1) || '42.8' }}%</span>
            </div>
          </Card>
        </div>
      </header>

      <!-- QUICK ACTIONS GRID -->
      <section
        class="animate-slide-up"
        style="animation-delay: 100ms"
      >
        <div class="flex items-center gap-2 mb-4">
          <Terminal class="w-4 h-4 text-accent-primary" />
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
              class="h-full p-4 flex flex-col items-start gap-4 transition-all"
            >
              <div 
                class="w-10 h-10 rounded-lg flex items-center justify-center transition-colors duration-300"
                :class="action.bgClass"
              >
                <component
                  :is="action.icon"
                  class="w-5 h-5 transition-transform group-hover:scale-110"
                  :class="action.textClass"
                />
              </div>
              <div>
                <h3 class="font-bold text-text-primary mb-1">
                  {{ action.title }}
                </h3>
                <p class="text-xs text-text-muted leading-relaxed line-clamp-2">
                  {{ action.desc }}
                </p>
              </div>
              <ArrowRight class="w-4 h-4 text-text-muted mt-auto self-end opacity-0 group-hover:opacity-100 -translate-x-2 group-hover:translate-x-0 transition-all" />
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
          <Grid class="w-4 h-4 text-accent-secondary" />
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
              <component
                :is="module.icon"
                class="absolute -right-6 -bottom-6 w-32 h-32 opacity-[0.03] group-hover:opacity-[0.07] transition-opacity rotate-12"
              />
              
              <div class="flex justify-between items-start z-10">
                <div class="p-3 rounded-xl bg-bg-surface/50 border border-white/5 backdrop-blur-sm">
                  <component
                    :is="module.icon"
                    class="w-6 h-6"
                    :style="{ color: module.color }"
                  />
                </div>
                <div class="px-2 py-1 rounded text-[10px] font-bold uppercase bg-bg-surface border border-white/5 text-text-muted">
                  v{{ module.version || '1.0' }}
                </div>
              </div>

              <div class="z-10">
                <h3
                  class="text-xl font-bold text-text-primary mb-2 group-hover:text-transparent group-hover:bg-clip-text group-hover:bg-gradient-to-r"
                  :class="module.gradientClass"
                >
                  {{ module.title }}
                </h3>
                <p class="text-sm text-text-secondary leading-relaxed">
                  {{ module.desc }}
                </p>
              </div>

              <div class="z-10 mt-auto pt-4 border-t border-white/5 flex items-center gap-2">
                <div
                  class="w-2 h-2 rounded-full"
                  :style="{ background: module.color }"
                />
                <span class="text-xs font-mono text-text-muted">{{ $t('home.operational') }}</span>
              </div>
            </Card>
          </RouterLink>
        </div>
      </section>

      <!-- STATS DASHBOARD -->
      <section
        class="animate-slide-up"
        style="animation-delay: 300ms"
      >
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-2">
            <Activity class="w-4 h-4 text-accent-info" />
            <h2 class="text-xs font-bold uppercase tracking-widest text-text-muted">
              {{ $t('home.systemActivity') }}
            </h2>
          </div>
          <Button
            variant="ghost"
            size="sm"
            @click="$router.push('/usage')"
          >
            {{ $t('home.fullReport') }} <ArrowRight class="w-3 h-3 ml-1" />
          </Button>
        </div>
        <UsageStatsDashboard />
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  Terminal, ArrowRight, Grid, Activity, 
  Code2, Sparkles, Zap, Bot, Settings, Cloud,
  Workflow
} from 'lucide-vue-next'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import UsageStatsDashboard from '@/components/UsageStatsDashboard.vue'
import { getSystemInfo } from '@/api/client'

const { t } = useI18n()

const systemInfo = ref<any>(null)

onMounted(async () => {
  try {
    systemInfo.value = await getSystemInfo()
  } catch (e) {
    console.error(e)
  }
})

const quickActions = computed(() => [
  { 
    title: t('home.actionCommandRunner'), 
    desc: t('home.actionCommandRunnerDesc'), 
    path: '/commands', 
    icon: Terminal, 
    bgClass: 'bg-blue-500/10',
    textClass: 'text-blue-500'
  },
  { 
    title: t('home.actionConfigManager'), 
    desc: t('home.actionConfigManagerDesc'), 
    path: '/configs', 
    icon: Settings, 
    bgClass: 'bg-purple-500/10',
    textClass: 'text-purple-500'
  },
  { 
    title: t('home.actionCloudSync'), 
    desc: t('home.actionCloudSyncDesc'), 
    path: '/sync', 
    icon: Cloud, 
    bgClass: 'bg-cyan-500/10',
    textClass: 'text-cyan-500'
  },
  { 
    title: t('home.actionUsageStats'), 
    desc: t('home.actionUsageStatsDesc'), 
    path: '/usage', 
    icon: Activity, 
    bgClass: 'bg-emerald-500/10',
    textClass: 'text-emerald-500'
  },
])

const mainModules = computed(() => [
  {
    title: t('home.claudeCodeTitle'),
    desc: t('home.claudeCodeDesc'),
    path: '/claude-code',
    icon: Code2,
    color: '#ef4444',
    gradientClass: 'from-red-400 to-orange-400',
    version: '2.1'
  },
  {
    title: t('home.geminiTitle'),
    desc: t('home.geminiDesc'),
    path: '/gemini-cli',
    icon: Sparkles,
    color: '#3b82f6',
    gradientClass: 'from-blue-400 to-cyan-400',
    version: '1.5-pro'
  },
  {
    title: t('home.qwenTitle'),
    desc: t('home.qwenDesc'),
    path: '/qwen',
    icon: Zap,
    color: '#f59e0b',
    gradientClass: 'from-amber-400 to-yellow-300',
    version: '2.5'
  },
  {
    title: t('home.iflowTitle'),
    desc: t('home.iflowDesc'),
    path: '/iflow',
    icon: Workflow,
    color: '#8b5cf6',
    gradientClass: 'from-violet-400 to-fuchsia-400',
    version: '1.0'
  },
  {
    title: t('home.codexTitle'),
    desc: t('home.codexDesc'),
    path: '/codex',
    icon: Settings,
    color: '#10b981',
    gradientClass: 'from-emerald-400 to-green-300',
    version: '3.0'
  },
  {
    title: t('home.factoryDroidTitle'),
    desc: t('home.factoryDroidDesc'),
    path: '/droid',
    icon: Bot,
    color: '#ec4899',
    gradientClass: 'from-pink-400 to-rose-400',
    version: '0.9'
  }
])
</script>
