<template>
  <div class="min-h-screen md:h-screen w-full bg-bg-primary text-white overflow-y-auto md:overflow-hidden flex flex-col relative transition-colors duration-300">
    <div class="absolute inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-0 right-0 w-[600px] h-[600px] rounded-full opacity-10 blur-3xl animate-pulse-slow"
        :style="{ background: 'radial-gradient(circle, var(--accent-primary) 0%, transparent 70%)' }"
      />
      <div
        class="absolute bottom-0 left-0 w-[500px] h-[500px] rounded-full opacity-10 blur-3xl animate-pulse-slow"
        :style="{ background: 'radial-gradient(circle, var(--accent-secondary) 0%, transparent 70%)', animationDelay: '1s' }"
      />
      <div
        class="absolute inset-0 opacity-[0.03]"
        style="background-image: linear-gradient(var(--accent-primary) 1px, transparent 1px), linear-gradient(90deg, var(--accent-primary) 1px, transparent 1px); background-size: 50px 50px;"
      />
      <div
        class="absolute inset-0 opacity-[0.02] pointer-events-none animate-scan-lines"
        style="background: repeating-linear-gradient(0deg, transparent, transparent 2px, var(--accent-primary) 2px, var(--accent-primary) 4px);"
      />
    </div>

    <header class="flex-none px-4 py-4 sm:px-6 flex flex-col gap-4 border-b border-border-color bg-bg-primary/80 backdrop-blur-md z-10 animate-fade-in-down sm:flex-row sm:items-center sm:justify-between">
      <div class="flex min-w-0 items-center gap-4">
        <div class="relative group">
          <div class="absolute inset-0 bg-accent-primary/30 blur-xl rounded-full group-hover:bg-accent-primary/50 transition-colors duration-500 animate-pulse-glow" />
          <div class="relative w-10 h-10 rounded-xl glass-effect flex items-center justify-center border border-accent-primary/30 shadow-neon-jade group-hover:scale-110 group-hover:border-accent-primary/60 transition-[color,background-color,border-color,transform] duration-300">
            <SIcon
              name="Terminal"
              size="w-5 h-5"
              class="text-accent-primary drop-shadow-neon"
            />
          </div>
        </div>
        <div>
          <h1 class="flex flex-wrap items-center gap-3 text-xl font-bold tracking-tight text-white neon-text-glow">
            {{ $t('ccrControl.title') }}
            <span
              v-if="versionInfo?.current_version"
              class="text-xs px-2 py-0.5 rounded-full bg-accent-primary/10 border border-accent-primary/20 text-accent-primary font-mono"
            >
              v{{ versionInfo.current_version }}
            </span>
          </h1>
          <p class="text-xs text-text-primary">
            {{ $t('ccrControl.description') }}
          </p>
        </div>
      </div>

      <div class="flex items-center justify-between gap-4 sm:justify-end">
        <div class="flex items-center gap-2 text-xs font-mono text-text-muted">
          <span class="w-2 h-2 rounded-full bg-accent-primary animate-pulse" />
          {{ $t('ccrControl.systemReady') }}
        </div>
        <ThemeToggle />
      </div>
    </header>

    <slot name="status" />

    <div class="flex-1 flex flex-col gap-4 overflow-visible p-3 animate-fade-in sm:p-4 xl:flex-row xl:overflow-hidden">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import ThemeToggle from '@/components/ThemeToggle.vue'
import type { VersionInfo } from '@/types'

defineProps<{
  versionInfo: VersionInfo | null
}>()
</script>
