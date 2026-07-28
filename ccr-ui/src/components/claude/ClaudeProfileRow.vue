<template>
  <article
    class="relative overflow-hidden rounded-xl border bg-bg-elevated px-4 py-3.5 shadow-[0_14px_24px_rgba(0,0,0,0.08)] transition-[border-color,transform,box-shadow,background-color] duration-200 hover:-translate-y-0.5 hover:border-border-strong hover:shadow-[0_18px_30px_rgba(0,0,0,0.1)]"
    :class="profile.is_current
      ? 'border-transparent'
      : 'border-border-default/65'"
    :style="profile.is_current ? currentCardStyle : {}"
  >
    <!-- 左侧状态条 -->
    <div
      class="absolute bottom-3 left-0 top-3 w-1 rounded-full transition-all duration-300"
      :class="statusBarClass"
      :style="profile.is_current ? { backgroundColor: `rgb(var(${providerColor.rgbVar}))`, boxShadow: `0 0 12px rgb(var(${providerColor.rgbVar}) / 0.45)` } : {}"
    />

    <div class="flex flex-col gap-3">
      <div class="flex flex-col gap-3 xl:flex-row xl:items-start xl:justify-between">
        <div class="min-w-0 space-y-2 pl-1">
          <div class="flex flex-wrap items-center gap-2">
            <!-- eslint-disable vue/no-v-html -- highlightedName 由 claudeProfiles 转义后仅注入 <mark> -->
            <h3
              class="max-w-full truncate text-[1.12rem] font-semibold tracking-tight text-text-primary"
              :title="profile.name"
              v-html="highlightedName"
            />
            <!-- eslint-enable vue/no-v-html -->

            <span
              class="inline-flex min-h-[26px] items-center rounded-full px-2.5 py-1 text-[11px] font-medium"
              :class="stateBadgeClass"
            >
              {{ stateLabel }}
            </span>

            <span
              class="inline-flex min-h-[26px] items-center gap-1.5 rounded-full border border-border-default/50 bg-bg-elevated px-2.5 py-1 text-[11px]"
              :style="{ color: `rgb(var(${providerColor.rgbVar}))` }"
            >
              <span
                class="h-1.5 w-1.5 rounded-full"
                :style="{ backgroundColor: `rgb(var(${providerColor.rgbVar}))` }"
              />
              {{ providerLabelValue }}
            </span>

            <span
              v-if="profile.provider_type"
              class="inline-flex min-h-[26px] items-center rounded-full border border-border-default/40 bg-bg-surface px-2.5 py-1 text-[11px] text-text-muted"
            >
              {{ profile.provider_type }}
            </span>

            <span
              v-for="tag in compactTags"
              :key="tag"
              class="inline-flex min-h-[24px] items-center rounded-full border border-accent-secondary/15 bg-accent-secondary/6 px-2 py-0.5 text-[11px] text-accent-secondary/85"
            >
              <span class="opacity-50">#</span>{{ tag }}
            </span>
          </div>
        </div>

        <div class="flex shrink-0 items-center gap-2 self-start">
          <span
            v-if="profile.is_current"
            class="inline-flex min-h-[36px] items-center gap-2 rounded-xl px-3.5 py-2 text-sm font-medium"
            :style="{
              backgroundColor: `rgb(var(${providerColor.rgbVar}) / 0.1)`,
              color: `rgb(var(${providerColor.rgbVar}))`,
            }"
          >
            <span class="relative flex h-2 w-2">
              <span
                class="absolute inline-flex h-full w-full rounded-full opacity-60 animate-ping"
                :style="{ backgroundColor: `rgb(var(${providerColor.rgbVar}))` }"
              />
              <span
                class="relative inline-flex h-2 w-2 rounded-full"
                :style="{ backgroundColor: `rgb(var(${providerColor.rgbVar}))` }"
              />
            </span>
            {{ currentStatusLabel }}
          </span>
          <button
            v-else
            type="button"
            :disabled="profile.enabled === false"
            class="inline-flex h-7 items-center justify-center gap-1.5 rounded-lg border px-2.5 text-xs font-medium transition-all duration-200 hover:shadow-md active:scale-[0.97] focus:outline-none focus:ring-2 focus:ring-offset-1"
            :class="profile.enabled === false ? 'cursor-not-allowed opacity-55 hover:shadow-none active:scale-100' : ''"
            :style="{
              background: `linear-gradient(to bottom, rgb(var(${providerColor.rgbVar}) / 0.14), rgb(var(${providerColor.rgbVar}) / 0.08))`,
              borderColor: `rgb(var(${providerColor.rgbVar}) / 0.28)`,
              color: `rgb(var(${providerColor.rgbVar}))`,
              '--tw-ring-color': `rgb(var(${providerColor.rgbVar}) / 0.2)`,
            }"
            @click="$emit('apply')"
          >
            <SIcon
              name="Play"
              size="w-3 h-3"
            />
            {{ $t('claudeProfiles.applyProfile') }}
          </button>

          <button
            type="button"
            class="inline-flex h-9 w-9 items-center justify-center rounded-xl border border-border-default/50 bg-bg-surface text-text-secondary transition-colors hover:border-border-default hover:bg-bg-elevated hover:text-text-primary focus:outline-none focus:ring-2 focus:ring-accent-secondary/20"
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
            class="inline-flex h-9 w-9 items-center justify-center rounded-xl border border-border-default/50 bg-bg-surface text-text-secondary transition-colors hover:border-accent-danger/30 hover:bg-accent-danger/10 hover:text-accent-danger focus:outline-none focus:ring-2 focus:ring-accent-danger/20"
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

      <div class="grid gap-x-4 gap-y-2 border-t border-border-default/35 pt-3 md:grid-cols-2 xl:grid-cols-[minmax(0,1.45fr)_repeat(4,minmax(0,0.9fr))]">
        <div
          class="min-w-0 md:col-span-2 xl:col-span-1"
        >
          <p class="text-[11px] font-semibold uppercase tracking-[0.2em] text-text-muted">
            {{ $t('claudeProfiles.descLabel') }}
          </p>
          <!-- eslint-disable vue/no-v-html -- highlightedDescription 由 claudeProfiles 转义后仅注入 <mark> -->
          <p
            class="mt-1 line-clamp-2 text-sm leading-5"
            :class="profile.description ? 'text-text-secondary' : 'text-text-muted'"
            v-html="highlightedDescription"
          />
          <!-- eslint-enable vue/no-v-html -->
        </div>

        <div
          v-for="item in detailItems"
          :key="item.label"
          class="min-w-0"
        >
          <dt class="text-[11px] font-semibold uppercase tracking-[0.2em] text-text-muted">
            {{ item.label }}
          </dt>
          <dd
            class="mt-1 truncate text-sm"
            :class="item.mono ? 'font-mono text-[13px]' : ''"
            :title="item.fullValue"
          >
            <template v-if="item.type === 'url'">
              <span class="text-text-primary">{{ truncateMiddle(item.value, 22, 14) }}</span>
            </template>
            <template v-else-if="item.type === 'model' && item.value !== $t('claudeProfiles.notSet')">
              <span
                class="font-medium"
                :style="{ color: `rgb(var(${providerColor.rgbVar}))` }"
              >{{ item.value }}</span>
            </template>
            <template v-else>
              <span class="text-text-primary">{{ item.value }}</span>
            </template>
          </dd>
        </div>
      </div>
    </div>
  </article>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type { ClaudeProfile } from '@/types'
import { highlightSearchMatch, type ProviderColorConfig } from '@/utils/claudeProfiles'
import { truncateMiddle } from '@/utils/text'

const props = defineProps<{
  profile: ClaudeProfile
  providerColor: ProviderColorConfig
  searchQuery?: string
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
  if (props.profile.is_current) {
    return props.profile.enabled === false
      ? t('claudeProfiles.currentDisabled')
      : t('claudeProfiles.currentBadge')
  }
  return props.profile.enabled !== false ? t('claudeProfiles.enabledText') : t('claudeProfiles.disabledText')
})

const stateBadgeClass = computed(() => {
  if (props.profile.is_current) {
    return props.profile.enabled === false
      ? 'border border-accent-danger/24 bg-accent-danger/10 text-accent-danger'
      : 'border border-accent-secondary/24 bg-accent-secondary/10 text-accent-secondary'
  }
  return props.profile.enabled !== false
    ? 'bg-accent-success/10 text-accent-success'
    : 'bg-accent-danger/10 text-accent-danger'
})

const currentStatusLabel = computed(() => (
  props.profile.enabled === false
    ? t('claudeProfiles.currentDisabled')
    : t('claudeProfiles.currentlyActive')
))

// 左侧状态条样式 (非 current 使用 class, current 使用 inline style)
const statusBarClass = computed(() => {
  if (props.profile.is_current) return '' // current 由 :style 控制
  return props.profile.enabled !== false ? 'bg-accent-success/55' : 'bg-accent-danger/55'
})

// 当前 profile 卡片的动态样式
const currentCardStyle = computed(() => ({
  borderColor: `rgb(var(${props.providerColor.rgbVar}) / 0.3)`,
  backgroundColor: `rgb(var(${props.providerColor.rgbVar}) / 0.03)`,
  boxShadow: `0 18px 38px rgb(var(${props.providerColor.rgbVar}) / 0.1)`,
}))

// 搜索高亮
const highlightedName = computed(() =>
  highlightSearchMatch(props.profile.name, props.searchQuery || ''),
)

const highlightedDescription = computed(() =>
  highlightSearchMatch(
    props.profile.description || t('claudeProfiles.descriptionFallback'),
    props.searchQuery || '',
  ),
)

const compactTags = computed(() => props.profile.tags ?? [])

// 详情字段
interface DetailItem {
  label: string
  value: string
  fullValue: string
  mono: boolean
  type: 'url' | 'model' | 'text'
}

const detailItems = computed<DetailItem[]>(() => {
  const baseUrlValue = displayValue(props.profile.base_url)
  const items: DetailItem[] = [
    {
      label: t('claudeProfiles.baseUrlLabel'),
      value: baseUrlValue,
      fullValue: baseUrlValue,
      mono: true,
      type: 'url',
    },
  ]

  const advancedFields: Array<{ value: string | null | undefined, label: string }> = [
    { value: props.profile.default_opus_model, label: t('claudeProfiles.defaultOpusModelLabel') },
    { value: props.profile.default_sonnet_model, label: t('claudeProfiles.defaultSonnetModelLabel') },
    { value: props.profile.default_haiku_model, label: t('claudeProfiles.defaultHaikuModelLabel') },
    { value: props.profile.subagent_model, label: t('claudeProfiles.subagentModelLabel') },
  ]

  for (const field of advancedFields) {
    if (field.value?.trim()) {
      items.push({
        label: field.label,
        value: field.value,
        fullValue: field.value,
        mono: true,
        type: 'model',
      })
    }
  }

  if (props.profile.effort_level?.trim()) {
    items.push({
      label: t('claudeProfiles.effortLevelLabel'),
      value: props.profile.effort_level,
      fullValue: props.profile.effort_level,
      mono: true,
      type: 'text',
    })
  }

  if (props.profile.account?.trim()) {
    items.push({
      label: t('claudeProfiles.accountLabel'),
      value: props.profile.account,
      fullValue: props.profile.account,
      mono: false,
      type: 'text',
    })
  }

  return items
})
</script>
