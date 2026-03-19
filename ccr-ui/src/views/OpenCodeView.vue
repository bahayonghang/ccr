<template>
  <div class="min-h-full p-6 lg:p-10 relative overflow-hidden">
    <AnimatedBackground variant="complex" />

    <div class="max-w-7xl mx-auto space-y-5">
      <!-- HEADER -->
      <section class="grid grid-cols-1 lg:grid-cols-3 gap-4 animate-slide-up">
        <!-- Hero Card -->
        <Card
          variant="glass"
          class="lg:col-span-2 relative overflow-hidden p-5 flex flex-col"
        >
          <div class="absolute top-0 right-0 w-48 h-48 bg-gradient-to-bl from-violet-500/10 to-transparent -mr-12 -mt-12 rounded-bl-full pointer-events-none" />

          <div class="relative z-10">
            <div class="flex items-center gap-3 mb-3">
              <div class="w-12 h-12 rounded-xl bg-violet-500/10 flex items-center justify-center border border-violet-500/20 shadow-lg backdrop-blur-md">
                <SIcon
                  name="TerminalSquare"
                  size="w-6 h-6"
                  class="text-violet-500"
                />
              </div>
              <div>
                <h1 class="text-3xl font-bold font-display text-white tracking-tight">
                  OpenCode
                </h1>
                <p class="text-white/80 text-base max-w-md">
                  基于 npm AI SDK 的叠加式 Provider 配置管理
                </p>
              </div>
            </div>

            <div class="flex flex-wrap gap-2">
              <span class="px-3 py-1 rounded-full text-xs font-bold uppercase tracking-wider bg-violet-500/10 text-violet-500 border border-violet-500/20 flex items-center gap-2">
                <SIcon
                  name="Layers"
                  size="w-3 h-3"
                /> npm AI SDK
              </span>
              <span class="px-3 py-1 rounded-full text-xs font-bold uppercase tracking-wider bg-accent-secondary/10 text-accent-secondary border border-accent-secondary/20">
                opencode.json
              </span>
            </div>
          </div>
        </Card>

        <!-- Status Grid -->
        <div class="grid grid-cols-1 gap-3">
          <!-- Provider Count -->
          <Card
            variant="elevated"
            class="p-3 flex items-center gap-3 border-l-4 border-l-violet-500"
          >
            <div class="w-10 h-10 rounded-lg bg-violet-500/10 flex items-center justify-center text-violet-500 shrink-0">
              <SIcon
                name="Layers"
                size="w-5 h-5"
              />
            </div>
            <div>
              <p class="text-xs font-bold text-white/50 uppercase tracking-wider mb-0.5">
                Providers
              </p>
              <p class="text-base font-bold text-white">
                {{ providersCount }}
              </p>
            </div>
          </Card>

          <!-- MCP Count -->
          <Card
            variant="elevated"
            class="p-3 flex items-center gap-3 border-l-4 border-l-blue-500"
          >
            <div class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center text-blue-500 shrink-0">
              <SIcon
                name="Server"
                size="w-5 h-5"
              />
            </div>
            <div>
              <p class="text-xs font-bold text-white/50 uppercase tracking-wider mb-0.5">
                MCP 服务器
              </p>
              <p class="text-base font-bold text-white">
                {{ mcpCount }}
              </p>
            </div>
          </Card>

          <!-- Plugin Count -->
          <Card
            variant="elevated"
            class="p-3 flex items-center gap-3 border-l-4 border-l-emerald-500"
          >
            <div class="w-10 h-10 rounded-lg bg-emerald-500/10 flex items-center justify-center text-emerald-500 shrink-0">
              <SIcon
                name="Puzzle"
                size="w-5 h-5"
              />
            </div>
            <div>
              <p class="text-xs font-bold text-white/50 uppercase tracking-wider mb-0.5">
                插件
              </p>
              <p class="text-base font-bold text-white">
                {{ pluginsCount }}
              </p>
            </div>
          </Card>
        </div>
      </section>

      <!-- MODULE NAVIGATION -->
      <section
        class="animate-slide-up"
      >
        <h2 class="text-lg font-bold text-white mb-3">
          功能模块
        </h2>
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          <RouterLink
            v-for="mod in modules"
            :key="mod.href"
            :to="mod.href"
            class="group"
          >
            <Card
              variant="glass"
              class="p-4 flex flex-col gap-3 cursor-pointer transition-[box-shadow,border-color,transform] duration-300 hover:scale-[1.02] hover:shadow-lg border border-transparent hover:border-violet-500/30"
            >
              <div
                class="w-10 h-10 rounded-lg flex items-center justify-center"
                :class="mod.bgClass"
              >
                <SIcon
                  :name="mod.icon"
                  size="w-5 h-5"
                  :class="mod.iconClass"
                />
              </div>
              <div>
                <h3 class="font-bold text-white text-sm mb-1">
                  {{ mod.title }}
                </h3>
                <p class="text-white/50 text-xs leading-relaxed">
                  {{ mod.description }}
                </p>
              </div>
            </Card>
          </RouterLink>
        </div>
      </section>

      <!-- CONFIG PATH INFO -->
      <section
        v-if="configPath"
        class="animate-slide-up"
        style="animation-delay: 0.2s"
      >
        <Card
          variant="elevated"
          class="p-4 flex items-center gap-3"
        >
          <SIcon
            name="FileJson"
            size="w-5 h-5"
            class="text-white/50 shrink-0"
          />
          <div>
            <p class="text-xs font-bold text-white/50 uppercase tracking-wider mb-0.5">
              配置文件路径
            </p>
            <p class="text-sm font-mono text-white/80">
              ~/.config/opencode/opencode.json
            </p>
          </div>
        </Card>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted } from 'vue'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import Card from '@/components/ui/Card.vue'
import {
  listOpenCodeProviders,
  listOpenCodeMcpServers,
  listOpenCodePlugins,
} from '@/api'
import type { OpenCodeProvider, OpenCodeMcpServer, OpenCodePlugin } from '@/types'

const providersCount = ref(0)
const mcpCount = ref(0)
const pluginsCount = ref(0)
const configPath = ref(true)

const modules = [
  {
    title: 'Provider 管理',
    description: '管理 npm AI SDK Provider，配置 API Key 和模型列表',
    href: '/opencode/providers',
    icon: 'Layers',
    bgClass: 'bg-violet-500/10',
    iconClass: 'text-violet-500',
  },
  {
    title: 'MCP 服务器',
    description: '管理本地（local）和远程（remote）MCP 服务器',
    href: '/opencode/mcp',
    icon: 'Server',
    bgClass: 'bg-blue-500/10',
    iconClass: 'text-blue-500',
  },
  {
    title: 'Skills',
    description: '管理 AI 技能库，跨平台共享 Skills 配置',
    href: '/skills',
    icon: 'Puzzle',
    bgClass: 'bg-amber-500/10',
    iconClass: 'text-amber-500',
  },
  {
    title: '插件管理',
    description: '管理 npm 插件包，扩展 OpenCode 功能',
    href: '/opencode/plugins',
    icon: 'Puzzle',
    bgClass: 'bg-emerald-500/10',
    iconClass: 'text-emerald-500',
  },
]

onMounted(async () => {
  try {
    const [providers, mcpServers, plugins] = await Promise.all([
      listOpenCodeProviders<OpenCodeProvider[]>(),
      listOpenCodeMcpServers<OpenCodeMcpServer[]>(),
      listOpenCodePlugins<OpenCodePlugin[]>(),
    ])
    providersCount.value = providers.length
    mcpCount.value = mcpServers.length
    pluginsCount.value = plugins.length
  } catch {
    // 静默失败，OpenCode 可能未安装
  }
})
</script>
