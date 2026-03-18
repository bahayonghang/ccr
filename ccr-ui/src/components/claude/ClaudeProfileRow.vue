<template>
  <article
    class="rounded-[28px] border border-border-default/50 bg-bg-surface/76 p-5 shadow-lg shadow-black/5 backdrop-blur-xl transition-[border-color,transform,box-shadow] duration-200 hover:-translate-y-0.5 hover:border-border-default hover:shadow-xl"
    :class="profile.is_current ? 'border-accent-secondary/35 shadow-[0_16px_40px_rgba(96,70,160,0.12)]' : ''"
  >
    <div class="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
      <div class="flex min-w-0 items-start gap-4">
        <div
          class="flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl"
          :class="profile.is_current ? 'bg-accent-secondary/12 text-accent-secondary' : 'bg-bg-elevated text-text-secondary'"
        >
          <SIcon
            name="User"
            size="w-5 h-5"
          />
        </div>

        <div class="min-w-0">
          <div class="flex flex-wrap items-center gap-2">
            <h3 class="max-w-full truncate text-lg font-semibold text-text-primary">
              {{ profile.name }}
            </h3>
            <span
              v-if="profile.is_current"
              class="rounded-full bg-accent-secondary/10 px-2.5 py-1 text-xs font-medium text-accent-secondary"
            >
              {{ $t('claudeProfiles.currentBadge') }}
            </span>
            <span
              class="rounded-full px-2.5 py-1 text-xs font-medium"
              :class="profile.enabled !== false ? 'bg-accent-success/10 text-accent-success' : 'bg-accent-danger/10 text-accent-danger'"
            >
              {{ profile.enabled !== false ? $t('claudeProfiles.enabledText') : $t('claudeProfiles.disabledText') }}
            </span>
          </div>

          <p
            class="mt-2 max-w-3xl text-sm leading-6 text-text-secondary"
            :class="profile.description ? '' : 'text-text-muted'"
          >
            {{ profile.description || $t('claudeProfiles.descriptionFallback') }}
          </p>
        </div>
      </div>

      <div class="flex shrink-0 items-center gap-2 self-start">
        <button
          type="button"
          class="inline-flex min-h-[40px] items-center justify-center rounded-xl border border-border-default/50 bg-bg-surface px-3 text-sm text-text-secondary transition-colors hover:border-border-default hover:bg-bg-elevated hover:text-text-primary focus:outline-none focus:ring-2 focus:ring-accent-secondary/20"
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
          class="inline-flex min-h-[40px] items-center justify-center rounded-xl border border-border-default/50 bg-bg-surface px-3 text-sm text-text-secondary transition-colors hover:border-accent-danger/30 hover:bg-accent-danger/10 hover:text-accent-danger focus:outline-none focus:ring-2 focus:ring-accent-danger/20"
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

    <div class="mt-5 grid grid-cols-1 gap-3 md:grid-cols-2 2xl:grid-cols-4">
      <div
        v-for="field in fieldItems"
        :key="field.label"
        class="rounded-2xl border border-border-default/45 bg-bg-surface/55 px-4 py-3"
      >
        <p class="text-xs font-medium uppercase tracking-[0.18em] text-text-muted">
          {{ field.label }}
        </p>
        <p
          class="mt-2 truncate text-sm text-text-primary"
          :class="field.mono ? 'font-mono' : ''"
          :title="field.fullValue"
        >
          {{ field.value }}
        </p>
      </div>
    </div>

    <div class="mt-4 flex flex-col gap-3 border-t border-border-default/45 pt-4 lg:flex-row lg:items-center lg:justify-between">
      <div class="flex flex-wrap items-center gap-2">
        <span
          class="rounded-full border border-border-default/50 bg-bg-surface px-3 py-1 text-xs text-text-secondary"
          :title="providerTypeValue"
        >
          {{ $t('claudeProfiles.providerTypeChip', { value: providerTypeValue }) }}
        </span>
        <span
          v-if="profile.small_fast_model"
          class="rounded-full bg-accent-info/10 px-3 py-1 text-xs font-medium text-accent-info"
          :title="profile.small_fast_model"
        >
          {{ $t('claudeProfiles.smallFastModelBadge') }} · {{ profile.small_fast_model }}
        </span>
        <span
          v-for="tag in profile.tags || []"
          :key="tag"
          class="rounded-full border border-border-default/50 bg-bg-surface px-3 py-1 text-xs text-text-secondary"
        >
          #{{ tag }}
        </span>
      </div>

      <div class="flex items-center gap-3">
        <div
          v-if="profile.is_current"
          class="rounded-2xl border border-accent-secondary/20 bg-accent-secondary/8 px-4 py-2 text-sm font-medium text-accent-secondary/80"
        >
          {{ $t('claudeProfiles.currentlyActive') }}
        </div>
        <button
          v-else
          type="button"
          class="inline-flex min-h-[42px] items-center justify-center rounded-2xl border border-accent-secondary/30 bg-accent-secondary/10 px-4 py-2 text-sm font-medium text-accent-secondary transition-colors hover:bg-accent-secondary/16 focus:outline-none focus:ring-2 focus:ring-accent-secondary/20"
          @click="$emit('apply')"
        >
          {{ $t('claudeProfiles.applyProfile') }}
        </button>
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

const providerTypeValue = computed(() => props.profile.provider_type?.trim() || t('claudeProfiles.notSet'))

const fieldItems = computed(() => [
  {
    label: t('claudeProfiles.baseUrlLabel'),
    value: props.profile.base_url || t('claudeProfiles.notSet'),
    fullValue: props.profile.base_url || t('claudeProfiles.notSet'),
    mono: true,
  },
  {
    label: t('claudeProfiles.modelLabel'),
    value: props.profile.model || t('claudeProfiles.notSet'),
    fullValue: props.profile.model || t('claudeProfiles.notSet'),
    mono: true,
  },
  {
    label: t('claudeProfiles.smallFastModelLabel'),
    value: props.profile.small_fast_model || t('claudeProfiles.notSet'),
    fullValue: props.profile.small_fast_model || t('claudeProfiles.notSet'),
    mono: true,
  },
  {
    label: t('claudeProfiles.accountLabel'),
    value: props.profile.account || t('claudeProfiles.notSet'),
    fullValue: props.profile.account || t('claudeProfiles.notSet'),
    mono: true,
  },
])
</script>
