<template>
  <div class="min-h-screen relative">
    <!-- 🎨 彩色背景装饰 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-20 right-20 w-96 h-96 rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{ background: 'linear-gradient(135deg, var(--accent-primary) 0%, var(--accent-secondary) 100%)' }"
      />
      <div
        class="absolute bottom-20 left-20 w-96 h-96 rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, var(--accent-tertiary) 0%, var(--accent-warning) 100%)',
          animationDelay: '1s'
        }"
      />
    </div>

    <div class="relative z-10 p-6 max-w-7xl mx-auto">
      <!-- Hero 区域 -->
      <div class="mb-8">
        <div class="flex items-center justify-between mb-6">
          <div class="flex items-center gap-4">
            <div
              class="p-4 rounded-3xl glass-card"
              :style="{ background: 'rgba(99, 102, 241, 0.1)' }"
            >
              <Code2
                class="w-10 h-10"
                :style="{ color: 'var(--accent-primary)' }"
              />
            </div>
            <div>
              <h1 class="text-4xl md:text-5xl font-bold mb-2 brand-gradient-text animate-fade-in">
                Claude Code
              </h1>
              <p
                class="text-lg"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ $t('claudeCode.subtitle') }}
              </p>
            </div>
          </div>
          <RouterLink 
            to="/" 
            class="glass-card flex items-center gap-2 px-5 py-3 hover:scale-105 transition-all duration-300"
          >
            <Home
              class="w-5 h-5"
              :style="{ color: 'var(--text-secondary)' }"
            />
            <span
              class="font-medium"
              :style="{ color: 'var(--text-secondary)' }"
            >{{ $t('claudeCode.backToHome') }}</span>
          </RouterLink>
        </div>

        <!-- 特性标签 -->
        <div class="flex flex-wrap gap-3 mb-6">
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(99, 102, 241, 0.1)', color: 'var(--accent-primary)' }"
          >
            ⚡ {{ $t('claudeCode.features.mcpSupport') }}
          </span>
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(139, 92, 246, 0.1)', color: 'var(--accent-secondary)' }"
          >
            🤖 {{ $t('claudeCode.features.aiAgents') }}
          </span>
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(236, 72, 153, 0.1)', color: 'var(--accent-tertiary)' }"
          >
            ☁️ {{ $t('claudeCode.features.cloudSync') }}
          </span>
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(16, 185, 129, 0.1)', color: 'var(--accent-success)' }"
          >
            🔧 {{ $t('claudeCode.features.configManagement') }}
          </span>
        </div>
      </div>

      <!-- 核心功能 (Config & Sync) -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
        <RouterLink
          to="/configs"
          class="block group"
        >
          <div class="glass-card p-8 h-full hover:scale-[1.02] transition-all duration-300 border-l-4 border-l-indigo-500">
            <div class="flex items-start gap-5">
              <div
                class="p-4 rounded-2xl"
                :style="{ background: 'rgba(99, 102, 241, 0.15)' }"
              >
                <Settings
                  class="w-8 h-8"
                  :style="{ color: 'var(--accent-primary)' }"
                />
              </div>
              <div class="flex-1">
                <h3
                  class="text-2xl font-bold mb-2"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('claudeCode.modules.configManagement.title') }}
                </h3>
                <p
                  class="text-base mb-4"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ $t('claudeCode.modules.configManagement.desc') }}
                </p>
                <span
                  class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                  :style="{
                    background: 'rgba(99, 102, 241, 0.15)',
                    color: 'var(--accent-primary)'
                  }"
                >
                  🔥 {{ $t('claudeCode.modules.configManagement.badge') }}
                </span>
              </div>
            </div>
          </div>
        </RouterLink>

        <RouterLink
          to="/sync"
          class="block group"
        >
          <div class="glass-card p-8 h-full hover:scale-[1.02] transition-all duration-300 border-l-4 border-l-purple-500">
            <div class="flex items-start gap-5">
              <div
                class="p-4 rounded-2xl"
                :style="{ background: 'rgba(139, 92, 246, 0.15)' }"
              >
                <Cloud
                  class="w-8 h-8"
                  :style="{ color: 'var(--accent-secondary)' }"
                />
              </div>
              <div class="flex-1">
                <h3
                  class="text-2xl font-bold mb-2"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('claudeCode.modules.cloudSync.title') }}
                </h3>
                <p
                  class="text-base mb-4"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ $t('claudeCode.modules.cloudSync.desc') }}
                </p>
                <span
                  class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                  :style="{
                    background: 'rgba(139, 92, 246, 0.15)',
                    color: 'var(--accent-secondary)'
                  }"
                >
                  ✨ {{ $t('claudeCode.modules.cloudSync.badge') }}
                </span>
              </div>
            </div>
          </div>
        </RouterLink>
      </div>

      <!-- Market Banner -->
      <div class="mb-8">
        <RouterLink to="/market" class="block group">
          <div class="glass-card p-8 relative overflow-hidden hover:scale-[1.01] transition-all duration-300">
            <div class="absolute top-0 right-0 w-64 h-64 bg-gradient-to-br from-red-500/10 to-orange-500/10 rounded-full blur-3xl -mr-16 -mt-16"></div>
            
            <div class="relative z-10 flex items-center justify-between">
              <div class="flex items-center gap-6">
                <div class="p-4 bg-gradient-to-br from-red-500 to-orange-500 rounded-2xl shadow-lg text-white">
                  <ShoppingBag class="w-8 h-8" />
                </div>
                <div>
                  <h2 class="text-2xl font-bold text-guofeng-text-primary mb-2 flex items-center gap-3">
                    {{ $t('market.title') }}
                    <span class="px-2 py-0.5 text-xs bg-red-100 text-red-600 rounded-full border border-red-200">Beta</span>
                  </h2>
                  <p class="text-guofeng-text-secondary text-lg">
                    {{ $t('market.description') }}
                  </p>
                </div>
              </div>
              <div class="hidden md:flex items-center gap-2 text-guofeng-text-muted font-medium">
                <span>{{ $t('market.explore') }}</span>
                <ExternalLink class="w-5 h-5 group-hover:translate-x-1 transition-transform" />
              </div>
            </div>
          </div>
        </RouterLink>
      </div>

      <!-- 扩展模块网格 (Extensions) -->
      <div class="mb-8">
        <h2
          class="text-xl font-bold mb-6 flex items-center gap-3"
          :style="{ color: 'var(--text-primary)' }"
        >
          <Boxes
            class="w-6 h-6"
            :style="{ color: 'var(--accent-primary)' }"
          />
          {{ $t('claudeCode.modules.extensionsTitle') }}
        </h2>
        
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-5">
          <!-- MCP Servers -->
          <RouterLink
            to="/mcp"
            class="block group"
          >
            <div class="glass-card p-5 h-full hover:scale-105 transition-all duration-300">
              <div class="flex flex-col h-full">
                <div class="flex items-center justify-between mb-4">
                  <div
                    class="p-2.5 rounded-xl"
                    :style="{ background: 'rgba(236, 72, 153, 0.15)' }"
                  >
                    <Server
                      class="w-6 h-6"
                      :style="{ color: 'var(--accent-tertiary)' }"
                    />
                  </div>
                  <span
                    class="px-2 py-1 rounded-md text-xs font-bold"
                    :style="{
                      background: 'rgba(236, 72, 153, 0.1)',
                      color: 'var(--accent-tertiary)'
                    }"
                  >MCP</span>
                </div>
                <h3
                  class="text-lg font-bold mb-2"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('claudeCode.modules.mcpServers.title') }}
                </h3>
                <p
                  class="text-sm line-clamp-2"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ $t('claudeCode.modules.mcpServers.desc') }}
                </p>
              </div>
            </div>
          </RouterLink>

          <!-- Agents -->
          <RouterLink
            to="/agents"
            class="block group"
          >
            <div class="glass-card p-5 h-full hover:scale-105 transition-all duration-300">
              <div class="flex flex-col h-full">
                <div class="flex items-center justify-between mb-4">
                  <div
                    class="p-2.5 rounded-xl"
                    :style="{ background: 'rgba(16, 185, 129, 0.15)' }"
                  >
                    <Users
                      class="w-6 h-6"
                      :style="{ color: 'var(--accent-success)' }"
                    />
                  </div>
                  <span
                    class="px-2 py-1 rounded-md text-xs font-bold"
                    :style="{
                      background: 'rgba(16, 185, 129, 0.1)',
                      color: 'var(--accent-success)'
                    }"
                  >AI</span>
                </div>
                <h3
                  class="text-lg font-bold mb-2"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('claudeCode.modules.agents.title') }}
                </h3>
                <p
                  class="text-sm line-clamp-2"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ $t('claudeCode.modules.agents.desc') }}
                </p>
              </div>
            </div>
          </RouterLink>

          <!-- Skills -->
          <RouterLink
            to="/skills"
            class="block group"
          >
            <div class="glass-card p-5 h-full hover:scale-105 transition-all duration-300">
              <div class="flex flex-col h-full">
                <div class="flex items-center justify-between mb-4">
                  <div
                    class="p-2.5 rounded-xl"
                    :style="{ background: 'rgba(239, 68, 68, 0.15)' }"
                  >
                    <Book
                      class="w-6 h-6"
                      :style="{ color: 'var(--accent-error)' }"
                    />
                  </div>
                  <span
                    class="px-2 py-1 rounded-md text-xs font-bold"
                    :style="{
                      background: 'rgba(239, 68, 68, 0.1)',
                      color: 'var(--accent-error)'
                    }"
                  >Skill</span>
                </div>
                <h3
                  class="text-lg font-bold mb-2"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('claudeCode.modules.skills.title') }}
                </h3>
                <p
                  class="text-sm line-clamp-2"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ $t('claudeCode.modules.skills.desc') }}
                </p>
              </div>
            </div>
          </RouterLink>

          <!-- Plugins -->
          <RouterLink
            to="/plugins"
            class="block group"
          >
            <div class="glass-card p-5 h-full hover:scale-105 transition-all duration-300">
              <div class="flex flex-col h-full">
                <div class="flex items-center justify-between mb-4">
                  <div
                    class="p-2.5 rounded-xl"
                    :style="{ background: 'rgba(59, 130, 246, 0.15)' }"
                  >
                    <Puzzle
                      class="w-6 h-6"
                      :style="{ color: 'var(--accent-info)' }"
                    />
                  </div>
                  <span
                    class="px-2 py-1 rounded-md text-xs font-bold"
                    :style="{
                      background: 'rgba(59, 130, 246, 0.1)',
                      color: 'var(--accent-info)'
                    }"
                  >Plugin</span>
                </div>
                <h3
                  class="text-lg font-bold mb-2"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('claudeCode.modules.plugins.title') }}
                </h3>
                <p
                  class="text-sm line-clamp-2"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ $t('claudeCode.modules.plugins.desc') }}
                </p>
              </div>
            </div>
          </RouterLink>

          <!-- Slash Commands -->
          <RouterLink
            to="/slash-commands"
            class="block group"
          >
            <div class="glass-card p-5 h-full hover:scale-105 transition-all duration-300">
              <div class="flex flex-col h-full">
                <div class="flex items-center justify-between mb-4">
                  <div
                    class="p-2.5 rounded-xl"
                    :style="{ background: 'rgba(245, 158, 11, 0.15)' }"
                  >
                    <Terminal
                      class="w-6 h-6"
                      :style="{ color: 'var(--accent-warning)' }"
                    />
                  </div>
                  <span
                    class="px-2 py-1 rounded-md text-xs font-bold"
                    :style="{
                      background: 'rgba(245, 158, 11, 0.1)',
                      color: 'var(--accent-warning)'
                    }"
                  >CMD</span>
                </div>
                <h3
                  class="text-lg font-bold mb-2"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('claudeCode.modules.slashCommands.title') }}
                </h3>
                <p
                  class="text-sm line-clamp-2"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ $t('claudeCode.modules.slashCommands.desc') }}
                </p>
              </div>
            </div>
          </RouterLink>
        </div>
      </div>

      <!-- 快速操作区 -->
      <div class="mb-8">
        <h2
          class="text-xl font-bold mb-6 flex items-center gap-3"
          :style="{ color: 'var(--text-primary)' }"
        >
          <Zap
            class="w-6 h-6"
            :style="{ color: 'var(--accent-warning)' }"
          />
          {{ $t('claudeCode.quickActions.title') }}
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div class="glass-card p-6">
            <h3
              class="text-lg font-bold mb-4 flex items-center gap-2"
              :style="{ color: 'var(--text-primary)' }"
            >
              <Terminal
                class="w-5 h-5"
                :style="{ color: 'var(--accent-primary)' }"
              />
              {{ $t('claudeCode.quickActions.commonCommands') }}
            </h3>
            <div class="space-y-3">
              <div
                class="flex items-center justify-between p-3 rounded-xl hover:scale-105 transition-all cursor-pointer"
                :style="{ background: 'rgba(99, 102, 241, 0.05)' }"
              >
                <span
                  class="text-sm font-medium"
                  :style="{ color: 'var(--text-secondary)' }"
                >{{ $t('claudeCode.quickActions.viewCurrentConfig') }}</span>
                <code
                  class="text-xs font-mono px-2 py-1 rounded"
                  :style="{ background: 'rgba(99, 102, 241, 0.1)', color: 'var(--accent-primary)' }"
                >ccr current</code>
              </div>
              <div
                class="flex items-center justify-between p-3 rounded-xl hover:scale-105 transition-all cursor-pointer"
                :style="{ background: 'rgba(139, 92, 246, 0.05)' }"
              >
                <span
                  class="text-sm font-medium"
                  :style="{ color: 'var(--text-secondary)' }"
                >{{ $t('claudeCode.quickActions.switchConfig') }}</span>
                <code
                  class="text-xs font-mono px-2 py-1 rounded"
                  :style="{ background: 'rgba(139, 92, 246, 0.1)', color: 'var(--accent-secondary)' }"
                >ccr switch</code>
              </div>
              <div
                class="flex items-center justify-between p-3 rounded-xl hover:scale-105 transition-all cursor-pointer"
                :style="{ background: 'rgba(236, 72, 153, 0.05)' }"
              >
                <span
                  class="text-sm font-medium"
                  :style="{ color: 'var(--text-secondary)' }"
                >{{ $t('claudeCode.quickActions.listAllConfigs') }}</span>
                <code
                  class="text-xs font-mono px-2 py-1 rounded"
                  :style="{ background: 'rgba(236, 72, 153, 0.1)', color: 'var(--accent-tertiary)' }"
                >ccr list</code>
              </div>
            </div>
          </div>

          <div class="glass-card p-6">
            <h3
              class="text-lg font-bold mb-4 flex items-center gap-2"
              :style="{ color: 'var(--text-primary)' }"
            >
              <BookOpen
                class="w-5 h-5"
                :style="{ color: 'var(--accent-success)' }"
              />
              {{ $t('claudeCode.quickActions.resources') }}
            </h3>
            <div class="space-y-3">
              <a
                href="https://docs.anthropic.com/claude/docs"
                target="_blank"
                class="flex items-center justify-between p-3 rounded-xl hover:scale-105 transition-all"
                :style="{ background: 'rgba(16, 185, 129, 0.05)' }"
              >
                <span
                  class="text-sm font-medium"
                  :style="{ color: 'var(--text-secondary)' }"
                >{{ $t('claudeCode.quickActions.officialDocs') }}</span>
                <ExternalLink
                  class="w-4 h-4"
                  :style="{ color: 'var(--accent-success)' }"
                />
              </a>
              <a
                href="https://modelcontextprotocol.io"
                target="_blank"
                class="flex items-center justify-between p-3 rounded-xl hover:scale-105 transition-all"
                :style="{ background: 'rgba(59, 130, 246, 0.05)' }"
              >
                <span
                  class="text-sm font-medium"
                  :style="{ color: 'var(--text-secondary)' }"
                >{{ $t('claudeCode.quickActions.mcpProtocol') }}</span>
                <ExternalLink
                  class="w-4 h-4"
                  :style="{ color: 'var(--accent-info)' }"
                />
              </a>
              <a
                href="https://github.com/anthropics/anthropic-sdk-typescript"
                target="_blank"
                class="flex items-center justify-between p-3 rounded-xl hover:scale-105 transition-all"
                :style="{ background: 'rgba(245, 158, 11, 0.05)' }"
              >
                <span
                  class="text-sm font-medium"
                  :style="{ color: 'var(--text-secondary)' }"
                >{{ $t('claudeCode.quickActions.sdkRepo') }}</span>
                <ExternalLink
                  class="w-4 h-4"
                  :style="{ color: 'var(--accent-warning)' }"
                />
              </a>
            </div>
          </div>
        </div>
      </div>

      <!-- 提示卡片 -->
      <div class="glass-card p-6">
        <div class="flex items-start gap-4">
          <div
            class="p-3 rounded-2xl"
            :style="{ background: 'rgba(59, 130, 246, 0.15)' }"
          >
            <Info
              class="w-6 h-6"
              :style="{ color: 'var(--accent-info)' }"
            />
          </div>
          <div class="flex-1">
            <h3
              class="text-lg font-bold mb-2"
              :style="{ color: 'var(--text-primary)' }"
            >
              💡 {{ $t('claudeCode.tips.title') }}
            </h3>
            <ul
              class="space-y-2 text-sm"
              :style="{ color: 'var(--text-secondary)' }"
            >
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>{{ $t('claudeCode.tips.tip1') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>{{ $t('claudeCode.tips.tip2') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>{{ $t('claudeCode.tips.tip3') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>{{ $t('claudeCode.tips.tip4') }}</span>
              </li>
            </ul>
          </div>
        </div>
      </div>
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
  Zap,
  BookOpen,
  ExternalLink,
  Info,
  Book,
  ShoppingBag
} from 'lucide-vue-next'
</script>