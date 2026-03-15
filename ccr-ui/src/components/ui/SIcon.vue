<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { computed } from 'vue'
import { iconMap, type IconName } from '@/config/icons'

const props = withDefaults(defineProps<{
  /** 语义名称 (如 'Home') 或原始 iconify ID (如 'solar:xxx') */
  name: IconName | string
  /** Tailwind 尺寸类，如 'w-4 h-4' */
  size?: string
}>(), {
  size: 'w-4 h-4',
})

const iconId = computed(() => {
  return (iconMap as Record<string, string>)[props.name] ?? props.name
})
</script>

<template>
  <Icon
    :icon="iconId"
    :class="size"
  />
</template>
