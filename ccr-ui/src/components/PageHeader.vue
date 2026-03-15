<template>
  <div class="flex items-center justify-between mb-6">
    <div class="flex items-center gap-4">
      <h2
        class="text-2xl font-bold"
        :style="{ color: 'var(--text-primary)' }"
      >
        <SIcon
          name="Command"
          size="w-7 h-7"
          class="inline-block mr-2"
        />
        {{ title }}
      </h2>
      <span
        class="px-3 py-1 rounded-full text-sm font-medium"
        :style="{
          background: 'var(--accent-primary)',
          color: '#fff'
        }"
      >
        {{ count }}/{{ total }}
      </span>
    </div>

    <div class="flex items-center gap-3">
      <RouterLink
        :to="homePath"
        class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors hover:opacity-80"
        :style="{
          background: 'var(--bg-secondary)',
          color: 'var(--text-secondary)',
          border: '1px solid var(--border-color)'
        }"
      >
        <SIcon
          name="Home"
          size="w-4 h-4"
        />
        <span>{{ $t('common.backToHome') }}</span>
      </RouterLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
// Props
interface Props {
  title: string
  subtitle?: string
  count: number
  total: number
  homePath: string
}

defineProps<Props>()
</script>
