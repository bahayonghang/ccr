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
              <Code2 class="w-8 h-8 text-guofeng-jade" />
            </div>
          </div>
          <div>
            <h1 class="text-3xl font-bold brand-gradient-text tracking-tight mb-1">
              Claude Code
            </h1>
            <p class="text-sm text-guofeng-text-secondary">
              {{ $t('claudeCode.subtitle') }}
            </p>
          </div>
        </div>

        <RouterLink 
          to="/" 
          class="flex items-center gap-2 px-5 py-2.5 rounded-xl glass-effect hover:scale-105 transition-all duration-300 text-guofeng-text-secondary hover:text-guofeng-jade"
        >
          <Home class="w-5 h-5" />
          <span class="font-medium">{{ $t('claudeCode.backToHome') }}</span>
        </RouterLink>
      </header>

      <!-- 特性标签 -->
      <div
        class="flex flex-wrap gap-3 animate-fade-in"
        style="animation-delay: 0.1s"
      >
        <span class="px-4 py-2 rounded-full text-sm font-medium glass-effect bg-guofeng-jade/5 text-guofeng-jade border border-guofeng-jade/20">
          ⚡ {{ $t('claudeCode.features.mcpSupport') }}
        </span>
        <span class="px-4 py-2 rounded-full text-sm font-medium glass-effect bg-guofeng-indigo/5 text-guofeng-indigo border border-guofeng-indigo/20">
          🤖 {{ $t('claudeCode.features.aiAgents') }}
        </span>
        <span class="px-4 py-2 rounded-full text-sm font-medium glass-effect bg-cyan-500/5 text-cyan-500 border border-cyan-500/20">
          ☁️ {{ $t('claudeCode.features.cloudSync') }}
        </span>
        <span class="px-4 py-2 rounded-full text-sm font-medium glass-effect bg-amber-500/5 text-amber-500 border border-amber-500/20">
          🔧 {{ $t('claudeCode.features.configManagement') }}
        </span>
      </div>

      <!-- 核心功能 (Config & Sync) -->
      <section
        class="grid grid-cols-1 md:grid-cols-2 gap-5 animate-fade-in"
        style="animation-delay: 0.2s"
      >
        <RouterLink
          to="/configs"
          class="block group"
        >
          <GuofengCard
            variant="glass"
            interactive
            class="h-full p-6 border-l-4 border-l-guofeng-jade"
          >
            <div class="flex items-start gap-4">
              <div class="w-14 h-14 rounded-2xl bg-guofeng-jade/10 flex items-center justify-center shrink-0 group-hover:scale-110 transition-transform duration-300">
                <Settings class="w-7 h-7 text-guofeng-jade" />
              </div>
              <div class="flex-1">
                <h3 class="text-xl font-bold text-guofeng-text-primary mb-2 group-hover:text-guofeng-jade transition-colors">
                  {{ $t('claudeCode.modules.configManagement.title') }}
                </h3>
                <p class="text-sm text-guofeng-text-secondary mb-3 leading-relaxed">
                  {{ $t('claudeCode.modules.configManagement.desc') }}
                </p>
                <span class="inline-flex items-center px-2.5 py-1 rounded-md text-xs font-bold bg-guofeng-jade/10 text-guofeng-jade">
                  🔥 {{ $t('claudeCode.modules.configManagement.badge') }}
                </span>
              </div>
            </div>
          </GuofengCard>
        </RouterLink>

        <RouterLink
          to="/sync"
          class="block group"
        >
          <GuofengCard
            variant="glass"
            interactive
            class="h-full p-6 border-l-4 border-l-guofeng-indigo"
          >
            <div class="flex items-start gap-4">
              <div class="w-14 h-14 rounded-2xl bg-guofeng-indigo/10 flex items-center justify-center shrink-0 group-hover:scale-110 transition-transform duration-300">
                <Cloud class="w-7 h-7 text-guofeng-indigo" />
              </div>
              <div class="flex-1">
                <h3 class="text-xl font-bold text-guofeng-text-primary mb-2 group-hover:text-guofeng-indigo transition-colors">
                  {{ $t('claudeCode.modules.cloudSync.title') }}
                </h3>
                <p class="text-sm text-guofeng-text-secondary mb-3 leading-relaxed">
                  {{ $t('claudeCode.modules.cloudSync.desc') }}
                </p>
                <span class="inline-flex items-center px-2.5 py-1 rounded-md text-xs font-bold bg-guofeng-indigo/10 text-guofeng-indigo">
                  ✨ {{ $t('claudeCode.modules.cloudSync.badge') }}
                </span>
              </div>
            </div>
          </GuofengCard>
        </RouterLink>
      </section>

      <!-- Market Banner -->
      <section
        class="animate-fade-in"
        style="animation-delay: 0.3s"
      >
        <RouterLink
          to="/market"
          class="block group"
        >
          <GuofengCard
            variant="glass"
            class="relative overflow-hidden p-6"
          >
            <div class="absolute top-0 right-0 w-64 h-64 bg-gradient-to-br from-red-500/10 to-orange-500/10 rounded-full blur-3xl -mr-16 -mt-16" />
            
            <div class="relative z-10 flex items-center justify-between">
              <div class="flex items-center gap-5">
                <div class="p-4 bg-gradient-to-br from-red-500 to-orange-500 rounded-2xl shadow-lg text-white group-hover:scale-110 transition-transform duration-300">
                  <ShoppingBag class="w-7 h-7" />
                </div>
                <div>
                  <h2 class="text-2xl font-bold text-guofeng-text-primary mb-1 flex items-center gap-3">
                    {{ $t('market.title') }}
                    <span class="px-2 py-0.5 text-xs bg-red-100 text-red-600 rounded-full border border-red-200">Beta</span>
                  </h2>
                  <p class="text-guofeng-text-secondary">
                    {{ $t('market.description') }}
                  </p>
                </div>
              </div>
              <div class="hidden md:flex items-center gap-2 text-guofeng-text-muted font-medium">
                <span>{{ $t('market.explore') }}</span>
                <ExternalLink class="w-5 h-5 group-hover:translate-x-1 transition-transform" />
              </div>
            </div>
          </GuofengCard>
        </RouterLink>
      </section>

      <!-- 扩展模块网格 (Extensions) -->
      <section
        class="animate-fade-in"
        style="animation-delay: 0.4s"
      >
        <div class="flex items-center gap-2 mb-4 px-1">
          <Boxes class="w-5 h-5 text-guofeng-jade" />
          <h2 class="text-lg font-bold text-guofeng-text-primary">
            {{ $t('claudeCode.modules.extensionsTitle') }}
          </h2>
        </div>
        
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-5">
          <!-- MCP Servers -->
          <RouterLink
            to="/mcp"
            class="block group"
          >
            <GuofengCard
              variant="glass"
              interactive
              pattern
              class="h-full p-5"
            >
              <div class="flex flex-col h-full">
                <div class="flex items-center justify-between mb-4">
                  <div class="w-12 h-12 rounded-xl bg-cyan-500/10 flex items-center justify-center group-hover:scale-110 transition-transform duration-300">
                    <Server class="w-6 h-6 text-cyan-500" />
                  </div>
                  <span class="px-2 py-1 rounded-md text-xs font-bold bg-cyan-500/10 text-cyan-500">MCP</span>
                </div>
                <h3 class="text-base font-bold text-guofeng-text-primary mb-2 group-hover:text-cyan-500 transition-colors">
                  {{ $t('claudeCode.modules.mcpServers.title') }}
                </h3>
                <p class="text-sm text-guofeng-text-secondary leading-relaxed line-clamp-2 flex-grow">
                  {{ $t('claudeCode.modules.mcpServers.desc') }}
                </p>
              </div>
            </GuofengCard>
          </RouterLink>

          <!-- Agents -->
          <RouterLink
            to="/agents"
            class="block group"
          >
            <GuofengCard
              variant="glass"
              interactive
              pattern
              class="h-full p-5"
            >
              <div class="flex flex-col h-full">
                <div class="flex items-center justify-between mb-4">
                  <div class="w-12 h-12 rounded-xl bg-guofeng-jade/10 flex items-center justify-center group-hover:scale-110 transition-transform duration-300">
                    <Users class="w-6 h-6 text-guofeng-jade" />
                  </div>
                  <span class="px-2 py-1 rounded-md text-xs font-bold bg-guofeng-jade/10 text-guofeng-jade">AI</span>
                </div>
                <h3 class="text-base font-bold text-guofeng-text-primary mb-2 group-hover:text-guofeng-jade transition-colors">
                  {{ $t('claudeCode.modules.agents.title') }}
                </h3>
                <p class="text-sm text-guofeng-text-secondary leading-relaxed line-clamp-2 flex-grow">
                  {{ $t('claudeCode.modules.agents.desc') }}
                </p>
              </div>
            </GuofengCard>
          </RouterLink>

          <!-- Skills -->
          <RouterLink
            to="/skills"
            class="block group"
          >
            <GuofengCard
              variant="glass"
              interactive
              pattern
              class="h-full p-5"
            >
              <div class="flex flex-col h-full">
                <div class="flex items-center justify-between mb-4">
                  <div class="w-12 h-12 rounded-xl bg-red-500/10 flex items-center justify-center group-hover:scale-110 transition-transform duration-300">
                    <Book class="w-6 h-6 text-red-500" />
                  </div>
                  <span class="px-2 py-1 rounded-md text-xs font-bold bg-red-500/10 text-red-500">Skill</span>
                </div>
                <h3 class="text-base font-bold text-guofeng-text-primary mb-2 group-hover:text-red-500 transition-colors">
                  {{ $t('claudeCode.modules.skills.title') }}
                </h3>
                <p class="text-sm text-guofeng-text-secondary leading-relaxed line-clamp-2 flex-grow">
                  {{ $t('claudeCode.modules.skills.desc') }}
                </p>
              </div>
            </GuofengCard>
          </RouterLink>

          <!-- Plugins -->
          <RouterLink
            to="/plugins"
            class="block group"
          >
            <GuofengCard
              variant="glass"
              interactive
              pattern
              class="h-full p-5"
            >
              <div class="flex flex-col h-full">
                <div class="flex items-center justify-between mb-4">
                  <div class="w-12 h-12 rounded-xl bg-blue-500/10 flex items-center justify-center group-hover:scale-110 transition-transform duration-300">
                    <Puzzle class="w-6 h-6 text-blue-500" />
                  </div>
                  <span class="px-2 py-1 rounded-md text-xs font-bold bg-blue-500/10 text-blue-500">Plugin</span>
                </div>
                <h3 class="text-base font-bold text-guofeng-text-primary mb-2 group-hover:text-blue-500 transition-colors">
                  {{ $t('claudeCode.modules.plugins.title') }}
                </h3>
                <p class="text-sm text-guofeng-text-secondary leading-relaxed line-clamp-2 flex-grow">
                  {{ $t('claudeCode.modules.plugins.desc') }}
                </p>
              </div>
            </GuofengCard>
          </RouterLink>

          <!-- Slash Commands -->
          <RouterLink
            to="/slash-commands"
            class="block group"
          >
            <GuofengCard
              variant="glass"
              interactive
              pattern
              class="h-full p-5"
            >
              <div class="flex flex-col h-full">
                <div class="flex items-center justify-between mb-4">
                  <div class="w-12 h-12 rounded-xl bg-amber-500/10 flex items-center justify-center group-hover:scale-110 transition-transform duration-300">
                    <Terminal class="w-6 h-6 text-amber-500" />
                  </div>
                  <span class="px-2 py-1 rounded-md text-xs font-bold bg-amber-500/10 text-amber-500">CMD</span>
                </div>
                <h3 class="text-base font-bold text-guofeng-text-primary mb-2 group-hover:text-amber-500 transition-colors">
                  {{ $t('claudeCode.modules.slashCommands.title') }}
                </h3>
                <p class="text-sm text-guofeng-text-secondary leading-relaxed line-clamp-2 flex-grow">
                  {{ $t('claudeCode.modules.slashCommands.desc') }}
                </p>
              </div>
            </GuofengCard>
          </RouterLink>
        </div>
      </section>

      <!-- 快速操作区 -->
      <section
        class="grid grid-cols-1 md:grid-cols-2 gap-5 animate-fade-in"
        style="animation-delay: 0.5s"
      >
        <!-- 常用命令 -->
        <GuofengCard
          variant="glass"
          class="p-6"
        >
          <h3 class="text-lg font-bold text-guofeng-text-primary mb-4 flex items-center gap-2">
            <Terminal class="w-5 h-5 text-guofeng-jade" />
            {{ $t('claudeCode.quickActions.commonCommands') }}
          </h3>
          <div class="space-y-2">
            <div class="flex items-center justify-between p-3 rounded-xl bg-guofeng-bg-secondary/50 hover:bg-guofeng-bg-tertiary/50 transition-colors cursor-pointer group">
              <span class="text-sm font-medium text-guofeng-text-secondary">{{ $t('claudeCode.quickActions.viewCurrentConfig') }}</span>
              <code class="text-xs font-mono px-2 py-1 rounded bg-guofeng-jade/10 text-guofeng-jade group-hover:bg-guofeng-jade group-hover:text-white transition-colors">ccr current</code>
            </div>
            <div class="flex items-center justify-between p-3 rounded-xl bg-guofeng-bg-secondary/50 hover:bg-guofeng-bg-tertiary/50 transition-colors cursor-pointer group">
              <span class="text-sm font-medium text-guofeng-text-secondary">{{ $t('claudeCode.quickActions.switchConfig') }}</span>
              <code class="text-xs font-mono px-2 py-1 rounded bg-guofeng-indigo/10 text-guofeng-indigo group-hover:bg-guofeng-indigo group-hover:text-white transition-colors">ccr switch</code>
            </div>
            <div class="flex items-center justify-between p-3 rounded-xl bg-guofeng-bg-secondary/50 hover:bg-guofeng-bg-tertiary/50 transition-colors cursor-pointer group">
              <span class="text-sm font-medium text-guofeng-text-secondary">{{ $t('claudeCode.quickActions.listAllConfigs') }}</span>
              <code class="text-xs font-mono px-2 py-1 rounded bg-cyan-500/10 text-cyan-500 group-hover:bg-cyan-500 group-hover:text-white transition-colors">ccr list</code>
            </div>
          </div>
        </GuofengCard>

        <!-- 文档资源 -->
        <GuofengCard
          variant="glass"
          class="p-6"
        >
          <h3 class="text-lg font-bold text-guofeng-text-primary mb-4 flex items-center gap-2">
            <BookOpen class="w-5 h-5 text-guofeng-jade" />
            {{ $t('claudeCode.quickActions.resources') }}
          </h3>
          <div class="space-y-2">
            <a
              href="https://docs.anthropic.com/claude/docs"
              target="_blank"
              class="flex items-center justify-between p-3 rounded-xl bg-guofeng-bg-secondary/50 hover:bg-guofeng-bg-tertiary/50 transition-colors group"
            >
              <span class="text-sm font-medium text-guofeng-text-secondary">{{ $t('claudeCode.quickActions.officialDocs') }}</span>
              <ExternalLink class="w-4 h-4 text-guofeng-jade group-hover:translate-x-1 transition-transform" />
            </a>
            <a
              href="https://modelcontextprotocol.io"
              target="_blank"
              class="flex items-center justify-between p-3 rounded-xl bg-guofeng-bg-secondary/50 hover:bg-guofeng-bg-tertiary/50 transition-colors group"
            >
              <span class="text-sm font-medium text-guofeng-text-secondary">{{ $t('claudeCode.quickActions.mcpProtocol') }}</span>
              <ExternalLink class="w-4 h-4 text-guofeng-jade group-hover:translate-x-1 transition-transform" />
            </a>
            <a
              href="https://github.com/anthropics/anthropic-sdk-typescript"
              target="_blank"
              class="flex items-center justify-between p-3 rounded-xl bg-guofeng-bg-secondary/50 hover:bg-guofeng-bg-tertiary/50 transition-colors group"
            >
              <span class="text-sm font-medium text-guofeng-text-secondary">{{ $t('claudeCode.quickActions.sdkRepo') }}</span>
              <ExternalLink class="w-4 h-4 text-guofeng-jade group-hover:translate-x-1 transition-transform" />
            </a>
          </div>
        </GuofengCard>
      </section>

      <!-- 提示卡片 -->
      <section
        class="animate-fade-in"
        style="animation-delay: 0.6s"
      >
        <GuofengCard
          variant="glass"
          class="p-6"
        >
          <div class="flex items-start gap-4">
            <div class="w-12 h-12 rounded-xl bg-blue-500/10 flex items-center justify-center shrink-0">
              <Info class="w-6 h-6 text-blue-500" />
            </div>
            <div class="flex-1">
              <h3 class="text-lg font-bold text-guofeng-text-primary mb-3">
                💡 {{ $t('claudeCode.tips.title') }}
              </h3>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                <div class="flex items-start gap-2 p-3 rounded-lg bg-guofeng-bg-secondary/30 border border-guofeng-border/30">
                  <span class="text-guofeng-jade font-bold">1.</span>
                  <span class="text-sm text-guofeng-text-secondary">{{ $t('claudeCode.tips.tip1') }}</span>
                </div>
                <div class="flex items-start gap-2 p-3 rounded-lg bg-guofeng-bg-secondary/30 border border-guofeng-border/30">
                  <span class="text-guofeng-indigo font-bold">2.</span>
                  <span class="text-sm text-guofeng-text-secondary">{{ $t('claudeCode.tips.tip2') }}</span>
                </div>
                <div class="flex items-start gap-2 p-3 rounded-lg bg-guofeng-bg-secondary/30 border border-guofeng-border/30">
                  <span class="text-cyan-500 font-bold">3.</span>
                  <span class="text-sm text-guofeng-text-secondary">{{ $t('claudeCode.tips.tip3') }}</span>
                </div>
                <div class="flex items-start gap-2 p-3 rounded-lg bg-guofeng-bg-secondary/30 border border-guofeng-border/30">
                  <span class="text-amber-500 font-bold">4.</span>
                  <span class="text-sm text-guofeng-text-secondary">{{ $t('claudeCode.tips.tip4') }}</span>
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
import { RouterLink } from 'vue-router'
import { 
  Settings, 
  Cloud, 
  Server, 
  Terminal, 
  Users, 
  Puzzle,
  Home,
  Code2,
  Boxes,
  BookOpen,
  ExternalLink,
  Info,
  Book,
  ShoppingBag
} from 'lucide-vue-next'
import GuofengCard from '@/components/common/GuofengCard.vue'
</script>