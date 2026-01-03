<template>
  <div class="min-h-screen p-6 transition-colors duration-300">
    <!-- 🎨 动态背景装饰 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-0 right-0 w-[600px] h-[600px] rounded-full opacity-10 blur-3xl"
        :style="{ background: 'radial-gradient(circle, var(--accent-primary) 0%, transparent 70%)' }"
      />
      <div
        class="absolute bottom-0 left-0 w-[500px] h-[500px] rounded-full opacity-10 blur-3xl"
        :style="{ background: 'radial-gradient(circle, var(--accent-secondary) 0%, transparent 70%)' }"
      />
    </div>

    <div class="max-w-[1800px] mx-auto space-y-6">
      <!-- 🌟 头部区域 - 紧凑型 -->
      <header class="flex flex-col lg:flex-row lg:items-center justify-between gap-6 animate-fade-in">
        <div class="flex items-center gap-5">
          <div class="relative group">
            <div class="absolute inset-0 bg-guofeng-jade/20 blur-lg rounded-full group-hover:bg-guofeng-jade/30 transition-all duration-500" />
            <div class="relative w-16 h-16 rounded-2xl glass-effect flex items-center justify-center border border-white/20 shadow-lg group-hover:scale-105 transition-transform duration-300">
              <img
                src="@/assets/logo.png"
                alt="CCR Logo"
                class="w-10 h-10 object-contain drop-shadow-md"
              >
            </div>
          </div>
          <div>
            <h1 class="text-3xl font-bold brand-gradient-text tracking-tight mb-1">
              {{ $t('home.title') }}
            </h1>
            <div class="flex items-center gap-3 text-sm text-guofeng-text-secondary">
              <p>{{ $t('home.subtitle') }}</p>
            </div>
          </div>
        </div>

        <!-- 系统状态小组件 -->
        <div
          v-if="systemInfo"
          class="flex items-center gap-3 bg-guofeng-bg-secondary/40 p-2 rounded-2xl border border-white/10 backdrop-blur-md shadow-sm"
        >
          <div class="flex items-center gap-3 px-4 py-2 rounded-xl bg-white/50 dark:bg-white/5 border border-white/20">
            <Cpu class="w-4 h-4 text-guofeng-jade" />
            <div class="flex flex-col">
              <span class="text-[10px] uppercase text-guofeng-text-muted font-bold">CPU</span>
              <span class="text-sm font-bold tabular-nums text-guofeng-text-primary">{{ systemInfo.cpu_usage?.toFixed(1) || '0.0' }}%</span>
            </div>
          </div>
          <div class="flex items-center gap-3 px-4 py-2 rounded-xl bg-white/50 dark:bg-white/5 border border-white/20">
            <HardDrive class="w-4 h-4 text-guofeng-indigo" />
            <div class="flex flex-col">
              <span class="text-[10px] uppercase text-guofeng-text-muted font-bold">MEM</span>
              <span class="text-sm font-bold tabular-nums text-guofeng-text-primary">{{ systemInfo.memory_usage_percent?.toFixed(1) || '0.0' }}%</span>
            </div>
          </div>
          <div class="hidden sm:flex items-center gap-3 px-4 py-2 rounded-xl bg-white/50 dark:bg-white/5 border border-white/20">
            <Activity class="w-4 h-4 text-guofeng-info" />
            <div class="flex flex-col">
              <span class="text-[10px] uppercase text-guofeng-text-muted font-bold">OS</span>
              <span
                class="text-sm font-bold text-guofeng-text-primary max-w-[100px] truncate"
                :title="systemInfo.os"
              >{{ systemInfo.os }}</span>
            </div>
          </div>
        </div>
      </header>

      <!-- ⚡ 快速操作 (Toolkit) -->
      <section
        class="animate-fade-in"
        style="animation-delay: 0.1s"
      >
        <div class="flex items-center gap-2 mb-4 px-1">
          <Zap class="w-5 h-5 text-guofeng-gold" />
          <h2 class="text-lg font-bold text-guofeng-text-primary">
            {{ $t('home.quickActions') }}
          </h2>
        </div>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-5">
          <RouterLink
            v-for="tool in configTools"
            :key="tool.href"
            :to="tool.href"
            class="block h-full"
          >
            <GuofengCard
              variant="glass"
              interactive
              class="h-full flex flex-col relative overflow-hidden group"
            >
              <div class="absolute top-0 right-0 p-3 opacity-5 group-hover:opacity-10 transition-opacity transform group-hover:scale-110 duration-500">
                <component
                  :is="tool.icon"
                  class="w-20 h-20"
                  :style="{ color: tool.color }"
                />
              </div>
              
              <div class="relative z-10 flex items-start gap-4">
                <div 
                  class="w-12 h-12 rounded-xl flex items-center justify-center shrink-0 transition-transform duration-300 group-hover:scale-110 shadow-sm"
                  :style="{ background: `${tool.color}15` }"
                >
                  <component
                    :is="tool.icon"
                    class="w-6 h-6"
                    :style="{ color: tool.color }"
                  />
                </div>
                <div>
                  <h3 class="text-base font-bold text-guofeng-text-primary mb-1 group-hover:text-guofeng-jade transition-colors">
                    {{ $t(tool.titleKey) }}
                  </h3>
                  <p class="text-xs text-guofeng-text-secondary leading-relaxed line-clamp-2">
                    {{ $t(tool.descriptionKey) }}
                  </p>
                </div>
              </div>
              
              <div class="mt-auto pt-4 flex items-center justify-between">
                <span 
                  class="text-[10px] font-bold px-2 py-0.5 rounded-md bg-guofeng-bg-tertiary text-guofeng-text-muted group-hover:bg-white group-hover:text-guofeng-jade transition-colors"
                >
                  {{ $t(tool.statsKey || 'common.open') }}
                </span>
                <ArrowRight class="w-4 h-4 text-guofeng-text-muted opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-300" />
              </div>
            </GuofengCard>
          </RouterLink>
        </div>
      </section>

      <!-- 🤖 AI 终端 (Core Modules) -->
      <section
        class="animate-fade-in"
        style="animation-delay: 0.2s"
      >
        <div class="flex items-center gap-2 mb-4 px-1">
          <Code2 class="w-5 h-5 text-guofeng-jade" />
          <h2 class="text-lg font-bold text-guofeng-text-primary">
            {{ $t('home.aiCliTools') }}
          </h2>
        </div>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-5">
          <RouterLink
            v-for="(tool, index) in cliTools"
            :key="tool.href"
            :to="tool.href"
            class="block h-full"
            :style="{ animationDelay: `${index * 0.05}s` }"
          >
            <GuofengCard
              variant="glass"
              interactive
              pattern
              class="h-full flex flex-col group"
            >
              <div class="relative z-10 flex flex-col h-full">
                <div class="flex justify-between items-start mb-4">
                  <div
                    class="w-14 h-14 rounded-2xl flex items-center justify-center shadow-sm ring-1 ring-white/50 dark:ring-white/10 backdrop-blur-md transition-transform duration-300 group-hover:scale-110"
                    :style="{ background: `linear-gradient(135deg, ${tool.color}10, ${tool.colorTo || tool.color}20)` }"
                  >
                    <component
                      :is="tool.icon"
                      class="w-7 h-7"
                      :style="{ color: tool.color }"
                    />
                  </div>
                  <div class="opacity-0 group-hover:opacity-100 transition-opacity duration-300 transform translate-x-2 group-hover:translate-x-0">
                    <div class="p-1.5 rounded-full bg-white/50 dark:bg-white/10 hover:bg-guofeng-jade hover:text-white transition-colors text-guofeng-text-muted">
                      <ArrowRight class="w-4 h-4" />
                    </div>
                  </div>
                </div>

                <h3
                  class="text-lg font-bold mb-2 text-guofeng-text-primary group-hover:text-transparent group-hover:bg-gradient-to-r group-hover:bg-clip-text transition-all"
                  :style="{ 
                    '--tw-gradient-from': tool.color,
                    '--tw-gradient-to': tool.colorTo || tool.color
                  }"
                >
                  {{ $t(tool.titleKey) }}
                </h3>

                <p class="text-sm text-guofeng-text-secondary leading-relaxed line-clamp-2 mb-4 flex-grow">
                  {{ $t(tool.descriptionKey) }}
                </p>

                <div class="pt-3 border-t border-dashed border-guofeng-border/50 flex items-center gap-2">
                  <span
                    class="w-2 h-2 rounded-full animate-pulse"
                    :style="{ background: tool.color }"
                  />
                  <span class="text-xs font-medium text-guofeng-text-muted">
                    {{ $t(tool.statsKey || 'common.ready') }}
                  </span>
                </div>
              </div>
            </GuofengCard>
          </RouterLink>
        </div>
      </section>

      <!-- 📊 使用统计仪表板 -->
      <section
        class="animate-fade-in"
        style="animation-delay: 0.25s"
      >
        <UsageStatsDashboard />
      </section>

      <!-- 📊 统计概览 -->
      <section
        class="animate-fade-in grid grid-cols-1 md:grid-cols-3 gap-5"
        style="animation-delay: 0.3s"
      >
        <GuofengCard
          variant="glass"
          class="relative overflow-hidden group"
        >
          <div class="absolute top-0 right-0 w-32 h-32 bg-guofeng-jade/10 rounded-full blur-2xl -mr-8 -mt-8 transition-all group-hover:bg-guofeng-jade/20" />
          <div class="relative z-10">
            <div class="flex items-center justify-between mb-4">
              <div class="w-12 h-12 rounded-xl bg-guofeng-jade/10 flex items-center justify-center">
                <FileText class="w-6 h-6 text-guofeng-jade" />
              </div>
              <TrendingUp class="w-5 h-5 text-guofeng-jade/50" />
            </div>
            <h3 class="text-2xl font-bold text-guofeng-text-primary mb-1">
              {{ $t('home.configsCount') }}
            </h3>
            <p class="text-sm text-guofeng-text-secondary">
              {{ $t('home.totalConfigurations') }}
            </p>
            <RouterLink
              to="/configs"
              class="mt-4 inline-flex items-center text-sm font-medium text-guofeng-jade hover:underline"
            >
              {{ $t('common.viewDetails') }}
              <ChevronRight class="w-4 h-4 ml-1" />
            </RouterLink>
          </div>
        </GuofengCard>

        <GuofengCard
          variant="glass"
          class="relative overflow-hidden group"
        >
          <div class="absolute top-0 right-0 w-32 h-32 bg-guofeng-indigo/10 rounded-full blur-2xl -mr-8 -mt-8 transition-all group-hover:bg-guofeng-indigo/20" />
          <div class="relative z-10">
            <div class="flex items-center justify-between mb-4">
              <div class="w-12 h-12 rounded-xl bg-guofeng-indigo/10 flex items-center justify-center">
                <Server class="w-6 h-6 text-guofeng-indigo" />
              </div>
              <TrendingUp class="w-5 h-5 text-guofeng-indigo/50" />
            </div>
            <h3 class="text-2xl font-bold text-guofeng-text-primary mb-1">
              {{ $t('home.mcpServers') }}
            </h3>
            <p class="text-sm text-guofeng-text-secondary">
              {{ $t('home.mcpServersDesc') }}
            </p>
            <RouterLink
              to="/claude-code"
              class="mt-4 inline-flex items-center text-sm font-medium text-guofeng-indigo hover:underline"
            >
              {{ $t('common.viewDetails') }}
              <ChevronRight class="w-4 h-4 ml-1" />
            </RouterLink>
          </div>
        </GuofengCard>

        <GuofengCard
          variant="glass"
          class="relative overflow-hidden group"
        >
          <div class="absolute top-0 right-0 w-32 h-32 bg-amber-500/10 rounded-full blur-2xl -mr-8 -mt-8 transition-all group-hover:bg-amber-500/20" />
          <div class="relative z-10">
            <div class="flex items-center justify-between mb-4">
              <div class="w-12 h-12 rounded-xl bg-amber-500/10 flex items-center justify-center">
                <Database class="w-6 h-6 text-amber-500" />
              </div>
              <TrendingUp class="w-5 h-5 text-amber-500/50" />
            </div>
            <h3 class="text-2xl font-bold text-guofeng-text-primary mb-1">
              {{ $t('home.backupStatus') }}
            </h3>
            <p class="text-sm text-guofeng-text-secondary">
              {{ $t('home.lastBackup') }}
            </p>
            <RouterLink
              to="/sync"
              class="mt-4 inline-flex items-center text-sm font-medium text-amber-500 hover:underline"
            >
              {{ $t('common.viewDetails') }}
              <ChevronRight class="w-4 h-4 ml-1" />
            </RouterLink>
          </div>
        </GuofengCard>
      </section>

      <!-- 💡 快速提示 -->
      <section
        class="animate-fade-in"
        style="animation-delay: 0.4s"
      >
        <GuofengCard
          variant="glass"
          class="relative overflow-hidden"
        >
          <div class="absolute top-0 right-0 w-64 h-64 bg-guofeng-jade/5 rounded-full blur-3xl -mr-16 -mt-16" />
          <div class="relative z-10">
            <div class="flex items-start gap-4">
              <div class="w-12 h-12 rounded-xl bg-guofeng-jade/10 flex items-center justify-center shrink-0">
                <Lightbulb class="w-6 h-6 text-guofeng-jade" />
              </div>
              <div class="flex-1">
                <h3 class="text-lg font-bold text-guofeng-text-primary mb-2">
                  {{ $t('home.quickTipsTitle') }}
                </h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div class="flex items-start gap-3 p-3 rounded-lg bg-guofeng-bg-secondary/30 border border-guofeng-border/30">
                    <div class="w-6 h-6 rounded-md bg-guofeng-jade/10 flex items-center justify-center shrink-0 mt-0.5">
                      <span class="text-xs font-bold text-guofeng-jade">1</span>
                    </div>
                    <div>
                      <h4 class="text-sm font-semibold text-guofeng-text-primary mb-1">
                        {{ $t('home.tip1Title') }}
                      </h4>
                      <p class="text-xs text-guofeng-text-secondary">
                        {{ $t('home.tip1Desc') }}
                      </p>
                    </div>
                  </div>
                  <div class="flex items-start gap-3 p-3 rounded-lg bg-guofeng-bg-secondary/30 border border-guofeng-border/30">
                    <div class="w-6 h-6 rounded-md bg-guofeng-indigo/10 flex items-center justify-center shrink-0 mt-0.5">
                      <span class="text-xs font-bold text-guofeng-indigo">2</span>
                    </div>
                    <div>
                      <h4 class="text-sm font-semibold text-guofeng-text-primary mb-1">
                        {{ $t('home.tip2Title') }}
                      </h4>
                      <p class="text-xs text-guofeng-text-secondary">
                        {{ $t('home.tip2Desc') }}
                      </p>
                    </div>
                  </div>
                  <div class="flex items-start gap-3 p-3 rounded-lg bg-guofeng-bg-secondary/30 border border-guofeng-border/30">
                    <div class="w-6 h-6 rounded-md bg-amber-500/10 flex items-center justify-center shrink-0 mt-0.5">
                      <span class="text-xs font-bold text-amber-500">3</span>
                    </div>
                    <div>
                      <h4 class="text-sm font-semibold text-guofeng-text-primary mb-1">
                        {{ $t('home.tip3Title') }}
                      </h4>
                      <p class="text-xs text-guofeng-text-secondary">
                        {{ $t('home.tip3Desc') }}
                      </p>
                    </div>
                  </div>
                  <div class="flex items-start gap-3 p-3 rounded-lg bg-guofeng-bg-secondary/30 border border-guofeng-border/30">
                    <div class="w-6 h-6 rounded-md bg-cyan-500/10 flex items-center justify-center shrink-0 mt-0.5">
                      <span class="text-xs font-bold text-cyan-500">4</span>
                    </div>
                    <div>
                      <h4 class="text-sm font-semibold text-guofeng-text-primary mb-1">
                        {{ $t('home.tip4Title') }}
                      </h4>
                      <p class="text-xs text-guofeng-text-secondary">
                        {{ $t('home.tip4Desc') }}
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </GuofengCard>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import {
  Settings,
  Cloud,
  Terminal,
  Zap,
  Code2,
  Sparkles,
  ArrowRight,
  Activity,
  Cpu,
  HardDrive,
  TrendingUp,
  Workflow,
  FileText,
  Server,
  Database,
  Lightbulb,
  ChevronRight
} from 'lucide-vue-next'
import GuofengCard from '@/components/common/GuofengCard.vue'
import UsageStatsDashboard from '@/components/UsageStatsDashboard.vue'
import { getSystemInfo, getVersion } from '@/api/client'

interface ModuleCard {
  titleKey: string;
  descriptionKey: string;
  icon: any;
  href: string;
  color: string;
  colorTo?: string;
  statsKey?: string;
}

interface SystemInfo {
  cpu_usage: number;
  memory_usage_percent: number;
  os: string;
  os_version: string;
}

const systemInfo = ref<SystemInfo | null>(null)
const version = ref<string>('')

onMounted(async () => {
  try {
    const [sysInfo, versionInfo] = await Promise.all([
      getSystemInfo(),
      getVersion()
    ])
    systemInfo.value = sysInfo
    version.value = versionInfo.current_version
  } catch (error) {
    console.error('Failed to load dashboard data:', error)
  }
})

// AI CLI 工具 (Core Modules)
const cliTools: ModuleCard[] = [
  {
    titleKey: 'home.claudeCodeTitle',
    descriptionKey: 'home.claudeCodeDesc',
    icon: Code2,
    href: '/claude-code',
    color: '#ff4d4f',
    colorTo: '#ff7875',
    statsKey: 'home.claudeCodeStats',
  },
  {
    titleKey: 'home.codexTitle',
    descriptionKey: 'home.codexDesc',
    icon: Settings,
    href: '/codex',
    color: '#1677b3',
    colorTo: '#40a9ff',
    statsKey: 'home.codexStats',
  },
  {
    titleKey: 'home.geminiTitle',
    descriptionKey: 'home.geminiDesc',
    icon: Sparkles,
    href: '/gemini-cli',
    color: '#10b981',
    colorTo: '#34d399',
    statsKey: 'home.geminiStats',
  },
  {
    titleKey: 'home.qwenTitle',
    descriptionKey: 'home.qwenDesc',
    icon: Zap,
    href: '/qwen',
    color: '#f59e0b',
    colorTo: '#fbbf24',
    statsKey: 'home.qwenStats',
  },
  {
    titleKey: 'home.iflowTitle',
    descriptionKey: 'home.iflowDesc',
    icon: Workflow,
    href: '/iflow',
    color: '#0891b2',
    colorTo: '#22d3ee',
    statsKey: 'home.iflowStats',
  },
]

// 配置与工具 (Quick Actions)
const configTools: ModuleCard[] = [
  {
    titleKey: 'home.commandsTitle',
    descriptionKey: 'home.commandsDesc',
    icon: Terminal,
    href: '/commands',
    color: '#1e293b',
    statsKey: 'home.commandsStats',
  },
  {
    titleKey: 'home.converterTitle',
    descriptionKey: 'home.converterDesc',
    icon: TrendingUp,
    href: '/converter',
    color: '#7c3aed',
    statsKey: 'home.converterStats',
  },
  {
    titleKey: 'home.syncTitle',
    descriptionKey: 'home.syncDesc',
    icon: Cloud,
    href: '/sync',
    color: '#0891b2',
    statsKey: 'home.syncStats',
  },
  {
    titleKey: 'home.usageTitle',
    descriptionKey: 'home.usageDesc',
    icon: Activity,
    href: '/usage',
    color: '#10b981',
    statsKey: 'home.usageStats',
  },
]
</script>