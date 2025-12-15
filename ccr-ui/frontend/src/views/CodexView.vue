<template>
  <div class="min-h-screen relative">
    <!-- 🎨 彩色背景装饰 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-20 right-20 w-96 h-96 rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{ background: 'linear-gradient(135deg, #f59e0b 0%, #ef4444 100%)' }"
      />
      <div
        class="absolute bottom-20 left-20 w-96 h-96 rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)',
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
              :style="{ background: 'rgba(245, 158, 11, 0.1)' }"
            >
              <Code2
                class="w-10 h-10"
                :style="{ color: '#f59e0b' }"
              />
            </div>
            <div>
              <h1 class="text-4xl md:text-5xl font-bold mb-2 bg-gradient-to-r from-[#f59e0b] via-[#ef4444] to-[#ec4899] bg-clip-text text-transparent">
                Codex
              </h1>
              <p
                class="text-lg"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ $t('codex.overview.subtitle') }}
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
            >{{ $t('codex.overview.backHome') }}</span>
          </RouterLink>
        </div>

        <!-- 特性标签 -->
        <div class="flex flex-wrap gap-3 mb-6">
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(139, 92, 246, 0.1)', color: '#8b5cf6' }"
          >
            {{ $t('codex.overview.features.mcpProtocol') }}
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
            :style="{ color: '#f59e0b' }"
          />
          {{ $t('codex.overview.modulesTitle') }}
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          <RouterLink
            to="/codex/mcp"
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
                    {{ $t('codex.overview.modules.mcp.title') }}
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('codex.overview.modules.mcp.description') }}
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(139, 92, 246, 0.15)',
                      color: '#8b5cf6'
                    }"
                  >
                    {{ $t('codex.overview.modules.mcp.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/codex/profiles"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(236, 72, 153, 0.15)' }"
                >
                  <Settings
                    class="w-7 h-7"
                    :style="{ color: '#ec4899' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ $t('codex.overview.modules.profiles.title') }}
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('codex.overview.modules.profiles.description') }}
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(236, 72, 153, 0.15)',
                      color: '#ec4899'
                    }"
                  >
                    {{ $t('codex.overview.modules.profiles.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/codex/slash-commands"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(245, 158, 11, 0.15)' }"
                >
                  <Command
                    class="w-7 h-7"
                    :style="{ color: '#f59e0b' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ $t('codex.overview.modules.slashCommands.title') }}
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('codex.overview.modules.slashCommands.description') }}
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(245, 158, 11, 0.15)',
                      color: '#f59e0b'
                    }"
                  >
                    {{ $t('codex.overview.modules.slashCommands.badge') }}
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
            :style="{ background: 'rgba(245, 158, 11, 0.15)' }"
          >
            <Info
              class="w-6 h-6"
              :style="{ color: '#f59e0b' }"
            />
          </div>
          <div class="flex-1">
            <h3
              class="text-lg font-bold mb-2"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ $t('codex.overview.tipsTitle') }}
            </h3>
            <ul
              class="space-y-2 text-sm"
              :style="{ color: 'var(--text-secondary)' }"
            >
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>{{ $t('codex.overview.tips.mcpConnection') }}</span>
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
import { Settings, Server, Home, Command, Code2, Boxes, Info } from 'lucide-vue-next'
</script>
