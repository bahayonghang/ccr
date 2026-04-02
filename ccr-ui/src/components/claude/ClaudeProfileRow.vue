<template>
  <article
    class="relative overflow-hidden rounded-[26px] border bg-bg-elevated/88 px-5 py-5 shadow-[0_16px_28px_rgba(0,0,0,0.08),inset_0_1px_0_rgba(255,255,255,0.04)] backdrop-blur-xl transition-[border-color,transform,box-shadow,background-color] duration-200 hover:-translate-y-0.5 hover:border-border-strong hover:shadow-[0_20px_36px_rgba(0,0,0,0.12)]"
    :class="profile.is_current
      ? 'border-transparent'
      : 'border-border-default/65'"
    :style="profile.is_current ? currentCardStyle : {}"
  >
    <!-- 左侧状态条 -->
    <div
      class="absolute bottom-4 left-0 top-4 w-1 rounded-full transition-all duration-300"
      :class="statusBarClass"
      :style="profile.is_current ? { backgroundColor: `rgb(var(${providerColor.rgbVar}))`, boxShadow: `0 0 12px rgb(var(${providerColor.rgbVar}) / 0.45)` } : {}"
    />

    <div class="flex flex-col gap-4">
      <!-- 头部：名称 + badges + 操作按钮 -->
      <div class="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
        <div class="min-w-0 space-y-3 pl-1">
          <div class="flex flex-wrap items-center gap-2.5">
            <h3
              class="max-w-full truncate text-lg font-semibold tracking-tight text-text-primary"
              :title="profile.name"
              v-html="highlightedName"
            />

            <span
              class="inline-flex min-h-[28px] items-center rounded-full px-3 py-1 text-xs font-medium"
              :class="stateBadgeClass"
            >
              {{ stateLabel }}
            </span>

            <span
              class="inline-flex min-h-[28px] items-center gap-1.5 rounded-full border border-border-default/50 bg-bg-elevated/72 px-3 py-1 text-xs"
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
              class="inline-flex min-h-[28px] items-center rounded-full border border-border-default/40 bg-bg-surface/72 px-3 py-1 text-xs text-text-muted"
            >
              {{ profile.provider_type }}
            </span>
          </div>

          <p
            class="max-w-4xl text-sm leading-6"
            :class="profile.description ? 'text-text-secondary' : 'text-text-muted'"
            v-html="highlightedDescription"
          />
        </div>

        <!-- 操作按钮区 -->
        <div class="flex shrink-0 items-center gap-2 self-start">
          <!-- 主 CTA: Apply / 当前活跃标识 -->
          <span
            v-if="profile.is_current"
            class="inline-flex min-h-[40px] items-center gap-2 rounded-2xl px-4 py-2 text-sm font-medium"
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
            {{ $t('claudeProfiles.currentlyActive') }}
          </span>
          <button
            v-else
            type="button"
            class="inline-flex min-h-[40px] items-center justify-center gap-2 rounded-2xl px-4 py-2 text-sm font-medium transition-all duration-200 hover:shadow-lg active:scale-[0.97] focus:outline-none focus:ring-2 focus:ring-offset-1"
            :style="{
              background: `linear-gradient(to bottom, rgb(var(${providerColor.rgbVar}) / 0.14), rgb(var(${providerColor.rgbVar}) / 0.08))`,
              borderColor: `rgb(var(${providerColor.rgbVar}) / 0.28)`,
              color: `rgb(var(${providerColor.rgbVar}))`,
              '--tw-ring-color': `rgb(var(${providerColor.rgbVar}) / 0.2)`,
            }"
            style="border-width: 1px; border-style: solid"
            @click="$emit('apply')"
          >
            <SIcon
              name="Play"
              size="w-3.5 h-3.5"
            />
            {{ $t('claudeProfiles.applyProfile') }}
          </button>

          <!-- 次要操作 -->
          <button
            type="button"
            class="inline-flex h-10 w-10 items-center justify-center rounded-xl border border-border-default/50 bg-bg-surface text-text-secondary transition-colors hover:border-border-default hover:bg-bg-elevated hover:text-text-primary focus:outline-none focus:ring-2 focus:ring-accent-secondary/20"
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
            class="inline-flex h-10 w-10 items-center justify-center rounded-xl border border-border-default/50 bg-bg-surface text-text-secondary transition-colors hover:border-accent-danger/30 hover:bg-accent-danger/10 hover:text-accent-danger focus:outline-none focus:ring-2 focus:ring-accent-danger/20"
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

      <!-- 技术字段详情 -->
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
            class="mt-1.5 truncate text-sm"
            :class="item.mono ? 'font-mono text-[13px]' : ''"
            :title="item.fullValue"
          >
            <!-- base_url: 协议低亮度 + host 正常色 -->
            <template v-if="item.type === 'url' && item.parsedUrl">
              <span class="text-accent-info/60">{{ item.parsedUrl.protocol }}</span>
              <span class="text-text-primary">{{ item.parsedUrl.host }}</span>
              <span
                v-if="item.parsedUrl.path"
                class="text-text-muted"
              >{{ item.parsedUrl.path }}</span>
            </template>
            <!-- model: provider 色渲染 -->
            <template v-else-if="item.type === 'model' && item.value !== $t('claudeProfiles.notSet')">
              <span
                class="font-medium"
                :style="{ color: `rgb(var(${providerColor.rgbVar}))` }"
              >{{ item.value }}</span>
            </template>
            <!-- 其他字段 / 未设置 -->
            <template v-else>
              <span class="text-text-primary">{{ item.value }}</span>
            </template>
          </dd>
        </div>
      </dl>

      <!-- Tags -->
      <div
        v-if="profile.tags?.length"
        class="flex flex-wrap gap-2"
      >
        <span
          v-for="tag in profile.tags"
          :key="tag"
          class="inline-flex min-h-[26px] items-center rounded-full border border-accent-secondary/15 bg-accent-secondary/6 px-2.5 py-0.5 text-xs text-accent-secondary/85 transition-colors hover:bg-accent-secondary/10"
        >
          <span class="opacity-50">#</span>{{ tag }}
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
import { highlightSearchMatch, type ProviderColorConfig } from '@/utils/claudeProfiles'

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

// 左侧状态条样式 (非 current 使用 class, current 使用 inline style)
const statusBarClass = computed(() => {
  if (props.profile.is_current) return '' // current 由 :style 控制
  return props.profile.enabled !== false ? 'bg-accent-success/55' : 'bg-accent-danger/55'
})

// 当前 profile 卡片的动态样式
const currentCardStyle = computed(() => ({
  borderColor: `rgb(var(${props.providerColor.rgbVar}) / 0.3)`,
  backgroundColor: `rgb(var(${props.providerColor.rgbVar}) / 0.03)`,
  boxShadow: `0 18px 38px rgb(var(${props.providerColor.rgbVar}) / 0.1), inset 0 1px 0 rgba(255,255,255,0.06)`,
}))

/** 解析 URL 为 protocol / host / path 三段 */
interface ParsedUrl {
  protocol: string
  host: string
  path: string
}

const parseUrl = (url: string): ParsedUrl | null => {
  const match = url.match(/^(https?:\/\/)([^/]+)(\/.*)?$/)
  if (!match) return null
  return {
    protocol: match[1],
    host: match[2],
    path: match[3] || '',
  }
}

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

// 详情字段
interface DetailItem {
  label: string
  value: string
  fullValue: string
  mono: boolean
  type: 'url' | 'model' | 'text'
  parsedUrl?: ParsedUrl | null
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
      parsedUrl: props.profile.base_url?.trim() ? parseUrl(props.profile.base_url.trim()) : null,
    },
    {
      label: t('claudeProfiles.modelLabel'),
      value: displayValue(props.profile.model),
      fullValue: displayValue(props.profile.model),
      mono: true,
      type: 'model',
    },
  ]

  if (props.profile.small_fast_model?.trim()) {
    items.push({
      label: t('claudeProfiles.smallFastModelLabel'),
      value: props.profile.small_fast_model,
      fullValue: props.profile.small_fast_model,
      mono: true,
      type: 'model',
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
