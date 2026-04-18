<template>
  <div
    class="space-y-10 animate-slide-up"
    style="animation-delay: 200ms"
  >
    <section
      v-for="(section, index) in providerSections"
      :id="section.id"
      :key="section.id"
      :ref="(element) => registerSectionRef(section.id, element)"
      class="scroll-mt-28"
    >
      <!-- 区块间分隔线 -->
      <div
        v-if="index > 0"
        class="mb-6 flex items-center gap-3"
        aria-hidden="true"
      >
        <div class="h-px flex-1 bg-border-default/25" />
        <span class="text-[11px] font-medium uppercase tracking-[0.2em] text-text-ghost">
          {{ section.title }}
        </span>
        <div class="h-px flex-1 bg-border-default/25" />
      </div>

      <!-- Provider 区块头部 -->
      <div class="mb-3 flex items-center gap-3">
        <!-- provider 色彩竖条 -->
        <div
          class="h-10 w-1 shrink-0 rounded-full"
          :style="{ backgroundColor: `rgb(var(${sectionColors[section.providerKey]?.rgbVar || '--color-accent-secondary-rgb'}))` }"
        />

        <!-- provider 图标 -->
        <div
          class="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl"
          :style="{ backgroundColor: `rgb(var(${sectionColors[section.providerKey]?.rgbVar || '--color-accent-secondary-rgb'}) / 0.1)` }"
        >
          <SIcon
            :name="sectionIcons[section.providerKey] || 'Server'"
            size="w-4.5 h-4.5"
            :style="{ color: `rgb(var(${sectionColors[section.providerKey]?.rgbVar || '--color-accent-secondary-rgb'}))` }"
          />
        </div>

        <!-- 标题和统计 -->
        <div class="min-w-0 flex-1">
          <div class="flex flex-wrap items-center gap-2">
            <h2 class="truncate text-[1.15rem] font-semibold tracking-tight text-text-primary">
              {{ section.title }}
            </h2>
            <span
              class="inline-flex items-center rounded-full px-2.5 py-0.5 text-[11px] font-medium"
              :style="{
                backgroundColor: `rgb(var(${sectionColors[section.providerKey]?.rgbVar || '--color-accent-secondary-rgb'}) / 0.1)`,
                color: `rgb(var(${sectionColors[section.providerKey]?.rgbVar || '--color-accent-secondary-rgb'}))`,
              }"
            >
              {{ getProviderNavCount(section.count) }}
            </span>
            <span class="inline-flex items-center rounded-full border border-border-default/40 bg-bg-elevated/62 px-2.5 py-0.5 text-[11px] text-text-muted">
              {{ getProviderEnabledCount(section.enabledCount) }}
            </span>
            <span
              v-if="section.isCurrentProvider"
              class="inline-flex items-center gap-1.5 rounded-full bg-accent-secondary/8 px-2.5 py-0.5 text-xs text-accent-secondary"
            >
              <span class="h-1.5 w-1.5 rounded-full bg-current animate-pulse" />
              {{ $t('claudeProfiles.currentBadge') }}
            </span>
          </div>
        </div>
      </div>

      <!-- Profile 卡片列表 (不再嵌套外层容器) -->
      <div class="space-y-3">
        <ClaudeProfileRow
          v-for="profile in section.profiles"
          :key="profile.name"
          :profile="profile"
          :provider-color="sectionColors[section.providerKey] || defaultColor"
          :search-query="searchQuery"
          @apply="$emit('apply', profile.name)"
          @edit="$emit('edit', profile)"
          @delete="$emit('delete', profile.name)"
        />
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, type ComponentPublicInstance } from 'vue'
import { useI18n } from 'vue-i18n'
import ClaudeProfileRow from '@/components/claude/ClaudeProfileRow.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { translateWithFallback } from '@/i18n/formatMessage'
import type { ClaudeProfile } from '@/types'
import {
  resolveProviderColor,
  resolveProviderIcon,
  type ClaudeProfileSection,
  type ProviderColorConfig,
} from '@/utils/claudeProfiles'

const props = defineProps<{
  providerSections: ClaudeProfileSection[]
  providerUnsetLabel: string
  registerSectionRef: (sectionId: string, target: Element | ComponentPublicInstance | null) => void
  searchQuery?: string
}>()

defineEmits<{
  apply: [name: string]
  delete: [name: string]
  edit: [profile: ClaudeProfile]
}>()

const { t } = useI18n()

const getProviderNavCount = (count: number) => translateWithFallback(
  t,
  'claudeProfiles.providerNavCount',
  '{count} 个 Profile',
  { count },
)
const getProviderEnabledCount = (count: number) => translateWithFallback(
  t,
  'claudeProfiles.providerEnabledCount',
  '{count} 已启用',
  { count },
)

// 默认色彩配置
const defaultColor: ProviderColorConfig = {
  key: 'default',
  cssVar: '--color-accent-secondary',
  rgbVar: '--color-accent-secondary-rgb',
  tailwindClass: 'accent-secondary',
}

// 缓存各 section 的 provider 色彩
const sectionColors = computed(() => {
  const map: Record<string, ProviderColorConfig> = {}
  for (const section of props.providerSections) {
    const providerName = section.providerKey === '__unset_provider__'
      ? props.providerUnsetLabel
      : section.title
    map[section.providerKey] = resolveProviderColor(providerName)
  }
  return map
})

// 缓存各 section 的 provider 图标
const sectionIcons = computed(() => {
  const map: Record<string, string> = {}
  for (const section of props.providerSections) {
    const providerName = section.providerKey === '__unset_provider__'
      ? props.providerUnsetLabel
      : section.title
    map[section.providerKey] = resolveProviderIcon(providerName)
  }
  return map
})
</script>
