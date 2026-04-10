<template>
  <div class="space-y-4">
    <!-- 统计区：当前 Profile + 总数 / 已启用 -->
    <div class="grid gap-3 lg:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)]">
      <!-- 当前活跃 Profile -->
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
            {{ currentProfileStatusLabel }}
          </span>
        </div>
      </section>

      <!-- 统计卡片 (含图标) -->
      <dl class="grid gap-3 sm:grid-cols-2">
        <div class="flex items-center gap-3 rounded-[24px] border border-border-default/45 bg-bg-surface/56 px-4 py-4">
          <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-accent-secondary/10 text-accent-secondary">
            <SIcon
              name="Layers"
              size="w-5 h-5"
            />
          </div>
          <div>
            <dt class="text-xs font-semibold uppercase tracking-[0.22em] text-text-muted">
              {{ $t('claudeProfiles.totalCount') }}
            </dt>
            <dd class="mt-0.5 text-2xl font-semibold tracking-tight text-text-primary">
              {{ totalProfiles }}
            </dd>
          </div>
        </div>

        <div class="flex items-center gap-3 rounded-[24px] border border-border-default/45 bg-bg-surface/56 px-4 py-4">
          <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-accent-success/10 text-accent-success">
            <SIcon
              name="CheckCircle"
              size="w-5 h-5"
            />
          </div>
          <div>
            <dt class="text-xs font-semibold uppercase tracking-[0.22em] text-text-muted">
              {{ $t('claudeProfiles.enabledCount') }}
            </dt>
            <dd class="mt-0.5 text-2xl font-semibold tracking-tight text-text-primary">
              {{ enabledProfilesCount }}
            </dd>
          </div>
        </div>
      </dl>
    </div>

    <!-- 快速切换：按 Provider 分组 -->
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
          {{ enabledProfilesCount }} / {{ totalProfiles }}
        </p>
      </div>

      <div
        class="mt-4 space-y-3"
        :class="totalProfiles > 20 ? 'max-h-[260px] overflow-y-auto pr-1' : ''"
      >
        <div
          v-for="group in groupedProfiles"
          :key="group.providerKey"
        >
          <!-- Provider 分组标签 -->
          <div class="mb-2 flex items-center gap-2">
            <span
              class="h-1.5 w-1.5 shrink-0 rounded-full"
              :style="{ backgroundColor: `rgb(var(${group.color.rgbVar}))` }"
            />
            <span
              class="text-[10px] font-semibold uppercase tracking-[0.2em]"
              :style="{ color: `rgb(var(${group.color.rgbVar}) / 0.7)` }"
            >
              {{ group.label }}
            </span>
            <span class="text-[10px] text-text-ghost">
              {{ group.profiles.length }}
            </span>
          </div>

          <!-- 组内 Profile pills -->
          <div class="flex flex-wrap gap-1.5">
            <button
              v-for="profile in group.profiles"
              :key="profile.name"
              type="button"
              :disabled="profile.is_current || profile.enabled === false"
              class="inline-flex min-h-[32px] items-center gap-1.5 rounded-full border px-3 py-1 text-[0.78rem] font-medium transition-[background-color,border-color,color,transform] duration-200 hover:-translate-y-px"
              :class="profile.is_current
                ? ''
                : (profile.enabled === false
                  ? 'cursor-not-allowed border-border-default/35 bg-bg-elevated/34 text-text-muted opacity-60'
                  : 'border-border-default/50 bg-bg-elevated/60 text-text-secondary hover:border-border-default hover:bg-bg-elevated/92 hover:text-text-primary')"
              :style="profile.is_current ? {
                borderColor: `rgb(var(${group.color.rgbVar}) / 0.28)`,
                backgroundColor: `rgb(var(${group.color.rgbVar}) / 0.12)`,
                color: `rgb(var(${group.color.rgbVar}))`,
              } : {}"
              @click="$emit('apply', profile.name)"
            >
              <span
                class="h-1.5 w-1.5 rounded-full"
                :class="profile.is_current ? '' : (profile.enabled !== false ? 'bg-accent-success/80' : 'bg-accent-danger/80')"
                :style="profile.is_current ? { backgroundColor: `rgb(var(${group.color.rgbVar}))` } : {}"
              />
              <span class="max-w-[16rem] truncate">{{ profile.name }}</span>
            </button>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import { translateWithFallback } from '@/i18n/formatMessage'
import type { ClaudeProfile } from '@/types'
import { groupProfilesByProvider, type ProviderGroup } from '@/utils/claudeProfiles'

const props = defineProps<{
  currentProfile: ClaudeProfile | null
  enabledProfilesCount: number
  quickSwitchProfiles: ClaudeProfile[]
  totalProfiles: number
}>()

defineEmits<{
  apply: [name: string]
}>()

const { t } = useI18n()

const currentProfileName = computed(() => props.currentProfile?.name ?? null)
const currentProfileStatusLabel = computed(() => (
  props.currentProfile?.enabled === false
    ? t('claudeProfiles.currentDisabled')
    : t('claudeProfiles.currentlyActive')
))

const groupedProfiles = computed<ProviderGroup[]>(() =>
  groupProfilesByProvider(
    props.quickSwitchProfiles,
    translateWithFallback(
      t,
      'claudeProfiles.providerUnset',
      '未设置 Provider',
    ),
  ),
)
</script>
