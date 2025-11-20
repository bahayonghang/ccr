<template>
  <div class="min-h-screen relative">
    <!-- 🎨 彩色背景装饰 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-20 right-20 w-96 h-96 rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{ background: 'linear-gradient(135deg, #8b5cf6 0%, #ec4899 100%)' }"
      />
      <div
        class="absolute bottom-20 left-20 w-96 h-96 rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, #f59e0b 0%, #ef4444 100%)',
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
              :style="{ background: 'rgba(139, 92, 246, 0.1)' }"
            >
              <Zap
                class="w-10 h-10"
                :style="{ color: '#8b5cf6' }"
              />
            </div>
            <div>
              <h1 class="text-4xl md:text-5xl font-bold mb-2 bg-gradient-to-r from-[#8b5cf6] via-[#ec4899] to-[#f59e0b] bg-clip-text text-transparent">
                {{ $t('qwen.overview.title') }}
              </h1>
              <p
                class="text-lg"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ $t('qwen.overview.description') }}
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
            >{{ $t('qwen.overview.backToHome') }}</span>
          </RouterLink>
        </div>

        <!-- 特性标签 -->
        <div class="flex flex-wrap gap-3 mb-6">
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(139, 92, 246, 0.1)', color: '#8b5cf6' }"
          >
            {{ $t('qwen.overview.tags.model') }}
          </span>
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(236, 72, 153, 0.1)', color: '#ec4899' }"
          >
            {{ $t('qwen.overview.tags.dialog') }}
          </span>
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(245, 158, 11, 0.1)', color: '#f59e0b' }"
          >
            {{ $t('qwen.overview.tags.mcp') }}
          </span>
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(239, 68, 68, 0.1)', color: '#ef4444' }"
          >
            {{ $t('qwen.overview.tags.chinese') }}
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
            :style="{ color: '#8b5cf6' }"
          />
          {{ $t('qwen.overview.features') }}
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          <RouterLink
            to="/qwen/mcp"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(139, 92, 246, 0.15)' }"
                >
                  <Server
                    class="w-7 h-7"
                    :style="{ color: '#8b5cf6' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ $t('qwen.mcp.title') }}
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('qwen.mcp.description') }}
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(139, 92, 246, 0.15)',
                      color: '#8b5cf6'
                    }"
                  >
                    {{ $t('qwen.mcp.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/qwen/agents"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(236, 72, 153, 0.15)' }"
                >
                  <Bot
                    class="w-7 h-7"
                    :style="{ color: '#ec4899' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ $t('qwen.agents.title') }}
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('qwen.agents.description') }}
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(236, 72, 153, 0.15)',
                      color: '#ec4899'
                    }"
                  >
                    {{ $t('qwen.agents.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/qwen/plugins"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(245, 158, 11, 0.15)' }"
                >
                  <Puzzle
                    class="w-7 h-7"
                    :style="{ color: '#f59e0b' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ $t('qwen.plugins.title') }}
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('qwen.plugins.description') }}
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(245, 158, 11, 0.15)',
                      color: '#f59e0b'
                    }"
                  >
                    {{ $t('qwen.plugins.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/qwen/slash-commands"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(239, 68, 68, 0.15)' }"
                >
                  <Command
                    class="w-7 h-7"
                    :style="{ color: '#ef4444' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ $t('qwen.slashCommands.title') }}
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('qwen.slashCommands.description') }}
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(239, 68, 68, 0.15)',
                      color: '#ef4444'
                    }"
                  >
                    {{ $t('qwen.slashCommands.badge') }}
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
            :style="{ background: 'rgba(139, 92, 246, 0.15)' }"
          >
            <Info
              class="w-6 h-6"
              :style="{ color: '#8b5cf6' }"
            />
          </div>
          <div class="flex-1">
            <h3
              class="text-lg font-bold mb-2"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ $t('qwen.overview.featureTitle') }}
            </h3>
            <ul
              class="space-y-2 text-sm"
              :style="{ color: 'var(--text-secondary)' }"
            >
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>{{ $t('qwen.overview.feature1') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>{{ $t('qwen.overview.feature2') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>{{ $t('qwen.overview.feature3') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>{{ $t('qwen.overview.feature4') }}</span>
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
import { Server, Bot, Puzzle, Command, Home, Zap, Boxes, Info } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>