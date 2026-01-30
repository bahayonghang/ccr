<template>
  <div class="min-h-full p-6 lg:p-10 relative overflow-hidden">
    <!-- Background Mesh (Consistent with Home) -->
    <div class="fixed inset-0 pointer-events-none -z-10 bg-bg-base">
      <div class="absolute top-[-10%] right-[-5%] w-[500px] h-[500px] bg-accent-primary/5 rounded-full blur-[120px] animate-pulse-subtle" />
      <div
        class="absolute bottom-[-10%] left-[-5%] w-[600px] h-[600px] bg-accent-secondary/5 rounded-full blur-[100px] animate-pulse-subtle"
        style="animation-delay: 2s"
      />
    </div>

    <div class="max-w-7xl mx-auto space-y-10">
      <!-- HEADER -->
      <header class="flex flex-col md:flex-row md:items-center justify-between gap-6 animate-slide-up">
        <div class="flex items-center gap-5">
          <div class="w-16 h-16 rounded-2xl bg-gradient-to-br from-accent-primary/20 to-accent-secondary/20 border border-white/10 flex items-center justify-center shadow-lg backdrop-blur-md">
            <Code2 class="w-8 h-8 text-accent-primary" />
          </div>
          <div>
            <h1 class="text-4xl font-bold font-display tracking-tight text-text-primary mb-1">
              Claude Code
            </h1>
            <p class="text-text-secondary text-lg">
              {{ $t('claudeCode.subtitle') }}
            </p>
          </div>
        </div>

        <RouterLink to="/">
          <Button
            variant="ghost"
            class="group"
          >
            <Home class="w-4 h-4 mr-2 group-hover:text-accent-primary transition-colors" />
            {{ $t('claudeCode.backToHome') }}
          </Button>
        </RouterLink>
      </header>

      <!-- FEATURES TAGS -->
      <div
        class="flex flex-wrap gap-3 animate-slide-up"
        style="animation-delay: 100ms"
      >
        <div class="px-3 py-1.5 rounded-full bg-accent-primary/10 border border-accent-primary/20 text-accent-primary text-xs font-bold uppercase tracking-wider flex items-center gap-2">
          <Zap class="w-3 h-3" /> {{ $t('claudeCode.features.mcpSupport') }}
        </div>
        <div class="px-3 py-1.5 rounded-full bg-purple-500/10 border border-purple-500/20 text-purple-500 text-xs font-bold uppercase tracking-wider flex items-center gap-2">
          <Bot class="w-3 h-3" /> {{ $t('claudeCode.features.aiAgents') }}
        </div>
        <div class="px-3 py-1.5 rounded-full bg-cyan-500/10 border border-cyan-500/20 text-cyan-500 text-xs font-bold uppercase tracking-wider flex items-center gap-2">
          <Cloud class="w-3 h-3" /> {{ $t('claudeCode.features.cloudSync') }}
        </div>
      </div>

      <!-- CORE MODULES (Grid) -->
      <section
        class="grid grid-cols-1 md:grid-cols-2 gap-6 animate-slide-up"
        style="animation-delay: 200ms"
      >
        <RouterLink
          to="/configs"
          class="group h-full"
        >
          <Card
            variant="glass"
            hover
            glow
            class="h-full p-6 flex items-start gap-5 relative overflow-hidden"
          >
            <!-- Decorative bg icon -->
            <Settings class="absolute -right-4 -bottom-4 w-32 h-32 text-accent-primary/5 rotate-12 group-hover:text-accent-primary/10 transition-colors" />
             
            <div class="w-12 h-12 rounded-xl bg-accent-primary/10 flex items-center justify-center shrink-0 border border-accent-primary/20 group-hover:scale-110 transition-transform duration-300">
              <Settings class="w-6 h-6 text-accent-primary" />
            </div>
            <div class="relative z-10">
              <div class="flex items-center gap-2 mb-2">
                <h3 class="text-xl font-bold text-text-primary group-hover:text-accent-primary transition-colors">
                  {{ $t('claudeCode.modules.configManagement.title') }}
                </h3>
                <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-accent-primary/20 text-accent-primary border border-accent-primary/20">
                  {{ $t('claudeCode.modules.configManagement.badge') }}
                </span>
              </div>
              <p class="text-text-secondary leading-relaxed">
                {{ $t('claudeCode.modules.configManagement.desc') }}
              </p>
            </div>
          </Card>
        </RouterLink>

        <RouterLink
          to="/sync"
          class="group h-full"
        >
          <Card
            variant="glass"
            hover
            glow
            class="h-full p-6 flex items-start gap-5 relative overflow-hidden"
          >
            <!-- Decorative bg icon -->
            <Cloud class="absolute -right-4 -bottom-4 w-32 h-32 text-accent-secondary/5 rotate-12 group-hover:text-accent-secondary/10 transition-colors" />

            <div class="w-12 h-12 rounded-xl bg-accent-secondary/10 flex items-center justify-center shrink-0 border border-accent-secondary/20 group-hover:scale-110 transition-transform duration-300">
              <Cloud class="w-6 h-6 text-accent-secondary" />
            </div>
            <div class="relative z-10">
              <div class="flex items-center gap-2 mb-2">
                <h3 class="text-xl font-bold text-text-primary group-hover:text-accent-secondary transition-colors">
                  {{ $t('claudeCode.modules.cloudSync.title') }}
                </h3>
                <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-accent-secondary/20 text-accent-secondary border border-accent-secondary/20">
                  {{ $t('claudeCode.modules.cloudSync.badge') }}
                </span>
              </div>
              <p class="text-text-secondary leading-relaxed">
                {{ $t('claudeCode.modules.cloudSync.desc') }}
              </p>
            </div>
          </Card>
        </RouterLink>
      </section>

      <!-- MARKET BANNER -->
      <section
        class="animate-slide-up"
        style="animation-delay: 300ms"
      >
        <RouterLink
          to="/market"
          class="block group"
        >
          <Card
            variant="glass"
            hover
            glow
            class="relative overflow-hidden p-8 border-accent-warning/20"
          >
            <div class="absolute top-0 right-0 w-96 h-96 bg-accent-warning/5 rounded-full blur-[100px] -mr-20 -mt-20 group-hover:bg-accent-warning/10 transition-colors" />
              
            <div class="relative z-10 flex flex-col md:flex-row md:items-center justify-between gap-6">
              <div class="flex items-start gap-6">
                <div class="w-16 h-16 rounded-2xl bg-gradient-to-br from-orange-400 to-red-500 shadow-lg flex items-center justify-center text-white group-hover:scale-110 transition-transform duration-300 transform rotate-3">
                  <ShoppingBag class="w-8 h-8" />
                </div>
                <div>
                  <h2 class="text-2xl font-bold text-text-primary mb-2 flex items-center gap-3">
                    {{ $t('market.title') }}
                    <span class="px-2 py-1 text-xs font-bold uppercase bg-orange-500/10 text-orange-500 border border-orange-500/20 rounded-md">Beta</span>
                  </h2>
                  <p class="text-text-secondary max-w-xl text-lg">
                    {{ $t('market.description') }}
                  </p>
                </div>
              </div>
              <div class="flex items-center gap-2 text-accent-warning font-bold group-hover:translate-x-1 transition-transform">
                <span>{{ $t('market.explore') }}</span>
                <ArrowRight class="w-5 h-5" />
              </div>
            </div>
          </Card>
        </RouterLink>
      </section>

      <!-- EXTENDED MODULES -->
      <section
        class="animate-slide-up"
        style="animation-delay: 400ms"
      >
        <div class="flex items-center gap-3 mb-6">
          <Boxes class="w-5 h-5 text-text-muted" />
          <h2 class="text-sm font-bold uppercase tracking-widest text-text-muted">
            {{ $t('claudeCode.modules.extensionsTitle') }}
          </h2>
          <div class="h-px flex-1 bg-border-subtle" />
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4">
          <RouterLink
            to="/mcp"
            class="group"
          >
            <Card
              variant="elevated"
              hover
              class="h-full p-4 flex flex-col gap-4"
            >
              <div class="flex items-center justify-between">
                <div class="w-10 h-10 rounded-lg bg-cyan-500/10 flex items-center justify-center text-cyan-500 group-hover:scale-110 transition-transform">
                  <Server class="w-5 h-5" />
                </div>
                <span class="text-[10px] font-bold uppercase text-text-muted bg-bg-base px-2 py-1 rounded">MCP</span>
              </div>
              <div>
                <h3 class="font-bold text-text-primary group-hover:text-cyan-500 transition-colors mb-1">
                  {{ $t('claudeCode.modules.mcpServers.title') }}
                </h3>
                <p class="text-xs text-text-muted line-clamp-2">
                  {{ $t('claudeCode.modules.mcpServers.desc') }}
                </p>
              </div>
            </Card>
          </RouterLink>

          <RouterLink
            to="/agents"
            class="group"
          >
            <Card
              variant="elevated"
              hover
              class="h-full p-4 flex flex-col gap-4"
            >
              <div class="flex items-center justify-between">
                <div class="w-10 h-10 rounded-lg bg-emerald-500/10 flex items-center justify-center text-emerald-500 group-hover:scale-110 transition-transform">
                  <Users class="w-5 h-5" />
                </div>
                <span class="text-[10px] font-bold uppercase text-text-muted bg-bg-base px-2 py-1 rounded">Agent</span>
              </div>
              <div>
                <h3 class="font-bold text-text-primary group-hover:text-emerald-500 transition-colors mb-1">
                  {{ $t('claudeCode.modules.agents.title') }}
                </h3>
                <p class="text-xs text-text-muted line-clamp-2">
                  {{ $t('claudeCode.modules.agents.desc') }}
                </p>
              </div>
            </Card>
          </RouterLink>

          <RouterLink
            to="/skills"
            class="group"
          >
            <Card
              variant="elevated"
              hover
              class="h-full p-4 flex flex-col gap-4"
            >
              <div class="flex items-center justify-between">
                <div class="w-10 h-10 rounded-lg bg-red-500/10 flex items-center justify-center text-red-500 group-hover:scale-110 transition-transform">
                  <Book class="w-5 h-5" />
                </div>
                <span class="text-[10px] font-bold uppercase text-text-muted bg-bg-base px-2 py-1 rounded">Skill</span>
              </div>
              <div>
                <h3 class="font-bold text-text-primary group-hover:text-red-500 transition-colors mb-1">
                  {{ $t('claudeCode.modules.skills.title') }}
                </h3>
                <p class="text-xs text-text-muted line-clamp-2">
                  {{ $t('claudeCode.modules.skills.desc') }}
                </p>
              </div>
            </Card>
          </RouterLink>

          <RouterLink
            to="/plugins"
            class="group"
          >
            <Card
              variant="elevated"
              hover
              class="h-full p-4 flex flex-col gap-4"
            >
              <div class="flex items-center justify-between">
                <div class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center text-blue-500 group-hover:scale-110 transition-transform">
                  <Puzzle class="w-5 h-5" />
                </div>
                <span class="text-[10px] font-bold uppercase text-text-muted bg-bg-base px-2 py-1 rounded">Plug</span>
              </div>
              <div>
                <h3 class="font-bold text-text-primary group-hover:text-blue-500 transition-colors mb-1">
                  {{ $t('claudeCode.modules.plugins.title') }}
                </h3>
                <p class="text-xs text-text-muted line-clamp-2">
                  {{ $t('claudeCode.modules.plugins.desc') }}
                </p>
              </div>
            </Card>
          </RouterLink>

          <RouterLink
            to="/slash-commands"
            class="group"
          >
            <Card
              variant="elevated"
              hover
              class="h-full p-4 flex flex-col gap-4"
            >
              <div class="flex items-center justify-between">
                <div class="w-10 h-10 rounded-lg bg-amber-500/10 flex items-center justify-center text-amber-500 group-hover:scale-110 transition-transform">
                  <Terminal class="w-5 h-5" />
                </div>
                <span class="text-[10px] font-bold uppercase text-text-muted bg-bg-base px-2 py-1 rounded">CMD</span>
              </div>
              <div>
                <h3 class="font-bold text-text-primary group-hover:text-amber-500 transition-colors mb-1">
                  {{ $t('claudeCode.modules.slashCommands.title') }}
                </h3>
                <p class="text-xs text-text-muted line-clamp-2">
                  {{ $t('claudeCode.modules.slashCommands.desc') }}
                </p>
              </div>
            </Card>
          </RouterLink>
        </div>
      </section>

      <!-- BOTTOM GRID (Commands & Tips) -->
      <section
        class="grid grid-cols-1 lg:grid-cols-2 gap-6 animate-slide-up"
        style="animation-delay: 500ms"
      >
        <!-- Common Commands -->
        <Card
          variant="glass"
          class="p-6"
        >
          <h3 class="text-lg font-bold text-text-primary mb-4 flex items-center gap-2">
            <Terminal class="w-5 h-5 text-accent-primary" />
            {{ $t('claudeCode.quickActions.commonCommands') }}
          </h3>
          <div class="space-y-3">
            <div
              v-for="cmd in commonCommands"
              :key="cmd.cmd"
              class="flex items-center justify-between p-3 rounded-xl bg-bg-surface/50 border border-border-subtle hover:border-accent-primary/20 transition-colors group cursor-copy"
              @click="copyCommand(cmd.cmd)"
            >
              <span class="text-sm font-medium text-text-secondary">{{ cmd.label }}</span>
              <div class="flex items-center gap-2">
                <code class="text-xs font-mono px-2 py-1 rounded bg-bg-base border border-border-subtle text-text-muted group-hover:text-accent-primary transition-colors">{{ cmd.cmd }}</code>
                <Copy class="w-3 h-3 text-text-muted opacity-0 group-hover:opacity-100 transition-opacity" />
              </div>
            </div>
          </div>
        </Card>

        <!-- Resources & Tips -->
        <div class="space-y-6">
          <Card
            variant="glass"
            class="p-6"
          >
            <h3 class="text-lg font-bold text-text-primary mb-4 flex items-center gap-2">
              <BookOpen class="w-5 h-5 text-accent-secondary" />
              {{ $t('claudeCode.quickActions.resources') }}
            </h3>
            <div class="grid grid-cols-2 gap-3">
              <a
                v-for="res in resources"
                :key="res.url"
                :href="res.url"
                target="_blank"
                class="flex items-center gap-3 p-3 rounded-xl bg-bg-surface/50 border border-border-subtle hover:bg-bg-elevated transition-colors group"
              >
                <component
                  :is="res.icon"
                  class="w-4 h-4 text-text-muted group-hover:text-accent-secondary transition-colors"
                />
                <span class="text-sm font-medium text-text-secondary group-hover:text-text-primary transition-colors">{{ res.label }}</span>
                <ExternalLink class="w-3 h-3 text-text-muted ml-auto opacity-0 group-hover:opacity-100" />
              </a>
            </div>
          </Card>

          <Card
            variant="outline"
            class="p-4 bg-accent-info/5 border-accent-info/20"
          >
            <div class="flex gap-3">
              <Info class="w-5 h-5 text-accent-info shrink-0 mt-0.5" />
              <div class="space-y-2">
                <h4 class="text-sm font-bold text-accent-info">
                  {{ $t('claudeCode.tips.title') }}
                </h4>
                <ul class="text-xs text-text-secondary space-y-1 list-disc list-inside opacity-80">
                  <li>{{ $t('claudeCode.tips.tip1') }}</li>
                  <li>{{ $t('claudeCode.tips.tip3') }}</li>
                </ul>
              </div>
            </div>
          </Card>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import { 
  Settings, Cloud, Server, Terminal, Users, Puzzle, Home,
  Code2, Boxes, BookOpen, ExternalLink, Info, Book,
  ShoppingBag, Zap, Bot, ArrowRight, Copy, Github
} from 'lucide-vue-next'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'

const { t } = useI18n()

const commonCommands = computed(() => [
   { label: t('claudeCode.quickActions.viewCurrentConfig'), cmd: 'ccr current' },
   { label: t('claudeCode.quickActions.switchConfig'), cmd: 'ccr switch' },
   { label: t('claudeCode.quickActions.listAllConfigs'), cmd: 'ccr list' },
])

const resources = computed(() => [
   { label: t('claudeCode.quickActions.officialDocs'), url: 'https://docs.anthropic.com/claude/docs', icon: BookOpen },
   { label: t('claudeCode.quickActions.mcpProtocol'), url: 'https://modelcontextprotocol.io', icon: Server },
   { label: t('claudeCode.quickActions.sdkRepo'), url: 'https://github.com/anthropics/anthropic-sdk-typescript', icon: Github },
])

const copyCommand = (cmd: string) => {
  navigator.clipboard.writeText(cmd)
  // Toast notification would go here
}
</script>