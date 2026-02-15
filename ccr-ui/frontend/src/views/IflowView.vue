<template>
  <div class="min-h-screen relative">
    <!-- 🎨 彩色背景装饰 -->
    <!-- Standard Animated Background -->
    <AnimatedBackground variant="mesh" />

    <div class="relative z-10 p-6 max-w-7xl mx-auto">
      <!-- Hero 区域 -->
      <div class="mb-8">
        <div class="flex items-center justify-between mb-6">
          <div class="flex items-center gap-4">
            <div
              class="p-4 rounded-3xl glass-card"
              :style="{ background: 'rgba(16, 185, 129, 0.1)' }"
            >
              <Workflow
                class="w-10 h-10"
                :style="{ color: '#10b981' }"
              />
            </div>
            <div>
              <h1 class="text-4xl md:text-5xl font-bold mb-2 bg-gradient-to-r from-[#10b981] via-[#06b6d4] to-[#3b82f6] bg-clip-text text-transparent">
                {{ $t('iflow.overview.title') }}
              </h1>
              <p
                class="text-lg"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ $t('iflow.overview.description') }}
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
            >{{ $t('iflow.overview.backToHome') }}</span>
          </RouterLink>
        </div>

        <!-- 特性标签 -->
        <div class="flex flex-wrap gap-3 mb-6">
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(16, 185, 129, 0.1)', color: '#10b981' }"
          >
            {{ $t('iflow.overview.tags.workflow') }}
          </span>
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(6, 182, 212, 0.1)', color: '#06b6d4' }"
          >
            {{ $t('iflow.overview.tags.orchestration') }}
          </span>
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(59, 130, 246, 0.1)', color: '#3b82f6' }"
          >
            {{ $t('iflow.overview.tags.mcp') }}
          </span>
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(139, 92, 246, 0.1)', color: '#8b5cf6' }"
          >
            {{ $t('iflow.overview.tags.agent') }}
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
            :style="{ color: '#10b981' }"
          />
          {{ $t('iflow.overview.features') }}
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          <RouterLink
            to="/iflow/mcp"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(16, 185, 129, 0.15)' }"
                >
                  <Server
                    class="w-7 h-7"
                    :style="{ color: '#10b981' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ $t('iflow.mcp.title') }}
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('iflow.mcp.description') }}
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(16, 185, 129, 0.15)',
                      color: '#10b981'
                    }"
                  >
                    {{ $t('iflow.mcp.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/iflow/agents"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(6, 182, 212, 0.15)' }"
                >
                  <Bot
                    class="w-7 h-7"
                    :style="{ color: '#06b6d4' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ $t('iflow.agents.title') }}
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('iflow.agents.description') }}
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(6, 182, 212, 0.15)',
                      color: '#06b6d4'
                    }"
                  >
                    {{ $t('iflow.agents.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/iflow/plugins"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(59, 130, 246, 0.15)' }"
                >
                  <Puzzle
                    class="w-7 h-7"
                    :style="{ color: '#3b82f6' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ $t('iflow.plugins.title') }}
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('iflow.plugins.description') }}
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(59, 130, 246, 0.15)',
                      color: '#3b82f6'
                    }"
                  >
                    {{ $t('iflow.plugins.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/iflow/slash-commands"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(139, 92, 246, 0.15)' }"
                >
                  <Command
                    class="w-7 h-7"
                    :style="{ color: '#8b5cf6' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ $t('iflow.slashCommands.title') }}
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('iflow.slashCommands.description') }}
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(139, 92, 246, 0.15)',
                      color: '#8b5cf6'
                    }"
                  >
                    {{ $t('iflow.slashCommands.badge') }}
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
            :style="{ background: 'rgba(16, 185, 129, 0.15)' }"
          >
            <Info
              class="w-6 h-6"
              :style="{ color: '#10b981' }"
            />
          </div>
          <div class="flex-1">
            <h3
              class="text-lg font-bold mb-2"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ $t('iflow.overview.featureTitle') }}
            </h3>
            <ul
              class="space-y-2 text-sm"
              :style="{ color: 'var(--text-secondary)' }"
            >
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>{{ $t('iflow.overview.feature1') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>{{ $t('iflow.overview.feature2') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>{{ $t('iflow.overview.feature3') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>{{ $t('iflow.overview.feature4') }}</span>
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
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import { Server, Bot, Puzzle, Command, Home, Workflow, Boxes, Info } from 'lucide-vue-next'
</script>