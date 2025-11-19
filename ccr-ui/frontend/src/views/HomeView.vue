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
      <!-- 🌟 头部区域 - 两列布局 -->
      <div class="mb-16 grid grid-cols-1 lg:grid-cols-2 gap-8 lg:gap-12 items-center">
        <!-- 左列：文字内容 -->
        <div class="animate-fade-in">
          <div class="mb-6">
            <div class="flex items-center w-20 h-20 rounded-3xl glass-card mb-6">
              <Code2
                class="w-10 h-10 mx-auto"
                :style="{ color: 'var(--accent-primary)' }"
              />
            </div>
          </div>

          <h1 class="text-5xl md:text-6xl lg:text-7xl font-bold mb-6 brand-gradient-text animate-fade-in">
            {{ $t('home.title') }}
          </h1>

          <p
            class="text-2xl md:text-3xl font-medium mb-4 leading-tight"
            :style="{ color: 'var(--text-primary)' }"
          >
            {{ $t('home.subtitle') }}
          </p>

          <p
            class="text-base md:text-lg mb-6 leading-relaxed max-w-2xl"
            :style="{ color: 'var(--text-secondary)' }"
          >
            {{ $t('home.description') }}
          </p>

          <div
            v-if="version"
            class="inline-flex items-center gap-2 px-5 py-2.5 glass-card text-sm font-semibold animate-slide-in-right"
            :style="{ color: 'var(--accent-primary)' }"
          >
            <Sparkles class="w-4 h-4" />
            <span>{{ $t('home.version') }}{{ version }}</span>
          </div>
        </div>

        <!-- 右列：信息卡片区域 -->
        <div class="space-y-4 animate-fade-in">
          <!-- 系统状态卡片 - 横向排列 -->
          <template v-if="systemInfo">
            <div class="glass-card p-6">
              <div class="grid grid-cols-3 gap-4 md:gap-6">
                <!-- CPU 使用率 -->
                <div 
                  class="text-center group cursor-pointer hover:scale-110 transition-all duration-300"
                  :style="{ animationDelay: '0.1s' }"
                >
                  <div class="relative inline-flex items-center justify-center w-16 h-16 md:w-20 md:h-20 mb-2 md:mb-3">
                    <!-- 背景圆环 -->
                    <svg
                      class="absolute w-full h-full -rotate-90"
                      viewBox="0 0 100 100"
                    >
                      <defs>
                        <linearGradient
                          id="cpuGradient"
                          x1="0%"
                          y1="0%"
                          x2="100%"
                          y2="100%"
                        >
                          <stop
                            offset="0%"
                            style="stop-color:var(--accent-primary)"
                          />
                          <stop
                            offset="100%"
                            style="stop-color:var(--accent-secondary)"
                          />
                        </linearGradient>
                      </defs>
                      <circle
                        cx="50"
                        cy="50"
                        r="40"
                        fill="none"
                        stroke="rgba(99, 102, 241, 0.1)"
                        stroke-width="8"
                      />
                      <circle
                        cx="50"
                        cy="50"
                        r="40"
                        fill="none"
                        stroke="url(#cpuGradient)"
                        stroke-width="8"
                        stroke-linecap="round"
                        :stroke-dasharray="`${(systemInfo.cpu_usage || 0) * 2.51} 251`"
                        class="transition-all duration-500"
                      />
                    </svg>
                    <!-- 图标 -->
                    <div class="relative">
                      <Cpu
                        class="w-8 h-8"
                        :style="{ color: 'var(--accent-primary)' }"
                      />
                    </div>
                  </div>
                  <p
                    class="text-xl md:text-2xl font-bold mb-0.5 md:mb-1"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ systemInfo.cpu_usage?.toFixed(1) || '0.0' }}%
                  </p>
                  <p
                    class="text-xs font-medium"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t('home.cpuUsage') }}
                  </p>
                </div>

                <!-- 内存使用 -->
                <div 
                  class="text-center group cursor-pointer hover:scale-110 transition-all duration-300"
                  :style="{ animationDelay: '0.2s' }"
                >
                  <div class="relative inline-flex items-center justify-center w-16 h-16 md:w-20 md:h-20 mb-2 md:mb-3">
                    <!-- 背景圆环 -->
                    <svg
                      class="absolute w-full h-full -rotate-90"
                      viewBox="0 0 100 100"
                    >
                      <defs>
                        <linearGradient
                          id="memGradient"
                          x1="0%"
                          y1="0%"
                          x2="100%"
                          y2="100%"
                        >
                          <stop
                            offset="0%"
                            style="stop-color:var(--accent-secondary)"
                          />
                          <stop
                            offset="100%"
                            style="stop-color:var(--accent-tertiary)"
                          />
                        </linearGradient>
                      </defs>
                      <circle
                        cx="50"
                        cy="50"
                        r="40"
                        fill="none"
                        stroke="rgba(139, 92, 246, 0.1)"
                        stroke-width="8"
                      />
                      <circle
                        cx="50"
                        cy="50"
                        r="40"
                        fill="none"
                        stroke="url(#memGradient)"
                        stroke-width="8"
                        stroke-linecap="round"
                        :stroke-dasharray="`${(systemInfo.memory_usage_percent || 0) * 2.51} 251`"
                        class="transition-all duration-500"
                      />
                    </svg>
                    <!-- 图标 -->
                    <div class="relative">
                      <HardDrive
                        class="w-8 h-8"
                        :style="{ color: 'var(--accent-secondary)' }"
                      />
                    </div>
                  </div>
                  <p
                    class="text-xl md:text-2xl font-bold mb-0.5 md:mb-1"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ systemInfo.memory_usage_percent?.toFixed(1) || '0.0' }}%
                  </p>
                  <p
                    class="text-xs font-medium"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t('home.memoryUsage') }}
                  </p>
                </div>

                <!-- 系统平台 -->
                <div 
                  class="text-center group cursor-pointer hover:scale-110 transition-all duration-300"
                  :style="{ animationDelay: '0.3s' }"
                >
                  <div
                    class="inline-flex items-center justify-center w-16 h-16 md:w-20 md:h-20 mb-2 md:mb-3 rounded-full"
                    :style="{ background: 'linear-gradient(135deg, rgba(16, 185, 129, 0.15), rgba(6, 182, 212, 0.15))' }"
                  >
                    <Activity
                      class="w-7 h-7 md:w-8 md:h-8"
                      :style="{ color: 'var(--accent-success)' }"
                    />
                  </div>
                  <p
                    class="text-base md:text-lg font-bold mb-0.5 md:mb-1 truncate px-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ systemInfo.os }}
                  </p>
                  <p
                    class="text-xs font-medium"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ systemInfo.os_version }}
                  </p>
                </div>
              </div>
            </div>
          </template>

          <!-- 快速操作卡片 -->
          <div
            class="glass-card p-6 hover:scale-105 transition-all duration-300"
            :style="{ animationDelay: '0.4s' }"
          >
            <div class="flex items-center gap-3 mb-4">
              <div
                class="p-3 rounded-2xl"
                :style="{ background: 'rgba(245, 158, 11, 0.1)' }"
              >
                <Zap
                  class="w-6 h-6"
                  :style="{ color: 'var(--accent-warning)' }"
                />
              </div>
              <h3
                class="text-lg font-bold"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ $t('home.quickActions') }}
              </h3>
            </div>
            <div class="space-y-2">
              <RouterLink
                to="/commands"
                class="flex items-center justify-between p-3 rounded-xl hover:bg-gradient-to-r hover:from-accent-primary/10 hover:to-accent-secondary/10 transition-all group"
              >
                <div class="flex items-center gap-2">
                  <Terminal
                    class="w-4 h-4"
                    :style="{ color: 'var(--text-secondary)' }"
                  />
                  <span
                    class="text-sm font-medium"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('home.executeCommands') }}
                  </span>
                </div>
                <ArrowRight
                  class="w-4 h-4 group-hover:translate-x-1 transition-transform"
                  :style="{ color: 'var(--text-muted)' }"
                />
              </RouterLink>
              <RouterLink
                to="/converter"
                class="flex items-center justify-between p-3 rounded-xl hover:bg-gradient-to-r hover:from-accent-primary/10 hover:to-accent-secondary/10 transition-all group"
              >
                <div class="flex items-center gap-2">
                  <TrendingUp
                    class="w-4 h-4"
                    :style="{ color: 'var(--accent-warning)' }"
                  />
                  <span
                    class="text-sm font-medium"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('home.configConverter') }}
                  </span>
                </div>
                <ArrowRight
                  class="w-4 h-4 group-hover:translate-x-1 transition-transform"
                  :style="{ color: 'var(--text-muted)' }"
                />
              </RouterLink>
              <RouterLink
                to="/sync"
                class="flex items-center justify-between p-3 rounded-xl hover:bg-gradient-to-r hover:from-accent-primary/10 hover:to-accent-secondary/10 transition-all group"
              >
                <div class="flex items-center gap-2">
                  <Cloud
                    class="w-4 h-4"
                    :style="{ color: 'var(--accent-info)' }"
                  />
                  <span
                    class="text-sm font-medium"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('home.cloudSync') }}
                  </span>
                </div>
                <ArrowRight
                  class="w-4 h-4 group-hover:translate-x-1 transition-transform"
                  :style="{ color: 'var(--text-muted)' }"
                />
              </RouterLink>
            </div>
          </div>
        </div>
      </div>

      <!-- 🤖 AI CLI 工具 -->
      <div class="mb-12">
        <div class="flex items-center gap-3 mb-6">
          <div
            class="p-3 rounded-2xl glass-card"
            :style="{ background: 'rgba(99, 102, 241, 0.15)' }"
          >
            <Code2
              class="w-6 h-6"
              :style="{ color: 'var(--accent-primary)' }"
            />
          </div>
          <div>
            <h2
              class="text-3xl font-bold"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ $t('home.aiCliTools') }}
            </h2>
            <p
              class="text-sm"
              :style="{ color: 'var(--text-muted)' }"
            >
              {{ $t('home.aiCliToolsDesc') }}
            </p>
          </div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-5">
          <RouterLink
            v-for="(tool, index) in cliTools"
            :key="tool.href"
            :to="tool.href"
            class="group block"
            :style="{ animationDelay: `${index * 0.05}s` }"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="mb-4">
                <div
                  class="inline-flex p-3 rounded-2xl"
                  :style="{ background: `${tool.color}15` }"
                >
                  <component
                    :is="tool.icon"
                    class="w-6 h-6"
                    :style="{ color: tool.color }"
                  />
                </div>
              </div>

              <h3
                class="text-lg font-bold mb-2 group-hover:text-transparent group-hover:bg-gradient-to-r group-hover:bg-clip-text transition-all"
                :style="{ 
                  color: 'var(--text-primary)',
                  '--tw-gradient-from': tool.color,
                  '--tw-gradient-to': tool.colorTo || tool.color
                }"
              >
                {{ $t(tool.titleKey) }}
              </h3>

              <p
                class="text-xs mb-3 leading-relaxed line-clamp-2 min-h-[2.5rem]"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ $t(tool.descriptionKey) }}
              </p>

              <div class="flex items-center justify-between">
                <span
                  class="text-xs font-semibold px-2.5 py-1 rounded-full"
                  :style="{
                    background: `${tool.color}20`,
                    color: tool.color
                  }"
                >
                  <span v-if="tool.statsKey">{{ $t(tool.statsKey) }}</span>
                </span>
                <ArrowRight
                  class="w-4 h-4 group-hover:translate-x-1 transition-transform"
                  :style="{ color: tool.color }"
                />
              </div>
            </div>
          </RouterLink>
        </div>
      </div>

      <!-- ⚙️ 配置与工具 -->
      <div>
        <div class="flex items-center gap-3 mb-6">
          <div
            class="p-3 rounded-2xl glass-card"
            :style="{ background: 'rgba(139, 92, 246, 0.15)' }"
          >
            <Settings
              class="w-6 h-6"
              :style="{ color: 'var(--accent-secondary)' }"
            />
          </div>
          <div>
            <h2
              class="text-3xl font-bold"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ $t('home.configAndTools') }}
            </h2>
            <p
              class="text-sm"
              :style="{ color: 'var(--text-muted)' }"
            >
              {{ $t('home.configAndToolsDesc') }}
            </p>
          </div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
          <RouterLink
            v-for="(config, index) in configTools"
            :key="config.href"
            :to="config.href"
            class="group block"
            :style="{ animationDelay: `${(index + cliTools.length) * 0.05}s` }"
          >
            <div class="glass-card p-7 h-full hover:scale-105 transition-all duration-300">
              <div class="mb-5">
                <div
                  class="inline-flex p-4 rounded-2xl"
                  :style="{ background: `${config.color}15` }"
                >
                  <component
                    :is="config.icon"
                    class="w-7 h-7"
                    :style="{ color: config.color }"
                  />
                </div>
              </div>

              <h3
                class="text-xl font-bold mb-3 group-hover:text-transparent group-hover:bg-gradient-to-r group-hover:bg-clip-text transition-all"
                :style="{ 
                  color: 'var(--text-primary)',
                  '--tw-gradient-from': config.color,
                  '--tw-gradient-to': config.colorTo || config.color
                }"
              >
                {{ $t(config.titleKey) }}
              </h3>

              <p
                class="text-sm mb-4 leading-relaxed line-clamp-2"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ $t(config.descriptionKey) }}
              </p>

              <div class="flex items-center justify-between mt-auto">
                <span
                  class="text-xs font-semibold px-3 py-1.5 rounded-full"
                  :style="{
                    background: `${config.color}20`,
                    color: config.color
                  }"
                >
                  <span v-if="config.statsKey">{{ $t(config.statsKey) }}</span>
                </span>
                <ArrowRight
                  class="w-5 h-5 group-hover:translate-x-1 transition-transform"
                  :style="{ color: config.color }"
                />
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
    color: '#6366f1',
    colorTo: '#8b5cf6',
    statsKey: 'home.claudeCodeStats',
  },
  {
    titleKey: 'home.codexTitle',
    descriptionKey: 'home.codexDesc',
    icon: Settings,
    href: '/codex',
    color: '#8b5cf6',
    colorTo: '#a855f7',
    statsKey: 'home.codexStats',
  },
  {
    titleKey: 'home.geminiTitle',
    descriptionKey: 'home.geminiDesc',
    icon: Sparkles,
    href: '/gemini-cli',
    color: '#f59e0b',
    colorTo: '#f97316',
    statsKey: 'home.geminiStats',
  },
  {
    titleKey: 'home.qwenTitle',
    descriptionKey: 'home.qwenDesc',
    icon: Zap,
    href: '/qwen',
    color: '#10b981',
    colorTo: '#14b8a6',
    statsKey: 'home.qwenStats',
  },
  {
    titleKey: 'home.iflowTitle',
    descriptionKey: 'home.iflowDesc',
    icon: Activity,
    href: '/iflow',
    color: '#3b82f6',
    colorTo: '#2563eb',
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
    color: '#64748b',
    colorTo: '#475569',
    statsKey: 'home.commandsStats',
  },
  {
    titleKey: 'home.converterTitle',
    descriptionKey: 'home.converterDesc',
    icon: TrendingUp,
    href: '/converter',
    color: '#f97316',
    colorTo: '#ea580c',
    statsKey: 'home.converterStats',
  },
  {
    titleKey: 'home.syncTitle',
    descriptionKey: 'home.syncDesc',
    icon: Cloud,
    href: '/sync',
    color: '#06b6d4',
    colorTo: '#0891b2',
    statsKey: 'home.syncStats',
  },
  {
    titleKey: 'home.usageTitle',
    descriptionKey: 'home.usageDesc',
    icon: Activity,
    href: '/usage',
    color: '#10b981',
    colorTo: '#059669',
    statsKey: 'home.usageStats',
  },
]
</script>