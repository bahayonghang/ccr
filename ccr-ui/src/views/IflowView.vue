<template>
  <div class="min-h-full relative overflow-hidden">
    <!-- 🎨 彩色背景装饰 -->
    <!-- Standard Animated Background -->
    <AnimatedBackground
      contained
      variant="mesh"
    />

    <div class="relative z-10 p-6 max-w-7xl mx-auto">
      <!-- Hero 区域 -->
      <div class="mb-8">
        <div class="flex items-center justify-between mb-6">
          <div class="flex items-center gap-4">
            <div
              class="rounded-3xl glass-card bg-accent-success/10 p-4"
            >
              <SIcon
                name="Workflow"
                size="w-10 h-10"
                class="text-accent-success"
              />
            </div>
            <div>
              <h1 class="mb-2 text-4xl font-bold text-text-primary md:text-5xl">
                {{ $t('iflow.overview.title') }}
              </h1>
              <p class="text-lg text-text-secondary">
                {{ $t('iflow.overview.description') }}
              </p>
            </div>
          </div>
          <RouterLink
            to="/"
            class="glass-card flex items-center gap-2 px-5 py-3 text-text-secondary transition-colors duration-300 hover:bg-bg-overlay/70 hover:text-text-primary"
          >
            <SIcon
              name="Home"
              size="w-5 h-5"
              class="text-text-muted"
            />
            <span class="font-medium">{{ $t('iflow.overview.backToHome') }}</span>
          </RouterLink>
        </div>

        <!-- 特性标签 -->
        <div class="flex flex-wrap gap-3 mb-6">
          <span
            class="glass-card rounded-full bg-accent-success/10 px-4 py-2 text-sm font-medium text-accent-success"
          >
            {{ $t('iflow.overview.tags.workflow') }}
          </span>
          <span
            class="glass-card rounded-full bg-accent-info/10 px-4 py-2 text-sm font-medium text-accent-info"
          >
            {{ $t('iflow.overview.tags.orchestration') }}
          </span>
          <span
            class="glass-card rounded-full bg-accent-primary/10 px-4 py-2 text-sm font-medium text-accent-primary"
          >
            {{ $t('iflow.overview.tags.mcp') }}
          </span>
          <span
            class="glass-card rounded-full bg-accent-secondary/10 px-4 py-2 text-sm font-medium text-accent-secondary"
          >
            {{ $t('iflow.overview.tags.agent') }}
          </span>
        </div>
      </div>

      <!-- 功能模块网格 -->
      <div class="mb-8">
        <h2 class="mb-6 flex items-center gap-3 text-2xl font-bold text-text-primary">
          <SIcon
            name="Boxes"
            size="w-7 h-7"
            class="text-accent-success"
          />
          {{ $t('iflow.overview.features') }}
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          <RouterLink
            to="/iflow/mcp"
            class="block group"
          >
            <div class="glass-card h-full p-6 transition-[transform,box-shadow,border-color] duration-300 hover:-translate-y-1 hover:border-white/30">
              <div class="flex items-start gap-4">
                <div class="rounded-2xl bg-accent-success/15 p-3">
                  <SIcon
                    name="Server"
                    size="w-7 h-7"
                    class="text-accent-success"
                  />
                </div>
                <div class="flex-1">
                  <h3 class="mb-2 text-xl font-bold text-text-primary">
                    {{ $t('iflow.mcp.title') }}
                  </h3>
                  <p class="mb-3 text-sm text-text-secondary">
                    {{ $t('iflow.mcp.description') }}
                  </p>
                  <span class="inline-flex items-center rounded-full bg-accent-success/15 px-3 py-1.5 text-xs font-bold text-accent-success">
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
            <div class="glass-card h-full p-6 transition-[transform,box-shadow,border-color] duration-300 hover:-translate-y-1 hover:border-white/30">
              <div class="flex items-start gap-4">
                <div class="rounded-2xl bg-accent-info/15 p-3">
                  <SIcon
                    name="Bot"
                    size="w-7 h-7"
                    class="text-accent-info"
                  />
                </div>
                <div class="flex-1">
                  <h3 class="mb-2 text-xl font-bold text-text-primary">
                    {{ $t('iflow.agents.title') }}
                  </h3>
                  <p class="mb-3 text-sm text-text-secondary">
                    {{ $t('iflow.agents.description') }}
                  </p>
                  <span class="inline-flex items-center rounded-full bg-accent-info/15 px-3 py-1.5 text-xs font-bold text-accent-info">
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
            <div class="glass-card h-full p-6 transition-[transform,box-shadow,border-color] duration-300 hover:-translate-y-1 hover:border-white/30">
              <div class="flex items-start gap-4">
                <div class="rounded-2xl bg-accent-primary/15 p-3">
                  <SIcon
                    name="Puzzle"
                    size="w-7 h-7"
                    class="text-accent-primary"
                  />
                </div>
                <div class="flex-1">
                  <h3 class="mb-2 text-xl font-bold text-text-primary">
                    {{ $t('iflow.plugins.title') }}
                  </h3>
                  <p class="mb-3 text-sm text-text-secondary">
                    {{ $t('iflow.plugins.description') }}
                  </p>
                  <span class="inline-flex items-center rounded-full bg-accent-primary/15 px-3 py-1.5 text-xs font-bold text-accent-primary">
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
            <div class="glass-card h-full p-6 transition-[transform,box-shadow,border-color] duration-300 hover:-translate-y-1 hover:border-white/30">
              <div class="flex items-start gap-4">
                <div class="rounded-2xl bg-accent-secondary/15 p-3">
                  <SIcon
                    name="Command"
                    size="w-7 h-7"
                    class="text-accent-secondary"
                  />
                </div>
                <div class="flex-1">
                  <h3 class="mb-2 text-xl font-bold text-text-primary">
                    {{ $t('iflow.slashCommands.title') }}
                  </h3>
                  <p class="mb-3 text-sm text-text-secondary">
                    {{ $t('iflow.slashCommands.description') }}
                  </p>
                  <span class="inline-flex items-center rounded-full bg-accent-secondary/15 px-3 py-1.5 text-xs font-bold text-accent-secondary">
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
            <SIcon
              name="Info"
              size="w-6 h-6"
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
import SIcon from '@/components/ui/SIcon.vue'
import { RouterLink } from 'vue-router'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
</script>
