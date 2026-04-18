<template>
  <div class="space-y-3.5">
    <div class="grid gap-3 xl:grid-cols-[minmax(0,1.55fr)_minmax(0,1fr)]">
      <section class="rounded-[26px] border border-border-default/50 bg-bg-surface/64 px-4 py-4 shadow-[0_14px_30px_rgba(15,12,18,0.06)]">
        <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
          <div class="min-w-0">
            <p class="text-xs font-semibold uppercase tracking-[0.22em] text-text-muted">
              {{ $t('claudeProfiles.currentProfile') }}
            </p>
            <div class="mt-2 flex flex-wrap items-center gap-2.5">
              <p
                class="max-w-full truncate text-[1.35rem] font-semibold tracking-tight text-text-primary"
                :title="currentProfileName || $t('claudeProfiles.notSet')"
              >
                {{ currentProfileName || $t('claudeProfiles.notSet') }}
              </p>
              <span
                v-if="currentProfileName"
                class="inline-flex items-center gap-2 rounded-full border border-accent-secondary/24 bg-accent-secondary/10 px-3 py-1 text-xs font-medium text-accent-secondary"
              >
                <span class="h-1.5 w-1.5 rounded-full bg-current opacity-85" />
                {{ currentProfileStatusLabel }}
              </span>
            </div>
            <p class="mt-2 max-w-3xl text-sm leading-5 text-text-secondary">
              {{ currentProfileDescription }}
            </p>
          </div>

          <div
            v-if="currentProfileChips.length > 0"
            class="flex flex-wrap gap-2"
          >
            <span
              v-for="chip in currentProfileChips"
              :key="chip"
              class="inline-flex min-h-[28px] items-center rounded-full border border-border-default/45 bg-bg-elevated/72 px-3 py-1 text-xs font-medium text-text-secondary"
            >
              {{ chip }}
            </span>
          </div>
        </div>
      </section>

      <dl class="grid gap-3 sm:grid-cols-2">
        <article
          v-for="tile in overviewTiles"
          :key="tile.label"
          class="relative overflow-hidden rounded-[22px] border border-border-default/45 bg-bg-surface/58 px-3.5 py-3.5 shadow-[0_12px_28px_rgba(12,10,16,0.05)]"
        >
          <div class="flex items-start gap-3">
            <span
              class="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-xl border"
              :class="tile.iconToneClass"
            >
              <SIcon
                :name="tile.icon"
                size="w-4 h-4"
              />
            </span>
            <div class="min-w-0">
              <dt class="text-[11px] font-semibold uppercase tracking-[0.2em] text-text-muted">
                {{ tile.label }}
              </dt>
              <dd
                class="mt-1 text-[1.45rem] font-semibold tracking-tight"
                :class="tile.valueToneClass"
              >
                {{ tile.value }}
              </dd>
              <p class="mt-1 text-xs leading-5 text-text-secondary">
                {{ tile.detail }}
              </p>
            </div>
          </div>
        </article>
      </dl>
    </div>

    <section class="flex flex-wrap gap-2 rounded-[20px] border border-border-default/42 bg-bg-surface/52 px-3.5 py-3">
      <span
        v-for="item in ribbonItems"
        :key="item.label"
        class="inline-flex min-h-[30px] items-center gap-2 rounded-full border border-border-default/42 bg-bg-elevated/64 px-3 py-1 text-xs text-text-secondary"
      >
        <span class="font-medium text-text-muted">{{ item.label }}</span>
        <strong class="font-semibold text-text-primary">{{ item.value }}</strong>
      </span>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type { ClaudeProfile } from '@/types'
import type { ClaudeProfilesOverviewSummary } from '@/utils/claudeProfiles'

const props = defineProps<{
  currentProfile: ClaudeProfile | null
  providerUnsetLabel: string
  summary: ClaudeProfilesOverviewSummary
}>()

const { t } = useI18n()

const currentProfileName = computed(() => props.currentProfile?.name ?? null)
const currentProfileStatusLabel = computed(() => (
  props.currentProfile?.enabled === false
    ? t('claudeProfiles.currentDisabled')
    : t('claudeProfiles.currentlyActive')
))

const formatAuthMode = (authMode?: string | null) => {
  if (authMode === 'api_key') return t('claudeProfiles.authModeApiKey')
  if (authMode === 'subscription') return t('claudeProfiles.authModeSubscription')
  return null
}

const currentProfileDescription = computed(() => {
  const description = props.currentProfile?.description?.trim()
  if (description) return description

  return props.currentProfile
    ? t('claudeProfiles.currentProfileSummaryFallback')
    : t('claudeProfiles.currentProfileMissingHint')
})

const currentProfileChips = computed(() => {
  const profile = props.currentProfile
  if (!profile) return []

  const chips: string[] = [
    profile.provider?.trim() || props.providerUnsetLabel,
  ]

  if (profile.provider_type?.trim()) {
    chips.push(profile.provider_type.trim())
  }

  if (profile.account?.trim()) {
    chips.push(`@${profile.account.trim()}`)
  }

  const authMode = formatAuthMode(profile.auth_mode)
  if (authMode) {
    chips.push(authMode)
  }

  return chips
})

const overviewTiles = computed(() => ([
  {
    label: t('claudeProfiles.overviewProfilesLabel'),
    value: String(props.summary.totalProfiles),
    detail: t('claudeProfiles.overviewProfilesDetail', {
      enabled: props.summary.enabledProfilesCount,
      disabled: props.summary.disabledProfilesCount,
    }),
    icon: 'Layers',
    iconToneClass: 'border-accent-secondary/18 bg-accent-secondary/10 text-accent-secondary',
    valueToneClass: 'text-text-primary',
  },
  {
    label: t('claudeProfiles.overviewProvidersLabel'),
    value: String(props.summary.providerSectionsCount),
    detail: t('claudeProfiles.overviewProvidersDetail', {
      sections: props.summary.providerSectionsCount,
      missing: props.summary.unsetProviderProfilesCount,
    }),
    icon: 'PanelLeftOpen',
    iconToneClass: 'border-info/18 bg-info/10 text-info',
    valueToneClass: 'text-info',
  },
  {
    label: t('claudeProfiles.overviewModelsLabel'),
    value: String(props.summary.configuredModelProfilesCount),
    detail: t('claudeProfiles.overviewModelsDetail', {
      primary: props.summary.configuredModelProfilesCount,
      fast: props.summary.configuredFastModelProfilesCount,
    }),
    icon: 'Cpu',
    iconToneClass: 'border-accent-primary/18 bg-accent-primary/10 text-accent-primary',
    valueToneClass: 'text-accent-primary',
  },
  {
    label: t('claudeProfiles.overviewAccessLabel'),
    value: String(props.summary.accountProfilesCount),
    detail: t('claudeProfiles.overviewAccessDetail', {
      subscription: props.summary.subscriptionProfilesCount,
      apiKey: props.summary.apiKeyProfilesCount,
      accounts: props.summary.accountProfilesCount,
    }),
    icon: 'ShieldCheck',
    iconToneClass: 'border-accent-success/18 bg-accent-success/10 text-accent-success',
    valueToneClass: 'text-accent-success',
  },
]))

const ribbonItems = computed(() => ([
  {
    label: t('claudeProfiles.overviewRibbonCustomBaseUrl'),
    value: String(props.summary.customBaseUrlProfilesCount),
  },
  {
    label: t('claudeProfiles.overviewRibbonTagged'),
    value: String(props.summary.taggedProfilesCount),
  },
  {
    label: t('claudeProfiles.overviewRibbonMissingModel'),
    value: String(props.summary.missingModelProfilesCount),
  },
  {
    label: t('claudeProfiles.overviewRibbonMissingAccount'),
    value: String(props.summary.missingAccountProfilesCount),
  },
]))
</script>
