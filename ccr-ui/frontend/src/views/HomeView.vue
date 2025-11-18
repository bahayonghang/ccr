<template>
  <div class="min-h-screen relative">
    <!-- 🎨 动态背景装饰 - 液态玻璃风格 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-20 right-20 w-96 h-96 rounded-full opacity-20 blur-3xl animate-pulse"
        :style="{ background: 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)' }"
      />
      <div
        class="absolute bottom-20 left-20 w-96 h-96 rounded-full opacity-20 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, #ec4899 0%, #f59e0b 100%)',
          animationDelay: '1s'
        }"
      />
      <div
        class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, #10b981 0%, #3b82f6 100%)',
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
                :style="{ color: '#6366f1' }"
              />
            </div>
          </div>

          <h1 class="text-5xl md:text-6xl lg:text-7xl font-bold mb-6 bg-gradient-to-r from-[#6366f1] via-[#8b5cf6] to-[#ec4899] bg-clip-text text-transparent">
            CCR UI
          </h1>

          <p
            class="text-2xl md:text-3xl font-medium mb-4 leading-tight"
            :style="{ color: 'var(--text-primary)' }"
          >
            AI CLI 配置管理中心
          </p>

          <p
            class="text-base md:text-lg mb-6 leading-relaxed max-w-xl"
            :style="{ color: 'var(--text-secondary)' }"
          >
            现代化的多 CLI 工具配置管理解决方案，支持 Claude、Codex、Gemini 等多种 AI 平台。集成配置转换、云同步、命令执行等强大功能，让 AI 工具配置管理更简单高效。
          </p>

          <div
            v-if="version"
            class="inline-flex items-center gap-2 px-5 py-2.5 glass-card text-sm font-semibold animate-slide-in-right"
            :style="{ color: 'var(--accent-primary)' }"
          >
            <Sparkles class="w-4 h-4" />
            <span>v{{ version }}</span>
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
                            style="stop-color:#6366f1"
                          />
                          <stop
                            offset="100%"
                            style="stop-color:#8b5cf6"
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
                        :style="{ color: '#6366f1' }"
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
                    CPU 使用率
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
                            style="stop-color:#8b5cf6"
                          />
                          <stop
                            offset="100%"
                            style="stop-color:#ec4899"
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
                        :style="{ color: '#8b5cf6' }"
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
                    内存使用
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
                      :style="{ color: '#10b981' }"
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
                  :style="{ color: '#f59e0b' }"
                />
              </div>
              <h3
                class="text-lg font-bold"
                :style="{ color: 'var(--text-primary)' }"
              >
                快速操作
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
                    :style="{ color: '#64748b' }"
                  />
                  <span
                    class="text-sm font-medium"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    执行命令
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
                    :style="{ color: '#f97316' }"
                  />
                  <span
                    class="text-sm font-medium"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    配置转换
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
                    :style="{ color: '#06b6d4' }"
                  />
                  <span
                    class="text-sm font-medium"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    云端同步
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
              :style="{ color: '#6366f1' }"
            />
          </div>
          <div>
            <h2
              class="text-3xl font-bold"
              :style="{ color: 'var(--text-primary)' }"
            >
              AI CLI 工具
            </h2>
            <p
              class="text-sm"
              :style="{ color: 'var(--text-muted)' }"
            >
              多种 AI 平台配置管理和工具集成
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
                {{ tool.title }}
              </h3>

              <p
                class="text-xs mb-3 leading-relaxed line-clamp-2 min-h-[2.5rem]"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ tool.description }}
              </p>

              <div class="flex items-center justify-between">
                <span
                  class="text-xs font-semibold px-2.5 py-1 rounded-full"
                  :style="{
                    background: `${tool.color}20`,
                    color: tool.color
                  }"
                >
                  {{ tool.stats }}
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
              :style="{ color: '#8b5cf6' }"
            />
          </div>
          <div>
            <h2
              class="text-3xl font-bold"
              :style="{ color: 'var(--text-primary)' }"
            >
              配置与工具
            </h2>
            <p
              class="text-sm"
              :style="{ color: 'var(--text-muted)' }"
            >
              配置转换、云同步和命令执行中心
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
                {{ config.title }}
              </h3>

              <p
                class="text-sm mb-4 leading-relaxed line-clamp-2"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ config.description }}
              </p>

              <div class="flex items-center justify-between mt-auto">
                <span
                  class="text-xs font-semibold px-3 py-1.5 rounded-full"
                  :style="{
                    background: `${config.color}20`,
                    color: config.color
                  }"
                >
                  {{ config.stats }}
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
          现代化的配置管理解决方案 · 支持多种 AI CLI 工具
        </p>
        <p
          class="text-xs"
          :style="{ color: 'var(--text-muted)' }"
        >
          Claude Code • Codex • Gemini • Qwen • IFLOW
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
  title: string;
  description: string;
  icon: any;
  href: string;
  color: string;
  colorTo?: string;
  stats?: string;
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
    title: 'Claude Code',
    description: '配置管理、云同步、MCP 服务器、Agents、插件',
    icon: Code2,
    href: '/claude-code',
    color: '#6366f1',
    colorTo: '#8b5cf6',
    stats: '核心模块'
  },
  {
    title: 'Codex',
    description: 'MCP 服务器、Profiles、基础配置管理',
    icon: Settings,
    href: '/codex',
    color: '#8b5cf6',
    colorTo: '#a855f7',
    stats: 'AI 编程'
  },
  {
    title: 'Gemini',
    description: 'Google Gemini 配置管理和工具集成',
    icon: Sparkles,
    href: '/gemini-cli',
    color: '#f59e0b',
    colorTo: '#f97316',
    stats: 'Google AI'
  },
  {
    title: 'Qwen',
    description: '阿里通义千问配置管理和服务集成',
    icon: Zap,
    href: '/qwen',
    color: '#10b981',
    colorTo: '#14b8a6',
    stats: '国产大模型'
  },
  {
    title: 'IFLOW',
    description: '内部工作流配置和自动化管理',
    icon: Activity,
    href: '/iflow',
    color: '#3b82f6',
    colorTo: '#2563eb',
    stats: '工作流'
  },
]

// 配置与工具
const configTools: ModuleCard[] = [
  {
    title: '命令执行中心',
    description: '统一的 CLI 命令执行和管理界面，支持多种 AI 平台',
    icon: Terminal,
    href: '/commands',
    color: '#64748b',
    colorTo: '#475569',
    stats: '多 CLI 支持'
  },
  {
    title: '配置转换器',
    description: '跨 CLI 工具的配置格式转换，无缝迁移配置',
    icon: TrendingUp,
    href: '/converter',
    color: '#f97316',
    colorTo: '#ea580c',
    stats: '格式互转'
  },
  {
    title: '云同步',
    description: 'WebDAV 云端配置同步和备份，保护你的配置安全',
    icon: Cloud,
    href: '/sync',
    color: '#06b6d4',
    colorTo: '#0891b2',
    stats: '自动备份'
  },
  {
    title: 'Token 使用统计',
    description: 'Token 使用量可视化分析，活动热力图和使用趋势',
    icon: Activity,
    href: '/usage',
    color: '#10b981',
    colorTo: '#059669',
    stats: '实时监控'
  },
]
</script>