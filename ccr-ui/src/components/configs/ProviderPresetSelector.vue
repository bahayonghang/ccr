<template>
  <section class="mb-8">
    <label class="block text-xs font-bold uppercase tracking-wider text-white/50 mb-4">
      {{ $t('configs.addConfig.presetProviders') }}
    </label>

    <!-- 预设按钮网格 -->
    <div class="flex flex-wrap gap-2">
      <!-- 自定义配置按钮 (固定在最前) -->
      <button
        class="relative px-4 py-2 rounded-xl border text-sm font-medium transition-all duration-200"
        :class="selectedId === null
          ? 'bg-accent-primary/15 border-accent-primary text-accent-primary ring-1 ring-accent-primary/50'
          : 'glass-surface border-white/10 text-white/70 hover:border-accent-primary/30 hover:text-white'"
        @click="handleSelect(null)"
      >
        {{ $t('configs.addConfig.customConfig') }}
      </button>

      <!-- 供应商预设按钮 -->
      <button
        v-for="preset in sortedPresets"
        :key="preset.id"
        class="relative px-4 py-2 rounded-xl border text-sm font-medium transition-all duration-200"
        :class="selectedId === preset.id
          ? 'bg-accent-primary/15 border-accent-primary text-accent-primary ring-1 ring-accent-primary/50'
          : 'glass-surface border-white/10 text-white/70 hover:border-accent-primary/30 hover:text-white'"
        @click="handleSelect(preset)"
      >
        {{ preset.name }}
        <span
          v-if="preset.isPartner"
          class="ml-0.5 text-amber-400 text-[10px]"
          title="Partner"
        >
          ★
        </span>
      </button>
    </div>

    <!-- 底部提示文字 -->
    <p class="mt-3 text-xs text-white/40">
      {{ selectedId === null
        ? $t('configs.addConfig.customConfigHint')
        : $t('configs.addConfig.presetHint')
      }}
    </p>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { PlatformPresets, ProviderPreset, PresetCategory } from '@/types/providerPresets'

const props = defineProps<{
  presets: PlatformPresets
  selectedId: string | null
}>()

const emit = defineEmits<{
  select: [preset: ProviderPreset | null]
}>()

// 按分类排序: official → cn_official → aggregator → third_party
const categoryOrder: Record<PresetCategory, number> = {
  official: 0,
  cn_official: 1,
  aggregator: 2,
  third_party: 3,
}

const sortedPresets = computed(() =>
  [...props.presets.presets].sort((a, b) => categoryOrder[a.category] - categoryOrder[b.category])
)

const handleSelect = (preset: ProviderPreset | null) => {
  emit('select', preset)
}
</script>
