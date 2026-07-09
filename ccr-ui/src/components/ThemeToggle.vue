<template>
  <button
    type="button"
    class="inline-flex min-h-[44px] min-w-[44px] flex-shrink-0 items-center justify-center rounded-full border p-0 leading-none text-text-secondary shadow-sm transition-interactive duration-200 hover:text-accent-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30 focus-visible:ring-offset-2 focus-visible:ring-offset-bg-base active:scale-95"
    :title="`切换到${currentTheme === 'dark' ? '明亮' : '深色'}模式`"
    :aria-label="`切换到${currentTheme === 'dark' ? '明亮' : '深色'}模式`"
    :style="{
      background: 'var(--surface-status-bg)',
      borderColor: 'var(--surface-status-border)',
      backdropFilter: 'var(--surface-status-blur)',
      boxShadow: 'var(--surface-status-shadow), inset 0 1px 0 rgb(255 251 245 / 12%)',
    }"
    @click.stop="toggleTheme"
  >
    <SIcon
      v-if="currentTheme === 'dark'"
      name="Moon"
      size="w-4 h-4"
      class="pointer-events-none block"
    />
    <SIcon
      v-else
      name="Sun"
      size="w-4 h-4"
      class="pointer-events-none block"
    />
  </button>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { computed } from 'vue'
import { useShellPreferencesStore } from '@/stores/shellPreferences'

const shellPreferencesStore = useShellPreferencesStore()

const currentTheme = computed(() => shellPreferencesStore.effectiveTheme)

const toggleTheme = () => {
  shellPreferencesStore.toggleTheme()
}
</script>
