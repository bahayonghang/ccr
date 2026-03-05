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
            <div class="w-16 h-16 rounded-2xl bg-platform-gemini/10 flex items-center justify-center border border-platform-gemini/20 shadow-lg backdrop-blur-md">
              <Sparkles class="w-8 h-8 text-platform-gemini" />
            </div>
            <div>
              <h1 class="text-4xl font-bold font-display bg-gradient-to-r from-platform-gemini via-platform-iflow to-accent-success bg-clip-text text-transparent tracking-tight">
                Gemini CLI
              </h1>
              <p class="text-text-secondary text-lg mt-1 max-w-xl">
                {{ $t('gemini.overview.description') }}
              </p>
            </div>
          </div>
          
          <RouterLink to="/">
            <Button
              variant="glass"
              class="gap-2"
            >
              <Home class="w-4 h-4" />
              {{ $t('common.backToHome') }}
            </Button>
          </RouterLink>
        </div>

        <!-- Tags -->
        <div class="flex flex-wrap gap-3">
          <span class="px-3 py-1.5 rounded-full text-xs font-bold uppercase tracking-wider bg-platform-gemini/10 text-platform-gemini border border-platform-gemini/20 flex items-center gap-2">
            <Server class="w-3 h-3" /> MCP Server
          </span>
          <span class="px-3 py-1.5 rounded-full text-xs font-bold uppercase tracking-wider bg-accent-success/10 text-accent-success border border-accent-success/20 flex items-center gap-2">
            <Bot class="w-3 h-3" /> Smart Agents
          </span>
          <span class="px-3 py-1.5 rounded-full text-xs font-bold uppercase tracking-wider bg-platform-claude/10 text-platform-claude border border-platform-claude/20 flex items-center gap-2">
            <Puzzle class="w-3 h-3" /> Plugin System
          </span>
          <span class="px-3 py-1.5 rounded-full text-xs font-bold uppercase tracking-wider bg-platform-iflow/10 text-platform-iflow border border-platform-iflow/20 flex items-center gap-2">
            <Sparkles class="w-3 h-3" /> Multimodal
          </span>
        </div>
      </section>

      <!-- MODULES -->
      <section
        class="animate-slide-up"
        style="animation-delay: 100ms"
      >
        <div class="flex items-center gap-3 mb-6">
          <Boxes class="w-5 h-5 text-platform-gemini" />
          <h2 class="text-lg font-bold uppercase tracking-widest text-text-muted">
            {{ $t('common.modules') }}
          </h2>
          <div class="h-px flex-1 bg-border-subtle" />
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          <!-- MCP -->
          <RouterLink
            to="/gemini-cli/mcp"
            class="group h-full"
          >
            <Card
              variant="glass"
              hover
              glow
              class="h-full p-6 flex items-start gap-4"
            >
              <div class="p-3 rounded-xl bg-platform-gemini/10 text-platform-gemini group-hover:scale-110 transition-transform duration-300">
                <Server class="w-6 h-6" />
              </div>
              <div>
                <h3 class="text-lg font-bold text-text-primary mb-1 group-hover:text-platform-gemini transition-colors">
                  {{ $t('gemini.mcp.title') }}
                </h3>
                <p class="text-sm text-text-secondary mb-3 leading-relaxed">
                  {{ $t('gemini.mcp.subtitle') }}
                </p>
                <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-1 rounded bg-platform-gemini/10 text-platform-gemini border border-platform-gemini/20">MCP</span>
              </div>
            </Card>
          </RouterLink>

          <!-- Agents -->
          <RouterLink
            to="/gemini-cli/agents"
            class="group h-full"
          >
            <Card
              variant="glass"
              hover
              glow
              class="h-full p-6 flex items-start gap-4"
            >
              <div class="p-3 rounded-xl bg-accent-success/10 text-accent-success group-hover:scale-110 transition-transform duration-300">
                <Bot class="w-6 h-6" />
              </div>
              <div>
                <h3 class="text-lg font-bold text-text-primary mb-1 group-hover:text-accent-success transition-colors">
                  {{ $t('gemini.agents.title') }}
                </h3>
                <p class="text-sm text-text-secondary mb-3 leading-relaxed">
                  {{ $t('gemini.agents.subtitle') }}
                </p>
                <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-1 rounded bg-accent-success/10 text-accent-success border border-accent-success/20">AI Agents</span>
              </div>
            </Card>
          </RouterLink>

          <!-- Plugins -->
          <RouterLink
            to="/gemini-cli/plugins"
            class="group h-full"
          >
            <Card
              variant="glass"
              hover
              glow
              class="h-full p-6 flex items-start gap-4"
            >
              <div class="p-3 rounded-xl bg-platform-claude/10 text-platform-claude group-hover:scale-110 transition-transform duration-300">
                <Puzzle class="w-6 h-6" />
              </div>
              <div>
                <h3 class="text-lg font-bold text-text-primary mb-1 group-hover:text-platform-claude transition-colors">
                  {{ $t('gemini.plugins.title') }}
                </h3>
                <p class="text-sm text-text-secondary mb-3 leading-relaxed">
                  {{ $t('gemini.plugins.subtitle') }}
                </p>
                <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-1 rounded bg-platform-claude/10 text-platform-claude border border-platform-claude/20">Plugins</span>
              </div>
            </Card>
          </RouterLink>

          <!-- Slash Commands -->
          <RouterLink
            to="/gemini-cli/slash-commands"
            class="group h-full md:col-span-2 lg:col-span-1"
          >
            <Card
              variant="glass"
              hover
              glow
              class="h-full p-6 flex items-start gap-4"
            >
              <div class="p-3 rounded-xl bg-platform-iflow/10 text-platform-iflow group-hover:scale-110 transition-transform duration-300">
                <Command class="w-6 h-6" />
              </div>
              <div>
                <h3 class="text-lg font-bold text-text-primary mb-1 group-hover:text-platform-iflow transition-colors">
                  {{ $t('gemini.slashCommands.title') }}
                </h3>
                <p class="text-sm text-text-secondary mb-3 leading-relaxed">
                  {{ $t('gemini.slashCommands.subtitle') }}
                </p>
                <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-1 rounded bg-platform-iflow/10 text-platform-iflow border border-platform-iflow/20">Commands</span>
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
            <div class="p-3 rounded-xl bg-platform-gemini/10 text-platform-gemini shrink-0">
              <Info class="w-6 h-6" />
            </div>
            <div class="space-y-4">
              <h3 class="text-lg font-bold text-text-primary">
                💡 {{ $t('gemini.overview.welcome') }}
              </h3>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div class="flex items-center gap-3 p-3 rounded-lg bg-bg-surface/30 border border-border-subtle/50">
                  <div class="w-2 h-2 rounded-full bg-platform-gemini shadow-[0_0_8px_var(--platform-gemini)]" />
                  <span class="text-sm text-text-secondary">{{ $t('gemini.overview.feature1') }}</span>
                </div>
                <div class="flex items-center gap-3 p-3 rounded-lg bg-bg-surface/30 border border-border-subtle/50">
                  <div class="w-2 h-2 rounded-full bg-platform-gemini shadow-[0_0_8px_var(--platform-gemini)]" />
                  <span class="text-sm text-text-secondary">{{ $t('gemini.overview.feature2') }}</span>
                </div>
                <div class="flex items-center gap-3 p-3 rounded-lg bg-bg-surface/30 border border-border-subtle/50">
                  <div class="w-2 h-2 rounded-full bg-platform-gemini shadow-[0_0_8px_var(--platform-gemini)]" />
                  <span class="text-sm text-text-secondary">{{ $t('gemini.overview.feature3') }}</span>
                </div>
                <div class="flex items-center gap-3 p-3 rounded-lg bg-bg-surface/30 border border-border-subtle/50">
                  <div class="w-2 h-2 rounded-full bg-platform-gemini shadow-[0_0_8px_var(--platform-gemini)]" />
                  <span class="text-sm text-text-secondary">{{ $t('gemini.overview.feature4') }}</span>
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
import { Server, Bot, Puzzle, Command, Home, Sparkles, Boxes, Info } from 'lucide-vue-next'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
</script>