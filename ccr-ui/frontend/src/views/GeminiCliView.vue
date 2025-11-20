<template>
  <div class="min-h-screen relative">
    <!-- 🎨 彩色背景装饰 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-20 right-20 w-96 h-96 rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{ background: 'linear-gradient(135deg, #3b82f6 0%, #06b6d4 100%)' }"
      />
      <div
        class="absolute bottom-20 left-20 w-96 h-96 rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, #10b981 0%, #3b82f6 100%)',
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
              :style="{ background: 'rgba(59, 130, 246, 0.1)' }"
            >
              <Sparkles
                class="w-10 h-10"
                :style="{ color: '#3b82f6' }"
              />
            </div>
            <div>
              <h1 class="text-4xl md:text-5xl font-bold mb-2 bg-gradient-to-r from-[#3b82f6] via-[#06b6d4] to-[#10b981] bg-clip-text text-transparent">
                Gemini CLI
              </h1>
              <p
                class="text-lg"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ $t('gemini.overview.description') }}
              </p>
            </div>
          </div>
          <RouterLink 
            to="/" 
            class="glass-card flex items-center gap-2 px-5 py-3 hover:scale-105 transition-all duration-300"
          >
            <Home
              class="w-5 h-5"
              :style="{ color: '#64748b' }"
            />
            <span
              class="font-medium"
              :style="{ color: 'var(--text-secondary)' }"
            >{{ $t('common.backToHome') }}</span>
          </RouterLink>
        </div>

        <!-- 特性标签 -->
        <div class="flex flex-wrap gap-3 mb-6">
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(59, 130, 246, 0.1)', color: '#3b82f6' }"
          >
            🔌 MCP 服务器
          </span>
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(16, 185, 129, 0.1)', color: '#10b981' }"
          >
            🤖 智能代理
          </span>
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(139, 92, 246, 0.1)', color: '#8b5cf6' }"
          >
            🧩 插件系统
          </span>
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(6, 182, 212, 0.1)', color: '#06b6d4' }"
          >
            🌟 多模态
          </span>
        </div>
      </div>

      <!-- 功能模块网格 -->
      <div class="mb-8">
        <h2
          class="text-2xl font-bold mb-6 flex items-center gap-3"
          :style="{ color: 'var(--text-primary)' }"
        >
          <Boxes
            class="w-7 h-7"
            :style="{ color: '#3b82f6' }"
          />
          {{ $t('common.modules') }}
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          <RouterLink
            to="/gemini-cli/mcp"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(59, 130, 246, 0.15)' }"
                >
                  <Server
                    class="w-7 h-7"
                    :style="{ color: '#3b82f6' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ $t('gemini.mcp.title') }}
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('gemini.mcp.subtitle') }}
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(59, 130, 246, 0.15)',
                      color: '#3b82f6'
                    }"
                  >
                    🔌 MCP
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/gemini-cli/agents"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(16, 185, 129, 0.15)' }"
                >
                  <Bot
                    class="w-7 h-7"
                    :style="{ color: '#10b981' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ $t('gemini.agents.title') }}
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('gemini.agents.subtitle') }}
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(16, 185, 129, 0.15)',
                      color: '#10b981'
                    }"
                  >
                    🤖 AI
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/gemini-cli/plugins"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(139, 92, 246, 0.15)' }"
                >
                  <Puzzle
                    class="w-7 h-7"
                    :style="{ color: '#8b5cf6' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ $t('gemini.plugins.title') }}
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('gemini.plugins.subtitle') }}
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(139, 92, 246, 0.15)',
                      color: '#8b5cf6'
                    }"
                  >
                    🧩 插件
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/gemini-cli/slash-commands"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(6, 182, 212, 0.15)' }"
                >
                  <Command
                    class="w-7 h-7"
                    :style="{ color: '#06b6d4' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ $t('gemini.slashCommands.title') }}
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('gemini.slashCommands.subtitle') }}
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(6, 182, 212, 0.15)',
                      color: '#06b6d4'
                    }"
                  >
                    ⌨️ 命令
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>
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
              :style="{ color: '#3b82f6' }"
            />
          </div>
          <div class="flex-1">
            <h3
              class="text-lg font-bold mb-2"
              :style="{ color: 'var(--text-primary)' }"
            >
              💡 {{ $t('gemini.overview.welcome') }}
            </h3>
            <ul
              class="space-y-2 text-sm"
              :style="{ color: 'var(--text-secondary)' }"
            >
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>{{ $t('gemini.overview.feature1') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>{{ $t('gemini.overview.feature2') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>{{ $t('gemini.overview.feature3') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>{{ $t('gemini.overview.feature4') }}</span>
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
import { useI18n } from 'vue-i18n'
import { Server, Bot, Puzzle, Command, Home, Sparkles, Boxes, Info } from 'lucide-vue-next'

const { t } = useI18n()
</script>