<template>
  <div class="min-h-full p-6 lg:p-10 relative overflow-hidden">
    <!-- Background Mesh -->
    <!-- Standard Animated Background -->
    <AnimatedBackground variant="complex" />

    <div class="max-w-7xl mx-auto space-y-8">
      <!-- HEADER -->
      <section class="animate-slide-up">
        <div class="flex flex-col md:flex-row md:items-center justify-between gap-6 mb-8">
          <div class="flex items-center gap-5">
            <div class="w-16 h-16 rounded-2xl bg-purple-500/10 flex items-center justify-center border border-purple-500/20 shadow-lg backdrop-blur-md">
              <Zap class="w-8 h-8 text-purple-600" />
            </div>
            <div>
              <h1 class="text-4xl font-bold font-display bg-gradient-to-r from-purple-500 via-pink-500 to-amber-500 bg-clip-text text-transparent tracking-tight">
                {{ $t('qwen.overview.title') }}
              </h1>
              <p class="text-text-secondary text-lg mt-1 max-w-xl">
                {{ $t('qwen.overview.description') }}
              </p>
            </div>
          </div>
          
          <RouterLink to="/">
            <Button
              variant="glass"
              class="gap-2"
            >
              <Home class="w-4 h-4" />
              {{ $t('qwen.overview.backToHome') }}
            </Button>
          </RouterLink>
        </div>

        <!-- Tags -->
        <div class="flex flex-wrap gap-3">
          <span class="px-3 py-1.5 rounded-full text-xs font-bold uppercase tracking-wider bg-purple-500/10 text-purple-500 border border-purple-500/20 flex items-center gap-2">
            <Bot class="w-3 h-3" /> {{ $t('qwen.overview.tags.model') }}
          </span>
          <span class="px-3 py-1.5 rounded-full text-xs font-bold uppercase tracking-wider bg-pink-500/10 text-pink-500 border border-pink-500/20 flex items-center gap-2">
            <MessageSquare class="w-3 h-3" /> {{ $t('qwen.overview.tags.dialog') }}
          </span>
          <span class="px-3 py-1.5 rounded-full text-xs font-bold uppercase tracking-wider bg-amber-500/10 text-amber-500 border border-amber-500/20 flex items-center gap-2">
            <Server class="w-3 h-3" /> {{ $t('qwen.overview.tags.mcp') }}
          </span>
          <span class="px-3 py-1.5 rounded-full text-xs font-bold uppercase tracking-wider bg-red-500/10 text-red-500 border border-red-500/20 flex items-center gap-2">
            <Globe class="w-3 h-3" /> {{ $t('qwen.overview.tags.chinese') }}
          </span>
        </div>
      </section>

      <!-- MODULES -->
      <section
        class="animate-slide-up"
        style="animation-delay: 100ms"
      >
        <div class="flex items-center gap-3 mb-6">
          <Boxes class="w-5 h-5 text-purple-500" />
          <h2 class="text-lg font-bold uppercase tracking-widest text-text-muted">
            {{ $t('qwen.overview.features') }}
          </h2>
          <div class="h-px flex-1 bg-border-subtle" />
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          <!-- MCP -->
          <RouterLink
            to="/qwen/mcp"
            class="group h-full"
          >
            <Card
              variant="glass"
              hover
              glow
              class="h-full p-6 flex items-start gap-4"
            >
              <div class="p-3 rounded-xl bg-purple-500/10 text-purple-500 group-hover:scale-110 transition-transform duration-300">
                <Server class="w-6 h-6" />
              </div>
              <div>
                <h3 class="text-lg font-bold text-text-primary mb-1 group-hover:text-purple-500 transition-colors">
                  {{ $t('qwen.mcp.title') }}
                </h3>
                <p class="text-sm text-text-secondary mb-3 leading-relaxed">
                  {{ $t('qwen.mcp.description') }}
                </p>
                <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-1 rounded bg-purple-500/10 text-purple-500 border border-purple-500/20">{{ $t('qwen.mcp.badge') }}</span>
              </div>
            </Card>
          </RouterLink>

          <!-- Agents -->
          <RouterLink
            to="/qwen/agents"
            class="group h-full"
          >
            <Card
              variant="glass"
              hover
              glow
              class="h-full p-6 flex items-start gap-4"
            >
              <div class="p-3 rounded-xl bg-pink-500/10 text-pink-500 group-hover:scale-110 transition-transform duration-300">
                <Bot class="w-6 h-6" />
              </div>
              <div>
                <h3 class="text-lg font-bold text-text-primary mb-1 group-hover:text-pink-500 transition-colors">
                  {{ $t('qwen.agents.title') }}
                </h3>
                <p class="text-sm text-text-secondary mb-3 leading-relaxed">
                  {{ $t('qwen.agents.description') }}
                </p>
                <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-1 rounded bg-pink-500/10 text-pink-500 border border-pink-500/20">{{ $t('qwen.agents.badge') }}</span>
              </div>
            </Card>
          </RouterLink>

          <!-- Plugins -->
          <RouterLink
            to="/qwen/plugins"
            class="group h-full"
          >
            <Card
              variant="glass"
              hover
              glow
              class="h-full p-6 flex items-start gap-4"
            >
              <div class="p-3 rounded-xl bg-amber-500/10 text-amber-500 group-hover:scale-110 transition-transform duration-300">
                <Puzzle class="w-6 h-6" />
              </div>
              <div>
                <h3 class="text-lg font-bold text-text-primary mb-1 group-hover:text-amber-500 transition-colors">
                  {{ $t('qwen.plugins.title') }}
                </h3>
                <p class="text-sm text-text-secondary mb-3 leading-relaxed">
                  {{ $t('qwen.plugins.description') }}
                </p>
                <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-1 rounded bg-amber-500/10 text-amber-500 border border-amber-500/20">{{ $t('qwen.plugins.badge') }}</span>
              </div>
            </Card>
          </RouterLink>

          <!-- Slash Commands -->
          <RouterLink
            to="/qwen/slash-commands"
            class="group h-full md:col-span-2 lg:col-span-1"
          >
            <Card
              variant="glass"
              hover
              glow
              class="h-full p-6 flex items-start gap-4"
            >
              <div class="p-3 rounded-xl bg-red-500/10 text-red-500 group-hover:scale-110 transition-transform duration-300">
                <Command class="w-6 h-6" />
              </div>
              <div>
                <h3 class="text-lg font-bold text-text-primary mb-1 group-hover:text-red-500 transition-colors">
                  {{ $t('qwen.slashCommands.title') }}
                </h3>
                <p class="text-sm text-text-secondary mb-3 leading-relaxed">
                  {{ $t('qwen.slashCommands.description') }}
                </p>
                <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-1 rounded bg-red-500/10 text-red-500 border border-red-500/20">{{ $t('qwen.slashCommands.badge') }}</span>
              </div>
            </Card>
          </RouterLink>
        </div>
      </section>

      <!-- INFO -->
      <section
        class="animate-slide-up"
        style="animation-delay: 200ms"
      >
        <Card
          variant="glass"
          class="p-6"
        >
          <div class="flex items-start gap-4">
            <div class="p-3 rounded-xl bg-purple-500/10 text-purple-500 shrink-0">
              <Info class="w-6 h-6" />
            </div>
            <div class="space-y-4">
              <h3 class="text-lg font-bold text-text-primary">
                💡 {{ $t('qwen.overview.featureTitle') }}
              </h3>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div class="flex items-center gap-3 p-3 rounded-lg bg-bg-surface/30 border border-border-subtle/50">
                  <div class="w-2 h-2 rounded-full bg-purple-500 shadow-[0_0_8px_rgba(168,85,247,0.5)]" />
                  <span class="text-sm text-text-secondary">{{ $t('qwen.overview.feature1') }}</span>
                </div>
                <div class="flex items-center gap-3 p-3 rounded-lg bg-bg-surface/30 border border-border-subtle/50">
                  <div class="w-2 h-2 rounded-full bg-purple-500 shadow-[0_0_8px_rgba(168,85,247,0.5)]" />
                  <span class="text-sm text-text-secondary">{{ $t('qwen.overview.feature2') }}</span>
                </div>
                <div class="flex items-center gap-3 p-3 rounded-lg bg-bg-surface/30 border border-border-subtle/50">
                  <div class="w-2 h-2 rounded-full bg-purple-500 shadow-[0_0_8px_rgba(168,85,247,0.5)]" />
                  <span class="text-sm text-text-secondary">{{ $t('qwen.overview.feature3') }}</span>
                </div>
                <div class="flex items-center gap-3 p-3 rounded-lg bg-bg-surface/30 border border-border-subtle/50">
                  <div class="w-2 h-2 rounded-full bg-purple-500 shadow-[0_0_8px_rgba(168,85,247,0.5)]" />
                  <span class="text-sm text-text-secondary">{{ $t('qwen.overview.feature4') }}</span>
                </div>
              </div>
            </div>
          </div>
        </Card>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { RouterLink } from 'vue-router'
import { Server, Bot, Puzzle, Command, Home, Zap, Boxes, Info, MessageSquare, Globe } from 'lucide-vue-next'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
</script>