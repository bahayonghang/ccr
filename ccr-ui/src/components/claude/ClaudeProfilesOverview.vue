<template>
  <div class="space-y-4">
    <div class="grid gap-3 lg:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)]">
      <section class="rounded-[24px] border border-border-default/50 bg-bg-surface/62 px-4 py-4">
        <p class="text-xs font-semibold uppercase tracking-[0.24em] text-text-muted">
          {{ $t('claudeProfiles.currentProfile') }}
        </p>
        <div class="mt-3 flex flex-wrap items-center gap-3">
          <p
            class="max-w-full truncate text-xl font-semibold tracking-tight text-text-primary"
            :title="currentProfileName || $t('claudeProfiles.notSet')"
          >
            {{ currentProfileName || $t('claudeProfiles.notSet') }}
          </p>
          <span
            v-if="currentProfileName"
            class="inline-flex items-center gap-2 rounded-full border border-accent-secondary/25 bg-accent-secondary/10 px-3 py-1 text-xs font-medium text-accent-secondary"
          >
            <span class="h-2 w-2 rounded-full bg-current opacity-80" />
            {{ $t('claudeProfiles.currentlyActive') }}
          </span>
        </div>
      </section>

      <dl class="grid gap-3 sm:grid-cols-2">
        <div class="rounded-[24px] border border-border-default/45 bg-bg-surface/56 px-4 py-4">
          <dt class="text-xs font-semibold uppercase tracking-[0.22em] text-text-muted">
            {{ $t('claudeProfiles.totalCount') }}
          </dt>
          <dd class="mt-2 text-2xl font-semibold tracking-tight text-text-primary">
            {{ totalProfiles }}
          </dd>
        </div>

        <div class="rounded-[24px] border border-border-default/45 bg-bg-surface/56 px-4 py-4">
          <dt class="text-xs font-semibold uppercase tracking-[0.22em] text-text-muted">
            {{ $t('claudeProfiles.enabledCount') }}
          </dt>
          <dd class="mt-2 text-2xl font-semibold tracking-tight text-text-primary">
            {{ enabledProfilesCount }}
          </dd>
        </div>
      </dl>
    </div>

    <section
      v-if="quickSwitchProfiles.length > 0"
      class="rounded-[24px] border border-border-default/45 bg-bg-surface/56 px-4 py-4"
    >
      <div class="flex flex-col gap-2 md:flex-row md:items-end md:justify-between">
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
        <p class="text-xs text-text-muted">
          {{ totalProfiles }} / {{ enabledProfilesCount }}
        </p>
      </div>

      <div class="mt-4 flex flex-wrap gap-2.5">
        <button
          v-for="profile in quickSwitchProfiles"
          :key="profile.name"
          type="button"
          class="inline-flex min-h-[40px] items-center gap-2 rounded-full border px-3.5 py-2 text-sm font-medium transition-[background-color,border-color,color,transform] duration-200"
          :class="profile.is_current
            ? 'border-accent-secondary/28 bg-accent-secondary/12 text-accent-secondary'
            : 'border-border-default/50 bg-bg-elevated/60 text-text-secondary hover:border-border-default hover:bg-bg-elevated/92 hover:text-text-primary'"
          @click="$emit('apply', profile.name)"
        >
          <span
            class="h-2 w-2 rounded-full"
            :class="profile.is_current ? 'bg-current opacity-85' : (profile.enabled !== false ? 'bg-accent-success/80' : 'bg-accent-danger/80')"
          />
          <span class="max-w-[18rem] truncate">{{ profile.name }}</span>
        </button>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import type { ClaudeProfile } from '@/types'

defineProps<{
  currentProfileName: string | null
  enabledProfilesCount: number
  quickSwitchProfiles: ClaudeProfile[]
  totalProfiles: number
}>()

defineEmits<{
  apply: [name: string]
}>()
</script>
