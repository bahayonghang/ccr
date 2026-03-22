<template>
  <div class="space-y-6">
    <div
      class="grid grid-cols-1 gap-4 animate-slide-up md:grid-cols-3"
      style="animation-delay: 80ms"
    >
      <div class="rounded-[28px] border border-border-default/50 bg-bg-surface/78 p-5 shadow-lg shadow-black/5">
        <div class="flex items-center gap-4">
          <div class="flex h-11 w-11 items-center justify-center rounded-2xl bg-accent-secondary/12 text-accent-secondary">
            <SIcon
              name="Zap"
              size="w-5 h-5"
            />
          </div>
          <div class="min-w-0">
            <p class="text-xs font-semibold uppercase tracking-[0.24em] text-text-muted">
              {{ $t('claudeProfiles.currentProfile') }}
            </p>
            <p
              class="mt-2 truncate text-lg font-semibold text-text-primary"
              :title="currentProfileName || $t('claudeProfiles.notSet')"
            >
              {{ currentProfileName || $t('claudeProfiles.notSet') }}
            </p>
          </div>
        </div>
      </div>

      <div class="rounded-[28px] border border-border-default/50 bg-bg-surface/78 p-5 shadow-lg shadow-black/5">
        <div class="flex items-center gap-4">
          <div class="flex h-11 w-11 items-center justify-center rounded-2xl bg-accent-primary/12 text-accent-primary">
            <SIcon
              name="Layers"
              size="w-5 h-5"
            />
          </div>
          <div>
            <p class="text-xs font-semibold uppercase tracking-[0.24em] text-text-muted">
              {{ $t('claudeProfiles.totalCount') }}
            </p>
            <p class="mt-2 text-lg font-semibold text-text-primary">
              {{ totalProfiles }}
            </p>
          </div>
        </div>
      </div>

      <div class="rounded-[28px] border border-border-default/50 bg-bg-surface/78 p-5 shadow-lg shadow-black/5">
        <div class="flex items-center gap-4">
          <div class="flex h-11 w-11 items-center justify-center rounded-2xl bg-accent-success/12 text-accent-success">
            <SIcon
              name="CheckCircle2"
              size="w-5 h-5"
            />
          </div>
          <div>
            <p class="text-xs font-semibold uppercase tracking-[0.24em] text-text-muted">
              {{ $t('claudeProfiles.enabledCount') }}
            </p>
            <p class="mt-2 text-lg font-semibold text-text-primary">
              {{ enabledProfilesCount }}
            </p>
          </div>
        </div>
      </div>
    </div>

    <div
      v-if="showNavigation"
      class="animate-slide-up rounded-[28px] border border-border-default/50 bg-bg-surface/78 p-4 shadow-lg shadow-black/5"
      style="animation-delay: 120ms"
    >
      <div class="mb-3 flex flex-col gap-2 md:flex-row md:items-end md:justify-between">
        <div>
          <p class="flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.24em] text-text-muted">
            <SIcon
              name="Shuffle"
              size="w-3.5 h-3.5"
            />
            {{ $t('claudeProfiles.quickSwitch') }}
          </p>
          <p class="mt-1 text-sm text-text-secondary">
            {{ $t('claudeProfiles.quickSwitchHint') }}
          </p>
        </div>
        <span class="rounded-full bg-bg-elevated px-3 py-1 text-xs font-medium text-text-muted">
          {{ $t('claudeProfiles.providerSectionsCount', { count: providerSectionsCount }) }}
        </span>
      </div>

      <div class="flex flex-wrap gap-2.5">
        <button
          v-for="profile in profiles"
          :key="profile.name"
          type="button"
          class="flex min-h-[40px] items-center gap-2 rounded-2xl border px-3.5 py-2 text-sm font-medium transition-colors"
          :class="profile.is_current
            ? 'border-accent-secondary/35 bg-accent-secondary/12 text-accent-secondary'
            : 'border-border-default/50 bg-bg-surface/60 text-text-secondary hover:border-border-default hover:bg-bg-elevated hover:text-text-primary'"
          @click="$emit('apply', profile.name)"
        >
          <SIcon
            v-if="profile.is_current"
            name="Check"
            size="w-3.5 h-3.5"
          />
          <span>{{ profile.name }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import type { ClaudeProfile } from '@/types'

defineProps<{
  currentProfileName: string | null
  enabledProfilesCount: number
  profiles: ClaudeProfile[]
  providerSectionsCount: number
  showNavigation: boolean
  totalProfiles: number
}>()

defineEmits<{
  apply: [name: string]
}>()
</script>
