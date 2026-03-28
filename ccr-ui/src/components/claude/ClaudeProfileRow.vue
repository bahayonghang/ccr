<template>
  <article
    class="relative overflow-hidden rounded-[26px] border border-border-default/65 bg-bg-elevated/88 px-5 py-5 shadow-[0_16px_28px_rgba(0,0,0,0.08),inset_0_1px_0_rgba(255,255,255,0.04)] backdrop-blur-xl transition-[border-color,transform,box-shadow,background-color] duration-200 hover:-translate-y-0.5 hover:border-border-strong hover:shadow-[0_20px_36px_rgba(0,0,0,0.12)]"
    :class="profile.is_current ? 'border-accent-secondary/34 bg-accent-secondary/[0.045] shadow-[0_18px_38px_rgba(96,70,160,0.16),inset_0_1px_0_rgba(255,255,255,0.06)]' : ''"
  >
    <div
      class="absolute bottom-4 left-0 top-4 w-[3px] rounded-full bg-border-default/45"
      :class="profile.is_current ? 'bg-accent-secondary/80 shadow-[0_0_18px_rgba(96,70,160,0.45)]' : (profile.enabled !== false ? 'bg-accent-success/55' : 'bg-accent-danger/55')"
    />

    <div class="flex flex-col gap-4">
      <div class="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
        <div class="min-w-0 space-y-3 pl-1">
          <div class="flex flex-wrap items-center gap-2.5">
            <h3
              class="max-w-full truncate text-lg font-semibold tracking-tight text-text-primary"
              :title="profile.name"
            >
              {{ profile.name }}
            </h3>

            <span
              class="inline-flex min-h-[28px] items-center rounded-full px-3 py-1 text-xs font-medium"
              :class="stateBadgeClass"
            >
              {{ stateLabel }}
            </span>

            <span class="inline-flex min-h-[28px] items-center rounded-full border border-border-default/50 bg-bg-elevated/72 px-3 py-1 text-xs text-text-secondary">
              {{ providerLabelValue }}
            </span>

            <span
              v-if="profile.provider_type"
              class="inline-flex min-h-[28px] items-center rounded-full border border-border-default/40 bg-bg-surface/72 px-3 py-1 text-xs text-text-muted"
            >
              {{ profile.provider_type }}
            </span>
          </div>

          <p
            class="max-w-4xl text-sm leading-6 text-text-secondary"
            :class="profile.description ? '' : 'text-text-muted'"
          >
            {{ profile.description || $t('claudeProfiles.descriptionFallback') }}
          </p>
        </div>

        <div class="self-start rounded-full border border-border-default/45 bg-bg-surface/72 p-1.5 shadow-[inset_0_1px_0_rgba(255,255,255,0.03)]">
          <div class="flex flex-wrap items-center gap-2">
            <span
              v-if="profile.is_current"
              class="inline-flex min-h-[40px] items-center rounded-2xl border border-accent-secondary/24 bg-accent-secondary/10 px-4 py-2 text-sm font-medium text-accent-secondary"
            >
              {{ $t('claudeProfiles.currentlyActive') }}
            </span>
            <button
              v-else
              type="button"
              class="inline-flex min-h-[40px] items-center justify-center rounded-2xl border border-accent-secondary/28 bg-accent-secondary/10 px-4 py-2 text-sm font-medium text-accent-secondary transition-colors hover:bg-accent-secondary/16 focus:outline-none focus:ring-2 focus:ring-accent-secondary/20"
              @click="$emit('apply')"
            >
              {{ $t('claudeProfiles.applyProfile') }}
            </button>

            <button
              type="button"
              class="inline-flex min-h-[40px] items-center justify-center rounded-2xl border border-border-default/50 bg-bg-surface px-3 text-sm text-text-secondary transition-colors hover:border-border-default hover:bg-bg-elevated hover:text-text-primary focus:outline-none focus:ring-2 focus:ring-accent-secondary/20"
              :title="$t('claudeProfiles.editTooltip')"
              @click="$emit('edit')"
            >
              <SIcon
                name="Pencil"
                size="w-4 h-4"
              />
            </button>

            <button
              type="button"
              class="inline-flex min-h-[40px] items-center justify-center rounded-2xl border border-border-default/50 bg-bg-surface px-3 text-sm text-text-secondary transition-colors hover:border-accent-danger/30 hover:bg-accent-danger/10 hover:text-accent-danger focus:outline-none focus:ring-2 focus:ring-accent-danger/20"
              :title="$t('claudeProfiles.deleteTooltip')"
              @click="$emit('delete')"
            >
              <SIcon
                name="Trash2"
                size="w-4 h-4"
              />
            </button>
          </div>
        </div>
      </div>

      <dl class="grid gap-x-6 gap-y-3 rounded-[22px] border border-border-default/45 bg-bg-surface/62 px-4 py-4 md:grid-cols-2 2xl:grid-cols-4">
        <div
          v-for="item in detailItems"
          :key="item.label"
          class="min-w-0"
        >
          <dt class="text-[11px] font-semibold uppercase tracking-[0.2em] text-text-muted">
            {{ item.label }}
          </dt>
          <dd
            class="mt-1.5 truncate text-sm text-text-primary"
            :class="item.mono ? 'font-mono text-[13px]' : ''"
            :title="item.fullValue"
          >
            {{ item.value }}
          </dd>
        </div>
      </dl>

      <div
        v-if="profile.tags?.length"
        class="flex flex-wrap gap-2 rounded-[20px] border border-border-default/38 bg-bg-surface/52 px-4 py-3"
      >
        <span
          v-for="tag in profile.tags"
          :key="tag"
          class="inline-flex min-h-[28px] items-center rounded-full border border-border-default/45 bg-bg-elevated/60 px-3 py-1 text-xs text-text-secondary"
        >
          #{{ tag }}
        </span>
      </div>
    </div>
  </article>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type { ClaudeProfile } from '@/types'

const props = defineProps<{
  profile: ClaudeProfile
}>()

defineEmits<{
  edit: []
  delete: []
  apply: []
}>()

const { t } = useI18n()

const displayValue = (value?: string | null): string => value?.trim() || t('claudeProfiles.notSet')

const providerLabelValue = computed(() => displayValue(props.profile.provider || t('claudeProfiles.providerUnset')))

const stateLabel = computed(() => {
  if (props.profile.is_current) return t('claudeProfiles.currentBadge')
  return props.profile.enabled !== false ? t('claudeProfiles.enabledText') : t('claudeProfiles.disabledText')
})

const stateBadgeClass = computed(() => {
  if (props.profile.is_current) {
    return 'border border-accent-secondary/24 bg-accent-secondary/10 text-accent-secondary'
  }

  return props.profile.enabled !== false
    ? 'bg-accent-success/10 text-accent-success'
    : 'bg-accent-danger/10 text-accent-danger'
})

const detailItems = computed(() => {
  const items = [
    {
      label: t('claudeProfiles.baseUrlLabel'),
      value: displayValue(props.profile.base_url),
      fullValue: displayValue(props.profile.base_url),
      mono: true,
    },
    {
      label: t('claudeProfiles.modelLabel'),
      value: displayValue(props.profile.model),
      fullValue: displayValue(props.profile.model),
      mono: true,
    },
  ]

  if (props.profile.small_fast_model?.trim()) {
    items.push({
      label: t('claudeProfiles.smallFastModelLabel'),
      value: props.profile.small_fast_model,
      fullValue: props.profile.small_fast_model,
      mono: true,
    })
  }

  if (props.profile.account?.trim()) {
    items.push({
      label: t('claudeProfiles.accountLabel'),
      value: props.profile.account,
      fullValue: props.profile.account,
      mono: false,
    })
  }

  return items
})
</script>
