<template>
  <div class="min-h-screen relative">
    <!-- 🎨 动态背景装饰 - 液态玻璃风格 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-20 right-20 w-96 h-96 rounded-full opacity-20 blur-3xl animate-pulse"
        :style="{ background: 'linear-gradient(135deg, var(--accent-primary) 0%, var(--accent-secondary) 100%)' }"
      />
      <div
        class="absolute bottom-20 left-20 w-96 h-96 rounded-full opacity-20 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, var(--accent-tertiary) 0%, var(--accent-warning) 100%)',
          animationDelay: '1s'
        }"
      />
      <div
        class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, var(--accent-success) 0%, var(--accent-info) 100%)',
          animationDelay: '2s'
        }"
      />
    </div>

    <div class="relative z-10 container mx-auto px-6 py-16">
      <!-- 🌟 头部区域 - 居中布局 -->
      <div class="flex flex-col items-center text-center mb-16 animate-fade-in">
        <div class="mb-8 relative group">
          <div class="absolute inset-0 bg-accent-primary/20 blur-xl rounded-full group-hover:bg-accent-primary/30 transition-all duration-500"></div>
          <div class="relative flex items-center justify-center w-24 h-24 rounded-3xl glass-effect mb-0 ring-1 ring-white/20 shadow-2xl group-hover:scale-105 transition-transform duration-500">
            <Code2
              class="w-12 h-12"
              :style="{ color: 'var(--accent-primary)' }"
            />
          </div>
        </div>

        <h1 class="text-6xl md:text-7xl lg:text-8xl font-bold mb-6 brand-gradient-text tracking-tight">
          {{ $t('home.title') }}
        </h1>

        <p
          class="text-2xl md:text-3xl font-medium mb-6 max-w-3xl mx-auto leading-tight"
          :style="{ color: 'var(--text-primary)' }"
        >
          {{ $t('home.subtitle') }}
        </p>

        <p
          class="text-lg md:text-xl mb-8 leading-relaxed max-w-2xl mx-auto"
          :style="{ color: 'var(--text-secondary)' }"
        >
          {{ $t('home.description') }}
        </p>

        <div
          v-if="version"
          class="inline-flex items-center gap-2 px-6 py-2.5 rounded-full glass-effect text-sm font-semibold hover:bg-white/10 transition-colors"
          :style="{ color: 'var(--accent-primary)' }"
        >
          <Sparkles class="w-4 h-4" />
          <span>{{ $t('home.version') }}{{ version }}</span>
        </div>
      </div>

      <!-- 📊 系统状态条 - 横向玻璃条 -->
      <div v-if="systemInfo" class="mb-20 animate-fade-in" style="animation-delay: 0.1s">
        <div class="glass-effect rounded-2xl p-4 md:p-6 border border-white/10 shadow-lg">
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6 divide-y md:divide-y-0 md:divide-x divide-white/10">
            <!-- CPU -->
            <div class="flex items-center justify-center gap-4 px-4">
              <div class="relative w-16 h-16">
                <svg class="w-full h-full -rotate-90" viewBox="0 0 100 100">
                  <circle cx="50" cy="50" r="45" fill="none" stroke="currentColor" stroke-width="8" class="text-slate-200 dark:text-slate-800" />
                  <circle
                    cx="50"
                    cy="50"
                    r="45"
                    fill="none"
                    :stroke="`var(--accent-primary)`"
                    stroke-width="8"
                    stroke-linecap="round"
                    :stroke-dasharray="`${(systemInfo.cpu_usage || 0) * 2.83} 283`"
                    class="transition-all duration-1000 ease-out"
                  />
                </svg>
                <Cpu class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-6 h-6 text-accent-primary" />
              </div>
              <div class="text-left">
                <div class="text-2xl font-bold tabular-nums">{{ systemInfo.cpu_usage?.toFixed(1) || '0.0' }}%</div>
                <div class="text-xs font-medium text-muted-foreground uppercase tracking-wider">{{ $t('home.cpuUsage') }}</div>
              </div>
            </div>

            <!-- Memory -->
            <div class="flex items-center justify-center gap-4 px-4 pt-6 md:pt-0">
              <div class="relative w-16 h-16">
                <svg class="w-full h-full -rotate-90" viewBox="0 0 100 100">
                  <circle cx="50" cy="50" r="45" fill="none" stroke="currentColor" stroke-width="8" class="text-slate-200 dark:text-slate-800" />
                  <circle
                    cx="50"
                    cy="50"
                    r="45"
                    fill="none"
                    :stroke="`var(--accent-secondary)`"
                    stroke-width="8"
                    stroke-linecap="round"
                    :stroke-dasharray="`${(systemInfo.memory_usage_percent || 0) * 2.83} 283`"
                    class="transition-all duration-1000 ease-out"
                  />
                </svg>
                <HardDrive class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-6 h-6 text-accent-secondary" />
              </div>
              <div class="text-left">
                <div class="text-2xl font-bold tabular-nums">{{ systemInfo.memory_usage_percent?.toFixed(1) || '0.0' }}%</div>
                <div class="text-xs font-medium text-muted-foreground uppercase tracking-wider">{{ $t('home.memoryUsage') }}</div>
              </div>
            </div>

            <!-- OS -->
            <div class="flex items-center justify-center gap-4 px-4 pt-6 md:pt-0">
              <div class="w-16 h-16 rounded-full bg-gradient-to-br from-emerald-500/10 to-cyan-500/10 flex items-center justify-center ring-1 ring-emerald-500/20">
                <Activity class="w-8 h-8 text-emerald-500" />
              </div>
              <div class="text-left">
                <div class="text-lg font-bold truncate max-w-[150px]" :title="systemInfo.os">{{ systemInfo.os }}</div>
                <div class="text-xs font-medium text-muted-foreground">{{ systemInfo.os_version }}</div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- ⚡ 快速操作 -->
      <div class="mb-20">
        <div class="flex items-center justify-center gap-3 mb-10">
          <div class="h-px w-12 bg-gradient-to-r from-transparent to-accent-warning/50"></div>
          <div class="flex items-center gap-2 px-4 py-1.5 rounded-full bg-accent-warning/10 text-accent-warning border border-accent-warning/20">
            <Zap class="w-4 h-4" />
            <span class="text-sm font-bold uppercase tracking-wider">{{ $t('home.quickActions') }}</span>
          </div>
          <div class="h-px w-12 bg-gradient-to-l from-transparent to-accent-warning/50"></div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-5xl mx-auto">
          <RouterLink
            to="/commands"
            class="group relative overflow-hidden rounded-2xl liquid-card p-6 hover:shadow-lg transition-all duration-300"
          >
            <div class="absolute top-0 right-0 p-4 opacity-10 group-hover:opacity-20 transition-opacity transform group-hover:scale-110 duration-500">
              <Terminal class="w-24 h-24" />
            </div>
            <div class="relative z-10">
              <div class="w-12 h-12 rounded-xl bg-slate-100 dark:bg-slate-800 flex items-center justify-center mb-4 group-hover:scale-110 transition-transform duration-300">
                <Terminal class="w-6 h-6 text-slate-600 dark:text-slate-300" />
              </div>
              <h3 class="text-lg font-bold mb-2">{{ $t('home.executeCommands') }}</h3>
              <div class="flex items-center text-sm text-muted-foreground group-hover:text-primary transition-colors">
                <span>{{ $t('home.executeCommandsDesc') || 'Run CLI commands directly' }}</span>
                <ArrowRight class="w-4 h-4 ml-2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-300" />
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/converter"
            class="group relative overflow-hidden rounded-2xl liquid-card p-6 hover:shadow-lg transition-all duration-300"
          >
            <div class="absolute top-0 right-0 p-4 opacity-10 group-hover:opacity-20 transition-opacity transform group-hover:scale-110 duration-500">
              <TrendingUp class="w-24 h-24 text-purple-500" />
            </div>
            <div class="relative z-10">
              <div class="w-12 h-12 rounded-xl bg-purple-500/10 flex items-center justify-center mb-4 group-hover:scale-110 transition-transform duration-300">
                <TrendingUp class="w-6 h-6 text-purple-500" />
              </div>
              <h3 class="text-lg font-bold mb-2">{{ $t('home.configConverter') }}</h3>
              <div class="flex items-center text-sm text-muted-foreground group-hover:text-purple-500 transition-colors">
                <span>{{ $t('home.configConverterDesc') || 'Convert config formats' }}</span>
                <ArrowRight class="w-4 h-4 ml-2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-300" />
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/sync"
            class="group relative overflow-hidden rounded-2xl liquid-card p-6 hover:shadow-lg transition-all duration-300"
          >
            <div class="absolute top-0 right-0 p-4 opacity-10 group-hover:opacity-20 transition-opacity transform group-hover:scale-110 duration-500">
              <Cloud class="w-24 h-24 text-cyan-500" />
            </div>
            <div class="relative z-10">
              <div class="w-12 h-12 rounded-xl bg-cyan-500/10 flex items-center justify-center mb-4 group-hover:scale-110 transition-transform duration-300">
                <Cloud class="w-6 h-6 text-cyan-500" />
              </div>
              <h3 class="text-lg font-bold mb-2">{{ $t('home.cloudSync') }}</h3>
              <div class="flex items-center text-sm text-muted-foreground group-hover:text-cyan-500 transition-colors">
                <span>{{ $t('home.cloudSyncDesc') || 'Sync across devices' }}</span>
                <ArrowRight class="w-4 h-4 ml-2 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-300" />
              </div>
            </div>
          </RouterLink>
        </div>
      </div>

      <!-- 🤖 AI CLI 工具 -->
      <div class="mb-20">
        <div class="flex items-center justify-center gap-3 mb-10">
          <div class="h-px w-12 bg-gradient-to-r from-transparent to-accent-primary/50"></div>
          <div class="flex items-center gap-2 px-4 py-1.5 rounded-full bg-accent-primary/10 text-accent-primary border border-accent-primary/20">
            <Code2 class="w-4 h-4" />
            <span class="text-sm font-bold uppercase tracking-wider">{{ $t('home.aiCliTools') }}</span>
          </div>
          <div class="h-px w-12 bg-gradient-to-l from-transparent to-accent-primary/50"></div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-6">
          <RouterLink
            v-for="(tool, index) in cliTools"
            :key="tool.href"
            :to="tool.href"
            class="group relative block h-full"
            :style="{ animationDelay: `${index * 0.05}s` }"
          >
            <div class="liquid-card h-full p-6 transition-all duration-300 group-hover:-translate-y-1 group-hover:shadow-xl border border-white/40 dark:border-white/10">
              <div class="absolute inset-0 bg-gradient-to-br from-white/40 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none"></div>
              
              <div class="relative z-10 flex flex-col h-full">
                <div class="mb-5 flex justify-between items-start">
                  <div
                    class="inline-flex p-3.5 rounded-2xl shadow-sm ring-1 ring-white/50 dark:ring-white/10 backdrop-blur-md"
                    :style="{ background: `${tool.color}15` }"
                  >
                    <component
                      :is="tool.icon"
                      class="w-7 h-7"
                      :style="{ color: tool.color }"
                    />
                  </div>
                  <div 
                    class="opacity-0 group-hover:opacity-100 transition-opacity duration-300 transform translate-x-2 group-hover:translate-x-0"
                  >
                    <ArrowRight class="w-5 h-5" :style="{ color: tool.color }" />
                  </div>
                </div>

                <h3
                  class="text-xl font-bold mb-2 group-hover:text-transparent group-hover:bg-gradient-to-r group-hover:bg-clip-text transition-all"
                  :style="{ 
                    color: 'var(--text-primary)',
                    '--tw-gradient-from': tool.color,
                    '--tw-gradient-to': tool.colorTo || tool.color
                  }"
                >
                  {{ $t(tool.titleKey) }}
                </h3>

                <p
                  class="text-sm mb-4 leading-relaxed line-clamp-2 text-muted-foreground flex-grow"
                >
                  {{ $t(tool.descriptionKey) }}
                </p>

                <div class="mt-auto pt-4 border-t border-dashed border-slate-200 dark:border-slate-700/50">
                  <span
                    class="inline-flex items-center text-xs font-bold px-2.5 py-1 rounded-md"
                    :style="{
                      background: `${tool.color}10`,
                      color: tool.color
                    }"
                  >
                    <span v-if="tool.statsKey">{{ $t(tool.statsKey) }}</span>
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>
        </div>
      </div>

      <!-- ⚙️ 配置与工具 -->
      <div class="mb-20">
        <div class="flex items-center justify-center gap-3 mb-10">
          <div class="h-px w-12 bg-gradient-to-r from-transparent to-accent-secondary/50"></div>
          <div class="flex items-center gap-2 px-4 py-1.5 rounded-full bg-accent-secondary/10 text-accent-secondary border border-accent-secondary/20">
            <Settings class="w-4 h-4" />
            <span class="text-sm font-bold uppercase tracking-wider">{{ $t('home.configAndTools') }}</span>
          </div>
          <div class="h-px w-12 bg-gradient-to-l from-transparent to-accent-secondary/50"></div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <RouterLink
            v-for="(config, index) in configTools"
            :key="config.href"
            :to="config.href"
            class="group relative block h-full"
            :style="{ animationDelay: `${(index + cliTools.length) * 0.05}s` }"
          >
            <div class="liquid-card h-full p-6 transition-all duration-300 group-hover:-translate-y-1 group-hover:shadow-xl border border-white/40 dark:border-white/10">
              <div class="absolute inset-0 bg-gradient-to-br from-white/40 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none"></div>
              
              <div class="relative z-10 flex flex-col h-full">
                <div class="mb-5 flex justify-between items-start">
                  <div
                    class="inline-flex p-3.5 rounded-2xl shadow-sm ring-1 ring-white/50 dark:ring-white/10 backdrop-blur-md"
                    :style="{ background: `${config.color}15` }"
                  >
                    <component
                      :is="config.icon"
                      class="w-7 h-7"
                      :style="{ color: config.color }"
                    />
                  </div>
                  <div 
                    class="opacity-0 group-hover:opacity-100 transition-opacity duration-300 transform translate-x-2 group-hover:translate-x-0"
                  >
                    <ArrowRight class="w-5 h-5" :style="{ color: config.color }" />
                  </div>
                </div>

                <h3
                  class="text-xl font-bold mb-2 group-hover:text-transparent group-hover:bg-gradient-to-r group-hover:bg-clip-text transition-all"
                  :style="{ 
                    color: 'var(--text-primary)',
                    '--tw-gradient-from': config.color,
                    '--tw-gradient-to': config.colorTo || config.color
                  }"
                >
                  {{ $t(config.titleKey) }}
                </h3>

                <p
                  class="text-sm mb-4 leading-relaxed line-clamp-2 text-muted-foreground flex-grow"
                >
                  {{ $t(config.descriptionKey) }}
                </p>

                <div class="mt-auto pt-4 border-t border-dashed border-slate-200 dark:border-slate-700/50">
                  <span
                    class="inline-flex items-center text-xs font-bold px-2.5 py-1 rounded-md"
                    :style="{
                      background: `${config.color}10`,
                      color: config.color
                    }"
                  >
                    <span v-if="config.statsKey">{{ $t(config.statsKey) }}</span>
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>
        </div>
      </div>

      <!-- 🌈 底部信息 -->
      <div class="mt-20 text-center">
        <p
          class="text-sm mb-2"
          :style="{ color: 'var(--text-muted)' }"
        >
          {{ $t('home.footer1') }}
        </p>
        <p
          class="text-xs"
          :style="{ color: 'var(--text-muted)' }"
        >
          {{ $t('home.footer2') }}
        </p>
      </div>
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
} from 'lucide-vue-next'
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

// AI CLI 工具
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
    icon: Activity,
    href: '/iflow',
    color: '#0891b2',
    colorTo: '#22d3ee',
    statsKey: 'home.iflowStats',
  },
]

// 配置与工具
const configTools: ModuleCard[] = [
  {
    titleKey: 'home.commandsTitle',
    descriptionKey: 'home.commandsDesc',
    icon: Terminal,
    href: '/commands',
    color: '#1e293b',
    colorTo: '#475569',
    statsKey: 'home.commandsStats',
  },
  {
    titleKey: 'home.converterTitle',
    descriptionKey: 'home.converterDesc',
    icon: TrendingUp,
    href: '/converter',
    color: '#7c3aed',
    colorTo: '#8b5cf6',
    statsKey: 'home.converterStats',
  },
  {
    titleKey: 'home.syncTitle',
    descriptionKey: 'home.syncDesc',
    icon: Cloud,
    href: '/sync',
    color: '#0891b2',
    colorTo: '#22d3ee',
    statsKey: 'home.syncStats',
  },
  {
    titleKey: 'home.usageTitle',
    descriptionKey: 'home.usageDesc',
    icon: Activity,
    href: '/usage',
    color: '#10b981',
    colorTo: '#34d399',
    statsKey: 'home.usageStats',
  },
]
</script>