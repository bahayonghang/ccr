<template>
  <div class="min-h-screen bg-bg-base p-6">
    <div class="max-w-[1800px] mx-auto">
      <Breadcrumb
        :items="[
          { label: $t('common.home'), path: '/', icon: Home },
          { label: 'Codex', path: '/codex', icon: Code2 }
        ]"
        module-color="#ec4899"
      />

      <div class="mt-8 space-y-8">
        <!-- Header & Status Section -->
        <section class="grid grid-cols-1 lg:grid-cols-4 gap-6">
          <!-- Hero Card -->
          <GuofengCard
            class="lg:col-span-2 relative overflow-hidden group"
            padding="lg"
            :gradient-border="true"
            glow-color="primary"
          >
            <!-- Background Decoration -->
            <div class="absolute top-0 right-0 w-64 h-64 bg-gradient-to-bl from-platform-codex/10 to-transparent -mr-16 -mt-16 rounded-bl-full pointer-events-none" />

            <div class="relative z-10 flex flex-col justify-between h-full">
              <div>
                <div class="flex items-center gap-3 mb-4">
                  <div class="p-3 rounded-xl bg-platform-codex/10 text-platform-codex">
                    <Code2 class="w-8 h-8" />
                  </div>
                  <div>
                    <h1 class="text-3xl font-bold bg-gradient-to-r from-text-primary to-text-secondary bg-clip-text text-transparent">
                      Codex
                    </h1>
                    <p class="text-sm text-text-secondary mt-1 max-w-md">
                      {{ $t('codex.overview.subtitle') }}
                    </p>
                  </div>
                </div>
              </div>
              
              <div class="flex flex-wrap gap-2 mt-4">
                <span class="px-3 py-1 rounded-full text-xs font-medium bg-platform-codex/10 text-platform-codex border border-platform-codex/20">
                  {{ $t('codex.overview.features.mcpProtocol') }}
                </span>
                <span class="px-3 py-1 rounded-full text-xs font-medium bg-accent-secondary/10 text-accent-secondary border border-accent-secondary/20">
                  v1.0.0
                </span>
              </div>
            </div>
          </GuofengCard>

          <!-- Status Cards -->
          <div class="lg:col-span-2 grid grid-cols-1 sm:grid-cols-2 gap-4">
            <!-- Active Profile -->
            <GuofengCard
              :interactive="false"
              class="flex flex-col justify-center"
            >
              <div class="flex items-center gap-4">
                <div class="p-3 rounded-xl bg-yellow-500/10 text-yellow-500">
                  <Zap class="w-6 h-6" />
                </div>
                <div>
                  <p class="text-xs font-medium text-text-muted uppercase tracking-wider mb-1">
                    {{ $t('codex.status.currentConfig') }}
                  </p>
                  <p class="text-xl font-bold text-text-primary truncate max-w-[150px]">
                    {{ currentProfile || $t('codex.status.notSet') }}
                  </p>
                </div>
              </div>
            </GuofengCard>

            <!-- Total Profiles -->
            <GuofengCard
              :interactive="false"
              class="flex flex-col justify-center"
            >
              <div class="flex items-center gap-4">
                <div class="p-3 rounded-xl bg-blue-500/10 text-blue-500">
                  <Settings class="w-6 h-6" />
                </div>
                <div>
                  <p class="text-xs font-medium text-text-muted uppercase tracking-wider mb-1">
                    {{ $t('codex.status.totalProfiles') }}
                  </p>
                  <p class="text-xl font-bold text-text-primary">
                    {{ profilesCount }}
                  </p>
                </div>
              </div>
            </GuofengCard>

            <!-- System Status (Mock) -->
            <GuofengCard
              :interactive="false"
              class="flex flex-col justify-center"
            >
              <div class="flex items-center gap-4">
                <div class="p-3 rounded-xl bg-emerald-500/10 text-emerald-500">
                  <Activity class="w-6 h-6" />
                </div>
                <div>
                  <p class="text-xs font-medium text-text-muted uppercase tracking-wider mb-1">
                    System Status
                  </p>
                  <p class="text-xl font-bold text-text-primary">
                    Online
                  </p>
                </div>
              </div>
            </GuofengCard>
              
            <!-- Quick Actions -->
            <GuofengCard
              :interactive="false"
              class="flex flex-col justify-center items-start gap-2"
            >
              <p class="text-xs font-medium text-text-muted uppercase tracking-wider w-full">
                Quick Access
              </p>
              <button class="w-full text-left text-sm font-medium text-accent-primary hover:underline flex items-center gap-1">
                See Documentation <ArrowRight class="w-3 h-3" />
              </button>
            </GuofengCard>
          </div>
        </section>

        <!-- Modules Grid -->
        <section>
          <div class="flex items-center gap-2 mb-4">
            <Boxes class="w-5 h-5 text-platform-codex" />
            <h2 class="text-xl font-bold text-text-primary">
              {{ $t('codex.overview.modulesTitle') }}
            </h2>
          </div>
           
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <!-- Profiles Module -->
            <GuofengCard 
              v-for="module in modules" 
              :key="module.path"
              :interactive="true"
              padding="lg"
              class="group h-full"
              @click="router.push(module.path)"
            >
              <div class="flex flex-col h-full">
                <div class="flex justify-between items-start mb-4">
                  <div
                    class="p-3 rounded-xl transition-transform duration-300 group-hover:scale-110"
                    :class="module.bgClass"
                  >
                    <component
                      :is="module.icon"
                      class="w-8 h-8"
                      :style="{ color: module.color }"
                    />
                  </div>
                  <div
                    class="px-2 py-1 rounded text-[10px] font-bold uppercase tracking-wide border" 
                    :class="module.badgeClass"
                  >
                    {{ module.badge }}
                  </div>
                </div>
                    
                <h3 class="text-xl font-bold text-text-primary mb-2 group-hover:text-platform-codex transition-colors">
                  {{ module.title }}
                </h3>
                <p class="text-sm text-text-secondary leading-relaxed flex-grow">
                  {{ module.description }}
                </p>
                    
                <div
                  class="mt-4 pt-4 border-t border-border-subtle flex items-center text-sm font-medium opacity-0 group-hover:opacity-100 transition-opacity transform translate-y-2 group-hover:translate-y-0"
                  :style="{ color: module.color }"
                >
                  <span>Open Module</span>
                  <ArrowRight class="w-4 h-4 ml-1" />
                </div>
              </div>
            </GuofengCard>
          </div>
        </section>

        <!-- Info & Tips Section -->
        <section class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <!-- Codex Usage Panel -->
          <GuofengCard>
            <div class="flex items-center justify-between mb-4">
              <div class="flex items-center gap-2">
                <BarChart3 class="w-5 h-5 text-platform-codex" />
                <h3 class="text-lg font-bold text-text-primary">
                  {{ $t('codex.overview.usageTitle') }}
                </h3>
              </div>
              <button
                class="p-1.5 rounded-lg hover:bg-bg-overlay/50 transition-colors text-text-muted hover:text-text-primary"
                :disabled="usageLoading"
                @click="refreshUsage"
              >
                <RefreshCw
                  class="w-4 h-4"
                  :class="{ 'animate-spin': usageLoading }"
                />
              </button>
            </div>

            <!-- Loading State -->
            <div
              v-if="usageLoading"
              class="space-y-3"
            >
              <div class="h-16 bg-bg-overlay/30 rounded-lg animate-pulse" />
              <div class="h-16 bg-bg-overlay/30 rounded-lg animate-pulse" />
            </div>

            <!-- Error State -->
            <div
              v-else-if="usageError"
              class="text-center py-6"
            >
              <AlertCircle class="w-8 h-8 text-text-muted mx-auto mb-2" />
              <p class="text-sm text-text-muted">
                {{ $t('codex.overview.usageError') }}
              </p>
              <button
                class="mt-2 text-xs text-accent-primary hover:underline"
                @click="refreshUsage"
              >
                {{ $t('common.retry') }}
              </button>
            </div>

            <!-- Empty State -->
            <div
              v-else-if="!usageData || usageData.all_time.total_requests === 0"
              class="text-center py-6"
            >
              <Clock class="w-8 h-8 text-text-muted mx-auto mb-2" />
              <p class="text-sm text-text-muted">
                {{ $t('codex.overview.noUsageData') }}
              </p>
            </div>

            <!-- Usage Data -->
            <div
              v-else
              class="space-y-4"
            >
              <!-- 5-Hour Window -->
              <div class="p-3 rounded-lg bg-bg-overlay/30 border border-border-subtle">
                <div class="flex items-center justify-between mb-2">
                  <span class="text-xs font-medium text-text-muted uppercase tracking-wider">
                    {{ $t('codex.overview.usage5h') }}
                  </span>
                  <span class="text-xs text-text-muted">
                    {{ usageData.five_hour.total_requests }} {{ $t('codex.overview.requests') }}
                  </span>
                </div>
                <div class="flex items-baseline gap-2">
                  <span class="text-2xl font-bold text-text-primary">
                    {{ formatTokens(usageData.five_hour.total_input_tokens + usageData.five_hour.total_output_tokens) }}
                  </span>
                  <span class="text-sm text-text-muted">tokens</span>
                </div>
                <div class="mt-1 text-xs text-text-muted">
                  {{ formatTokens(usageData.five_hour.total_input_tokens) }} in / {{ formatTokens(usageData.five_hour.total_output_tokens) }} out
                </div>
              </div>

              <!-- 7-Day Window -->
              <div class="p-3 rounded-lg bg-bg-overlay/30 border border-border-subtle">
                <div class="flex items-center justify-between mb-2">
                  <span class="text-xs font-medium text-text-muted uppercase tracking-wider">
                    {{ $t('codex.overview.usage7d') }}
                  </span>
                  <span class="text-xs text-text-muted">
                    {{ usageData.seven_day.total_requests }} {{ $t('codex.overview.requests') }}
                  </span>
                </div>
                <div class="flex items-baseline gap-2">
                  <span class="text-2xl font-bold text-text-primary">
                    {{ formatTokens(usageData.seven_day.total_input_tokens + usageData.seven_day.total_output_tokens) }}
                  </span>
                  <span class="text-sm text-text-muted">tokens</span>
                </div>
                <div class="mt-1 text-xs text-text-muted">
                  {{ formatTokens(usageData.seven_day.total_input_tokens) }} in / {{ formatTokens(usageData.seven_day.total_output_tokens) }} out
                </div>
              </div>
            </div>

            <!-- Usage Dashboard Link -->
            <div class="mt-4 pt-4 border-t border-border-subtle">
              <div class="flex items-start gap-3 p-2 rounded-lg bg-blue-500/5 border border-blue-500/10">
                <Info class="w-4 h-4 text-blue-500 shrink-0 mt-0.5" />
                <div class="text-xs text-text-secondary">
                  <p>{{ $t('codex.overview.usageTip') }}</p>
                  <p class="mt-1 text-text-muted">
                    {{ $t('codex.overview.usageStatusTip') }}
                  </p>
                </div>
              </div>
            </div>
          </GuofengCard>

          <!-- Tips Card -->
          <GuofengCard>
            <div class="flex items-center gap-2 mb-4">
              <Cpu class="w-5 h-5 text-platform-codex" />
              <h3 class="text-lg font-bold text-text-primary">
                System Capabilities
              </h3>
            </div>
            <div class="space-y-3">
              <div
                v-for="(feature, index) in features"
                :key="index"
                class="flex items-center gap-3 p-2 rounded-lg hover:bg-bg-overlay/50 transition-colors"
              >
                <div class="w-1.5 h-1.5 rounded-full bg-platform-codex shadow-[0_0_8px_var(--platform-codex)]" />
                <span class="text-sm text-text-secondary">{{ feature }}</span>
              </div>
            </div>
          </GuofengCard>

          <!-- Tips Card -->
          <GuofengCard>
            <div class="flex items-center gap-2 mb-4">
              <Info class="w-5 h-5 text-platform-codex" />
              <h3 class="text-lg font-bold text-text-primary">
                {{ $t('codex.overview.tipsTitle') }}
              </h3>
            </div>
            <div class="flex items-start gap-4 p-2">
              <div class="p-2 rounded-lg bg-yellow-500/10 text-yellow-500 shrink-0">
                <Lightbulb class="w-5 h-5" />
              </div>
              <div>
                <p class="text-sm text-text-secondary leading-relaxed">
                  {{ $t('codex.overview.tips.mcpConnection') }}
                </p>
              </div>
            </div>
          </GuofengCard>
        </section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  Settings, Server, Home, Command, Code2, Boxes, Info,
  Zap, Activity, ArrowRight, Lightbulb, Cpu, KeyRound,
  BarChart3, RefreshCw, AlertCircle, Clock
} from 'lucide-vue-next'

import Breadcrumb from '@/components/Breadcrumb.vue'
import GuofengCard from '@/components/common/GuofengCard.vue'
import { listCodexProfiles, getCodexUsage } from '@/api'
import type { CodexUsageResponse } from '@/types'

const router = useRouter()
const { t } = useI18n()

// State
const profilesCount = ref(0)
const currentProfile = ref<string | null>(null)

// Usage State
const usageData = ref<CodexUsageResponse | null>(null)
const usageLoading = ref(false)
const usageError = ref(false)

// Format tokens with K/M suffix
const formatTokens = (tokens: number): string => {
  if (tokens >= 1_000_000) {
    return `${(tokens / 1_000_000).toFixed(1)}M`
  } else if (tokens >= 1_000) {
    return `${(tokens / 1_000).toFixed(1)}K`
  }
  return tokens.toString()
}

// Refresh usage data
const refreshUsage = async () => {
  usageLoading.value = true
  usageError.value = false
  try {
    usageData.value = await getCodexUsage()
  } catch (error) {
    console.error('Failed to load usage data:', error)
    usageError.value = true
  } finally {
    usageLoading.value = false
  }
}

// Modules Data
const modules = computed(() => [
  {
    path: '/codex/profiles',
    title: t('codex.overview.modules.profiles.title'),
    description: t('codex.overview.modules.profiles.description'),
    badge: t('codex.overview.modules.profiles.badge'),
    icon: Settings,
    color: 'var(--platform-codex)',
    bgClass: 'bg-platform-codex/10',
    badgeClass: 'bg-platform-codex/10 text-platform-codex border-platform-codex/20'
  },
  {
    path: '/codex/mcp',
    title: t('codex.overview.modules.mcp.title'),
    description: t('codex.overview.modules.mcp.description'),
    badge: t('codex.overview.modules.mcp.badge'),
    icon: Server,
    color: 'var(--platform-claude)',
    bgClass: 'bg-indigo-500/10',
    badgeClass: 'bg-indigo-500/10 text-indigo-500 border-indigo-500/20'
  },
  {
    path: '/codex/slash-commands',
    title: t('codex.overview.modules.slashCommands.title'),
    description: t('codex.overview.modules.slashCommands.description'),
    badge: t('codex.overview.modules.slashCommands.badge'),
    icon: Command,
    color: 'var(--accent-tertiary)',
    bgClass: 'bg-pink-500/10',
    badgeClass: 'bg-pink-500/10 text-pink-500 border-pink-500/20'
  },
  {
    path: '/codex/auth',
    title: t('codex.overview.modules.auth.title'),
    description: t('codex.overview.modules.auth.description'),
    badge: t('codex.overview.modules.auth.badge'),
    icon: KeyRound,
    color: 'var(--accent-warning)',
    bgClass: 'bg-amber-500/10',
    badgeClass: 'bg-amber-500/10 text-amber-500 border-amber-500/20'
  }
])

const features = [
  'Advanced MCP Protocol Integration (Stdio & SSE)',
  'Multi-Profile Configuration Management',
  'Custom Slash Commands Support',
  'Context-Aware AI Coding Assistance'
]

onMounted(async () => {
  try {
    const data = await listCodexProfiles()
    if (data.profiles) {
      profilesCount.value = data.profiles.length
      currentProfile.value = data.current_profile ?? null
    }
  } catch (error) {
    console.error('Failed to load profile status:', error)
  }

  // Load usage data
  refreshUsage()
})
</script>
