<template>
  <div 
    class="liquid-card p-6 transition-all duration-300 hover:transform hover:translate-y-[-4px] hover:scale-[1.01]"
  >
    <div v-if="$slots.header" class="mb-4">
      <slot name="header" />
    </div>
    <slot />
    <div v-if="$slots.footer" class="mt-4 pt-4 border-t border-opacity-20" :style="{ borderColor: 'var(--border-color)' }">
      <slot name="footer" />
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  padding?: string;
}

const props = withDefaults(defineProps<Props>(), {
  padding: 'p-6',
})
</script>