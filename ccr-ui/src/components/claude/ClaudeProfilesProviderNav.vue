<template>
  <nav
    :class="[
      mobile
        ? 'rounded-2xl border border-border-default/45 bg-bg-surface/68 p-3'
        : 'sticky top-6 rounded-[28px] border border-border-default/45 bg-bg-surface/72 p-4 shadow-xl shadow-black/5 backdrop-blur-xl',
    ]"
    :aria-label="$t('claudeProfiles.providerNavTitle')"
  >
    <div
      v-if="!mobile"
      class="mb-4 space-y-1"
    >
      <p class="flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.24em] text-text-muted">
        <SIcon
          name="PanelLeftOpen"
          size="w-3.5 h-3.5"
        />
        {{ $t('claudeProfiles.providerNavTitle') }}
      </p>
      <p class="text-sm text-text-secondary">
        {{ $t('claudeProfiles.providerNavHint') }}
      </p>
    </div>

    <div
      :class="[
        mobile
          ? 'flex gap-2 overflow-x-auto pb-1'
          : 'flex flex-col gap-2',
      ]"
    >
      <button
        v-for="section in sections"
        :key="section.id"
        type="button"
        :aria-current="activeSectionId === section.id ? 'location' : undefined"
        :class="[
          mobile
            ? 'min-h-[40px] whitespace-nowrap px-3 py-2 text-sm'
            : 'relative min-h-[52px] w-full px-4 py-3 text-left',
          'group rounded-2xl border transition-[background-color,border-color,color,transform] duration-200',
          activeSectionId === section.id
            ? 'border-transparent text-text-primary shadow-[0_10px_30px_rgba(96,70,160,0.12)]'
            : 'border-border-default/50 bg-bg-surface/55 text-text-secondary hover:border-border-default hover:bg-bg-elevated/70 hover:text-text-primary',
        ]"
        :style="activeSectionId === section.id ? {
          backgroundColor: `rgb(var(${sectionColors[section.providerKey]?.rgbVar || '--color-accent-secondary-rgb'}) / 0.08)`,
          borderColor: `rgb(var(${sectionColors[section.providerKey]?.rgbVar || '--color-accent-secondary-rgb'}) / 0.2)`,
        } : {}"
        @click="$emit('navigate', section.id)"
      >
        <!-- 含当前活跃 profile 的指示圆点 -->
        <span
          v-if="!mobile && section.isCurrentProvider"
          class="absolute right-2.5 top-2.5 flex h-2.5 w-2.5 items-center justify-center"
          :aria-label="$t('claudeProfiles.currentProviderBadge')"
        >
          <span class="absolute h-full w-full rounded-full bg-accent-secondary/40 animate-ping" />
          <span class="relative h-2 w-2 rounded-full bg-accent-secondary" />
        </span>

        <div class="flex items-center justify-between gap-3">
          <div class="flex min-w-0 items-center gap-2.5">
            <!-- Provider 色彩圆点 -->
            <span
              class="h-2 w-2 shrink-0 rounded-full"
              :style="{ backgroundColor: `rgb(var(${sectionColors[section.providerKey]?.rgbVar || '--color-accent-secondary-rgb'}))` }"
            />
            <div class="min-w-0">
              <p
                :class="[
                  'truncate font-medium',
                  activeSectionId === section.id ? 'text-text-primary' : '',
                ]"
              >
                {{ section.title }}
              </p>
              <p
                v-if="!mobile"
                class="mt-1 text-xs text-text-muted"
              >
                {{ getProviderNavCount(section.count) }}
              </p>
            </div>
          </div>
          <div class="flex shrink-0 items-center gap-2">
            <span
              class="rounded-full px-2 py-1 text-xs font-medium"
              :style="activeSectionId === section.id ? {
                backgroundColor: `rgb(var(${sectionColors[section.providerKey]?.rgbVar || '--color-accent-secondary-rgb'}) / 0.12)`,
                color: `rgb(var(${sectionColors[section.providerKey]?.rgbVar || '--color-accent-secondary-rgb'}))`,
              } : {}"
              :class="activeSectionId !== section.id ? 'bg-bg-elevated text-text-muted' : ''"
            >
              {{ section.count }}
            </span>
          </div>
        </div>
      </button>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import {
  resolveProviderColor,
  type ClaudeProfileSection,
  type ProviderColorConfig,
} from '@/utils/claudeProfiles'

const props = defineProps<{
  sections: ClaudeProfileSection[]
  activeSectionId: string | null
  mobile?: boolean
}>()

defineEmits<{
  navigate: [sectionId: string]
}>()

const { t } = useI18n()

const getProviderNavCount = (count: number) => t('claudeProfiles.providerNavCount', { count })

// 缓存各 section 的 provider 色彩
const sectionColors = computed(() => {
  const map: Record<string, ProviderColorConfig> = {}
  for (const section of props.sections) {
    map[section.providerKey] = resolveProviderColor(section.title)
  }
  return map
})
</script>
